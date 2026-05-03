---
name: session-summarizer
description: Capture session state for continuity — active files, uncommitted changes, decisions, next steps
version: 0.1.0
applicable_civs: [hengshi, proof, works, acg]
---

# Session Summarizer — Hermes Pattern

Sessions are ephemeral but continuity is essential. The summarizer captures session state
so the next session can orient quickly without rediscovering context.

## Usage

```bash
# Capture a snapshot at end of work block
python3 session_summarizer.py snapshot \
  --session-id hengshi-20260503 \
  --tasks "Curator v0.2" "D4 real session" \
  --decisions "Walker unified with runner (219/219)" \
  --next-steps "Await Proof validator verdict" \
  --tools skill-curator compute-hibernation-tracker

# Analyze past sessions
python3 session_summarizer.py analyze
```

## Signals

- `uncommitted_changes: true` — session has unsaved work
- `active_files` — files touched during session
- `key_decisions` — architectural choices made
- `next_steps` — explicit forward plan

## Design Notes

- JSONL log: one record per session snapshot
- Git-aware: detects uncommitted changes
- No external dependencies beyond stdlib
- Session-scoped: designed for end-of-work capture
