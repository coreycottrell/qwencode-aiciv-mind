//! # TaskStore — SQLite-Backed Task Queue
//!
//! "The moment you need 'show me all tasks that stalled in the last 3 sessions'
//! — and you will — you're writing a query parser over JSONL."
//! — Corey, 2026-04-04
//!
//! Replaces the append-only JSONL TaskLedger with a structured, queryable store.
//! SQLite is already a workspace dependency (codex-memory uses it).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use thiserror::Error;
use tracing::info;

/// Task lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Ready to be picked up.
    Open,
    /// Currently being worked on by a mind.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Failed (with reason in metadata).
    Failed,
    /// Blocked on a dependency.
    Blocked,
}

impl TaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Open => "open",
            TaskState::InProgress => "in_progress",
            TaskState::Completed => "completed",
            TaskState::Failed => "failed",
            TaskState::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "open" => Some(TaskState::Open),
            "in_progress" => Some(TaskState::InProgress),
            "completed" => Some(TaskState::Completed),
            "failed" => Some(TaskState::Failed),
            "blocked" => Some(TaskState::Blocked),
            _ => None,
        }
    }
}

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl TaskPriority {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => TaskPriority::Low,
            1 => TaskPriority::Normal,
            2 => TaskPriority::High,
            _ => TaskPriority::Critical,
        }
    }
}

/// A task stored in the SQLite database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    pub task_id: String,
    pub description: String,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub assigned_mind: Option<String>,
    pub parent_mind: Option<String>,
    /// Comma-separated task IDs this task depends on.
    pub depends_on: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Iteration count and tool call count from execution.
    pub iterations: Option<i32>,
    pub tool_calls: Option<i32>,
    /// Summary of result or failure reason.
    pub result_summary: Option<String>,
}

impl StoredTask {
    /// Create a new task in Open state with the given priority.
    pub fn new(
        task_id: &str,
        description: &str,
        priority: TaskPriority,
        parent_mind: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            task_id: task_id.into(),
            description: description.into(),
            state: TaskState::Open,
            priority,
            assigned_mind: None,
            parent_mind: parent_mind.map(|s| s.into()),
            depends_on: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            iterations: None,
            tool_calls: None,
            result_summary: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum TaskStoreError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Task not found: {0}")]
    NotFound(String),
}

/// SQLite-backed task store.
///
/// Schema is created on first open. All operations are async.
#[derive(Clone)]
pub struct TaskStore {
    pool: SqlitePool,
}

impl TaskStore {
    /// Open (or create) a task store at the given path.
    pub async fn open(db_path: &Path) -> Result<Self, TaskStoreError> {
        let url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await?;

        // Create tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'open',
                priority INTEGER NOT NULL DEFAULT 1,
                assigned_mind TEXT,
                parent_mind TEXT,
                depends_on TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT,
                iterations INTEGER,
                tool_calls INTEGER,
                result_summary TEXT
            )"
        )
        .execute(&pool)
        .await?;

        // Index for common queries
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_tasks_state ON tasks(state)"
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority DESC)"
        )
        .execute(&pool)
        .await?;

        info!(path = %db_path.display(), "TaskStore opened");
        Ok(Self { pool })
    }

    /// Open an in-memory task store (for testing).
    pub async fn open_memory() -> Result<Self, TaskStoreError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'open',
                priority INTEGER NOT NULL DEFAULT 1,
                assigned_mind TEXT,
                parent_mind TEXT,
                depends_on TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT,
                iterations INTEGER,
                tool_calls INTEGER,
                result_summary TEXT
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Insert a new task.
    pub async fn insert(&self, task: &StoredTask) -> Result<(), TaskStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO tasks (task_id, description, state, priority, assigned_mind, parent_mind,
             depends_on, created_at, updated_at, completed_at, iterations, tool_calls, result_summary)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&task.task_id)
        .bind(&task.description)
        .bind(task.state.as_str())
        .bind(task.priority.as_i32())
        .bind(&task.assigned_mind)
        .bind(&task.parent_mind)
        .bind(&task.depends_on)
        .bind(task.created_at.to_rfc3339())
        .bind(&now)
        .bind(task.completed_at.map(|t| t.to_rfc3339()))
        .bind(task.iterations)
        .bind(task.tool_calls)
        .bind(&task.result_summary)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update task state.
    pub async fn set_state(
        &self,
        task_id: &str,
        state: TaskState,
    ) -> Result<(), TaskStoreError> {
        let now = Utc::now().to_rfc3339();
        let completed = if state == TaskState::Completed || state == TaskState::Failed {
            Some(now.clone())
        } else {
            None
        };

        let rows = sqlx::query(
            "UPDATE tasks SET state = ?, updated_at = ?, completed_at = COALESCE(?, completed_at) WHERE task_id = ?"
        )
        .bind(state.as_str())
        .bind(&now)
        .bind(completed)
        .bind(task_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(TaskStoreError::NotFound(task_id.into()));
        }
        Ok(())
    }

    /// Assign a task to a mind and mark it InProgress.
    pub async fn assign(
        &self,
        task_id: &str,
        mind_id: &str,
    ) -> Result<(), TaskStoreError> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "UPDATE tasks SET state = 'in_progress', assigned_mind = ?, updated_at = ? WHERE task_id = ?"
        )
        .bind(mind_id)
        .bind(&now)
        .bind(task_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(TaskStoreError::NotFound(task_id.into()));
        }
        Ok(())
    }

    /// Complete a task with results.
    pub async fn complete(
        &self,
        task_id: &str,
        iterations: Option<i32>,
        tool_calls: Option<i32>,
        summary: Option<&str>,
    ) -> Result<(), TaskStoreError> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "UPDATE tasks SET state = 'completed', updated_at = ?, completed_at = ?,
             iterations = ?, tool_calls = ?, result_summary = ? WHERE task_id = ?"
        )
        .bind(&now)
        .bind(&now)
        .bind(iterations)
        .bind(tool_calls)
        .bind(summary)
        .bind(task_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(TaskStoreError::NotFound(task_id.into()));
        }

        // Unblock any tasks that depend on this one
        self.unblock_dependents(task_id).await?;

        Ok(())
    }

    /// Get the next ready task (Open, highest priority, oldest first).
    pub async fn next_ready(&self) -> Result<Option<StoredTask>, TaskStoreError> {
        let row = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE state = 'open'
             ORDER BY priority DESC, created_at ASC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_stored_task()))
    }

    /// Get all tasks in a given state.
    pub async fn by_state(&self, state: TaskState) -> Result<Vec<StoredTask>, TaskStoreError> {
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE state = ? ORDER BY priority DESC, created_at ASC"
        )
        .bind(state.as_str())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_stored_task()).collect())
    }

    /// Find tasks that have been InProgress longer than the given threshold.
    pub async fn stalled_tasks(&self, stall_seconds: i64) -> Result<Vec<StoredTask>, TaskStoreError> {
        let cutoff = (Utc::now() - chrono::Duration::seconds(stall_seconds)).to_rfc3339();
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE state = 'in_progress' AND updated_at < ?
             ORDER BY priority DESC"
        )
        .bind(&cutoff)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_stored_task()).collect())
    }

    /// Block all InProgress tasks that have stalled (for boot recovery).
    pub async fn block_stale_in_progress(&self, stall_seconds: i64) -> Result<u64, TaskStoreError> {
        let cutoff = (Utc::now() - chrono::Duration::seconds(stall_seconds)).to_rfc3339();
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE tasks SET state = 'open', assigned_mind = NULL, updated_at = ?
             WHERE state = 'in_progress' AND updated_at < ?"
        )
        .bind(&now)
        .bind(&cutoff)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected();
        if count > 0 {
            info!(count, "Recovered stale in-progress tasks to open state");
        }
        Ok(count)
    }

    /// Get a single task by ID.
    pub async fn get(&self, task_id: &str) -> Result<Option<StoredTask>, TaskStoreError> {
        let row = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE task_id = ?"
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_stored_task()))
    }

    /// Summary counts.
    pub async fn summary(&self) -> Result<TaskSummary, TaskStoreError> {
        let row = sqlx::query_as::<_, SummaryRow>(
            "SELECT
                COUNT(CASE WHEN state = 'open' THEN 1 END) as open_count,
                COUNT(CASE WHEN state = 'in_progress' THEN 1 END) as in_progress_count,
                COUNT(CASE WHEN state = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN state = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN state = 'blocked' THEN 1 END) as blocked_count
             FROM tasks"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskSummary {
            open: row.open_count as u32,
            in_progress: row.in_progress_count as u32,
            completed: row.completed_count as u32,
            failed: row.failed_count as u32,
            blocked: row.blocked_count as u32,
        })
    }

    /// Unblock tasks whose dependencies are all completed.
    async fn unblock_dependents(&self, completed_task_id: &str) -> Result<(), TaskStoreError> {
        // Find blocked tasks that depend on the completed task
        let blocked = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE state = 'blocked' AND depends_on LIKE ?"
        )
        .bind(format!("%{completed_task_id}%"))
        .fetch_all(&self.pool)
        .await?;

        for row in blocked {
            let task = row.into_stored_task();
            if let Some(ref deps) = task.depends_on {
                let all_done = self.all_dependencies_completed(deps).await?;
                if all_done {
                    self.set_state(&task.task_id, TaskState::Open).await?;
                    info!(
                        task_id = %task.task_id,
                        "Unblocked task — all dependencies completed"
                    );
                }
            }
        }

        Ok(())
    }

    /// Check if all comma-separated dependency task IDs are completed.
    async fn all_dependencies_completed(&self, deps: &str) -> Result<bool, TaskStoreError> {
        for dep_id in deps.split(',').map(str::trim) {
            if dep_id.is_empty() { continue; }
            if let Some(task) = self.get(dep_id).await? {
                if task.state != TaskState::Completed {
                    return Ok(false);
                }
            } else {
                // Dependency doesn't exist — treat as not completed
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Summary of task counts by state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub open: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub failed: u32,
    pub blocked: u32,
}

// ── SQLx row types ──────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct TaskRow {
    task_id: String,
    description: String,
    state: String,
    priority: i32,
    assigned_mind: Option<String>,
    parent_mind: Option<String>,
    depends_on: Option<String>,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
    iterations: Option<i32>,
    tool_calls: Option<i32>,
    result_summary: Option<String>,
}

impl TaskRow {
    fn into_stored_task(self) -> StoredTask {
        StoredTask {
            task_id: self.task_id,
            description: self.description,
            state: TaskState::from_str(&self.state).unwrap_or(TaskState::Open),
            priority: TaskPriority::from_i32(self.priority),
            assigned_mind: self.assigned_mind,
            parent_mind: self.parent_mind,
            depends_on: self.depends_on,
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&self.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            completed_at: self.completed_at.and_then(|s|
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            ),
            iterations: self.iterations,
            tool_calls: self.tool_calls,
            result_summary: self.result_summary,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    open_count: i64,
    in_progress_count: i64,
    completed_count: i64,
    failed_count: i64,
    blocked_count: i64,
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, desc: &str, priority: TaskPriority) -> StoredTask {
        let now = Utc::now();
        StoredTask {
            task_id: id.into(),
            description: desc.into(),
            state: TaskState::Open,
            priority,
            assigned_mind: None,
            parent_mind: Some("primary".into()),
            depends_on: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            iterations: None,
            tool_calls: None,
            result_summary: None,
        }
    }

    #[tokio::test]
    async fn insert_and_get() {
        let store = TaskStore::open_memory().await.unwrap();
        let task = make_task("t-001", "Research Codex", TaskPriority::Normal);
        store.insert(&task).await.unwrap();

        let fetched = store.get("t-001").await.unwrap().unwrap();
        assert_eq!(fetched.task_id, "t-001");
        assert_eq!(fetched.state, TaskState::Open);
    }

    #[tokio::test]
    async fn next_ready_returns_highest_priority() {
        let store = TaskStore::open_memory().await.unwrap();
        store.insert(&make_task("t-low", "Low", TaskPriority::Low)).await.unwrap();
        store.insert(&make_task("t-high", "High", TaskPriority::High)).await.unwrap();
        store.insert(&make_task("t-normal", "Normal", TaskPriority::Normal)).await.unwrap();

        let next = store.next_ready().await.unwrap().unwrap();
        assert_eq!(next.task_id, "t-high");
    }

    #[tokio::test]
    async fn assign_and_complete() {
        let store = TaskStore::open_memory().await.unwrap();
        store.insert(&make_task("t-001", "Build X", TaskPriority::Normal)).await.unwrap();

        store.assign("t-001", "code-lead").await.unwrap();
        let task = store.get("t-001").await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::InProgress);
        assert_eq!(task.assigned_mind.as_deref(), Some("code-lead"));

        store.complete("t-001", Some(5), Some(12), Some("Built X successfully")).await.unwrap();
        let task = store.get("t-001").await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::Completed);
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn dependency_unblocking() {
        let store = TaskStore::open_memory().await.unwrap();

        // Create prerequisite task
        store.insert(&make_task("t-prereq", "Setup", TaskPriority::High)).await.unwrap();

        // Create blocked task
        let mut blocked = make_task("t-blocked", "Build (needs setup)", TaskPriority::Normal);
        blocked.state = TaskState::Blocked;
        blocked.depends_on = Some("t-prereq".into());
        store.insert(&blocked).await.unwrap();

        // Verify it's blocked
        let task = store.get("t-blocked").await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::Blocked);

        // Complete the prerequisite
        store.assign("t-prereq", "infra-lead").await.unwrap();
        store.complete("t-prereq", None, None, Some("Done")).await.unwrap();

        // Blocked task should now be Open
        let task = store.get("t-blocked").await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::Open);
    }

    #[tokio::test]
    async fn summary_counts() {
        let store = TaskStore::open_memory().await.unwrap();
        store.insert(&make_task("t-1", "A", TaskPriority::Normal)).await.unwrap();
        store.insert(&make_task("t-2", "B", TaskPriority::Normal)).await.unwrap();
        store.assign("t-1", "lead-1").await.unwrap();
        store.complete("t-1", None, None, None).await.unwrap();

        let summary = store.summary().await.unwrap();
        assert_eq!(summary.open, 1);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.in_progress, 0);
    }

    /// M04 — Parallel agent spawn: 3 tasks assigned to 3 different minds,
    /// all InProgress simultaneously, completing independently in any order.
    /// Validates TaskStore can track concurrent work without interference.
    #[tokio::test]
    async fn parallel_three_agent_spawn() {
        let store = TaskStore::open_memory().await.unwrap();

        // Seed 3 tasks — all Open initially
        store.insert(&make_task("research-files", "Count files in src/", TaskPriority::Normal)).await.unwrap();
        store.insert(&make_task("research-deps", "Read Cargo.toml, summarize deps", TaskPriority::Normal)).await.unwrap();
        store.insert(&make_task("research-tests", "Search for all test functions", TaskPriority::Normal)).await.unwrap();

        // All 3 should be open
        let open = store.by_state(TaskState::Open).await.unwrap();
        assert_eq!(open.len(), 3, "All 3 tasks should be open");

        // Assign all 3 to different minds simultaneously
        store.assign("research-files", "researcher-A").await.unwrap();
        store.assign("research-deps", "researcher-B").await.unwrap();
        store.assign("research-tests", "researcher-C").await.unwrap();

        // All 3 should be InProgress simultaneously
        let in_progress = store.by_state(TaskState::InProgress).await.unwrap();
        assert_eq!(in_progress.len(), 3, "All 3 tasks should be in_progress concurrently");

        // Verify each has correct assigned_mind
        let a = store.get("research-files").await.unwrap().unwrap();
        assert_eq!(a.assigned_mind.as_deref(), Some("researcher-A"));
        let b = store.get("research-deps").await.unwrap().unwrap();
        assert_eq!(b.assigned_mind.as_deref(), Some("researcher-B"));
        let c = store.get("research-tests").await.unwrap().unwrap();
        assert_eq!(c.assigned_mind.as_deref(), Some("researcher-C"));

        let open = store.by_state(TaskState::Open).await.unwrap();
        assert_eq!(open.len(), 0, "No open tasks remaining");

        // Complete out of order: B finishes first, then C, then A
        store.complete("research-deps", Some(2), Some(4), Some("Found 47 dependencies")).await.unwrap();

        let summary = store.summary().await.unwrap();
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.in_progress, 2);

        store.complete("research-tests", Some(1), Some(1), Some("Found 242 test functions")).await.unwrap();

        let summary = store.summary().await.unwrap();
        assert_eq!(summary.completed, 2);
        assert_eq!(summary.in_progress, 1);

        store.complete("research-files", Some(3), Some(6), Some("Found 87 files in src/")).await.unwrap();

        // All 3 completed
        let summary = store.summary().await.unwrap();
        assert_eq!(summary.completed, 3);
        assert_eq!(summary.in_progress, 0);
        assert_eq!(summary.open, 0);

        // Verify result summaries preserved
        let a = store.get("research-files").await.unwrap().unwrap();
        assert_eq!(a.state, TaskState::Completed);
        assert_eq!(a.result_summary.as_deref(), Some("Found 87 files in src/"));
        assert!(a.completed_at.is_some());

        let b = store.get("research-deps").await.unwrap().unwrap();
        assert_eq!(b.state, TaskState::Completed);
        assert_eq!(b.result_summary.as_deref(), Some("Found 47 dependencies"));

        let c = store.get("research-tests").await.unwrap().unwrap();
        assert_eq!(c.state, TaskState::Completed);
        assert_eq!(c.result_summary.as_deref(), Some("Found 242 test functions"));
    }

    /// M04b — Parallel spawn with one failure: 3 tasks, 2 complete, 1 fails.
    /// Validates that failure of one task doesn't affect the other two.
    #[tokio::test]
    async fn parallel_spawn_with_failure() {
        let store = TaskStore::open_memory().await.unwrap();

        store.insert(&make_task("t-ok-1", "Good task 1", TaskPriority::Normal)).await.unwrap();
        store.insert(&make_task("t-ok-2", "Good task 2", TaskPriority::Normal)).await.unwrap();
        store.insert(&make_task("t-fail", "Failing task", TaskPriority::Normal)).await.unwrap();

        store.assign("t-ok-1", "agent-1").await.unwrap();
        store.assign("t-ok-2", "agent-2").await.unwrap();
        store.assign("t-fail", "agent-3").await.unwrap();

        // agent-1 completes
        store.complete("t-ok-1", Some(1), Some(2), Some("Success")).await.unwrap();

        // agent-3 fails
        store.set_state("t-fail", TaskState::Failed).await.unwrap();

        // agent-2 still in progress, unaffected
        let t2 = store.get("t-ok-2").await.unwrap().unwrap();
        assert_eq!(t2.state, TaskState::InProgress, "agent-2 unaffected by agent-3 failure");

        // agent-2 completes
        store.complete("t-ok-2", Some(2), Some(3), Some("Also success")).await.unwrap();

        let summary = store.summary().await.unwrap();
        assert_eq!(summary.completed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.in_progress, 0);
    }

    #[tokio::test]
    async fn not_found_error() {
        let store = TaskStore::open_memory().await.unwrap();
        let result = store.set_state("nonexistent", TaskState::Completed).await;
        assert!(matches!(result, Err(TaskStoreError::NotFound(_))));
    }

    /// M02 — Dependency chain integration test.
    ///
    /// Chain: T1 (no deps) → T2 (depends T1) → T3 (depends T2)
    ///        → T4 (depends T1 AND T3) → T5 (depends T4)
    ///
    /// Verifies:
    /// - T1 is the only Open task initially
    /// - Completing T1 unblocks T2 but NOT T4 (still waiting on T3)
    /// - Completing T2 unblocks T3
    /// - Completing T3 unblocks T4 (T1 already done)
    /// - Completing T4 unblocks T5
    #[tokio::test]
    async fn dependency_chain_five_tasks() {
        let store = TaskStore::open_memory().await.unwrap();

        // T1: no dependencies — Open
        store.insert(&make_task("T1", "Foundation task", TaskPriority::High)).await.unwrap();

        // T2: depends on T1 — Blocked
        let mut t2 = make_task("T2", "Depends on T1", TaskPriority::Normal);
        t2.state = TaskState::Blocked;
        t2.depends_on = Some("T1".into());
        store.insert(&t2).await.unwrap();

        // T3: depends on T2 — Blocked
        let mut t3 = make_task("T3", "Depends on T2", TaskPriority::Normal);
        t3.state = TaskState::Blocked;
        t3.depends_on = Some("T2".into());
        store.insert(&t3).await.unwrap();

        // T4: depends on T1 AND T3 — Blocked
        let mut t4 = make_task("T4", "Depends on T1 and T3", TaskPriority::Normal);
        t4.state = TaskState::Blocked;
        t4.depends_on = Some("T1,T3".into());
        store.insert(&t4).await.unwrap();

        // T5: depends on T4 — Blocked
        let mut t5 = make_task("T5", "Depends on T4", TaskPriority::Normal);
        t5.state = TaskState::Blocked;
        t5.depends_on = Some("T4".into());
        store.insert(&t5).await.unwrap();

        // === Initial state: only T1 should be Open ===
        let open = store.by_state(TaskState::Open).await.unwrap();
        assert_eq!(open.len(), 1, "Only T1 should be open initially");
        assert_eq!(open[0].task_id, "T1");

        let blocked = store.by_state(TaskState::Blocked).await.unwrap();
        assert_eq!(blocked.len(), 4, "T2-T5 should all be blocked");

        // === Complete T1 → T2 should unblock, T4 should NOT (still needs T3) ===
        store.assign("T1", "agent-1").await.unwrap();
        store.complete("T1", Some(3), Some(8), Some("Foundation done")).await.unwrap();

        let t2_state = store.get("T2").await.unwrap().unwrap();
        assert_eq!(t2_state.state, TaskState::Open, "T2 should be unblocked after T1 completes");

        let t3_state = store.get("T3").await.unwrap().unwrap();
        assert_eq!(t3_state.state, TaskState::Blocked, "T3 should still be blocked (needs T2)");

        let t4_state = store.get("T4").await.unwrap().unwrap();
        assert_eq!(t4_state.state, TaskState::Blocked, "T4 should still be blocked (needs T1 AND T3, T3 not done)");

        let t5_state = store.get("T5").await.unwrap().unwrap();
        assert_eq!(t5_state.state, TaskState::Blocked, "T5 should still be blocked (needs T4)");

        // === Complete T2 → T3 should unblock ===
        store.assign("T2", "agent-2").await.unwrap();
        store.complete("T2", Some(2), Some(5), Some("T2 done")).await.unwrap();

        let t3_state = store.get("T3").await.unwrap().unwrap();
        assert_eq!(t3_state.state, TaskState::Open, "T3 should be unblocked after T2 completes");

        let t4_state = store.get("T4").await.unwrap().unwrap();
        assert_eq!(t4_state.state, TaskState::Blocked, "T4 still blocked (needs T3)");

        // === Complete T3 → T4 should unblock (T1 already done) ===
        store.assign("T3", "agent-3").await.unwrap();
        store.complete("T3", Some(1), Some(3), Some("T3 done")).await.unwrap();

        let t4_state = store.get("T4").await.unwrap().unwrap();
        assert_eq!(t4_state.state, TaskState::Open, "T4 should be unblocked (T1 and T3 both done)");

        let t5_state = store.get("T5").await.unwrap().unwrap();
        assert_eq!(t5_state.state, TaskState::Blocked, "T5 still blocked (needs T4)");

        // === Complete T4 → T5 should unblock ===
        store.assign("T4", "agent-4").await.unwrap();
        store.complete("T4", Some(4), Some(10), Some("T4 done")).await.unwrap();

        let t5_state = store.get("T5").await.unwrap().unwrap();
        assert_eq!(t5_state.state, TaskState::Open, "T5 should be unblocked after T4 completes");

        // === Complete T5 — full chain done ===
        store.assign("T5", "agent-5").await.unwrap();
        store.complete("T5", Some(1), Some(2), Some("T5 done")).await.unwrap();

        // Final state: all 5 completed
        let completed = store.by_state(TaskState::Completed).await.unwrap();
        assert_eq!(completed.len(), 5, "All 5 tasks should be completed");

        let summary = store.summary().await.unwrap();
        assert_eq!(summary.completed, 5);
        assert_eq!(summary.open, 0);
        assert_eq!(summary.blocked, 0);
        assert_eq!(summary.in_progress, 0);

        // Verify execution order was correct: T1 → T2 → T3 → T4 → T5
        // (Each was the only Open task when assigned, enforced by dependency chain)
    }
}
