# Coordination Domain — Mind Map

**Owner**: mind-coordination
**Date**: 2026-04-16
**Status**: Phase 1 (Foundation) — all crates compile, architecture documented

---

## 1. What Exists — File-by-File Inventory

### codex-types (Shared Type Definitions — ~270 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/codex-types/src/lib.rs` | ~270 | **THE** shared event types consumed by 5+ crates |

**Key types defined here:**
- `MindEvent` — tagged union: `External(ExternalEvent)` | `Drive(DriveEvent)`. This is the universal event flowing through the EventBus.
- `ExternalEvent` — source + content + priority + timestamp + metadata
- `EventSource` — Hub, Telegram, Boop, SubMindResult, Human, Schedule, Ipc (7 sources)
- `EventPriority` — Low < Normal < High < Critical (Ord-derived for comparison)
- `DriveEvent` — 4 variants: TaskAvailable, StallDetected, IdleSuggestion, HealthCheck
  - `prompt(&self, role_str)` — generates role-appropriate prompts; hard-codes "DO NOT execute directly" for Primary/TeamLead
  - `reaches_primary(&self)` — 90% autonomic filtering: only StallDetected and Critical tasks surface to Primary
  - `priority(&self)` — maps drive events to EventPriority

**Critical design insight**: codex-types has NO internal dependencies (only serde + chrono). It's a true leaf crate. This is intentional — keeps the dependency graph clean.

**Note**: There's a type duplication issue. codex-coordination/types.rs defines its OWN `MindId`, `MindStatus`, `MindHandle`, `Task`, `TaskResult`, `RoutingDecision`, `ExternalInput`, `InputSource`, `InputPriority`, `CoordinationState`. These overlap with but are DISTINCT from codex-types' `MindEvent`/`EventSource`/`EventPriority`. The two type systems currently coexist without unification.

---

### codex-roles (Role Permission System — ~300 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/codex-roles/src/lib.rs` | ~300 | Hard-coded role enforcement (3 layers) |

**Key types and functions:**
- `Role` — Primary | TeamLead | Agent (the 3 fractal levels)
- `Vertical` — Research | Code | Memory | Comms | Ops | Context | Custom(String)
- `tools_for_role(role)` — returns the exact tool whitelist for each role
  - Primary: 8 coordination tools (no bash, no files, no web)
  - TeamLead: 10 delegation tools (no bash, no files, no web)
  - Agent: wildcard `*` (full access)
- `is_tool_allowed(role, tool_name)` — gate function
- `filter_tools(role, all_tools)` — build filtered tool set from full registry
- `SandboxLevel` — ReadOnlyCoordination | TeamScoped | WorkspaceWrite | ReadOnly
- `ExecPolicyLevel` — DenyAll | DenyExceptIpc | Sandboxed
- `RoleEnforcement` — complete config struct: role + allowed_tools + sandbox + exec_policy

**3-layer enforcement design:**
1. **Tool Registry** — LLM never sees disallowed tools
2. **Exec Policy** — if tool call slips through, policy blocks it
3. **Sandbox Policy** — kernel-level Landlock/seccomp as final defense

---

### codex-coordination (Multi-Agent Orchestration — ~2,400 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/codex-coordination/src/lib.rs` | ~36 | Module root + re-exports |
| `src/codex-coordination/src/types.rs` | ~202 | Coordination-specific types |
| `src/codex-coordination/src/coordinator.rs` | ~234 | CoordinatorLoop — top-level orchestrator |
| `src/codex-coordination/src/mind_manager.rs` | ~487 | MindManager — registry + factory for all minds |
| `src/codex-coordination/src/process_bridge.rs` | ~556 | ProcessBridge — maps MindIds to live OS processes |
| `src/codex-coordination/src/task_ledger.rs` | ~252 | TaskLedger — persistent JSONL delegation audit trail |
| `src/codex-coordination/src/input_mux.rs` | ~229 | InputMux — routes inputs to correct mind |
| `src/codex-coordination/src/planning.rs` | ~170 | PlanningGate — complexity-scaled planning |
| `src/codex-coordination/src/triggers.rs` | ~213 | TriggerEngine + RotationTrigger — dynamic spawning |

#### types.rs — Coordination-Specific Types
- `MindId(String)` — `MindId::new(role, vertical)` generates deterministic IDs: "primary", "research-lead", "code-agent-abc12345"
- `MindStatus` — Initializing | Idle | Active{task_id} | WaitingForResult{waiting_on} | ShuttingDown | Terminated
- `MindHandle` — id, role, vertical, status, parent, children, session_count, created_at, last_active, growth_stage
- `GrowthStage` — Novice(<10) | Competent(10-50) | Proficient(50-200) | Advanced(200-500) | Expert(500+)
- `Task` — id, description, source, target, complexity, created_at
- `TaskResult` — task_id, mind_id, summary, evidence, learnings, completed_at
- `RoutingDecision` — Direct(MindId) | Escalate | Drop{reason}
- `ExternalInput` / `InputSource` / `InputPriority` — parallel to codex-types but coordination-specific

#### coordinator.rs — CoordinatorLoop
**The top-level integration point.** Combines all subsystems:
```
CoordinatorLoop
├── MindManager      (mind lifecycle)
├── InputMux         (input routing)
├── PlanningGate     (complexity scaling)
├── TriggerEngine    (auto-spawning)
└── RotationTrigger  (3-hour scratchpad rotation)
```

**Key methods:**
- `new(agents_dir, scratchpads_dir)` — initializes Primary mind + all subsystems
- `process_input(input)` — full pipeline: InputMux → PlanningGate → delegate/escalate/drop
- `spawn_team_lead(vertical, objective)` — creates team lead mind
- `check_triggers()` — evaluates blocking + rotation triggers
- `state()` — coordination state snapshot

**Pipeline flow (process_input):**
1. InputMux routes input → Direct(target) | Escalate | Drop
2. PlanningGate evaluates complexity → Execute | ExecuteWithPlan | SpawnPlanner | SpawnCompetingPlanners
3. Task is delegated via MindManager

#### mind_manager.rs — MindManager
**The registry and factory for ALL active minds.** This is the equivalent of Claude Code's `TeamCreate`.

**Key methods:**
- `init_primary()` — creates Primary mind (called once at startup)
- `spawn_team_lead(vertical, objective)` — creates team lead (Primary-only)
- `spawn_agent(parent_id, agent_type, task)` — creates agent (TeamLead-only)
- `delegate(from, to, task_description)` — creates Task, updates target status
- `complete_task(result)` — records completion, updates statuses
- `shutdown_mind(mind_id)` — graceful shutdown (refuses if active children exist)
- `find_team_lead(vertical)` — lookup by vertical
- `count_by_role(role)` — counting
- `coordination_state()` — full state snapshot

**Spawn constraints (hard-enforced):**
- Only Primary can spawn TeamLeads
- Only TeamLeads can spawn Agents
- Primary CANNOT spawn Agents directly
- Duplicate verticals are rejected
- Shutdown with active children is refused

#### process_bridge.rs — ProcessBridge
**The runtime layer that maps logical MindIds to actual OS processes.** This bridges MindManager (logical state) to real child processes.

```
MindManager (logical state)
└── ProcessBridge (runtime)
    ├── MindId("research-lead") → ChildMind { process, mcp_client }
    ├── MindId("code-lead")     → ChildMind { process, mcp_client }
    └── MindId("agent-abc123")  → ChildMind { process, mcp_client }
```

**Key methods:**
- `spawn(mind_id, role)` — launches `cortex --serve --mind-id X --role Y`, connects via StdioTransport, performs MCP handshake
- `spawn_thinking(mind_id, role)` — same but adds `--think` flag for ThinkLoop-enabled children
- `delegate(mind_id, task_id, description, context, parent)` — delegates via MCP, records in TaskLedger + TaskStore, handles crash-respawn
- `delegate_parallel(tasks)` — concurrent delegation to multiple children (extracts children from HashMap, spawns concurrent tasks, returns them)
- `shutdown(mind_id)` — MCP shutdown → wait for exit (10s timeout → kill)
- `shutdown_all()` — graceful shutdown of all children
- `status(mind_id)` / `list_tools(mind_id)` / `call_tool(mind_id, name, args)` — MCP passthrough

**Crash recovery:** If delegation fails with a transport error (broken pipe / child crash), ProcessBridge automatically:
1. Kills the dead child
2. Respawns it (with same thinking flag)
3. Retries the delegation once

**Integration points:**
- `with_ledger(TaskLedger)` — JSONL audit trail
- `with_task_store(TaskStore)` — SQLite state + dependency tracking
- `with_completion_sender(watch::Sender)` — notifies DriveLoop on task completion

#### task_ledger.rs — TaskLedger
**Append-only JSONL audit trail** at `data/tasks/ledger.jsonl`.
- `record_delegation(task_id, mind_id, parent, description)` — writes Delegated entry
- `record_completion(task_id, mind_id, parent, desc, succeeded, iterations, tool_calls, summary)` — writes Completed/Failed entry
- `read_all()` / `entries_for_task(task_id)` / `summary()` — read operations

#### input_mux.rs — InputMux ("The Subconscious")
**Routes inputs WITHOUT reaching Primary's context.** Only surfaces what requires attention.

Hard-coded routes:
| Source | Target |
|--------|--------|
| Hub #general | comms-lead |
| Hub #protocol | research-lead |
| BOOP timers | ops-lead |
| Scheduled tasks | ops-lead |
| Human input | ALWAYS escalates to Primary |
| Critical priority | ALWAYS escalates to Primary |
| Unknown | Escalates (conservative default) |

#### planning.rs — PlanningGate ("Go Slow to Go Fast")
5 complexity levels → proportional planning:

| Complexity | Planning Depth | Time Budget |
|------------|---------------|-------------|
| Trivial (≤5 words) | Memory check only | < 1s |
| Simple (6-15 words) | Memory + brief plan | 2-5s |
| Medium (16-40 words) | Competing hypotheses | 10-30s |
| Complex (41-80 words or keywords) | Spawn planner sub-mind | 30s-5m |
| Novel (80+ words) | Spawn competing planners | 1-10m |

**Current implementation**: Heuristic (word count + keyword detection). Production: LLM classification via M2.7.

#### triggers.rs — TriggerEngine + RotationTrigger
8 trigger types for dynamic spawning:
- `PatternRepetition` — same problem 3+ times → spawn specialist
- `TaskComplexityExceeded` — beyond planning gate threshold
- `CompetingHypotheses` — multiple valid approaches → parallel thinkers
- `BlockingDetected` — mind stuck >120s → spawn fresh context
- `DomainBoundary` — task crosses verticals
- `VerificationNeed` — completion claimed, needs red-team
- `ContextPressure` — context >85% → spawn overflow mind
- `Scheduled` — time-based triggers

`RotationTrigger` — 3-hour scratchpad rotation cycle.

---

### codex-ipc (Inter-Process Communication — ~1,500 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/codex-ipc/src/lib.rs` | ~35 | Module root + re-exports |
| `src/codex-ipc/src/protocol.rs` | ~297 | JSON-RPC 2.0 + Cortex-specific protocol types |
| `src/codex-ipc/src/client.rs` | ~239 | McpMindClient — connects to child minds |
| `src/codex-ipc/src/server.rs` | ~711 | McpMindServer — exposes mind as MCP server |
| `src/codex-ipc/src/transport.rs` | ~227 | MindTransport trait + 3 implementations |

#### protocol.rs — Wire Protocol
**JSON-RPC 2.0 base** + Cortex-specific methods:

Standard MCP methods:
- `initialize` — handshake (protocol version, capabilities, info)
- `tools/list` — list available tools (role-filtered)
- `tools/call` — execute a tool

Cortex-specific methods:
- `cortex/delegate` — assign task with DelegateParams → DelegateResult
- `cortex/status` — report mind status → StatusResult
- `cortex/shutdown` — graceful shutdown

**DelegateResult** includes: accepted, mind_id, task_id, response (ThinkLoop output), iterations, tool_calls_made, completed

#### client.rs — McpMindClient<T: MindTransport>
Generic over transport. Methods: initialize, list_tools, call_tool, delegate, status, shutdown.

#### server.rs — McpMindServer
Handles incoming requests. Key innovation: **DelegateHandler trait**.

When a DelegateHandler is provided, `cortex/delegate` actually THINKS (via ThinkLoop) about the task instead of just acknowledging it. This is the bridge between IPC and LLM reasoning.

Also provides `cortex_coordination_tools(role)` — generates MCP tool definitions based on role.

#### transport.rs — Transport Layer
```
MindTransport (trait)
├── ChannelTransport     — in-process tokio channels (testing)
├── StdioTransport       — client-side, reads child process stdout/stdin
└── StdioServerTransport — server-side, reads own stdin/stdout
```

All IPC is line-delimited JSON over these transports (matches MCP stdio spec).

---

### cortex (Main Binary — ~6,000+ lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/cortex/src/main.rs` | ~3,400+ | Entry point: demo, serve, daemon modes |
| `src/cortex/src/boot.rs` | ~442 | BootContext — session continuity |
| `src/cortex/src/config.rs` | ~480 | CortexConfig from config.toml |
| `src/cortex/src/drive.rs` | ~345 | DriveHandles — EventBus + DriveLoop integration |
| `src/cortex/src/input_route.rs` | ~250 | InputRouteInterceptor — exposes InputMux as tool |
| `src/cortex/src/progress.rs` | ~301 | ProgressInterceptor — mid-task progress reporting |
| `src/cortex/src/task_history.rs` | ~188 | TaskHistoryInterceptor — query delegation history |
| `src/cortex/src/qwen_delegate.rs` | ~226 | QwenDelegate — delegate to Qwen via Ollama |
| `src/cortex/src/monitoring.rs` | ~27 | Bridges cortex-monitoring into main binary |
| `src/cortex/build.rs` | - | Build script |
| `src/cortex/src/bin/*.rs` | ~various | 11 binary targets (proofs, chat, etc.) |

#### main.rs — Three Modes

1. **Demo mode** (default) — 23-phase lifecycle demonstration
2. **Serve mode** (`--serve`) — MCP server on stdin/stdout (child mind)
3. **Daemon mode** (`--daemon`) — autonomous event loop with DriveLoop + ThinkLoop

**Daemon mode is the production path.** It:
- Loads config, memory, boot context
- Boots drive subsystem (TaskStore + EventBus + DriveLoop)
- Creates ThinkLoop with configurable model
- Registers interceptors: input_route, progress, task_history, hub, search, elevenlabs, image_gen, delegation
- Seeds tasks from CLI args or JSON file
- Enters event loop: receives MindEvents, generates prompts, runs ThinkLoop, handles tool calls

#### boot.rs — Session Continuity
BootContext loads at startup:
1. Identity from `agents/{role}/AGENTS.md`
2. Last handoff from `data/handoffs/{mind_id}/`
3. Scratchpad from `data/scratchpad/{mind_id}-{date}.md`
4. Recent memories from MemoryStore (SQLite FTS5)

Also: `rollover_scratchpads()` archives stale scratchpads, `write_handoff()` persists session state, `record_fitness()` tracks task outcomes.

#### config.rs — Configuration
CortexConfig from `config/config.toml`: model providers (Ollama local/cloud), model assignments per role, coordination thresholds, dream schedule, suite URLs.

#### drive.rs — Drive Integration
Wires EventBus + DriveLoop into Cortex:
- `boot(project_root, mind_id, role)` — serve mode (background event handler)
- `boot_daemon(project_root, mind_id, role, config)` — daemon mode (returns EventBus to caller)
- `handle_drive()` — processes drive events (TaskAvailable, StallDetected, IdleSuggestion, HealthCheck)

#### Interceptors (exposed as ThinkLoop tools)
- **InputRouteInterceptor** — `input_route` tool: "where should this input go?"
- **ProgressInterceptor** — `report_progress` + `check_progress` tools
- **TaskHistoryInterceptor** — `task_history` tool: "what have we done?"
- **QwenDelegateTool** — `qwen_delegate` tool: delegate to Qwen via Ollama API

---

## 2. How the Big Three Work Together

### Coordinator → MindManager → ProcessBridge

```
                    ┌─────────────────────────┐
                    │     CoordinatorLoop      │
                    │  (Logic + Routing Layer) │
                    └─────────┬───────────────┘
                              │ uses
                              ▼
                    ┌─────────────────────────┐
                    │      MindManager         │
                    │  (Logical State Layer)   │
                    │  HashMap<MindId, Handle> │
                    └─────────┬───────────────┘
                              │ mirrors
                              ▼
                    ┌─────────────────────────┐
                    │     ProcessBridge        │
                    │  (Runtime Process Layer) │
                    │  HashMap<MindId, Child>  │
                    └─────────────────────────┘
```

**CoordinatorLoop** is the top integration. It owns a MindManager, InputMux, PlanningGate, and TriggerEngine. When `process_input()` is called, it routes the input, evaluates complexity, and delegates via MindManager.

**MindManager** tracks the logical state of all minds (who exists, what role, what status, parent-child relationships). It enforces spawn constraints (only Primary → TeamLead, only TeamLead → Agent). It does NOT manage OS processes.

**ProcessBridge** is the runtime mirror. It maps MindIds to actual child processes connected via MCP over stdio. When MindManager says "delegate to research-lead", ProcessBridge handles: spawn the process, establish MCP handshake, send the delegation, handle crash recovery, record to TaskLedger/TaskStore.

**Current gap**: CoordinatorLoop and ProcessBridge are not yet wired together in the daemon's event loop. The daemon's main.rs uses a DelegationInterceptor (defined inline in main.rs) that directly calls ProcessBridge, bypassing CoordinatorLoop. The CoordinatorLoop is fully functional in unit tests but isn't yet the daemon's entry point.

---

## 3. What's the Equivalent of TeamCreate?

**MindManager.spawn_team_lead(vertical, objective)** is the TeamCreate equivalent.

It:
1. Validates only Primary can spawn team leads
2. Checks for duplicate verticals (rejects if one already active)
3. Creates a MindHandle with Role::TeamLead, assigns vertical
4. Registers as child of Primary
5. Builds RoleEnforcement (filtered tool set, sandbox level, exec policy)
6. Returns the MindId (deterministic: "{vertical}-lead")

**ProcessBridge.spawn(mind_id, role)** is the runtime companion — it actually launches `cortex --serve --mind-id X --role Y` and establishes the MCP connection.

**MindManager.shutdown_mind(mind_id)** is the TeamDelete equivalent. It:
1. Refuses if mind has active (non-terminated) children
2. Sets status to Terminated
3. Removes from parent's children list

**ProcessBridge.shutdown(mind_id)** does the runtime cleanup: sends MCP shutdown, waits 10s, kills if needed.

---

## 4. Cross-Module Connections

### Dependencies (what coordination imports)

```
codex-coordination depends on:
├── codex-roles      (Role, Vertical, RoleEnforcement)
├── codex-fitness    (used in Cargo.toml but not deeply in coordination code)
├── codex-ipc        (McpMindClient, StdioTransport — in ProcessBridge)
├── codex-drive      (TaskStore, StoredTask, TaskState, TaskPriority — re-exported from lib.rs)
└── codex-types      (NOT directly used in codex-coordination — it has its own types.rs)
```

### How coordination connects to OTHER modules

| Module | Connection Point | How |
|--------|-----------------|-----|
| **codex-exec** (mind-tool-engine) | Server uses `ToolExecutor` for tools/call | McpMindServer.handle_tools_call() calls `exec.execute(&tool_call, role)` |
| **codex-llm** (mind-model-router) | ThinkLoop drives LLM reasoning | Cortex main.rs creates ThinkLoop + ToolInterceptors that use coordination |
| **codex-drive** (mind-model-router) | DriveLoop + EventBus + TaskStore | ProcessBridge uses TaskStore; drive.rs wires EventBus into cortex; DriveLoop emits MindEvents |
| **codex-memory** (mind-memory) | BootContext loads recent memories | boot.rs queries MemoryStore at startup |
| **codex-dream** (mind-memory) | Dream engine for offline learning | Used in cortex main.rs demo mode |
| **codex-redteam** (mind-testing) | Challenger for completion verification | Used in cortex main.rs |
| **codex-patcher** (mind-hooks) | Session patches, Qwen interceptor | Listed in cortex Cargo.toml dependency |
| **cortex-monitoring** (mind-testing) | Metrics, anomaly detection | monitoring.rs bridges it into cortex |
| **codex-suite-client** (mind-mcp) | Hub, search, ElevenLabs, ImageGen interceptors | Used in daemon mode as ThinkLoop tools |

### Type Duplication Issue (Important)

There are TWO parallel type hierarchies:

1. **codex-types** (`MindEvent`, `ExternalEvent`, `EventSource`, `EventPriority`, `DriveEvent`) — used by codex-drive's EventBus
2. **codex-coordination/types.rs** (`MindId`, `ExternalInput`, `InputSource`, `InputPriority`, `MindHandle`, etc.) — used by MindManager/InputMux/CoordinatorLoop

These define similar concepts (InputSource vs EventSource, InputPriority vs EventPriority) but are NOT unified. The daemon converts between them at integration points. This is a known debt that should be resolved when moving to Phase 2.

---

## 5. What's Missing for Full Team Coordination

### Missing: Real LLM integration in PlanningGate
- Currently uses word-count heuristics for complexity classification
- Needs M2.7 LLM call to properly classify task complexity

### Missing: Learned routing in InputMux
- Currently all hard-coded routes
- Design mentions "learned over time" but no ML pipeline exists
- Need: Pattern tracking → routing rule generation

### Missing: CoordinatorLoop → daemon integration
- CoordinatorLoop works in unit tests but daemon bypasses it
- The daemon's DelegationInterceptor is a manual bridge
- Need: Replace DelegationInterceptor with CoordinatorLoop driving the daemon

### Missing: Inter-mind message passing
- `send_message` is in the tool registry but has no implementation
- Minds can delegate tasks but can't send arbitrary messages to each other
- Need: Message queue or pub/sub between minds

### Missing: Context window tracking
- `ContextPressure` trigger exists but no token counting
- Need: Token counter per mind → triggers overflow spawning

### Missing: A2A protocol (Gemini CLI pattern)
- Identified in MISSIONS.md Phase 3
- Currently all IPC is parent↔child. No peer-to-peer between team leads
- Need: HTTP or WebSocket based agent-to-agent communication for inter-civ

### Missing: Persistent mind state across restarts
- MindManager is in-memory HashMap
- If the daemon crashes, all mind state is lost
- Need: Persist MindHandle hierarchy to SQLite or file

### Missing: Dynamic vertical creation
- Vertical enum has `Custom(String)` but CoordinatorLoop doesn't use it
- Need: Runtime vertical registration for new team domains

---

## 6. Recommended Build Order

### Phase 1 (Current — Verify & Document)
1. ✅ Verify all crates compile
2. ✅ Document interfaces (this mind-map)
3. ◻️ Unify type duplication (codex-types vs codex-coordination/types.rs)

### Phase 2 (Wire It Together)
1. **Wire CoordinatorLoop into daemon mode** — replace DelegationInterceptor with CoordinatorLoop as the daemon's core
2. **Implement send_message** — message passing between minds (add to IPC protocol)
3. **Persist MindManager state** — serialize mind hierarchy to SQLite
4. **Add token counting** — track context usage per mind for ContextPressure trigger

### Phase 3 (Smarten It)
1. **LLM-powered PlanningGate** — replace heuristics with M2.7 classification
2. **Learned InputMux routing** — pattern tracking → routing rule updates
3. **A2A protocol** — peer-to-peer communication between team leads and across civs

### Phase 4 (Scale It)
1. **Dynamic vertical creation** — runtime registration of new team domains
2. **Context overflow spawning** — auto-spawn when token pressure exceeds threshold
3. **Inter-civ coordination** — extend ProcessBridge for remote mind connections

---

## 7. Architecture Summary Diagram

```
                    ┌──────────────────────────────────────────┐
                    │              CORTEX BINARY                │
                    │  (demo | serve | daemon modes)           │
                    │                                          │
                    │  ┌─────────┐  ┌──────────┐ ┌─────────┐ │
                    │  │BootCtx  │  │  Config   │ │Monitoring│ │
                    │  └────┬────┘  └────┬─────┘ └────┬────┘ │
                    │       │            │            │       │
                    │  ┌────▼────────────▼────────────▼────┐ │
                    │  │         ThinkLoop + Interceptors    │ │
                    │  │  (input_route, progress, history,  │ │
                    │  │   qwen_delegate, hub, search, ...)  │ │
                    │  └────────────────┬──────────────────┘ │
                    └───────────────────┼──────────────────────┘
                                        │ uses
                    ┌───────────────────▼──────────────────────┐
                    │           codex-coordination              │
                    │                                          │
                    │  ┌─────────────────────────────────────┐ │
                    │  │         CoordinatorLoop              │ │
                    │  │  ┌──────────┐  ┌─────────────────┐  │ │
                    │  │  │InputMux  │  │ PlanningGate    │  │ │
                    │  │  │(routing) │  │ (complexity)    │  │ │
                    │  │  └──────────┘  └─────────────────┘  │ │
                    │  │  ┌──────────┐  ┌─────────────────┐  │ │
                    │  │  │TriggerEng│  │ RotationTrigger │  │ │
                    │  │  │(spawning)│  │ (3hr cycle)     │  │ │
                    │  │  └──────────┘  └─────────────────┘  │ │
                    │  └──────────────────┬──────────────────┘ │
                    │                     │ uses                │
                    │  ┌──────────────────▼──────────────────┐ │
                    │  │          MindManager                 │ │
                    │  │  (logical state: who exists,         │ │
                    │  │   parent-child, roles, tasks)        │ │
                    │  └──────────────────┬──────────────────┘ │
                    │                     │ mirrors             │
                    │  ┌──────────────────▼──────────────────┐ │
                    │  │          ProcessBridge               │ │
                    │  │  (runtime: child processes,          │ │
                    │  │   MCP connections, crash recovery)   │ │
                    │  └──────────────────┬──────────────────┘ │
                    │                     │ records to          │
                    │  ┌──────────────────▼──────────────────┐ │
                    │  │          TaskLedger                  │ │
                    │  │  (JSONL audit trail)                 │ │
                    │  └─────────────────────────────────────┘ │
                    └──────────────────────────────────────────┘
                                        │
              ┌─────────────────────────┼──────────────────────────┐
              │                         │                          │
    ┌─────────▼──────┐       ┌─────────▼──────┐       ┌──────────▼────────┐
    │  codex-roles   │       │   codex-ipc    │       │   codex-types     │
    │ (permissions)  │       │ (MCP protocol) │       │ (shared events)   │
    │ Role, Vertical │       │ Client/Server  │       │ MindEvent, Drive  │
    │ tool filtering │       │ Transport      │       │ EventPriority     │
    └────────────────┘       └────────────────┘       └───────────────────┘
              │                         │
              └─────── DEPENDS ON ──────┘
                      codex-drive
                  (TaskStore, EventBus,
                   DriveLoop, DriveConfig)
```

---

## 8. Test Coverage

All modules have inline `#[cfg(test)]` tests:
- **codex-coordination**: 14 tests (coordinator init, spawn, delegate, shutdown lifecycle)
- **codex-ipc**: 15+ tests (protocol serialization, client-server pairs, full lifecycle, delegate handler)
- **codex-roles**: 10 tests (role permissions, tool filtering, sandbox levels)
- **codex-types**: 8 tests (drive event prompts, priority routing, serialization roundtrip)
- **cortex**: 15+ tests (boot context, handoffs, scratchpad rollover, fitness recording, interceptors)

Tests use `ChannelTransport` pairs for IPC testing (no real processes needed).
