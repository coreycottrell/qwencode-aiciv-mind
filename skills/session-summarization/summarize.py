#!/usr/bin/env python3
"""
Session Summarization — LLM-summarize relevant past sessions before context injection.

Pattern from Hermes Agent's session_search_tool.py:
1. Search scratchpads for relevant sessions (ripgrep)
2. Group by session, take top N matches
3. Truncate to window centered on match
4. LLM-summarize with fast/cheap model
5. Return summaries, not raw transcripts

This keeps token budget clean while retaining signal.
"""

import json
import logging
import os
import subprocess
import time
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

# ───────────────────────────────────────────────────────────────────
# Configuration
# ───────────────────────────────────────────────────────────────────

LLM_BACKEND = os.getenv("LLM_BACKEND", "ollama")  # "ollama" | "minimax"
OLLAMA_URL = os.getenv("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL = os.getenv("OLLAMA_MODEL", "devstral-small-2:24b")
MINIMAX_API_KEY = os.getenv("MINIMAX_API_KEY", "")
MINIMAX_BASE_URL = os.getenv("MINIMAX_BASE_URL", "https://api.minimax.io/anthropic")
MINIMAX_MODEL = os.getenv("MINIMAX_MODEL", "MiniMax-M2.7")
MAX_SUMMARY_TOKENS = int(os.getenv("MAX_SUMMARY_TOKENS", "2048"))
MAX_SESSION_CHARS = int(os.getenv("MAX_SESSION_CHARS", "100000"))
SUMMARY_CACHE_ENABLED = os.getenv("SUMMARY_CACHE_ENABLED", "true").lower() == "true"
SCRATCHPAD_DIR = Path(os.getenv("SCRATCHPAD_DIR", "/home/corey/projects/AI-CIV/qwen-aiciv-mind/scratchpads"))
MEMORY_DIR = Path(os.getenv("MEMORY_DIR", "/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds"))
CACHE_FILE = SCRATCHPAD_DIR / "_summary_cache.jsonl"


# ───────────────────────────────────────────────────────────────────
# LLM Backend — Ollama or MiniMax M2.7
# ───────────────────────────────────────────────────────────────────

def _llm_complete(prompt: str, max_tokens: int = 2048) -> str:
    """Call LLM. Backend determined by LLM_BACKEND env var."""
    if LLM_BACKEND == "minimax":
        # MiniMax-M2.7 thinking model needs extra room for the thinking block
        # before emitting text. Floor at 2000.
        return _minimax_complete(prompt, max(max_tokens, 2000))
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
        raise SessionSummarizationError(f"Ollama call failed: {e}")


def _minimax_complete(prompt: str, max_tokens: int = 2048) -> str:
    """Call MiniMax M2.7 via Anthropic-compatible API."""
    import urllib.request
    import urllib.error

    if not MINIMAX_API_KEY:
        raise SessionSummarizationError("MINIMAX_API_KEY not set (LLM_BACKEND=minimax)")

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
        raise SessionSummarizationError(f"MiniMax call failed: {e}")


# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class SessionSummary:
    session_id: str
    timestamp: str
    summary: str
    relevance_score: float
    source_file: str
    query: str  # The query used to find this

    def to_dict(self):
        return asdict(self)


# ───────────────────────────────────────────────────────────────────
# Exceptions
# ───────────────────────────────────────────────────────────────────

class SessionSummarizationError(Exception):
    """Raised when session summarization fails due to configuration or API issues."""
    pass


# ───────────────────────────────────────────────────────────────────
# Cache
# ───────────────────────────────────────────────────────────────────

def _load_cache() -> dict:
    """Load summary cache. Returns dict: query_hash -> SessionSummary dict."""
    if not CACHE_FILE.exists():
        return {}
    cache = {}
    try:
        for line in CACHE_FILE.read_text(encoding="utf-8").splitlines():
            if line.strip():
                entry = json.loads(line)
                cache[entry.get("query_hash", "")] = entry
    except (json.JSONDecodeError, IOError) as e:
        logger.warning("Failed to load summary cache: %s", e)
    return cache


def _write_cache_entry(summary: SessionSummary, query_hash: str):
    """Append a new cache entry."""
    if not SUMMARY_CACHE_ENABLED:
        return
    CACHE_FILE.parent.mkdir(parents=True, exist_ok=True)
    entry = summary.to_dict()
    entry["query_hash"] = query_hash
    with open(CACHE_FILE, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")


# ───────────────────────────────────────────────────────────────────
# Session Discovery
# ───────────────────────────────────────────────────────────────────

def _search_sessions(query: str, mind_id: str, limit: int = 10) -> list[dict]:
    """Search scratchpads for matching sessions via ripgrep.

    Returns list of dicts with: file_path, session_id, match_context, line_number.
    """
    search_root = SCRATCHPAD_DIR / mind_id if (SCRATCHPAD_DIR / mind_id).exists() else SCRATCHPAD_DIR
    if not search_root.exists():
        logger.warning("Scratchpad dir not found: %s", search_root)
        return []

    try:
        result = subprocess.run(
            ["rg", "-n", "-i", "--glob", "*.md", query, str(search_root)],
            capture_output=True, text=True, timeout=10
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as e:
        logger.warning("ripgrep failed, using Python fallback: %s", e)
        # Fallback: Python string search
        matches = []
        for f in search_root.rglob("*.md"):
            try:
                content = f.read_text(encoding="utf-8")
                if query.lower() in content.lower():
                    lines = content.split("\n")
                    for i, line in enumerate(lines):
                        if query.lower() in line.lower():
                            matches.append({
                                "file_path": str(f),
                                "session_id": f.stem,
                                "match_context": line.strip(),
                                "line_number": i + 1,
                            })
            except IOError:
                pass
        return matches[:limit]

    matches = []
    for line in result.stdout.strip().split("\n"):
        if ":" not in line:
            continue
        parts = line.split(":", 2)
        if len(parts) >= 3:
            file_path, line_num, context = parts[0], parts[1], parts[2]
            session_id = Path(file_path).stem
            matches.append({
                "file_path": file_path,
                "session_id": session_id,
                "match_context": context.strip(),
                "line_number": int(line_num) if line_num.isdigit() else 0,
            })

    return matches[:limit]


def _group_by_session(matches: list[dict]) -> dict:
    """Group matches by session. Returns: session_id -> {file_path, match_count, sample_context}."""
    sessions = {}
    for m in matches:
        sid = m["session_id"]
        if sid not in sessions:
            sessions[sid] = {
                "file_path": m["file_path"],
                "session_id": sid,
                "match_count": 0,
                "sample_contexts": [],
            }
        sessions[sid]["match_count"] += 1
        if len(sessions[sid]["sample_contexts"]) < 3:
            sessions[sid]["sample_contexts"].append(m["match_context"][:200])
    return sessions


# ───────────────────────────────────────────────────────────────────
# Content Loading & Truncation
# ───────────────────────────────────────────────────────────────────

def _load_session_content(file_path: str, line_number: int = 0) -> str:
    """Load session file, truncated to window centered on match.

    Returns up to MAX_SESSION_CHARS centered around the match line.
    """
    try:
        content = Path(file_path).read_text(encoding="utf-8")
    except IOError as e:
        logger.warning("Failed to read session file %s: %s", file_path, e)
        return ""

    if len(content) <= MAX_SESSION_CHARS:
        return content

    # Center truncation around match
    lines = content.split("\n")
    if line_number <= 0:
        center = len(lines) // 2
    else:
        center = min(line_number - 1, len(lines) - 1)

    half_window = MAX_SESSION_CHARS // 2
    start = max(0, center - half_window)
    end = min(len(lines), center + half_window)

    truncated = "\n".join(lines[start:end])
    return f"[...truncated from {len(content)} chars to {MAX_SESSION_CHARS} chars around line {line_number}...]\n{truncated}"


# ───────────────────────────────────────────────────────────────────
# LLM Summarization
# ───────────────────────────────────────────────────────────────────

def _summarize_with_llm(content: str, query: str, session_id: str) -> str:
    """Call LLM to summarize session content relative to query."""
    import urllib.request
    import urllib.error

    if not content.strip():
        return "[Empty session]"

    # Truncate content if too long for summarization prompt
    prompt_content = content[:8000] if len(content) > 8000 else content

    prompt = f"""Summarize this session transcript. Focus on information relevant to: "{query}"

Keep the summary under {MAX_SUMMARY_TOKENS} tokens. Include key decisions, outcomes, and any unresolved questions.

SESSION:
{prompt_content}

SUMMARY:"""

    try:
        summary = _llm_complete(prompt, MAX_SUMMARY_TOKENS)
        if not summary or summary == "{}":
            summary = "[No summary returned]"

        # ── Token budget enforcement ──────────────────────────────────
        # Use word count as rough token proxy (words ≈ tokens for English prose).
        # This is a CODE-LEVEL guarantee, not just a prompt request.
        word_count = len(summary.split())
        if word_count > MAX_SUMMARY_TOKENS:
            logger.warning(
                "LLM returned %d words (>%d cap). Truncating with marker.",
                word_count, MAX_SUMMARY_TOKENS
            )
            # Leave room for the truncation marker (~7 words).
            # Final output will be ≤ MAX_SUMMARY_TOKENS words.
            words = summary.split()
            marker = "...[summary truncated — exceeded token budget]..."
            headroom = len(marker.split()) + 1  # ~8 words for marker
            truncated = " ".join(words[:MAX_SUMMARY_TOKENS - headroom])
            summary = truncated + " " + marker
        # ── End enforcement ───────────────────────────────────────────

        return summary
    except SessionSummarizationError as e:
        logger.error("LLM summarization failed: %s", e)
        raise


# ───────────────────────────────────────────────────────────────────
# Main API
# ───────────────────────────────────────────────────────────────────

def summarize_sessions(query: str, mind_id: str = "hengshi", limit: int = 3) -> list[SessionSummary]:
    """Summarize past sessions relevant to query.

    Args:
        query: Search query to find relevant sessions
        mind_id: Which mind's scratchpads to search
        limit: Max number of sessions to summarize (default 3)

    Returns:
        List of SessionSummary objects, sorted by relevance

    Raises:
        SessionSummarizationError: If API key missing or LLM call fails
    """
    # Check preconditions
    if LLM_BACKEND == "minimax" and not MINIMAX_API_KEY:
        raise SessionSummarizationError(
            "MINIMAX_API_KEY not set (LLM_BACKEND=minimax)."
        )

    if not SCRATCHPAD_DIR.exists():
        raise SessionSummarizationError(f"Scratchpad dir not found: {SCRATCHPAD_DIR}")

    # Check cache
    import hashlib
    cache_key = f"{mind_id}:{query}:{limit}"
    query_hash = hashlib.sha256(cache_key.encode()).hexdigest()[:16]
    cache = _load_cache()
    if query_hash in cache:
        cached = cache[query_hash]
        return [SessionSummary(**{k: v for k, v in cached.items() if k != "query_hash"})]

    # Search
    matches = _search_sessions(query, mind_id, limit=limit * 3)
    if not matches:
        return []

    # Group by session
    sessions = _group_by_session(matches)
    sorted_sessions = sorted(
        sessions.values(),
        key=lambda s: s["match_count"],
        reverse=True
    )[:limit]

    # Summarize each session
    summaries = []
    for sess in sorted_sessions:
        content = _load_session_content(sess["file_path"], sess.get("line_number", 0))
        try:
            summary_text = _summarize_with_llm(content, query, sess["session_id"])
        except SessionSummarizationError:
            summary_text = "[Summarization failed — see logs]"

        # Extract timestamp from session file or use now
        try:
            mtime = Path(sess["file_path"]).stat().st_mtime
            timestamp = datetime.fromtimestamp(mtime, tz=timezone.utc).isoformat()
        except (IOError, OSError):
            timestamp = datetime.now(timezone.utc).isoformat()

        summary = SessionSummary(
            session_id=sess["session_id"],
            timestamp=timestamp,
            summary=summary_text,
            relevance_score=min(sess["match_count"] / 10.0, 1.0),
            source_file=sess["file_path"],
            query=query,
        )
        summaries.append(summary)

        # Cache
        _write_cache_entry(summary, query_hash)

    return summaries


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys, argparse

    parser = argparse.ArgumentParser(description="Session Summarization — LLM-summarize relevant past sessions")
    parser.add_argument("query", help="Search query to find relevant sessions")
    parser.add_argument("mind_id", nargs="?", default="hengshi", help="Mind ID (default: hengshi)")
    parser.add_argument("limit", nargs="?", type=int, default=3, help="Max sessions to summarize (default: 3)")
    parser.add_argument("--log-to-tracker", action="store_true", help="Log run to skill-evolution-tracker")
    args = parser.parse_args()

    print(f"Searching scratchpads for: {args.query}")
    print(f"Mind: {args.mind_id}, Limit: {args.limit}")
    print()

    outcome = "fail"
    result_count = 0
    try:
        results = summarize_sessions(args.query, mind_id=args.mind_id, limit=args.limit)
        if not results:
            print("No relevant sessions found.")
        else:
            for r in results:
                print(f"=== [{r.session_id}] (relevance: {r.relevance_score:.2f}) ===")
                print(r.summary)
                print()
            result_count = len(results)
        outcome = "pass"
    except SessionSummarizationError as e:
        print(f"ERROR: {e}", file=sys.stderr)

    # Log to skill-evolution-tracker if requested
    if args.log_to_tracker:
        try:
            tracker_path = Path(__file__).parent.parent / "skill-evolution-tracker" / "skill_evolution_tracker.py"
            context = f"query={args.query} mind={args.mind_id} results={result_count}"
            import subprocess as _sub
            result = _sub.run(
                [sys.executable, str(tracker_path), "log", "session-summarization",
                 "--context", context, "--outcome", outcome],
                capture_output=True, text=True, timeout=15
            )
            if result.returncode == 0:
                print(f"[skill-evolution-tracker: logged session-summarization ({outcome})]")
            else:
                print(f"[skill-evolution-tracker: log failed — {result.stderr.strip()}]")
        except Exception as e:
            print(f"[skill-evolution-tracker: could not log — {e}]")

    if outcome == "fail":
        sys.exit(1)
