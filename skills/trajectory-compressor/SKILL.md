---
name: trajectory-compressor
description: Compress session ledgers for context management. LLM-summarizes middle turns, protects first+last. Reuses session-summarization skill pattern. Independent of GPU/Hub.
version: 1.0.0
author: Hengshi (from Hermes exploration)
license: MIT
metadata:
  hengshi:
    tags: [trajectory, compression, session, context-management, llm]
    related_skills: [session-summarization, tdd]
    source: Hermes Agent trajectory_compressor.py (hermes-exploration-memo.md iter 4)
---

# Trajectory Compressor — Session Ledger Compression

## What It Is

Ports Hermes Agent's `trajectory_compressor.py` pattern to compress our own session ledgers. Keeps context windows clean while preserving critical information: system setup, initial instructions, final decisions.

**Key insight:** Full session transcripts waste tokens on middle turns that don't matter for next-session context. But you can't just truncate — the middle often contains working state, partial attempts, debugging sequences that are informative.

**Compression strategy:**
1. Protect first N turns (system, human, first GPT, first tool)
2. Protect last N turns (final actions and conclusions)
3. LLM-summarize MIDDLE turns into one summary turn
4. Token budget enforced: default 15250 (Atropos SFT budget), configurable

## Architecture

```
Session JSONL (full trajectory)
    ↓
Turn parser → list of Turn(role, content, ...)
    ↓
Token budget check → if under budget, return as-is
    ↓
Protect first N + last N
    ↓
LLM-summarize middle turns (reuse session-summarization model)
    ↓
Replace middle with single [COMPRESSED N turns → 1 summary] turn
    ↓
Compressed JSONL + metadata
```

## Use Cases

1. **Context management** — Compress old sessions before injecting as memory context
2. **Training data prep** — Compress trajectories for Atropos SFT/DPO training
3. **Session archival** — Long sessions stored in compressed form with metadata
4. **Token budget enforcement** — Per-session token budgets (default 15250)

## API

```python
from trajectory_compressor import compress_trajectory, CompressedSession

# Compress a session
compressed: CompressedSession = compress_trajectory(turns)

# Stats
print(f"Compression ratio: {compressed.compression_ratio():.1%}")
print(f"Turns: {compressed.original_turns}→{compressed.compressed_turns}")
print(f"Tokens: {compressed.original_tokens}→{compressed.compressed_tokens}")

# Access compressed turns
for turn in compressed.turns:
    print(turn.role, turn.content[:100])
```

## CLI

```bash
# Compress single file
python3 trajectory_compressor.py compress input.jsonl output.jsonl

# Compress directory
python3 trajectory_compressor.py compress-dir sessions/ compressed/

# Stats without compressing
python3 trajectory_compressor.py stats input.jsonl
```

## Token Budget

| Budget | Use Case |
|--------|----------|
| 15250 | Atropos SFT default (matches Hermes Agent pipeline) |
| 5000 | Short context windows |
| 3000 | Aggressive compression for very long sessions |
| 750 | Single-summary mode (same as session-summarization) |

Set via `MAX_TRAJECTORY_TOKENS` env var.

## Middle-Turn Compression

The middle of a session often contains:
- Repeated debugging attempts
- Tool call sequences exploring the same space
- Clarifying questions that inform the final approach
- Working state that's already incorporated into the final answer

LLM summarization preserves the **signal** (decisions made, approaches tried, results achieved) without the **noise** (exact token sequences, intermediate states).

## Turn Types Protected

| Region | What's Protected | Why |
|--------|-----------------|-----|
| First N | system prompt, human instruction, first assistant, first tool | Critical setup + intent |
| Last N | final answer, conclusions, decisions made | The actual deliverable |
| Middle | everything else | Compressed via LLM |

## Env Vars

| Variable | Default | Purpose |
|----------|---------|---------|
| `MAX_TRAJECTORY_TOKENS` | `15250` | Token budget |
| `PROTECT_FIRST_N` | `4` | First turns to protect |
| `PROTECT_LAST_N` | `2` | Last turns to protect |
| `SUMMARY_MODEL` | `devstral-small-2:24b` | LLM for summarization |
| `OLLAMA_BASE_URL` | `https://api.ollama.com` | Summarization endpoint |

## Related Work

- Hermes Agent `trajectory_compressor.py` — source pattern
- `session-summarization` skill — shares summarization infrastructure
- `atropos-grpo` skill — Atropos SFT training that consumes compressed trajectories
- Hermes exploration memo: `../hermes-exploration-memo.md`
