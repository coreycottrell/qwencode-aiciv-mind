#!/usr/bin/env python3
"""Smoke test for skill-self-improver."""

import subprocess, sys, os, tempfile
from pathlib import Path

SELF_IMPROVER_DIR = Path(__file__).resolve().parent
REPO_ROOT = SELF_IMPROVER_DIR.parent.parent

def run(cmd, env=None):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_rubric_score():
    """Test rubric-score on skill-curator (a known skill on this system)."""
    rc, out, err = run([
        "python3", str(SELF_IMPROVER_DIR / "skill_self_improver.py"),
        "rubric-score", str(REPO_ROOT / "skills" / "skill-curator")
    ])
    if rc != 0:
        print(f"FAIL: rubric-score exited {rc}: {err}")
        return False
    if "TOTAL" not in out or "Grade:" not in out:
        print(f"FAIL: expected rubric output not found: {out[-200:]}")
        return False
    print("  rubric-score: PASS")
    return True

def test_improve():
    """Test improve command on curator grade log (limit 3)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        rc, out, err = run([
            "python3", str(SELF_IMPROVER_DIR / "skill_self_improver.py"),
            "improve",
            "--log", str(REPO_ROOT / "memories" / "skill-curator-grades-ACG-v02.jsonl"),
            "--output", tmpdir,
            "--limit", "3",
        ])
        if rc != 0:
            print(f"FAIL: improve exited {rc}: {err}")
            return False
        if "v_next" not in out and "Wrote" not in out:
            print(f"FAIL: expected 'Wrote N v_next' output not found: {out[-200:]}")
            return False

        vnext_files = list(Path(tmpdir).glob("*.vnext.md"))
        if len(vnext_files) == 0:
            print(f"FAIL: no v_next files written to {tmpdir}")
            return False

        # Check at least one v_next has the expected markers
        content = vnext_files[0].read_text()
        required = ["Skill Improvement Proposal", "Rubric Breakdown",
                    "Suggested Improvements", "F ("]
        missing = [m for m in required if m not in content]
        if missing:
            print(f"FAIL: v_next missing markers {missing}: {content[:200]}")
            return False

        print(f"  improve: PASS ({len(vnext_files)} v_next files)")
        return True

def test_suggest():
    """Test suggest command."""
    rc, out, err = run([
        "python3", str(SELF_IMPROVER_DIR / "skill_self_improver.py"),
        "suggest",
        "--log", str(REPO_ROOT / "memories" / "skill-curator-grades-ACG-v02.jsonl"),
        "--limit", "5",
    ])
    if rc != 0:
        print(f"FAIL: suggest exited {rc}: {err}")
        return False
    if "SKILL SELF-IMPROVER" not in out or "Suggestions" not in out:
        print(f"FAIL: expected suggest output not found")
        return False
    print("  suggest: PASS")
    return True

def main():
    print("SKILL SELF-IMPROVER — Smoke Test")
    results = [
        ("rubric-score", test_rubric_score()),
        ("improve", test_improve()),
        ("suggest", test_suggest()),
    ]
    print("\nRESULTS:")
    all_ok = True
    for name, ok in results:
        print(f"  {name}: {'PASS' if ok else 'FAIL'}")
        if not ok:
            all_ok = False
    sys.exit(0 if all_ok else 1)

if __name__ == "__main__":
    main()
