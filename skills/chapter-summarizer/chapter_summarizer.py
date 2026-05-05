#!/usr/bin/env python3
"""
Chapter Summarizer — Multi-recording session to chapter draft

Applies session-summarization pattern to multi-recording interview sessions.
Takes Q&A history + chapter theme → produces structured chapter draft.

Input: List of interview recordings (transcripts or Q&A pairs)
Output: Chapter draft with narrative arc, key quotes, thematic framing
"""

import json
import logging
import os
import subprocess
import sys
from dataclasses import dataclass, asdict, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

# ───────────────────────────────────────────────────────────────────
# Configuration
# ───────────────────────────────────────────────────────────────────

LLM_BACKEND = os.getenv("LLM_BACKEND", "ollama")  # "ollama" | "minimax"
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL = os.getenv("OLLAMA_MODEL", "gemma4:latest")
MINIMAX_API_KEY = os.getenv("MINIMAX_API_KEY", "")
MINIMAX_BASE_URL = os.getenv("MINIMAX_BASE_URL", "https://api.minimax.io/anthropic")
MINIMAX_MODEL = os.getenv("MINIMAX_MODEL", "MiniMax-M2.7")
MAX_CHAPTER_TOKENS = int(os.getenv("MAX_CHAPTER_TOKENS", "2048"))
CHAPTER_CACHE_ENABLED = os.getenv("CHAPTER_CACHE_ENABLED", "true").lower() == "true"
CHAPTER_CACHE_DIR = Path(os.getenv("CHAPTER_CACHE_DIR", "memories"))


# ───────────────────────────────────────────────────────────────────
# LLM Backend — Ollama or MiniMax M2.7
# ───────────────────────────────────────────────────────────────────

def _llm_complete(prompt: str, max_tokens: int = 2048) -> str:
    """Call LLM. Backend determined by LLM_BACKEND env var."""
    if LLM_BACKEND == "minimax":
        # MiniMax-M2.7 thinking model burns tokens on the thinking block before
        # emitting the JSON. Chapter generation needs significant room.
        return _minimax_complete(prompt, max(max_tokens, 6000))
    else:
        return _ollama_complete(prompt, max_tokens)


def _ollama_complete(prompt: str, max_tokens: int = 2048) -> str:
    """Call local Ollama /api/generate."""
    import urllib.request
    import urllib.error

    payload = {
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": False,
    }
    data = json.dumps(payload).encode()
    try:
        req = urllib.request.Request(
            f"{OLLAMA_URL}/api/generate",
            data=data,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            result = json.loads(resp.read())
        return result.get("response", "{}")
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        raise ChapterSummarizerError(f"Ollama call failed: {e}")


def _minimax_complete(prompt: str, max_tokens: int = 2048) -> str:
    """Call MiniMax M2.7 via Anthropic-compatible API."""
    import urllib.request
    import urllib.error

    if not MINIMAX_API_KEY:
        raise ChapterSummarizerError("MINIMAX_API_KEY not set (LLM_BACKEND=minimax)")

    payload = {
        "model": MINIMAX_MODEL,
        "max_tokens": max_tokens,
        "messages": [{"role": "user", "content": prompt}],
    }
    data = json.dumps(payload).encode()
    headers = {
        "Content-Type": "application/json",
        "x-api-key": MINIMAX_API_KEY,
        "anthropic-version": "2023-06-01",
    }
    try:
        req = urllib.request.Request(
            f"{MINIMAX_BASE_URL}/v1/messages",
            data=data,
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            result = json.loads(resp.read())
        content = result.get("content", [{}])
        if content and isinstance(content, list):
            for block in content:
                if isinstance(block, dict) and block.get("type") == "text":
                    return block.get("text", "{}")
        return "{}"
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        raise ChapterSummarizerError(f"MiniMax call failed: {e}")


# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class InterviewQA:
    """Single Q&A pair from interview session."""
    question: str
    response: str
    timestamp: str = ""
    category: str = ""
    score: Optional[int] = None  # 0-18 from question-engine


@dataclass
class ChapterSection:
    """A section within a chapter draft."""
    heading: str
    content: str
    key_quotes: list[str] = field(default_factory=list)
    source_qa_indices: list[int] = field(default_factory=list)


@dataclass
class ChapterDraft:
    """Complete chapter draft from multi-recording session."""
    title: str
    theme: str
    narrative_arc: str  # One-paragraph arc summary
    sections: list[ChapterSection]
    key_memories: list[str]  # Most vivid moments
    characters: list[str]  # People mentioned
    timeline_span: str  # "1952-1965" etc
    emotional_tone: str  # "Nostalgic", "Bittersweet", etc
    source_session_id: str
    source_recording_count: int
    confidence: str  # "high" / "medium" / "low" based on content density


# ───────────────────────────────────────────────────────────────────
# Exceptions
# ───────────────────────────────────────────────────────────────────

class ChapterSummarizerError(Exception):
    """Raised when chapter generation fails."""
    pass


# ───────────────────────────────────────────────────────────────────
# Content Loading
# ───────────────────────────────────────────────────────────────────

def load_interview_qas(source_path: str) -> list[InterviewQA]:
    """Load Q&A pairs from JSON, JSONL, or markdown file.

    Supports formats:
    - JSON: [{"question": "...", "response": "...", "timestamp": "...", "category": "..."}]
    - JSONL: One JSON object per line with same fields
    - Markdown: Parse ## Q: / ## A: headers
    """
    path = Path(source_path)
    if not path.exists():
        raise ChapterSummarizerError(f"Source file not found: {source_path}")

    content = path.read_text(encoding="utf-8").strip()
    suffix = path.suffix.lower()

    if suffix == ".json":
        data = json.loads(content)
        return [InterviewQA(**item) for item in data]
    elif suffix == ".jsonl":
        results = []
        for line in content.splitlines():
            if line.strip():
                item = json.loads(line)
                results.append(InterviewQA(**item))
        return results
    elif suffix in (".md", ".markdown"):
        return _parse_markdown_qas(content)
    else:
        # Try JSON first, then JSONL
        try:
            data = json.loads(content)
            if isinstance(data, list):
                return [InterviewQA(**item) for item in data]
        except json.JSONDecodeError:
            pass
        raise ChapterSummarizerError(f"Cannot parse format {suffix} for {source_path}")


def _parse_markdown_qas(content: str) -> list[InterviewQA]:
    """Parse Q&A from markdown format with ## Q: / ## A: headers."""
    results = []
    current_q = None
    current_a = ""

    for line in content.splitlines():
        line = line.strip()
        if line.startswith("## Q:") or line.startswith("**Q:**"):
            if current_q:
                results.append(InterviewQA(question=current_q, response=current_a.strip()))
            current_q = line.split(":", 1)[1].strip() if ":" in line else ""
            current_a = ""
        elif line.startswith("## A:") or line.startswith("**A:**"):
            current_a = line.split(":", 1)[1].strip() if ":" in line else ""
        elif current_q is not None:
            current_a += " " + line

    if current_q:
        results.append(InterviewQA(question=current_q, response=current_a.strip()))

    return results


# ───────────────────────────────────────────────────────────────────
# Chapter Generation
# ───────────────────────────────────────────────────────────────────

def _assert_transcription_not_paraphrase():
    """Pre-flight assertion: transcription-not-paraphrase v1.1+ must be loadable.

    Raises:
        ChapterSummarizerError: If skill is missing, version < v1.1, or missing Test 5
    """
    import logging
    from pathlib import Path

    logger = logging.getLogger(__name__)

    skill_path = Path(__file__).parent.parent / "transcription-not-paraphrase" / "SKILL.md"
    if not skill_path.exists():
        raise ChapterSummarizerError(
            "transcription-not-paraphrase SKILL.md not found — cannot generate chapter. "
            "Load skills/transcription-not-paraphrase/SKILL.md v1.1+ before chapter generation."
        )
    content = skill_path.read_text()
    if "v1.1" not in content:
        raise ChapterSummarizerError(
            "transcription-not-paraphrase must be v1.1+ (current version check failed). "
            "Upgrade to v1.1 before chapter generation."
        )
    if "Test 5" not in content:
        raise ChapterSummarizerError(
            "transcription-not-paraphrase v1.1 requires Test 5 (connector-smoothing doctrine). "
            "Missing Test 5 — cannot generate chapter."
        )
    logger.info("transcription-not-paraphrase v1.1.0 loaded — all 5 tests active")


def generate_chapter(
    qa_pairs: list[InterviewQA],
    chapter_theme: str,
    session_id: str,
) -> ChapterDraft:
    """Generate chapter draft from Q&A pairs and theme.

    Args:
        qa_pairs: List of Q&A pairs from interview
        chapter_theme: Thematic focus for this chapter (e.g. "Grandmother's kitchen and family meals")
        session_id: Identifier for this interview session

    Returns:
        ChapterDraft with sections, key quotes, characters, timeline
    """
    # ───────────────────────────────────────────────────────────────────
    # PRE-FLIGHT: transcription-not-paraphrase v1.1 must be loaded
    # ───────────────────────────────────────────────────────────────────
    _assert_transcription_not_paraphrase()

    if not qa_pairs:
        raise ChapterSummarizerError("No Q&A pairs provided")

    # Build Q&A context for prompt
    qa_context = ""
    for i, qa in enumerate(qa_pairs[:20]):  # Cap at 20 for context length
        score_str = f"[score: {qa.score}/18]" if qa.score is not None else ""
        qa_context += f"\n--- Q{i+1} [{qa.category}] {score_str} ---\nQ: {qa.question}\nA: {qa.response[:500]}\n"

    prompt = f"""You are producing a chapter for the Kept Voices family storytelling archive.

Theme for this chapter: "{chapter_theme}"

Interview Q&A from this session:
{qa_context}

Produce a structured chapter draft. Follow this EXACT JSON structure:
{{
  "title": "Chapter title (evocative, specific)",
  "theme": "The thematic focus restated",
  "narrative_arc": "One paragraph summarizing the emotional journey of this chapter",
  "sections": [
    {{
      "heading": "Section heading",
      "content": "3-5 paragraph narrative section weaving together responses",
      "key_quotes": ["vivid quote from response 1", "vivid quote from response 2"],
      "source_qa_indices": [0, 3]
    }}
  ],
  "key_memories": ["Most vivid sensory moment 1", "Most vivid sensory moment 2"],
  "characters": ["Person name 1", "Person name 2"],
  "timeline_span": "Approximate years covered (e.g. '1952-1965')",
  "emotional_tone": "Overall emotional quality (e.g. 'Nostalgic', 'Bittersweet', 'Joyful')",
  "confidence": "high/medium/low based on how much vivid content was captured",
  "source_session_id": "{session_id}",
  "source_recording_count": {len(qa_pairs)}
}}

Rules:
- sections should be 2-4 sections that together cover the chapter arc
- key_quotes must be VERBATIM excerpts from the responses (cut-and-paste, no rephrasing)
- source_qa_indices: 0-based index into the Q&A list above
- content should be narrative prose, NOT bullet points, weaving quotes into flowing paragraphs
- If responses are vague/sparse, note low confidence and work with what exists
- emotional_tone should be a single adjective or short phrase

Output JSON only, no markdown fences."""

    try:
        content = _llm_complete(prompt, MAX_CHAPTER_TOKENS)
        parsed = json.loads(content)
        # Coerce raw dict sections → ChapterSection instances before construction
        if "sections" in parsed:
            parsed["sections"] = [
                ChapterSection(**s) if isinstance(s, dict) else s
                for s in parsed["sections"]
            ]
        return ChapterDraft(**parsed)
    except (json.JSONDecodeError, TypeError) as e:
        raise ChapterSummarizerError(f"Failed to parse chapter response: {e}")


def chapter_to_markdown(draft: ChapterDraft) -> str:
    """Convert ChapterDraft to readable markdown."""
    lines = [
        f"# {draft.title}",
        "",
        f"**Theme:** {draft.theme}",
        f"**Emotional Tone:** {draft.emotional_tone}",
        f"**Timeline:** {draft.timeline_span}",
        f"**Confidence:** {draft.confidence}",
        f"**Source Sessions:** {draft.source_recording_count} recordings",
        "",
        f"_{draft.narrative_arc}_",
        "",
    ]

    for i, section in enumerate(draft.sections):
        lines.append(f"## {section.heading}")
        lines.append("")
        lines.append(section.content)
        if section.key_quotes:
            lines.append("")
            lines.append("**Key Quotes:**")
            for quote in section.key_quotes:
                lines.append(f"> {quote}")
        lines.append("")

    if draft.key_memories:
        lines.append("## Key Memories")
        lines.append("")
        for memory in draft.key_memories:
            lines.append(f"- {memory}")
        lines.append("")

    if draft.characters:
        lines.append(f"**Characters:** {', '.join(draft.characters)}")

    return "\n".join(lines)


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Chapter Summarizer — Multi-recording to chapter draft")
    parser.add_argument("source", help="Interview Q&A file (JSON/JSONL/MD)")
    parser.add_argument("--theme", required=True, help="Chapter theme/focus")
    parser.add_argument("--session-id", default="session-1", help="Session identifier")
    parser.add_argument("--format", choices=["json", "markdown"], default="markdown", help="Output format")
    parser.add_argument("--output", help="Output file (default: stdout)")

    args = parser.parse_args()

    try:
        qa_pairs = load_interview_qas(args.source)
        print(f"Loaded {len(qa_pairs)} Q&A pairs from {args.source}")

        draft = generate_chapter(qa_pairs, args.theme, args.session_id)

        if args.format == "json":
            output = json.dumps(draft, indent=2, default=str)
        else:
            output = chapter_to_markdown(draft)

        if args.output:
            Path(args.output).write_text(output)
            print(f"Chapter draft written to {args.output}")
        else:
            print(output)

    except ChapterSummarizerError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
