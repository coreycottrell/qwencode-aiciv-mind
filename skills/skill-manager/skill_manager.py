#!/usr/bin/env python3
"""
Skill Manager — Autonomous Skill Crystallizer

Detects repeated patterns, generates skill files, tracks known skills.
The meta-skill that creates skills from successful integration cycles.

Usage:
    python3 skill_manager.py list
    python3 skill_manager.py check "<pattern>"
    python3 skill_manager.py check-file <skill-name>
    python3 skill_manager.py verify <skill-name>
"""

import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

SKILLS_DIR = Path(__file__).parent.parent  # /skills/
PROJECT_ROOT = Path(__file__).parent.parent.parent


# ───────────────────────────────────────────────────────────────────
# Skill Registry
# ───────────────────────────────────────────────────────────────────

@dataclass
class SkillInfo:
    name: str
    category: str
    status: str  # PASS | PASS-DORMANT | PARTIAL | MERGED | BLOCKED | PENDING
    validator: Optional[str]  # Proof | Works | ACG
    version: str
    has_skill_md: bool
    has_firing_contract: bool
    has_tests: bool
    lines_of_code: int
    path: str


SKILL_REGISTRY: list[SkillInfo] = []


def scan_skills_dir() -> list[SkillInfo]:
    """Scan skills/ directory and build registry of known skills."""
    skills = []
    skills_base = PROJECT_ROOT / "skills"

    if not skills_base.exists():
        return skills

    for skill_path in skills_base.iterdir():
        if not skill_path.is_dir():
            continue
        if skill_path.name.startswith("_") or skill_path.name.startswith("."):
            continue

        name = skill_path.name

        # Check SKILL.md
        skill_md = skill_path / "SKILL.md"
        has_skill_md = skill_md.exists()

        # Check FIRING_CONTRACT.md
        firing_contract = skill_path / "FIRING_CONTRACT.md"
        has_firing_contract = firing_contract.exists()

        # Check test files
        test_files = list(skill_path.glob("test_*.py"))
        has_tests = len(test_files) > 0

        # Count lines of code
        total_lines = 0
        for py_file in skill_path.glob("*.py"):
            try:
                total_lines += len(py_file.read_text().splitlines())
            except Exception:
                pass

        # Determine status from metadata or file presence
        status = "PENDING"
        validator = None
        version = "unknown"

        if has_skill_md:
            try:
                content = skill_md.read_text()
                if "VALIDATOR-PASSED" in content or "VALIDATOR PASS" in content:
                    status = "PASS"
                elif "PASS-DORMANT" in content:
                    status = "PASS-DORMANT"
                elif "PARTIAL" in content:
                    status = "PARTIAL"
                elif "MERGED" in content:
                    status = "MERGED"
                elif "BLOCKED" in content:
                    status = "BLOCKED"

                # Extract version
                for line in content.split("\n"):
                    if line.strip().startswith("version:"):
                        version = line.split("version:")[1].strip()
                        break
            except Exception:
                pass

        # Category
        category = "unknown"
        if has_skill_md:
            try:
                for line in skill_md.read_text().split("\n"):
                    if line.strip().startswith("tags:"):
                        tags_content = line.split("tags:")[1].strip()
                        category = tags_content.split("[")[1].split("]")[0] if "[" in tags_content else "general"
                        break
            except Exception:
                pass

        skills.append(SkillInfo(
            name=name,
            category=category,
            status=status,
            validator=validator,
            version=version,
            has_skill_md=has_skill_md,
            has_firing_contract=has_firing_contract,
            has_tests=has_tests,
            lines_of_code=total_lines,
            path=str(skill_path.relative_to(PROJECT_ROOT)),
        ))

    return skills


def list_skills() -> None:
    """List all known skills."""
    skills = scan_skills_dir()
    print(f"{'Name':<30} {'Status':<15} {'Version':<8} {'SKILL':<6} {'FC':<4} {'Tests':<6} {'LOC':<5} {'Path'}")
    print("-" * 100)
    for s in sorted(skills, key=lambda x: x.name):
        print(
            f"{s.name:<30} {s.status:<15} {s.version:<8} "
            f"{'YES' if s.has_skill_md else 'NO':<6} "
            f"{'YES' if s.has_firing_contract else 'NO':<4} "
            f"{'YES' if s.has_tests else 'NO':<6} "
            f"{s.lines_of_code:<5} {s.path}"
        )


def check_pattern(pattern: str) -> None:
    """Check if a pattern already exists in known skills."""
    skills = scan_skills_dir()
    pattern_lower = pattern.lower()

    print(f"Checking pattern: {pattern!r}")
    print()

    matches = []
    for s in skills:
        if pattern_lower in s.name.lower():
            matches.append((s, f"name match: {s.name}"))
        elif pattern_lower in s.category.lower():
            matches.append((s, f"category match: {s.category}"))

    if matches:
        print(f"Found {len(matches)} potential overlap(s):")
        for s, reason in matches:
            print(f"  ⚠️  {s.name} ({reason}) — {s.path}")
            print(f"      Status: {s.status}, SKILL.md: {'YES' if s.has_skill_md else 'NO'}, "
                  f"FC: {'YES' if s.has_firing_contract else 'NO'}, Tests: {'YES' if s.has_tests else 'NO'}")
    else:
        print("✅ No existing skill matches this pattern.")
        print("   Safe to create new skill.")


def check_file(skill_name: str) -> None:
    """Verify a skill has all required files."""
    skill_path = PROJECT_ROOT / "skills" / skill_name

    print(f"Verifying skill: {skill_name}")
    print(f"Path: {skill_path}")
    print()

    if not skill_path.exists():
        print(f"❌ Skill directory does not exist: {skill_path}")
        return

    if not skill_path.is_dir():
        print(f"❌ {skill_path} is not a directory")
        return

    required = {
        "SKILL.md": skill_path / "SKILL.md",
        "FIRING_CONTRACT.md": skill_path / "FIRING_CONTRACT.md",
    }

    optional_files = list(skill_path.glob("test_*.py"))
    optional_py = [f for f in skill_path.glob("*.py") if f.name not in ["__init__.py", "test_runner.py"]]
    optional_evidence = skill_path / "test_run_evidence.md"

    all_ok = True
    for name, path in required.items():
        if path.exists():
            print(f"  ✅ {name}")
        else:
            print(f"  ❌ {name} — MISSING (required)")
            all_ok = False

    if optional_files:
        print(f"  ✅ test_*.py: {[p.name for p in optional_files]}")
    else:
        print(f"  ⚠️  test_*.py: not found (optional)")

    if optional_py:
        print(f"  ✅ *.py: {[p.name for p in optional_py]}")
    else:
        print(f"  ⚠️  *.py: not found")

    if optional_evidence.exists():
        print(f"  ✅ test_run_evidence.md: {optional_evidence.name}")
    else:
        print(f"  ⚠️  test_run_evidence.md: not found (optional)")

    # Check SKILL.md has all 6 firing contract fields
    skill_md = skill_path / "SKILL.md"
    if skill_md.exists():
        content = skill_md.read_text()
        has_version = "version:" in content
        has_description = "description:" in content
        print(f"\n  SKILL.md checks:")
        print(f"    version field: {'✅' if has_version else '❌'}")
        print(f"    description field: {'✅' if has_description else '❌'}")

    # Check FIRING_CONTRACT.md has all 6 fields
    fc = skill_path / "FIRING_CONTRACT.md"
    if fc.exists():
        content = fc.read_text()
        required_fc_fields = ["WHEN", "WHAT", "PRE", "POST", "FAILURE", "OBSERVABILITY"]
        print(f"\n  FIRING_CONTRACT.md checks:")
        for field in required_fc_fields:
            has_field = f"## {field}" in content
            print(f"    {field}: {'✅' if has_field else '❌'}")

    if all_ok:
        print(f"\n✅ {skill_name} verification complete — all required files present")
    else:
        print(f"\n⚠️  {skill_name} verification complete — some required files missing")


def verify_all() -> None:
    """Verify all skills in the skills directory."""
    skills = scan_skills_dir()
    print(f"Verifying all {len(skills)} skills...")
    print()

    results = []
    for s in sorted(skills, key=lambda x: x.name):
        skill_path = PROJECT_ROOT / "skills" / s.name
        checks = {
            "SKILL.md": (skill_path / "SKILL.md").exists(),
            "FC.md": (skill_path / "FIRING_CONTRACT.md").exists(),
            "tests": len(list(skill_path.glob("test_*.py"))) > 0,
        }
        all_ok = all(checks.values())
        status_icon = "✅" if all_ok else "⚠️"
        checks_str = " ".join(f"{k[0]}={v}" for k, v in checks.items())
        print(f"  {status_icon} {s.name:<30} {checks_str}")
        results.append((s.name, all_ok))

    print()
    passed = sum(1 for _, ok in results if ok)
    print(f"Summary: {passed}/{len(results)} skills have all required files")


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: skill_manager.py <list|check|check-file|verify|verify-all> [args...]")
        print()
        print("Commands:")
        print("  list              — List all known skills")
        print("  check <pattern>   — Check if pattern already exists")
        print("  check-file <name> — Verify skill has all required files")
        print("  verify-all        — Verify all skills")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "list":
        list_skills()
    elif cmd == "check":
        if len(sys.argv) < 3:
            print("Usage: skill_manager.py check <pattern>")
            sys.exit(1)
        check_pattern(sys.argv[2])
    elif cmd == "check-file":
        if len(sys.argv) < 3:
            print("Usage: skill_manager.py check-file <skill-name>")
            sys.exit(1)
        check_file(sys.argv[2])
    elif cmd == "verify-all":
        verify_all()
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
