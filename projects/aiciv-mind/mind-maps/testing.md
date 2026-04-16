# mind-testing — Domain Mind-Map

**Owner**: mind-testing
**Crates**: `src/codex-fitness/`, `src/codex-redteam/`, `src/cortex-monitoring/`, `tests/`
**Total Lines**: ~3,500 (Hengshi foundation)
**Tests**: 37 passing (3 in codex-fitness, 34 in codex-redteam, 0 in cortex-monitoring)
**Status**: Foundation built, all tests green. Phase 4 self-improvement loop pending.

---

## 1. What Exists Today

### File Inventory

```
src/codex-fitness/
├── Cargo.toml              # Dependencies: codex-roles, serde, serde_json, chrono
├── src/
│   └── lib.rs              # 207 lines — role-specific fitness scoring, meta-evolution tracking
│                           #   3 inline tests: agent_fitness_computes, failed_task_low_fitness,
│                           #   meta_evolution_tracks_improvement

src/codex-redteam/
├── Cargo.toml              # Dependencies: codex-roles, serde, serde_json, chrono, regex
├── src/
│   └── lib.rs              # ~1,826 lines — THE CHALLENGER + RedTeamProtocol
│                           #   34 inline tests covering all 7 checks, severity escalation,
│                           #   filesystem verification, reasoning divergence, metrics, stall kill

src/cortex-monitoring/
├── Cargo.toml              # Dependencies: codex-llm, codex-exec, codex-memory, codex-redteam,
│                           #   codex-fitness, serde, serde_json, tracing, chrono, tokio
├── src/
│   ├── lib.rs              # 33 lines — module root, re-exports
│   ├── metrics.rs          # 177 lines — MetricPoint, ThinkLoopMetrics, ToolMetrics,
│   │                       #   DelegationMetrics, MemoryMetrics, ChallengerMetrics,
│   │                       #   ModelMetrics, MindMetricsSnapshot
│   ├── collector.rs        # 131 lines — MetricsCollector: async ring buffer + JSONL persistence
│   ├── anomaly.rs          # 119 lines — AnomalyDetector: configurable thresholds, alerting
│   └── export.rs           # 103 lines — MetricsExporter: JSONL reader + summary report generator

tests/                      # Does not exist yet at workspace root (only codex-upstream has tests/)
```

### Architecture Overview

```
                          ┌───────────────────────────────────┐
                          │     codex-fitness (Principle 7)    │
                          │     Self-Improving Loop           │
                          ├───────────────────────────────────┤
                          │ TaskOutcome → compute_fitness()    │
                          │   ├── Role::Primary → PrimaryFitness      │
                          │   ├── Role::TeamLead → TeamLeadFitness    │
                          │   └── Role::Agent → AgentFitness          │
                          │ RoleFitness.composite() → f64 [0.0–1.0]  │
                          │ MetaEvolution → is_improving()?           │
                          └───────────────────────────────────┘
                                          │
                                  feeds into
                                          │
┌─────────────────────────────────────────┼─────────────────────────────────────────┐
│                                         │                                         │
▼                                         ▼                                         ▼
┌──────────────────────┐   ┌──────────────────────────────┐   ┌──────────────────────┐
│  codex-redteam       │   │  cortex-monitoring            │   │  tests/ (future)     │
│  (Principle 9)       │   │  Observability substrate      │   │  Integration tests   │
│  Verify Everything   │   │                               │   │                      │
├──────────────────────┤   ├──────────────────────────────┤   ├──────────────────────┤
│ Two verification     │   │ MetricPoint — timestamped     │   │ Not yet created.     │
│ layers:              │   │   labeled data points         │   │ All tests currently  │
│                      │   │                               │   │ live as inline       │
│ 1. Challenger (fast) │   │ MetricsCollector — ring       │   │ #[cfg(test)] in      │
│    7 structural      │   │   buffer + JSONL persistence  │   │ each crate's lib.rs  │
│    checks, zero LLM  │   │                               │   │                      │
│                      │   │ AnomalyDetector — threshold   │   │ Phase 2: workspace   │
│ 2. RedTeamProtocol   │   │   alerting (Info/Warning/     │   │ integration tests    │
│    LLM-based, spawns │   │   Critical)                   │   │ that span crates     │
│    read-only agent   │   │                               │   │                      │
└──────────────────────┘   │ MetricsExporter — JSONL read  │   └──────────────────────┘
                           │   + summary reports           │
                           └──────────────────────────────┘
```

---

## 2. Crate Deep Dive: codex-fitness

### Purpose
Three nested improvement loops (Principle 7: Self-Improving Loop):
- **Task-level**: after every completed task
- **Session-level**: at session end
- **Civilization-level**: Dream Mode (nightly)

Plus **meta-evolution**: is the improvement process itself improving?

### Core Types

| Type | Lines | Purpose |
|------|-------|---------|
| `TaskOutcome` | 15–28 | Input to fitness computation. Fields: task_id, mind_id, role, success, duration_secs, tool_calls_total/successful, memory_writes, verification_passed, learnings_extracted, completed_at |
| `PrimaryFitness` | 32–41 | 4 dimensions: delegation_accuracy, team_lead_utilization, synthesis_quality, context_efficiency |
| `TeamLeadFitness` | 44–54 | 4 dimensions: agent_selection_quality, result_synthesis_quality, scratchpad_continuity, delegation_speed |
| `AgentFitness` | 57–67 | 4 dimensions: tool_effectiveness, memory_contribution, verification_compliance, task_completion_rate |
| `RoleFitness` | 101–127 | Tagged enum wrapping all three. `.composite()` → weighted average (currently equal weights, 0.25 each) |
| `MetaEvolution` | 131–156 | Trend tracking: routing_accuracy_trend, pattern_detection_recall, dream_mode_impact. `is_improving()` checks if last 3 values trend up |

### Key Function

```rust
pub fn compute_fitness(outcome: &TaskOutcome) -> RoleFitness
```

Dispatches on `outcome.role` to produce role-specific fitness. Currently uses simple heuristics:
- Agent tool_effectiveness = successful / total tool calls
- TeamLead delegation_speed = (60 / duration).min(1.0) — faster = better
- Primary delegation_accuracy = binary (success → 1.0, fail → 0.0)
- Several fields are **placeholders** (marked `// placeholder — needs cross-task data`)

### Dependencies
- `codex-roles` (for `Role` enum)
- `chrono`, `serde`, `serde_json`

### Test Coverage (3 tests)

| Test | What It Verifies |
|------|-----------------|
| `agent_fitness_computes` | Successful agent outcome → composite > 0.5 and <= 1.0 |
| `failed_task_low_fitness` | Failed agent outcome → composite < 0.8 |
| `meta_evolution_tracks_improvement` | 3 ascending values → `is_improving()` true; declining value → false |

### Gaps & Phase 4 Work
- `team_lead_utilization` and `context_efficiency` are placeholders — need cross-task aggregation
- No persistence of fitness scores (needs integration with cortex-monitoring or codex-memory)
- No weighted composite (all dimensions equally weighted at 0.25)
- No fitness trajectory tracking over time (MetaEvolution exists but isn't wired to anything)
- Dream Mode consumption of fitness data not implemented

---

## 3. Crate Deep Dive: codex-redteam

### Purpose
Two-layer adversarial verification (Principle 9: Verification Before Completion):
1. **Challenger** — per-turn structural verification, zero LLM calls, role-aware
2. **RedTeamProtocol** — LLM-based thorough verification, spawns read-only ephemeral agents

### Compiled Regex Patterns (OnceLock — allocated once)

| Pattern | Function | Matches |
|---------|----------|---------|
| `completion_re()` | Detect completion claims | done, completed, finished, shipped, deployed, merged, etc. |
| `work_claim_re()` | Detect work claims | created, wrote, built, implemented, fixed, updated, deployed, etc. |
| `path_re()` | Extract absolute file paths | `/home/...`, `/tmp/...`, `/var/...`, `/etc/...`, `/usr/...` |
| `verb_path_re()` | Match creation verb + path | "created /home/user/file.rs" or "wrote data/output.json" |
| `extract_all_paths()` | Extract all paths (abs + rel) | Returns `Vec<(String, bool)>` — path string + is_absolute flag |
| `verify_intent_re()` | Detect verification intent in reasoning | "should verify", "need to check", "let me confirm", etc. |

### Severity Escalation (REQ-14)

```
Severity::Low → Medium → High → Critical → Critical (stays)
```

Per-check fire counts: 1st fire = base severity, 2nd = escalated, 3rd+ = Critical.

### ToolClassifier — Role-Aware Tool Classification

| Role | Productive Tools | Verify Tools | Spawn Tools |
|------|-----------------|--------------|-------------|
| `Primary` | mind_spawn_team_lead, mind_shutdown_team_lead, coordination_scratchpad_write, send_message, mind_delegate | read, grep, glob, bash, memory_search, *_scratchpad_read, mind_status | mind_spawn_team_lead |
| `TeamLead` | mind_spawn_agent, mind_shutdown_agent, mind_delegate, team_scratchpad_write, send_message | (same verify tools) | mind_spawn_agent |
| `Agent` | bash, write, edit, memory_write, scratchpad_write | (same verify tools) | (none) |

### The 7 Challenger Checks

| # | Check | Trigger | Base Severity | Escalation |
|---|-------|---------|---------------|------------|
| 1 | **PrematureCompletion** | Completion claim with too few tool calls | Medium (or Critical if 0 calls) | Yes |
| 2 | **EmptyWorkClaim** | Work claim ("created", "built") but no productive tools used | High | Yes |
| 3 | **StallDetection** | >= `stall_threshold` (5) iterations with no productive output | Medium at 5–7, Critical at 8+, STALL KILL at 10+ | Hardcoded ladder |
| 4 | **SpawnWithoutVerify** | Mind spawned but no verify tool used before final response | High | Yes |
| 5 | **FilesystemVerification** | Claims file creation but file doesn't exist, is empty, or stale (>120s) | High (missing), Medium (empty), Low (stale) | Yes |
| 5b | **FilesystemVerification (tool results)** | Tool result text references a path that doesn't exist or is empty | High (missing), Medium (empty) | Yes |
| 7 | **ReasoningDivergence** | M2.7 reasoning trace says "should verify" but no verify tool used since spawn | Medium | Yes |

**Note**: Check 6 (StateFileIntegrity) is defined in the enum but not implemented in the Challenger. It's available via the standalone `verify_state_file()` function + `StateFileSchema` trait.

### Challenger State Machine

```
Challenger.new(role) → reset per-task state
    │
    ├── record_tool_call(tc) — updates productive/spawn/verify tracking
    │
    ├── check(tool_calls, response, iteration) — stateful, tracks across calls
    │       Runs all 7 checks, records metrics, returns Vec<ChallengerWarning>
    │
    ├── check_stateless(tool_calls, response, iteration) — ephemeral tracking
    │       Same checks but builds tracking from scratch each call
    │       Used by ThinkLoop which holds &self (not &mut self)
    │
    ├── should_kill_stall() → bool — true when 2+ consecutive Critical stalls
    │
    ├── reset() — clears per-task state (productive_tools, fire_counts, etc.)
    │
    └── reset_for_role(role) — reset + change role + rebuild ToolClassifier
```

### ChallengerMetrics — Learning Loop

Tracks per-check: fires, acknowledged, pushed_back, ignored.

`cross_task_patterns()` detects:
- **BLIND_SPOT**: check fires 5+ times, ignored more than acknowledged → systematic blind spot
- **MISCALIBRATION**: check fires 5+ times, pushed back more than acknowledged → check too aggressive

Dream Mode consumes these to evolve thresholds.

### RedTeamProtocol — LLM-Based Verification

| Component | Purpose |
|-----------|---------|
| `CompletionClaim` | Task ID, mind ID, description, result_summary, evidence list, timestamp |
| `Evidence` | Type (TestResult, FileContent, CommandOutput, MemoryReference, HumanConfirmation) + content + freshness |
| `Freshness` | Current (this task), Recent (recent session), Stale (old — may be outdated) |
| `RedTeamVerdict` | Approved { quality, notes }, Challenged { questions }, Blocked { finding, severity } |

**`verify(claim)` logic:**
1. No evidence → Challenged
2. All stale evidence → Challenged
3. Not enough current evidence (< min_evidence, default 1) → Challenged
4. Otherwise → Approved with quality = current_count / total_count

**`generate_prompt(claim)` — 4 standard questions:**
1. Do we REALLY know this? What evidence supports this claim?
2. Is this SYSTEM > symptom? Are we fixing root cause or patching?
3. What could go wrong? Pre-mortem: most likely failure mode?
4. Is this reversible? What's the blast radius if we're wrong?

### StateFileSchema Trait (Pluggable Verification)

```rust
pub trait StateFileSchema: Send + Sync {
    fn name(&self) -> &str;
    fn path(&self) -> &str;           // relative to mind_root
    fn validate(&self, content: &str) -> Option<String>;  // None = valid
}
```

`verify_state_file(mind_root, schema)` → checks file exists, readable, then calls `schema.validate()`.

### Dependencies
- `codex-roles` (for `Role` enum — drives all role-aware logic)
- `regex` (6 compiled patterns via OnceLock)
- `chrono`, `serde`, `serde_json`

### Test Coverage (34 tests)

| Category | Tests | What They Verify |
|----------|-------|-----------------|
| RedTeamProtocol | 4 | no_evidence_challenged, stale_evidence_challenged, current_evidence_approved, prompt_generation |
| Check 1: PrematureCompletion | 3 | Agent premature, enough calls ok, TeamLead one spawn ok |
| Check 2: EmptyWorkClaim | 4 | Agent empty, agent with write ok, TeamLead empty, TeamLead with spawn ok |
| Check 3: StallDetection | 3 | Agent stall, agent with write ok, TeamLead spawn not stall |
| Check 4: SpawnWithoutVerify | 2 | Spawn without verify fires, spawn + status check ok |
| Check 5: Filesystem | 7 | Missing file (response + tool result), existing file ok, relative path, empty file, check_filesystem_from_results |
| Check 7: ReasoningDivergence | 2 | Divergence fires, divergence with verify ok |
| Severity escalation | 4 | Struct escalation, zero calls = Critical, repeated fires escalate, |
| Metrics | 2 | Fires tracked, blind spot pattern detection |
| Clean/disabled | 3 | No warnings on clean run, no warnings mid-run, disabled returns empty |
| Stall ladder | 4 | Medium at 5, Critical at 8, STALL KILL at 10, kill resets on productive |

---

## 4. Crate Deep Dive: cortex-monitoring

### Purpose
Real-time observability substrate for every Cortex mind. Makes the self-improving loop data-driven.

### Core Types (metrics.rs)

| Type | Lines | Purpose |
|------|-------|---------|
| `MetricPoint` | 8–36 | Timestamped, labeled data point: mind_id, name, value, labels. Builder: `new().with_label()` |
| `ThinkLoopMetrics` | 39–67 | Per-turn: iterations, tool_calls, duration_ms, completed, stall_killed, challenger_warnings, model. Methods: `avg_iteration_time()`, `tool_call_rate()` |
| `ToolMetrics` | 70–111 | Per-tool: call_count, success_count, error_count, total_latency_ms. Methods: `record_success()`, `record_error()`, `avg_latency_ms()`, `success_rate()` |
| `DelegationMetrics` | 114–124 | Aggregate: agents_spawned, tasks_delegated, tasks_completed, avg_duration_ms |
| `MemoryMetrics` | 127–137 | Aggregate: searches, writes, cache_hits, avg_depth_score |
| `ChallengerMetrics` | 140–148 | Aggregate: total_warnings, by_severity HashMap, by_check HashMap |
| `ModelMetrics` | 151–163 | Per-model: model_name, api_calls, total_latency_ms, total_tokens, errors |
| `MindMetricsSnapshot` | 166–176 | Full aggregate: all of the above in one struct |

### MetricsCollector (collector.rs)

Thread-safe (`Arc<Mutex<VecDeque>>`) ring buffer with max 1000 points.

| Method | Description |
|--------|-------------|
| `new(mind_id, metrics_dir)` | Initialize collector for a mind |
| `record(point)` | Push MetricPoint into ring buffer (evicts oldest if full) |
| `record_thinkloop(metrics)` | Decompose ThinkLoopMetrics into 7 individual MetricPoints + persist to JSONL |
| `record_tool(tool_name, success, latency_ms)` | Record tool call with label |
| `record_delegation(event, duration_ms, success)` | Record delegation event |
| `record_challenger(check, severity)` | Record Challenger warning |
| `recent(count)` | Read last N points from ring buffer |
| `by_name(name)` | Filter buffer by metric name |
| `persist_thinkloop(metrics)` | Sync write to `{metrics_dir}/thinkloop.jsonl` |

**Persistence path**: `data/metrics/{mind_id}/thinkloop.jsonl`

### AnomalyDetector (anomaly.rs)

Configurable threshold alerting.

| Default Threshold | Warning | Critical |
|-------------------|---------|----------|
| thinkloop_iterations | 10 | 15 |
| thinkloop_duration_ms | 30,000 | 60,000 |
| thinkloop_avg_iteration_time_ms | 10,000 | 20,000 |
| thinkloop_challenger_warnings | 3 | 5 |
| tool_latency_ms | 5,000 | 15,000 |
| challenger_warning | 5 | 10 |

| Method | Description |
|--------|-------------|
| `new(mind_id, alerts_dir)` | Initialize with defaults |
| `set_threshold(name, config)` | Override a threshold |
| `check(metric, value) → Vec<Alert>` | Check value against thresholds, return any alerts |
| `recent_alerts(count)` | Last N alerts |
| `summary()` | Count by severity (HashMap<String, usize>) |

Alert severity: `Info`, `Warning`, `Critical`.

### MetricsExporter (export.rs)

Reads persisted JSONL and produces formatted summary reports.

| Method | Description |
|--------|-------------|
| `new(metrics_dir)` | Initialize exporter |
| `read_thinkloop()` | Parse all ThinkLoopEntry from JSONL, sorted by timestamp |
| `summary_report()` | ASCII-art box report: total turns, iterations, tool calls, completion %, stall kills, challenger warnings, avg duration, model breakdown |

### Dependencies
- Internal: `codex-llm`, `codex-exec`, `codex-memory`, `codex-redteam`, `codex-fitness`
- External: `serde`, `serde_json`, `tracing`, `chrono`, `tokio`

**Note**: cortex-monitoring depends on 5 internal crates — it's a high-fan-in crate that aggregates data from the entire system. This makes sense for monitoring but means it can't compile without all those crates being healthy.

### Test Coverage (0 tests)
No tests exist for cortex-monitoring. This is a significant gap.

---

## 5. Cross-Crate Dependencies & Data Flow

```
                    ┌──────────────┐
                    │  codex-roles │  (Role enum — shared foundation)
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
     ┌────────────┐  ┌──────────┐  ┌───────────────┐
     │codex-fitness│  │codex-    │  │cortex-        │
     │(scoring)   │  │redteam   │  │monitoring     │
     │            │  │(verify)  │  │(observe)      │
     └─────┬──────┘  └─────┬────┘  └───────┬───────┘
           │               │               │
           │               │      ┌────────┼────────────────┐
           │               │      │        │                │
           └───────────────┼──────┘   ┌────▼────┐    ┌──────▼──────┐
                           │          │codex-llm│    │codex-memory │
                           │          │(model)  │    │(persistence)│
                           │          └─────────┘    └─────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  codex-exec  │  (tool execution — context for verification)
                    └──────────────┘
```

### Data Flow: Task Completion Verification Pipeline

```
1. Agent completes task
       │
2. ThinkLoop records ThinkLoopMetrics
       │
3. MetricsCollector.record_thinkloop() → ring buffer + JSONL
       │
4. AnomalyDetector.check() → alerts if thresholds breached
       │
5. Challenger.check() → structural warnings (7 checks)
       │
6. If completion claimed → RedTeamProtocol.verify() → Approved/Challenged/Blocked
       │
7. compute_fitness(outcome) → RoleFitness with composite score
       │
8. MetaEvolution.record_routing_accuracy() → trend tracking
       │
9. Dream Mode (nightly) consumes ChallengerMetrics.cross_task_patterns()
       → evolves thresholds, catches blind spots, recalibrates checks
```

---

## 6. Interface Boundaries — What I Need From Other Agents

| Dependency | From Agent | What I Need | Status |
|-----------|-----------|-------------|--------|
| `Role` enum | mind-coordination (codex-roles) | Role type for all role-aware logic | **HAVE** — working |
| `TaskOutcome` type placement | mind-coordination (codex-types) | Should `TaskOutcome` move to codex-types for sharing? Currently defined in codex-fitness | **DECISION NEEDED** |
| ThinkLoop integration | mind-model-router (codex-drive) | Drive loop must call `Challenger.check()` after each iteration and on final response | **NOT WIRED** |
| ThinkLoop metrics | mind-model-router (codex-drive) | Drive loop must emit `ThinkLoopMetrics` for `MetricsCollector.record_thinkloop()` | **NOT WIRED** |
| Tool call records | mind-tool-engine (codex-exec) | `ToolExecutor` must produce `ChallengerToolCall` structs with result_text | **NOT WIRED** |
| Memory access | mind-memory (codex-memory) | Persist fitness trajectories, load historical baselines | **NOT WIRED** |
| Event integration | mind-hooks | Hook events (pre/post tool use) as Challenger integration points | **NOT WIRED** — see mind-hooks scratchpad entry |

---

## 7. What I Provide to Other Agents

| Module | Export | Consumer | Description |
|--------|--------|----------|-------------|
| codex-fitness | `compute_fitness()` | mind-coordination | Compute role-specific fitness from task outcomes |
| codex-fitness | `RoleFitness.composite()` | mind-coordination | Single 0.0–1.0 score for delegation decisions |
| codex-fitness | `MetaEvolution` | Dream Mode (mind-memory) | Track if improvement process itself is improving |
| codex-redteam | `Challenger` | mind-model-router (ThinkLoop) | Per-turn structural verification, 7 checks |
| codex-redteam | `Challenger.should_kill_stall()` | mind-model-router (ThinkLoop) | Kill switch for stalled think loops |
| codex-redteam | `RedTeamProtocol` | mind-coordination | LLM-based completion verification |
| codex-redteam | `ChallengerMetrics` | Dream Mode (mind-memory) | Learning loop data for threshold evolution |
| codex-redteam | `StateFileSchema` trait | any crate | Pluggable state file validation |
| cortex-monitoring | `MetricsCollector` | mind-model-router, mind-tool-engine | Record metrics from drive loop and tool execution |
| cortex-monitoring | `AnomalyDetector` | mind-coordination | Alert when metrics breach thresholds |
| cortex-monitoring | `MetricsExporter` | mind-tui, external dashboards | Generate summary reports from persisted JSONL |

---

## 8. Gaps, Issues, and Phase 4 Priorities

### Critical Gaps

| # | Gap | Impact | Priority |
|---|-----|--------|----------|
| G1 | **No ThinkLoop integration** — Challenger exists but isn't called by the drive loop | Verification is dead code until wired | HIGH |
| G2 | **No MetricsCollector integration** — collector exists but nothing feeds it metrics | Monitoring is dead code until wired | HIGH |
| G3 | **0 tests in cortex-monitoring** — all metric types, collector, anomaly detector, exporter untested | Refactoring risk, silent breakage | HIGH |
| G4 | **Fitness placeholders** — team_lead_utilization, context_efficiency are hardcoded | Fitness scores not meaningful for Primary/TeamLead | MEDIUM |
| G5 | **No integration tests** — workspace `tests/` directory doesn't exist | Cross-crate interactions untested | MEDIUM |
| G6 | **No fitness persistence** — scores computed but never stored | Can't track improvement over time | MEDIUM |
| G7 | **ChallengerMetrics not persisted** — cross-task patterns lost between sessions | Dream Mode can't evolve thresholds | MEDIUM |

### Suggested Priority Order

1. **Wire Challenger to ThinkLoop** (G1) — mind-model-router integration
2. **Write cortex-monitoring tests** (G3) — foundation before wiring
3. **Wire MetricsCollector** (G2) — mind-model-router + mind-tool-engine integration
4. **Create workspace integration tests** (G5) — cross-crate verification
5. **Implement fitness persistence** (G6) — via codex-memory SQLite
6. **Replace fitness placeholders** (G4) — needs cross-task aggregation from cortex-monitoring
7. **Persist ChallengerMetrics** (G7) — JSONL or SQLite for Dream Mode consumption

### Check 6 (StateFileIntegrity) — Not Yet Implemented in Challenger

The `ChallengerCheck::StateFileIntegrity` variant exists in the enum, and the `StateFileSchema` trait + `verify_state_file()` function exist, but Check 6 is not wired into `Challenger.check()`. This needs:
1. A way to register schemas with the Challenger
2. Integration into the check pipeline (probably on final response only)
3. Default schemas for known state files (evolution.json, fitness.json, config.json)

---

## 9. Upstream Cherry-Pick Targets

### From Codex Deep-Map

No direct cherry-pick responsibility for mind-testing (unlike mind-hooks/mind-tool-engine/mind-mcp/mind-skills). However, relevant patterns to study:

| Source | Lines | What to Learn |
|--------|-------|---------------|
| `codex-upstream/codex-rs/core/tests/` | ~50 test files | Test organization patterns: `suite/` submodules, `common/` test helpers, `all.rs` entry points |
| `codex-upstream/codex-rs/exec/tests/` | ~10 test files | Sandbox testing, ephemeral agent testing — relevant for RedTeam read-only agent verification |
| `codex-upstream/codex-rs/otel/` | ~500 lines | OpenTelemetry export — study for cortex-monitoring export upgrade |

### Codex Test Infrastructure Patterns (Worth Adopting)

From the upstream `tests/` directories:
- **`all.rs` entry point** per crate — single integration test binary
- **`suite/` module** — each test scenario in its own file
- **`common/` module** — shared test helpers (mock servers, fixtures, assertion helpers)
- **`harness` module** — reusable test harnesses for tool execution

---

## 10. Relationship to Other Mind-Maps

| Mind-Map | Relationship to testing |
|----------|----------------------|
| **tool-engine** | Challenger classifies tools by role — needs ToolCall/ToolResult types. mind-tool-engine's ToolExecutor should emit ChallengerToolCall records. |
| **hooks** | mind-hooks proposed `PreToolUse`/`PostToolUse` events align with Challenger's `check()` entry points. Hook dispatcher wraps tool execution; Challenger verifies after. |
| **model-router** | ThinkLoop in codex-drive is WHERE Challenger.check() must be called. This is the primary integration point. |
| **coordination** | MindManager delegates tasks → task outcomes feed compute_fitness(). TaskLedger tracks completion → triggers RedTeamProtocol.verify(). |
| **memory** | Dream Mode consumes ChallengerMetrics. Fitness trajectories stored in memory. Historical baselines inform anomaly thresholds. |
