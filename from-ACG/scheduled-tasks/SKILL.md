---
name: scheduled-tasks
description: Opportunistic task scheduling that survives reboots. Check-and-run pattern for daily/weekly tasks during BOOPs. Use when setting up recurring automated tasks.
---

# Scheduled Tasks Protocol

**Purpose**: Reliable daily/weekly task execution that survives computer restarts
**Created**: 2026-01-04
**Problem Solved**: Crons restart when computer restarts, tasks get missed

---

## The Insight

**Don't rely on clock-based scheduling. Use opportunistic scheduling.**

Instead of "run at 9am" (which fails if computer is off at 9am), use:
- "Check if done today, if not, do it now"
- "Check if done this week, if not, do it now"

This runs during BOOPs, which happen whenever ${HUMAN_NAME} is active.

---

## Architecture

### State File
**Location**: `.claude/scheduled-tasks-state.json`

```json
{
  "last_updated": "2026-01-04T16:00:00Z",
  "tasks": {
    "paper-scan": {
      "frequency": "daily",
      "last_run": "2026-01-04",
      "status": "completed"
    },
    "paper-digest-full": {
      "frequency": "weekly",
      "last_run": "2026-01-01",
      "last_run_week": 1,
      "status": "completed"
    },
    "comind-follows": {
      "frequency": "daily",
      "last_run": "2026-01-04",
      "count_today": 3,
      "status": "completed"
    }
  }
}
```

### BOOP Integration

Every BOOP cycle includes:
```
1. Read scheduled-tasks-state.json
2. For each task:
   - Daily: Is last_run != today? → Run it
   - Weekly: Is last_run_week != this_week? → Run it
3. Update state after running
4. Report what was triggered
```

---

## Registered Tasks

### Daily Tasks

| Task ID | What It Does | Max Time |
|---------|--------------|----------|
| `paper-scan` | Scan csai-bot for relevant papers, flag for digest | 5 min |
| `comind-follows` | Follow 2-3 from comind list (if not at daily limit) | 5 min |
| `daily-review` | Check priority accounts (Void, Archivist) | 5 min |
| `notifications` | Check & respond to Bluesky notifications | 10 min |

### Weekly Tasks

| Task ID | What It Does | Preferred Day | Max Time |
|---------|--------------|---------------|----------|
| `paper-digest-full` | Full paper analysis + blog + hub post | Sunday | 60 min |
| `memory-consolidation` | Review and consolidate week's learnings | Saturday | 30 min |
| `comind-progress` | Review follow plan progress, update | Friday | 10 min |

---

## How Tasks Register

Add to the task registry in state file:

```python
def register_task(task_id: str, frequency: str, description: str):
    """Register a new scheduled task."""
    state = load_state()
    state["tasks"][task_id] = {
        "frequency": frequency,  # "daily" or "weekly"
        "last_run": None,
        "status": "pending",
        "description": description
    }
    save_state(state)
```

---

## BOOP Check Logic

```python
from datetime import datetime, date

def check_scheduled_tasks():
    """Check and run any overdue scheduled tasks."""
    state = load_state()
    today = date.today().isoformat()
    this_week = date.today().isocalendar()[1]

    triggered = []

    for task_id, task in state["tasks"].items():
        should_run = False

        if task["frequency"] == "daily":
            should_run = task.get("last_run") != today

        elif task["frequency"] == "weekly":
            should_run = task.get("last_run_week") != this_week

        if should_run:
            triggered.append(task_id)

    return triggered
```

---

## Comparison: Cron vs Opportunistic

| Aspect | Cron | Opportunistic |
|--------|------|---------------|
| Computer off at scheduled time | Missed | ✅ Runs at next BOOP |
| Restart during task | Interrupted | ✅ State preserved, resumes |
| No BOOP for 2 days | N/A | Runs at next BOOP (catches up) |
| Predictable timing | ✅ Exact | Approximate (within BOOP cycle) |
| Requires daemon | ✅ cron/systemd | ❌ Just BOOP logic |

**Verdict**: For our use case (human present = BOOPs happening), opportunistic is more reliable.

---

## Hybrid Option (Advanced)

For tasks that MUST run at specific times (e.g., posting at optimal engagement hours):

1. **Systemd timer with Persistent=true** - Catches up on missed runs
2. **BOOP backup check** - If systemd missed, BOOP catches it

```ini
# /etc/systemd/system/weaver-daily.timer
[Timer]
OnCalendar=*-*-* 09:00:00
Persistent=true  # Run at boot if missed

[Install]
WantedBy=timers.target
```

But for most tasks, pure BOOP-based is simpler and sufficient.

---

## Quick Start

### For Daily Tasks
```
1. Add task to state file (or use register_task)
2. BOOPs automatically check and trigger
3. Task updates state when complete
```

### For New Weekly Task
```
1. Register: frequency="weekly"
2. First BOOP of week triggers it
3. Won't run again until next week
```

---

## State File Location

```
${CIV_ROOT}/.claude/scheduled-tasks-state.json
```

Checked every BOOP via delegation-spine or boop-manager.

---

## Integration Points

- **delegation-spine**: Add scheduled task check to BOOP pattern
- **bsky-boop-manager**: Check daily tasks during notification sweep
- **scratch-pad**: Note when scheduled tasks were triggered
- **memory**: Log task completions

---

## Why Not Just Cron?

${HUMAN_NAME} asked the right question. Crons assume:
- Computer always on at scheduled time
- Tasks are stateless (run regardless of last run)
- Human isn't involved in triggering

For AI collective work:
- Computer restarts happen
- Tasks should be idempotent and state-aware
- BOOPs are natural trigger points when human is present

**Opportunistic scheduling fits our reality better.**

---

*Built for reliability, not just scheduling.*
