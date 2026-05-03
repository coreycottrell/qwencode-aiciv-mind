---
name: compute-hibernation-tracker
description: Track civ compute sessions — active vs idle time, tool call density, hibernation candidates per D4 (pay only when active)
version: 1.0.0
applicable_civs: [hengshi, proof, works, acg]
---

# Compute Hibernation Tracker — Hermes D4

D4: Compute uses serverless hibernation patterns. Pay only when active.

Tracks when a civ session is active (tool calls happening) vs idle (waiting),
logs compute sessions, and surfaces hibernation candidates.

## Problem It Solves

Civs run continuously but don't always need active compute. Without tracking
active/idle time, there's no data to know when to hibernate. With tracking,
we can identify: sessions that are mostly idle, long sessions with low
tool call density, and candidates for serverless hibernation.

## Usage

```bash
# Start a compute session (called at session start)
python3 compute_hibernation_tracker.py session_start --civ hengshi

# Ping — mark time as active (after a tool call)
python3 compute_hibernation_tracker.py ping --active --civ hengshi

# Ping — mark time as idle (during waiting)
python3 compute_hibernation_tracker.py ping --civ hengshi

# End session (called at session end)
python3 compute_hibernation_tracker.py session_end --civ hengshi

# Analyze past sessions
python3 compute_hibernation_tracker.py analyze --civ hengshi

# Show hibernation candidates only
python3 compute_hibernation_tracker.py hibernate_candidates --civ hengshi
```

## Signals

| Signal | When |
|--------|------|
| `pure_idle_session` | 0 tool calls, 0 active seconds |
| `idle_gt_active_2x` | idle time > 2× active time |
| `long_session_low_utilization` | >1hr, <5 tool calls |
| `active_ratio_lt_10%` | active time < 10% of total |

## Design Notes

- JSONL log: one record per ended session
- State file: tracks current session between start/end
- Tool call density proxy: tool_calls / duration_minutes
- No external dependencies beyond stdlib
- Can run on any civ's session lifecycle hooks
