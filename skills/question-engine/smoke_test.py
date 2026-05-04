#!/usr/bin/env python3
"""Smoke test for question-engine skill."""

import subprocess, sys, os
from pathlib import Path

ENGINE_DIR = Path(__file__).resolve().parent

def run(cmd, env=None):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_module_import():
    """Test that question_engine module imports cleanly."""
    code = f"import sys; sys.path.insert(0, '{ENGINE_DIR}'); from question_engine import generate_question, score_response, ResponseScore, InterviewQuestion, ScoredResponse, QuestionEngineError; print('OK')"
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: import failed: {err}")
        return False
    print("  module import: PASS")
    return True

def test_cli_help():
    """Test CLI main help."""
    rc, out, err = run(["python3", str(ENGINE_DIR / "question_engine.py"), "--help"])
    if rc != 0:
        print(f"FAIL: --help failed: {err}")
        return False
    if "generate" not in out or "score" not in out:
        print(f"FAIL: expected subcommands in help: {out}")
        return False
    print("  CLI help: PASS")
    return True

def test_generate_help():
    """Test generate subcommand help."""
    rc, out, err = run(["python3", str(ENGINE_DIR / "question_engine.py"), "generate", "--help"])
    if rc != 0:
        print(f"FAIL: generate --help failed: {err}")
        return False
    if "--category" not in out or "--context" not in out:
        print(f"FAIL: expected arguments in generate help: {out}")
        return False
    print("  generate --help: PASS")
    return True

def test_score_help():
    """Test score subcommand help."""
    rc, out, err = run(["python3", str(ENGINE_DIR / "question_engine.py"), "score", "--help"])
    if rc != 0:
        print(f"FAIL: score --help failed: {err}")
        return False
    if "--response" not in out or "--question" not in out:
        print(f"FAIL: expected arguments in score help: {out}")
        return False
    print("  score --help: PASS")
    return True

def test_response_score_dataclass():
    """Test ResponseScore grade calculation."""
    code = (
        f"import sys; sys.path.insert(0, '{ENGINE_DIR}'); "
        "from question_engine import ResponseScore; "
        "s = ResponseScore(3, 3, 3, 3, 3, 3); "
        "assert s.total == 18, f'total={s.total}'; "
        "assert s.grade == 'A', f'grade={s.grade}'; "
        "s2 = ResponseScore(1, 1, 1, 1, 1, 1); "
        "assert s2.total == 6, f'total={s2.total}'; "
        "assert s2.grade == 'D', f'grade={s2.grade}'; "
        "print('OK')"
    )
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: ResponseScore dataclass: {err}")
        return False
    print("  ResponseScore grade calc: PASS")
    return True

def test_llm_generate():
    """Integration test: real LLM call to local Ollama."""
    script = ENGINE_DIR / "_llm_test.py"
    script.write_text(f"""
import sys
sys.path.insert(0, '{ENGINE_DIR}')
from question_engine import generate_question, QuestionEngineError
try:
    q = generate_question('childhood_memory', 'Born 1942 in rural Ohio. Raised by grandparents.')
    assert q.question, 'question is empty'
    assert q.category == 'childhood_memory', 'category mismatch: ' + q.category
    assert q.rationale, 'rationale is empty'
    print('OK:' + q.question[:60])
except QuestionEngineError as e:
    print('FAIL:' + str(e))
    sys.exit(1)
""")
    rc, out, err = run(["python3", str(script)])
    script.unlink()
    if rc != 0 or "OK:" not in out:
        print(f"FAIL: LLM generate: {err}")
        return False
    print(f"  LLM generate (local Ollama): PASS ({out.split('OK:')[1].strip()[:60]})")
    return True

def main():
    print("QUESTION ENGINE — Smoke Test")
    results = [
        ("module import", test_module_import()),
        ("CLI help", test_cli_help()),
        ("generate --help", test_generate_help()),
        ("score --help", test_score_help()),
        ("ResponseScore dataclass", test_response_score_dataclass()),
        ("LLM generate (local Ollama)", test_llm_generate()),
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
