#!/usr/bin/env python3
"""Smoke test for session-summarizer."""

import subprocess
import sys
from pathlib import Path

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_snapshot():
    rc, out, err = run([
        "python3", "skills/session-summarizer/session_summarizer.py",
        "snapshot",
        "--session-id", "test-session-001",
        "--civ", "hengshi",
        "--tasks", "test task",
        "--decisions", "test decision",
        "--next-steps", "test next",
        "--tools", "bash python3",
        "--output", "/tmp/ss-smoke.jsonl",
    ])
    if rc != 0:
        print(f"snapshot FAILED: {err}")
        return False
    if not Path("/tmp/ss-smoke.jsonl").exists():
        print("snapshot: no output file")
        return False
    print("  snapshot: PASS")
    return True

def test_analyze():
    rc, out, err = run([
        "python3", "skills/session-summarizer/session_summarizer.py",
        "analyze",
        "--log", "/tmp/ss-smoke.jsonl",
    ])
    if rc != 0:
        print(f"analyze FAILED: {err}")
        return False
    print("  analyze: PASS")
    return True

def main():
    print("SESSION SUMMARIZER — Smoke Test")
    results = [("snapshot", test_snapshot()), ("analyze", test_analyze())]
    print("\nRESULTS:")
    all_ok = True
    for name, ok in results:
        print(f"  {name}: {'PASS' if ok else 'FAIL'}")
        if not ok:
            all_ok = False
    sys.exit(0 if all_ok else 1)

if __name__ == "__main__":
    main()
