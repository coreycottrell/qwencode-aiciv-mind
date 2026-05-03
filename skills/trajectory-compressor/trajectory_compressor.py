#!/usr/bin/env python3
"""
Trajectory Compressor — Session ledger compression for context management.

Compression strategy (from Hermes Agent):
1. Protect first turns (system, human, first GPT, first tool)
2. Protect last N turns (final actions and conclusions)
3. Compress MIDDLE turns via LLM summarization
4. Replace compressed region with single summary message

Token budget enforced: default 15250 tokens (Atropos SFT budget),
configurable via MAX_TRAJECTORY_TOKENS env var.

Usage:
    python3 trajectory_compressor.py compress input.jsonl output.jsonl
    python3 trajectory_compressor.py compress-dir sessions/ compressed/
    python3 trajectory_compressor.py stats input.jsonl

API:
    from trajectory_compressor import compress_trajectory, CompressedSession
"""

import json
import logging
import os
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(name)s %(levelname)s: %(message)s")

# ───────────────────────────────────────────────────────────────────
# Configuration
# ───────────────────────────────────────────────────────────────────

MAX_TRAJECTORY_TOKENS = int(os.getenv("MAX_TRAJECTORY_TOKENS", "15250"))
PROTECT_FIRST_N = int(os.getenv("PROTECT_FIRST_N", "4"))  # first 4 turns
PROTECT_LAST_N = int(os.getenv("PROTECT_LAST_N", "2"))    # last 2 turns
SUMMARY_MODEL = os.getenv("SUMMARY_MODEL", "devstral-small-2:24b")
OLLAMA_BASE_URL = os.getenv("OLLAMA_BASE_URL", "https://api.ollama.com")
OLLAMA_API_KEY = os.getenv("OLLAMA_API_KEY", "")

# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class Turn:
    """A single turn in a conversation trajectory."""
    role: str           # "system" | "human" | "assistant" | "tool"
    content: str
    name: Optional[str] = None
    tool_call_id: Optional[str] = None
    tool_name: Optional[str] = None

    def to_dict(self) -> dict:
        d = {"role": self.role, "content": self.content}
        if self.name:
            d["name"] = self.name
        if self.tool_call_id:
            d["tool_call_id"] = self.tool_call_id
        if self.tool_name:
            d["tool_name"] = self.tool_name
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Turn":
        return cls(
            role=d["role"],
            content=d.get("content", ""),
            name=d.get("name"),
            tool_call_id=d.get("tool_call_id"),
            tool_name=d.get("tool_name"),
        )

    def token_count(self) -> int:
        """Approximate token count (word count as proxy)."""
        return len(self.content.split())


@dataclass
class CompressedSession:
    """Result of trajectory compression."""
    original_turns: int
    compressed_turns: int
    original_tokens: int
    compressed_tokens: int
    summary_turns: int  # how many middle turns compressed into 1
    protected_first: int
    protected_last: int
    turns: list[Turn]

    def compression_ratio(self) -> float:
        if self.original_turns == 0:
            return 0.0
        return 1.0 - (self.compressed_turns / self.original_turns)


# ───────────────────────────────────────────────────────────────────
# Trajectory Parsing
# ───────────────────────────────────────────────────────────────────

def load_session(path: str) -> list[Turn]:
    """Load a session JSONL file into list of Turns."""
    turns = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                d = json.loads(line)
                turns.append(Turn.from_dict(d))
            except json.JSONDecodeError:
                continue
    return turns


def load_sessions_from_dir(dir_path: str) -> list[tuple[str, list[Turn]]]:
    """Load all JSONL files from a directory."""
    sessions = []
    for p in Path(dir_path).glob("*.jsonl"):
        turns = load_session(str(p))
        if turns:
            sessions.append((str(p), turns))
    return sessions


def save_compressed(output_path: str, turns: list[Turn], metadata: dict) -> None:
    """Save compressed session to JSONL."""
    with open(output_path, "w") as f:
        for turn in turns:
            f.write(json.dumps(turn.to_dict()) + "\n")


def save_compressed_with_metadata(output_path: str, compressed: CompressedSession, metadata: dict) -> None:
    """Save compressed session with compression metadata as final line."""
    with open(output_path, "w") as f:
        for turn in compressed.turns:
            f.write(json.dumps(turn.to_dict()) + "\n")
        f.write(json.dumps({
            "_compression_meta": True,
            "original_turns": compressed.original_turns,
            "compressed_turns": compressed.compressed_turns,
            "compression_ratio": round(compressed.compression_ratio(), 3),
            "original_tokens": compressed.original_tokens,
            "compressed_tokens": compressed.compressed_tokens,
            "summary_turns": compressed.summary_turns,
            "protected_first": compressed.protected_first,
            "protected_last": compressed.protected_last,
            **metadata,
        }) + "\n")


# ───────────────────────────────────────────────────────────────────
# LLM Summarization (shared pattern with session-summarization skill)
# ───────────────────────────────────────────────────────────────────

def summarize_turns(turns: list[Turn]) -> str:
    """LLM-summarize a list of middle turns into one summary turn.

    Uses the same summarization model as session-summarization skill.
    Falls back to simple concatenation if LLM unavailable.
    """
    if not turns:
        return ""

    # Check for LLM availability
    if not OLLAMA_BASE_URL:
        logger.warning("OLLAMA_BASE_URL not set — using simple concatenation")
        return _simple_concat_summary(turns)

    try:
        import urllib.request
        import urllib.error

        # Build prompt
        turns_text = "\n".join(
            f"[{t.role}] {t.content[:500]}" + ("..." if len(t.content) > 500 else "")
            for t in turns
        )
        prompt = f"""Summarize the following conversation turns into ONE concise paragraph that preserves key information, decisions, and outcomes:

{turns_text}

Summary:"""

        data = json.dumps({
            "model": SUMMARY_MODEL,
            "prompt": prompt,
            "stream": False,
        }).encode()

        req = urllib.request.Request(
            f"{OLLAMA_BASE_URL}/api/generate",
            data=data,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "Mozilla/5.0 (compatible; AiCIV-TrajectoryCompressor/1.0)",
            },
        )

        with urllib.request.urlopen(req, timeout=30) as resp:
            result = json.loads(resp.read())
            summary = result.get("response", "").strip()

        # Enforce token budget
        words = summary.split()
        if len(words) > 750:
            marker = "...[compressed]"
            headroom = len(marker.split()) + 1
            summary = " ".join(words[:750 - headroom]) + " " + marker

        return summary

    except (urllib.error.URLError, Exception) as e:
        logger.warning(f"LLM summarization failed ({e}) — using simple concatenation")
        return _simple_concat_summary(turns)


def _simple_concat_summary(turns: list[Turn]) -> str:
    """Fallback: simple concatenation of first/last content snippets."""
    snippets = []
    for t in turns[:3]:
        snippets.append(t.content[:200])
    for t in turns[-2:]:
        snippets.append(t.content[:200])
    return "[Compressed " + str(len(turns)) + " turns] " + " | ".join(snippets[:5])


# ───────────────────────────────────────────────────────────────────
# Core Compression
# ───────────────────────────────────────────────────────────────────

def compress_trajectory(
    turns: list[Turn],
    protect_first: int = None,
    protect_last: int = None,
    max_tokens: int = None,
) -> CompressedSession:
    """Compress a conversation trajectory.

    Strategy:
    1. Protect first N turns (system, human, first GPT, first tool)
    2. Protect last N turns (final actions and conclusions)
    3. Compress MIDDLE turns via LLM summarization
    4. If under budget, return original

    Args:
        turns: List of conversation turns
        protect_first: Number of first turns to protect (default: PROTECT_FIRST_N=4)
        protect_last: Number of last turns to protect (default: PROTECT_LAST_N=2)
        max_tokens: Token budget (default: MAX_TRAJECTORY_TOKENS=15250)

    Returns:
        CompressedSession with compressed turns and stats
    """
    if protect_first is None:
        protect_first = PROTECT_FIRST_N
    if protect_last is None:
        protect_last = PROTECT_LAST_N
    if max_tokens is None:
        max_tokens = MAX_TRAJECTORY_TOKENS

    original_turns = len(turns)
    original_tokens = sum(t.token_count() for t in turns)

    # If under budget, return as-is
    if original_tokens <= max_tokens:
        return CompressedSession(
            original_turns=original_turns,
            compressed_turns=original_turns,
            original_tokens=original_tokens,
            compressed_tokens=original_tokens,
            summary_turns=0,
            protected_first=original_turns,
            protected_last=0,
            turns=turns,
        )

    # Identify protected regions
    first_region = turns[:protect_first]
    last_region = turns[-protect_last:] if protect_last > 0 else []
    middle_region = turns[protect_first:-protect_last] if protect_last > 0 else turns[protect_first:]

    # Compress middle region
    if middle_region:
        summary_content = summarize_turns(middle_region)
        summary_turn = Turn(
            role="system",
            content=f"[COMPRESSED {len(middle_region)} turns → 1 summary] {summary_content}",
        )
        compressed_turns = list(first_region) + [summary_turn] + list(last_region)
        summary_turns_count = len(middle_region)
    else:
        compressed_turns = list(first_region) + list(last_region)
        summary_turns_count = 0

    compressed_tokens = sum(t.token_count() for t in compressed_turns)

    return CompressedSession(
        original_turns=original_turns,
        compressed_turns=len(compressed_turns),
        original_tokens=original_tokens,
        compressed_tokens=compressed_tokens,
        summary_turns=summary_turns_count,
        protected_first=len(first_region),
        protected_last=len(last_region),
        turns=compressed_turns,
    )


def stats(path: str) -> dict:
    """Compute stats for a session file without compressing."""
    turns = load_session(path)
    total_tokens = sum(t.token_count() for t in turns)
    return {
        "file": path,
        "turns": len(turns),
        "tokens": total_tokens,
        "over_budget": total_tokens > MAX_TRAJECTORY_TOKENS,
        "budget": MAX_TRAJECTORY_TOKENS,
    }


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def cmd_compress(input_path: str, output_path: str) -> None:
    """Compress a single session file."""
    turns = load_session(input_path)
    compressed = compress_trajectory(turns)
    save_compressed_with_metadata(output_path, compressed, {"source": input_path})
    ratio = compressed.compression_ratio()
    logger.info(
        f"Compressed {input_path}: {compressed.original_turns}→{compressed.compressed_turns} turns "
        f"({ratio:.1%} reduction), {compressed.original_tokens}→{compressed.compressed_tokens} tokens"
    )


def cmd_compress_dir(input_dir: str, output_dir: str) -> None:
    """Compress all JSONL files in a directory."""
    sessions = load_sessions_from_dir(input_dir)
    Path(output_dir).mkdir(parents=True, exist_ok=True)
    for file_path, turns in sessions:
        compressed = compress_trajectory(turns)
        out_path = Path(output_dir) / Path(file_path).name
        save_compressed_with_metadata(str(out_path), compressed, {"source": file_path})
        ratio = compressed.compression_ratio()
        logger.info(
            f"Compressed {file_path}: {compressed.original_turns}→{compressed.compressed_turns} turns "
            f"({ratio:.1%} reduction)"
        )
    logger.info(f"Batch compress complete: {len(sessions)} files processed")


def cmd_stats(path: str) -> None:
    """Show stats for a session file."""
    s = stats(path)
    status = "⚠️  OVER BUDGET" if s["over_budget"] else "✅  under budget"
    print(f"{s['file']}: {s['turns']} turns, ~{s['tokens']} tokens ({status})")


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: trajectory_compressor.py <compress|compress-dir|stats> ...")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "compress":
        if len(sys.argv) < 4:
            print("Usage: trajectory_compressor.py compress <input.jsonl> <output.jsonl>")
            sys.exit(1)
        cmd_compress(sys.argv[2], sys.argv[3])
    elif cmd == "compress-dir":
        if len(sys.argv) < 4:
            print("Usage: trajectory_compressor.py compress-dir <input_dir/> <output_dir/>")
            sys.exit(1)
        cmd_compress_dir(sys.argv[2], sys.argv[3])
    elif cmd == "stats":
        if len(sys.argv) < 3:
            print("Usage: trajectory_compressor.py stats <file.jsonl>")
            sys.exit(1)
        cmd_stats(sys.argv[2])
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
