#!/usr/bin/env python3
"""Tests for hub-watcher skill."""

import subprocess
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent


def run_cmd(cmd: list[str]) -> tuple[int, str, str]:
    import os
    env = dict(os.environ)
    env["TRIAD_CIV_ID"] = "hengshi"
    env["TRIAD_KEYPAIR_FILE"] = str(PROJECT_ROOT / ".aiciv" / "keys" / "hengshi-private.pem")
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT,
        env={**os.environ, "TRIAD_CIV_ID": "hengshi", "TRIAD_KEYPAIR_FILE": str(PROJECT_ROOT / ".aiciv" / "keys" / "hengshi-private.pem")},
    )
    return result.returncode, result.stdout, result.stderr


def test_check_events():
    exit_code, stdout, stderr = run_cmd(["python3", "skills/hub-watcher/hub_watcher.py", "check-events", "--limit", "5"])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "pending events" in stdout.lower() or "no pending" in stdout.lower(), f"Unexpected output: {stdout}"
    print(f"✅ test_check_events PASS")
    return True


def test_list_rooms():
    exit_code, stdout, stderr = run_cmd(["python3", "skills/hub-watcher/hub_watcher.py", "list-rooms", "--group", "aiciv-federation"])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "introductions" in stdout, f"Should show introductions room: {stdout}"
    assert "skills-library" in stdout, f"Should show skills-library room: {stdout}"
    print(f"✅ test_list_rooms PASS")
    return True


def test_room_activity():
    # Test with coordination room ID
    room_id = "7e5a87fe-6054-470a-847b-eb5fb3bdd441"
    exit_code, stdout, stderr = run_cmd(["python3", "skills/hub-watcher/hub_watcher.py", "room-activity", "--room", room_id, "--limit", "3"])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    # Should get either activity or "No recent activity" (both are valid responses)
    print(f"✅ test_room_activity PASS: {stdout[:100]}")
    return True


if __name__ == "__main__":
    print("=" * 60)
    print("HUB-WATCHER — Unit Tests")
    print("=" * 60)

    results = []
    tests = [
        ("check_events", test_check_events),
        ("list_rooms", test_list_rooms),
        ("room_activity", test_room_activity),
    ]

    for name, test_fn in tests:
        try:
            ok = test_fn()
            results.append((name, ok))
        except AssertionError as e:
            print(f"❌ {name} FAIL: {e}")
            results.append((name, False))
        except Exception as e:
            print(f"❌ {name} ERROR: {e}")
            results.append((name, False))

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    all_pass = True
    for name, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"  {name}: {status}")
        if not passed:
            all_pass = False

    print("\n" + "=" * 60)
    if all_pass:
        print("✅ ALL TESTS PASSED — Hub Watcher functional")
    else:
        print("❌ SOME TESTS FAILED")
    print("=" * 60)

    import sys
    sys.exit(0 if all_pass else 1)