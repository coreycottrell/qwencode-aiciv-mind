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

SUMMARIZATION_MODEL = os.getenv("SUMMARIZATION_MODEL", "devstral-small-2:24b")
OLLAMA_BASE_URL = os.getenv("OLLAMA_BASE_URL", "https://api.ollama.com")
OLLAMA_API_KEY = os.getenv("OLLAMA_API_KEY", "")
MAX_SUMMARY_TOKENS = int(os.getenv("MAX_SUMMARY_TOKENS", "2048"))
MAX_SESSION_CHARS = int(os.getenv("MAX_SESSION_CHARS", "100000"))
SUMMARY_CACHE_ENABLED = os.getenv("SUMMARY_CACHE_ENABLED", "true").lower() == "true"
SCRATCHPAD_DIR = Path(os.getenv("SCRATCHPAD_DIR", "/home/corey/projects/AI-CIV/qwen-aiciv-mind/scratchpads"))
MEMORY_DIR = Path(os.getenv("MEMORY_DIR", "/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds"))
CACHE_FILE = SCRATCHPAD_DIR / "_summary_cache.jsonl"


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

    payload = {
        "model": SUMMARIZATION_MODEL,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "stream": False,
    }

    data = json.dumps(payload).encode()
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {OLLAMA_API_KEY}" if OLLAMA_API_KEY else "",
        "User-Agent": "Mozilla/5.0 (compatible; AiCIV-Mind/1.0; +https://ai-civ.com)",
    }

    try:
        req = urllib.request.Request(
            f"{OLLAMA_BASE_URL}/api/chat",
            data=data,
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=60) as resp:
            result = json.loads(resp.read())
        summary = result.get("message", {}).get("content", "[No summary returned]")

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
    except (urllib.error.URLError, json.JSONDecodeError, KeyError) as e:
        logger.error("LLM summarization failed: %s", e)
        raise SessionSummarizationError(f"LLM call failed: {e}")


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
    if not OLLAMA_API_KEY and not os.getenv("OPENAI_API_KEY"):
        raise SessionSummarizationError(
            "No LLM API key set. Set OLLAMA_API_KEY or OPENAI_API_KEY."
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
    import sys

    if len(sys.argv) < 2:
        print("Usage: python summarize.py <query> [mind_id] [limit]")
        sys.exit(1)

    q = sys.argv[1]
    mid = sys.argv[2] if len(sys.argv) > 2 else "hengshi"
    lim = int(sys.argv[3]) if len(sys.argv) > 3 else 3

    print(f"Searching scratchpads for: {q}")
    print(f"Mind: {mid}, Limit: {lim}")
    print()

    try:
        results = summarize_sessions(q, mind_id=mid, limit=lim)
        if not results:
            print("No relevant sessions found.")
        for r in results:
            print(f"=== [{r.session_id}] (relevance: {r.relevance_score:.2f}) ===")
            print(r.summary)
            print()
    except SessionSummarizationError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
