---
name: session-summarizer
description: Session Summarizer — Hermes pattern for session continuity
version: 1.0.0
---

# Session Summarizer — Firing Contract

## WHEN

```bash
python3 session_summarizer.py snapshot --session-id hengshi-20260503 --tasks "Curator v0.2" --decisions "Walker unified" --next-steps "Await validator" --tools skill-curator compute-hibernation-tracker
python3 session_summarizer.py analyze
```

Triggered by:
- End of work block (capture continuity for next session)
- On-demand for session analysis

## WHAT

Captures session state: active files, git status, open tasks, decisions, next steps.
Logs to JSONL: one record per session.

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| Civ name provided | Non-empty string |
| Output dir writable | `Path(output).parent.exists()` or creatable |

## POST

| Condition | Output |
|-----------|--------|
| snapshot | "Session snapshot saved: {session_id}" + JSONL append |
| analyze | Summary table of recent snapshots |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Git unavailable | `subprocess.run` returns non-zero | Status shows "unavailable" |
| Log write fails | `IOError` | Print error, exit 1 |

## OBSERVABILITY

```
SESSION SUMMARIZER — Recent Sessions
  hengshi-20260503 [hengshi] 2026-05-03 — 12 files, uncommitted=✓
  hengshi-20260502 [hengshi] 2026-05-02 — 8 files, uncommitted=✗

Total snapshots: 2
```

## Evidence for Claims

Hermes: Sessions are ephemeral but continuity is essential.
Snapshot output is the evidence artifact — next session reads it to orient quickly.
