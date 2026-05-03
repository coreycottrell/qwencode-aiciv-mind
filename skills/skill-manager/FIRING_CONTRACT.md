---
name: skill-manager
description: Autonomous skill crystallizer — scan skills/, detect duplicates, verify completeness, track known skills registry.
version: 1.0.0
trigger: python3 skill_manager.py list|check|check-file|verify-all
---

# Skill Manager Firing Contract

## WHEN

CLI invocation or programmatic import:
```bash
python3 skill_manager.py list              # List all known skills
python3 skill_manager.py check "<pattern>"  # Check if pattern exists
python3 skill_manager.py check-file <name>  # Verify skill completeness
python3 skill_manager.py verify-all         # Verify all skills

from skill_manager import scan_skills_dir, SkillInfo
skills = scan_skills_dir()
```

## WHAT

**Scans** `PROJECT_ROOT/skills/` and builds a registry with:
- Skill name, category, status
- Whether SKILL.md, FIRING_CONTRACT.md, tests exist
- Lines of code per skill

**Checks** if a proposed pattern overlaps with existing skills.

**Verifies** a skill has all required files (SKILL.md + FIRING_CONTRACT.md required, tests optional).

## PRE

| Prerequisite | How Verified |
|--------------|--------------|
| `skills/` directory exists | `Path(skills_base).exists()` |
| Skill directory is valid | Not startswith `_` or `.` |
| Python 3.10+ | `sys.version_info >= (3, 10)` |

## POST

| State | Condition |
|-------|-----------|
| `list` succeeds | Prints tabulated skills with status |
| `check` — no overlap | "✅ No existing skill matches this pattern." |
| `check` — overlap found | Lists matching skills with reason |
| `check-file` — complete | "✅ {name} verification complete" |
| `check-file` — missing files | Lists missing required files |
| `verify-all` — all complete | 6/6 skills pass |
| `verify-all` — some missing | Shows count of pass/fail |

## FAILURE

| Failure Mode | Detection | Recovery |
|-------------|-----------|----------|
| skills/ dir not found | `skills_base.exists()` returns False | Create `skills/` directory |
| Permission error reading files | `PermissionError` | Check file permissions |
| JSON parse in SKILL.md | Exception in `yaml.safe_load` | Validate frontmatter |

## OBSERVABILITY

All outputs are self-documenting CLI:
- `list`: Tabulated output with status, version, file presence, LOC
- `check`: Overlap warnings or clear path
- `check-file`: Per-file ✅/❌/⚠️ indicators
- `verify-all`: Per-skill verification summary

No external logging required — CLI output is the contract.

## SKILL COMPLETENESS STANDARD

A skill is **complete** if it has:
- ✅ `SKILL.md` (with `name:`, `description:`, `version:` frontmatter)
- ✅ `FIRING_CONTRACT.md` (with all 6 sections: WHEN/WHAT/PRE/POST/FAILURE/OBSERVABILITY)
- ⚠️ `test_*.py` (optional but recommended)

Current complete skills: `session-summarization`, `tdd`, `trajectory-compressor`
Missing tests: `atropos-grpo`, `hub-triad`, `skill-manager`

## CLI OUTPUT EXAMPLES

```
$ skill_manager.py list
Name                           Status    Version  SKILL  FC   Tests  LOC   Path
----------------------------------------------------------------------------------------------------
tdd                            PASS      1.0.0    YES    YES  YES    231   skills/tdd
session-summarization          PASS      1.0.0    YES    YES  YES    547   skills/session-summarization
trajectory-compressor          PASS      1.0.0    YES    YES  YES    650   skills/trajectory-compressor

$ skill_manager.py check "token cap enforcement"
Checking pattern: "token cap enforcement"
⚠️  session-summarization (category match: context) — skills/session-summarization

$ skill_manager.py verify-all
Verifying all 6 skills...
  ✅ tdd                       S=True F=True t=True
  ✅ session-summarization     S=True F=True t=True
  ⚠️  hub-triad                S=True F=True t=False
Summary: 3/6 skills have all required files
```
