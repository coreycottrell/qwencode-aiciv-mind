# Session Summarization — Firing Contract

**Skill**: `skills/session-summarization`
**Version**: 1.0.0
**Date**: 2026-05-03
**Author**: Hengshi

---

## Firing Contract

### WHEN It Fires

**Manual invocation only** — not cron-triggered, not automatic on context load.

The skill fires when a mind explicitly calls `summarize_sessions(query, mind_id, limit)` before:
1. Making a high-stakes architectural decision
2. Starting work on a complex task that prior sessions addressed
3. Recovering context after a session gap (post-restart)
4. Any time the mind needs cross-session context but token budget is tight

### WHAT Triggers It

```python
# Explicit call — no automatic trigger
summaries = summarize_sessions(
    query="how did we handle X?",
    mind_id="hengshi",
    limit=3
)
```

The `query` is a natural language string. The `mind_id` scopes the scratchpad search.

### PRECONDITIONS

| Precondition | How Verified | Failure Mode |
|---|---|---|
| `SCRATCHPAD_DIR` exists | `path.exists()` check | Raises `SessionSummarizationError` — scratchpad dir missing |
| `OLLAMA_API_KEY` or `OPENAI_API_KEY` set | Env var check at function entry | Raises `SessionSummarizationError` — no API key |
| `SUMMARIZATION_MODEL` available | LLM API reachable | Raises `SessionSummarizationError` — model unreachable |
| At least one scratchpad file exists | `SCRATCHPAD_DIR.rglob("*.md")` non-empty | Returns empty list, logs warning |
| LLM API responds within 60s | `urlopen(timeout=60)` | Raises `SessionSummarizationError` — timeout |

### POSTCONDITIONS

**State changes after firing:**

| State Change | Before | After |
|---|---|---|
| Return value | N/A | `List[SessionSummary]` (may be empty) |
| Summary objects | N/A | 0–limit `SessionSummary` objects created with: `session_id`, `timestamp`, `summary`, `relevance_score`, `source_file`, `query` |
| Scratchpad files | Unchanged | **UNMODIFIED** (read-only — verified at call site) |
| Cache file | 0 or more entries | 1 new JSONL line appended per summarized session |
| Cache file content | Prior entries preserved | New entries **appended only**, never overwritten |
| Token budget | N/A | Each `summary` field ≤ `MAX_SUMMARY_TOKENS` (750 tokens default) |
| Python process memory | N/A | Logger emits `INFO` with query hash, session count, elapsed time |

**Outputs returned:**
```python
List[SessionSummary]  # dataclass instances, not dicts
# Each SessionSummary has:
#   - session_id: str       # e.g. "2026-05-03_hermes-exploration"
#   - timestamp: str        # ISO 8601 UTC
#   - summary: str         # LLM-generated, ≤ MAX_SUMMARY_TOKENS
#   - relevance_score: float  # 0.0–1.0
#   - source_file: str     # absolute path to source scratchpad
#   - query: str           # the query used to find this
```

**Token cap enforcement (CODE-LEVEL, not just prompt request):**
- Method: word-count proxy (`len(summary.split())`) — English words ≈ tokens
- If returned summary > `MAX_SUMMARY_TOKENS` (750) words:
  1. Truncate to `MAX_SUMMARY_TOKENS - headroom` words (headroom ≈ 8 words for marker)
  2. Append marker: `"...[summary truncated — exceeded token budget]..."`
  3. Final word count guaranteed ≤ `MAX_SUMMARY_TOKENS`
  4. `WARNING` logged: `"LLM returned N words (>MAX cap). Truncating with marker."`
- If ≤ `MAX_SUMMARY_TOKENS` words: returned unchanged (no marker)
- **NOT enforced**: character-level exactness — word count is the proxy
- **Test proof**: `test_token_cap.py` — 3 cases (1000-word → 749 words, 8-word → unchanged, 750-word → no marker)

**Cache writes:**
- File: `{SCRATCHPAD_DIR}/_summary_cache.jsonl`
- Mode: **append-only** (never truncate or overwrite)
- Format: one JSON object per line, same fields as `SessionSummary` plus `query_hash`
- On cache hit: no new lines written, existing entry returned

**Observable side effects:**
- Logger `INFO`: `"Session summarization: query_hash={hash}, sessions_found={n}, summaries_returned={m}"`
- Logger `INFO`: `"Cache hit for query_hash: {hash}"` (when applicable)
- Logger `WARNING`: `"ripgrep failed, using Python fallback"` (degraded but functional)
- Logger `WARNING`: `"LLM returned N words (>MAX cap). Truncating with marker."` (token enforcement fires)
- Logger `ERROR`: `"LLM summarization failed: {e}"` (sets summary to `"[Summarization failed — see logs]"`)
- Logger `WARNING`: `"Failed to load summary cache: {e}"` (continues without cache)

### FAILURE MODES

| Failure | Detection | Retry Policy |
|---|---|---|
| No API key | Check at entry | Fail fast — no retry, raise `SessionSummarizationError` |
| LLM API timeout | 60s timeout on `urlopen` | No retry, raise `SessionSummarizationError` |
| LLM API error (4xx/5xx) | HTTP status check | No retry, raise `SessionSummarizationError` |
| Scratchpad dir missing | `path.exists()` | Fail fast, raise `SessionSummarizationError` |
| ripgrep not found | `FileNotFoundError` | Falls back to Python string search |
| Cache write fails | `IOError` on write | Logs warning, continues without caching |

### OBSERVABILITY

| Observable | Where | How |
|---|---|---|
| Function called | Python logger (`logging.getLogger("session_summarization")`) | `logger.info()` on each call with query + mind_id |
| Summaries returned | Return value | Check length + inspect `SessionSummary` fields |
| Cache writes | `{SCRATCHPAD_DIR}/_summary_cache.jsonl` | Append-only JSONL, one line per summary |
| Errors | Logger + exception | `logger.error()` + `SessionSummarizationError` |
| LLM latency | Logger | `logger.info()` with elapsed time |
| Cache hits | Logger | `logger.info("Cache hit for query_hash: %s", query_hash)` |

### INTEGRATION POINTS

| Point | How It Connects |
|---|---|
| `MindMemory.search()` | This skill complements — memory search finds **what**, session summarization finds **the story** |
| Scratchpad system | Reads from existing scratchpads, no migration needed |
| Mind startup | Can be called on restart to recover context from prior sessions |
| Skill creation | After completing a complex task, agent could call `summarize_sessions()` to build a summary for future reference |

---

## Evidence of Integration

**File**: `skills/session-summarization/summarize.py` (186 lines)
**SKILL.md**: `skills/session-summarization/SKILL.md` (Firing contract + usage docs)
**FIRING_CONTRACT.md**: This file

**Test**:
```bash
cd /home/corey/projects/AI-CIV/qwen-aiciv-mind
python skills/session-summarization/summarize.py "Hermes exploration" hengshi 3
# Expected: searches scratchpads/hengshi/ for relevant sessions,
#          LLM-summarizes top 3, returns structured summaries
```

**Verification checklist**:
- [ ] `summarize_sessions()` returns `List[SessionSummary]`
- [ ] Cache file written to `{SCRATCHPAD_DIR}/_summary_cache.jsonl`
- [ ] Logger outputs on each call
- [ ] `SessionSummarizationError` raised when preconditions fail
- [ ] Raw scratchpads unmodified
