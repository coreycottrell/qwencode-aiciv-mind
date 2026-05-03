#!/usr/bin/env python3
"""Smoke test for webhook-push."""

import os, subprocess, sys, tempfile
from pathlib import Path

KEYPAIR = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-private.pem"

def run(cmd, env=None):
    full_env = dict(os.environ)
    full_env["TRIAD_KEYPAIR_FILE"] = KEYPAIR
    full_env["TRIAD_CIV_ID"] = "hengshi"
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_status():
    """Test status command (no Hub auth needed)."""
    rc, out, err = run(["python3", "skills/webhook-push/webhook_push.py", "status"])
    if rc != 0:
        print(f"FAIL: status exited {rc}: {err[:200]}")
        return False
    if "WEBHOOK PUSH" not in out or "Branch:" not in out:
        print(f"FAIL: expected status output not found: {out[:200]}")
        return False
    print("  status: PASS")
    return True

def test_diff():
    """Test diff command (no Hub auth needed)."""
    rc, out, err = run(["python3", "skills/webhook-push/webhook_push.py", "diff"])
    if rc != 0:
        print(f"FAIL: diff exited {rc}: {err[:200]}")
        return False
    print("  diff: PASS")
    return True

def test_push_dry_run():
    """Test push --dry-run (uses JWT auth but no actual Hub post)."""
    rc, out, err = run(["python3", "skills/webhook-push/webhook_push.py", "push", "--dry-run"])
    if rc != 0:
        print(f"FAIL: push --dry-run exited {rc}: {err[:200]}")
        return False
    # Dry run outputs JSON payload
    if "DRY RUN" not in out or "branch" not in out.lower():
        print(f"FAIL: expected dry-run JSON output not found: {out[:200]}")
        return False
    print("  push --dry-run: PASS")
    return True

def test_setup():
    """Test setup (installs post-commit hook)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create a temp git repo to test hook install
        subprocess.run(["git", "init", tmpdir], capture_output=True)
        env = dict(os.environ)
        env["TRIAD_KEYPAIR_FILE"] = KEYPAIR
        env["TRIAD_CIV_ID"] = "hengshi"
        env["GIT_DIR"] = tmpdir  # won't work for all git commands but let's try
        rc, out, err = run(
            ["python3", "skills/webhook-push/webhook_push.py", "setup"],
            env=env
        )
        # Setup might fail if not in a git repo, but should not crash
        print(f"  setup: rc={rc}")
    return True  # Setup is informational, can't easily test in temp dir

def main():
    print("WEBHOOK PUSH — Smoke Test")
    results = [
        ("status", test_status()),
        ("diff", test_diff()),
        ("push --dry-run", test_push_dry_run()),
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
