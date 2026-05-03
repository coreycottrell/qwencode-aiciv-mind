#!/usr/bin/env python3
"""Smoke test for compute-hibernation-tracker."""

import subprocess, sys, os, tempfile
from pathlib import Path

TRACKER_DIR = Path(__file__).resolve().parent

def run(cmd, env=None):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_session_lifecycle():
    """Test session_start -> ping -> session_end -> analyze."""
    # Start a session
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "session_start", "--civ", "hengshi", "--session-id", "smoke-test-session"
    ])
    if rc != 0:
        print(f"FAIL: session_start failed: {err}")
        return False

    # Ping active
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "ping", "--active", "--civ", "hengshi"
    ])
    if rc != 0:
        print(f"FAIL: ping --active failed: {err}")
        return False

    # Ping idle
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "ping", "--civ", "hengshi"
    ])
    if rc != 0:
        print(f"FAIL: ping (idle) failed: {err}")
        return False

    # End session
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "session_end", "--civ", "hengshi"
    ])
    if rc != 0:
        print(f"FAIL: session_end failed: {err}")
        return False

    if "tool_calls" not in out and "Session ended" not in out:
        print(f"FAIL: session_end output unexpected: {out}")
        return False

    print("  session lifecycle: PASS")
    return True

def test_analyze():
    """Test analyze command with existing log."""
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "analyze", "--civ", "hengshi"
    ])
    if rc != 0:
        print(f"FAIL: analyze failed: {err}")
        return False
    if "COMPUTE HIBERNATION TRACKER" not in out:
        print(f"FAIL: analyze output unexpected: {out}")
        return False
    print("  analyze: PASS")
    return True

def test_hibernate_candidates():
    """Test hibernate_candidates command."""
    rc, out, err = run([
        "python3", str(TRACKER_DIR / "compute_hibernation_tracker.py"),
        "hibernate_candidates", "--civ", "hengshi"
    ])
    if rc != 0:
        print(f"FAIL: hibernate_candidates failed: {err}")
        return False
    print("  hibernate_candidates: PASS")
    return True

def main():
    print("COMPUTE HIBERNATION TRACKER — Smoke Test")
    results = [
        ("session lifecycle", test_session_lifecycle()),
        ("analyze", test_analyze()),
        ("hibernate_candidates", test_hibernate_candidates()),
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
