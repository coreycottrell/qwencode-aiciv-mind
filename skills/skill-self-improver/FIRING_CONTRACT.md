---
name: skill-self-improver
description: Class-first self-improvement loop — rubric-based skill review biased toward just-loaded skill per Hermes D1
version: 0.1.0
---

# Skill Self-Improver — Firing Contract

## WHEN

```bash
# Run improvement on FAIL/WARN skills from curator log (primary use)
python3 skills/skill-self-improver/skill_self_improver.py improve --log memories/skill-curator-grades-ACG-v02.jsonl

# Score a single skill against rubric
python3 skills/skill-self-improver/skill_self_improver.py rubric-score skills/skill-name [--verbose]

# List top improvement suggestions
python3 skills/skill-self-improver/skill_self_improver.py suggest --limit 10

# Run full improvement on skills directory
python3 skills/skill-self-improver/skill_self_improver.py run --skills-dir autonomy/skills
```

Triggered by:
- ACG directive to ship Hermes #2 self-improvement loop
- Skill library review cycles
- Curator grade data available (complementary to Curator which GRADES skills)

## WHAT

Class-first self-improvement loop. Takes FAIL/WARN skills from Curator's grade log, applies a 6-dimension
rubric (name_quality, description_clarity, examples_coverage, fc_completeness, test_coverage, co_use_readiness),
and generates a `v_next.md` improvement proposal per skill.

For each FAIL/WARN skill:
1. Read SKILL.md + FIRING_CONTRACT.md
2. Score against 6-dimension rubric
3. Generate actionable suggestions
4. Write v_next.md proposal to memories/vnext/
5. Log one record per skill improved

## PRE

| Prerequisite | How Verified |
|--------------|-------------|
| Grade log exists | `Path(log).exists()` |
| Skills dir readable | Attempt `listdir()` on skill_path |
| Python yaml module | `import yaml` succeeds |

## POST

| Condition | Output |
|-----------|--------|
| improve | v_next.md files in memories/vnext/, one per FAIL/WARN skill |
| rubric-score | Printed rubric breakdown + suggestions |
| suggest | Ranked list of top suggestions from grade log |
| run | Full scan, v_next files for all below-threshold skills |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Grade log not found | `Path.exists()` returns False | Print error, exit 1 |
| Skill path not found | Directory doesn't exist | Skip skill, log |
| YAML parse error | `yaml.safe_load()` raises | Skip skill, continue |
| Output dir write fails | `IOError` | Print error, exit 1 |

## OBSERVABILITY

```
SKILL SELF-IMPROVER — Hermes #2
  Grade log:         memories/skill-curator-grades-ACG-v02.jsonl
  Total FAIL/WARN:  216
  High priority:    14
  v_next files:     memories/vnext/
  Run complete:     216 skills scored
```

## Evidence for Claims

Hermes #2: Class-first loop complements Curator (GRADES) by IMPROVING based on grades.
Rubric scoring provides objective improvement targets.
v_next.md IS the evidence artifact per O15.
