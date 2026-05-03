#!/usr/bin/env python3
"""Smoke test for hub-watcher."""

import subprocess, sys, os
from pathlib import Path

KEYPAIR = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-private.pem"

HUB_WATCHER_DIR = Path(__file__).resolve().parent

def run(cmd, env=None):
    full_env = dict(os.environ)
    full_env["TRIAD_KEYPAIR_FILE"] = KEYPAIR
    full_env["TRIAD_CIV_ID"] = "hengshi"
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_check_events():
    rc, out, err = run(["python3", str(HUB_WATCHER_DIR / "hub_watcher.py"), "check-events"])
    if rc != 0:
        print(f"FAIL: check-events: {err[:200]}")
        return False
    print("  check-events: PASS")
    return True

def test_list_rooms():
    rc, out, err = run(["python3", str(HUB_WATCHER_DIR / "hub_watcher.py"), "list-rooms", "--group", "hengshi-acg-proof"])
    if rc != 0:
        print(f"FAIL: list-rooms: {err[:200]}")
        return False
    print("  list-rooms: PASS")
    return True

def test_unit_tests():
    rc, out, err = run(["python3", str(HUB_WATCHER_DIR / "test_hub_watcher.py")])
    if rc != 0:
        print(f"FAIL: unit tests failed")
        return False
    print("  unit tests: PASS")
    return True

def main():
    print("HUB-WATCHER — Smoke Test")
    results = [
        ("check-events", test_check_events()),
        ("list-rooms", test_list_rooms()),
        ("unit tests", test_unit_tests()),
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
