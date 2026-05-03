#!/usr/bin/env python3
"""
Smoke test for skill-curator.
Verifies: grade, generate_fc (dry-run), analyze commands work end-to-end.
"""

import json
import subprocess
import sys
from pathlib import Path

def run(cmd: list[str]) -> tuple[int, str, str]:
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_grade_command() -> bool:
    """Test grade discovers skills and emits JSONL."""
    rc, out, err = run([
        "python3", "skills/skill-curator/skill_curator.py",
        "grade",
        "--skills-dir", "skills/skill-curator",
        "--output", "/tmp/curator-smoke-grades.jsonl",
        "--civ", "hengshi",
    ])
    if rc != 0:
        print(f"FAIL: grade command exited {rc}: {err}")
        return False

    # Should find the curator's own skill directory (skill-curator has SKILL.md + FC)
    path = Path("/tmp/curator-smoke-grades.jsonl")
    if not path.exists():
        print("FAIL: no JSONL output produced")
        return False

    grades = []
    with open(path) as f:
        for line in f:
            grades.append(json.loads(line))

    if not grades:
        print("FAIL: no grades produced")
        return False

    # skill-curator itself should PASS (has SKILL.md + FC)
    passed = [g for g in grades if g["grade"] == "PASS"]
    if not passed:
        print(f"FAIL: Curator itself did not PASS: {grades}")
        return False

    print(f"  grade: PASS ({len(grades)} skills found, {len(passed)} passed)")
    return True

def test_generate_fc_dry_run() -> bool:
    """Test generate_fc --dry-run does not write files."""
    rc, out, err = run([
        "python3", "skills/skill-curator/skill_curator.py",
        "generate_fc",
        "--skills-dir", "skills/skill-curator",
        "--civ", "hengshi",
        "--dry-run",
    ])
    if rc != 0:
        print(f"FAIL: generate_fc dry-run exited {rc}: {err}")
        return False
    print("  generate_fc --dry-run: PASS")
    return True

def test_analyze_empty() -> bool:
    """Test analyze with empty/invalid log."""
    rc, out, err = run([
        "python3", "skills/skill-curator/skill_curator.py",
        "analyze",
        "--log", "/tmp/curator-nonexistent.jsonl",
    ])
    # Should exit 1 for nonexistent log
    if rc == 0:
        print(f"FAIL: analyze should fail for nonexistent log")
        return False
    print("  analyze (empty log): PASS")
    return True

def main():
    print("=" * 60)
    print("SKILL CURATOR — Smoke Test")
    print("=" * 60)

    results = []
    results.append(("grade", test_grade_command()))
    results.append(("generate_fc --dry-run", test_generate_fc_dry_run()))
    results.append(("analyze (empty log)", test_analyze_empty()))

    print("")
    print("RESULTS:")
    all_pass = True
    for name, ok in results:
        print(f"  {name}: {'PASS' if ok else 'FAIL'}")
        if not ok:
            all_pass = False

    if all_pass:
        print("\nAll smoke tests PASSED")
        sys.exit(0)
    else:
        print("\nSome smoke tests FAILED")
        sys.exit(1)

if __name__ == "__main__":
    main()
