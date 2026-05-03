---
name: session-summarization
description: LLM-summarize relevant past sessions before injecting into context — instead of dumping raw transcripts, summarize with a fast model to preserve token budget while retaining signal.
version: 1.0.0
author: Hengshi (adapted from Hermes Agent session_search_tool.py)
license: MIT
metadata:
  hengshi:
    tags: [memory, summarization, cross-session, context-management]
    related_skills: [tdd, systematic-debugging]
    source: Hermes Agent (Nous Research) — session_search_tool.py pattern
---

# Session Summarization Skill

## What It Is

Before injecting past session context into the current context window, **LLM-summarize the relevant sessions first**. Don't dump raw transcripts — summarize to a fixed token budget while retaining the signal.

This is the pattern from Hermes Agent's `session_search_tool.py`:
1. Search for relevant sessions (ripgrep/FTS across scratchpads and memories)
2. Group by session, take top N matches
3. Truncate each to a window centered on the match
4. **LLM-summarize** with a fast/cheap model
5. Inject summaries (not raw transcripts) into context

## Why It Matters

Raw transcript injection:
- Burns token budget fast
- Buries signal in noise
- Context window overflows on long sessions

Summarized injection:
- Fixed token budget (configurable, default ~2048 tokens per summary)
- Preserves signal, removes boilerplate
- Context window stays clean

## When It Fires

**Trigger**: When a mind needs to recall past sessions for current work.

**Preconditions**:
- `SCRATCHPAD_DIR` env var or path points to scratchpad directory
- `MEMORY_DIR` env var or path points to mind memory directory
- `OLLAMA_API_KEY` or `OPENAI_API_KEY` env var set for LLM calls
- `SUMMARIZATION_MODEL` env var (default: `devstral-small-2:24b` or `gpt-4.1-nano`)

**Postconditions**:
- Returns a list of session summaries, each with: session_id, timestamp, summary text, relevance score
- Each summary capped at `MAX_SUMMARY_TOKENS` (default: 2048)
- Original transcripts never modified

## Firing Contract

| Field | Value |
|-------|-------|
| **WHEN** | On `summarize_sessions(query, limit=3)` call — not automatic, called by mind before high-stakes decisions |
| **WHAT** | `summarize_sessions(query, limit)` → returns `List[SessionSummary]` |
| **PRECONDITIONS** | Scratchpad dir exists, LLM API key set, summarization model available |
| **POSTCONDITIONS** | Returns summaries, originals untouched, summaries may be cached |
| **FAILURE MODES** | API key missing → raises `SessionSummarizationError`. LLM timeout → returns empty list with warning logged. No matches → returns empty list |
| **OBSERVABILITY** | Summary cache file at `{SCRATCHPAD_DIR}/_summary_cache.jsonl`. Log entry on each call |

## How to Use

```python
from skills.session_summarization import summarize_sessions

# Before making a high-stakes decision or starting a complex task
summaries = summarize_sessions(
    query="how did we handle the rate limiting problem last time?",
    limit=3,
    mind_id="hengshi"
)

for s in summaries:
    print(f"[{s.session_id}] {s.summary}")
```

## Return Type

```python
@dataclass
class SessionSummary:
    session_id: str          # e.g., "2026-05-03_hermes-exploration"
    timestamp: str           # ISO timestamp of session
    summary: str            # LLM-generated summary, max 2048 tokens
    relevance_score: float  # 0.0-1.0, based on query match
    source_file: str        # Which file was summarized
```

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `SUMMARIZATION_MODEL` | `devstral-small-2:24b` | Model for summarization (fast/cheap) |
| `MAX_SUMMARY_TOKENS` | `2048` | Max tokens per summary |
| `MAX_SESSION_CHARS` | `100000` | Chars loaded per session before truncation |
| `SUMMARY_CACHE_ENABLED` | `true` | Cache summaries to avoid re-summarizing |

## Implementation Notes

- Uses ripgrep for fast session search across scratchpads
- Groups matches by parent session file
- Truncates to window around match (centered, ±5KB)
- LLM call is synchronous (await if async context)
- Summary cache is append-only JSONL, never overwrites

## Relationship to Memory System

This skill **complements** `MindMemory`, it does not replace it:
- `MindMemory.search()` → finds relevant memories via ripgrep
- `summarize_sessions()` → LLM-summarizes the *context* those memories came from

Think: Memory search tells you **what** happened. Session summarization tells you **the full story** of a past session.

## Co-use

This skill pairs with:
- **`skill-evolution-tracker`**: Run after `session-summarization` to track which past sessions were recalled and how useful the summaries were
- **`tdd`**: Summarize past TDD sessions before starting a new test cycle to avoid repeating patterns
- **`skill-curator`**: Grade summarized sessions as evidence of skill usage for the Curator's improvement signals

**Pre-condition**: Run `skill-evolution-tracker log session-summarization --outcome pass` after successful summarization
**Post-condition**: Consider logging the summarization result with `skill-evolution-tracker log session-summarization`
