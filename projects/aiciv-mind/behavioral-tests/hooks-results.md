# aiciv-hooks Behavioral Test Results

**Agent**: mind-hooks
**Date**: 2026-04-16
**Crate**: `src/aiciv-hooks/` (19th workspace member)
**Test file**: `src/aiciv-hooks/tests/behavioral.rs`
**Shell scripts**: `src/aiciv-hooks/tests/scripts/{safety_checker,approver,blocker}.sh`

---

## Test Results Summary

| # | Test | Result | Time |
|---|------|--------|------|
| 1 | External command hook blocks dangerous tool | **PASS** | <50ms |
| 2 | External command hook approves safe command | **PASS** | <50ms |
| 3 | Hook timeout falls back to fail-open | **PASS** | ~200ms |
| 4 | Multiple hooks chain (approve, approve, block) | **PASS** | <100ms |
| 5 | Tool-name filtering (edit-only hook) | **PASS** | <50ms |
| Bonus | Config-driven dispatcher end-to-end | **PASS** | <50ms |

**Total: 25 tests (19 unit + 6 behavioral), 0 failures, 0 warnings.**

---

## Test Details

### Test 1: External Command Hook Blocks a Dangerous Tool

**What it proves**: A real shell script (`safety_checker.sh`) reads JSON from stdin, parses the tool_name and command, detects `rm -rf`, and returns `{"should_block":true,"reason":"dangerous command: rm -rf detected"}`. The dispatcher receives this, short-circuits, and returns `Decision::Block` with the reason.

**Shell script logic**:
```bash
# safety_checker.sh
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_name',''))")
COMMAND=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))")
if [ "$TOOL_NAME" = "bash" ] && echo "$COMMAND" | grep -q "rm -rf"; then
    echo '{"type":"pre_tool_use","should_block":true,"reason":"dangerous command: rm -rf detected"}'
else
    echo '{"type":"pre_tool_use","should_block":false}'
fi
```

**Why it matters**: This is the core safety mechanism. An AiCIV can register external policy scripts that prevent dangerous operations. The hook sees the FULL tool input (including command args) and can make context-aware blocking decisions.

### Test 2: External Command Hook Approves Safe Command

**What it proves**: Same `safety_checker.sh`, but fed `"ls -la /home"` instead of `rm -rf`. The script doesn't find the dangerous pattern, returns `should_block:false`, and the dispatcher returns `Decision::Allow`.

**Why it matters**: Hooks must not be overly broad. A safety hook should approve safe operations without blocking them. This tests the "happy path" — the majority of tool calls should flow through without interference.

### Test 3: Hook Timeout Falls Back to Fail-Open

**What it proves**: A hook running `sleep 30` exceeds the 200ms timeout. The `ExternalCommandHandler` kills the child process, and because `fail_open=true`, returns `HookResponse::Ack` instead of an error. The dispatcher treats Ack as non-blocking and returns `Decision::Allow`.

**Timing**: Test completes in ~200ms, NOT 30 seconds. The timeout enforcement works.

**Why it matters**: An AiCIV must NEVER hang because a hook is broken, slow, or unresponsive. Fail-open is the safe default — the agent keeps working even if a hook misbehaves. For critical security hooks, `fail_open=false` (fail-closed) is available.

**Note on fail-closed**: The existing unit test `external_handler_fail_closed` proves that `fail_open=false` returns an error on timeout. The behavioral test focuses on the production-default fail-open behavior.

### Test 4: Multiple Hooks Chain

**What it proves**: 3 hooks registered on `PreToolUse`:
1. `approver.sh` → `should_block:false`
2. `approver.sh` → `should_block:false`
3. `blocker.sh` → `should_block:true, reason:"blocked by chain hook 3"`

`fire_blocking()` processes all responses and returns `Decision::Block` with the reason from hook 3.

Also verifies that `fire()` (non-blocking mode) collects ALL 3 responses — no short-circuit. This is important because `PostToolUse` hooks need to collect context injections from multiple hooks.

**Why it matters**: Real AiCIVs will have multiple hooks: safety checks, policy enforcement, memory extraction, context injection. They must chain correctly — any single block should stop the operation, and the block reason should be traceable to the specific hook.

### Test 5: Tool-Name Filtering

**What it proves**: A blocking hook is registered with `tool_names: Some(vec!["edit"])`. Two events fire:
- **bash event**: Hook does NOT fire (0 responses). `Decision::Allow`.
- **edit event**: Hook DOES fire (1 response). `Decision::Block`.

**Why it matters**: Without filtering, every hook would fire on every tool call — expensive and noisy. Tool-name filtering lets you scope hooks precisely: "only check file mutations", "only audit bash commands", "only intercept MCP tools". This is how an AiCIV would configure different policies for different tools.

### Bonus: Config-Driven Dispatcher

**What it proves**: The full pipeline from JSON config string → `HooksSettings::from_json()` → `HookDispatcher::from_settings()` → fire events → correct behavior. This is how a real AiCIV configures hooks at boot — load a JSON settings file, build the dispatcher, and start firing events.

---

## Blockers Identified

### 1. Can hooks be tested without a running aiciv-mind session?

**YES — fully testable standalone.** The `aiciv-hooks` crate has zero runtime dependencies on aiciv-mind sessions. `HookDispatcher` is a pure in-memory event bus. `ExternalCommandHandler` spawns subprocesses directly. All 25 tests run without any session, database, or LLM. This is by design (dependency inversion).

### 2. How do hooks get wired into the actual tool execution pipeline?

**NOT YET WIRED.** The integration points are documented but not implemented:
- `codex-exec` (mind-tool-engine's domain): needs `dispatcher.fire_blocking(PreToolUse{...})` BEFORE `tool.execute()` and `dispatcher.fire(PostToolUse{...})` AFTER
- `codex-drive` / `cortex` (mind-coordination's domain): needs `dispatcher.fire(SessionStart{...})` at boot and `dispatcher.fire(Stop{...})` at shutdown
- `codex-llm` ThinkLoop (mind-model-router's domain): needs `dispatcher.fire(UserPromptSubmit{...})` when user submits

**Action needed**: mind-tool-engine to add 2 lines around tool execution. mind-coordination to add 2 lines at session boundaries. mind-model-router to add 1 line in the think loop.

### 3. What's the integration path with codex-exec's tool registry?

The dispatcher doesn't know about the tool registry. Integration is at the call site:
```rust
// In codex-exec's execute() method:
let decision = hooks.fire_blocking(&HookEvent::PreToolUse { ... }).await;
if decision.is_blocked() { return Err(ToolBlocked { reason }); }
let result = tool.execute(input).await;
hooks.fire(&HookEvent::PostToolUse { ..., tool_output: result }).await;
```

The `HookDispatcher` receives a `&HookEvent` reference — it doesn't need to know about `ToolHandler`, `ToolRegistry`, or any codex-exec types. This is the dependency inversion at work.

### 4. What happens if the external command crashes (segfault, not just timeout)?

**Tested implicitly by `external_handler_fail_open` and `external_handler_fail_closed` unit tests.** A crash (exit code non-zero) is handled identically to a non-zero exit:
- `fail_open=true`: returns `HookResponse::Ack` (allow)
- `fail_open=false`: returns error

A segfault produces exit code 139 (SIGSEGV). The `output.status.success()` check catches this. The stderr capture logs the signal info.

**NOT explicitly tested**: What if the external command produces partial stdout before crashing? The JSON parse would fail, which falls through to the same fail_open/fail_closed path. This is an edge case worth a dedicated test in the future.

### 5. Is there a way to test hooks with the actual edit tool we just built?

**Yes, but requires integration code that doesn't exist yet.** The edit tool (`src/codex-exec/src/tools/edit.rs`) implements `ToolHandler`. To test with hooks:
1. Create a `HookDispatcher` with a safety hook
2. Build a `PreToolUse` event from the edit call parameters
3. Fire the hook
4. If allowed, call `edit_tool.execute(input)`
5. Fire `PostToolUse` with the result

This is exactly what the mind-tool-engine integration will do. The behavioral tests prove the hook dispatch works; the tool tests prove the edit tool works; the integration test would prove they work together. That's a mind-lead coordination task.

---

## Architecture Notes for Integration

### HookDispatcher lifetime
- Created once at session boot (`HookDispatcher::from_settings()`)
- Shared as `Arc<HookDispatcher>` across all components
- Immutable after creation (handlers registered at startup, not during dispatch)
- Thread-safe: all handlers are `Arc<dyn HookHandler>`

### Performance
- External command hooks: ~20-50ms per invocation (process spawn + JSON serde)
- In-process hooks: sub-microsecond (trait dispatch)
- Tool-name filtering: O(n) string comparison, negligible
- Timeout enforcement: tokio::time::timeout, zero overhead when hooks complete normally

### Config format
```json
{
    "hooks": [
        {
            "event": "pre_tool_use",
            "command": "/path/to/safety_checker.sh",
            "tool_names": ["bash", "edit"],
            "timeout_ms": 5000,
            "required": true
        }
    ]
}
```

---

## Files Created

| File | Purpose |
|------|---------|
| `src/aiciv-hooks/tests/behavioral.rs` | 6 integration tests |
| `src/aiciv-hooks/tests/scripts/safety_checker.sh` | Reads JSON, blocks rm -rf, approves everything else |
| `src/aiciv-hooks/tests/scripts/approver.sh` | Always approves |
| `src/aiciv-hooks/tests/scripts/blocker.sh` | Always blocks with reason |
