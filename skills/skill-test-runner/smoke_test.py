#!/usr/bin/env python3
"""Smoke test for skill-test-runner."""

import subprocess, sys, os, tempfile
from pathlib import Path

RUNNER_DIR = Path(__file__).resolve().parent
REPO_ROOT = RUNNER_DIR.parent.parent

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_runner_local():
    """Run skill-test-runner on local skills dir, verify 14/14 PASS."""
    with tempfile.TemporaryDirectory() as tmpdir:
        rc, out, err = run([
            "python3", str(RUNNER_DIR / "skill_test_runner.py"),
            "--skills-dir", str(REPO_ROOT / "skills"),
            "--output", f"{tmpdir}/test-log.jsonl",
        ])
        if rc != 0:
            print(f"FAIL: runner exited {rc}: {err[:200]}")
            return False
        if "PASS:" not in out or "FAIL:" not in out:
            print(f"FAIL: expected pass/fail counts not found: {out[-200:]}")
            return False
        # Local skills should have 14 PASS
        if "PASS:           14" not in out and "PASS:            14" not in out:
            print(f"FAIL: expected 14 PASS for local skills: {out}")
            return False
        print("  runner local: PASS")
        return True

def test_runner_output_format():
    """Verify runner produces valid JSONL output on skills dir."""
    with tempfile.TemporaryDirectory() as tmpdir:
        log_path = f"{tmpdir}/format-test.jsonl"
        rc, out, err = run([
            "python3", str(RUNNER_DIR / "skill_test_runner.py"),
            "--skills-dir", str(REPO_ROOT / "skills"),
            "--output", log_path,
        ])
        if not Path(log_path).exists():
            print(f"FAIL: JSONL log not created at {log_path}")
            return False
        content = Path(log_path).read_text()
        lines = [l for l in content.strip().split("\n") if l]
        if not lines:
            print("FAIL: JSONL log is empty")
            return False
        import json
        try:
            record = json.loads(lines[0])
            required = ["skill_name", "skill_path", "status", "checks"]
            missing = [k for k in required if k not in record]
            if missing:
                print(f"FAIL: JSONL record missing fields {missing}")
                return False
        except json.JSONDecodeError as e:
            print(f"FAIL: invalid JSON in log: {e}")
            return False
        print("  runner output format: PASS")
        return True

def main():
    print("SKILL TEST RUNNER — Smoke Test")
    results = [
        ("runner local", test_runner_local()),
        ("runner output format", test_runner_output_format()),
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
