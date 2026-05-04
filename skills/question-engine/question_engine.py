#!/usr/bin/env python3
"""
Question Engine — AI Conversational Interviewer Brain

Takes (question_category, storyteller_context) → asks 1 question,
scores response quality across 6 dimensions, suggests follow-up.

Rubric pattern from skill-self-improver (6 dimensions, 0-3 scale).
"""

import json
import os
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────
# Configuration
# ───────────────────────────────────────────────────────────────────

LLM_BACKEND = os.getenv("LLM_BACKEND", "ollama")  # "ollama" | "minimax"

OLLAMA_URL = os.getenv("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL = os.getenv("OLLAMA_MODEL", "hermes3:8b-llama3.1-q8_0")

MINIMAX_API_KEY = os.getenv("MINIMAX_API_KEY", "")
MINIMAX_BASE_URL = os.getenv("MINIMAX_BASE_URL", "https://api.minimax.io/anthropic")
MINIMAX_MODEL = os.getenv("MINIMAX_MODEL", "MiniMax-M2.7")

MAX_QUESTION_TOKENS = int(os.getenv("MAX_QUESTION_TOKENS", "256"))
MAX_FOLLOWUP_TOKENS = int(os.getenv("MAX_FOLLOWUP_TOKENS", "256"))


# ───────────────────────────────────────────────────────────────────
# LLM Backend — Ollama or MiniMax M2.7
# ───────────────────────────────────────────────────────────────────

def _llm_complete(prompt: str, max_tokens: int = 256) -> str:
    """Call LLM. Backend determined by LLM_BACKEND env var."""
    if LLM_BACKEND == "minimax":
        # MiniMax-M2.7 is a thinking model — needs extra budget for the thinking
        # block before it emits text. Floor at 4000.
        return _minimax_complete(prompt, max(max_tokens, 4000))
    else:
        return _ollama_complete(prompt, max_tokens)


def _ollama_complete(prompt: str, max_tokens: int = 256) -> str:
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
        with urllib.request.urlopen(req, timeout=60) as resp:
            result = json.loads(resp.read())
        return result.get("response", "{}")
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        raise QuestionEngineError(f"Ollama call failed: {e}")


def _minimax_complete(prompt: str, max_tokens: int = 256) -> str:
    """Call MiniMax M2.7 via Anthropic-compatible API."""
    import urllib.request
    import urllib.error

    if not MINIMAX_API_KEY:
        raise QuestionEngineError("MINIMAX_API_KEY not set (LLM_BACKEND=minimax)")

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
        with urllib.request.urlopen(req, timeout=60) as resp:
            result = json.loads(resp.read())
        content = result.get("content", [{}])
        if content and isinstance(content, list):
            for block in content:
                if isinstance(block, dict) and block.get("type") == "text":
                    return block.get("text", "{}")
        return "{}"
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        raise QuestionEngineError(f"MiniMax call failed: {e}")


# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class ResponseScore:
    """6-dimension rubric score for interview response quality."""
    emotional_resonance: int      # 0-3: Does response connect emotionally?
    specificity: int              # 0-3: Concrete details vs vague generalities?
    narrative_continuity: int     # 0-3: Does it build on previous responses?
    completeness: int             # 0-3: Full answer vs deflection/dodge?
    authenticity_markers: int     # 0-3: Hesitation, emotion, sensory detail?
    coherence: int               # 0-3: Logical flow, no contradictions?

    @property
    def total(self) -> int:
        return (self.emotional_resonance + self.specificity +
                self.narrative_continuity + self.completeness +
                self.authenticity_markers + self.coherence)

    @property
    def grade(self) -> str:
        t = self.total
        if t >= 15:
            return "A"
        elif t >= 11:
            return "B"
        elif t >= 7:
            return "C"
        elif t >= 4:
            return "D"
        else:
            return "F"

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class InterviewQuestion:
    question: str
    category: str
    rationale: str  # Why this question now


@dataclass
class ScoredResponse:
    response_text: str
    score: ResponseScore
    feedback: str  # Natural language summary
    suggested_follow_up: Optional[str]
    dimension_notes: dict[str, str]  # Per-dimension explanation


# ───────────────────────────────────────────────────────────────────
# Question Generation
# ───────────────────────────────────────────────────────────────────

def generate_question(category: str, context: str, history: list[dict] | None = None) -> InterviewQuestion:
    """Generate 1 interview question for given category and storyteller context.

    Args:
        category: Question category (e.g. "childhood_memory", "turning_point")
        context: Storyteller background, prior responses, interview phase
        history: List of prior {question, response} dicts

    Returns:
        InterviewQuestion with question text, rationale, and follow-up prompt
    """
    history_section = ""
    if history:
        history_section = "\nPrior exchanges:\n"
        for h in history[-3:]:  # Last 3 only
            history_section += f"Q: {h.get('question', '')[:100]}\n"
            history_section += f"A: {h.get('response', '')[:200]}\n"

    prompt = f"""You are a Kept Voices interviewer — warm, attentive, preserving family stories.
Your job: produce a single spoken response for each turn, written as continuous narration for a warm British storyteller voice (Kokoro).

STRUCTURE (mandatory — do not deviate):
- The "question" field contains ONE spoken passage with TWO sentences
- Sentence 1: EMPATHETIC REFLECTION — 1-2 sentences. You MUST reflect a SPECIFIC DETAIL from what the storyteller just said. Never generic. For example, if they said "grandfather built furniture in his garage every weekend," your reflection must reference "the garage" or "furniture" or "every weekend" — a concrete detail, not an evaluation.
- Sentence 2: FOLLOW-UP QUESTION — a specific question that builds directly on that detail.
- TWO SEPARATE SENTENCES — reflection ends, then the question begins.
- Do NOT combine into one interrogative sentence.

Example of correct format:
"There is something quietly beautiful about a person who builds things on weekends — the patience of it, the ritual. What kind of pieces did he make?"

The first sentence reflects SPECIFIC DETAILS (builds on weekends, patience, ritual). The second sentence asks about those details (what pieces).

Example of WRONG format:
"Did your grandfather inspire you?" ← NO specific reflection, too generic
"It's clear woodworking was important to him. What did he make?" ← Second sentence is a question, not a follow-up building on the reflection

Category requested: {category}

Storyteller context:
{context[:2000]}
{history_section}

Output JSON with fields:
{{"question": "...", "category": "{category}", "rationale": "..."}}

Output JSON only, no markdown fences."""

    try:
        content = _llm_complete(prompt, MAX_QUESTION_TOKENS)
        return InterviewQuestion(**json.loads(content))
    except (json.JSONDecodeError, TypeError) as e:
        raise QuestionEngineError(f"Failed to parse question response: {e}")


# ───────────────────────────────────────────────────────────────────
# Response Scoring
# ───────────────────────────────────────────────────────────────────

def score_response(response_text: str, question: str, context: str,
                   history: list[dict] | None = None) -> ScoredResponse:
    """Score interview response across 6-dimension rubric.

    Args:
        response_text: The storyteller's response
        question: The question that was asked
        context: Storyteller context for grounding
        history: Prior Q&A pairs for continuity check

    Returns:
        ScoredResponse with score, feedback, suggested follow-up
    """
    history_section = ""
    if history:
        history_section = "\nPrior Q&A for continuity:\n"
        for h in history[-3:]:
            history_section += f"Q: {h.get('question', '')[:100]}\n"
            history_section += f"A: {h.get('response', '')[:200]}\n"

    prompt = f"""You are a Kept Voices quality assessor — evaluating interview responses for the family storytelling archive.

Rate the response on each of 6 dimensions (0-3 scale):

1. emotional_resonance: Does the response connect emotionally? (0=vague/disconnected, 3=deeply moving)
2. specificity: Does it have concrete details vs vague generalities? (0=generic, 3=vivid particulars)
3. narrative_continuity: Does it build on previous responses? (0=ignores context, 3=seamless building)
4. completeness: Did they fully answer vs deflect/dodge? (0=evasive, 3=complete answer)
5. authenticity_markers: Does it show real memory? (0=plausible-but-flat, 3=hesitation, emotion, sensory)
6. coherence: Is it logically consistent with itself? (0=contradictory, 3=clear and consistent)

Response to evaluate:
---
{response_text[:1500]}
---

Question asked: {question[:300]}
Context: {context[:500]}
{history_section}

Output JSON with fields:
{{
  "emotional_resonance": 0-3,
  "specificity": 0-3,
  "narrative_continuity": 0-3,
  "completeness": 0-3,
  "authenticity_markers": 0-3,
  "coherence": 0-3,
  "dimension_notes": {{
    "emotional_resonance": "...",
    "specificity": "...",
    "narrative_continuity": "...",
    "completeness": "...",
    "authenticity_markers": "...",
    "coherence": "..."
  }},
  "feedback": "2-3 sentence natural language summary of response quality",
  "suggested_follow_up": "specific follow-up question if any dimension < 2, else null"
}}

Output JSON only, no markdown fences."""

    try:
        content = _llm_complete(prompt, MAX_FOLLOWUP_TOKENS)
        data = json.loads(content)

        score = ResponseScore(**{k: data[k] for k in [
            "emotional_resonance", "specificity", "narrative_continuity",
            "completeness", "authenticity_markers", "coherence"
        ]})

        return ScoredResponse(
            response_text=response_text,
            score=score,
            feedback=data.get("feedback", ""),
            suggested_follow_up=data.get("suggested_follow_up"),
            dimension_notes=data.get("dimension_notes", {}),
        )
    except (json.JSONDecodeError, TypeError) as e:
        raise QuestionEngineError(f"Failed to parse score response: {e}")


# ───────────────────────────────────────────────────────────────────
# Exceptions
# ───────────────────────────────────────────────────────────────────

class QuestionEngineError(Exception):
    """Raised when question generation or scoring fails."""
    pass


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Question Engine — AI Conversational Interviewer")
    sub = parser.add_subparsers(dest="cmd", required=True)

    gen = sub.add_parser("generate", help="Generate 1 interview question")
    gen.add_argument("--category", required=True, help="Question category")
    gen.add_argument("--context", required=True, help="Storyteller context file or text")
    gen.add_argument("--history", help="JSON file with prior Q&A history")

    score = sub.add_parser("score", help="Score an interview response")
    score.add_argument("--response", required=True, help="Response text")
    score.add_argument("--question", required=True, help="Question asked")
    score.add_argument("--context", required=True, help="Storyteller context")
    score.add_argument("--history", help="JSON file with prior Q&A history")
    score.add_argument("--output", help="Write score JSON to file")

    args = parser.parse_args()

    if args.cmd == "generate":
        # Load context (file or literal)
        if Path(args.context).exists():
            context = Path(args.context).read_text()
        else:
            context = args.context

        history = None
        if args.history and Path(args.history).exists():
            history = json.loads(Path(args.history).read_text())

        q = generate_question(args.category, context, history)
        print(f"QUESTION [{q.category}]:")
        print(q.question)
        print(f"\nRationale: {q.rationale}")

    elif args.cmd == "score":
        if Path(args.response).exists():
            response = Path(args.response).read_text()
        else:
            response = args.response

        if Path(args.context).exists():
            context = Path(args.context).read_text()
        else:
            context = args.context

        history = None
        if args.history and Path(args.history).exists():
            history = json.loads(Path(args.history).read_text())

        result = score_response(response, args.question, context, history)
        print(f"SCORE: {result.score.grade} ({result.score.total}/18)")
        print(f"\n{result.feedback}")
        print(f"\nDimension breakdown:")
        for dim, val in result.score.to_dict().items():
            note = result.dimension_notes.get(dim, "")
            print(f"  {dim}: {val}/3 — {note}")
        if result.suggested_follow_up:
            print(f"\nSuggested follow-up: {result.suggested_follow_up}")

        if args.output:
            Path(args.output).write_text(json.dumps({
                "score": result.score.to_dict(),
                "grade": result.score.grade,
                "feedback": result.feedback,
                "suggested_follow_up": result.suggested_follow_up,
                "dimension_notes": result.dimension_notes,
            }, indent=2))
            print(f"\nScore written to {args.output}")
