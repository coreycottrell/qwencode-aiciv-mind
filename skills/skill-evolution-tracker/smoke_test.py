#!/usr/bin/env python3
"""Smoke test for skill-evolution-tracker."""

import subprocess, sys
from pathlib import Path

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_log():
    rc, out, err = run([
        "python3", "skills/skill-evolution-tracker/skill_evolution_tracker.py",
        "log", "tdd",
        "--civ", "hengshi",
        "--context", "smoke-test",
        "--outcome", "pass",
        "--log", "/tmp/set-smoke.jsonl",
    ])
    if rc != 0:
        print(f"FAIL: log failed: {err}")
        return False
    if not Path("/tmp/set-smoke.jsonl").exists():
        print("FAIL: log file not created")
        return False
    print("  log: PASS")
    return True

def test_analyze():
    rc, out, err = run([
        "python3", "skills/skill-evolution-tracker/skill_evolution_tracker.py",
        "analyze",
        "--log", "/tmp/set-smoke.jsonl",
    ])
    if rc != 0:
        print(f"FAIL: analyze failed: {err}")
        return False
    print("  analyze: PASS")
    return True

def test_signals():
    rc, out, err = run([
        "python3", "skills/skill-evolution-tracker/skill_evolution_tracker.py",
        "signals",
        "--log", "/tmp/set-smoke.jsonl",
    ])
    if rc != 0:
        print(f"FAIL: signals failed: {err}")
        return False
    print("  signals: PASS")
    return True

def main():
    print("SKILL EVOLUTION TRACKER — Smoke Test")
    results = [
        ("log", test_log()),
        ("analyze", test_analyze()),
        ("signals", test_signals()),
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
