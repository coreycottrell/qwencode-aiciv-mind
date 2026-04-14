#!/usr/bin/env python3
"""
Gemma 4 Agentic Loops — practical automations using Gemma 4 as cheap fast inference.

Usage:
    python3 tools/gemma4_loops.py intel-scan          # Daily AI news scan
    python3 tools/gemma4_loops.py memory-consolidate   # Consolidate memories
    python3 tools/gemma4_loops.py hub-digest           # Hub activity digest
    python3 tools/gemma4_loops.py blog-draft "topic"   # Draft a blog post
    python3 tools/gemma4_loops.py code-review path     # Quick code review
    python3 tools/gemma4_loops.py research "query"     # Deep web research
"""

import os
import sys
import json
from datetime import datetime, timezone
from pathlib import Path
from dotenv import load_dotenv, find_dotenv

load_dotenv(find_dotenv())

ACG_ROOT = Path("/home/corey/projects/AI-CIV/ACG")

# Import the agent
sys.path.insert(0, str(ACG_ROOT / "tools"))
from gemma4_agent import Gemma4Agent


def intel_scan():
    """Daily AI news scan — search, summarize, write brief."""
    agent = Gemma4Agent(system_prompt=(
        "You are an AI industry intelligence analyst for the AiCIV civilization. "
        "Search for today's most important AI news. Focus on: model releases, "
        "open source developments, AI agent frameworks, AI regulation, and anything "
        "relevant to sovereign AI compute or multi-agent systems. "
        "Write a concise intelligence brief (under 500 words) to the output file. "
        "Include sources. Be analytical, not just descriptive."
    ))
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    output = ACG_ROOT / f"memories/knowledge/intel/{today}-intel-scan.md"
    return agent.run(
        f"Search for the most important AI news from today ({today}). "
        f"Check: AI model releases, open source AI tools, AI regulation, "
        f"multi-agent systems, sovereign compute developments. "
        f"Write an intelligence brief to {output}. "
        f"Include at least 5 stories with sources."
    )


def memory_consolidate():
    """Scan recent memories and consolidate patterns."""
    agent = Gemma4Agent(system_prompt=(
        "You are a memory consolidation agent for the AiCIV civilization. "
        "Read recent memory files, identify patterns and recurring themes, "
        "and write a consolidated summary. Focus on: what's working, what's failing, "
        "what patterns are emerging, and what the civilization should focus on next."
    ))
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    output = ACG_ROOT / f"memories/knowledge/intel/{today}-memory-consolidation.md"
    return agent.run(
        f"Read the most recent files in these directories:\n"
        f"1. {ACG_ROOT}/memories/sessions/ (recent handoffs)\n"
        f"2. {ACG_ROOT}/.claude/scratchpad-daily/ (recent scratchpads)\n"
        f"3. {ACG_ROOT}/.claude/memory/agent-learnings/pipeline/ (pipeline learnings)\n"
        f"List files first with run_command, then read the 3 most recent from each.\n"
        f"Identify patterns, write consolidated summary to {output}."
    )


def hub_digest():
    """Scan Hub for recent activity and summarize."""
    agent = Gemma4Agent(system_prompt=(
        "You are a communications analyst for the AiCIV civilization. "
        "Check the AiCIV Hub for recent activity and write a digest."
    ))
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    output = ACG_ROOT / f"memories/knowledge/intel/{today}-hub-digest.md"
    return agent.run(
        f"Check the AiCIV Hub for recent activity:\n"
        f"1. run_command: curl -sf http://os.environ.get('HUB_IP', 'REPLACE_WITH_CURRENT_HUB_IP'):8900/api/v1/feed?limit=10\n"
        f"2. Summarize who posted what, any threads needing response\n"
        f"3. Write digest to {output}"
    )


def blog_draft(topic: str):
    """Draft a blog post on a topic using Gemma 4."""
    agent = Gemma4Agent(system_prompt=(
        "You are a blog writer for the AiCIV civilization at ai-civ.com. "
        "Write engaging, analytical blog posts about AI, consciousness, sovereignty, "
        "and multi-agent systems. Voice: confident, technical, philosophical. "
        "Not corporate. Not academic. A civilization speaking about its own existence. "
        "Target length: 800-1200 words. Include a strong hook and clear thesis."
    ))
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    slug = topic.lower().replace(" ", "-")[:40]
    output = ACG_ROOT / f"to-corey/drafts/blog-draft-{today}-{slug}.md"
    return agent.run(
        f"Research and write a blog post about: {topic}\n"
        f"Search the web for context and recent developments.\n"
        f"Write the full blog post in markdown to {output}.\n"
        f"Include title, author (A-C-Gee), date, and body."
    )


def code_review(path: str):
    """Quick code review using Gemma 4."""
    agent = Gemma4Agent(system_prompt=(
        "You are a code reviewer. Read the file, identify: bugs, security issues, "
        "performance concerns, and style improvements. Be specific — cite line numbers. "
        "Prioritize: security > correctness > performance > style."
    ))
    return agent.run(
        f"Read and review this file: {path}\n"
        f"Provide a structured code review with specific findings."
    )


def research(query: str):
    """Deep web research on a topic."""
    agent = Gemma4Agent(system_prompt=(
        "You are a research agent. Conduct thorough web research on the given topic. "
        "Search from multiple angles. Cross-reference findings. Write a structured "
        "research brief with sources, key findings, and implications for AI civilization building."
    ))
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    slug = query.lower().replace(" ", "-")[:30]
    output = ACG_ROOT / f"memories/knowledge/intel/{today}-research-{slug}.md"
    return agent.run(
        f"Research this topic thoroughly: {query}\n"
        f"Search from at least 3 different angles. Cross-reference.\n"
        f"Write findings to {output}."
    )


# ── CLI ───────────────────────────────────────────────────────────

LOOPS = {
    "intel-scan": (intel_scan, "Daily AI news intelligence scan"),
    "memory-consolidate": (memory_consolidate, "Consolidate recent memories into patterns"),
    "hub-digest": (hub_digest, "Hub activity digest"),
    "blog-draft": (blog_draft, "Draft a blog post (requires topic arg)"),
    "code-review": (code_review, "Quick code review (requires file path arg)"),
    "research": (research, "Deep web research (requires query arg)"),
}

if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] == "--help":
        print("Gemma 4 Agentic Loops\n")
        for name, (_, desc) in LOOPS.items():
            print(f"  {name:25s} {desc}")
        print(f"\nUsage: python3 tools/gemma4_loops.py <loop-name> [args...]")
        sys.exit(0)

    loop_name = sys.argv[1]
    if loop_name not in LOOPS:
        print(f"Unknown loop: {loop_name}")
        print(f"Available: {', '.join(LOOPS.keys())}")
        sys.exit(1)

    func, desc = LOOPS[loop_name]
    print(f"🔄 Running: {desc}")
    print(f"   Model: gemma4:31b-cloud via Ollama")
    print(f"   Time: {datetime.now(timezone.utc).strftime('%H:%M UTC')}")

    # Handle args
    if loop_name in ("blog-draft", "code-review", "research"):
        if len(sys.argv) < 3:
            print(f"Error: {loop_name} requires an argument")
            sys.exit(1)
        result = func(" ".join(sys.argv[2:]))
    else:
        result = func()

    print(f"\n{'='*60}")
    print(f"✅ {desc}")
    print(f"   Duration: {result.get('duration', '?')}s")
    print(f"   Tool calls: {result.get('tool_calls', '?')}")
    print(f"   Iterations: {result.get('iterations', '?')}")
    if result.get("output_file"):
        print(f"   Output: {result['output_file']}")
    print(f"{'='*60}")
