#!/usr/bin/env python3
"""Smoke test for web-chat-wrapper skill."""

import subprocess, sys, os, json, time, tempfile
from pathlib import Path
import urllib.request
import urllib.error

WRAPPER_DIR = Path(__file__).resolve().parent
SERVER_SCRIPT = WRAPPER_DIR / "server.py"

def run(cmd, env=None):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_module_import():
    """Test that question-engine imports within wrapper context."""
    code = f"import sys; sys.path.insert(0, '{WRAPPER_DIR}'); import server; print('OK')"
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: import failed: {err}")
        return False
    print("  module import: PASS")
    return True

def test_server_help():
    """Test server CLI help."""
    rc, out, err = run(["python3", str(SERVER_SCRIPT), "--help"])
    if rc != 0:
        print(f"FAIL: --help failed: {err}")
        return False
    print("  server --help: PASS")
    return True

def test_handle_respond():
    """Test handle_respond with mock data (no server needed)."""
    code = (
        f"import sys; sys.path.insert(0, '{WRAPPER_DIR}'); "
        f"import server, json; "
        f"data = {{'storyteller_id': 'test-001', 'category': 'childhood_memory', 'context': 'Born 1942 Ohio.', 'paid_tier': False}}; "
        f"result = server.handle_respond(data); "
        f"assert 'next_question' in result, f'missing next_question: {{result}}'; "
        f"assert result['storyteller_id'] == 'test-001'; "
        f"print('OK:' + result['next_question'][:50])"
    )
    rc, out, err = run(["python3", "-c", code], env={"PIPER_BIN": "/bin/false"})
    if rc != 0 or "OK:" not in out:
        print(f"FAIL: handle_respond: {err}")
        return False
    print(f"  handle_respond: PASS ({out.strip()[:50]})")
    return True

def test_server_start_stop():
    """Start server, call /health, stop."""
    env = dict(os.environ)
    env["PORT"] = "18765"

    proc = subprocess.Popen(
        ["python3", str(SERVER_SCRIPT), "--port", "18765"],
        stderr=subprocess.PIPE,
        stdout=subprocess.PIPE,
        env=env,
    )
    time.sleep(1.5)

    try:
        req = urllib.request.Request("http://localhost:18765/health")
        with urllib.request.urlopen(req, timeout=5) as resp:
            result = json.loads(resp.read())
        if result.get("status") != "ok":
            print(f"FAIL: health check failed: {result}")
            return False
        print("  /health check: PASS")
        ok = True
    except (urllib.error.URLError, urllib.error.HTTPError) as e:
        print(f"FAIL: server not responding: {e}")
        ok = False

    proc.terminate()
    proc.wait(timeout=5)
    return ok

def test_respond_endpoint():
    """Test POST /chat/respond against running server."""
    env = dict(os.environ)
    env["PORT"] = "18766"

    proc = subprocess.Popen(
        ["python3", str(SERVER_SCRIPT), "--port", "18766"],
        stderr=subprocess.PIPE,
        stdout=subprocess.PIPE,
        env=env,
    )
    time.sleep(1.5)

    try:
        payload = json.dumps({
            "storyteller_id": "test-002",
            "category": "childhood_memory",
            "context": "Born 1942 in rural Ohio.",
            "paid_tier": False,
        }).encode()
        req = urllib.request.Request(
            "http://localhost:18766/chat/respond",
            data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=15) as resp:
            result = json.loads(resp.read())
        if "next_question" not in result:
            print(f"FAIL: no next_question in response: {result}")
            return False
        if "error" in result and "generation failed" in result["error"]:
            print(f"FAIL: question generation error: {result}")
            return False
        print(f"  /chat/respond: PASS — '{result['next_question'][:50]}...'")
        ok = True
    except (urllib.error.URLError, urllib.error.HTTPError) as e:
        print(f"FAIL: /chat/respond failed: {e}")
        ok = False

    proc.terminate()
    proc.wait(timeout=5)
    return ok

def main():
    print("WEB-CHAT-WRAPPER — Smoke Test")
    results = [
        ("module import", test_module_import()),
        ("server --help", test_server_help()),
        ("handle_respond", test_handle_respond()),
        ("/health endpoint", test_server_start_stop()),
        ("/chat/respond endpoint", test_respond_endpoint()),
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
