#!/usr/bin/env python3
"""
Skill Curator — Hermes D1 + D5

Autonomous skill library curator: grades/consolidates/prunes skills per cycle.
Addresses the 219-FC gap discovered by skill-test-runner.

Usage:
    python3 skill_curator.py grade [--skills-dir <path>] [--output <path>]
    python3 skill_curator.py generate_fc [--skills-dir <path>] [--output <path>]
    python3 skill_curator.py analyze [--log <path>]

Grade output (JSONL):
    {"skill": "...", "grade": "PASS|FAIL|WARN", "reasons": [...], "civ": "hengshi", "ts": "..."}

Generate FC output:
    Stub FIRING_CONTRACT.md files for FAIL skills.
"""

import argparse
import json
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

from skill_walker import find_skills

# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

SKILL_MD = "SKILL.md"
FC_MD = "FIRING_CONTRACT.md"

SPECIAL_CONTAINERS = {"custom", "flows", "wake-up-modes", "autonomy"}


@dataclass
class SkillGrade:
    skill_path: str
    skill_name: str
    grade: str  # PASS | FAIL | WARN
    reasons: list[str]
    has_skill_md: bool
    has_fc: bool
    frontmatter_ok: bool
    frontmatter: Optional[dict] = None  # parsed from SKILL.md for use by generate_fc
    civ: str = "unknown"
    ts: str = ""

    def to_dict(self) -> dict:
        return {
            "skill": self.skill_name,
            "skill_path": self.skill_path,
            "grade": self.grade,
            "reasons": self.reasons,
            "has_skill_md": self.has_skill_md,
            "has_fc": self.has_fc,
            "frontmatter_ok": self.frontmatter_ok,
            "frontmatter": self.frontmatter,
            "civ": self.civ,
            "ts": self.ts,
        }


# ───────────────────────────────────────────────────────────────────
# Skill Discovery
# ───────────────────────────────────────────────────────────────────


def parse_frontmatter(text: str) -> tuple[Optional[dict], str]:
    """Parse YAML frontmatter from markdown text. Returns (frontmatter dict, body text)."""
    if not text.strip().startswith("---"):
        return None, text
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None, text
    yaml_block = parts[1]
    body = parts[2]
    # Simple YAML parsing for name + description
    fm = {}
    for line in yaml_block.splitlines():
        if ":" in line:
            key, _, value = line.partition(":")
            fm[key.strip()] = value.strip().strip('"').strip("'")
    return fm, body


def check_frontmatter(skill_md: Path) -> tuple[bool, list[str]]:
    """Check if SKILL.md has required name+description frontmatter fields."""
    ok, reasons, _ = check_frontmatter_full(skill_md)
    return ok, reasons


def check_frontmatter_full(skill_md: Path) -> tuple[bool, list[str], Optional[dict]]:
    """Check if SKILL.md has required name+description frontmatter fields. Returns (ok, reasons, fm_dict)."""
    try:
        text = skill_md.read_text()
    except Exception:
        return False, ["skill_md_read_error"], None

    fm, _ = parse_frontmatter(text)
    if fm is None:
        return False, ["no_frontmatter"], None

    missing = []
    if not fm.get("name"):
        missing.append("name_missing")
    if not fm.get("description"):
        missing.append("description_missing")

    return len(missing) == 0, missing, fm


def grade_skill(skill_path: Path, civ: str) -> SkillGrade:
    """Grade a single skill: PASS (has FC + frontmatter), FAIL (missing FC), WARN (partial)."""
    skill_name = skill_path.name
    reasons = []
    has_skill_md = False
    has_fc = False
    frontmatter_ok = False
    frontmatter = None

    skill_md = skill_path / SKILL_MD
    if skill_md.is_file():
        has_skill_md = True
        frontmatter_ok, fm_reasons, fm = check_frontmatter_full(skill_md)
        frontmatter = fm
        reasons.extend(fm_reasons)
    else:
        reasons.append("missing_skill_md")

    fc_path = skill_path / FC_MD
    if fc_path.is_file():
        has_fc = True
    else:
        reasons.append("missing_firing_contract")

    # Grade
    if has_fc and frontmatter_ok:
        grade = "PASS"
    elif has_fc and not frontmatter_ok:
        grade = "WARN"
    else:
        grade = "FAIL"

    return SkillGrade(
        skill_path=str(skill_path),
        skill_name=skill_name,
        grade=grade,
        reasons=reasons,
        has_skill_md=has_skill_md,
        has_fc=has_fc,
        frontmatter_ok=frontmatter_ok,
        frontmatter=frontmatter,
        civ=civ,
        ts=datetime.now(timezone.utc).isoformat(),
    )


# ───────────────────────────────────────────────────────────────────
# FC Generation
# ───────────────────────────────────────────────────────────────────

def generate_fc_stub(skill_path: Path, frontmatter: Optional[dict] = None) -> str:
    """Generate a FIRING_CONTRACT.md stub for a skill missing one.

    If frontmatter is provided (from SKILL.md parsing), use it to pre-fill
    name and description so the stub is more meaningful from the start.
    """
    skill_name = skill_path.name
    fm_name = frontmatter.get("name", "") if frontmatter else ""
    fm_desc = frontmatter.get("description", "") if frontmatter else ""

    return f"""---
name: {fm_name or skill_name}
description: {fm_desc or "TODO: Write skill description — what problem does this solve?"}
version: 0.1.0
---

# {skill_name} — Firing Contract

## WHEN

```bash
python3 skills/{skill_name}/run.py
```

Triggered by:
- TODO: What triggers this skill?

## WHAT

{fm_desc or "TODO: Describe what this skill does."}

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| TODO | TODO |

## POST

| Condition | Output |
|-----------|--------|
| TODO | TODO |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| TODO | TODO | TODO |

## OBSERVABILITY

TODO: What evidence does this skill produce?

## Evidence for Claims

TODO: What claims does this skill make, and what evidence proves them?
"""


# ───────────────────────────────────────────────────────────────────
# CLI Commands
# ───────────────────────────────────────────────────────────────────

def cmd_grade(args):
    root = Path(args.skills_dir).expanduser() if args.skills_dir else Path("autonomy/skills")
    output = Path(args.output).expanduser() if args.output else Path("memories/skill-curator-grades.jsonl")
    civ = args.civ or "hengshi"

    if not root.exists():
        print(f"Skills directory not found: {root}")
        sys.exit(1)

    skills = find_skills(root)
    if not skills:
        print(f"No skills found under {root}")
        sys.exit(1)

    output.parent.mkdir(parents=True, exist_ok=True)
    grades = []
    for sp in skills:
        g = grade_skill(sp, civ)
        grades.append(g)

    output.parent.mkdir(parents=True, exist_ok=True)
    with open(output, "w") as f:
        for g in grades:
            f.write(json.dumps(g.to_dict()) + "\n")

    # Summary
    passed = sum(1 for g in grades if g.grade == "PASS")
    warned = sum(1 for g in grades if g.grade == "WARN")
    failed = sum(1 for g in grades if g.grade == "FAIL")
    total = len(grades)

    print("")
    print("=" * 60)
    print("SKILL CURATOR — Hermes D1+D5")
    print("=" * 60)
    print(f"  Skills directory: {root}")
    print(f"  Total skills:     {total}")
    print(f"  PASS:             {passed} ({passed/total*100:.0f}%)" if total else "  PASS: 0")
    print(f"  WARN:             {warned} ({warned/total*100:.0f}%)" if total else "  WARN: 0")
    print(f"  FAIL:             {failed} ({failed/total*100:.0f}%)" if total else "  FAIL: 0")
    print(f"  Log:              {output}")
    print("")

    if failed > 0:
        print("FAILURES (missing FIRING_CONTRACT.md):")
        for g in grades:
            if g.grade == "FAIL":
                print(f"  {g.skill_name}: {', '.join(g.reasons)}")
        print("")

    return grades


def cmd_generate_fc(args):
    root = Path(args.skills_dir).expanduser() if args.skills_dir else Path("autonomy/skills")
    civ = args.civ or "hengshi"
    dry_run = args.dry_run

    if not root.exists():
        print(f"Skills directory not found: {root}")
        sys.exit(1)

    skills = find_skills(root)
    grades = {sp.name: grade_skill(sp, civ) for sp in skills}
    generated = 0

    for sp in skills:
        fc_path = sp / FC_MD
        if fc_path.exists():
            continue

        g = grades.get(sp.name)
        stub = generate_fc_stub(sp, frontmatter=g.frontmatter if g else None)
        if dry_run:
            print(f"[DRY RUN] Would create: {fc_path}")
        else:
            fc_path.write_text(stub)
            print(f"Created: {fc_path}")
        generated += 1

    print(f"\nGenerated {generated} FIRING_CONTRACT.md stubs")
    if dry_run:
        print("(dry run — no files written)")


def cmd_analyze(args):
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skill-curator-grades.jsonl")
    if not log_path.exists():
        print(f"No log found: {log_path}")
        sys.exit(1)

    grades = []
    with open(log_path) as f:
        for line in f:
            try:
                grades.append(json.loads(line))
            except json.JSONDecodeError:
                continue

    if not grades:
        print(f"No records in {log_path}")
        sys.exit(0)

    passed = [g for g in grades if g.get("grade") == "PASS"]
    warned = [g for g in grades if g.get("grade") == "WARN"]
    failed = [g for g in grades if g.get("grade") == "FAIL"]
    total = len(grades)

    print("")
    print("=" * 60)
    print("SKILL CURATOR — Grade Analysis")
    print("=" * 60)
    print(f"  Total grades:  {total}")
    print(f"  PASS:         {len(passed)} ({len(passed)/total*100:.0f}%)" if total else "  PASS: 0")
    print(f"  WARN:         {len(warned)} ({len(warned)/total*100:.0f}%)" if total else "  WARN: 0")
    print(f"  FAIL:         {len(failed)} ({len(failed)/total*100:.0f}%)" if total else "  FAIL: 0")
    print("")

    if failed:
        print("FAILURES:")
        for g in failed:
            print(f"  {g.get('skill')}: {', '.join(g.get('reasons', []))}")
        print("")


def main():
    parser = argparse.ArgumentParser(description="Skill Curator — Hermes D1+D5")
    sub = parser.add_subparsers()

    p_grade = sub.add_parser("grade", help="Grade all skills in a directory")
    p_grade.add_argument("--skills-dir", default=None, help="Skills root directory")
    p_grade.add_argument("--output", default=None, help="Output JSONL path")
    p_grade.add_argument("--civ", default="hengshi", help="Civ name")
    p_grade.set_defaults(func=cmd_grade)

    p_gen = sub.add_parser("generate_fc", help="Generate stub FC files for FAIL skills")
    p_gen.add_argument("--skills-dir", default=None, help="Skills root directory")
    p_gen.add_argument("--civ", default="hengshi", help="Civ name")
    p_gen.add_argument("--dry-run", action="store_true", help="Show what would be generated")
    p_gen.set_defaults(func=cmd_generate_fc)

    p_analyze = sub.add_parser("analyze", help="Analyze grades from JSONL log")
    p_analyze.add_argument("--log", default=None, help="Log path")
    p_analyze.set_defaults(func=cmd_analyze)

    args = parser.parse_args()
    if hasattr(args, "func"):
        args.func(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
