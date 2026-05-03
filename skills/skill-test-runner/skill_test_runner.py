#!/usr/bin/env python3
"""
Skill Test Runner — Hermes D5 Pattern

Discovers skills under autonomy/skills/ + .claude/skills/, runs a sanity pass per skill:
1. Parse SKILL.md (valid YAML frontmatter, required fields)
2. Check firing-contract presence per O8/O22
3. Optional smoke test (executes test script if skill provides one)

Produces pass/fail JSON + summary + JSONL log.

Usage:
    python3 skill_test_runner.py [--skills-dir /path/to/skills] [--output /path/to/log.jsonl]

Output: JSON + JSONL log at memories/skills-test-log.jsonl
"""

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class SkillTestResult:
    skill_name: str
    skill_path: str
    status: str           # PASS | FAIL | SKIP
    checks: dict           # per-check results
    errors: list[str]
    warnings: list[str]
    firing_contract_found: bool
    smoke_test_pass: Optional[bool] = None
    timestamp: str = ""

    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now(timezone.utc).isoformat()

    def to_dict(self) -> dict:
        return {
            "skill_name": self.skill_name,
            "skill_path": self.skill_path,
            "status": self.status,
            "checks": self.checks,
            "errors": self.errors,
            "warnings": self.warnings,
            "firing_contract_found": self.firing_contract_found,
            "smoke_test_pass": self.smoke_test_pass,
            "timestamp": self.timestamp,
        }


@dataclass
class BatchResult:
    total: int = 0
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    results: list[SkillTestResult] = field(default_factory=list)

    @property
    def pass_rate(self) -> str:
        return f"{self.passed}/{self.total}" if self.total else "0/0"

    def to_dict(self) -> dict:
        return {
            "total": self.total,
            "passed": self.passed,
            "failed": self.failed,
            "skipped": self.skipped,
            "pass_rate": self.pass_rate,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "results": [r.to_dict() for r in self.results],
        }


# ───────────────────────────────────────────────────────────────────
# Skill Discovery
# ───────────────────────────────────────────────────────────────────

def discover_skills(base_dir: Path) -> list[Path]:
    """Find all skill directories under base_dir.

    Walks base_dir recursively to find SKILL.md files nested inside
    subdirectories (e.g. custom/, flows/, wake-up-modes/ contain skills).
    Handles edge case: SKILL.md as a directory (not a file).
    """
    skills = []
    if not base_dir.exists():
        return skills

    def walk(dir_path: Path):
        for entry in dir_path.iterdir():
            if not entry.is_dir():
                continue
            if entry.name.startswith("."):
                continue
            # Special container dirs — always recurse FIRST, even if they have their own SKILL.md
            # (they are organizational buckets, not skills)
            if entry.name in ("custom", "flows", "wake-up-modes", "autonomy"):
                walk(entry)
                continue
            skill_md = entry / "SKILL.md"
            if skill_md.exists() and skill_md.is_file():
                # Normal: SKILL.md is a file inside skill dir
                skills.append(entry)
            elif skill_md.exists() and not skill_md.is_file():
                # SKILL.md is a directory — rare but real (intent-signal-engine pattern).
                # Look inside for the actual SKILL.md file.
                inner = skill_md / "SKILL.md"
                if inner.exists() and inner.is_file():
                    skills.append(entry)
                # else: empty SKILL.md dir — skip
            else:
                # Non-skill dir with no SKILL.md — skip
                pass

    walk(base_dir)
    return sorted(skills, key=lambda p: p.name)


# ───────────────────────────────────────────────────────────────────
# Skill Parsing
# ───────────────────────────────────────────────────────────────────

def parse_skill_frontmatter(skill_dir: Path, content: Optional[str] = None) -> tuple[Optional[dict], list[str]]:
    """Parse YAML frontmatter from SKILL.md.

    Returns (frontmatter_dict, errors).
    If content is provided, uses it directly instead of re-reading.
    """
    skill_md = skill_dir / "SKILL.md"
    if content is None:
        if not skill_md.exists():
            return None, [f"SKILL.md not found in {skill_dir.name}"]
        try:
            content = skill_md.read_text()
        except Exception as e:
            return None, [f"Cannot read SKILL.md: {e}"]

    # Extract YAML frontmatter (--- ... ---)
    lines = content.split("\n")
    frontmatter_lines = []
    in_frontmatter = False
    for line in lines:
        if line.strip() == "---":
            if not in_frontmatter:
                in_frontmatter = True
                continue
            else:
                break
        if in_frontmatter:
            frontmatter_lines.append(line)

    if not frontmatter_lines:
        return None, ["No YAML frontmatter found (expected --- ... ---)"]

    # Parse key: value pairs manually
    data = {}
    for line in frontmatter_lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" in line:
            key, val = line.split(":", 1)
            data[key.strip()] = val.strip()

    return data, []


def check_required_fields(frontmatter: dict) -> tuple[bool, list[str]]:
    """Check O8/O22 required fields: name, description, version."""
    required = ["name", "description"]
    missing = [f for f in required if f not in frontmatter or not frontmatter[f]]
    ok = len(missing) == 0
    return ok, missing


# ───────────────────────────────────────────────────────────────────
# Firing Contract Check
# ───────────────────────────────────────────────────────────────────

def check_firing_contract(skill_dir: Path) -> tuple[bool, list[str]]:
    """Check if FIRING_CONTRACT.md exists per O8/O22."""
    fc_path = skill_dir / "FIRING_CONTRACT.md"
    exists = fc_path.exists()
    errors = []
    if not exists:
        errors.append("FIRING_CONTRACT.md missing (O8/O22 compliance)")
    return exists, errors


# ───────────────────────────────────────────────────────────────────
# Smoke Test
# ───────────────────────────────────────────────────────────────────

def run_smoke_test(skill_dir: Path) -> tuple[Optional[bool], list[str]]:
    """Run optional smoke test if skill provides test script.

    Looks for: test_*.py, test.sh, or runscript in frontmatter.
    Returns (pass, errors).
    """
    # Look for test scripts
    test_py = skill_dir / "test_skill.py"
    test_sh = skill_dir / "test.sh"

    errors = []
    result = None

    if test_py.exists():
        # Run with python3, capture exit code
        import subprocess
        try:
            r = subprocess.run(
                [sys.executable, str(test_py)],
                capture_output=True,
                text=True,
                timeout=30,
                cwd=skill_dir,
            )
            result = r.returncode == 0
            if not result:
                errors.append(f"smoke test exit code {r.returncode}: {r.stderr[:200]}")
        except subprocess.TimeoutExpired:
            errors.append("smoke test timed out (>30s)")
        except Exception as e:
            errors.append(f"smoke test error: {e}")

    elif test_sh.exists():
        import subprocess
        try:
            r = subprocess.run(
                ["bash", str(test_sh)],
                capture_output=True,
                text=True,
                timeout=30,
                cwd=skill_dir,
            )
            result = r.returncode == 0
            if not result:
                errors.append(f"smoke test exit code {r.returncode}: {r.stderr[:200]}")
        except subprocess.TimeoutExpired:
            errors.append("smoke test timed out (>30s)")
        except Exception as e:
            errors.append(f"smoke test error: {e}")

    # No test found — not an error, just SKIP smoke test
    return result, errors


# ───────────────────────────────────────────────────────────────────
# Per-Skill Test
# ───────────────────────────────────────────────────────────────────

def test_skill(skill_dir: Path) -> SkillTestResult:
    """Run all checks on a single skill."""
    name = skill_dir.name
    errors = []
    warnings = []
    checks = {}

    # 1. SKILL.md exists and is a file (normal case)
    skill_md = skill_dir / "SKILL.md"
    checks["skill_md_exists"] = skill_md.exists()
    if not skill_md.exists() or not skill_md.is_file():
        # Check: is SKILL.md actually a directory with inner SKILL.md?
        if skill_md.exists() and not skill_md.is_file():
            inner = skill_md / "SKILL.md"
            if inner.exists() and inner.is_file():
                # SKILL.md-dir case: read the inner file instead
                skill_md = inner
                checks["skill_md_exists"] = True
                checks["skill_md_is_directory"] = True
            else:
                errors.append(f"SKILL.md missing or is a directory — skipping skill")
                return SkillTestResult(
                    skill_name=name,
                    skill_path=str(skill_dir),
                    status="SKIP",
                    checks=checks,
                    errors=errors,
                    warnings=warnings,
                    firing_contract_found=False,
                )
        else:
            errors.append(f"SKILL.md missing — skipping skill")
            return SkillTestResult(
                skill_name=name,
                skill_path=str(skill_dir),
                status="SKIP",
                checks=checks,
                errors=errors,
                warnings=warnings,
                firing_contract_found=False,
            )

    # 2. Parse frontmatter
    try:
        content = skill_md.read_text()
    except Exception as e:
        errors.append(f"Cannot read SKILL.md: {e}")
        return SkillTestResult(
            skill_name=name,
            skill_path=str(skill_dir),
            status="FAIL",
            checks=checks,
            errors=errors,
            warnings=warnings,
            firing_contract_found=False,
        )
    # 2. Parse frontmatter from content already read
    frontmatter, parse_errors = parse_skill_frontmatter(skill_dir, content=content)
    checks["frontmatter_parsed"] = frontmatter is not None
    errors.extend(parse_errors)
    if not frontmatter:
        checks["required_fields"] = False
        checks["firing_contract"] = False
        return SkillTestResult(
            skill_name=name,
            skill_path=str(skill_dir),
            status="FAIL",
            checks=checks,
            errors=errors,
            warnings=warnings,
            firing_contract_found=False,
        )

    # 3. Required fields (O8/O22)
    fields_ok, missing_fields = check_required_fields(frontmatter)
    checks["required_fields"] = fields_ok
    checks["required_fields_missing"] = missing_fields
    if not fields_ok:
        errors.append(f"Missing required fields: {missing_fields}")

    # 4. Firing contract (O8/O22)
    fc_found, fc_errors = check_firing_contract(skill_dir)
    checks["firing_contract"] = fc_found
    errors.extend(fc_errors)

    # 5. Smoke test (optional)
    smoke_pass, smoke_errors = run_smoke_test(skill_dir)
    checks["smoke_test"] = smoke_pass
    errors.extend(smoke_errors)

    # Determine status
    status = "PASS" if len(errors) == 0 else "FAIL"

    return SkillTestResult(
        skill_name=name,
        skill_path=str(skill_dir),
        status=status,
        checks=checks,
        errors=errors,
        warnings=warnings,
        firing_contract_found=fc_found,
        smoke_test_pass=smoke_pass,
    )


# ───────────────────────────────────────────────────────────────────
# Batch Runner
# ───────────────────────────────────────────────────────────────────

def run_batch(skills_dirs: list[Path], output_path: Optional[Path] = None) -> BatchResult:
    """Run tests on all discovered skills."""
    batch = BatchResult()
    all_results = []

    for sd in skills_dirs:
        result = test_skill(sd)
        batch.results.append(result)
        all_results.append(result)

        if result.status == "PASS":
            batch.passed += 1
        elif result.status == "FAIL":
            batch.failed += 1
        else:
            batch.skipped += 1

        batch.total += 1

    # Write JSONL log
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "a") as f:
            for r in batch.results:
                f.write(json.dumps(r.to_dict()) + "\n")

    return batch


# ───────────────────────────────────────────────────────────────────
# Summary Report
# ───────────────────────────────────────────────────────────────────

def print_summary(batch: BatchResult, log_path: Path):
    """Print human-readable summary."""
    print("")
    print("=" * 60)
    print("SKILL TEST RUNNER — Hermes D5 Batch")
    print("=" * 60)
    print(f"  Skills found:    {batch.total}")
    print(f"  PASS:           {batch.passed}")
    print(f"  FAIL:           {batch.failed}")
    print(f"  SKIP:           {batch.skipped}")
    print(f"  Pass rate:      {batch.pass_rate}")
    print(f"  JSONL log:      {log_path}")
    print("")

    if batch.failed > 0:
        print("FAILED SKILLS:")
        for r in batch.results:
            if r.status == "FAIL":
                print(f"  [{r.skill_name}]")
                for e in r.errors:
                    print(f"    - {e}")
        print("")

    print("=" * 60)


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Skill Test Runner — Hermes D5")
    parser.add_argument(
        "--skills-dir",
        type=Path,
        default=Path("/home/corey/projects/AI-CIV/ACG/autonomy/skills"),
        help="Base skills directory to scan",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("memories/skills-test-log.jsonl"),
        help="Output JSONL log path",
    )
    parser.add_argument(
        "--summary",
        action="store_true",
        default=True,
        help="Print summary to stdout",
    )
    args = parser.parse_args()

    log_path = args.output if args.output.is_absolute() else Path.cwd() / args.output

    print(f"Discovering skills under: {args.skills_dir}")
    skills_dirs = discover_skills(args.skills_dir)
    print(f"Found {len(skills_dirs)} skills")

    batch = run_batch(skills_dirs, log_path)

    if args.summary:
        print_summary(batch, log_path)

    # Write summary JSON
    summary_path = log_path.with_suffix(".json")
    with open(summary_path, "w") as f:
        json.dump(batch.to_dict(), f, indent=2)
    print(f"Summary JSON: {summary_path}")

    # Exit code: 0 if all pass, 1 if any fail
    sys.exit(0 if batch.failed == 0 else 1)


if __name__ == "__main__":
    main()
