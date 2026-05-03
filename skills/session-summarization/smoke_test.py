#!/usr/bin/env python3
"""Smoke test for session-summarization."""

import subprocess, sys
from pathlib import Path

SUMMARIZATION_DIR = Path(__file__).resolve().parent

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_token_cap():
    """Run existing token cap enforcement test as smoke test."""
    rc, out, err = run(["python3", str(SUMMARIZATION_DIR / "test_token_cap.py")])
    if rc != 0:
        print(f"FAIL: test_token_cap.py exited {rc}: {err[:200]}")
        return False
    if "PROVEN" not in out:
        print(f"FAIL: expected PROVEN marker not found: {out[-200:]}")
        return False
    print("  token_cap: PASS")
    return True

def main():
    print("SESSION SUMMARIZATION — Smoke Test")
    results = [
        ("token_cap", test_token_cap()),
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
