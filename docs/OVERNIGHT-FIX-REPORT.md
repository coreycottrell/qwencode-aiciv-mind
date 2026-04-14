# Overnight Daemon Fix Report — 2026-04-05

**Daemon run**: 76 missions, 18 FAILED (all child agent delegations)
**Log**: `data/logs/daemon-20260404-202407.log` (3086 lines)
**Principle**: SYSTEM > SYMPTOM — fix the architecture, not the instance

---

## Diagnosis

### Failure Pattern 1: LLM HTTP 500 Errors (15 of 18 failures)

Every DelegateHandler failure was identical:
```
LLM error: API error (status 500): {"error":"Internal Server Error (ref: ...)"}
```

**Root cause**: `OllamaClient::chat()` made a single HTTP request with **zero retry logic**. One transient 500 from Ollama Cloud killed the entire ThinkLoop, which killed the task, which marked it `Failed` in TaskStore. No backoff, no retry, no recovery.

The error propagation chain:
1. `OllamaClient::chat()` → returns `Err(LlmError::Api{500})` on first try
2. `ThinkLoop::run_full()` → `?` operator immediately returns `ThinkError::Llm`
3. `ThinkDelegateHandler::process_task()` → `?` returns `Err(String)`
4. `McpMindServer::handle_delegate()` → logs "DelegateHandler error", returns `completed: false`
5. `ProcessBridge::delegate()` → records `TaskState::Failed` in SQLite

**System gap**: `ProcessBridge` had retry logic for *transport errors* (child crashes), but LLM 500s came back as successful MCP responses containing failure payloads — bypassing all retry paths.

### Failure Pattern 2: Hub UUID Validation (5 of 7 Hub errors)

The LLM passed group *names* ("civsubstrate", "federation", "aiciv") instead of UUIDs to Hub tools:
```
hub_list_rooms failed group_id="civsubstrate" error=Hub error: HTTP 422: uuid_parsing
```

**Root cause**: `HubInterceptor::handle()` only checked for empty strings. No UUID format validation. Whatever the LLM provided went straight into the URL path, causing 422s from the Hub server.

### Failure Pattern 3: Hub Auth Missing (2 of 7 Hub errors)

```
hub_reply failed error=Hub error: HTTP 401: Authorization header required
```

**Root cause**: `HubClient` reads `HUB_JWT_TOKEN` env var at construction time. If not set (or expired), write operations silently send requests without Authorization headers. No auto-auth, no token refresh.

---

## Fixes Applied

### Fix 1: LLM Retry with Exponential Backoff
**File**: `src/codex-llm/src/ollama.rs`

Added to `OllamaClient`:
- **Retry loop**: up to 3 retries with exponential backoff (1s, 2s, 4s)
- **Retryable errors**: HTTP 429/500/502/503/504, connection errors
- **Non-retryable errors**: HTTP 4xx (except 429), parse errors — fail immediately
- **Connect timeout**: 10 seconds (was: none — could hang forever)
- **Request timeout**: 5 minutes (was: none — could hang forever)
- **`LlmError::is_retryable()`**: structured method for callers to check error type
- **Logging**: warns on each retry attempt with backoff duration, logs success after retry

Impact: eliminates 15/18 overnight failures. Transient Ollama Cloud 500s will be retried and likely succeed on attempt 2 or 3.

### Fix 2: Hub UUID Validation
**File**: `src/codex-suite-client/src/hub_interceptor.rs`

Added `validate_uuid()` helper using the `uuid` crate (already a dependency). Applied to:
- `hub_list_rooms` — validates `group_id`
- `hub_list_threads` — validates `room_id`
- `hub_read_thread` — validates `thread_id`
- `hub_create_thread` — validates `room_id`
- `hub_reply` — validates `thread_id`

Error messages are instructive: `"Invalid group_id: 'civsubstrate' is not a valid UUID. Use the actual UUID..."` — guiding the LLM to correct its input.

Impact: eliminates 5/7 Hub errors. Invalid IDs are caught before the HTTP call, saving a round-trip and giving the LLM actionable feedback to self-correct.

### Fix 3: Hub Auth (noted, not fixed in this pass)

The 401 errors require daemon-level auth integration: authenticate with AgentAUTH at daemon boot, pass JWT to HubClient, and refresh on expiry. This is a larger change (touches `main.rs` daemon wiring) — documented for next session.

---

## Verification

```
$ cargo test
239 passed, 0 failed, 3 ignored

$ cargo build --release
Finished `release` profile [optimized]
```

New tests added:
- `hub_interceptor_rejects_non_uuid_ids` — verifies group names and mission-style IDs are rejected
- Updated `hub_interceptor_validates_required_args` — uses valid UUIDs where UUID validation precedes other checks

---

## Impact Assessment

| Category | Before | After |
|----------|--------|-------|
| LLM transient 500s | Task dies on first error | 3 retries with backoff (1s/2s/4s) |
| Hub UUID misuse | HTTP 422 from server | Client-side rejection with guidance |
| Hub auth missing | HTTP 401 at runtime | Documented for next fix |
| Request hangs | No timeout (infinite) | 10s connect, 5min request |
| Expected failure reduction | 18/76 = 24% | Estimated 2-3/76 = 3-4% |

---

## Files Changed

| File | Change |
|------|--------|
| `src/codex-llm/src/ollama.rs` | Retry loop, timeouts, `is_retryable()` |
| `src/codex-suite-client/src/hub_interceptor.rs` | UUID validation, new test |

---

## Remaining Work

1. **Hub JWT auth at daemon boot** — authenticate with AgentAUTH, propagate token to HubClient (eliminates 2 remaining Hub errors)
2. **ThinkLoop-level retry** — currently retry is in OllamaClient. Consider adding task-level retry in ProcessBridge for when the entire ThinkLoop fails (belt + suspenders)
3. **StallDetection calibration** — log shows false positive stall warnings; the detector doesn't track agent output correctly
