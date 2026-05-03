#!/usr/bin/env python3
"""
Trajectory Compressor — Unit Tests

Tests compression logic with synthetic session data.

Run:
    python3 skills/trajectory-compressor/test_trajectory_compressor.py
"""

import sys
import json
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from trajectory_compressor import (
    Turn, compress_trajectory, CompressedSession,
    load_session, save_compressed_with_metadata, stats,
    PROTECT_FIRST_N, PROTECT_LAST_N, MAX_TRAJECTORY_TOKENS,
)


def make_turn(role: str, content: str) -> Turn:
    return Turn(role=role, content=content)


def test_turn_token_count():
    """Verify token count approximation."""
    t = make_turn("human", "hello world")
    assert t.token_count() == 2, f"Expected 2, got {t.token_count()}"
    t2 = make_turn("assistant", "one two three four five")
    assert t2.token_count() == 5
    print("✅ test_turn_token_count PASS")


def test_under_budget_unchanged():
    """Under-budget sessions returned unchanged."""
    turns = [
        make_turn("system", "You are a helpful assistant."),
        make_turn("human", "Hello"),
        make_turn("assistant", "Hi there!"),
    ]
    # Force low budget
    compressed = compress_trajectory(turns, protect_first=1, protect_last=1, max_tokens=5000)
    assert compressed.compression_ratio() == 0.0
    assert compressed.original_turns == 3
    assert compressed.compressed_turns == 3
    assert compressed.summary_turns == 0
    print("✅ test_under_budget_unchanged PASS")


def test_over_budget_compressed():
    """Over-budget session compressed."""
    # Build a session that exceeds budget
    middle_turns = [
        make_turn("assistant", "Attempt " + str(i) + ": trying a different approach " + "word " * 50)
        for i in range(10)
    ]
    turns = [
        make_turn("system", "You are a helpful assistant."),
        make_turn("human", "Please solve this problem step by step."),
    ] + middle_turns + [
        make_turn("assistant", "Final answer: the solution is 42."),
        make_turn("human", "Thanks!"),
    ]

    compressed = compress_trajectory(
        turns,
        protect_first=2,
        protect_last=2,
        max_tokens=100,  # Force compression
    )

    assert compressed.compression_ratio() > 0, "Should compress something"
    assert compressed.original_turns == 14
    assert compressed.compressed_turns < compressed.original_turns
    assert compressed.summary_turns >= 10, f"Should compress 10 middle turns, got {compressed.summary_turns}"
    assert compressed.protected_first == 2
    assert compressed.protected_last == 2

    # Check summary content
    summary_turn = compressed.turns[2]  # After first 2 protected
    assert "[COMPRESSED" in summary_turn.content
    assert "→ 1 summary" in summary_turn.content
    print(f"✅ test_over_budget_compressed PASS: {compressed.original_turns}→{compressed.compressed_turns} turns ({compressed.compression_ratio():.1%})")


def test_compression_ratio():
    """Verify compression ratio calculation."""
    compressed = CompressedSession(
        original_turns=10,
        compressed_turns=3,
        original_tokens=1000,
        compressed_tokens=300,
        summary_turns=7,
        protected_first=2,
        protected_last=1,
        turns=[],
    )
    assert abs(compressed.compression_ratio() - 0.7) < 0.001
    print("✅ test_compression_ratio PASS")


def test_empty_middle():
    """Middle turn gets compressed to summary when over budget."""
    turns = [
        make_turn("system", "You are a helpful assistant."),
        make_turn("human", "word " * 200),  # 200 words, forces over budget
        make_turn("assistant", "Short answer."),
    ]
    compressed = compress_trajectory(
        turns,
        protect_first=1,
        protect_last=1,
        max_tokens=50,  # Force compression of middle 200-word turn
    )
    # With first=1, last=1 on 3 turns: middle is 1 turn → compressed to 1 summary
    assert compressed.summary_turns == 1, f"Expected 1, got {compressed.summary_turns}"
    assert compressed.compressed_turns == 3  # first + summary + last
    print("✅ test_empty_middle PASS")


def test_jsonl_roundtrip():
    """Compressed session survives JSONL serialize/deserialize."""
    turns = [
        make_turn("system", "You are a helpful assistant."),
        make_turn("human", "Hello"),
        make_turn("assistant", "Hi!"),
    ]

    with tempfile.NamedTemporaryFile(suffix=".jsonl", delete=False) as f:
        input_path = f.name

    with tempfile.NamedTemporaryFile(suffix=".jsonl", delete=False) as f:
        output_path = f.name

    try:
        # Write turns as JSONL
        with open(input_path, "w") as f:
            for turn in turns:
                f.write(json.dumps(turn.to_dict()) + "\n")

        # Load and compress
        loaded = load_session(input_path)
        compressed = compress_trajectory(loaded, max_tokens=5000)

        # Save
        save_compressed_with_metadata(output_path, compressed, {"source": input_path})

        # Verify output has N+1 lines (N turns + 1 metadata)
        with open(output_path) as f:
            lines = [json.loads(l) for l in f if l.strip()]
        assert len(lines) == compressed.compressed_turns + 1, f"Expected {compressed.compressed_turns+1} lines"

        # Last line should be metadata
        assert lines[-1].get("_compression_meta") == True

        print(f"✅ test_jsonl_roundtrip PASS: {len(lines)} lines written")
    finally:
        Path(input_path).unlink(missing_ok=True)
        Path(output_path).unlink(missing_ok=True)


def test_stats():
    """Stats command works on synthetic file."""
    turns = [make_turn("human", "word " * 100) for _ in range(5)]
    with tempfile.NamedTemporaryFile(suffix=".jsonl", delete=False, mode="w") as f:
        path = f.name
        for turn in turns:
            f.write(json.dumps(turn.to_dict()) + "\n")

    try:
        s = stats(path)
        assert s["turns"] == 5
        assert s["tokens"] == 500  # 5 * 100
        assert s["over_budget"] == False  # 500 << 15250
        print(f"✅ test_stats PASS: {s}")
    finally:
        Path(path).unlink(missing_ok=True)


if __name__ == "__main__":
    print("=" * 60)
    print("TRAJECTORY COMPRESSOR — Unit Tests")
    print("=" * 60)

    results = []
    results.append(("token_count", True))
    results.append(("under_budget", True))
    results.append(("over_budget", True))
    results.append(("ratio", True))
    results.append(("empty_middle", True))
    results.append(("roundtrip", True))
    results.append(("stats", True))

    try:
        test_turn_token_count()
    except AssertionError as e:
        results[0] = ("token_count", False)
        print(f"❌ test_turn_token_count FAIL: {e}")

    try:
        test_under_budget_unchanged()
    except AssertionError as e:
        results[1] = ("under_budget", False)
        print(f"❌ test_under_budget_unchanged FAIL: {e}")

    try:
        test_over_budget_compressed()
    except AssertionError as e:
        results[2] = ("over_budget", False)
        print(f"❌ test_over_budget_compressed FAIL: {e}")

    try:
        test_compression_ratio()
    except AssertionError as e:
        results[3] = ("ratio", False)
        print(f"❌ test_compression_ratio FAIL: {e}")

    try:
        test_empty_middle()
    except AssertionError as e:
        results[4] = ("empty_middle", False)
        print(f"❌ test_empty_middle FAIL: {e}")

    try:
        test_jsonl_roundtrip()
    except AssertionError as e:
        results[5] = ("roundtrip", False)
        print(f"❌ test_jsonl_roundtrip FAIL: {e}")

    try:
        test_stats()
    except AssertionError as e:
        results[6] = ("stats", False)
        print(f"❌ test_stats FAIL: {e}")

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
        print("✅ ALL TESTS PASSED")
    else:
        print("❌ SOME TESTS FAILED")
    print("=" * 60)

    sys.exit(0 if all_pass else 1)
