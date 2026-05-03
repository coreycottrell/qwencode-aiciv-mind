---
name: skill-evolution-tracker
description: Track skill invocations and surface improvement signals per Hermes D1
version: 1.0.0
---

# Skill Evolution Tracker — Firing Contract

## WHEN

```bash
# After running a skill, log the invocation:
python3 skill_evolution_tracker.py log <skill-name> [--context <text>] [--outcome pass|fail]

# At end of session or on demand:
python3 skill_evolution_tracker.py analyze [--log memories/skills-usage-log.jsonl]
python3 skill_evolution_tracker.py signals [--log memories/skills-usage-log.jsonl]
```

## WHAT

1. **Log invocations**: skill name, timestamp, context, outcome (pass/fail), civ
2. **Analyze**: per-skill stats — invocations, pass/fail rate, co-use patterns
3. **Signal**: improvement signals from usage data

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| Skill name provided | Non-empty string |
| Log path writable | Parent dir exists or creatable |
| Outcome is pass/fail | Value in ["pass", "fail"] |

## POST

| Condition | Output |
|-----------|--------|
| Log invoked | `"Logged: <skill> (<outcome>) → <log_path>"` |
| Analyze run | Usage table + improvement signals |
| No records | `"No records in <log_path>"` |
| Skills with signals | List of skill + signal type |
| Empty log | Exit 0, no error |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Skill name empty | `argparse` validation | Print error, exit 1 |
| Log path unreadable | `open()` raises `IOError` | Print warning, skip |
| Malformed JSONL line | `json.JSONDecodeError` | Skip line, continue |
| Missing required fields | KeyError on load | Skip line, continue |

## OBSERVABILITY

CLI output is self-documenting:
```
SKILL EVOLUTION TRACKER — D1 Analysis
  Total skills tracked: 3
  Total invocations:    47
  Total signals:        1

IMPROVEMENT SIGNALS:
  [hub-triad] high_fail_rate (3/7)
```
