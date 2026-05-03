#!/usr/bin/env python3
"""Smoke test for tdd skill."""

import subprocess, sys
from pathlib import Path

TD_DIR = Path(__file__).resolve().parent

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_tdd_cycle():
    rc, out, err = run(["python3", str(TD_DIR / "test_tdd_cycle.py")])
    if rc != 0:
        print(f"FAIL: tdd cycle exited {rc}: {err}")
        return False
    if "✅ TDD CYCLE PROOF COMPLETE" not in out:
        print(f"FAIL: expected completion marker not found: {out[-200:]}")
        return False
    if "RED PHASE" not in out or "GREEN PHASE" not in out or "REFACTOR PHASE" not in out:
        print(f"FAIL: missing phase markers in output")
        return False
    print("  TDD cycle: PASS")
    return True

def main():
    print("TDD — Smoke Test")
    results = [
        ("test_tdd_cycle", test_tdd_cycle()),
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
