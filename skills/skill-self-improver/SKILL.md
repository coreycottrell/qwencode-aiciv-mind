---
name: skill-self-improver
description: Class-first self-improvement loop — rubric-based skill review biased toward just-loaded skill per Hermes D1
version: 0.1.0
applicable_civs: [hengshi, proof, works, acg]
---

# Skill Self-Improver — Hermes #2

Complements Curator (which GRADES skills) by IMPROVING them based on grade data.
Class-first: models skill improvement as first-class objects with runtime inheritance.

## Problem It Solves

Curator grades skills (PASS/FAIL/WARN) but produces no improvement prescription.
Skill-evolution-tracker tracks invocations but surfaces only coarse signals.
This skill bridges both: takes FAIL/WARN skills, applies a structured rubric,
and generates a `v_next` improvement proposal.

## How It Works

1. **Input**: grade log from Curator (`memories/skill-curator-grades-ACG-v02.jsonl`)
   and/or invocation log from evolution tracker (`memories/skills-usage-log.jsonl`)
2. **Rubric scoring**: 6-dimension rubric (name, description, examples, FC completeness,
   test coverage, co-use readiness)
3. **Runtime inheritance**: reads existing SKILL.md as parent class, extends it
4. **Output**: per-skill improvement suggestion stored as `v_next.md` next to SKILL.md

## Design Notes

- Biased toward "just-loaded" skill: skills recently graded by Curator get priority
- Handles references (links to related skills), templates (reusable patterns),
  and sub-files (supplementary docs)
- v_next.md format mirrors SKILL.md frontmatter so it can be merged via inheritance
- Self-improving: the skill itself must pass its own rubric (meta-improvement)
