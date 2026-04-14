//! # TaskLedger — Persistent record of all delegations.
//!
//! Every time a task is delegated via ProcessBridge, an entry is appended to
//! `data/tasks/ledger.jsonl`. When the result comes back, the entry is updated.
//!
//! This answers: "What has been delegated? To whom? What happened?"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Status of a ledger entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Delegated,
    Completed,
    Failed,
}

/// A single task delegation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    pub task_id: String,
    pub mind_id: String,
    pub parent_mind_id: String,
    pub description: String,
    pub status: TaskStatus,
    pub delegated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_summary: Option<String>,
}

/// Append-only JSONL task ledger.
#[derive(Clone)]
pub struct TaskLedger {
    path: PathBuf,
}

impl TaskLedger {
    /// Open (or create) a ledger at `data/tasks/ledger.jsonl` under the given root.
    pub fn open(project_root: &Path) -> Self {
        let dir = project_root.join("data").join("tasks");
        let _ = std::fs::create_dir_all(&dir);
        Self {
            path: dir.join("ledger.jsonl"),
        }
    }

    /// Record a new delegation.
    pub fn record_delegation(
        &self,
        task_id: &str,
        mind_id: &str,
        parent_mind_id: &str,
        description: &str,
    ) {
        let entry = TaskEntry {
            task_id: task_id.to_string(),
            mind_id: mind_id.to_string(),
            parent_mind_id: parent_mind_id.to_string(),
            description: description.to_string(),
            status: TaskStatus::Delegated,
            delegated_at: Utc::now(),
            completed_at: None,
            iterations: None,
            tool_calls: None,
            response_summary: None,
        };
        self.append(&entry);
        info!(task_id, mind_id, "Task delegation recorded");
    }

    /// Record a task completion (or failure).
    pub fn record_completion(
        &self,
        task_id: &str,
        mind_id: &str,
        parent_mind_id: &str,
        description: &str,
        succeeded: bool,
        iterations: Option<u32>,
        tool_calls: Option<u32>,
        response_summary: Option<&str>,
    ) {
        let entry = TaskEntry {
            task_id: task_id.to_string(),
            mind_id: mind_id.to_string(),
            parent_mind_id: parent_mind_id.to_string(),
            description: description.to_string(),
            status: if succeeded {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            },
            delegated_at: Utc::now(), // approximation; true timestamp is in the delegation record
            completed_at: Some(Utc::now()),
            iterations,
            tool_calls,
            response_summary: response_summary.map(|s| truncate(s, 500).to_string()),
        };
        self.append(&entry);
        info!(task_id, mind_id, succeeded, "Task completion recorded");
    }

    /// Read all entries from the ledger.
    pub fn read_all(&self) -> Vec<TaskEntry> {
        let Ok(content) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Get entries for a specific task_id (delegation + completion).
    pub fn entries_for_task(&self, task_id: &str) -> Vec<TaskEntry> {
        self.read_all()
            .into_iter()
            .filter(|e| e.task_id == task_id)
            .collect()
    }

    /// Summary: count of delegated, completed, failed tasks.
    pub fn summary(&self) -> (usize, usize, usize) {
        let entries = self.read_all();
        let delegated = entries.iter().filter(|e| e.status == TaskStatus::Delegated).count();
        let completed = entries.iter().filter(|e| e.status == TaskStatus::Completed).count();
        let failed = entries.iter().filter(|e| e.status == TaskStatus::Failed).count();
        (delegated, completed, failed)
    }

    fn append(&self, entry: &TaskEntry) {
        use std::io::Write;
        if let Ok(json) = serde_json::to_string(entry) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                let _ = writeln!(file, "{json}");
            }
        }
    }
}

/// Truncate a string to at most `max` bytes on a char boundary.
fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn record_delegation_creates_file() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "researcher", "primary", "Research Codex");

        let entries = ledger.read_all();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task_id, "t-001");
        assert_eq!(entries[0].status, TaskStatus::Delegated);
    }

    #[test]
    fn record_completion_appends() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "researcher", "primary", "Research Codex");
        ledger.record_completion(
            "t-001",
            "researcher",
            "primary",
            "Research Codex",
            true,
            Some(4),
            Some(3),
            Some("Found 12 crates"),
        );

        let entries = ledger.read_all();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].status, TaskStatus::Completed);
        assert_eq!(entries[1].iterations, Some(4));
    }

    #[test]
    fn entries_for_task_filters() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "researcher", "primary", "Research");
        ledger.record_delegation("t-002", "coder", "primary", "Build");
        ledger.record_completion("t-001", "researcher", "primary", "Research", true, None, None, None);

        let task_entries = ledger.entries_for_task("t-001");
        assert_eq!(task_entries.len(), 2);

        let task_entries = ledger.entries_for_task("t-002");
        assert_eq!(task_entries.len(), 1);
    }

    #[test]
    fn summary_counts() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        ledger.record_delegation("t-001", "a", "p", "Task 1");
        ledger.record_delegation("t-002", "b", "p", "Task 2");
        ledger.record_completion("t-001", "a", "p", "Task 1", true, None, None, None);
        ledger.record_completion("t-002", "b", "p", "Task 2", false, None, None, None);

        let (delegated, completed, failed) = ledger.summary();
        assert_eq!(delegated, 2);
        assert_eq!(completed, 1);
        assert_eq!(failed, 1);
    }

    #[test]
    fn empty_ledger_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let ledger = TaskLedger::open(tmp.path());
        assert!(ledger.read_all().is_empty());
        assert_eq!(ledger.summary(), (0, 0, 0));
    }

    #[test]
    fn truncate_respects_char_boundary() {
        let s = "Hello, 世界!";
        let t = truncate(s, 9);
        assert!(t.len() <= 9);
        assert!(t.is_char_boundary(t.len()));
    }
}
