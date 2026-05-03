---
name: compute-hibernation-tracker
description: Track compute sessions, measure active vs idle time, surface hibernation candidates per D4
version: 1.0.0
---

# Compute Hibernation Tracker — Firing Contract

## WHEN

```bash
python3 compute_hibernation_tracker.py session_start --civ hengshi
# After each tool call:
python3 compute_hibernation_tracker.py ping --active
# During idle/waiting:
python3 compute_hibernation_tracker.py ping
# At session end:
python3 compute_hibernation_tracker.py session_end
# On demand:
python3 compute_hibernation_tracker.py analyze --civ hengshi
python3 compute_hibernation_tracker.py hibernate_candidates --civ hengshi
```

Triggered by:
- Session start hook (mark compute session begins)
- Every tool call (ping --active marks active interval)
- Idle periods (ping without --active marks idle interval)
- Session end hook (mark compute session ends, log to JSONL)
- On demand for analysis

## WHAT

Tracks compute sessions: active vs idle time per session.
Logs to JSONL: one record per ended session.
Surfaces hibernation candidates based on idle/active ratios.

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| Civ name provided | Non-empty string |
| State file writable | `STATE_FILE.parent.exists()` or creatable |
| Log path writable | Parent dir exists or creatable |

## POST

| Condition | Output |
|-----------|--------|
| session_start | `"Session started: <sid> (<civ>)"` |
| ping | State updated (no output) |
| session_end | `"Session ended: <sid> — N tool_calls, Xs active, Ys idle"` |
| analyze | Usage table + hibernation candidates |
| hibernate_candidates | List of candidates with reasons |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| State file corrupt | `json.JSONDecodeError` | Reset state, start fresh |
| Session end without start | `current_session is None` | Print warning, no-op |
| Malformed JSONL line | `json.JSONDecodeError` | Skip line, continue |

## OBSERVABILITY

CLI output is self-documenting:
```
COMPUTE HIBERNATION TRACKER — D4 Analysis
  Total sessions:      12
  Hibernation cands:  3 (25%)
  Total active time:   14400s (4.0h)
  Total idle time:     28800s (8.0h)
  Total tool calls:    847

HIBERNATION CANDIDATES:
  hengshi-20260503-1430: 3600s, 2 calls, 120s active, 3480s idle
    → idle_gt_active_2x
    → long_session_low_utilization
```

## Evidence for Claims

D4: Pay only when active. Hibernation candidates = sessions where idle >> active.
The analysis output is the evidence artifact.
