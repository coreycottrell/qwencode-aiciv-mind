#!/usr/bin/env python3
"""End-to-end integration test for aiciv-mcp binary.

Tests: spawn aiciv-mcp as subprocess, send JSON-RPC, verify response.
This simulates what an MCP client (Claude Code, Cursor, etc.) would do.
"""

import subprocess
import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent.parent  # workspace root
BINARY = PROJECT_ROOT / "target" / "release" / "aiciv-mcp"


def rpc(tool: str, args: dict, id=1) -> dict:
    """Send JSON-RPC request to aiciv-mcp, return parsed response."""
    payload = json.dumps({"tool": tool, "args": args, "id": id})
    result = subprocess.run(
        [str(BINARY)],
        input=payload + "\n",
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT,
    )
    return json.loads(result.stdout.strip())


def test_unknown_tool():
    resp = rpc("nonexistent_tool", {})
    assert resp.get("error") == "unknown tool", f"Expected error, got {resp}"
    print("✅ unknown tool → error")


def test_heartbeat_returns_raw_from_triadj_client():
    resp = rpc("hengshi_heartbeat", {})
    # Should get either an error (TRIAD_KEYPAIR_FILE missing) or raw output
    assert "error" in resp or "result" in resp, f"No error or result: {resp}"
    print(f"✅ heartbeat → {resp}")


def test_compress_trajectory_missing_arg():
    resp = rpc("hengshi_compress_trajectory", {"something": "value"})
    assert resp.get("error") is not None, f"Expected error, got {resp}"
    print("✅ compress_trajectory missing arg → error")


def test_summarize_session_missing_arg():
    resp = rpc("hengshi_summarize_session", {"not_session_ledger": "value"})
    assert resp.get("error") is not None, f"Expected error, got {resp}"
    print("✅ summarize_session missing arg → error")


def test_post_to_room_missing_message():
    resp = rpc("hengshi_post_to_room", {"room_id": "test-room"})
    assert resp.get("error") is not None, f"Expected error, got {resp}"
    print("✅ post_to_room missing message → error")


def test_poll_events_missing_agent_id():
    resp = rpc("hengshi_poll_events", {"limit": 5})
    assert resp.get("error") is not None, f"Expected error, got {resp}"
    print("✅ poll_events missing agent_id → error")


def test_tdd_cycle_missing_test_file():
    resp = rpc("hengshi_tdd_cycle", {"function_name": "test_add"})
    assert resp.get("error") is not None, f"Expected error, got {resp}"
    print("✅ tdd_cycle missing test_file → error")


if __name__ == "__main__":
    print("=" * 60)
    print("aiciv-mcp end-to-end integration test")
    print("=" * 60)
    print(f"Binary: {BINARY}")
    print(f"Exists: {BINARY.exists()}")
    print()

    if not BINARY.exists():
        print("⚠️  Release binary not found. Run: cargo build -p aiciv-mcp --release")
        sys.exit(1)

    tests = [
        test_unknown_tool,
        test_heartbeat_returns_raw_from_triadj_client,
        test_compress_trajectory_missing_arg,
        test_summarize_session_missing_arg,
        test_post_to_room_missing_message,
        test_poll_events_missing_agent_id,
        test_tdd_cycle_missing_test_file,
    ]

    passed = 0
    for t in tests:
        try:
            t()
            passed += 1
        except Exception as e:
            print(f"❌ {t.__name__}: {e}")

    print()
    print("=" * 60)
    print(f"Result: {passed}/{len(tests)} passed")
    if passed == len(tests):
        print("✅ ALL END-TO-END TESTS PASSED")
    else:
        print("❌ SOME TESTS FAILED")
    print("=" * 60)
    sys.exit(0 if passed == len(tests) else 1)