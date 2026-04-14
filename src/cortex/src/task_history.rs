//! # TaskHistoryInterceptor — Query delegation history from within ThinkLoop.
//!
//! Exposes a `task_history` tool that reads the TaskLedger during reasoning,
//! so any mind can answer "what have we done?"

use async_trait::async_trait;
use codex_coordination::TaskLedger;
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use std::sync::{Arc, Mutex};

/// Interceptor that exposes task delegation history to the ThinkLoop.
pub struct TaskHistoryInterceptor {
    ledger: Arc<Mutex<TaskLedger>>,
}

impl TaskHistoryInterceptor {
    pub fn new(ledger: TaskLedger) -> Self {
        Self {
            ledger: Arc::new(Mutex::new(ledger)),
        }
    }
}

#[async_trait]
impl ToolInterceptor for TaskHistoryInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "task_history".into(),
                description: "Query the delegation history. Shows all tasks that have been delegated, their status (delegated/completed/failed), which mind handled them, and results. Use to answer 'what have we done?' or check prior work.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "Optional: filter by specific task_id. Omit to see all tasks."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max entries to return. Default: 20."
                        }
                    }
                }),
            },
        }]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        if name != "task_history" {
            return None;
        }

        let ledger = match self.ledger.lock() {
            Ok(l) => l,
            Err(_) => return Some(ToolResult::err("TaskLedger lock poisoned")),
        };

        // Filter by task_id if provided
        if let Some(task_id) = args.get("task_id").and_then(|v| v.as_str()) {
            let entries = ledger.entries_for_task(task_id);
            if entries.is_empty() {
                return Some(ToolResult::ok(format!("No entries found for task_id: {task_id}")));
            }
            let formatted = entries.iter()
                .map(format_entry)
                .collect::<Vec<_>>()
                .join("\n---\n");
            return Some(ToolResult::ok(formatted));
        }

        // Otherwise return all (with limit)
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let all = ledger.read_all();
        let (delegated, completed, failed) = ledger.summary();

        if all.is_empty() {
            return Some(ToolResult::ok("No delegation history yet."));
        }

        let recent: Vec<_> = all.iter().rev().take(limit).collect();
        let formatted = recent.iter()
            .map(|e| format_entry(e))
            .collect::<Vec<_>>()
            .join("\n---\n");

        Some(ToolResult::ok(format!(
            "Task History (showing {}/{} entries)\nSummary: {} delegated, {} completed, {} failed\n\n{}",
            recent.len(), all.len(), delegated, completed, failed, formatted
        )))
    }
}

fn format_entry(entry: &codex_coordination::task_ledger::TaskEntry) -> String {
    let status = match entry.status {
        codex_coordination::task_ledger::TaskStatus::Delegated => "DELEGATED",
        codex_coordination::task_ledger::TaskStatus::Completed => "COMPLETED",
        codex_coordination::task_ledger::TaskStatus::Failed => "FAILED",
    };
    let mut parts = vec![
        format!("[{}] task:{} → mind:{}", status, entry.task_id, entry.mind_id),
        format!("  description: {}", entry.description),
        format!("  delegated_at: {}", entry.delegated_at.format("%H:%M:%S UTC")),
    ];
    if let Some(at) = entry.completed_at {
        parts.push(format!("  completed_at: {}", at.format("%H:%M:%S UTC")));
    }
    if let Some(iter) = entry.iterations {
        parts.push(format!("  iterations: {}", iter));
    }
    if let Some(tc) = entry.tool_calls {
        parts.push(format!("  tool_calls: {}", tc));
    }
    if let Some(ref summary) = entry.response_summary {
        let short = if summary.len() > 200 {
            // Find a char boundary at or before 200 bytes
            let mut end = 200;
            while end > 0 && !summary.is_char_boundary(end) { end -= 1; }
            &summary[..end]
        } else {
            summary
        };
        parts.push(format!("  response: {}", short));
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn task_history_empty() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        let interceptor = TaskHistoryInterceptor::new(ledger);

        let result = interceptor.handle("task_history", &serde_json::json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("No delegation history"));
    }

    #[tokio::test]
    async fn task_history_with_entries() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "researcher", "primary", "Research Codex");
        ledger.record_completion("t-001", "researcher", "primary", "Research Codex", true, Some(4), Some(3), Some("Found 12 crates"));

        let interceptor = TaskHistoryInterceptor::new(ledger);
        let result = interceptor.handle("task_history", &serde_json::json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("DELEGATED"));
        assert!(result.output.contains("COMPLETED"));
        assert!(result.output.contains("Research Codex"));
    }

    #[tokio::test]
    async fn task_history_filter_by_id() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "a", "p", "Task A");
        ledger.record_delegation("t-002", "b", "p", "Task B");

        let interceptor = TaskHistoryInterceptor::new(ledger);
        let result = interceptor.handle("task_history", &serde_json::json!({"task_id": "t-001"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Task A"));
        assert!(!result.output.contains("Task B"));
    }

    #[tokio::test]
    async fn passthrough_other_tools() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        let interceptor = TaskHistoryInterceptor::new(ledger);

        let result = interceptor.handle("other_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }
}
