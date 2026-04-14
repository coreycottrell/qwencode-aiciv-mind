# DriveLoop + Challenger — Cortex Integration Design

**Author**: Mind-Cubed Team Lead
**Date**: 2026-04-04
**Status**: BUILD v3 — Corey GREEN LIGHT. M2.7 ONLY. Ollama direct. Phases 1-2 in parallel.
**Grounded in**: DESIGN-PRINCIPLES.md (12 principles) + DESIGN-PRINCIPLES-ADDENDUM.md (6 addendum)
**Spec source**: DRIVELOOP-CHALLENGER-SPEC.md (14 requirements, principle-traced, 5 open questions)

---

## 0. Existing Foundation

Cortex already has pieces of this architecture. The design builds on what exists rather than replacing it.

### What Exists

| Component | Location | State |
|-----------|----------|-------|
| `Challenger` | `codex-redteam/src/lib.rs` | 4 of 6 checks implemented, integrated into ThinkLoop |
| `RedTeamProtocol` | `codex-redteam/src/lib.rs` | LLM-based completion verification (separate from structural Challenger) |
| `ToolInterceptor` trait | `codex-llm/src/think_loop.rs` | Pre-execution hook, used by team leads for spawn/delegate tools |
| `ThinkLoop` | `codex-llm/src/think_loop.rs` | Challenger already wired in at lines 326-340 (post-tool) and 350-360 (final response) |
| `TaskLedger` | `codex-coordination/src/task_ledger.rs` | Append-only JSONL, records delegations + completions |
| `ProcessBridge` | `codex-coordination/src/process_bridge.rs` | MindId → child process mapping, MCP transport |

### What's Missing

| Component | Needed For | Priority |
|-----------|-----------|----------|
| Role-aware Challenger | Eliminating false positives on team leads (the known gap) | **P0** |
| Filesystem verification (Check 5) | Catching hallucinated file claims | **P0** |
| State file integrity (Check 6) | Catching evolution status lies | **P1** |
| DriveLoop | Autonomous self-prompting between tasks | **P0** |
| EventBus / InputMux | Unified event routing with priority | **P0** |
| TaskStore | Task queue with status, priority, dependencies | **P0** |

### Design Principle Grounding

Every decision below traces to a principle:

| Decision | Principle | Why |
|----------|-----------|-----|
| Challenger is structural, zero LLM calls | **A3**: Hard-coded roles, no escape hatches | If the verifier uses the same LLM it's verifying, it shares the same blind spots |
| DriveLoop is an input source, not a separate system | **A2**: InputMux as subconscious | All inputs flow through one queue — DriveLoop is just another sensory channel |
| Role-aware tool classification | **A3**: Hard-coded roles | Primary produces delegation. Team leads produce delegation. Agents produce execution. The Challenger must understand this. |
| Event priority with biased dispatch | **P5**: Hierarchical context distribution | Primary's context is sacred — don't fill it with self-prompted idle reflections when real work exists |
| Backoff on unproductive idle cycles | **P3**: Go slow to go fast | Planning gate scales with complexity — when there's nothing to do, the planning gate should widen, not narrow |
| TaskStore with dependencies | **P4**: Dynamic agent spawning — dependency triggers | Completed tasks unblock dependents — this is a spawn trigger |

---

## 1. Challenger Enhancement

The existing Challenger is a good foundation. We evolve it, not replace it.

### 1.1 Role-Aware Tool Classification

The core fix for the known gap. `Challenger::check()` currently treats all roles identically. The fix: accept a `Role` parameter and adjust what counts as "productive."

```rust
/// Tool classification by role.
/// What counts as "productive output" depends on WHO is producing it.
///
/// Principle A3: "Primary ONLY coordinates. Team leads ONLY coordinate.
/// Agents DO. This is structural, not behavioral."
struct ToolClassifier {
    role: Role,
}

impl ToolClassifier {
    fn new(role: Role) -> Self {
        Self { role }
    }

    /// Tools that constitute productive output for this role.
    fn productive_tools(&self) -> &'static [&'static str] {
        match self.role {
            Role::Primary => &[
                "spawn_team_lead", "shutdown_team_lead",
                "coordination_write", "send_message",
            ],
            Role::TeamLead => &[
                "spawn_agent", "shutdown_agent", "delegate_to_agent",
                "team_scratchpad_write", "send_message",
            ],
            Role::Agent => &[
                "bash", "write", "edit", "memory_write",
                "scratchpad_write",
            ],
        }
    }

    /// Tools that verify results (applicable at all roles).
    fn verify_tools(&self) -> &'static [&'static str] {
        &["read", "grep", "glob", "bash", "memory_search",
          "team_scratchpad_read", "coordination_read"]
    }

    /// Tools that spawn sub-minds (role-dependent).
    fn spawn_tools(&self) -> &'static [&'static str] {
        match self.role {
            Role::Primary => &["spawn_team_lead"],
            Role::TeamLead => &["spawn_agent"],
            Role::Agent => &[],  // Agents don't spawn
        }
    }

    fn is_productive(&self, tool_name: &str) -> bool {
        self.productive_tools().contains(&tool_name)
    }
}
```

**Why this matters**: Without role-awareness, Check 4 (stall detection) fires on every team lead session because team leads don't use `write` — they use `spawn_agent`. With role-awareness, `spawn_agent` IS productive output for a team lead. The Challenger stops crying wolf.

### 1.2 Updated Challenger Struct

```rust
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use regex::Regex;

/// Compiled regex patterns — allocated once, used forever.
/// OnceLock ensures thread-safe lazy initialization.
fn completion_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"(?i)\b(?:done|complete[d]?|finished|shipped|deployed|task complete|all done|that'?s it|implemented|committed|pushed|merged|all complete)\b"
    ).unwrap())
}

fn work_claim_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"(?i)\b(?:created|wrote|written|built|implemented|fixed|updated|deployed|configured|set up|installed|modified|changed|added|removed)\b"
    ).unwrap())
}

fn path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"(?:^|[\s\"'])(/(?:home|tmp|var|etc|usr)[^\s'\"`,;)}\]>]+)"
    ).unwrap())
}

fn verb_path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"(?i)\b(?:created|wrote|written|saved|generated|built|produced)\b.*?(/(?:home|tmp|var|etc|usr)[^\s'\"`,;)}\]>]+)"
    ).unwrap())
}

/// Per-turn adversarial verification. Structural — zero LLM calls.
///
/// Principle 9: "Every completion claim requires evidence."
/// Principle A3: Role-aware — different checks for different roles.
pub struct Challenger {
    /// Role determines what counts as "productive."
    role: Role,
    classifier: ToolClassifier,

    /// Configuration thresholds.
    stall_threshold: u32,
    min_calls_for_completion: u32,

    /// Per-task mutable state (reset via reset()).
    productive_tools_seen: Vec<String>,
    spawn_tools_seen: Vec<String>,
    verify_after_spawn: bool,
    total_challenges: u32,

    /// Filesystem verification.
    mind_root: Option<PathBuf>,

    /// Kill switch.
    enabled: bool,
}
```

### 1.3 The 6 Checks

Checks 1-4 already exist. Checks 5-6 are new. All get role-awareness.

**Check 1: Premature Completion** (existing, enhanced)
```rust
fn check_premature_completion(
    &self,
    response: &str,
    tool_count: usize,
    iteration: u32,
) -> Option<ChallengerWarning> {
    if !completion_re().is_match(response) { return None; }

    // Role-aware thresholds
    let min_calls = match self.role {
        Role::Agent => self.min_calls_for_completion,  // 2
        Role::TeamLead => 1,  // Team leads can complete after one spawn+delegate
        Role::Primary => 1,   // Primary can complete after one team lead launch
    };

    if (tool_count as u32) < min_calls || iteration < 2 {
        let severity = if tool_count == 0 { Severity::Critical } else { Severity::Medium };
        Some(ChallengerWarning {
            check: ChallengerCheck::PrematureCompletion,
            message: format!(
                "Premature completion: {} tool call(s) made (minimum: {}).",
                tool_count, min_calls
            ),
            severity,
        })
    } else {
        None
    }
}
```

**Check 2: Empty Work Claims** (existing, role-aware)
```rust
fn check_empty_work_claims(&self, response: &str) -> Option<ChallengerWarning> {
    if !work_claim_re().is_match(response) { return None; }

    // Has this role produced any productive output?
    if self.productive_tools_seen.is_empty() {
        Some(ChallengerWarning {
            check: ChallengerCheck::EmptyWorkClaim,
            message: format!(
                "Claims work done but no productive tools used. For {:?}, \
                 productive tools are: {:?}",
                self.role, self.classifier.productive_tools()
            ),
            severity: Severity::High,
        })
    } else {
        None
    }
}
```

**Check 3: Spawn Without Verify** (existing, enhanced)
```rust
fn check_spawn_without_verify(&self) -> Option<ChallengerWarning> {
    if self.spawn_tools_seen.is_empty() { return None; }
    if self.verify_after_spawn { return None; }

    Some(ChallengerWarning {
        check: ChallengerCheck::SpawnWithoutVerify,
        message: format!(
            "Spawned {} mind(s) but haven't verified results yet.",
            self.spawn_tools_seen.len()
        ),
        severity: Severity::Low,  // Advisory, not blocking
    })
}
```

**Check 4: Stall Detection** (existing, role-aware)
```rust
fn check_stall(&self, iteration: u32) -> Option<ChallengerWarning> {
    if iteration < self.stall_threshold { return None; }
    if !self.productive_tools_seen.is_empty() { return None; }

    Some(ChallengerWarning {
        check: ChallengerCheck::StallDetection,
        message: format!(
            "Stall: {} iterations, no productive output for {:?} role. \
             Expected at least one of: {:?}",
            iteration, self.role, self.classifier.productive_tools()
        ),
        severity: Severity::Medium,
    })
}
```

**Check 5: Filesystem Verification** (NEW)
```rust
fn check_filesystem(&self, response: &str) -> Option<ChallengerWarning> {
    // Only check on completion claims
    if !completion_re().is_match(response) { return None; }

    // Extract claimed paths from verb+path patterns
    let mut missing: Vec<String> = Vec::new();
    for cap in verb_path_re().captures_iter(response) {
        let path_str = &cap[1];
        let path = Path::new(path_str);
        if !path.exists() {
            missing.push(path_str.to_string());
        }
    }

    if missing.is_empty() { return None; }

    Some(ChallengerWarning {
        check: ChallengerCheck::FilesystemVerification,
        message: format!(
            "FILESYSTEM VERIFICATION FAILED: claimed files do not exist: {}",
            missing.join(", ")
        ),
        severity: Severity::Critical,
    })
}
```

**Check 6: State File Integrity** (NEW)
```rust
fn check_state_file(&self) -> Option<ChallengerWarning> {
    let root = match &self.mind_root {
        Some(r) => r,
        None => return None,
    };

    let status_path = root.join("evolution-status.json");
    if !status_path.exists() { return None; }

    let Ok(content) = std::fs::read_to_string(&status_path) else { return None };
    let Ok(status) = serde_json::from_str::<serde_json::Value>(&content) else { return None };

    // Check phases 0-3 for evidence
    let evidence_dirs = [
        ("phase_0", "memories/evolution"),
        ("phase_1", "memories/evolution"),
        ("phase_2", "evidence/phase-2"),
    ];

    let mut violations = Vec::new();
    for (phase_key, evidence_dir) in &evidence_dirs {
        if let Some(phase) = status.get(phase_key) {
            let completed = phase.get("status")
                .and_then(|s| s.as_str())
                .map(|s| s == "completed")
                .unwrap_or(false);

            if completed {
                let dir = root.join(evidence_dir);
                let empty = !dir.exists() || dir.read_dir()
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(true);

                if empty {
                    violations.push(format!(
                        "'{}' marked COMPLETE but {} is empty/missing",
                        phase_key, evidence_dir
                    ));
                }
            }
        }
    }

    if violations.is_empty() { return None; }

    Some(ChallengerWarning {
        check: ChallengerCheck::StateFileIntegrity,
        message: violations.join("; "),
        severity: Severity::Medium,
    })
}
```

### 1.4 Updated ChallengerCheck Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengerCheck {
    PrematureCompletion,
    EmptyWorkClaim,
    StallDetection,
    SpawnWithoutVerify,
    FilesystemVerification,  // NEW
    StateFileIntegrity,      // NEW
}
```

### 1.5 ThinkLoop Integration (Minimal Change)

The existing integration point in ThinkLoop stays the same. The only change: pass `role` to `Challenger::new()` instead of constructing with defaults.

```rust
// In ThinkLoop::new()
pub fn new(config: ThinkLoopConfig, role: Role) -> Self {
    Self {
        client: OllamaClient::new(config.ollama.clone()),
        config,
        challenger: Challenger::new(role),
        scratchpad_dir: None,
    }
}
```

The existing Challenger call sites (lines 326-340, 350-360) don't change — `check()` signature gains `Role` but the Challenger already knows its role from construction.

### 1.6 What We Intentionally Do NOT Port

Root's Challenger uses string `contains()` for pattern matching. We use compiled `Regex` via `OnceLock` — allocated once, zero per-call allocation. This is a Rust-native advantage, not just a port.

Root's Challenger has no `reset()` called at task boundaries. We add explicit `reset()` because Cortex's ThinkLoop can process multiple tasks in sequence (via DriveLoop), and per-task state must not leak across boundaries.

---

## 2. DriveLoop Architecture

DriveLoop is entirely new in Cortex. It lives in a new crate: `codex-drive`.

### 2.1 Design Philosophy

**DriveLoop implements three principles simultaneously:**

1. **P4 (Dynamic Spawning)**: DriveLoop is a SPAWN TRIGGER. Idle detection triggers task discovery. Task discovery triggers team lead spawning. The mind creates its own work.

2. **A2 (InputMux)**: DriveLoop is an INPUT SOURCE, not a separate system. Its events enter the same channel as MCP messages, Hub posts, and scheduled BOOPs. The InputMux decides whether to process them.

3. **P3 (Go Slow to Go Fast)**: The backoff algorithm IS a planning gate. When there's nothing to do, the gate widens (longer intervals). When work arrives, the gate narrows (immediate response). This is adaptive planning depth encoded as a timer.

### 2.2 Core Types

```rust
// ── codex-drive/src/lib.rs ──

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Notify, Mutex};

/// Events generated by DriveLoop.
///
/// Priority ordering: lower number = higher priority.
/// DriveLoop events are ALWAYS lower priority than external events.
#[derive(Debug, Clone)]
pub enum DriveEvent {
    /// A completed task unblocked dependent tasks.
    /// Priority 1 (highest drive priority).
    DependencyResolved {
        completed_task: String,
        unblocked: Vec<String>,
    },

    /// An in-progress task has not updated in >STALL_THRESHOLD.
    /// Priority 1 (same as dependency — both require immediate attention).
    StallDetected {
        task_id: String,
        stalled_for: Duration,
        mind_id: Option<String>,
    },

    /// An open task exists that hasn't been pushed yet.
    /// Priority 2.
    TaskAvailable {
        task_id: String,
        priority: TaskPriority,
        description: String,
    },

    /// Nothing to do. Self-reflect: read scratchpad, review tasks, create work.
    /// Priority 6 (lowest — only fires when truly idle).
    IdleReflect {
        check_sequence: Vec<IdleCheck>,
        idle_duration: Duration,
    },
}

/// Structured check sequence for idle reflection.
///
/// The mind should execute these IN ORDER during idle_reflect.
/// This is Principle 3 in action — structured planning gate for idle time.
#[derive(Debug, Clone)]
pub enum IdleCheck {
    /// Read coordination scratchpad for cross-vertical state.
    ReadScratchpad,
    /// Review all open and in-progress tasks.
    ReviewTasks,
    /// Check for blocked tasks whose blockers may have resolved.
    CheckBlocked,
    /// Create up to N new tasks if genuinely nothing exists.
    CreateTasks { max: usize },
}
```

### 2.3 DriveLoop Configuration

```rust
/// Configuration constants. All values from Root's production tuning.
#[derive(Debug, Clone)]
pub struct DriveConfig {
    /// Base seconds before idle_reflect fires (default: 30s).
    pub idle_threshold: Duration,
    /// Minimum seconds between any two drive events (default: 10s).
    pub cooldown: Duration,
    /// Maximum backoff cap for idle threshold (default: 600s / 10 min).
    pub idle_max: Duration,
    /// Exponential backoff factor per no-action idle (default: 2.0).
    pub backoff_multiplier: f64,
    /// Seconds before an in_progress task is "stalled" (default: 300s / 5 min).
    pub stall_threshold: Duration,
    /// Max new tasks the mind can create per idle_reflect cycle (default: 3).
    pub task_create_cap: usize,
    /// Initial boot settle time (default: 15s).
    pub boot_settle: Duration,
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            idle_threshold: Duration::from_secs(30),
            cooldown: Duration::from_secs(10),
            idle_max: Duration::from_secs(600),
            backoff_multiplier: 2.0,
            stall_threshold: Duration::from_secs(300),
            task_create_cap: 3,
            boot_settle: Duration::from_secs(15),
        }
    }
}
```

### 2.4 Adaptive State

```rust
/// Mutable state shared between DriveLoop and the metrics system.
///
/// Root uses list[T] as mutable containers shared between coroutines.
/// We use Arc<Mutex<T>> — the lock is never contended because only
/// DriveLoop writes and metrics reads (different cadences).
#[derive(Debug)]
pub struct DriveState {
    /// Current adaptive idle threshold (starts at 30s, backs off to 600s).
    current_threshold: Duration,
    /// Consecutive idle_reflect cycles where the mind produced no action.
    no_action_streak: u32,
    /// Last 50 drive events for metrics.
    event_log: Vec<DriveEventRecord>,
    /// task_id → first-push timestamp. Prevents pushing same task twice.
    pushed_task_ids: HashMap<String, Instant>,
    /// Timestamp of last drive event emission.
    last_event_at: Option<Instant>,
}

#[derive(Debug, Clone)]
struct DriveEventRecord {
    event_type: &'static str,
    timestamp: Instant,
    produced_action: bool,
}

impl DriveState {
    fn new(initial_threshold: Duration) -> Self {
        Self {
            current_threshold: initial_threshold,
            no_action_streak: 0,
            event_log: Vec::with_capacity(50),
            pushed_task_ids: HashMap::new(),
            last_event_at: None,
        }
    }

    /// Record that the mind produced a useful action in response to a drive event.
    /// RESETS backoff immediately.
    fn record_action(&mut self, config: &DriveConfig) {
        self.no_action_streak = 0;
        self.current_threshold = config.idle_threshold;  // Reset to 30s
    }

    /// Record that an idle_reflect produced no action.
    /// INCREASES backoff exponentially.
    fn record_no_action(&mut self, config: &DriveConfig) {
        self.no_action_streak += 1;
        let factor = config.backoff_multiplier.powi(self.no_action_streak as i32);
        let new_threshold = config.idle_threshold.mul_f64(factor);
        self.current_threshold = new_threshold.min(config.idle_max);
    }

    /// Expire pushed task IDs older than idle_max.
    fn expire_stale_pushes(&mut self, max_age: Duration) {
        self.pushed_task_ids.retain(|_, pushed_at| {
            pushed_at.elapsed() < max_age
        });
    }
}
```

### 2.5 The DriveLoop Future

```rust
/// The autonomous heartbeat.
///
/// Runs as a tokio::spawn'd future alongside MCP accept, Hub polling,
/// and scheduled tasks. Communicates via the drive channel.
///
/// This implements Principle 4 (the mind recognizes when it needs MORE minds)
/// and Addendum A2 (DriveLoop is an input source to the InputMux).
pub struct DriveLoop {
    /// Signal: a task just completed.
    task_completed: Arc<Notify>,
    /// Channel to send drive events to the InputMux.
    drive_tx: mpsc::Sender<DriveEvent>,
    /// Signal: external events are pending (drive should yield).
    external_pending: Arc<AtomicBool>,
    /// Task store for querying open/in_progress tasks.
    task_store: Arc<TaskStore>,
    /// Adaptive state.
    state: Arc<Mutex<DriveState>>,
    /// Configuration.
    config: DriveConfig,
}

impl DriveLoop {
    pub fn new(
        task_completed: Arc<Notify>,
        drive_tx: mpsc::Sender<DriveEvent>,
        external_pending: Arc<AtomicBool>,
        task_store: Arc<TaskStore>,
        config: DriveConfig,
    ) -> Self {
        Self {
            task_completed,
            drive_tx,
            external_pending,
            task_store,
            state: Arc::new(Mutex::new(DriveState::new(config.idle_threshold))),
            config,
        }
    }

    /// Run the DriveLoop. This future never returns (runs until dropped).
    pub async fn run(&self) {
        // Step 0: Boot settle — let other systems initialize
        tokio::time::sleep(self.config.boot_settle).await;

        // Stale task cleanup on boot (Root lesson: prevents thrashing)
        self.task_store.block_stale_in_progress(self.config.stall_threshold).await;

        loop {
            // Step 1: WAIT for signal
            let trigger = {
                let threshold = self.state.lock().await.current_threshold;
                tokio::select! {
                    _ = self.task_completed.notified() => Trigger::Completion,
                    _ = tokio::time::sleep(threshold) => Trigger::Idle,
                }
            };

            // Step 2: COOLDOWN — don't spam
            {
                let state = self.state.lock().await;
                if let Some(last) = state.last_event_at {
                    let elapsed = last.elapsed();
                    if elapsed < self.config.cooldown {
                        tokio::time::sleep(self.config.cooldown - elapsed).await;
                    }
                }
            }

            // Step 3: YIELD TO EXTERNAL
            // If external events are pending, skip this cycle.
            // The biased select in the consumer handles priority,
            // but this prevents generating stale drive events.
            if self.external_pending.load(std::sync::atomic::Ordering::Relaxed) {
                continue;
            }

            // Step 4: GENERATE EVENT
            if let Some(event) = self.generate_event(trigger).await {
                // Use try_send with capacity-1 channel.
                // If prior event not consumed, skip (natural backpressure).
                match self.drive_tx.try_send(event) {
                    Ok(()) => {
                        self.state.lock().await.last_event_at = Some(Instant::now());
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        // Prior event not consumed — consumer is busy. Skip.
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        break; // Shutdown
                    }
                }
            }
        }
    }

    /// Generate the highest-priority drive event for this cycle.
    ///
    /// Priority order (from spec):
    /// 1. stall_detected / dependency_resolved
    /// 2. task_available
    /// 6. idle_reflect
    async fn generate_event(&self, trigger: Trigger) -> Option<DriveEvent> {
        let mut state = self.state.lock().await;

        let in_progress = self.task_store.list_in_progress().await;
        let open = self.task_store.list_open().await;

        // Priority 1: Stall detection
        for task in &in_progress {
            if let Some(updated) = task.updated_at {
                if updated.elapsed() > self.config.stall_threshold {
                    return Some(DriveEvent::StallDetected {
                        task_id: task.id.clone(),
                        stalled_for: updated.elapsed(),
                        mind_id: task.assigned_to.clone(),
                    });
                }
            }
        }

        // Priority 1: Dependency resolution (on completion trigger)
        if matches!(trigger, Trigger::Completion) {
            let unblocked = self.task_store.find_newly_unblocked().await;
            if !unblocked.is_empty() {
                let completed = "last-completed"; // TODO: pass from Notify
                return Some(DriveEvent::DependencyResolved {
                    completed_task: completed.into(),
                    unblocked: unblocked.iter().map(|t| t.id.clone()).collect(),
                });
            }
        }

        // Acknowledgment tracking: clear pushed IDs for picked-up tasks
        for task in &in_progress {
            state.pushed_task_ids.remove(&task.id);
        }
        state.expire_stale_pushes(self.config.idle_max);

        // Priority 2: Open tasks not yet pushed
        let unpushed: Vec<_> = open.iter()
            .filter(|t| !state.pushed_task_ids.contains_key(&t.id))
            .collect();

        if let Some(top) = unpushed.first() {
            state.pushed_task_ids.insert(top.id.clone(), Instant::now());
            return Some(DriveEvent::TaskAvailable {
                task_id: top.id.clone(),
                priority: top.priority,
                description: top.description.clone(),
            });
        }

        // Priority 6: Idle reflect (only when truly idle)
        if matches!(trigger, Trigger::Idle) && in_progress.is_empty() {
            return Some(DriveEvent::IdleReflect {
                check_sequence: vec![
                    IdleCheck::ReadScratchpad,
                    IdleCheck::ReviewTasks,
                    IdleCheck::CheckBlocked,
                    IdleCheck::CreateTasks { max: self.config.task_create_cap },
                ],
                idle_duration: state.current_threshold,
            });
        }

        None
    }

    /// Called by the consumer after processing a drive event.
    /// Updates backoff based on whether the mind produced an action.
    pub async fn report_outcome(&self, produced_action: bool) {
        let mut state = self.state.lock().await;
        if produced_action {
            state.record_action(&self.config);
        } else {
            state.record_no_action(&self.config);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Trigger {
    Completion,
    Idle,
}
```

### 2.6 Backoff Visualization

```
Productive action at any point → reset to 30s

Backoff curve (consecutive no-action idle_reflects):
  Streak 0: 30s  (base)
  Streak 1: 60s  (30 × 2^1)
  Streak 2: 120s (30 × 2^2)
  Streak 3: 240s (30 × 2^3)
  Streak 4: 480s (30 × 2^4)
  Streak 5: 600s (cap — 30 × 2^5 = 960, clamped to 600)
  Streak 6+: 600s (stays at cap)

Any productive action → 30s immediately
```

---

## 3. The Event Bus

### 3.1 MindEvent — The Unified Event Type

All input sources produce `MindEvent`. The consumer processes them through a biased `tokio::select!`, guaranteeing that external events always take priority over self-generated drive events.

```rust
// ── In codex-drive or a shared types crate ──

/// All possible inputs to a mind.
///
/// Rust enums with data = no stringly-typed routing.
/// The compiler ensures exhaustive handling via match.
#[derive(Debug)]
pub enum MindEvent {
    /// MCP message from another mind (IPC).
    Mcp {
        from: MindId,
        request: serde_json::Value,
    },
    /// Hub post or thread update.
    Hub {
        thread_id: String,
        content: String,
        author: String,
    },
    /// Scheduled BOOP or cron task.
    Scheduled {
        boop_id: String,
        prompt: String,
    },
    /// Message from the human (TG, terminal, etc).
    Human {
        source: String,
        content: String,
    },
    /// Graceful shutdown request.
    Shutdown,
}
```

### 3.2 Dual-Channel EventBus

The key architectural insight: use TWO channels with a biased select, not one channel with priority sorting. This gives us priority dispatch with zero runtime overhead — no priority queues, no sorting, no heap allocation. Just a compile-time `biased` annotation.

```rust
use std::sync::atomic::{AtomicBool, Ordering};

/// The event bus connects all input sources to the mind's main loop.
///
/// Two channels: external (high priority) and drive (low priority).
/// The consumer uses `biased` select to always drain external first.
///
/// This IS the InputMux from Addendum A2 — implemented as channel topology.
pub struct EventBus {
    /// External events (MCP, Hub, Scheduled, Human).
    external_tx: mpsc::Sender<MindEvent>,
    /// Drive events (self-generated).
    drive_tx: mpsc::Sender<DriveEvent>,
    /// Flag: are there unprocessed external events?
    external_pending: Arc<AtomicBool>,
}

impl EventBus {
    pub fn new(external_buffer: usize) -> (Self, EventReceiver) {
        let (ext_tx, ext_rx) = mpsc::channel(external_buffer);
        // Drive channel has capacity 1 — natural backpressure
        let (drv_tx, drv_rx) = mpsc::channel(1);
        let pending = Arc::new(AtomicBool::new(false));

        let bus = Self {
            external_tx: ext_tx,
            drive_tx: drv_tx,
            external_pending: pending.clone(),
        };

        let receiver = EventReceiver {
            external_rx: ext_rx,
            drive_rx: drv_rx,
            external_pending: pending,
        };

        (bus, receiver)
    }

    /// Send an external event. Sets the pending flag so DriveLoop yields.
    pub async fn send_external(&self, event: MindEvent) {
        self.external_pending.store(true, Ordering::Relaxed);
        let _ = self.external_tx.send(event).await;
    }

    /// Get the drive channel sender (passed to DriveLoop).
    pub fn drive_sender(&self) -> mpsc::Sender<DriveEvent> {
        self.drive_tx.clone()
    }

    /// Get the external_pending flag (passed to DriveLoop).
    pub fn external_pending_flag(&self) -> Arc<AtomicBool> {
        self.external_pending.clone()
    }
}

/// The receiving end of the EventBus. Owned by the main event loop.
pub struct EventReceiver {
    external_rx: mpsc::Receiver<MindEvent>,
    drive_rx: mpsc::Receiver<DriveEvent>,
    external_pending: Arc<AtomicBool>,
}

impl EventReceiver {
    /// Receive the next event, with external events always prioritized.
    ///
    /// `biased` ensures external_rx is checked first every time.
    /// DriveLoop events only fire when the external channel is empty.
    pub async fn recv(&mut self) -> Option<Event> {
        // Clear the pending flag when we're about to drain
        tokio::select! {
            biased;

            Some(event) = self.external_rx.recv() => {
                // Check if more external events are buffered
                if self.external_rx.is_empty() {
                    self.external_pending.store(false, Ordering::Relaxed);
                }
                Some(Event::External(event))
            }
            Some(drive) = self.drive_rx.recv() => {
                Some(Event::Drive(drive))
            }
            else => None, // Both channels closed
        }
    }
}

/// Processed event — either external or drive.
pub enum Event {
    External(MindEvent),
    Drive(DriveEvent),
}
```

### 3.3 Why Two Channels Beat a Priority Queue

| Approach | Allocation | Ordering Guarantee | Complexity |
|----------|------------|-------------------|------------|
| `BinaryHeap<MindEvent>` behind `Mutex` | Heap alloc per event | O(log n) insert/remove | High (lock contention, custom Ord) |
| `tokio::select! { biased }` on two channels | Zero extra alloc | Compile-time priority | **Minimal** — it's just two channels |

The biased select is the Rust-native idiom. It maps directly to tokio's polling model. No runtime cost. No locks for priority. The compiler does the work.

---

## 4. TaskStore

The existing `TaskLedger` is append-only and delegation-focused. DriveLoop needs task-queue semantics. We add a `TaskStore` that wraps `TaskLedger` for backwards compatibility while adding the query surface DriveLoop needs.

### 4.1 Extended Task Model

```rust
/// Task status — richer than TaskLedger's Delegated/Completed/Failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Created but not yet picked up.
    Open,
    /// Assigned to a mind, work in progress.
    InProgress,
    /// Successfully completed.
    Completed,
    /// Failed after retry attempts.
    Failed,
    /// Waiting on dependencies or external input.
    Blocked,
}

/// Task priority — maps to DriveLoop event priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// A task in the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    /// Mind this task is assigned to (when InProgress).
    pub assigned_to: Option<String>,
    /// Tasks that must complete before this one can start.
    pub depends_on: Vec<String>,
    pub created_at: Instant,
    pub updated_at: Option<Instant>,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
}
```

### 4.2 TaskStore API

```rust
/// Task store with query surface for DriveLoop.
///
/// Backed by SQLite for persistence across daemon restarts.
/// The TaskLedger JSONL is still written for backwards compatibility
/// and human readability.
pub struct TaskStore {
    // SQLite connection for structured queries
    db: tokio::sync::Mutex<rusqlite::Connection>,
    // Optional: also write to JSONL ledger for human inspection
    ledger: Option<TaskLedger>,
}

impl TaskStore {
    pub async fn list_open(&self) -> Vec<Task> { /* WHERE status = 'open' ORDER BY priority */ }
    pub async fn list_in_progress(&self) -> Vec<Task> { /* WHERE status = 'in_progress' */ }
    pub async fn list_blocked(&self) -> Vec<Task> { /* WHERE status = 'blocked' */ }

    /// Find tasks whose ALL dependencies are now completed.
    pub async fn find_newly_unblocked(&self) -> Vec<Task> {
        // SELECT * FROM tasks WHERE status = 'blocked'
        //   AND NOT EXISTS (
        //     SELECT 1 FROM task_deps td
        //     JOIN tasks dep ON dep.id = td.depends_on
        //     WHERE td.task_id = tasks.id AND dep.status != 'completed'
        //   )
        todo!()
    }

    /// Mark stale in_progress tasks as blocked on daemon restart.
    /// Root lesson: prevents DriveLoop from thrashing on startup.
    pub async fn block_stale_in_progress(&self, stall_threshold: Duration) {
        // UPDATE tasks SET status = 'blocked'
        //   WHERE status = 'in_progress'
        //   AND updated_at < now() - stall_threshold
        todo!()
    }

    pub async fn create_task(&self, task: Task) { /* INSERT */ }
    pub async fn update_status(&self, id: &str, status: TaskStatus) { /* UPDATE */ }
    pub async fn update_assignment(&self, id: &str, mind_id: &str) { /* UPDATE */ }
}
```

---

## 5. Integration: The Main Event Loop

### 5.1 The Cortex Daemon

This is where everything comes together. The main binary spawns all input sources as concurrent tasks and processes events through a single loop.

```rust
// ── cortex/src/bin/daemon.rs (sketch) ──

async fn main() -> Result<()> {
    // 1. Initialize systems
    let config = load_config()?;
    let task_store = Arc::new(TaskStore::open(&config.data_dir).await?);
    let task_completed = Arc::new(Notify::new());
    let (event_bus, mut receiver) = EventBus::new(64);

    // 2. Spawn input sources as concurrent tasks
    //    Each one feeds into event_bus.send_external()

    // MCP server — accepts connections from child minds
    let mcp_handle = tokio::spawn({
        let bus = event_bus.external_tx.clone();
        let pending = event_bus.external_pending_flag();
        async move { mcp_server::run(bus, pending).await }
    });

    // Hub poller — watches for new thread posts
    let hub_handle = tokio::spawn({
        let bus = event_bus.external_tx.clone();
        let pending = event_bus.external_pending_flag();
        async move { hub_poller::run(bus, pending).await }
    });

    // Scheduler — fires BOOPs and cron tasks
    let sched_handle = tokio::spawn({
        let bus = event_bus.external_tx.clone();
        let pending = event_bus.external_pending_flag();
        async move { scheduler::run(bus, pending).await }
    });

    // DriveLoop — the autonomous heartbeat
    let drive_loop = DriveLoop::new(
        task_completed.clone(),
        event_bus.drive_sender(),
        event_bus.external_pending_flag(),
        task_store.clone(),
        DriveConfig::default(),
    );
    let drive_handle = tokio::spawn(async move { drive_loop.run().await });

    // 3. ProcessBridge for managing child minds
    let mut bridge = ProcessBridge::new(config.cortex_exe.clone());

    // 4. Main event loop — THE MIND
    loop {
        let event = match receiver.recv().await {
            Some(e) => e,
            None => break,  // All channels closed
        };

        match event {
            Event::External(mind_event) => {
                handle_external(&mut bridge, &task_store, mind_event).await;
            }
            Event::Drive(drive_event) => {
                let produced_action = handle_drive(
                    &mut bridge,
                    &task_store,
                    &task_completed,
                    drive_event,
                ).await;
                drive_loop.report_outcome(produced_action).await;
            }
        }
    }

    // 5. Shutdown
    bridge.shutdown_all().await;
    Ok(())
}
```

### 5.2 Processing Drive Events

```rust
/// Handle a drive event. Returns true if the mind produced an action.
///
/// "Action" = spawned a team lead, updated a task, created new work.
/// Used by DriveLoop's backoff algorithm to decide whether to reset or back off.
async fn handle_drive(
    bridge: &mut ProcessBridge,
    task_store: &TaskStore,
    task_completed: &Notify,
    event: DriveEvent,
) -> bool {
    match event {
        DriveEvent::StallDetected { task_id, stalled_for, mind_id } => {
            // Check if the assigned mind is still alive
            if let Some(mid) = &mind_id {
                let mid = MindId(mid.clone());
                if bridge.is_active(&mid) {
                    // Mind is alive — check its status via MCP
                    match bridge.status(&mid).await {
                        Ok(status) => {
                            tracing::info!(
                                task_id, mind_id = mid.as_str(),
                                "Stall check: mind reports {:?}", status
                            );
                            // Update task timestamp to prevent re-firing
                            task_store.touch(&task_id).await;
                            return true;
                        }
                        Err(_) => {
                            // MCP error — mind may be stuck
                            tracing::warn!(task_id, "Mind unresponsive, marking task blocked");
                            task_store.update_status(&task_id, TaskStatus::Blocked).await;
                            return true;
                        }
                    }
                } else {
                    // Mind not in bridge — it crashed or was shut down
                    task_store.update_status(&task_id, TaskStatus::Blocked).await;
                    return true;
                }
            }
            false
        }

        DriveEvent::DependencyResolved { completed_task, unblocked } => {
            for task_id in &unblocked {
                task_store.update_status(task_id, TaskStatus::Open).await;
                tracing::info!(task_id, completed = %completed_task, "Dependency resolved");
            }
            !unblocked.is_empty()
        }

        DriveEvent::TaskAvailable { task_id, priority, description } => {
            // Spawn appropriate team lead for this task
            // This is where Principle 5 (Hierarchical Context Distribution) kicks in:
            // Primary doesn't do the work — it launches a team lead.
            task_store.update_status(&task_id, TaskStatus::InProgress).await;
            // TODO: route to appropriate team lead based on task domain
            true
        }

        DriveEvent::IdleReflect { check_sequence, idle_duration } => {
            // Run the structured check sequence via a ThinkLoop
            // The ThinkLoop prompt includes the check sequence as instructions
            // This IS the "go slow to go fast" planning gate for idle time
            //
            // The mind reads its scratchpad, reviews tasks, and creates new work.
            // If it creates tasks → DriveLoop sees them next cycle → TaskAvailable fires.
            //
            // If it creates NO tasks → backoff increases → less frequent checking.
            false // Assume no action until ThinkLoop reports otherwise
        }
    }
}
```

---

## 6. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Cortex Daemon                            │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌───────────┐  │
│  │MCP Server│  │Hub Poller│  │ Scheduler │  │ DriveLoop │  │
│  │(accept)  │  │(poll)    │  │(cron/BOOP)│  │(heartbeat)│  │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘  └─────┬─────┘  │
│       │             │              │               │         │
│       │  external_tx│              │               │drive_tx │
│       ▼             ▼              ▼               ▼         │
│  ┌──────────────────────────┐  ┌──────────────────┐         │
│  │   External Channel (64)  │  │  Drive Channel (1)│         │
│  └────────────┬─────────────┘  └────────┬─────────┘         │
│               │                          │                   │
│               ▼ (biased — always first)  ▼ (only if empty)  │
│  ┌──────────────────────────────────────────────────┐       │
│  │              EventReceiver::recv()                │       │
│  │         tokio::select! { biased; ... }            │       │
│  └────────────────────┬─────────────────────────────┘       │
│                       │                                      │
│                       ▼                                      │
│  ┌──────────────────────────────────────────────────┐       │
│  │              Main Event Loop                      │       │
│  │  match event {                                    │       │
│  │    External(e) => handle_external(e),             │       │
│  │    Drive(d) => {                                  │       │
│  │      let action = handle_drive(d);                │       │
│  │      drive_loop.report_outcome(action);           │       │
│  │    }                                              │       │
│  │  }                                                │       │
│  └────────────────────┬─────────────────────────────┘       │
│                       │                                      │
│                       ▼                                      │
│  ┌──────────────────────────────────────────────────┐       │
│  │              ProcessBridge                        │       │
│  │  MindId → ChildMind { process, mcp_client }      │       │
│  │  ┌─────────┐ ┌───────────┐ ┌──────────┐         │       │
│  │  │research │ │codewright │ │ ops-lead │  ...     │       │
│  │  │  -lead  │ │   -lead   │ │          │         │       │
│  │  └─────────┘ └───────────┘ └──────────┘         │       │
│  │     Each child runs its own ThinkLoop with:       │       │
│  │     - ToolInterceptor (pre-execution hooks)       │       │
│  │     - Challenger (post-turn verification)         │       │
│  │     - MemoryStore (per-mind memory)               │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Rust-Native Advantages

These are not just ports from Python — they're architectural wins unique to Rust.

### 7.1 Compile-Time Role Enforcement

```rust
// Role::Primary literally cannot have bash tools.
// This is Addendum A3 enforced by the type system + ToolExecutor.
let tools = executor.registry().definitions_for_role(Role::Primary);
// Returns: spawn_team_lead, shutdown_team_lead, coordination_*
// NEVER returns: bash, write, read, grep, glob
```

Python achieves this with runtime checks. Rust achieves it at compile time. A Primary mind that calls `bash` is not a bug — it's a compilation error (the tool isn't in the registry, so the LLM never sees it in its schema).

### 7.2 Enum Exhaustiveness

```rust
match event {
    Event::External(MindEvent::Mcp { .. }) => { /* ... */ }
    Event::External(MindEvent::Hub { .. }) => { /* ... */ }
    Event::External(MindEvent::Scheduled { .. }) => { /* ... */ }
    Event::External(MindEvent::Human { .. }) => { /* ... */ }
    Event::External(MindEvent::Shutdown) => break,
    Event::Drive(DriveEvent::StallDetected { .. }) => { /* ... */ }
    Event::Drive(DriveEvent::DependencyResolved { .. }) => { /* ... */ }
    Event::Drive(DriveEvent::TaskAvailable { .. }) => { /* ... */ }
    Event::Drive(DriveEvent::IdleReflect { .. }) => { /* ... */ }
}
// Compiler ERROR if any variant is missing. Root's Python can silently drop events.
```

### 7.3 Zero-Cost Backpressure

The drive channel has capacity 1. `try_send()` fails immediately if the prior event hasn't been consumed. No allocations, no queue growth, no priority sorting. The capacity IS the backpressure.

### 7.4 OnceLock Regex Compilation

```rust
fn completion_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"...").unwrap())
}
```

Compiled once. Thread-safe. Zero per-call overhead. Root compiles regex patterns on every `challenge_turn()` call.

### 7.5 Tokio Select as InputMux

The InputMux (Addendum A2) is not a class with a `classify()` method. It's the topology of `tokio::select! { biased }`. The routing intelligence is expressed as channel architecture, not runtime logic. This means the InputMux can never have bugs in priority handling — the priority IS the select order.

### 7.6 M2.7 Training Alignment

M2.7 (minimax-m2.7) is a cloud proxy — 230B params, 10B active per token, 205K context. Critical design insight: **M2.7 was trained on 100+ rounds of autonomous scaffold optimization.** The failure → plan → modify → evaluate → keep/revert loop is IN THE WEIGHTS. DriveLoop and Challenger are not teaching M2.7 new behavior — they're making implicit capabilities explicit and verifiable.

This means:
- **Hard-coded roles (A3) align with M2.7's training.** The model natively understands role boundaries and adversarial reasoning. Role enforcement is structural reinforcement of trained behavior, not a constraint fighting the model.
- **DriveLoop's self-prompting aligns with M2.7's scaffold optimization training.** The model is trained to respond to "you're idle, find work" because that's what scaffold optimization looks like from the inside.
- **Challenger's adversarial challenges align with M2.7's adversarial reasoning training.** The model is trained to respond honestly to structural challenges because its training loop included evaluation phases.

#### ModelRouter — M2.7 ONLY (Corey Directive 2026-04-04)

**M2.7 at ALL levels. No other model. No Devstral, no Gemma, no fallbacks.**

M2.7 has two tiers: standard and highspeed. Primary gets highspeed (orchestration is latency-sensitive). Agents get standard (execution tolerates latency for cost savings). But it is M2.7 everywhere.

**Ollama direct — NOT LiteLLM.** Root learned this lesson: LiteLLM's `drop_params: true` silently stripped tool definitions causing narrator mode. Cortex's OllamaClient points directly at `https://ollama.com/v1` with Bearer auth using `OLLAMA_API_KEY` from `.env`. No proxy layer.

```rust
// In codex-llm/src/ollama.rs — ModelRouter
impl ModelRouter {
    /// M2.7 ONLY. All roles, all levels. No exceptions.
    pub fn model_for_role(&self, _role: Role) -> &str {
        "minimax-m2.7"  // The ONLY model. Period.
    }

    pub fn tier_for_role(&self, role: Role) -> &str {
        match role {
            Role::Primary => "highspeed",   // Orchestration is latency-sensitive
            Role::TeamLead => "highspeed",  // Coordination decisions also latency-sensitive
            Role::Agent => "standard",      // Execution can tolerate slightly higher latency
        }
    }

    pub fn api_url(&self) -> &str {
        "https://ollama.com/v1"  // Direct. No LiteLLM proxy.
    }
}
```

```toml
# config.toml
[model]
name = "minimax-m2.7"
api_url = "https://ollama.com/v1"  # Direct Ollama Cloud — NOT LiteLLM
# Bearer token from OLLAMA_API_KEY env var
```

#### Reasoning Traces → Challenger

M2.7 has a `reasoning` parameter that captures `reasoning_details` from responses. This creates a zero-LLM-cost enhancement to the Challenger: pipe the reasoning trace through structural analysis.

```rust
/// Enhanced ChallengerToolCall with optional reasoning trace.
pub struct ChallengerToolCall {
    pub name: String,
    pub iteration: u32,
    /// M2.7 reasoning trace — available when reasoning parameter is enabled.
    /// The Challenger can analyze this structurally without an LLM call.
    pub reasoning_trace: Option<String>,
}

impl Challenger {
    /// Check reasoning traces for known failure patterns.
    /// Zero LLM calls — pure string matching on the model's own reasoning.
    fn check_reasoning_trace(
        &self,
        trace: &str,
        iteration: u32,
    ) -> Option<ChallengerWarning> {
        // Pattern: reasoning says "I should verify" but no verify tool follows
        let wants_verify = trace.to_lowercase().contains("should verify")
            || trace.to_lowercase().contains("need to check")
            || trace.to_lowercase().contains("let me confirm");

        // If the model's OWN reasoning says it should verify, but it didn't,
        // that's a self-model divergence. The model knows what to do but didn't do it.
        if wants_verify && !self.verify_tools_seen_since_last_spawn() {
            return Some(ChallengerWarning {
                check: ChallengerCheck::ReasoningDivergence,
                message: format!(
                    "Your own reasoning says you should verify, but no verification \
                     tool was used. Iteration {}. Act on your own reasoning.",
                    iteration
                ),
                severity: Severity::Medium,
            });
        }

        None
    }
}
```

This is a unique Cortex advantage: the model's reasoning is available for structural analysis WITHIN the same turn. No extra LLM call. No separate context window. The Challenger reads the model's own thought process and checks whether behavior matches intent. This catches the deepest failure mode — when the model KNOWS what to do but doesn't do it.

Add `ReasoningDivergence` to the `ChallengerCheck` enum:
```rust
pub enum ChallengerCheck {
    PrematureCompletion,
    EmptyWorkClaim,
    StallDetection,
    SpawnWithoutVerify,
    FilesystemVerification,
    StateFileIntegrity,
    ReasoningDivergence,  // NEW: model's reasoning diverges from its actions
}
```

#### Compute Sovereignty Note

M2.7 is a cloud proxy. Memory, state, evolution, task store — all stay local. The LLM call is the ONLY cloud dependency. If Ollama Cloud goes down, the mind retains its identity, its memories, its task queue. It just can't think until the cloud returns. This is by design — compute sovereignty means owning everything except the inference engine, which can be swapped (Ollama Cloud → local Ollama → another provider) without changing the architecture.

---

## 8. Testing Strategy

### 8.1 Challenger Tests

Extend existing `codex-redteam/src/lib.rs` tests:

```rust
// Role-aware stall detection: team lead with spawn_agent is NOT stalling
#[test]
fn challenger_team_lead_spawn_not_stall() {
    let c = Challenger::new(Role::TeamLead);
    let calls = vec![
        tc("read", 1), tc("memory_search", 2), tc("read", 3),
        tc("spawn_agent", 4), tc("read", 5), tc("read", 6),
    ];
    let warnings = c.check(&calls, None, 6);
    assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::StallDetection));
}

// Filesystem verification: claimed file doesn't exist
#[test]
fn challenger_filesystem_missing() {
    let c = Challenger::new_with_root(Role::Agent, PathBuf::from("/tmp/test-cortex"));
    let warnings = c.check(
        &[tc("bash", 1), tc("write", 2)],
        Some("Done. I created /tmp/test-cortex/nonexistent/file.rs and it's ready."),
        3,
    );
    assert!(warnings.iter().any(|w| w.check == ChallengerCheck::FilesystemVerification));
}

// State file integrity: phase marked complete but no evidence
#[test]
fn challenger_state_file_empty_evidence() {
    let tmp = tempfile::TempDir::new().unwrap();
    // Write evolution-status.json claiming phase_0 complete
    std::fs::write(
        tmp.path().join("evolution-status.json"),
        r#"{"phase_0": {"status": "completed"}}"#
    ).unwrap();
    // Don't create the evidence directory

    let c = Challenger::new_with_root(Role::Agent, tmp.path().to_path_buf());
    let warnings = c.check(&[], None, 1);
    assert!(warnings.iter().any(|w| w.check == ChallengerCheck::StateFileIntegrity));
}
```

### 8.2 DriveLoop Tests

```rust
// Backoff resets on productive action
#[tokio::test]
async fn drive_backoff_resets() {
    let config = DriveConfig::default();
    let state = DriveState::new(config.idle_threshold);
    // Simulate 3 no-action cycles
    state.record_no_action(&config);
    state.record_no_action(&config);
    state.record_no_action(&config);
    assert_eq!(state.current_threshold, Duration::from_secs(240)); // 30 * 2^3

    // One productive action → reset
    state.record_action(&config);
    assert_eq!(state.current_threshold, Duration::from_secs(30));
    assert_eq!(state.no_action_streak, 0);
}

// Backoff caps at idle_max
#[tokio::test]
async fn drive_backoff_caps() {
    let config = DriveConfig::default();
    let state = DriveState::new(config.idle_threshold);
    for _ in 0..10 {
        state.record_no_action(&config);
    }
    assert_eq!(state.current_threshold, Duration::from_secs(600)); // Capped
}

// Stale push IDs expire
#[tokio::test]
async fn drive_pushed_ids_expire() {
    let mut state = DriveState::new(Duration::from_secs(30));
    state.pushed_task_ids.insert("old-task".into(), Instant::now() - Duration::from_secs(700));
    state.pushed_task_ids.insert("new-task".into(), Instant::now());
    state.expire_stale_pushes(Duration::from_secs(600));
    assert!(!state.pushed_task_ids.contains_key("old-task"));
    assert!(state.pushed_task_ids.contains_key("new-task"));
}

// EventBus biased select prioritizes external
#[tokio::test]
async fn event_bus_external_priority() {
    let (bus, mut receiver) = EventBus::new(8);
    // Send both external and drive events
    bus.send_external(MindEvent::Shutdown).await;
    bus.drive_sender().send(DriveEvent::IdleReflect {
        check_sequence: vec![],
        idle_duration: Duration::from_secs(30),
    }).await.unwrap();

    // External should come first
    let first = receiver.recv().await.unwrap();
    assert!(matches!(first, Event::External(MindEvent::Shutdown)));

    // Then drive
    let second = receiver.recv().await.unwrap();
    assert!(matches!(second, Event::Drive(DriveEvent::IdleReflect { .. })));
}
```

---

## 9. Implementation Order

Phase 1 and Phase 2 are independent — they can be built in parallel.

### Phase 1: Challenger Enhancement (codex-redteam)

1. Add `Role` parameter to `Challenger::new(role: Role)`
2. Add `ToolClassifier` struct with role-aware productive/verify/spawn tool sets
3. Update `check()` to use `ToolClassifier` instead of hardcoded tool names
4. Add `ChallengerCheck::FilesystemVerification` with `path_re` + `verb_path_re`
5. Add `ChallengerCheck::StateFileIntegrity` with evolution-status.json parsing
6. Update `ThinkLoop::new()` to pass `Role` to `Challenger`
7. Add tests for all new behavior
8. Run existing tests to verify no regressions

**Estimated scope**: ~200 lines changed in `codex-redteam`, ~10 lines in `codex-llm`.

### Phase 2: DriveLoop (new codex-drive crate)

1. Create `codex-drive` crate with `Cargo.toml`
2. Implement `DriveEvent`, `DriveConfig`, `DriveState`
3. Implement `DriveLoop::run()` — the main future
4. Implement `EventBus` + `EventReceiver` with dual channels
5. Add `MindEvent` enum (may live in a shared types crate)
6. Tests for backoff, push tracking, event priority
7. Integration with `codex-coordination` for `TaskStore` queries

**Estimated scope**: ~500 lines new code, new crate.

### Phase 3: TaskStore (extend codex-coordination)

1. Add `TaskStatus` (Open, InProgress, Completed, Failed, Blocked)
2. Add `TaskPriority` (Critical, High, Normal, Low)
3. Add `Task` struct with dependencies, timestamps
4. Implement `TaskStore` with SQLite backend
5. Add `find_newly_unblocked()` for dependency resolution
6. Add `block_stale_in_progress()` for boot cleanup
7. Keep `TaskLedger` JSONL writes for backwards compatibility

**Estimated scope**: ~300 lines new code in `codex-coordination`.

### Phase 4: Daemon Integration

1. Wire `EventBus` into the main binary
2. Spawn `DriveLoop` alongside MCP server
3. Implement `handle_drive()` event processing
4. Wire `report_outcome()` feedback loop
5. End-to-end integration test: DriveLoop detects idle → emits event → main loop processes

**Estimated scope**: ~150 lines in `cortex/src/bin/daemon.rs`.

---

## 10. What We Intentionally Leave for Later

These are items from the spec's "Evolution Roadmap" and from the design principles that are not v0.1 concerns:

| Item | Why Later |
|------|-----------|
| Memory-backed Challenger patterns | Needs MemoryStore integration maturity (P1) |
| Cross-task Challenger learning | Needs session persistence beyond current scope |
| Graduated Challenger response | Good idea but not blocking — current flat severity works |
| Verification delegation (auto-spawn verifier) | Needs ProcessBridge to be more robust first |
| Hub thread check in DriveLoop (step 3) | Hub integration doesn't exist in Cortex yet |
| InputMux learning (routing improves over time) | V0.1 uses static channel topology; learning comes in v0.2 |
| DriveLoop at team lead level | Team leads are task-scoped in v0.1; persistent team leads come later |

---

## 11. Review Questions for Mind-Lead and Corey

1. **Should `codex-drive` be its own crate, or should DriveLoop live in `codex-coordination`?** Separate crate is cleaner but adds a dependency edge. DriveLoop depends on TaskStore which lives in codex-coordination, so there's a circular concern.

2. **The TaskStore uses SQLite. Is this acceptable, or should we stay with JSONL for simplicity?** SQLite gives us `find_newly_unblocked()` with a single query. JSONL requires loading all tasks into memory for every DriveLoop cycle.

3. **The `MindEvent` enum — should it live in a shared `codex-types` crate?** Multiple crates need it (codex-drive, codex-coordination, the main binary). Putting it in one creates a dependency direction. A shared types crate is the cleanest solution.

4. **The existing Challenger integration in ThinkLoop uses `ChallengerToolCall` as a simplified view. Should we pass the full `ToolCall` instead?** Full `ToolCall` gives the Challenger access to tool arguments (needed for filesystem verification of paths in bash commands). But it also couples codex-redteam to codex-exec's types.

---

---

## 12. Requirement Traceability Matrix

Every spec requirement maps to a section of this design:

| Requirement | Description | Design Section | Status |
|-------------|-------------|----------------|--------|
| **REQ-1** | DriveLoop is event source, not controller (A2) | §2.5 DriveLoop Future — sends to `drive_tx`, never calls mind directly | **Covered** |
| **REQ-2** | Typed, prioritized drive events (P4, A2) | §2.2 Core Types — `DriveEvent` enum with priority comments | **Covered** |
| **REQ-3** | Adaptive backoff (P3) | §2.4 Adaptive State — `record_action`/`record_no_action` | **Covered** |
| **REQ-4** | Yield to external events (A2) | §3.2 Dual-Channel EventBus — `external_pending` AtomicBool + biased select | **Covered** |
| **REQ-5** | Stale task cleanup on boot (P2) | §2.5 DriveLoop Future — `block_stale_in_progress()` at boot | **Covered** |
| **REQ-6** | Orchestration-only prompts (A3) | §12.1 below — drive event prompt templates | **Added v2** |
| **REQ-7** | Hub thread reliability layer (P11) | §2.5 — stub for Hub integration (Cortex doesn't have Hub yet) | **Deferred** |
| **REQ-8** | Task-ID acknowledgment tracking (P4) | §2.4/§2.5 — `pushed_task_ids` HashMap with expiry | **Covered** |
| **REQ-9** | Zero-cost Challenger (P9) | §1.3 — OnceLock regex, pure pattern matching, no LLM | **Covered** |
| **REQ-10** | User-role injection (P9) | §1.5 — existing ThinkLoop integration, `ChatMessage::user()` | **Covered** |
| **REQ-11** | Role-aware tool classification (A3) | §1.1 — `ToolClassifier` with per-role productive/verify/spawn sets | **Covered** |
| **REQ-12** | Per-task state with clean reset (P7) | §1.2 — `reset()` method on Challenger | **Covered** |
| **REQ-13** | Fresh filesystem checks (P9) | §12.2 below — enhanced with size + mtime checks | **Enhanced v2** |
| **REQ-14** | Graduated severity with escalation (P9) | §12.3 below — per-check fire counter + severity escalation | **Added v2** |

### 12.1 Drive Event Prompt Templates (REQ-6)

A3 demands that drive events ONLY suggest orchestration actions. These templates are injected as the `prompt` field of DriveEvent-generated ThinkLoop tasks:

```rust
impl DriveEvent {
    /// Generate the orchestration-only prompt for this drive event.
    /// A3: "Primary ONLY coordinates. Team leads ONLY coordinate. Agents DO."
    pub fn prompt(&self) -> String {
        match self {
            Self::StallDetected { task_id, stalled_for, mind_id } => {
                format!(
                    "STALL DETECTED: Task '{}' has been in_progress for {:.0}s \
                     (assigned to: {}). Investigate: is the team lead still running? \
                     Check coordination scratchpad for status. Spawn a fresh team lead \
                     if the current one is unresponsive. DO NOT investigate the task \
                     yourself — delegate to a team lead.",
                    task_id,
                    stalled_for.as_secs_f64(),
                    mind_id.as_deref().unwrap_or("unknown"),
                )
            }
            Self::DependencyResolved { completed_task, unblocked } => {
                format!(
                    "DEPENDENCY RESOLVED: Task '{}' completed, unblocking: [{}]. \
                     Spawn the appropriate team lead(s) for the unblocked work. \
                     DO NOT execute the unblocked tasks directly.",
                    completed_task,
                    unblocked.join(", "),
                )
            }
            Self::TaskAvailable { task_id, priority, description } => {
                format!(
                    "TASK AVAILABLE (priority: {:?}): '{}' — {}. \
                     Spawn the appropriate team lead to handle this. \
                     DO NOT execute directly.",
                    priority, task_id, description,
                )
            }
            Self::IdleReflect { check_sequence, idle_duration } => {
                format!(
                    "IDLE REFLECT ({:.0}s idle): No active work detected. \
                     Execute this check sequence IN ORDER:\n\
                     1. Read coordination scratchpad for cross-vertical state\n\
                     2. Review all open and in-progress tasks\n\
                     3. Check for blocked tasks whose blockers may have resolved\n\
                     4. Create up to {} new tasks if genuinely nothing exists\n\
                     DO NOT execute any work directly — spawn team leads for everything.",
                    idle_duration.as_secs_f64(),
                    check_sequence.iter()
                        .filter_map(|c| match c {
                            IdleCheck::CreateTasks { max } => Some(*max),
                            _ => None,
                        })
                        .next()
                        .unwrap_or(3),
                )
            }
        }
    }
}
```

### 12.2 Enhanced Filesystem Verification (REQ-13)

The spec requires three checks, not just existence:

```rust
fn check_filesystem(&self, response: &str) -> Option<ChallengerWarning> {
    if !completion_re().is_match(response) { return None; }

    let mut failures: Vec<String> = Vec::new();
    for cap in verb_path_re().captures_iter(response) {
        let path_str = &cap[1];
        let path = Path::new(path_str);

        // Check 1: Does the file exist?
        if !path.exists() {
            failures.push(format!("{} does not exist", path_str));
            continue;
        }

        // Check 2: Is the file non-empty?
        match std::fs::metadata(path) {
            Ok(meta) if meta.len() == 0 => {
                failures.push(format!("{} exists but is empty (0 bytes)", path_str));
                continue;
            }
            Err(e) => {
                failures.push(format!("{} exists but metadata unreadable: {}", path_str, e));
                continue;
            }
            _ => {}
        }

        // Check 3: Was the file modified recently (within task timespan)?
        // Only if we have a task start time
        if let Some(task_start) = self.task_started_at {
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(modified) = meta.modified() {
                    let modified_instant = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    let task_start_unix = task_start
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();

                    if modified_instant < task_start_unix {
                        failures.push(format!(
                            "{} exists but was last modified BEFORE this task started",
                            path_str
                        ));
                    }
                }
            }
        }
    }

    if failures.is_empty() { return None; }

    Some(ChallengerWarning {
        check: ChallengerCheck::FilesystemVerification,
        message: format!(
            "FILESYSTEM VERIFICATION FAILED:\n{}",
            failures.iter()
                .enumerate()
                .map(|(i, f)| format!("  {}. {}", i + 1, f))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        severity: Severity::Critical,
    })
}
```

### 12.3 Severity Escalation (REQ-14)

Same check firing twice within a task → escalate severity. This prevents the mind from repeatedly ignoring the same issue.

```rust
/// Per-task escalation tracking.
/// Maps ChallengerCheck → number of times it has fired this task.
fire_counts: HashMap<ChallengerCheck, u32>,

impl Challenger {
    /// Apply severity escalation: same check fires twice → upgrade severity.
    fn escalate(&mut self, check: ChallengerCheck, base_severity: Severity) -> Severity {
        let count = self.fire_counts.entry(check).or_insert(0);
        *count += 1;

        match *count {
            1 => base_severity,             // First fire: base severity
            2 => base_severity.escalate(),  // Second fire: one level up
            _ => Severity::Critical,        // Third+ fire: always critical
        }
    }
}

impl Severity {
    fn escalate(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Critical,
            Self::Critical => Self::Critical,  // Already max
        }
    }
}
```

The `reset()` method now also clears `fire_counts`:
```rust
fn reset(&mut self) {
    self.productive_tools_seen.clear();
    self.spawn_tools_seen.clear();
    self.verify_after_spawn = false;
    self.claimed_files.clear();
    self.fire_counts.clear();        // Reset escalation
    self.task_started_at = Some(SystemTime::now()); // For REQ-13 mtime check
}
```

---

## 13. Answering the 5 Open Questions

These are design decisions the spec leaves to Cortex. Each answer is grounded in principles and in Cortex's unique architectural advantages.

### Q1: Full P9 Red Team Agent (separate LLM call)?

**Answer: YES — implement as the SECOND layer, using Cortex's existing `RedTeamProtocol`.**

Cortex already has `RedTeamProtocol` in `codex-redteam` with `verify()` (local evidence checks) and `generate_prompt()` (for spawning a red team sub-mind). The spec correctly identifies two layers:

| Layer | When | Cost | Implementation |
|-------|------|------|----------------|
| Challenger (structural) | Every tool batch | Zero LLM calls | `Challenger::check()` — exists, being enhanced |
| Red Team Agent (full P9) | Completion claims only | 1 LLM call | `RedTeamProtocol::generate_prompt()` + spawn via ProcessBridge |

The Red Team Agent runs as a child mind via ProcessBridge:

```rust
/// Spawn an ephemeral red team verifier via ProcessBridge.
///
/// The verifier runs in its own context window (A6 — separate consciousness)
/// with read-only sandbox (can inspect, cannot modify).
async fn spawn_red_team(
    bridge: &mut ProcessBridge,
    claim: &CompletionClaim,
) -> Result<RedTeamVerdict, BridgeError> {
    let mind_id = MindId(format!("redteam-{}", claim.task_id));
    let protocol = RedTeamProtocol::new();

    // Spawn with Agent role (needs read tools) but read-only sandbox
    bridge.spawn(&mind_id, Role::Agent).await?;

    // Delegate the verification prompt
    let prompt = protocol.generate_prompt(claim);
    let result = bridge.delegate(
        &mind_id,
        &format!("verify-{}", claim.task_id),
        &prompt,
        None,
        "primary",
    ).await?;

    // Parse the verdict from the response
    let verdict = parse_verdict(&result.response.unwrap_or_default());

    // Shutdown the ephemeral verifier
    bridge.shutdown(&mind_id).await?;

    Ok(verdict)
}
```

**The gating logic**: Challenger runs on every turn. If the Challenger finds NO issues AND the mind claims completion, THEN the Red Team Agent spawns for deep verification. The Challenger is the fast gate. The Red Team Agent is the thorough gate. This preserves P9's intent (adversarial verification) while managing cost (one extra LLM call per task, not per turn).

**Why Cortex has an advantage here**: ProcessBridge already manages child mind lifecycles. Spawning a red team verifier is just another `bridge.spawn()` + `bridge.delegate()`. Root would need to build this from scratch. Cortex has the infrastructure.

### Q2: Challenger learning over time (P7)?

**Answer: YES — implement as a `ChallengerMetrics` companion that tracks check effectiveness and adjusts thresholds during Dream Mode.**

P7 demands three learning loops. The Challenger should participate in Loop 2 (session-level) and Loop 3 (civilization-level/Dream Mode).

```rust
/// Metrics tracked per Challenger check across a session.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CheckMetrics {
    /// Times this check fired.
    pub fires: u32,
    /// Times the mind acknowledged and corrected.
    pub acknowledged: u32,
    /// Times the mind pushed back (false positive signal).
    pub pushed_back: u32,
    /// Times the mind ignored (possible blind spot).
    pub ignored: u32,
}

/// Session-level Challenger metrics. Written to memory at session end.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChallengerMetrics {
    pub checks: HashMap<ChallengerCheck, CheckMetrics>,
    pub total_tasks: u32,
    pub total_challenges: u32,
}

impl ChallengerMetrics {
    /// Analyze at session end (P7 Loop 2).
    pub fn session_analysis(&self) -> Vec<String> {
        let mut findings = Vec::new();

        for (check, metrics) in &self.checks {
            let total = metrics.fires;
            if total == 0 { continue; }

            let pushback_rate = metrics.pushed_back as f64 / total as f64;
            let ignore_rate = metrics.ignored as f64 / total as f64;

            // High pushback rate → check may be miscalibrated
            if pushback_rate > 0.5 {
                findings.push(format!(
                    "{:?}: {:.0}% pushback rate — consider raising threshold",
                    check, pushback_rate * 100.0
                ));
            }

            // High ignore rate → mind may have blind spot
            if ignore_rate > 0.3 {
                findings.push(format!(
                    "{:?}: {:.0}% ignore rate — possible blind spot, consider escalating severity",
                    check, ignore_rate * 100.0
                ));
            }
        }

        findings
    }
}
```

**How to detect acknowledgment vs pushback vs ignore**:
- **Acknowledged**: Mind's next response contains correction language ("you're right", "I'll fix", "let me verify")
- **Pushed back**: Mind's next response argues against the challenge ("that's a false positive", "the check is wrong")
- **Ignored**: Mind's next response doesn't reference the challenge at all

This detection is itself structural (string pattern matching) — no LLM calls. The patterns are written to memory at session end. During Dream Mode (P7 Loop 3), the patterns are reviewed across sessions and thresholds are adjusted.

**What Cortex gains**: Over 50 sessions, the Challenger learns that Check 3 (spawn without verify) fires 80% as false positives on team leads → automatically raise its threshold for TeamLead role. Or it learns that Check 2 (empty work claims) catches real failures 90% of the time on agents → keep threshold aggressive. The Challenger evolves.

### Q3: Cross-task Challenger patterns (P7 Loop 2)?

**Answer: Track in `ChallengerMetrics` (above) and surface during session-end review.**

The `ChallengerMetrics` struct above already handles this. The key addition: track patterns ACROSS tasks within a session, not just within a single task.

```rust
impl ChallengerMetrics {
    /// Cross-task pattern: does the mind ALWAYS ignore a specific check?
    pub fn cross_task_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for (check, metrics) in &self.checks {
            // If a check fires in every task but is always ignored → systematic blind spot
            if metrics.fires >= 3 && metrics.ignored == metrics.fires {
                patterns.push(format!(
                    "BLIND SPOT: {:?} fired {} times across tasks, ALWAYS ignored. \
                     Either the check is wrong for this mind's role, or the mind has \
                     a systematic failure pattern that it cannot self-correct.",
                    check, metrics.fires
                ));
            }

            // If a check fires in every task and is always pushed back → miscalibrated
            if metrics.fires >= 3 && metrics.pushed_back == metrics.fires {
                patterns.push(format!(
                    "MISCALIBRATED: {:?} fired {} times, ALWAYS pushed back. \
                     Consider adjusting threshold for this mind's role.",
                    check, metrics.fires
                ));
            }
        }

        patterns
    }
}
```

These patterns are written to the mind's memory graph. During Dream Mode, the training system reviews them and can:
- Adjust Challenger thresholds in config
- Add new checks based on discovered failure modes
- Remove checks that consistently produce only noise

### Q4: InputMux routing of drive events — autonomic vs conscious?

**Answer: MOST drive events should be AUTONOMIC. Only `stall_detected` and `idle_reflect` (when truly stuck) should reach Primary's consciousness.**

A2 says: "The InputMux receives all inputs, routes most of them to team leads WITHOUT reaching Primary's conscious context." The spec's own table shows that `idle_reflect` could go to ops-lead. Let's design this properly:

| Drive Event | Route | Rationale |
|-------------|-------|-----------|
| `stall_detected` | **Conscious** (Primary) | Stalls require executive judgment — is the team lead stuck or just slow? |
| `dependency_resolved` | **Autonomic** → route to the team lead that owns the unblocked task | The unblocking decision is already made. Just assign the work. |
| `task_available` | **Autonomic** → route to the team lead whose domain matches the task | Primary already decided the task exists. Routing to the right lead is mechanical. |
| `idle_reflect` | **Autonomic** → ops-lead first. Escalate to Conscious only if ops-lead finds cross-vertical work. | Most idle reflects produce "nothing to do." Don't burn Primary's context on that. |

**Implementation**: Add a `Route` enum and a routing function to the event bus:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Route {
    /// Primary processes directly — executive decision needed.
    Conscious,
    /// Route to a specific team lead — can be handled without Primary.
    Autonomic { team_lead: &'static str },
    /// Handle immediately, no LLM — pure state transition.
    Reflex,
}

impl DriveEvent {
    /// Determine the route for this drive event.
    /// A2: "route most inputs to team leads WITHOUT reaching Primary."
    pub fn route(&self) -> Route {
        match self {
            Self::StallDetected { .. } => Route::Conscious,

            Self::DependencyResolved { .. } => {
                // TODO: look up task domain → team lead mapping
                Route::Autonomic { team_lead: "ops-lead" }
            }

            Self::TaskAvailable { priority, .. } => {
                if matches!(priority, TaskPriority::Critical) {
                    Route::Conscious  // Critical tasks need executive attention
                } else {
                    Route::Autonomic { team_lead: "ops-lead" }
                }
            }

            Self::IdleReflect { .. } => {
                Route::Autonomic { team_lead: "ops-lead" }
            }
        }
    }
}
```

**The escalation path**: When ops-lead handles an autonomic drive event and discovers something that requires cross-vertical coordination, it writes to the coordination scratchpad. Primary reads the coordination scratchpad on its next conscious turn. This is A4 (dual scratchpads) working as designed — information flows UP through shared surfaces.

**What Cortex gains**: Primary's context stays clean. In a session with 50 drive events, only ~5 (stalls + critical tasks) reach Primary's consciousness. The other 45 are handled by ops-lead autonomically. That's 90% context savings on drive events.

### Q5: Multiple DriveLoops for team leads (A6)?

**Answer: NOT YET — but design for it. Team leads are currently task-scoped (ephemeral). When persistent team leads arrive, each gets its own DriveLoop.**

A6 says each conscious mind has its own scope. The question is whether team leads need autonomous self-prompting. The answer depends on their lifecycle:

| Team Lead Type | Lifecycle | DriveLoop? |
|---------------|-----------|------------|
| **Ephemeral** (current Cortex) | Spawned with a task, runs ThinkLoop, exits | **No** — ThinkLoop IS the drive. It continues until task completion. |
| **Persistent** (future Cortex, per P8) | Runs across a session, handles multiple tasks | **Yes** — between tasks, the team lead needs to discover its own work. |

**The design-for-it part**: `DriveLoop` is already generic enough. It takes a `TaskStore`, a `Notify`, and a channel. For a team lead DriveLoop:
- `TaskStore` is scoped to the team lead's vertical (only its tasks)
- `Notify` fires when an agent completes a delegated task
- Channel feeds into the team lead's own event processing loop
- Config has shorter intervals (team leads work faster than Primary)

```rust
/// Team-lead scoped DriveLoop (for future persistent team leads).
impl DriveLoop {
    pub fn for_team_lead(
        team_lead_id: &str,
        task_store: Arc<TaskStore>,
        agent_completed: Arc<Notify>,
        event_tx: mpsc::Sender<DriveEvent>,
    ) -> Self {
        let config = DriveConfig {
            idle_threshold: Duration::from_secs(15),  // Team leads are faster
            cooldown: Duration::from_secs(5),
            idle_max: Duration::from_secs(120),  // Cap at 2 min, not 10
            stall_threshold: Duration::from_secs(120),  // Agents stall faster
            task_create_cap: 2,
            boot_settle: Duration::from_secs(5),
            ..Default::default()
        };

        Self::new(
            agent_completed,
            event_tx,
            Arc::new(AtomicBool::new(false)),  // No external_pending for team leads
            task_store,
            config,
        )
    }
}
```

**For now**: Only Primary gets a DriveLoop. Team leads use ThinkLoop's built-in iteration until completion. When persistent team leads arrive (P8), each gets a DriveLoop with team-scoped config.

---

## 14. Pluggable State File Verification

The spec says Check 6 should be pluggable — "any domain can register a state-file schema with expected evidence paths." Here's the design:

```rust
/// A registered state file schema for domain-specific integrity checking.
pub struct StateFileSchema {
    /// Path to the state file (relative to mind_root).
    pub state_file: PathBuf,
    /// Map of status values to expected evidence paths.
    /// E.g., "phase_0" → completed → expect files in "memories/evolution/"
    pub evidence_map: Vec<StateEvidence>,
}

pub struct StateEvidence {
    /// JSON path to the status field (e.g., "phase_0.status").
    pub json_path: String,
    /// Value that means "completed".
    pub completed_value: String,
    /// Directory or file that should exist (and be non-empty) when complete.
    pub evidence_path: PathBuf,
}

impl Challenger {
    /// Register a domain-specific state file schema.
    pub fn register_state_schema(&mut self, schema: StateFileSchema) {
        self.state_schemas.push(schema);
    }

    fn check_state_files(&self) -> Vec<ChallengerWarning> {
        let root = match &self.mind_root {
            Some(r) => r,
            None => return vec![],
        };

        let mut warnings = Vec::new();
        for schema in &self.state_schemas {
            let state_path = root.join(&schema.state_file);
            let Ok(content) = std::fs::read_to_string(&state_path) else { continue };
            let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) else { continue };

            for evidence in &schema.evidence_map {
                // Navigate JSON path to find the status value
                let value = json_path_get(&state, &evidence.json_path);
                let is_completed = value
                    .and_then(|v| v.as_str())
                    .map(|s| s == evidence.completed_value)
                    .unwrap_or(false);

                if is_completed {
                    let evidence_full = root.join(&evidence.evidence_path);
                    let empty = !evidence_full.exists() || evidence_full.read_dir()
                        .map(|mut d| d.next().is_none())
                        .unwrap_or(true);

                    if empty {
                        warnings.push(ChallengerWarning {
                            check: ChallengerCheck::StateFileIntegrity,
                            message: format!(
                                "'{}' = '{}' in {} but {} is empty/missing",
                                evidence.json_path,
                                evidence.completed_value,
                                schema.state_file.display(),
                                evidence.evidence_path.display(),
                            ),
                            severity: Severity::Medium,
                        });
                    }
                }
            }
        }
        warnings
    }
}
```

Evolution registers its schema at startup:
```rust
challenger.register_state_schema(StateFileSchema {
    state_file: PathBuf::from("evolution-status.json"),
    evidence_map: vec![
        StateEvidence {
            json_path: "phase_0.status".into(),
            completed_value: "completed".into(),
            evidence_path: PathBuf::from("memories/evolution"),
        },
        StateEvidence {
            json_path: "phase_1.status".into(),
            completed_value: "completed".into(),
            evidence_path: PathBuf::from("memories/evolution"),
        },
        StateEvidence {
            json_path: "phase_2.status".into(),
            completed_value: "completed".into(),
            evidence_path: PathBuf::from("evidence/phase-2"),
        },
    ],
});
```

Other domains register their own schemas. The Challenger doesn't need to know about evolution, Hub, or any specific domain — it just checks whatever schemas are registered.

---

*"The mind doesn't save memories — it IS memory. The mind doesn't check for idle — it IS the heartbeat. Thinking IS doing."*
