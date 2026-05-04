#!/usr/bin/env python3
"""
keptvoices_chat_server — TDD smoke test suite.
RED phase: write failing tests first, then implement to pass.

Tests cover:
1. Module imports cleanly (no circular deps, no missing imports)
2. handle_respond: first question generates context-aware question
3. handle_respond: follow-up passes prior_score
4. handle_finalize: returns chapter_preview with required fields
5. session JSONL roundtrip (append_qa → load_session)
6. summarize_storyteller_history: empty for new storyteller
7. health endpoint returns 200
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Paths
TOOLS_DIR = Path(__file__).resolve().parent
SERVER_MODULE = TOOLS_DIR / "keptvoices_chat_server.py"


class TestModuleImports(unittest.TestCase):
    """GREEN: Module imports without circular deps or missing imports."""

    def test_imports_cleanly(self):
        # Ensure we can import the server module
        result = subprocess.run(
            [sys.executable, "-c", f"import sys; sys.path.insert(0, '{TOOLS_DIR}'); import keptvoices_chat_server"],
            capture_output=True, text=True, timeout=10
        )
        self.assertEqual(result.returncode, 0, f"Import failed: {result.stderr}")


class TestHandleRespond(unittest.TestCase):
    """Test /chat/respond endpoint logic."""

    def setUp(self):
        # Use temp session dir
        self.temp_session = tempfile.mkdtemp()
        with patch.object(sys, "path", sys.path + [str(TOOLS_DIR)]):
            import keptvoices_chat_server as k
            k.SESSION_DIR = Path(self.temp_session)
            self.k = k

    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_session, ignore_errors=True)

    def test_first_question_returns_next_question(self):
        """First question: handle_respond returns next_question_text."""
        resp = self.k.handle_respond({
            "session_id": "tdd-test-001",
            "storyteller_id": "tdd-tester",
            "last_response": "",
            "question_count": 0,
            "category": "childhood_memory",
        })
        self.assertIn("next_question_text", resp)
        self.assertIsInstance(resp["next_question_text"], str)
        self.assertGreater(len(resp["next_question_text"]), 10)
        self.assertEqual(resp["question_count"], 1)
        self.assertEqual(resp["is_final"], False)

    def test_follow_up_returns_prior_score(self):
        """Follow-up: handle_respond returns prior_score from score_response."""
        resp = self.k.handle_respond({
            "session_id": "tdd-test-002",
            "storyteller_id": "tdd-tester",
            "last_response": "My grandmother made the best apple pie I ever tasted.",
            "question_count": 1,
            "category": "childhood_memory",
        })
        self.assertIn("next_question_text", resp)
        self.assertIn("prior_score", resp)
        # prior_score is int or None
        self.assertIsInstance(resp["prior_score"], (int, type(None)))

    def test_is_final_at_free_tier_max(self):
        """is_final=True when question_count + 1 >= FREE_TIER_MAX."""
        resp = self.k.handle_respond({
            "session_id": "tdd-test-003",
            "storyteller_id": "tdd-tester",
            "last_response": "",
            "question_count": 6,  # FREE_TIER_MAX=7, so 7th question is final
            "category": "childhood_memory",
        })
        self.assertEqual(resp["is_final"], True)


class TestHandleFinalize(unittest.TestCase):
    """Test /chat/finalize endpoint logic."""

    def setUp(self):
        self.temp_session = tempfile.mkdtemp()
        with patch.object(sys, "path", sys.path + [str(TOOLS_DIR)]):
            import keptvoices_chat_server as k
            k.SESSION_DIR = Path(self.temp_session)
            self.k = k

    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_session, ignore_errors=True)

    def test_finalize_returns_chapter_preview(self):
        """handle_finalize returns chapter_preview with required fields."""
        resp = self.k.handle_finalize({
            "session_id": "tdd-finalize-001",
            "all_qa_pairs": [
                {"question": "What is your earliest memory?",
                 "response": "Standing in my grandmother's kitchen."},
                {"question": "Describe that kitchen.",
                 "response": "Yellow curtains, the smell of cinnamon, morning light."},
            ]
        })
        self.assertIn("chapter_preview", resp)
        chapter = resp["chapter_preview"]
        required_fields = ["title", "body_markdown", "word_count", "key_quotes", "characters", "timeline_hint"]
        for field in required_fields:
            self.assertIn(field, chapter, f"Missing field: {field}")
        self.assertIsInstance(chapter["title"], str)
        self.assertGreater(len(chapter["title"]), 0)
        self.assertIsInstance(chapter["word_count"], int)
        self.assertGreater(chapter["word_count"], 0)


class TestSessionState(unittest.TestCase):
    """Test session JSONL roundtrip."""

    def setUp(self):
        self.temp_session = tempfile.mkdtemp()
        with patch.object(sys, "path", sys.path + [str(TOOLS_DIR)]):
            import keptvoices_chat_server as k
            k.SESSION_DIR = Path(self.temp_session)
            self.k = k

    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_session, ignore_errors=True)

    def test_append_qa_roundtrips(self):
        """append_qa + load_session preserves Q&A."""
        self.k.append_qa("session-roundtrip", "What is your name?", "Margaret.")
        records = self.k.load_session("session-roundtrip")
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["question"], "What is your name?")
        self.assertEqual(records[0]["response"], "Margaret.")

    def test_session_q_count(self):
        """session_q_count counts non-empty responses."""
        self.k.append_qa("session-count", "Q1", "A1")
        self.k.append_qa("session-count", "Q2", "")  # empty response
        self.k.append_qa("session-count", "Q3", "A3")
        count = self.k.session_q_count("session-count")
        self.assertEqual(count, 2)


class TestSummarizeStoryteller(unittest.TestCase):
    """Test storyteller history recovery."""

    def setUp(self):
        self.temp_session = tempfile.mkdtemp()
        with patch.object(sys, "path", sys.path + [str(TOOLS_DIR)]):
            import keptvoices_chat_server as k
            k.SESSION_DIR = Path(self.temp_session)
            self.k = k

    def tearDown(self):
        import shutil
        shutil.rmtree(self.temp_session, ignore_errors=True)

    def test_new_storyteller_returns_empty(self):
        """summarize_storyteller_history returns '' for no prior sessions."""
        result = self.k.summarize_storyteller_history("nonexistent-storyteller-xyz")
        self.assertEqual(result, "")


if __name__ == "__main__":
    unittest.main(verbosity=2)
