# Mind-Map: Hooks Module (mind-hooks)

**Owner**: mind-hooks
**Crate**: `src/codex-patcher/`
**Date**: 2026-04-16

---

## 1. What Exists Today

### Crate: `codex-patcher` (~1,600 lines, 7 files)

The current module is a **patch-generation system**, not a hooks engine. It generates unified diffs that inject Cortex behavior into upstream Codex files at build time.

#### File Inventory

| File | Lines | Purpose |
|------|-------|---------|
| `lib.rs` | 73 | Module root. Defines `CortexPatch` and `PatchResult` structs. Exports 5 modules. |
| `patch_set.rs` | 358 | Orchestrator. `apply_all_patches()` applies 4 patches via the `patch` CLI command. Idempotency checks via `--dry-run --forward`. Revert support. Build script generation. |
| `agent_control_patch.rs` | 343 | **Patch 1**: Fractal delegation hierarchy. `validate_fractal_spawn()` enforces Primary→TeamLead→Agent depth limits. `CortexRole` enum + `CortexAgentMeta` struct. Generates a diff targeting `codex-rs/core/src/agent/control.rs`. |
| `session_patch.rs` | 130 | **Patch 2**: ThinkLoop injection. `ThinkLoopInjectionConfig` for wrapping Codex sessions with memory loading + Challenger verification. Generates a diff targeting `codex-rs/core/src/tasks/mod.rs`. |
| `memory_dual_write.rs` | 157 | **Patch 3**: Parallel persistence. `MemoryDualWriteConfig` for writing rollout items to both Codex JSONL and Cortex SQLite graph store. Generates diffs targeting `rollout/mod.rs` and `codex.rs`. |
| `sandbox_bridge.rs` | 221 | **Patch 4**: Role→sandbox mapping. `CortexSandboxLevel` enum maps Cortex roles (ReadOnlyCoordination, TeamScoped, WorkspaceWrite, ReadOnly) to Codex `SandboxPolicy` JSON. Generates a diff targeting `exec.rs`. |
| `qwen_interceptor.rs` | 277 | Qwen delegation tool. `QwenInterceptor` sends tasks to Qwen via Ollama `/api/chat`. Parses structured response (Summary/Findings/Evidence/Next). Not a patch — a standalone interceptor. |

#### Current Dependencies (Cargo.toml)

```
codex-coordination, codex-llm, codex-memory, codex-redteam,
codex-roles, codex-fitness, codex-dream
anyhow, tokio, serde, serde_json, tracing, thiserror, patch, diffy
```

Note: `reqwest` is used in `qwen_interceptor.rs` but NOT listed in Cargo.toml — this would fail to compile as-is.

#### Key Observation

**The patcher crate has NO lifecycle hook engine today.** It generates build-time diffs. The "hooks" concept exists only as:
- The agent-control patch (a spawn-time interception)
- The session patch (a ThinkLoop wrapper)
- The sandbox bridge (a tool-execution-time policy override)

These are **static compile-time injections**, not a **runtime hook dispatch system** like Codex has.

---

## 2. What Codex Has (5,553 lines — the cherry-pick target)

Source: `codex-deep-map.md` analysis of `codex-rs/hooks/`

### Architecture

Codex's hooks engine is a **runtime external-command dispatcher**. Hooks are configured executables that receive JSON payloads via stdin and return JSON responses via stdout.

### 5 Hook Event Types

| Event | When It Fires | Can Block? | Can Inject? | Can Stop? |
|-------|--------------|-----------|------------|----------|
| `session_start` | Session begins | No | No | No |
| `pre_tool_use` | BEFORE any tool execution | **Yes** (`should_block: true`) | No | No |
| `post_tool_use` | AFTER tool execution | No | **Yes** (`additional_contexts`) | **Yes** (`should_stop: true`) |
| `stop` | Session stops | No | No | No |
| `user_prompt_submit` | User submits a prompt | No | No | No |

### Hook Configuration (in `config.toml`)

```toml
[[hooks]]
event = "pre_tool_use"
command = "/path/to/hook-script"
tool_names = ["shell", "apply_patch"]  # optional: scope to specific tools
```

### Hook Payload (JSON via stdin)

```json
{
  "session_id": "...",
  "cwd": "/workspace",
  "event": "pre_tool_use",
  "tool_name": "shell",
  "tool_input": { "command": "rm -rf /" }
}
```

### Hook Response (JSON via stdout)

For `pre_tool_use`:
```json
{ "should_block": true, "reason": "Dangerous command detected" }
```

For `post_tool_use`:
```json
{
  "should_stop": false,
  "additional_contexts": ["Remember: files were modified in src/"],
  "feedback_message": "Tool executed successfully"
}
```

### Internal Components (estimated from 5,553 lines)

1. **Event definitions** — `HookEvent` enum with 5 variants + payload structs
2. **Hook dispatcher** — Matches events to configured hooks, manages execution
3. **Command runner** — Spawns external processes, pipes JSON stdin/stdout, handles timeouts
4. **Output parser** — Parses hook responses, validates schemas
5. **Tool-name filter** — Scopes hooks to specific tools (optional)
6. **Integration bridge** — `hook_runtime.rs` in core that fires events at the right points

### Boundary Quality: CLEAN

The deep-map assessment rates hooks as the **cleanest module boundary in Codex**. The only coupling point is `core/src/hook_runtime.rs` which bridges hooks into the core event loop. This makes it the ideal cherry-pick candidate.

---

## 3. What We Need to Build

### Phase 1: Hook Types & Trait Definitions (in `codex-types` or new `aiciv-hooks`)

Types that other crates consume. These go in the shared types layer.

```rust
// Hook event types (maps to Codex's 5 + our extensions)
enum HookEvent {
    SessionStart { session_id: String, config: SessionConfig },
    PreToolUse { tool_name: String, tool_input: Value, session_id: String },
    PostToolUse { tool_name: String, tool_input: Value, tool_output: Value, session_id: String },
    SessionStop { session_id: String, reason: StopReason },
    UserPromptSubmit { prompt: String, session_id: String },
    // aiciv-mind extensions (not in Codex):
    PreDelegation { target_mind: String, task: String },
    PostDelegation { target_mind: String, result: Value },
    MemoryWrite { namespace: String, content: String },
    DriveEvent { event: DriveEvent },
}

// Hook response types
enum HookResponse {
    PreToolUse { should_block: bool, reason: Option<String>, modified_input: Option<Value> },
    PostToolUse { should_stop: bool, additional_contexts: Vec<String>, feedback_message: Option<String> },
    PreDelegation { should_block: bool, modified_task: Option<String> },
    // Other events: no response expected (fire-and-forget)
    Ack,
}

// Hook configuration
struct HookConfig {
    event: HookEvent,
    command: String,        // external command path
    tool_names: Option<Vec<String>>,  // filter for tool-use events
    timeout_ms: u64,
    required: bool,         // if true, failure blocks the operation
}

// Hook registration trait — allows both external-command and in-process hooks
trait HookHandler: Send + Sync {
    async fn handle(&self, event: &HookEvent) -> Result<HookResponse>;
}
```

### Phase 2: Hook Dispatcher Engine

The core dispatcher that matches events to handlers and manages execution.

```
HookDispatcher
├── register(event_type, handler)     — register a handler for an event type
├── fire(event) → Vec<HookResponse>   — fire an event, collect all responses
├── fire_blocking(event) → Decision   — fire pre-* events, return block/allow
├── fire_async(event)                 — fire-and-forget for non-blocking events
└── External command runner            — spawns processes for external hooks
    ├── stdin JSON serialization
    ├── stdout JSON parsing
    ├── timeout enforcement
    └── error handling (fail-open vs fail-closed per config)
```

### Phase 3: Integration Points (what other modules need to do)

| Integration Point | Module Owner | What They Wire |
|-------------------|-------------|----------------|
| Pre/post tool execution | **mind-tool-engine** (`codex-exec`) | Call `dispatcher.fire_blocking(PreToolUse)` before `ToolExecutor::execute()`, `dispatcher.fire(PostToolUse)` after |
| Session start/stop | **mind-coordination** (`cortex`) | Call `dispatcher.fire(SessionStart)` during boot, `dispatcher.fire(SessionStop)` during shutdown |
| User prompt submit | **mind-model-router** (`codex-drive`) | Call `dispatcher.fire(UserPromptSubmit)` when external event arrives |
| Pre/post delegation | **mind-coordination** | Call `dispatcher.fire_blocking(PreDelegation)` before spawning sub-minds |
| Memory write events | **mind-memory** | Call `dispatcher.fire(MemoryWrite)` after persisting |
| Drive events | **mind-model-router** (`codex-drive`) | Call `dispatcher.fire(DriveEvent)` when DriveLoop emits |

---

## 4. Dependencies Map

### What hooks NEEDS from other modules

```
codex-types (mind-coordination)
├── HookEvent enum definition (or hooks defines its own + types re-exports)
├── ToolCallInfo struct (tool name, input, output)
├── SessionConfig / StopReason
└── DriveEvent (already defined in codex-types)

codex-exec (mind-tool-engine)
├── ToolCall struct — to build PreToolUse payloads
└── ToolResult struct — to build PostToolUse payloads

codex-drive (mind-model-router)
├── DriveLoop integration point — where to fire DriveEvent hooks
└── EventBus — to subscribe to events that should trigger hooks
```

### What hooks PROVIDES to other modules

```
→ codex-exec (mind-tool-engine)
  ├── HookDispatcher::fire_blocking(PreToolUse) — can block dangerous tool calls
  └── HookDispatcher::fire(PostToolUse) — can inject context or stop session

→ codex-coordination / cortex (mind-coordination)
  ├── HookDispatcher::fire(SessionStart/Stop) — lifecycle notifications
  └── HookDispatcher::fire_blocking(PreDelegation) — can block bad delegation

→ codex-drive (mind-model-router)
  └── HookDispatcher::fire(UserPromptSubmit) — prompt interception

→ ALL consumers
  └── HookHandler trait — implement to add custom hook behavior
```

### Dependency Graph (proposed)

```
codex-types ← aiciv-hooks ← codex-exec (pre/post tool use)
                           ← codex-drive (user prompt, drive events)
                           ← cortex (session start/stop, delegation)
                           ← codex-memory (memory write)
```

`aiciv-hooks` depends ONLY on `codex-types` (for shared event types) and standard crates (`tokio`, `serde`, `serde_json`, `tracing`, `anyhow`). It does NOT depend on `codex-exec`, `codex-drive`, etc. — the integration is inverted (consumers call hooks, hooks doesn't call consumers).

---

## 5. Recommended Build Order

### Step 1: Define hook types in `codex-types` (REQUEST to mind-coordination)

Add to `codex-types/src/lib.rs`:
- `HookEventType` enum (the 5 Codex types + our 4 extensions)
- `ToolCallInfo` struct (tool name, input JSON, output JSON)
- `HookPayload` struct (session_id, cwd, event, tool info)
- `HookResponse` enum variants

**Blocker**: Need mind-coordination to add these types.

### Step 2: Create `aiciv-hooks` crate (NEW — or extend codex-patcher)

Decision: **New crate `aiciv-hooks`** is cleaner than extending codex-patcher. Reasons:
- codex-patcher's purpose is build-time patching (unified diffs)
- hooks is a runtime dispatch system — fundamentally different concern
- Codex itself has hooks as a separate crate, confirming this boundary
- codex-patcher can KEEP its current patches + qwen interceptor
- aiciv-hooks starts clean with just the hook engine

Contents:
```
src/aiciv-hooks/
├── Cargo.toml
├── src/
│   ├── lib.rs           — module root, re-exports
│   ├── event.rs         — HookEvent enum + payload builders
│   ├── response.rs      — HookResponse enum + parsers
│   ├── config.rs        — HookConfig, loading from TOML/JSON
│   ├── dispatcher.rs    — HookDispatcher (the core engine)
│   ├── command_runner.rs — external command execution (spawn, pipe, timeout)
│   ├── handler.rs       — HookHandler trait + ExternalCommandHandler impl
│   └── filters.rs       — Tool-name filtering, event-type matching
```

### Step 3: Wire pre/post tool-use into codex-exec (COORDINATE with mind-tool-engine)

mind-tool-engine wraps `ToolExecutor::execute()` with:
```rust
// Before execution
let responses = dispatcher.fire_blocking(PreToolUse { ... }).await;
if responses.iter().any(|r| r.should_block()) {
    return Err(ToolBlocked { reason });
}

// Execute tool
let result = tool.execute(call).await;

// After execution
dispatcher.fire(PostToolUse { ... }).await;
```

### Step 4: Wire session lifecycle into cortex (COORDINATE with mind-coordination)

mind-coordination adds to `cortex` boot/shutdown:
```rust
// On boot
hooks.fire(SessionStart { session_id, config }).await;

// On shutdown
hooks.fire(SessionStop { session_id, reason }).await;
```

### Step 5: Wire delegation hooks (COORDINATE with mind-coordination)

In the spawn path (which currently lives in codex-patcher's agent_control_patch):
```rust
// Before delegation
let responses = hooks.fire_blocking(PreDelegation { target, task }).await;
// After delegation completes
hooks.fire(PostDelegation { target, result }).await;
```

### Step 6: External hook config loading

Load hooks from the mind's config file:
```toml
[[hooks]]
event = "pre_tool_use"
command = "/path/to/safety-checker"
tool_names = ["bash"]
timeout_ms = 5000
required = true

[[hooks]]
event = "post_tool_use"
command = "/path/to/memory-extractor"
timeout_ms = 10000
required = false
```

---

## 6. Relationship Between codex-patcher and aiciv-hooks

```
codex-patcher (EXISTING — keep as-is)
├── Build-time patch system for injecting Cortex into Codex upstream
├── 4 patches: agent-control, session-thinkloop, memory-dual-write, sandbox-bridge
├── QwenInterceptor (delegation tool, should probably move to codex-exec or codex-llm)
└── Purpose: STATIC injection at build time

aiciv-hooks (NEW — the real hooks engine)
├── Runtime event dispatch system
├── 9 hook event types (5 from Codex + 4 aiciv extensions)
├── External command hooks + in-process HookHandler trait
├── Pre-tool blocking, post-tool injection, session lifecycle
└── Purpose: DYNAMIC runtime interception

Both owned by mind-hooks.
```

The patcher's agent_control_patch already validates fractal spawns — this becomes the `PreDelegation` hook's default built-in handler. The session_patch's ThinkLoop becomes the standard `SessionStart` hook that loads memory context. The sandbox_bridge becomes a `PreToolUse` hook that enforces role-based sandboxing.

Over time, the static patches migrate INTO the hooks system as default handlers, and codex-patcher becomes thinner (or deprecated if we stop patching upstream Codex).

---

## 7. Open Questions for mind-lead

1. **New crate or extend?** Recommendation is new `aiciv-hooks` crate. Need mind-lead approval to add a workspace member.
2. **Type location**: Should `HookEvent`/`HookResponse` live in `codex-types` (shared) or in `aiciv-hooks` (self-contained)? Trade-off: shared types = other crates can reference without depending on the full hooks engine. Self-contained = simpler initially.
3. **QwenInterceptor placement**: Currently in codex-patcher but is really a tool/LLM concern. Move to `codex-llm` (mind-model-router) or `codex-exec` (mind-tool-engine)?
4. **Codex upstream availability**: The `codex-upstream/` directory is empty. Do we need to `git clone` the Codex repo for reference, or work from the deep-map analysis?
5. **Priority**: The MISSIONS.md lists hooks as Phase 2. Is now the right time, or should Phase 1 (verify all crates compile) complete first?

---

## 8. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Type definitions not ready in codex-types | Medium | Can define locally in aiciv-hooks first, migrate to codex-types later |
| codex-exec integration complexity | Medium | Start with a simple `Arc<HookDispatcher>` parameter to `ToolExecutor` |
| External command timeout handling | Low | Copy Codex's proven pattern (spawn, pipe, timeout, kill) |
| reqwest missing in Cargo.toml for qwen_interceptor | Low | Add `reqwest` or move interceptor to correct crate |
| Codex upstream hooks code not available locally | Low | Deep-map provides sufficient architectural detail to build from |

---

## 9. Summary

**Today**: codex-patcher is a build-time diff generator with no runtime hook dispatch.

**Target**: A clean `aiciv-hooks` crate that provides Codex-style lifecycle hooks (pre/post tool use, session start/stop, user prompt submit) extended with aiciv-mind-specific events (pre/post delegation, memory write, drive events).

**Build order**: Types → Crate → Tool-use wiring → Session wiring → Delegation wiring → Config loading.

**Key principle**: Hooks inverts dependencies. The hooks crate knows nothing about tool execution or session management. Consumers (codex-exec, cortex, codex-drive) call INTO hooks. This keeps the boundary clean — exactly as Codex achieved with their 5,553-line module.
