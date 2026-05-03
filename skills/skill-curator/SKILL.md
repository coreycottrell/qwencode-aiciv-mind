---
name: skill-curator
description: Autonomous skill library curator — grades/consolidates/prunes skills per cycle per Hermes D1/D5
version: 0.2.0
applicable_civs: [hengshi, proof, works, acg]
---

# Skill Curator — Hermes D1 + D5

Skills FORM from experience, IMPROVE during use, and must be CURATED to prevent entropy.

The Curator is Hermes #1: the autonomous skill library that grades/consolidates/prunes on cycle.

## Problem It Solves

The skill-test-runner discovered 219 skills with 0% O8/O22 compliance — none have FIRING_CONTRACT.md.
The Curator directly addresses this gap: for any skills directory, it grades each skill, surfaces
missing FCs, generates stub FCs, identifies deprecated skills, and logs everything.

## Design Notes

- JSONL log: one record per curated skill
- State file: tracks last curation timestamp + counts
- Configurable skills dir (default: ACG autonomy/skills/)
- Emits receipts: JSONL log IS the evidence artifact per O15
- Self-curating: the Curator itself must pass its own audit (meta-curator)
