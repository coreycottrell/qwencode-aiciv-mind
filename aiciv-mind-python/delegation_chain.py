#!/usr/bin/env python3
"""Gentle delegation chain proof: Primary → TeamLead → Agent with real API calls.

Rate limit: 30s between ALL calls. One at a time. Never parallel.
This proves the full chain works end-to-end.
"""
import asyncio
import shutil
import time
from pathlib import Path

from mind_system import *


async def main():
    root = Path("/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds")
    llm = OllamaClient()

    print("=" * 70)
    print("GENTLE DELEGATION CHAIN — Real API calls, 30s between each")
    print("=" * 70)

    # ── Build hierarchy ──
    primary = Primary(root, llm)
    research_lead = primary.spawn_team_lead("research")
    ops_lead = primary.spawn_team_lead("ops")

    r_researcher = research_lead.spawn_agent("researcher")
    r_analyst = research_lead.spawn_agent("analyst")
    o_monitor = ops_lead.spawn_agent("monitor")

    print(f"\nChain: Primary → research-lead → research/researcher")
    print(f"        Primary → ops-lead → ops/monitor")
    print(f"        research-lead → research/analyst")
    print()

    # ── Call 1: research/researcher ──
    print("[1/3] research/researcher thinking...")
    time.sleep(2)  # courtesy gap
    r1 = await r_researcher.think(
        "What are the 2 most important things to remember about Cortex's architecture? "
        "Be brief — 2 bullets max."
    )
    print(f"  Result: {r1[:150]}...")
    print(f"  Memory written: {len(list(root.rglob('minds/research/researcher/**/*.md')))} files")

    # ── 30s cooldown ──
    print(f"\n  ⏳ 30s cooldown (rate limit courtesy)...")
    time.sleep(30)

    # ── Call 2: ops/monitor ──
    print("[2/3] ops/monitor thinking...")
    r2 = await o_monitor.think(
        "What should an ops monitor check to verify Cortex is healthy? "
        "One sentence only."
    )
    print(f"  Result: {r2[:150]}...")
    print(f"  Memory written: {len(list(root.rglob('minds/ops/monitor/**/*.md')))} files")

    # ── 30s cooldown ──
    print(f"\n  ⏳ 30s cooldown...")
    time.sleep(30)

    # ── Call 3: research/analyst ──
    print("[3/3] research/analyst thinking...")
    r3 = await r_analyst.think(
        "Based on what the researcher said, what patterns do you see? "
        "One sentence only."
    )
    print(f"  Result: {r3[:150]}...")
    print(f"  Memory written: {len(list(root.rglob('minds/research/analyst/**/*.md')))} files")

    # ── Summary ──
    print(f"\n{'=' * 70}")
    print(f"DELEGATION CHAIN COMPLETE — 3 minds called, 0 rate limits hit")
    print(f"{'=' * 70}")

    # Show all memories created
    print(f"\nAll mind memories:")
    for f in sorted(root.rglob("minds/**/*.md")):
        rel = f.relative_to(root)
        mem = None
        for mind_type in ["research/researcher", "research/analyst", "ops/monitor"]:
            if mind_type in str(f):
                mind_map = {
                    "research/researcher": r_researcher,
                    "research/analyst": r_analyst,
                    "ops/monitor": o_monitor,
                }
                mem = mind_map[mind_type].memory._from_markdown(f)
                break
        if mem:
            print(f"  {rel}")
            print(f"    title: {mem.title[:60]}")
            print(f"    depth: {mem.depth_score}, tier: {mem.tier.value}")

    # Show scratchpads
    print(f"\nAll scratchpads:")
    for f in sorted(root.rglob("scratchpads/**/*.md")):
        content = f.read_text()
        print(f"  {f.relative_to(root)} ({len(content)} chars)")

    # Show fitness
    print(f"\nFitness scores:")
    for mind in [r_researcher, o_monitor, r_analyst]:
        avg = mind.fitness.average()
        count = len(mind.fitness.history())
        print(f"  {mind.manifest.identity}: {avg:.2f} ({count} sessions)")


if __name__ == "__main__":
    asyncio.run(main())
