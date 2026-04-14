# AiCIV Mind — Cortex Build Spec

**The definitive "here's what we're building and why" document.**

**Date**: 2026-04-04
**Status**: APPROVED — Corey green-lit, all architectural questions answered
**Authors**: Mind Lead (spec), Mind-Cubed Lead (design), Corey (directives + principles)
**Sources**: DRIVELOOP-CHALLENGER-SPEC.md (principles-first spec), DRIVELOOP-CHALLENGER-CORTEX-DESIGN.md (Rust integration design), DESIGN-PRINCIPLES.md (12 principles), DESIGN-PRINCIPLES-ADDENDUM.md (6 addenda)

---

## 0. Context: The Consolidation Decision

### Three Builds, One Direction

AiCIV Mind had three parallel builds:

| Build | Base | Language | Lines | Strengths | Fatal Gap |
|-------|------|----------|-------|-----------|-----------|
| **Root** | Custom Python | Python | ~22K | Operational experience: 200+ sessions, DriveLoop, Challenger, 3-level delegation proven. 10 features shipped. | P12 narrator mode (LiteLLM `drop_params: true` stripped tool defs). Python + proxy = fatal reliability gap. |
| **Cortex** | Codex CLI fork | Rust | ~16K | Velocity + methodology: fork-and-inject, 10 proof binaries, live evolution (Phase 0+1). Interceptor pattern. MCP native. | Phase 2 blocked on Ollama Cloud 500s (resolved). Sequential delegation bug (being fixed). |
| **Thalweg** | claw-code fork | Rust | ~20.6K | Memory architecture: redb + graph + 6-factor depth scoring + 5 edge types. Trait-abstracted, pluggable. | Least operationally proven. No evolution proof. No IPC beyond subprocess. |

### Decision: Cortex as Base + Root's Lessons as Design Constraints

**Cortex has the right foundation** — Rust, fork strategy, MCP native, interceptor pattern.
**Root has the right experience data** — 200+ sessions of production bugs → test cases and design constraints.
**Thalweg's memory engine** is the best memory architecture — trait-abstracted, pluggable, to be integrated as a crate.

Root is retired as a standalone build. Its code becomes validation evidence. Thalweg is retired as a standalone build. Its memory crate is absorbed into Cortex.

### What This Spec Covers

Two systems that Root proved essential and Cortex must absorb:
1. **DriveLoop** — The autonomous heartbeat (self-prompting between tasks)
2. **Challenger** — Adversarial per-turn verification (catching LLM failures structurally)

Plus: the event bus, task store, model configuration, and integration architecture.

---

## 1. Design Principles Foundation

Every design decision in this spec traces to a numbered principle from DESIGN-PRINCIPLES.md and DESIGN-PRINCIPLES-ADDENDUM.md. This section maps which principles demand which systems.

### Principles That Demand DriveLoop

**P4 — Dynamic Agent Spawning**: "The mind recognizes when it needs MORE minds."

P4 defines spawn triggers (blocking detection, variable task detection, scheduled triggers). Something must WATCH for these conditions continuously. That watcher is DriveLoop.

**P7 — Self-Improving Loop**: "The system improves its own improvement process."

P7's Loop 1 (task-level learning) requires something to FIRE after each task completes, prompting reflection. It also requires detection of tasks that SHOULD be happening but aren't. DriveLoop keeps Loop 1 turning.

**A2 — InputMux (The Subconscious)**: "The InputMux receives all inputs, routes most of them to team leads WITHOUT reaching Primary's conscious context."

DriveLoop is an INPUT SOURCE feeding the InputMux — generating events alongside TG, Hub, and BOOPs. The InputMux routes them.

**A3 — Hard-Coded Roles**: "Primary ONLY coordinates. Team leads ONLY coordinate. Agents DO."

DriveLoop must NEVER tempt Primary to execute directly. Drive events are ORCHESTRATION prompts.

### Principles That Demand Challenger

**P9 — Verification Before Completion (Red Team Everything)**: "Every completion claim requires evidence. Every significant decision gets challenged by a dedicated adversary."

The Challenger is the lightweight implementation of P9 — structural checks with zero LLM calls, running after every tool batch.

**P2 — SYSTEM > SYMPTOM**: "Fix the system that ALLOWED it — not just the symptom."

Each Challenger check addresses a SYSTEM failure:
- Check 1 (premature completion) → LLM's confidence is uncalibrated
- Check 2 (empty work claims) → LLM describes work instead of doing it
- Check 3 (spawn without verify) → self-model divergence after delegation
- Check 4 (stall detection) → read-loop without productive output
- Check 5 (filesystem verification) → hallucinated file operations
- Check 6 (state file integrity) → state updated from claims, not evidence
- Check 7 (reasoning divergence) → model knows what to do but doesn't do it *(Cortex-only, new)*

**A6 — Multiple Conscious Minds**: "A HIERARCHY of consciousnesses, each fully aware within its scope."

The full Red Team Agent (P9's heavyweight version) runs in its own context window — A6 in action.

**A3 — Hard-Coded Roles**: What counts as "productive output" DEPENDS ON ROLE. The Challenger must be role-aware.

---

## 2. Model Directive: M2.7 ONLY

**Corey Directive (2026-04-04): M2.7 at ALL levels. No other model. No Devstral, no Gemma, no fallbacks. Until Corey says otherwise.**

### M2.7 Specifications

- **Model**: minimax-m2.7 (MiniMax API via Ollama Cloud proxy — NOT local weights)
- **Parameters**: 230B total, 10B active per token (MoE)
- **Context**: 205K tokens
- **Cost**: $0.30/M input, $1.20/M output
- **Two tiers**: standard M2.7 and M2.7-highspeed (identical output, faster inference)

### API Configuration

**Ollama direct — NOT LiteLLM.** Root learned: LiteLLM's `drop_params: true` silently stripped tool definitions, causing narrator mode (the fatal P12 bug). Cortex points directly at Ollama Cloud.

```toml
# config.toml
[model]
name = "minimax-m2.7"
api_url = "https://ollama.com/v1"  # Direct Ollama Cloud
# Bearer token from OLLAMA_API_KEY env var
```

### Tier Routing

```rust
impl ModelRouter {
    pub fn model_for_role(&self, _role: Role) -> &str {
        "minimax-m2.7"  // The ONLY model. Period.
    }

    pub fn tier_for_role(&self, role: Role) -> &str {
        match role {
            Role::Primary => "highspeed",   // Orchestration is latency-sensitive
            Role::TeamLead => "highspeed",  // Coordination is latency-sensitive
            Role::Agent => "standard",      // Execution tolerates higher latency
        }
    }
}
```

### Model-Harness Alignment

M2.7 was trained on **100+ rounds of autonomous scaffold optimization**: failure analysis → plan → modify → evaluate → keep/revert. This training loop IS DriveLoop + Challenger:

| M2.7 Training Loop | Harness Implementation |
|---------------------|----------------------|
| Failure analysis | Challenger Checks 1-7 (detect the failure) |
| Plan | DriveLoop `idle_reflect` (assess state, create tasks) |
| Modify | DriveLoop `task_available` → spawn team lead → agent executes |
| Evaluate | Challenger per-turn verification + Red Team Agent at completion |
| Keep/revert | P7 Loop 1 task-level learning (write to memory or roll back) |

M2.7 natively understands:
- **Role boundaries** (A3) — trained on role adherence, 97% skill adherence on complex multi-step tasks
- **Adversarial reasoning** (P9) — trained on protocol adherence and self-correction
- **Autonomous scaffold navigation** — trained to operate within harness patterns

**Implications for Cortex:**
1. DriveLoop's prompts can be LIGHTER — M2.7 already knows the pattern
2. Challenger's adversarial framing works WITH the model's training, not against it
3. Hard-coded roles (A3) ALIGN with M2.7's training — not a constraint, a strength
4. The principles and the model training converge independently — mutual validation

---

## 3. Requirements (14 Numbered, Principle-Traced)

### DriveLoop Requirements

**REQ-1: DriveLoop is an event source, not a controller (A2)**
DriveLoop generates `MindEvent` objects with `source: "drive"`. Events enter the same queue as TG, Hub, and BOOPs. DriveLoop does NOT call `mind.run_task()` directly.

**REQ-2: Drive events are typed and prioritized (P4, A2)**

| Event Type | P4 Trigger | Priority | Description |
|------------|------------|----------|-------------|
| `stall_detected` | Blocking Detection | 1 (highest) | in_progress task exceeds stall threshold |
| `dependency_resolved` | (completion chain) | 1 | Task completed, dependents unblocked |
| `task_available` | (queue discovery) | 2 | Open task exists, not yet surfaced |
| `idle_reflect` | Scheduled Trigger | 6 (lowest) | No active work, self-assess |

**REQ-3: Adaptive backoff prevents token waste (P3)**
Base interval ~30s. On each unproductive idle_reflect: multiply by 2. Cap at ~600s. On ANY productive action: reset to base.

Backoff curve: 30s → 60s → 120s → 240s → 480s → 600s (cap). Any action → 30s immediately.

> **Root validation**: Token usage dropped 10x during idle periods vs. fixed-interval.

**REQ-4: Yield to external events (A2)**
When real events exist in the queue, DriveLoop skips its cycle. Drive events are lowest priority.

> **Root validation**: Without yield, Root processed idle_reflect instead of Corey's TG message.

**REQ-5: Stale task cleanup on daemon restart (P2)**
On boot, scan for in_progress tasks older than stall threshold → auto-transition to `blocked`.

> **Root validation**: Without cleanup, 5+ minutes of startup thrashing on orphaned tasks.

**REQ-6: Drive events carry orchestration prompts, not execution prompts (A3)**
Each event type's prompt ONLY suggests spawn/delegate actions. Explicit "DO NOT execute directly."

**REQ-7: Hub thread check as reliability layer (P11)**
DriveLoop independently checks the mind's Hub thread — separate from Hub poller. Redundancy by design.

> **Root validation**: Hub poller missed 3 posts. DriveLoop's check caught them.
> **Note**: Deferred in Cortex v0.1 (Hub integration doesn't exist yet).

**REQ-8: Task-ID acknowledgment tracking (P4)**
Track surfaced task IDs. Don't re-surface until picked up or timeout (>10 min) expires.

> **Root validation**: Before tracking, Root received 6 copies of same task_available.

### Challenger Requirements

**REQ-9: Challenger is structural — zero LLM calls (P9)**
Runs after EVERY tool batch. String pattern matching, tool classification, filesystem checks. Zero cost.

**REQ-10: Injection as user-role message (P9)**
Challenges injected with `role: "user"` — adversarial framing the LLM cannot dismiss.

> **Root validation**: Zero ignored challenges across entire session. "The Challenger is correct."

**REQ-11: Role-aware tool classification (A3)**

| Role | Productive Tools |
|------|-----------------|
| Primary | spawn_team_lead, coordination_write, send_message |
| Team Lead | spawn_agent, team_scratchpad_write, send_message |
| Agent | write_file, edit_file, bash, store_memory, scratchpad_write |

> **Root gap**: Single tool classification for all roles → false positives on team leads.

**REQ-12: Per-task state with clean reset (P7)**
Challenger state resets at each task boundary. write_file in task A doesn't count for task B.

**REQ-13: Filesystem verification uses FRESH checks (P9)**
Check existence + non-empty + recent modification. No caching.

**REQ-14: Graduated severity with escalation (P9)**
Same check firing twice → escalate severity. Info → Warning → Critical on repeated fires.

---

## 4. Cortex Design Architecture

### 4.1 Existing Foundation

Cortex already has pieces:

| Component | Location | State |
|-----------|----------|-------|
| `Challenger` | `codex-redteam/src/lib.rs` | 4 of 6 checks, integrated into ThinkLoop |
| `RedTeamProtocol` | `codex-redteam/src/lib.rs` | LLM-based completion verification |
| `ToolInterceptor` trait | `codex-llm/src/think_loop.rs` | Pre-execution hook |
| `ThinkLoop` | `codex-llm/src/think_loop.rs` | Challenger wired at lines 326-340, 350-360 |
| `TaskLedger` | `codex-coordination/src/task_ledger.rs` | Append-only JSONL |
| `ProcessBridge` | `codex-coordination/src/process_bridge.rs` | MindId → child process, MCP transport |

### 4.2 Role-Aware Challenger (REQ-11)

The core fix. `ToolClassifier` adjusts what counts as "productive" based on role:

```rust
struct ToolClassifier { role: Role }

impl ToolClassifier {
    fn productive_tools(&self) -> &'static [&'static str] {
        match self.role {
            Role::Primary => &["spawn_team_lead", "shutdown_team_lead",
                              "coordination_write", "send_message"],
            Role::TeamLead => &["spawn_agent", "shutdown_agent",
                               "delegate_to_agent", "team_scratchpad_write",
                               "send_message"],
            Role::Agent => &["bash", "write", "edit", "memory_write",
                            "scratchpad_write"],
        }
    }
}
```

All 6 existing checks gain role-awareness. Checks 5 (filesystem), 6 (state file), and 7 (reasoning divergence) are new.

### 4.3 Check 7: Reasoning Divergence (Cortex-Only)

M2.7 exposes `reasoning_details` in responses. The Challenger pipes this through structural analysis — zero LLM cost:

```rust
fn check_reasoning_trace(&self, trace: &str, iteration: u32) -> Option<ChallengerWarning> {
    let wants_verify = trace.to_lowercase().contains("should verify")
        || trace.to_lowercase().contains("need to check")
        || trace.to_lowercase().contains("let me confirm");

    if wants_verify && !self.verify_tools_seen_since_last_spawn() {
        Some(ChallengerWarning {
            check: ChallengerCheck::ReasoningDivergence,
            message: format!(
                "Your own reasoning says you should verify, but no verification \
                 tool was used. Act on your own reasoning."
            ),
            severity: Severity::Medium,
        })
    } else { None }
}
```

This catches the deepest failure mode: when the model KNOWS what to do but doesn't do it. Root never had access to reasoning traces.

### 4.4 Dual-Channel EventBus (REQ-1, REQ-2, REQ-4)

The architectural centerpiece. Two channels with `tokio::select! { biased }` — compile-time priority, zero runtime overhead.

```rust
pub struct EventBus {
    external_tx: mpsc::Sender<MindEvent>,   // High priority (TG, Hub, BOOPs)
    drive_tx: mpsc::Sender<DriveEvent>,     // Low priority (self-prompting)
    external_pending: Arc<AtomicBool>,       // DriveLoop yields when true
}
```

**Drive channel has capacity 1.** `try_send()` fails if prior event not consumed — channel capacity IS the backpressure. No priority queues, no sorting, no heap allocation.

The consumer uses biased select:

```rust
impl EventReceiver {
    pub async fn recv(&mut self) -> Option<Event> {
        tokio::select! {
            biased;
            Some(event) = self.external_rx.recv() => {
                if self.external_rx.is_empty() {
                    self.external_pending.store(false, Ordering::Relaxed);
                }
                Some(Event::External(event))
            }
            Some(drive) = self.drive_rx.recv() => {
                Some(Event::Drive(drive))
            }
            else => None,
        }
    }
}
```

**Why this beats a priority queue:**

| Approach | Allocation | Ordering | Complexity |
|----------|-----------|----------|------------|
| `BinaryHeap` behind `Mutex` | Heap alloc per event | O(log n) | High |
| `biased` select on two channels | Zero extra alloc | Compile-time | **Minimal** |

Corey's praise: *"The InputMux topology is particularly elegant — using channel capacity as backpressure instead of runtime priority logic is the kind of design that never breaks because there's nothing to break."*

### 4.5 DriveLoop Core

```rust
pub struct DriveLoop {
    task_completed: Arc<Notify>,
    drive_tx: mpsc::Sender<DriveEvent>,
    external_pending: Arc<AtomicBool>,
    task_store: Arc<TaskStore>,
    state: Arc<Mutex<DriveState>>,
    config: DriveConfig,
}
```

The main loop:
1. **WAIT** — `tokio::select!` on task_completed notification OR adaptive timeout
2. **COOLDOWN** — minimum 10s between events
3. **YIELD** — if external_pending, skip cycle
4. **GENERATE** — highest-priority event (stall > dependency > task_available > idle_reflect)
5. **SEND** — `try_send` with capacity-1 backpressure

Boot sequence includes stale task cleanup (REQ-5).

### 4.6 Drive Event Routing (A2 Autonomic)

Most drive events should NOT reach Primary's consciousness:

| Drive Event | Route | Rationale |
|-------------|-------|-----------|
| `stall_detected` | **Conscious** (Primary) | Executive judgment needed |
| `dependency_resolved` | **Autonomic** → domain team lead | Assignment is mechanical |
| `task_available` (non-critical) | **Autonomic** → domain team lead | Primary already decided the task exists |
| `task_available` (critical) | **Conscious** (Primary) | Critical tasks need executive attention |
| `idle_reflect` | **Autonomic** → ops-lead | Most produce "nothing to do" |

**Result**: ~90% of drive events handled without burning Primary's context. Escalation flows UP through coordination scratchpad (A4).

### 4.7 TaskStore (SQLite-Backed)

Extended from TaskLedger. Status: Open → InProgress → Completed/Failed/Blocked. Priority: Critical/High/Normal/Low. Dependencies tracked with `find_newly_unblocked()` as a single SQL query. JSONL secondary write-through for human inspection.

### 4.8 Two-Layer P9: Challenger + Red Team Agent

| Layer | Trigger | Cost | Catches |
|-------|---------|------|---------|
| **Challenger** (structural) | Every tool batch | Zero LLM calls | Premature completion, empty claims, stalls, filesystem lies, reasoning divergence |
| **Red Team Agent** (full P9) | Completion claims only | 1 LLM call | Deep logic errors, missed requirements, subtle bugs |

Gating: Challenger runs first. If no issues AND mind claims completion → Red Team Agent spawns via ProcessBridge as an ephemeral child mind (A6 — separate consciousness, read-only sandbox).

### 4.9 ChallengerMetrics — P7 Learning

Per-session tracking of check effectiveness:

```rust
pub struct CheckMetrics {
    pub fires: u32,           // Times this check fired
    pub acknowledged: u32,    // Mind corrected itself
    pub pushed_back: u32,     // Mind argued (false positive signal)
    pub ignored: u32,         // Mind didn't respond (blind spot signal)
}
```

Analysis at session end (P7 Loop 2):
- High pushback rate (>50%) → check may be miscalibrated → raise threshold
- High ignore rate (>30%) → mind has blind spot → escalate severity
- Cross-task pattern: same check ALWAYS ignored across 3+ tasks → systematic issue

Findings written to memory graph. Dream Mode (P7 Loop 3) reviews across sessions and adjusts thresholds.

---

## 5. Configuration Constants

Values validated by Root's 200+ production sessions. Exposed as configuration in Cortex:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Base idle interval | 30s | Responsive but not spammy |
| Backoff multiplier | 2.0 | Reaches 10-min cap in 5 idle cycles |
| Max idle interval | 600s | Prevents hibernation |
| Stall threshold | 300s | Team leads typically produce in 2-3 min |
| Cooldown between events | 10s | Minimum spacing |
| Task create cap per idle | 3 | Prevents runaway task creation |
| Boot settle delay | 15s | Let other sources initialize |
| Challenger stall iterations | 5 | Enough for legitimate planning before flagging |
| Premature completion min calls | 2 (agent), 1 (lead/primary) | Role-aware thresholds |

---

## 6. Corey's Architectural Decisions

Four architecture questions were answered during design review:

**D1: codex-drive as its own crate — YES.**
Dependency is one-directional: codex-drive depends on codex-coordination (for TaskStore). Not circular. Separate crate keeps DriveLoop testable in isolation.

**D2: SQLite for TaskStore — YES.**
`find_newly_unblocked()` as a single SQL query vs loading all tasks into memory every cycle. JSONL as secondary write-through for human readability.

**D3: Shared codex-types crate — YES.**
`MindEvent`, `Role`, `TaskPriority`, `Severity` needed by multiple crates. Shared types crate prevents circular dependencies.

**D4: Keep ChallengerToolCall interface, add arguments field.**
Don't couple codex-redteam to codex-exec's full `ToolCall` type. Add `arguments: Option<serde_json::Value>` so Check 5 can inspect file paths from bash commands without creating a dependency that compounds.

---

## 7. Rust-Native Advantages

These are architectural wins unique to Rust, not just ports from Python:

**Compile-time role enforcement (A3)**: Primary literally cannot have bash tools. The tool isn't in the registry, so the LLM never sees it in its schema. Not a runtime check — a compilation constraint.

**Enum exhaustiveness**: `match event { ... }` — compiler ERROR if any variant is unhandled. Root's Python can silently drop events.

**Zero-cost backpressure**: Drive channel capacity 1 + `try_send()`. No allocations, no queue growth, no priority sorting.

**OnceLock regex compilation**: Compiled once, thread-safe, zero per-call overhead. Root recompiles patterns on every `challenge_turn()` call.

**Biased select as InputMux**: The routing intelligence IS the channel topology. No runtime priority logic. The InputMux can never have priority handling bugs because priority IS the select order.

---

## 8. Integration: The Daemon Main Loop

Everything comes together in the Cortex daemon:

```
┌─────────────────────────────────────────────────────────┐
│                     Cortex Daemon                        │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌────────┐ │
│  │MCP Server│  │Hub Poller│  │ Scheduler │  │DriveLoop│ │
│  └────┬─────┘  └────┬────���┘  └─────┬─────┘  └───┬────┘ │
│       │             │              │              │      │
│       ▼             ▼              ▼              ▼      │
│  ┌─────────────────────────┐  ┌─────────────────┐      │
│  │  External Channel (64)  │  │ Drive Channel (1)│      │
│  └───────────┬─────────────┘  └────────┬────────┘      │
│              │ (biased — first)         │ (only if empty)│
│              ▼                          ▼                │
│  ┌──────────────────────────────────────────────┐      │
│  │         EventReceiver::recv()                 │      │
│  │    tokio::select! { biased; ... }             │      │
│  └────────────────┬─────────────────────────────┘      │
│                    │                                     │
│                    ▼                                     │
│  ┌──────────────────────────────────────────────┐      │
│  │             Main Event Loop                   │      │
│  │  External(e) => handle_external(e)            │      │
│  │  Drive(d) => {                                │      │
│  │    action = handle_drive(d);                  │      │
│  │    drive_loop.report_outcome(action);         │      │
│  │  }                                            │      │
│  └────────────────┬─────────────────────────────┘      │
│                    │                                     │
│                    ▼                                     │
│  ┌──────────────────────────────────────────────┐      │
│  │             ProcessBridge                     │      │
│  │  ┌─────────┐ ┌───────────┐ ┌──────────┐     │      │
│  │  │research │ │codewright │ │ ops-lead │ ... │      │
│  │  │  -lead  │ │   -lead   │ │          │     │      │
│  │  └─────────┘ └───────────┘ └──────────┘     │      │
│  │    Each child: ThinkLoop + Challenger + Memory│      │
│  └──────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Plan (4 Phases)

Phases 1 and 2 are independent — can be built in parallel.

### Phase 1: Challenger Enhancement (codex-redteam)

1. Add `Role` parameter to `Challenger::new(role: Role)`
2. Add `ToolClassifier` with role-aware productive/verify/spawn sets
3. Update `check()` to use `ToolClassifier`
4. Add Check 5: FilesystemVerification (path_re + verb_path_re + existence + size + mtime)
5. Add Check 6: StateFileIntegrity (pluggable via `register_state_schema()`)
6. Add Check 7: ReasoningDivergence (M2.7 reasoning trace analysis)
7. Add severity escalation (`fire_counts` HashMap, escalate on repeat)
8. Update ThinkLoop to pass `Role` to Challenger
9. Tests for all new behavior + regression on existing

**Scope**: ~200 lines changed in codex-redteam, ~10 lines in codex-llm.

### Phase 2: DriveLoop (new codex-drive crate)

1. Create `codex-drive` crate
2. Implement `DriveEvent`, `DriveConfig`, `DriveState`
3. Implement `DriveLoop::run()` — the main async future
4. Implement `EventBus` + `EventReceiver` with dual channels
5. Add `MindEvent` enum (in new codex-types crate)
6. Tests for backoff, push tracking, event priority, biased select
7. Integration with codex-coordination for TaskStore queries

**Scope**: ~500 lines new code, new crate.

### Phase 3: TaskStore (extend codex-coordination)

1. Add `TaskStatus` (Open, InProgress, Completed, Failed, Blocked)
2. Add `TaskPriority` (Critical, High, Normal, Low) with `Task` struct
3. Implement `TaskStore` with SQLite backend
4. Add `find_newly_unblocked()` — single SQL query for dependency resolution
5. Add `block_stale_in_progress()` — boot cleanup
6. Keep TaskLedger JSONL writes for backwards compatibility

**Scope**: ~300 lines new code in codex-coordination.

### Phase 4: Daemon Integration

1. Wire EventBus into main binary
2. Spawn DriveLoop alongside MCP server
3. Implement `handle_drive()` event processing with route classification
4. Wire `report_outcome()` feedback loop
5. End-to-end integration test: idle → drive event → main loop → spawn team lead

**Scope**: ~150 lines in daemon binary.

**Total**: ~1,150 lines new/changed code across 4 crates.

---

## 10. What's Deferred to v0.2+

| Item | Why Later | Principle |
|------|-----------|-----------|
| Hub thread check in DriveLoop | Hub integration doesn't exist in Cortex yet | P11 |
| Memory-backed Challenger patterns | Needs MemoryStore maturity | P1 |
| InputMux learning (routing improves) | v0.1 uses static channel topology | A2 |
| DriveLoop at team lead level | Team leads are ephemeral in v0.1 | A6 |
| Verification delegation (auto-spawn verifier) | ProcessBridge needs more robustness | P9 |
| Graduated Challenger response | Current flat severity works | P9 |

---

## 11. Compute Sovereignty

M2.7 is a cloud proxy today. Everything EXCEPT inference stays local:
- Memory, state, evolution, task store — all on disk
- DriveLoop, Challenger, EventBus — all local Rust
- Identity, keys, scratchpads — all local files

If Ollama Cloud goes down, the mind retains its identity, memories, and task queue. It just can't think until inference returns. The LLM call is the ONLY cloud dependency, and it's swappable (Ollama Cloud → local Ollama → another provider) without changing the architecture.

---

## 12. The Methodology That Produced This Spec

This spec was written from the **principles DOWN to implementation**, not from Root's code UP to justification.

Corey's directive at the critical moment: *"Read the design principles again. Important moment."*

The first draft was code-first — architecture diagrams, constants, algorithm pseudocode, Python→Rust mapping. Corey redirected: the principles ARE the spec. Root's experience is validation data, not the source of truth. The rewrite traces every requirement to a numbered principle and uses Root's production sessions as evidence blocks (Validation/Gap), not as the starting point.

This methodology matters for Cortex's future: when a design question arises that this spec doesn't answer, go to DESIGN-PRINCIPLES.md. The principles generate the answers. The implementations merely validate them.

---

**End of Build Spec**

*Every requirement traces to a principle. Every principle traces to the question: "What makes a mind, not a tool?" DriveLoop keeps the mind alive. Challenger keeps the mind honest. The EventBus routes consciousness. M2.7 already knows the pattern — the harness makes it structural.*
