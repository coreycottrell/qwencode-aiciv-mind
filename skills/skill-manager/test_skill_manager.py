#!/usr/bin/env python3
"""
Skill Manager — Unit Tests

Run:
    python3 skills/skill-manager/test_skill_manager.py
"""

import sys
import subprocess
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent


def run_cmd(cmd: list[str]) -> tuple[int, str, str]:
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT,
    )
    return result.returncode, result.stdout, result.stderr


def test_list_command():
    """list command prints tabular output."""
    exit_code, stdout, stderr = run_cmd(["python3", "skills/skill-manager/skill_manager.py", "list"])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "Name" in stdout, "Header row missing"
    assert "tdd" in stdout, "tdd should appear in list"
    assert "session-summarization" in stdout, "session-summarization should appear"
    assert "Status" in stdout or "PASS" in stdout or "PENDING" in stdout, "Status column missing"
    print(f"✅ test_list_command PASS: {len(stdout.splitlines())} lines output")
    return True


def test_verify_all_command():
    """verify-all command runs and reports summary."""
    exit_code, stdout, stderr = run_cmd(["python3", "skills/skill-manager/skill_manager.py", "verify-all"])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "Verifying all" in stdout, "Missing verification header"
    assert "Summary:" in stdout, "Missing summary line"
    assert "✅" in stdout or "⚠️" in stdout, "Missing pass/fail indicators"
    print(f"✅ test_verify_all_command PASS")
    return True


def test_check_pattern_no_overlap():
    """check with novel pattern finds no overlap."""
    exit_code, stdout, stderr = run_cmd([
        "python3", "skills/skill-manager/skill_manager.py", "check",
        "completely novel quantum blockchain AI pattern"
    ])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "No existing skill matches" in stdout, "Should find no match for novel pattern"
    print(f"✅ test_check_pattern_no_overlap PASS")
    return True


def test_check_pattern_overlap():
    """check with known pattern finds overlap."""
    exit_code, stdout, stderr = run_cmd([
        "python3", "skills/skill-manager/skill_manager.py", "check",
        "summarization"
    ])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    # Should find session-summarization as overlap
    assert "session-summarization" in stdout.lower(), "Should detect summarization overlap"
    print(f"✅ test_check_pattern_overlap PASS")
    return True


def test_check_file_complete_skill():
    """check-file on complete skill returns clean verification."""
    exit_code, stdout, stderr = run_cmd([
        "python3", "skills/skill-manager/skill_manager.py", "check-file",
        "tdd"
    ])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "tdd" in stdout, "Should mention tdd"
    # Should show SKILL.md ✅ and FIRING_CONTRACT.md ✅
    assert "SKILL.md" in stdout, "Should check SKILL.md"
    print(f"✅ test_check_file_complete_skill PASS")
    return True


def test_check_file_nonexistent():
    """check-file on nonexistent skill handles gracefully."""
    exit_code, stdout, stderr = run_cmd([
        "python3", "skills/skill-manager/skill_manager.py", "check-file",
        "nonexistent-skill-xyz"
    ])
    assert exit_code == 0, f"Expected exit 0, got {exit_code}: {stderr}"
    assert "does not exist" in stdout, "Should report nonexistent"
    print(f"✅ test_check_file_nonexistent PASS")
    return True


def test_scan_skills_dir_import():
    """scan_skills_dir function works as API."""
    sys.path.insert(0, str(PROJECT_ROOT / "skills" / "skill-manager"))
    from skill_manager import scan_skills_dir
    skills = scan_skills_dir()
    assert len(skills) >= 5, f"Expected at least 5 skills, got {len(skills)}"
    names = [s.name for s in skills]
    assert "tdd" in names, "tdd should be in registry"
    assert "session-summarization" in names, "session-summarization should be in registry"
    assert "trajectory-compressor" in names, "trajectory-compressor should be in registry"
    print(f"✅ test_scan_skills_dir_import PASS: {len(skills)} skills in registry")
    return True


if __name__ == "__main__":
    print("=" * 60)
    print("SKILL MANAGER — Unit Tests")
    print("=" * 60)

    results = []

    tests = [
        ("list_command", test_list_command),
        ("verify_all", test_verify_all_command),
        ("check_no_overlap", test_check_pattern_no_overlap),
        ("check_overlap", test_check_pattern_overlap),
        ("check_file_complete", test_check_file_complete_skill),
        ("check_file_missing", test_check_file_nonexistent),
        ("import_scan", test_scan_skills_dir_import),
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
        print("✅ ALL TESTS PASSED — Skill Manager functional")
    else:
        print("❌ SOME TESTS FAILED")
    print("=" * 60)

    sys.exit(0 if all_pass else 1)
