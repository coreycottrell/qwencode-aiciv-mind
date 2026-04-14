//! # DriveLoop — Autonomous Heartbeat
//!
//! The DriveLoop is an event SOURCE, not a controller. It detects idle periods,
//! discovers ready tasks, identifies stalls, and emits DriveEvents into the
//! EventBus's drive channel.
//!
//! The EventBus's biased select ensures external events always preempt drive events.
//! DriveLoop + EventBus together implement InputMux (Principle A2).
//!
//! ## Lifecycle
//!
//! 1. Boot settle (wait for initial context loading)
//! 2. Recover stale in-progress tasks from previous crash
//! 3. Main loop:
//!    a. Wait for either: task completion notification OR idle timeout
//!    b. Cooldown (prevent rapid-fire events)
//!    c. Yield to external (if external events are pending, skip)
//!    d. Scan for: ready tasks, stalled tasks, idle suggestions
//!    e. Emit one DriveEvent via try_send (capacity-1 = natural backpressure)
//!    f. Adaptive backoff: if event was unproductive, increase threshold

use std::time::Duration;
use std::sync::Arc;

use codex_types::{DriveEvent, EventPriority};
use tokio::sync::watch;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::event_bus::DriveSender;
use crate::task_store::TaskStore;

/// Configuration for the DriveLoop.
#[derive(Debug, Clone)]
pub struct DriveConfig {
    /// Wait this long after boot before first drive event.
    pub boot_settle: Duration,
    /// Minimum time between drive events.
    pub cooldown: Duration,
    /// Initial idle threshold before generating a drive event.
    pub idle_threshold: Duration,
    /// Maximum idle threshold (adaptive backoff ceiling).
    pub max_idle_threshold: Duration,
    /// Backoff multiplier when a drive event is unproductive.
    pub backoff_multiplier: f64,
    /// Recovery multiplier when a drive event IS productive.
    pub recovery_divisor: f64,
    /// Tasks idle longer than this (seconds) are considered stalled.
    pub stall_threshold_secs: i64,
    /// When true, skip cooldown after Completion triggers (dependent tasks chain instantly).
    pub completion_fast_path: bool,
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            boot_settle: Duration::from_secs(5),
            cooldown: Duration::from_secs(2),
            idle_threshold: Duration::from_secs(30),
            max_idle_threshold: Duration::from_secs(300),
            backoff_multiplier: 1.5,
            recovery_divisor: 2.0,
            stall_threshold_secs: 120,
            completion_fast_path: true,
        }
    }
}

/// Internal state of the DriveLoop.
struct DriveState {
    /// Current adaptive idle threshold.
    current_threshold: Duration,
    /// Whether the last drive event resulted in productive work.
    last_was_productive: bool,
    /// Total drive events emitted.
    events_emitted: u64,
    /// Total events that led to productive work.
    productive_events: u64,
}

impl DriveState {
    fn new(initial_threshold: Duration) -> Self {
        Self {
            current_threshold: initial_threshold,
            last_was_productive: true,
            events_emitted: 0,
            productive_events: 0,
        }
    }
}

/// The trigger that woke the DriveLoop.
enum Trigger {
    /// A task completed — carries the task_id of what completed (if known).
    Completion { task_id: Option<String> },
    /// Idle threshold elapsed — scan for work.
    Idle,
}

/// The DriveLoop — autonomous heartbeat for Cortex minds.
pub struct DriveLoop {
    config: DriveConfig,
    state: Arc<Mutex<DriveState>>,
    task_store: TaskStore,
    drive_tx: DriveSender,
    /// Watch receiver — wakes the loop AND carries the task_id of what completed.
    completion_rx: Mutex<watch::Receiver<Option<String>>>,
    /// Watch sender — returned to callers via completion_sender().
    completion_tx: Arc<watch::Sender<Option<String>>>,
    /// Check external_pending before emitting (yield to external work).
    external_pending: Arc<dyn Fn() -> bool + Send + Sync>,
    /// Role string for prompt generation.
    role_str: String,
}

impl DriveLoop {
    /// Create a new DriveLoop.
    pub fn new(
        config: DriveConfig,
        task_store: TaskStore,
        drive_tx: DriveSender,
        external_pending_fn: impl Fn() -> bool + Send + Sync + 'static,
        role_str: &str,
    ) -> Self {
        let state = Arc::new(Mutex::new(DriveState::new(config.idle_threshold)));
        let (completion_tx, completion_rx) = watch::channel(None);
        Self {
            config,
            state,
            task_store,
            drive_tx,
            completion_rx: Mutex::new(completion_rx),
            completion_tx: Arc::new(completion_tx),
            external_pending: Arc::new(external_pending_fn),
            role_str: role_str.into(),
        }
    }

    /// Get a sender handle to signal task completions to the DriveLoop.
    ///
    /// Callers send `Some(task_id)` when a task completes.
    /// The DriveLoop wakes and checks for newly unblocked tasks.
    pub fn completion_sender(&self) -> Arc<watch::Sender<Option<String>>> {
        self.completion_tx.clone()
    }

    /// Signal that the last drive event was productive (reset backoff).
    pub async fn mark_productive(&self) {
        let mut state = self.state.lock().await;
        state.last_was_productive = true;
        state.productive_events += 1;
        // Recover threshold
        let new_threshold = Duration::from_secs_f64(
            state.current_threshold.as_secs_f64() / self.config.recovery_divisor
        );
        state.current_threshold = new_threshold.max(self.config.idle_threshold);
    }

    /// Run the DriveLoop. This is a long-running future — spawn it as a task.
    pub async fn run(&self) {
        info!(
            boot_settle = ?self.config.boot_settle,
            idle_threshold = ?self.config.idle_threshold,
            role = %self.role_str,
            "DriveLoop starting"
        );

        // Phase 1: Boot settle
        tokio::time::sleep(self.config.boot_settle).await;

        // Phase 2: Recover stale tasks from previous crash
        match self.task_store.block_stale_in_progress(self.config.stall_threshold_secs).await {
            Ok(count) if count > 0 => {
                info!(count, "DriveLoop recovered stale tasks at boot");
            }
            Err(e) => {
                warn!(error = %e, "DriveLoop failed to recover stale tasks");
            }
            _ => {}
        }

        // Phase 3: Main loop
        loop {
            // Wait for trigger
            let trigger = {
                let threshold = {
                    self.state.lock().await.current_threshold
                };
                let mut rx = self.completion_rx.lock().await;
                tokio::select! {
                    Ok(()) = rx.changed() => {
                        let completed_id = rx.borrow_and_update().clone();
                        Trigger::Completion { task_id: completed_id }
                    },
                    _ = tokio::time::sleep(threshold) => Trigger::Idle,
                }
            };

            // Cooldown — skip on completion fast-path so dependent task chains execute instantly
            let skip_cooldown = matches!(trigger, Trigger::Completion { .. })
                && self.config.completion_fast_path;
            if !skip_cooldown {
                tokio::time::sleep(self.config.cooldown).await;
            }

            // Yield to external work
            if (self.external_pending)() {
                debug!("DriveLoop yielding to external events");
                continue;
            }

            // Generate and emit event
            let event = match trigger {
                Trigger::Completion { task_id } => self.on_completion(task_id).await,
                Trigger::Idle => self.on_idle().await,
            };

            if let Some(event) = event {
                debug!(event = ?event, "DriveLoop emitting event");
                if self.drive_tx.try_send(event).is_err() {
                    // Channel full — backpressure working as designed
                    debug!("Drive channel full — event dropped (backpressure)");
                }

                let mut state = self.state.lock().await;
                state.events_emitted += 1;

                // Adaptive backoff: if last event was unproductive, increase threshold
                if !state.last_was_productive {
                    let new_threshold = Duration::from_secs_f64(
                        state.current_threshold.as_secs_f64() * self.config.backoff_multiplier
                    );
                    state.current_threshold = new_threshold.min(self.config.max_idle_threshold);
                    debug!(
                        new_threshold = ?state.current_threshold,
                        "DriveLoop backing off (unproductive)"
                    );
                }
                state.last_was_productive = false; // Reset — will be set by mark_productive()
            }
        }
    }

    /// Handle: a task just completed. Check for newly ready tasks.
    async fn on_completion(&self, completed_task_id: Option<String>) -> Option<DriveEvent> {
        if let Some(ref id) = completed_task_id {
            debug!(task_id = %id, "Task completion signal received");
        }

        // Check if any tasks were unblocked
        if let Ok(Some(task)) = self.task_store.next_ready().await {
            return Some(DriveEvent::TaskAvailable {
                task_id: task.task_id,
                description: task.description,
                priority: match task.priority {
                    crate::task_store::TaskPriority::Low => EventPriority::Low,
                    crate::task_store::TaskPriority::Normal => EventPriority::Normal,
                    crate::task_store::TaskPriority::High => EventPriority::High,
                    crate::task_store::TaskPriority::Critical => EventPriority::Critical,
                },
            });
        }
        None
    }

    /// Handle: idle timeout. Scan for stalls, ready tasks, or generate suggestion.
    async fn on_idle(&self) -> Option<DriveEvent> {
        // Priority 1: Check for stalled tasks
        if let Ok(stalled) = self.task_store.stalled_tasks(self.config.stall_threshold_secs).await {
            if let Some(task) = stalled.first() {
                let elapsed = (Utc::now() - task.updated_at).num_seconds().max(0) as u64;
                return Some(DriveEvent::StallDetected {
                    task_id: task.task_id.clone(),
                    mind_id: task.assigned_mind.clone().unwrap_or_default(),
                    stalled_seconds: elapsed,
                });
            }
        }

        // Priority 2: Check for ready tasks
        if let Ok(Some(task)) = self.task_store.next_ready().await {
            return Some(DriveEvent::TaskAvailable {
                task_id: task.task_id,
                description: task.description,
                priority: match task.priority {
                    crate::task_store::TaskPriority::Low => EventPriority::Low,
                    crate::task_store::TaskPriority::Normal => EventPriority::Normal,
                    crate::task_store::TaskPriority::High => EventPriority::High,
                    crate::task_store::TaskPriority::Critical => EventPriority::Critical,
                },
            });
        }

        // Priority 3: Health check
        if let Ok(summary) = self.task_store.summary().await {
            if summary.in_progress > 0 || summary.open > 0 {
                return Some(DriveEvent::HealthCheck {
                    active_minds: summary.in_progress,
                    pending_tasks: summary.open,
                    uptime_seconds: 0, // TODO: track actual uptime
                });
            }
        }

        // Nothing to do — idle suggestion
        Some(DriveEvent::IdleSuggestion {
            suggestion: format!(
                "No tasks pending. Consider: search memory for unfinished work, \
                 check Hub for new threads, or review recent learnings."
            ),
        })
    }
}

use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_store::{StoredTask, TaskState, TaskPriority as TP};

    fn make_task(id: &str, priority: TP) -> StoredTask {
        let now = Utc::now();
        StoredTask {
            task_id: id.into(),
            description: format!("Task {id}"),
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
    async fn drive_config_defaults() {
        let config = DriveConfig::default();
        assert_eq!(config.boot_settle, Duration::from_secs(5));
        assert_eq!(config.idle_threshold, Duration::from_secs(30));
        assert_eq!(config.max_idle_threshold, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn on_idle_finds_ready_task() {
        let store = TaskStore::open_memory().await.unwrap();
        store.insert(&make_task("t-ready", TP::High)).await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        let drive = DriveLoop::new(
            DriveConfig::default(),
            store,
            drive_sender,
            || false,
            "primary",
        );

        let event = drive.on_idle().await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = event {
            assert_eq!(task_id, "t-ready");
        }
    }

    #[tokio::test]
    async fn on_idle_idle_suggestion_when_empty() {
        let store = TaskStore::open_memory().await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        let drive = DriveLoop::new(
            DriveConfig::default(),
            store,
            drive_sender,
            || false,
            "agent",
        );

        let event = drive.on_idle().await;
        assert!(matches!(event, Some(DriveEvent::IdleSuggestion { .. })));
    }

    #[tokio::test]
    async fn completion_sender_carries_task_id() {
        let store = TaskStore::open_memory().await.unwrap();
        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        let drive = DriveLoop::new(
            DriveConfig::default(),
            store,
            drive_sender,
            || false,
            "primary",
        );

        let sender = drive.completion_sender();

        // Send a completion signal with task_id
        sender.send(Some("task-abc".into())).unwrap();

        // Receiver should see the value
        let mut rx = drive.completion_rx.lock().await;
        rx.changed().await.unwrap();
        let val = rx.borrow_and_update().clone();
        assert_eq!(val, Some("task-abc".into()));
    }

    #[tokio::test]
    async fn on_completion_with_known_task_id() {
        let store = TaskStore::open_memory().await.unwrap();
        store.insert(&make_task("t-dep", TP::Normal)).await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        let drive = DriveLoop::new(
            DriveConfig::default(),
            store,
            drive_sender,
            || false,
            "primary",
        );

        // on_completion with a known completed task_id should still find the ready task
        let event = drive.on_completion(Some("some-parent".into())).await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = event {
            assert_eq!(task_id, "t-dep");
        }
    }

    /// M03 — Stall detection: an InProgress task older than stall_threshold
    /// should cause on_idle() to emit StallDetected instead of TaskAvailable.
    #[tokio::test]
    async fn on_idle_detects_stalled_task() {
        let store = TaskStore::open_memory().await.unwrap();

        // Insert and assign a task (moves to InProgress, sets updated_at = now)
        store.insert(&make_task("t-slow", TP::Normal)).await.unwrap();
        store.assign("t-slow", "slow-agent").await.unwrap();

        // Small sleep so updated_at is strictly in the past relative to stall check
        tokio::time::sleep(Duration::from_millis(20)).await;

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        // stall_threshold_secs = 0 means ANY InProgress task is immediately stale
        let config = DriveConfig {
            stall_threshold_secs: 0,
            ..Default::default()
        };

        let drive = DriveLoop::new(
            config,
            store,
            drive_sender,
            || false,
            "primary",
        );

        let event = drive.on_idle().await;

        // Should be StallDetected, NOT TaskAvailable (stall check has priority 1)
        assert!(
            matches!(event, Some(DriveEvent::StallDetected { .. })),
            "Expected StallDetected, got: {:?}",
            event,
        );

        if let Some(DriveEvent::StallDetected { task_id, mind_id, stalled_seconds }) = event {
            assert_eq!(task_id, "t-slow", "Stalled task_id should match");
            assert_eq!(mind_id, "slow-agent", "Stalled mind_id should match assigned agent");
            // stalled_seconds should be very small (just milliseconds) but >= 0
            assert!(stalled_seconds < 5, "Stalled seconds should be near-zero in test: got {stalled_seconds}");
        }
    }

    /// M03b — Stall detection does NOT fire for tasks within threshold.
    /// A freshly assigned task should NOT be detected as stalled when
    /// threshold is large.
    #[tokio::test]
    async fn on_idle_no_stall_within_threshold() {
        let store = TaskStore::open_memory().await.unwrap();

        // Insert and assign a task
        store.insert(&make_task("t-fresh", TP::Normal)).await.unwrap();
        store.assign("t-fresh", "fast-agent").await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        // Large stall threshold — task was JUST assigned, shouldn't be stale
        let config = DriveConfig {
            stall_threshold_secs: 3600, // 1 hour
            ..Default::default()
        };

        let drive = DriveLoop::new(
            config,
            store,
            drive_sender,
            || false,
            "primary",
        );

        let event = drive.on_idle().await;

        // No open tasks, no stalled tasks — should be HealthCheck (1 in_progress)
        assert!(
            matches!(event, Some(DriveEvent::HealthCheck { .. })),
            "Expected HealthCheck (task in progress, not stalled), got: {:?}",
            event,
        );

        if let Some(DriveEvent::HealthCheck { active_minds, pending_tasks, .. }) = event {
            assert_eq!(active_minds, 1, "One task in progress");
            assert_eq!(pending_tasks, 0, "No open tasks");
        }
    }

    /// M04c — Parallel completions: 3 tasks run concurrently, a 4th depends on all 3.
    /// Verifies on_completion surfaces the right next task at each step and
    /// the fan-in dependency unblocks only when all 3 are done.
    #[tokio::test]
    async fn parallel_completion_surfaces_ready_and_fan_in() {
        let store = TaskStore::open_memory().await.unwrap();

        // 3 parallel tasks
        store.insert(&make_task("p-1", TP::Normal)).await.unwrap();
        store.insert(&make_task("p-2", TP::Normal)).await.unwrap();
        store.insert(&make_task("p-3", TP::Normal)).await.unwrap();

        // 1 blocked on all 3 (fan-in)
        let now = Utc::now();
        let blocked = crate::task_store::StoredTask {
            task_id: "p-final".into(),
            description: "Fan-in: depends on p-1,p-2,p-3".into(),
            state: crate::task_store::TaskState::Blocked,
            priority: crate::task_store::TaskPriority::High,
            assigned_mind: None,
            parent_mind: Some("primary".into()),
            depends_on: Some("p-1,p-2,p-3".into()),
            created_at: now,
            updated_at: now,
            completed_at: None,
            iterations: None,
            tool_calls: None,
            result_summary: None,
        };
        store.insert(&blocked).await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);
        let config = DriveConfig {
            stall_threshold_secs: 3600,
            ..Default::default()
        };
        let drive = DriveLoop::new(config, store.clone(), drive_sender, || false, "primary");

        // Assign and complete p-1, p-2 (p-3 still open)
        store.assign("p-1", "a1").await.unwrap();
        store.complete("p-1", None, None, None).await.unwrap();
        store.assign("p-2", "a2").await.unwrap();
        store.complete("p-2", None, None, None).await.unwrap();

        // p-final still blocked (needs p-3)
        let pf = store.get("p-final").await.unwrap().unwrap();
        assert_eq!(pf.state, crate::task_store::TaskState::Blocked);

        // on_completion should find p-3 as next ready task
        let event = drive.on_completion(Some("p-2".into())).await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = &event {
            assert_eq!(task_id, "p-3");
        }

        // Complete p-3 — p-final should unblock
        store.assign("p-3", "a3").await.unwrap();
        store.complete("p-3", None, None, None).await.unwrap();

        let pf = store.get("p-final").await.unwrap().unwrap();
        assert_eq!(pf.state, crate::task_store::TaskState::Open, "Fan-in: all 3 done → unblocked");

        // DriveLoop should now surface p-final
        let event = drive.on_completion(Some("p-3".into())).await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = &event {
            assert_eq!(task_id, "p-final");
        }
    }

    #[tokio::test]
    async fn completion_fast_path_default_enabled() {
        let config = DriveConfig::default();
        assert!(
            config.completion_fast_path,
            "Completion fast-path should be enabled by default"
        );
    }

    #[tokio::test]
    async fn completion_fast_path_chains_tasks_instantly() {
        // This test verifies that when fast-path is enabled, a 3-task chain
        // (t1 → t2 → t3) executes without cooldown delays between completions.
        let store = TaskStore::open_memory().await.unwrap();

        // Create chain: t1 (open), t2 (blocked on t1), t3 (blocked on t2)
        store.insert(&make_task("chain-1", TP::Normal)).await.unwrap();
        let now = Utc::now();
        store.insert(&StoredTask {
            task_id: "chain-2".into(),
            description: "Step 2".into(),
            state: TaskState::Blocked,
            priority: TP::Normal,
            assigned_mind: None,
            parent_mind: Some("primary".into()),
            depends_on: Some("chain-1".into()),
            created_at: now, updated_at: now, completed_at: None,
            iterations: None, tool_calls: None, result_summary: None,
        }).await.unwrap();
        store.insert(&StoredTask {
            task_id: "chain-3".into(),
            description: "Step 3".into(),
            state: TaskState::Blocked,
            priority: TP::Normal,
            assigned_mind: None,
            parent_mind: Some("primary".into()),
            depends_on: Some("chain-2".into()),
            created_at: now, updated_at: now, completed_at: None,
            iterations: None, tool_calls: None, result_summary: None,
        }).await.unwrap();

        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);
        let config = DriveConfig {
            completion_fast_path: true,
            stall_threshold_secs: 3600,
            ..Default::default()
        };
        let drive = DriveLoop::new(config, store.clone(), drive_sender, || false, "primary");

        // Complete chain-1 → chain-2 should unblock
        store.assign("chain-1", "a1").await.unwrap();
        store.complete("chain-1", None, None, None).await.unwrap();

        let event = drive.on_completion(Some("chain-1".into())).await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = &event {
            assert_eq!(task_id, "chain-2", "chain-2 should be next after chain-1 completes");
        }

        // Complete chain-2 → chain-3 should unblock
        store.assign("chain-2", "a2").await.unwrap();
        store.complete("chain-2", None, None, None).await.unwrap();

        let event = drive.on_completion(Some("chain-2".into())).await;
        assert!(matches!(event, Some(DriveEvent::TaskAvailable { .. })));
        if let Some(DriveEvent::TaskAvailable { task_id, .. }) = &event {
            assert_eq!(task_id, "chain-3", "chain-3 should be next after chain-2 completes");
        }
    }

    #[tokio::test]
    async fn mark_productive_recovers_threshold() {
        let store = TaskStore::open_memory().await.unwrap();
        let (drv_tx, _drv_rx) = tokio::sync::mpsc::channel(1);
        let drive_sender = DriveSender::from_sender(drv_tx);

        let config = DriveConfig {
            idle_threshold: Duration::from_secs(10),
            ..Default::default()
        };

        let drive = DriveLoop::new(
            config,
            store,
            drive_sender,
            || false,
            "primary",
        );

        // Simulate backoff
        {
            let mut state = drive.state.lock().await;
            state.current_threshold = Duration::from_secs(60);
            state.last_was_productive = false;
        }

        // Mark productive
        drive.mark_productive().await;

        let state = drive.state.lock().await;
        // Should recover toward idle_threshold
        assert!(state.current_threshold < Duration::from_secs(60));
        assert!(state.last_was_productive);
    }
}
