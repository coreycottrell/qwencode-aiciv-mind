---
name: skill-evolution-tracker
description: Track skill invocations and surface improvement signals per Hermes D1 — skills FORM from experience and IMPROVE during use
version: 1.0.0
applicable_civs: [hengshi, proof, works, acg]
---

# Skill Evolution Tracker — Hermes D1

Tracks skill invocations from real usage, detects patterns, surfaces improvement signals. D1: skills FORM from experience and IMPROVE during use — authored-static skills are a lower-energy state.

## Problem It Solves

ACG has 219 skills. None track usage. Without usage data, skills that should evolve stay static, broken skills stay in production, and co-use patterns go undetected. The Hermes D1 doctrine says: if skills don't improve from use, they decay toward irrelevance.

`skill-evolution-tracker` provides the data layer: log invocations, analyze patterns, surface signals.

## Usage

```bash
# Log a skill invocation (after running a skill)
python3 skill_evolution_tracker.py log tdd --context "rust crates" --outcome pass

# Analyze all recorded usage
python3 skill_evolution_tracker.py analyze --log memories/skills-usage-log.jsonl

# Show improvement signals only
python3 skill_evolution_tracker.py signals --log memories/skills-usage-log.jsonl
```

## How It Works

1. **Log**: After running any skill, log it with context + outcome
2. **Analyze**: Scans all logged invocations, computes per-skill stats
3. **Signal**: Surfaces improvement signals:
   - `never_invocated` — exists but never used
   - `high_fail_rate(N/M)` — failure rate >30%
   - `stable_high_use` — >10 passes, 0 fails
   - `needs_review` — high invocations but pass_rate <70%

## Output Format

```
SKILL EVOLUTION TRACKER — D1 Analysis
  Total skills tracked: 3
  Total invocations:    47
  Total signals:        1

IMPROVEMENT SIGNALS:
  [hub-triad] high_fail_rate (3/7)

TOP SKILLS BY USAGE:
  session-summarization: 20 inv (90% pass)
  hub-triad: 7 inv (57% pass)
  skill-test-runner: 20 inv (100% pass)
```

## Design Notes

- JSONL log: one record per invocation, append-only
- Co-use detection: skills invoked by same civ within same day
- Improvement signals are advisory — they flag, not enforce
- No external dependencies beyond stdlib
- Log path configurable per invocation and analysis
