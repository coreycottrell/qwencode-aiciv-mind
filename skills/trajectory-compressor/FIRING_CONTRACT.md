---
name: trajectory-compressor
description: Session ledger compression via LLM summarization. Protect first+last turns, compress middle. Token budget enforced.
version: 1.0.0
trigger: python3 trajectory_compressor.py compress|compress-dir|stats
---

# Trajectory Compressor Firing Contract

## WHEN

CLI or API call:
```bash
python3 trajectory_compressor.py compress input.jsonl output.jsonl
python3 trajectory_compressor.py compress-dir sessions/ compressed/
python3 trajectory_compressor.py stats input.jsonl

from trajectory_compressor import compress_trajectory, CompressedSession
compressed = compress_trajectory(turns)
```

## WHAT

**Compresses** conversation session JSONL by:
1. Protecting first N turns (system, human, first GPT, first tool) — `PROTECT_FIRST_N=4`
2. Protecting last N turns (final actions and conclusions) — `PROTECT_LAST_N=2`
3. LLM-summarizing MIDDLE turns into one `[COMPRESSED N turns → 1 summary]` turn
4. Enforcing `MAX_TRAJECTORY_TOKENS=15250` budget

**Returns** `CompressedSession` with:
- `turns`: compressed list of Turn objects
- `compression_ratio()`: fraction of turns removed
- `original_turns`, `compressed_turns`, `original_tokens`, `compressed_tokens`

## PRE

| Prerequisite | How Verified |
|--------------|--------------|
| Input JSONL valid | `json.loads()` each line without error |
| OLLAMA_BASE_URL set (optional) | For LLM summarization; falls back to simple concat |
| Input file readable | `open(path)` succeeds |
| Output dir writable | `Path(output_dir).mkdir(parents=True, exist_ok=True)` |

**If LLM unavailable:** Falls back to `_simple_concat_summary()` — no error thrown.

## POST

| State | Condition |
|-------|-----------|
| Under budget | Returns original turns unchanged, `summary_turns=0` |
| Over budget, middle exists | Middle compressed to 1 summary turn |
| Over budget, no middle | First+last only, `summary_turns=0` |
| LLM unavailable | Uses simple concatenation, logged warning |
| Stats mode | Prints: `{file}: {turns} turns, ~{tokens} tokens (status)` |

## FAILURE

| Failure Mode | Detection | Recovery |
|-------------|-----------|----------|
| JSON parse error | `json.JSONDecodeError` skipped, continues | Validate input before running |
| File not found | `FileNotFoundError` | Check path |
| Disk full | `IOError` on write | Free space |
| LLM timeout | `urllib.error.URLError` | Falls back to simple concat |

## OBSERVABILITY

Logger outputs:
- `"Compressed {path}: {orig}→{comp} turns ({ratio:.1%} reduction)"`
- `"Compressed {path}: {orig}→{comp} turns ({ratio:.1%} reduction)"` (batch)
- `"OLLAMA_BASE_URL not set — using simple concatenation"` (warning)
- `"LLM summarization failed (...) — using simple concatenation"` (warning)

**Claim provenance:** The ~64% turn-reduction figure is sourced from Hermes Agent's original `trajectory_compressor.py` implementation (Nous Research, MIT license). Not measured on local session data in this PoC. Real-world compression ratio depends on session length distribution and middle-turn noise level.

## ENV VARS

| Variable | Default | Purpose |
|----------|---------|---------|
| `MAX_TRAJECTORY_TOKENS` | `15250` | Token budget |
| `PROTECT_FIRST_N` | `4` | First turns to protect |
| `PROTECT_LAST_N` | `2` | Last turns to protect |
| `SUMMARY_MODEL` | `devstral-small-2:24b` | LLM model |
| `OLLAMA_BASE_URL` | `https://api.ollama.com` | Summarization endpoint |
| `OLLAMA_API_KEY` | `""` | API key (unused if unauthed) |
