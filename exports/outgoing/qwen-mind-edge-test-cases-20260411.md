# qwen-mind Edge Case Test Plan

**Date**: 2026-04-11
**Purpose**: Draft test cases for Phase 1b planning — what breaks, how it should respond
**Status**: CASE LIST — not code yet

---

## LLM / Ollama Layer

### LLM-01: Malformed JSON Response
**Scenario**: Ollama returns a response that doesn't parse as valid JSON (truncated, corrupt, etc.)
**Expected**: Clean error (`LlmError::ApiError` or similar), no panic. Retry exhausted path. Task persists to scratchpad as "deferred — LLM response corrupt."
**Current gap**: `OllamaResponse` deserialization failure is caught in the retry loop, but what if the outer response body itself is not JSON at all (e.g., HTML error page from a proxy)?

### LLM-02: Empty Response Content
**Scenario**: Ollama returns `{"message": {"content": ""}}` — valid structure, zero content.
**Expected**: Treated as a result (not an error). Fitness score reflects low specificity. Memory written with empty content (edge case — should we skip writing empty memories?).
**Current gap**: Empty content gets written to memory. Should filter or flag.

### LLM-03: API Key Rejected (401/403)
**Scenario**: Ollama Cloud returns 401 (expired key) or 403 (quota exhausted).
**Expected**: Immediate error (no retries on auth failures). Task persisted to scratchpad with "LLM auth failed — check OLLAMA_API_KEY." Mind enters fallback mode: can still search memory, read scratchpad, persist tasks for later.
**Current gap**: 4xx errors are not retried (correct), but the mind has no "fallback mode" concept yet.

### LLM-04: Response Too Long (exceeds context window)
**Scenario**: LLM generates 100K+ tokens that exceed downstream processing capacity.
**Expected**: Truncate at `num_predict` limit (already set to 4096). If somehow exceeded, cap content before writing to memory.
**Current gap**: No content length cap on `persist_result`.

### LLM-05: Rate Limit (429)
**Scenario**: Ollama returns 429 Too Many Requests.
**Expected**: Exponential backoff with longer delay. If rate limit persists after max retries, enter fallback mode.
**Current gap**: 429 is a client error — currently treated like other 4xx (no retry). Should be retried with backoff like 5xx.

---

## Memory / SQLite Layer

### MEM-01: Database File Locked
**Scenario**: Another process holds a write lock on the SQLite DB (unlikely with single-process, but possible if parent mind queries while child writes).
**Expected**: SQLite busy timeout kicks in (default 5s). If timeout exceeded, error propagated as `MemoryError::Db`. Task deferred.
**Current gap**: No explicit busy_timeout configured in `MemoryStore::new()`.

### MEM-02: Corrupt Database
**Scenario**: DB file is corrupt (disk failure, partial write from crash).
**Expected**: `MemoryStore::new()` fails with clear error. Mind starts with empty memory but does not crash. Scratchpad and fitness continue.
**Current gap**: `MemoryStore::new()` will return `Db` error — mind constructor propagates it (mind fails to initialize). Should recover with empty store + warning.

### MEM-03: Disk Full
**Scenario**: No space left on device when writing memory or scratchpad.
**Expected**: Clean error. Fitness records "write_failed" flag. Mind continues with in-memory operations.
**Current gap**: `std::fs::write` panics on IO error in scratchpad (`.ok()` swallows but does not signal). Fitness `record()` opens file with `.unwrap()` — panics.

### MEM-04: FTS5 Index Out of Sync
**Scenario**: Trigger fails, FTS5 index diverges from main table.
**Expected**: Search returns partial results. Mind logs warning. Self-repair: rebuild FTS index on detection.
**Current gap**: No detection or repair. FTS5 triggers are defined in SQL — should be reliable, but if they break, search degrades silently.

### MEM-05: Concurrent Writes from Same Mind
**Scenario**: Mind's think loop called from two places simultaneously (parent delegates two tasks at once).
**Expected**: SQLite handles concurrent writes via WAL mode or serialized transactions. No data corruption. Results may interleave but each write is atomic.
**Current gap**: cortex-memory uses `SqlitePool` with max_connections(5) — concurrent access is expected. But qwen-mind's Mind struct holds a `MindMemory` which holds a `MemoryStore` which holds a `SqlitePool` — shared reference needs `Arc` or `&self` pattern. Current `Mind::think()` takes `&self` — concurrent calls would share the pool. This should work, but needs testing.

---

## Identity / Manifest Layer

### ID-01: Expired Role Keypair (Phase 1b)
**Scenario**: Ed25519 keypair revoked or rotated by ACG/Hub admin.
**Expected**: AgentAuth challenge fails with 401. Mind logs "identity revoked" and continues in degraded mode (local-only, no Hub). Scratchpad entry: "Hub auth failed — check keypair status."
**Current gap**: SuiteClient not wired yet (Phase 1b).

### ID-02: Manifest File Corrupt
**Scenario**: `manifests/qwen-lead.json` is corrupt JSON.
**Expected**: Mind creates new manifest from defaults. Logs warning. Session count resets (loss of growth history, but mind continues).
**Current gap**: `Manifest::load()` returns error — mind constructor fails.

### ID-03: Manifest Missing Parent Reference
**Scenario**: `parent_mind` field references a mind that no longer exists.
**Expected**: No impact on mind operation. Parent reference is informational. Delegation rules still work based on role, not parent identity.
**Current gap**: Not validated anywhere — which is correct, it's just metadata.

---

## Delegation Layer

### DEL-01: Delegate to Non-Existent Child
**Scenario**: Parent mind tries to delegate to a child subprocess that crashed or was never spawned.
**Expected**: IPC timeout (configurable, default 60s). Error propagated to parent. Parent marks task as "failed — child unreachable."
**Current gap**: Phase 1a IPC is file-based — no timeout concept yet. Phase 1b ZeroMQ will have built-in timeouts.

### DEL-02: Child Returns Malformed Result
**Scenario**: Child mind's result file contains invalid JSON (partial write, corrupt data).
**Expected**: Parent parses error, returns `TaskResult { success: false, content: "Child returned malformed result" }`. Child marked as unhealthy.
**Current gap**: IPC parsing in `run_ipc_loop` expects valid JSON — `serde_json::from_str` returns `Err` which is not currently handled.

### DEL-03: Delegation Rule Bypass Attempt
**Scenario**: Agent mind somehow constructed to call `think()` with a delegation task (trying to delegate to another mind).
**Expected**: `DelegationRules::can_delegate_to()` returns `Err(DelegationError)`. The delegation function short-circuits before any IPC or tool execution.
**Current**: Already implemented and tested (8 tests in `delegation.rs`). This is the strongest part of the codebase.

---

## Subprocess / IPC Layer

### IPC-01: Parent Dies While Child Running
**Scenario**: Parent mind process crashes or is killed while child is mid-execution.
**Expected**: Child continues executing (independent process). When complete, writes result file. On next parent check, result is found and consumed. No orphan processes.
**Current gap**: File-based IPC has no cleanup — result file persists until parent reads it. Acceptable for Phase 1a. Phase 1b ZeroMQ handles this with connection state.

### IPC-02: Task File Written Partially
**Scenario**: Parent crashes mid-write to task file. Child reads incomplete task text.
**Expected**: Child executes with whatever text it received. Result may be partial. Task ID or checksum in task file header would allow detection of partial writes (Phase 1b improvement).
**Current gap**: No checksum or length header on task files.

### IPC-03: Zombie Accumulation
**Scenario**: Multiple task/result files accumulate in the IPC directory (parent not cleaning up).
**Expected**: Stale files (>1 hour old) cleaned up periodically. Disk usage monitored.
**Current gap**: Task file is deleted after read (correct). Result file is not cleaned — should be deleted after parent reads it.

---

## Fitness Layer

### FIT-01: Fitness File Grows Unbounded
**Scenario**: Mind runs thousands of tasks, fitness JSONL grows to GBs.
**Expected**: Periodic rotation: old entries archived, summary statistics preserved (average, trend). Current file trimmed to last N entries.
**Current gap**: No rotation. Fitness file grows indefinitely.

### FIT-02: Fitness File Corrupt Mid-Write
**Scenario**: Crash during fitness write leaves partial JSON line at end of file.
**Expected**: History reader skips corrupt lines (already does — `filter_map` with `serde_json::from_str`). Average computed from valid entries only.
**Current**: Already handled by the `filter_map` pattern in `history()`.

### FIT-03: Fitness Score Always Zero
**Scenario**: Mind consistently fails tasks (fitness always 0.0).
**Expected**: After N consecutive failures (configurable, default 5), mind writes to scratchpad: "Persistent failure detected — consider spawning a fresh-context mind or requesting help."
**Current gap**: No failure detection or escalation.

---

## Priority for Phase 1b Implementation

**Must-have** (block Phase 1b sign-off):
- LLM-01: Malformed JSON handling
- LLM-05: Rate limit (429) retry
- MEM-03: Disk full — fitness unwrap panic fix
- MEM-05: Concurrent access test
- DEL-02: Malformed child result handling
- FIT-02: Already handled ✅

**Should-have** (Phase 1b polish):
- LLM-02: Empty response filtering
- MEM-01: SQLite busy timeout
- MEM-02: Corrupt DB recovery
- IPC-03: Result file cleanup
- FIT-03: Persistent failure detection

**Defer to Phase 2** (nice-to-have):
- MEM-04: FTS5 self-repair
- ID-01: Expired keypair (Phase 1b suite integration)
- ID-02: Corrupt manifest recovery
- FIT-01: Fitness rotation
- IPC-02: Task file checksum

---

*27 edge cases identified. 6 must-have, 5 should-have, 5 defer.*
*This list helps scope Phase 1b testing and prevents "it works until it doesn't" surprises.*
