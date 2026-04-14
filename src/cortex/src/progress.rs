//! # ProgressInterceptor — Mid-Task Progress Reporting
//!
//! Exposes `report_progress` and `check_progress` tools so minds can
//! report intermediate state during long tasks, and parents can check
//! without blocking until completion.
//!
//! Progress entries are written to `data/progress/{mind_id}.jsonl` —
//! one file per mind, append-only, JSONL format.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single progress entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub mind_id: String,
    pub task_id: String,
    pub iteration: u32,
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Interceptor that exposes progress reporting/checking to the ThinkLoop.
pub struct ProgressInterceptor {
    /// Directory where progress files live: `data/progress/`
    progress_dir: PathBuf,
    /// This mind's ID (for report_progress).
    mind_id: String,
}

impl ProgressInterceptor {
    pub fn new(progress_dir: &Path, mind_id: &str) -> Self {
        let _ = std::fs::create_dir_all(progress_dir);
        Self {
            progress_dir: progress_dir.to_path_buf(),
            mind_id: mind_id.to_string(),
        }
    }

    fn progress_file(&self, mind_id: &str) -> PathBuf {
        self.progress_dir.join(format!("{mind_id}.jsonl"))
    }

    fn append_entry(&self, entry: &ProgressEntry) {
        use std::io::Write;
        if let Ok(json) = serde_json::to_string(entry) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.progress_file(&entry.mind_id))
            {
                let _ = writeln!(file, "{json}");
            }
        }
    }

    fn read_entries(&self, mind_id: &str) -> Vec<ProgressEntry> {
        let path = self.progress_file(mind_id);
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }
}

#[async_trait]
impl ToolInterceptor for ProgressInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "report_progress".into(),
                    description: "Report intermediate progress on the current task. Use this during long tasks to let the parent mind know what you're doing. Include what you've done so far and what's next.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "task_id": {
                                "type": "string",
                                "description": "The task being worked on."
                            },
                            "status": {
                                "type": "string",
                                "enum": ["starting", "in_progress", "blocked", "nearly_done"],
                                "description": "Current status."
                            },
                            "message": {
                                "type": "string",
                                "description": "What you've done so far and what's next."
                            },
                            "iteration": {
                                "type": "integer",
                                "description": "Current iteration number."
                            }
                        },
                        "required": ["task_id", "status", "message"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "check_progress".into(),
                    description: "Check the progress of a child mind. Returns the latest progress reports from the specified mind. Use to monitor delegated tasks without blocking.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "mind_id": {
                                "type": "string",
                                "description": "The mind to check progress for."
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Max entries to return (default: 5, most recent first)."
                            }
                        },
                        "required": ["mind_id"]
                    }),
                },
            },
        ]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        match name {
            "report_progress" => {
                let task_id = match args.get("task_id").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => return Some(ToolResult::err("Missing required parameter: task_id")),
                };
                let status = match args.get("status").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => return Some(ToolResult::err("Missing required parameter: status")),
                };
                let message = match args.get("message").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => return Some(ToolResult::err("Missing required parameter: message")),
                };
                let iteration = args.get("iteration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let entry = ProgressEntry {
                    mind_id: self.mind_id.clone(),
                    task_id: task_id.to_string(),
                    iteration,
                    status: status.to_string(),
                    message: message.to_string(),
                    timestamp: Utc::now(),
                };

                self.append_entry(&entry);
                Some(ToolResult::ok(format!(
                    "Progress reported: [{}] {} — {}",
                    status, task_id, message
                )))
            }
            "check_progress" => {
                let mind_id = match args.get("mind_id").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => return Some(ToolResult::err("Missing required parameter: mind_id")),
                };
                let limit = args.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;

                let entries = self.read_entries(mind_id);
                if entries.is_empty() {
                    return Some(ToolResult::ok(format!(
                        "No progress reports from mind: {mind_id}"
                    )));
                }

                let recent: Vec<_> = entries.iter().rev().take(limit).collect();
                let formatted = recent.iter()
                    .map(|e| format!(
                        "[{}] {} task:{} iter:{} — {}",
                        e.timestamp.format("%H:%M:%S"),
                        e.status,
                        e.task_id,
                        e.iteration,
                        e.message,
                    ))
                    .collect::<Vec<_>>()
                    .join("\n");

                Some(ToolResult::ok(format!(
                    "Progress for {} (showing {}/{} entries):\n{}",
                    mind_id, recent.len(), entries.len(), formatted
                )))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn report_and_check_progress() {
        let tmp = TempDir::new().unwrap();
        let interceptor = ProgressInterceptor::new(tmp.path(), "researcher");

        // Report progress
        let result = interceptor.handle("report_progress", &serde_json::json!({
            "task_id": "t-001",
            "status": "in_progress",
            "message": "Found 5 relevant files, analyzing now",
            "iteration": 3
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("in_progress"));

        // Check progress
        let result = interceptor.handle("check_progress", &serde_json::json!({
            "mind_id": "researcher"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Found 5 relevant files"));
        assert!(result.output.contains("in_progress"));
    }

    #[tokio::test]
    async fn check_no_progress() {
        let tmp = TempDir::new().unwrap();
        let interceptor = ProgressInterceptor::new(tmp.path(), "primary");

        let result = interceptor.handle("check_progress", &serde_json::json!({
            "mind_id": "nonexistent"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("No progress reports"));
    }

    #[tokio::test]
    async fn multiple_reports_ordered() {
        let tmp = TempDir::new().unwrap();
        let interceptor = ProgressInterceptor::new(tmp.path(), "coder");

        // Report 3 progress updates
        for (i, msg) in ["Starting analysis", "Found the bug", "Fix applied"].iter().enumerate() {
            interceptor.handle("report_progress", &serde_json::json!({
                "task_id": "t-002",
                "status": if i == 2 { "nearly_done" } else { "in_progress" },
                "message": msg,
                "iteration": i + 1
            })).await.unwrap();
        }

        // Check with limit=2
        let result = interceptor.handle("check_progress", &serde_json::json!({
            "mind_id": "coder",
            "limit": 2
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("showing 2/3"));
        assert!(result.output.contains("Fix applied"));
        assert!(result.output.contains("Found the bug"));
        // Oldest entry should NOT be in limited output
        assert!(!result.output.contains("Starting analysis"));
    }

    #[tokio::test]
    async fn missing_required_params() {
        let tmp = TempDir::new().unwrap();
        let interceptor = ProgressInterceptor::new(tmp.path(), "test");

        // Missing task_id
        let result = interceptor.handle("report_progress", &serde_json::json!({
            "status": "in_progress",
            "message": "test"
        })).await.unwrap();
        assert!(!result.success);

        // Missing mind_id for check
        let result = interceptor.handle("check_progress", &serde_json::json!({})).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn passthrough_other_tools() {
        let tmp = TempDir::new().unwrap();
        let interceptor = ProgressInterceptor::new(tmp.path(), "test");

        let result = interceptor.handle("other_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }
}
