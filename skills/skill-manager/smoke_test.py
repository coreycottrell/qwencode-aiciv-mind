#!/usr/bin/env python3
"""Smoke test for skill-manager."""

import subprocess, sys
from pathlib import Path

MANAGER_DIR = Path(__file__).resolve().parent

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_list():
    """Test list command — shows known skills."""
    rc, out, err = run(["python3", str(MANAGER_DIR / "skill_manager.py"), "list"])
    if rc != 0:
        print(f"FAIL: list exited {rc}: {err[:200]}")
        return False
    if "tdd" not in out.lower() and "skill" not in out.lower():
        print(f"FAIL: expected skill list output not found: {out[:200]}")
        return False
    print("  list: PASS")
    return True

def test_verify_all():
    """Test verify-all command — basic file checks for all skills."""
    rc, out, err = run(["python3", str(MANAGER_DIR / "skill_manager.py"), "verify-all"])
    if rc != 0:
        print(f"FAIL: verify-all exited {rc}: {err[:200]}")
        return False
    if "Verifying" not in out and "Summary" not in out:
        print(f"FAIL: expected verify-all output not found: {out[:200]}")
        return False
    print("  verify-all: PASS")
    return True

def test_check_file():
    """Test check-file command on skill-curator."""
    rc, out, err = run([
        "python3", str(MANAGER_DIR / "skill_manager.py"),
        "check-file", "tdd"
    ])
    if rc != 0:
        print(f"FAIL: check-file exited {rc}: {err[:200]}")
        return False
    if "tdd" not in out.lower():
        print(f"FAIL: expected check-file output not found: {out[:200]}")
        return False
    print("  check-file: PASS")
    return True

def main():
    print("SKILL MANAGER — Smoke Test")
    results = [
        ("list", test_list()),
        ("verify-all", test_verify_all()),
        ("check-file", test_check_file()),
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
