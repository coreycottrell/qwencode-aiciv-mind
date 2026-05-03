#!/usr/bin/env python3
"""
TDD Cycle Proof Test

Proves the TDD skill works by running a live RED-GREEN-REFACTOR cycle
on a trivial example: a retry operation with exponential backoff.

Pattern:
1. RED: Write test that fails (retry with exponential backoff)
2. GREEN: Write minimal implementation
3. REFACTOR: Clean up (if needed)
4. Verify: All tests pass

Run:
    python3 skills/tdd/test_tdd_cycle.py
"""

import subprocess
import sys
from pathlib import Path

# Project root for imports
PROJECT_ROOT = Path(__file__).parent.parent.parent


def run_cmd(cmd: list[str], cwd: Path = None) -> tuple[int, str, str]:
    """Run a command, return (exit_code, stdout, stderr)."""
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=cwd or PROJECT_ROOT,
    )
    return result.returncode, result.stdout, result.stderr


def test_tdd_cycle_proof():
    """Live RED-GREEN-REFACTOR cycle proof.

    We test a retry_with_backoff function:
    - RED: test_retry_fails_before_implementation → FAIL
    - GREEN: implementation added → test passes
    - REFACTOR: no regressions
    """
    print("=" * 60)
    print("TDD CYCLE PROOF — retry_with_backoff example")
    print("=" * 60)

    # ── SETUP: Create test file in temp location ───────────────────────
    test_dir = PROJECT_ROOT / "skills" / "tdd" / "test_example"
    test_dir.mkdir(exist_ok=True)
    test_file = test_dir / "test_retry.py"
    impl_file = test_dir / "retry.py"

    # Clean up any prior run
    if test_file.exists():
        test_file.unlink()
    if impl_file.exists():
        impl_file.unlink()

    # ── RED PHASE ──────────────────────────────────────────────────────
    print("\n[RED PHASE] Writing failing test...")
    print("-" * 40)

    test_code = '''"""RED phase: This test MUST fail before we write the implementation."""

import pytest
import sys
sys.path.insert(0, __file__ + "/..")

def test_retry_succeeds_after_3_attempts():
    """Retry should succeed after 3 failed attempts."""
    from retry import retry_with_backoff

    attempts = []
    def failing_op():
        attempts.append(1)
        if len(attempts) < 3:
            raise RuntimeError(f"Attempt {len(attempts)} failed")
        return "success"

    result = retry_with_backoff(failing_op, max_attempts=3, base_delay=0.01)
    assert result == "success", f"Expected 'success', got {result!r}"
    assert len(attempts) == 3, f"Expected 3 attempts, got {len(attempts)}"

def test_retry_raises_after_max_attempts():
    """After max_attempts, should raise the exception."""
    from retry import retry_with_backoff

    attempts = []
    def always_fails():
        attempts.append(1)
        raise RuntimeError("Always fails")

    with pytest.raises(RuntimeError, match="Always fails"):
        retry_with_backoff(always_fails, max_attempts=3, base_delay=0.001)
    assert len(attempts) == 3
'''

    test_file.write_text(test_code)
    print(f"Test written to: {test_file}")

    # Write empty impl so import doesn't crash (but function doesn't exist)
    impl_file.write_text("# Placeholder — implementation not yet written\n")

    # Run RED test — MUST FAIL (function doesn't exist)
    exit_code, stdout, stderr = run_cmd(
        ["python3", "-m", "pytest", str(test_file), "-v", "--tb=short"]
    )

    print(f"\nRED phase result:")
    print(f"  Exit code: {exit_code}")
    print(f"  stdout: {stdout[:500]}")
    if stderr:
        print(f"  stderr: {stderr[:300]}")

    # RED phase is SUCCESSFUL if test fails (exit_code != 0)
    # We expect: FAILED (ImportError or AttributeError — function doesn't exist)
    if exit_code == 0:
        print("\n❌ RED FAIL: Test passed when it should have FAILED.")
        print("   This means the test is testing existing behavior, not new behavior.")
        print("   TDD violated: implementation was written before the test.")
        return False

    if "PASSED" in stdout and "FAILED" not in stdout:
        print("\n❌ RED FAIL: Test passed in RED phase.")
        print("   TDD violated.")
        return False

    print("\n✅ RED PHASE PASSED: Test fails as expected (function not implemented)")

    # ── GREEN PHASE ────────────────────────────────────────────────────
    print("\n[GREEN PHASE] Writing minimal implementation...")
    print("-" * 40)

    impl_code = '''"""GREEN phase: Minimal implementation to make tests pass."""

import time

def retry_with_backoff(operation, max_attempts=3, base_delay=0.01):
    """Retry an operation with exponential backoff.

    Args:
        operation: Callable to execute
        max_attempts: Maximum number of attempts
        base_delay: Base delay in seconds

    Returns:
        Result of operation

    Raises:
        Exception: If all attempts fail
    """
    last_exception = None
    for attempt in range(1, max_attempts + 1):
        try:
            return operation()
        except Exception as e:
            last_exception = e
            if attempt < max_attempts:
                delay = base_delay * (2 ** (attempt - 1))
                time.sleep(delay)
    raise last_exception
'''

    impl_file.write_text(impl_code)
    print(f"Implementation written to: {impl_file}")

    # Run GREEN test — MUST PASS
    exit_code, stdout, stderr = run_cmd(
        ["python3", "-m", "pytest", str(test_file), "-v", "--tb=short"]
    )

    print(f"\nGREEN phase result:")
    print(f"  Exit code: {exit_code}")
    print(f"  stdout: {stdout[:500]}")

    if exit_code != 0:
        print("\n❌ GREEN FAIL: Test failed after implementation.")
        print("   Fix the implementation, not the test.")
        print(f"   Output: {stdout}")
        print(f"   Error: {stderr}")
        return False

    if "FAILED" in stdout:
        print("\n❌ GREEN FAIL: Some tests failed.")
        return False

    print("\n✅ GREEN PHASE PASSED: All tests pass")

    # ── REFACTOR PHASE ─────────────────────────────────────────────────
    print("\n[REFACTOR PHASE] Verify no regressions...")
    print("-" * 40)

    # Import and verify the function works interactively
    sys.path.insert(0, str(test_dir))
    from retry import retry_with_backoff

    # Quick sanity check
    counter = [0]
    def succeed_on_2nd():
        counter[0] += 1
        if counter[0] < 2:
            raise RuntimeError(f"Attempt {counter[0]}")
        return "ok"

    result = retry_with_backoff(succeed_on_2nd, max_attempts=5, base_delay=0.001)
    assert result == "ok", f"Expected 'ok', got {result!r}"
    assert counter[0] == 2, f"Expected 2 attempts, got {counter[0]}"
    print("✅ REFACTOR PHASE PASSED: Function works correctly, no regressions")

    # ── CLEANUP ─────────────────────────────────────────────────────────
    print("\n[Cleanup] Removing test artifacts...")
    if test_file.exists():
        test_file.unlink()
    if impl_file.exists():
        impl_file.unlink()
    print("✅ Cleanup done")

    print("\n" + "=" * 60)
    print("✅ TDD CYCLE PROOF COMPLETE")
    print("   RED: test written → fails (function not implemented) ✅")
    print("   GREEN: implementation written → all tests pass ✅")
    print("   REFACTOR: function verified, no regressions ✅")
    print("=" * 60)
    return True


if __name__ == "__main__":
    success = test_tdd_cycle_proof()
    sys.exit(0 if success else 1)
