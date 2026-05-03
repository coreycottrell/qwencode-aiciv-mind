---
name: skill-curator
description: Track compute sessions, measure active vs idle time, surface hibernation candidates per D4
version: 1.0.0
---

# Skill Curator — Firing Contract

## WHEN

```bash
# Grade all skills (discover + grade + log to JSONL)
python3 skill_curator.py grade --skills-dir autonomy/skills --output memories/skill-curator-grades.jsonl

# Generate stub FC files for FAIL skills
python3 skill_curator.py generate_fc --skills-dir autonomy/skills

# Analyze grades from log
python3 skill_curator.py analyze --log memories/skill-curator-grades.jsonl
```

Triggered by:
- ACG directive to address 219-FC gap
- Periodic skill library audits
- On-demand for specific skills directories

## WHAT

Autonomous skill library curator: grades, consolidates, prunes skills per cycle.
For each skill discovered:
- Grades: PASS (has FC + frontmatter), FAIL (missing FC), WARN (frontmatter partial)
- Logs one JSONL record per skill graded
- Generates stub FC files for FAIL skills (generate_fc subcommand)
- Produces summary with counts

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| Skills dir exists | `Path(skills_dir).exists()` |
| Skills dir readable | Attempt `listdir()` |

## POST

| Condition | Output |
|-----------|--------|
| grade | JSONL with one record per skill + summary with pass/fail/warn counts |
| generate_fc | Stub FC.md files written for FAIL skills |
| analyze | Summary table from pre-existing JSONL log |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Skills dir not found | `Path.exists()` returns False | Print error, exit 1 |
| Malformed SKILL.md | YAML parse fails | Skip skill, log reason |
| Log write fails | `IOError` | Print error, exit 1 |

## OBSERVABILITY

```
SKILL CURATOR — Hermes D1+D5
  Skills directory: /path/to/skills
  Total skills:     219
  PASS:             1 (0%)
  WARN:             0 (0%)
  FAIL:             218 (100%)
  Log:              memories/skill-curator-grades.jsonl

FAILURES (missing FIRING_CONTRACT.md):
  test-driven-development: missing_frontmatter
  capability-curator: missing_firing_contract
  ...
```

## Evidence for Claims

D1+D5: Skills FORM from experience, IMPROVE during use, and must be CURATED.
Grade output is the evidence artifact. JSONL log IS the receipt per O15.
