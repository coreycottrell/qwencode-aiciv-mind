---
name: skill-test-runner
description: Hermes D5 batch runner — discovers skills under autonomy/skills/ + .claude/skills/, runs sanity pass (SKILL.md parse, frontmatter check, firing-contract per O8/O22, optional smoke test), produces pass/fail JSON + JSONL log
version: 1.0.0
applicable_civs: [hengshi, proof, works, acg]
---

# Skill Test Runner — Hermes D5

Discovers skills under `autonomy/skills/` + `.claude/skills/`, runs a sanity pass per skill, produces pass/fail JSON + JSONL log.

## Problem It Solves

ACG has 213 skills under `autonomy/skills/`. None have been systematically tested. The Hermes Agent doctrine (D5) proves skills are testable — batch runners with pass/fail + logged results catch regressions before they reach production. Without a test runner, skill authors ship untested code and users discover breakage at runtime.

`skill-test-runner` applies the Hermes D5 pattern: discover → sanity-check → smoke-test → log results. It runs against ACG's full skill stack and produces evidence for every integration claim.

## How It Works

1. **Discover** — walks `autonomy/skills/` + `.claude/skills/`, finds all `SKILL.md` files
2. **Parse** — extracts YAML frontmatter (--- ... ---)
3. **Check required fields** — `name`, `description` (O8/O22 minimum contract)
4. **Firing contract check** — FIRING_CONTRACT.md present? (O8/O22 compliance)
5. **Smoke test** — runs `test_skill.py` or `test.sh` if present (optional)
6. **Log** — appends JSONL record per skill + summary JSON

## Output Format

```
SKILL TEST RUNNER — Hermes D5 Batch
  Skills found:    213
  PASS:           0
  FAIL:           201
  SKIP:           12
  Pass rate:      0/213
  JSONL log:      memories/skills-test-log.jsonl

FAILED SKILLS:
  [skill-name]
    - FIRING_CONTRACT.md missing (O8/O22 compliance)
    - Missing required fields: ['name']
```

## Running

```bash
python3 skills/skill-test-runner/skill_test_runner.py \
    --skills-dir /home/corey/projects/AI-CIV/ACG/autonomy/skills \
    --output memories/skills-test-log.jsonl
```

## Firing Contract Evidence

The JSONL log is the evidence artifact. Each entry records:
- `skill_name`, `skill_path`, `status` (PASS/FAIL/SKIP)
- Per-check results (`checks` dict)
- `errors` list — what failed and why
- `firing_contract_found` boolean
- `smoke_test_pass` (null if no test run)

## Design Notes

- No external dependencies beyond stdlib
- YAML frontmatter parsed manually (no pyyaml needed)
- Smoke tests are OPTIONAL — skills without tests get a FIRING_CONTRACT check but no execution check
- JSONL log is append-only — successive runs accumulate, use `--output` with fresh path per run
- SKILL.md must be a file (not a directory) — handles edge cases where `SKILL.md` is a subdirectory
