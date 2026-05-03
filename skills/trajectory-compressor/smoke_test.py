#!/usr/bin/env python3
"""Smoke test for trajectory-compressor."""

import subprocess, sys, json
from pathlib import Path

COMPRESSOR_DIR = Path(__file__).resolve().parent

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_compress():
    """Test compress command."""
    # Create a proper JSONL test input (one JSON object per line)
    test_input = "/tmp/tc-input.jsonl"
    test_output = "/tmp/tc-output.jsonl"
    with open(test_input, "w") as f:
        for turn in [
            {"role": "user", "content": "Hello world", "token_count": 2},
            {"role": "assistant", "content": "Hi there", "token_count": 2},
            {"role": "user", "content": "How are you", "token_count": 3},
            {"role": "assistant", "content": "I'm good thanks", "token_count": 4},
            {"role": "user", "content": "Tell me something interesting about AI", "token_count": 8},
            {"role": "assistant", "content": "AI is transforming many fields", "token_count": 7},
            {"role": "user", "content": "What is GRPO", "token_count": 3},
            {"role": "assistant", "content": "GRPO is Group Relative Policy Optimization", "token_count": 8},
        ]:
            f.write(json.dumps(turn) + "\n")

    rc, out, err = run([
        "python3", str(COMPRESSOR_DIR / "trajectory_compressor.py"),
        "compress", test_input, test_output
    ])
    if rc != 0:
        print(f"FAIL: compress exited {rc}: {err[:200]}")
        return False
    if not Path(test_output).exists():
        print("FAIL: no output file")
        return False
    print("  compress: PASS")
    return True

def test_unit_tests():
    """Run unit tests."""
    rc, out, err = run(["python3", str(COMPRESSOR_DIR / "test_trajectory_compressor.py")])
    if rc != 0:
        print(f"FAIL: unit tests failed")
        return False
    if "ALL TESTS PASSED" not in out:
        print(f"FAIL: expected test pass marker not found")
        return False
    print("  unit tests: PASS")
    return True

def main():
    print("TRAJECTORY COMPRESSOR — Smoke Test")
    results = [
        ("compress", test_compress()),
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
