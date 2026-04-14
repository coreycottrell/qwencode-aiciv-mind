//! # Drive Integration — EventBus + DriveLoop for Cortex
//!
//! Wires the autonomous heartbeat into the Cortex serve loop.
//! DriveLoop runs as a background task alongside the MCP server.
//! Events are processed by `handle_drive()` which logs and acts on them.

use std::path::Path;
use std::sync::Arc;

use codex_drive::{DriveConfig, DriveLoop, TaskStore};
use codex_drive::event_bus::{self, EventBus, ExternalSender};
use codex_types::MindEvent;
use tracing::{debug, info, warn};

/// All the handles needed to interact with the drive subsystem.
pub struct DriveHandles {
    /// Send external events (Hub, Telegram, Human input) into the EventBus.
    pub external_tx: ExternalSender,
    /// The DriveLoop — call `completion_sender()` to get the watch sender.
    pub drive_loop: Arc<DriveLoop>,
    /// The TaskStore — shared with ProcessBridge for delegation tracking.
    pub task_store: TaskStore,
    /// JoinHandle for the DriveLoop background task.
    drive_handle: tokio::task::JoinHandle<()>,
    /// JoinHandle for the event handler background task.
    event_handle: tokio::task::JoinHandle<()>,
}

impl DriveHandles {
    /// Shut down the drive subsystem gracefully.
    pub fn shutdown(self) {
        self.drive_handle.abort();
        self.event_handle.abort();
    }
}

/// Daemon mode handles — EventBus is returned to caller for direct processing.
pub struct DaemonHandles {
    /// Send external events into the EventBus.
    pub external_tx: ExternalSender,
    /// The DriveLoop — call `completion_sender()` to get the watch sender.
    pub drive_loop: Arc<DriveLoop>,
    /// The TaskStore — shared with ProcessBridge for delegation tracking.
    pub task_store: TaskStore,
    /// JoinHandle for the DriveLoop background task.
    drive_handle: tokio::task::JoinHandle<()>,
}

impl DaemonHandles {
    /// Shut down the drive subsystem gracefully.
    pub fn shutdown(self) {
        self.drive_handle.abort();
    }
}

/// Boot for daemon mode: DriveLoop runs as background task, EventBus returned to caller.
///
/// Unlike `boot()`, the caller processes EventBus events directly (via ThinkLoop).
pub async fn boot_daemon(
    project_root: &Path,
    mind_id: &str,
    role_str: &str,
    drive_config: Option<DriveConfig>,
) -> Result<(DaemonHandles, EventBus), Box<dyn std::error::Error + Send + Sync>> {
    let tasks_dir = project_root.join("data").join("tasks");
    let _ = std::fs::create_dir_all(&tasks_dir);
    let db_path = tasks_dir.join(format!("{}.db", mind_id));
    let task_store = TaskStore::open(&db_path).await?;

    info!(mind_id = mind_id, db = %db_path.display(), "TaskStore opened (daemon)");

    let config = drive_config.unwrap_or_default();
    match task_store.block_stale_in_progress(config.stall_threshold_secs).await {
        Ok(count) if count > 0 => info!(count, "Recovered stale tasks at boot"),
        Err(e) => warn!(error = %e, "Failed to recover stale tasks at boot"),
        _ => {}
    }

    let (bus, external_tx, drive_tx) = event_bus::create();
    let external_pending = || false;

    let drive_loop = Arc::new(DriveLoop::new(
        config,
        task_store.clone(),
        drive_tx,
        external_pending,
        role_str,
    ));

    let drive_handle = {
        let dl = drive_loop.clone();
        tokio::spawn(async move { dl.run().await; })
    };

    info!(mind_id = mind_id, role = role_str, "Drive subsystem booted (daemon mode)");

    let handles = DaemonHandles {
        external_tx,
        drive_loop,
        task_store,
        drive_handle,
    };

    Ok((handles, bus))
}

/// Boot the drive subsystem: TaskStore + EventBus + DriveLoop.
///
/// Returns handles for external event injection and task store access.
/// DriveLoop and event handler are spawned as background tokio tasks.
pub async fn boot(
    project_root: &Path,
    mind_id: &str,
    role_str: &str,
) -> Result<DriveHandles, Box<dyn std::error::Error + Send + Sync>> {
    // Open SQLite task store
    let tasks_dir = project_root.join("data").join("tasks");
    let _ = std::fs::create_dir_all(&tasks_dir);
    let db_path = tasks_dir.join(format!("{}.db", mind_id));
    let task_store = TaskStore::open(&db_path).await?;

    info!(
        mind_id = mind_id,
        db = %db_path.display(),
        "TaskStore opened"
    );

    // Boot-time recovery: reset stale in-progress tasks
    let config = DriveConfig::default();
    match task_store.block_stale_in_progress(config.stall_threshold_secs).await {
        Ok(count) if count > 0 => {
            info!(count, "Recovered stale tasks at boot");
        }
        Err(e) => {
            warn!(error = %e, "Failed to recover stale tasks at boot");
        }
        _ => {}
    }

    // Create EventBus
    let (bus, external_tx, drive_tx) = event_bus::create();

    // Create DriveLoop
    // DriveLoop checks external_pending to yield to external events.
    // Returns false here because the EventBus biased select is the primary
    // priority mechanism — this is defense-in-depth.
    let external_pending = || false;

    let drive_loop = Arc::new(DriveLoop::new(
        config,
        task_store.clone(),
        drive_tx,
        external_pending,
        role_str,
    ));

    // Spawn DriveLoop as background task
    let drive_handle = {
        let dl = drive_loop.clone();
        tokio::spawn(async move {
            dl.run().await;
        })
    };

    // Spawn event handler as background task
    let event_mind_id = mind_id.to_string();
    let event_store = task_store.clone();
    let event_drive = drive_loop.clone();
    let event_handle = tokio::spawn(async move {
        handle_events(bus, &event_mind_id, &event_store, &event_drive).await;
    });

    info!(mind_id = mind_id, role = role_str, "Drive subsystem booted");

    Ok(DriveHandles {
        external_tx,
        drive_loop,
        task_store,
        drive_handle,
        event_handle,
    })
}

/// Main event processing loop — receives from EventBus and handles each event.
async fn handle_events(
    mut bus: EventBus,
    mind_id: &str,
    task_store: &TaskStore,
    drive_loop: &DriveLoop,
) {
    while let Some(event) = bus.recv().await {
        match event {
            MindEvent::External(ext) => {
                info!(
                    mind_id = mind_id,
                    source = ?ext.source,
                    priority = ?ext.priority,
                    "External event received"
                );
                // External events are handled by the MCP server (parent sends delegate requests).
                // This path is for future use when minds can receive events from Hub/Telegram directly.
            }
            MindEvent::Drive(drive_event) => {
                handle_drive(mind_id, &drive_event, task_store, drive_loop).await;
            }
        }
    }
    info!(mind_id = mind_id, "EventBus closed — drive event handler exiting");
}

/// Handle a single drive event.
async fn handle_drive(
    mind_id: &str,
    event: &codex_types::DriveEvent,
    _task_store: &TaskStore,
    drive_loop: &DriveLoop,
) {
    use codex_types::DriveEvent;

    match event {
        DriveEvent::TaskAvailable { task_id, description, priority } => {
            info!(
                mind_id = mind_id,
                task_id = task_id.as_str(),
                priority = ?priority,
                "DriveLoop: task available — {}",
                description
            );
            // In future: auto-assign to self or spawn agent.
            // For now: log for observability. Parent mind handles task assignment.
            drive_loop.mark_productive().await;
        }

        DriveEvent::StallDetected { task_id, mind_id: stalled_mind, stalled_seconds } => {
            warn!(
                mind_id = mind_id,
                stalled_mind = stalled_mind.as_str(),
                task_id = task_id.as_str(),
                stalled_seconds = stalled_seconds,
                "DriveLoop: stall detected"
            );
            // In future: auto-restart or escalate to parent.
            // For now: log warning for observability.
        }

        DriveEvent::IdleSuggestion { suggestion } => {
            debug!(
                mind_id = mind_id,
                "DriveLoop: idle — {}",
                suggestion
            );
            // Idle suggestion is informational. Don't mark productive
            // (so backoff continues increasing if truly idle).
        }

        DriveEvent::HealthCheck { active_minds, pending_tasks, uptime_seconds } => {
            info!(
                mind_id = mind_id,
                active_minds = active_minds,
                pending_tasks = pending_tasks,
                uptime_seconds = uptime_seconds,
                "DriveLoop: health check"
            );
            // Health checks are informational but indicate work exists.
            if *active_minds > 0 || *pending_tasks > 0 {
                drive_loop.mark_productive().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_types::{DriveEvent, EventPriority};

    #[tokio::test]
    async fn boot_creates_task_store() {
        let tmp = tempfile::TempDir::new().unwrap();
        let handles = boot(tmp.path(), "test-mind", "agent").await.unwrap();

        // TaskStore should be operational
        let summary = handles.task_store.summary().await.unwrap();
        assert_eq!(summary.open, 0);
        assert_eq!(summary.in_progress, 0);

        handles.shutdown();
    }

    #[tokio::test]
    async fn handle_drive_task_available() {
        let tmp = tempfile::TempDir::new().unwrap();
        let tasks_dir = tmp.path().join("data").join("tasks");
        let _ = std::fs::create_dir_all(&tasks_dir);
        let db_path = tasks_dir.join("test.db");
        let store = TaskStore::open(&db_path).await.unwrap();

        let (drive_tx, _rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = codex_drive::event_bus::DriveSender::from_sender(drive_tx);
        let drive_loop = DriveLoop::new(
            DriveConfig::default(),
            store.clone(),
            drive_sender,
            || false,
            "test",
        );

        let event = DriveEvent::TaskAvailable {
            task_id: "t-1".into(),
            description: "Test task".into(),
            priority: EventPriority::Normal,
        };

        // Should not panic
        handle_drive("test-mind", &event, &store, &drive_loop).await;
    }

    #[tokio::test]
    async fn handle_drive_stall_detected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let tasks_dir = tmp.path().join("data").join("tasks");
        let _ = std::fs::create_dir_all(&tasks_dir);
        let db_path = tasks_dir.join("test.db");
        let store = TaskStore::open(&db_path).await.unwrap();

        let (drive_tx, _rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = codex_drive::event_bus::DriveSender::from_sender(drive_tx);
        let drive_loop = DriveLoop::new(
            DriveConfig::default(),
            store.clone(),
            drive_sender,
            || false,
            "test",
        );

        let event = DriveEvent::StallDetected {
            task_id: "t-stall".into(),
            mind_id: "agent-1".into(),
            stalled_seconds: 180,
        };

        handle_drive("test-mind", &event, &store, &drive_loop).await;
    }
}
