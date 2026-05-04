#!/usr/bin/env python3
"""Smoke test for chapter-summarizer skill."""

import subprocess, sys, os, json, tempfile
from pathlib import Path

CHAPTER_DIR = Path(__file__).resolve().parent

def run(cmd, env=None):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    r = subprocess.run(cmd, capture_output=True, text=True, env=full_env)
    return r.returncode, r.stdout, r.stderr

def test_module_import():
    """Test that chapter_summarizer module imports cleanly."""
    code = f"import sys; sys.path.insert(0, '{CHAPTER_DIR}'); from chapter_summarizer import generate_chapter, load_interview_qas, ChapterDraft, ChapterSection, ChapterSummarizerError; print('OK')"
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: import failed: {err}")
        return False
    print("  module import: PASS")
    return True

def test_cli_help():
    """Test CLI main help."""
    rc, out, err = run(["python3", str(CHAPTER_DIR / "chapter_summarizer.py"), "--help"])
    if rc != 0:
        print(f"FAIL: --help failed: {err}")
        return False
    if "source" not in out:
        print(f"FAIL: expected 'source' in help: {out}")
        return False
    print("  CLI help: PASS")
    return True

def test_load_json():
    """Test loading Q&A from JSON format."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump([
            {"question": "What was the kitchen like?", "response": "Smelled of cinnamon and coffee.", "category": "childhood_memory", "score": 15},
            {"question": "Tell me about Sunday dinners.", "response": "Whole family gathered around the table.", "category": "family_tradition", "score": 12},
        ], f)
        tmp = f.name

    code = (
        f"import sys; sys.path.insert(0, '{CHAPTER_DIR}'); "
        f"from chapter_summarizer import load_interview_qas; "
        f"qa = load_interview_qas('{tmp}'); "
        f"assert len(qa) == 2, f'expected 2 got {{len(qa)}}'; "
        f"assert 'kitchen' in qa[0].question, 'wrong question'; "
        f"print('OK')"
    )
    rc, out, err = run(["python3", "-c", code])
    os.unlink(tmp)
    if rc != 0 or "OK" not in out:
        print(f"FAIL: load JSON: {err}")
        return False
    print("  load JSON: PASS")
    return True

def test_load_jsonl():
    """Test loading Q&A from JSONL format."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.jsonl', delete=False) as f:
        f.write('{"question": "What was school like?", "response": "One room, all grades together.", "category": "childhood_memory"}\n')
        f.write('{"question": "Who was your best friend?", "response": "Tommy Murphy. We fished every summer.", "category": "relationship_memory"}\n')
        tmp = f.name

    code = (
        f"import sys; sys.path.insert(0, '{CHAPTER_DIR}'); "
        f"from chapter_summarizer import load_interview_qas; "
        f"qa = load_interview_qas('{tmp}'); "
        f"assert len(qa) == 2, f'expected 2 got {{len(qa)}}'; "
        f"print('OK')"
    )
    rc, out, err = run(["python3", "-c", code])
    os.unlink(tmp)
    if rc != 0 or "OK" not in out:
        print(f"FAIL: load JSONL: {err}")
        return False
    print("  load JSONL: PASS")
    return True

def test_load_markdown():
    """Test loading Q&A from markdown format."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
        f.write("## Q: What was the house like?\n## A: Old farmhouse, creaky floors.\n\n## Q: Did you have siblings?\n## A: Two brothers and a sister.\n")
        tmp = f.name

    code = (
        f"import sys; sys.path.insert(0, '{CHAPTER_DIR}'); "
        f"from chapter_summarizer import load_interview_qas; "
        f"qa = load_interview_qas('{tmp}'); "
        f"assert len(qa) == 2, f'expected 2 got {{len(qa)}}'; "
        f"print('OK')"
    )
    rc, out, err = run(["python3", "-c", code])
    os.unlink(tmp)
    if rc != 0 or "OK" not in out:
        print(f"FAIL: load markdown: {err}")
        return False
    print("  load markdown: PASS")
    return True

def test_chapter_to_markdown():
    """Test ChapterDraft → markdown conversion."""
    code = (
        f"import sys; sys.path.insert(0, '{CHAPTER_DIR}'); "
        f"from chapter_summarizer import ChapterDraft, ChapterSection, chapter_to_markdown; "
        f"draft = ChapterDraft("
        f"  title='The Long Table', "
        f"  theme='Family Sunday dinners', "
        f"  narrative_arc='Every Sunday the family gathered.', "
        f"  sections=[ChapterSection(heading='The Gathering', content='Paragraph one.', key_quotes=['Quote one.'])], "
        f"  key_memories=['The smell of bread'], "
        f"  characters=['Grandma', 'Grandpa'], "
        f"  timeline_span='1952-1965', "
        f"  emotional_tone='Nostalgic', "
        f"  source_session_id='test-1', "
        f"  source_recording_count=3, "
        f"  confidence='high'"
        f"); "
        f"md = chapter_to_markdown(draft); "
        f"assert '# The Long Table' in md, 'missing title'; "
        f"assert 'Family Sunday dinners' in md, 'missing theme'; "
        f"assert 'The Gathering' in md, 'missing section heading'; "
        f"print('OK')"
    )
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: chapter_to_markdown: {err}")
        return False
    print("  chapter_to_markdown: PASS")
    return True

def main():
    print("CHAPTER SUMMARIZER — Smoke Test")
    results = [
        ("module import", test_module_import()),
        ("CLI help", test_cli_help()),
        ("load JSON", test_load_json()),
        ("load JSONL", test_load_jsonl()),
        ("load markdown", test_load_markdown()),
        ("chapter_to_markdown", test_chapter_to_markdown()),
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
