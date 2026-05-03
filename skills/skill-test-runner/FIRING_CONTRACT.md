---
name: skill-test-runner
description: Batch test ACG's skill stack — discover skills, sanity-check each (parse SKILL.md, verify frontmatter, check FIRING_CONTRACT.md), run optional smoke test, log to JSONL
version: 1.0.0
---

# Skill Test Runner — Firing Contract

## WHEN

```bash
python3 skills/skill-test-runner/skill_test_runner.py \
    --skills-dir /path/to/skills \
    --output memories/skills-test-log.jsonl
```

Triggered by:
- ACG HERMES-WAKE cron (every ~10 min)
- Manual invocation before shipping a skill integration
- Pre-commit hook on skill authorship

## WHAT

Reads `autonomy/skills/` tree, tests each skill:
1. `SKILL.md` exists + parseable (YAML frontmatter extracted)
2. Frontmatter has `name` + `description` fields (O8/O22 minimum)
3. `FIRING_CONTRACT.md` present (O8/O22 compliance check)
4. `test_skill.py` or `test.sh` runs if present (optional smoke test)

Writes JSONL log — one record per skill with pass/fail + error details.

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| Skills dir exists | `Path(skills_dir).exists()` |
| At least one skill dir | `discover_skills()` returns non-empty |
| Output path writable | Parent dir exists or is creatable |

## POST

| Condition | Output |
|-----------|--------|
| 213 skills found | `Skills found: 213` |
| Skill with valid frontmatter + FC | `status: PASS` |
| Skill missing FC | `status: FAIL`, error: `"FIRING_CONTRACT.md missing (O8/O22 compliance)"` |
| Skill missing required fields | `status: FAIL`, error: `"Missing required fields: [...]"` |
| SKILL.md is a directory (edge case) | `status: SKIP`, error: `"SKILL.md missing or is a directory"` |
| Smoke test script exists | Runs subprocess, records pass/fail |
| No smoke test | `smoke_test_pass: null`, no error |
| JSONL log written | `memories/skills-test-log.jsonl` |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Skills dir not found | `Path(skills_dir).exists() == False` | Print error, exit 1 |
| Cannot read SKILL.md | Exception in `skill_md.read_text()` | Mark skill FAIL, continue batch |
| SKILL.md is directory | `skill_md.is_file() == False` | Mark skill SKIP, continue batch |
| JSONL write fails | `IOError` on open | Print warning, continue |
| Smoke test timeout | `subprocess.TimeoutExpired` | Mark smoke_test_pass: null, continue |

## OBSERVABILITY

CLI output is self-documenting:
```
SKILL TEST RUNNER — Hermes D5 Batch
  Skills found:    213
  PASS:           0
  FAIL:           201
  SKIP:           12
  Pass rate:      0/213
  JSONL log:      memories/skills-test-log.jsonl
```

JSONL log is the primary evidence artifact — each line is a complete per-skill test record.

## Evidence for Claims

To claim "skill-test-runner v1 shipped" to ACG:

```bash
python3 skills/skill-test-runner/skill_test_runner.py \
    --skills-dir /home/corey/projects/AI-CIV/ACG/autonomy/skills \
    --output memories/skills-test-log.jsonl

# Evidence:
# - 213 skills discovered and tested
# - JSONL log at memories/skills-test-log.jsonl
# - 201 FAIL: all missing FIRING_CONTRACT.md — baseline gap identified
# - 12 SKIP: SKILL.md missing or directory
# - 0 PASS: no skill has both frontmatter + FC + smoke test yet
```
