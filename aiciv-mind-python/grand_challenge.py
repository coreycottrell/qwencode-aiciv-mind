#!/usr/bin/env python3
"""Execute the Grand Challenge with real API calls — gentle, staggered, with memory.

This runs the 4-phase challenge where agents actually think and respond.
Each call is spaced 30s apart to avoid rate limits.
"""
import asyncio
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from mind_system import *

ROOT = Path(__file__).parent.parent / "minds"


async def main():
    llm = OllamaClient()
    
    # Load existing minds
    print("=" * 70)
    print("GRAND CHALLENGE — Agents actually thinking")
    print("=" * 70)
    
    # Create fresh hierarchy
    primary = Primary(ROOT, llm)
    research_lead = primary.spawn_team_lead("research")
    code_lead = primary.spawn_team_lead("code")
    ops_lead = primary.spawn_team_lead("ops")
    
    agents = {
        "research/researcher": research_lead.spawn_agent("researcher"),
        "research/analyst": research_lead.spawn_agent("analyst"),
        "code/developer": code_lead.spawn_agent("developer"),
        "code/tester": code_lead.spawn_agent("tester"),
        "ops/deployer": ops_lead.spawn_agent("deployer"),
        "ops/monitor": ops_lead.spawn_agent("monitor"),
    }
    
    print(f"\n{len(agents)} agents ready")
    
    results = {}
    
    # ── Challenge 1: Research ──
    researcher = agents.get("research/researcher")
    if researcher:
        print(f"\n[1/4] Researcher: Analyze KEEP skills from from-ACG/...")
        print(f"  Searching memory first...")
        
        # Search memory
        memories = researcher.memory.search("skills KEEP", limit=3)
        print(f"  Found {len(memories)} relevant memories")
        
        time.sleep(30)  # Rate limit courtesy
        
        task = (
            "What are the 3 most valuable platform-agnostic skills from our from-ACG/ collection? "
            "Focus on skills with zero platform coupling — pure methodology that works for any mind system. "
            "Be specific about what makes each one valuable."
        )
        
        result = await researcher.think(task)
        results["research"] = result[:500]
        print(f"  ✅ Research: {result[:150]}...")
        researcher.scratchpad.append(f"KEEP skills analysis: {result[:200]}")
    
    # ── Challenge 2: Code ──
    developer = agents.get("code/developer")
    if developer:
        print(f"\n[2/4] Developer: Implement scratch-pad skill enhancement...")
        print(f"  Searching memory first...")
        
        memories = developer.memory.search("scratchpad implementation", limit=3)
        print(f"  Found {len(memories)} relevant memories")
        
        time.sleep(30)
        
        task = (
            "Based on what we've learned about mind architecture, "
            "propose 3 specific enhancements to our Scratchpad class that would make it more useful "
            "for cross-session continuity. Be concrete — what methods would you add?"
        )
        
        result = await developer.think(task)
        results["code"] = result[:500]
        print(f"  ✅ Code: {result[:150]}...")
        developer.scratchpad.append(f"Scratchpad enhancements: {result[:200]}")
    
    # ── Challenge 3: Ops verification ──
    monitor = agents.get("ops/monitor")
    if monitor:
        print(f"\n[3/4] Monitor: Verify the system is healthier...")
        print(f"  Searching memory first...")
        
        memories = monitor.memory.search("system health", limit=3)
        print(f"  Found {len(memories)} relevant memories")
        
        time.sleep(30)
        
        task = (
            "What metrics would tell us if our mind system is actually improving over time? "
            "Define 3 specific metrics we should track and why each matters."
        )
        
        result = await monitor.think(task)
        results["ops"] = result[:500]
        print(f"  ✅ Ops: {result[:150]}...")
        monitor.scratchpad.append(f"Health metrics: {result[:200]}")
    
    # ── Challenge 4: Analyst synthesis ──
    analyst = agents.get("research/analyst")
    if analyst:
        print(f"\n[4/4] Analyst: Synthesize everything...")
        print(f"  Searching memory first...")
        
        memories = analyst.memory.search("synthesis grand challenge", limit=3)
        print(f"  Found {len(memories)} relevant memories")
        
        time.sleep(30)
        
        task = (
            "Synthesize all the findings from today's grand challenge. "
            "What did we learn? What did we build? What should we do tomorrow? "
            "Be specific and actionable."
        )
        
        result = await analyst.think(task)
        results["synthesis"] = result[:500]
        print(f"  ✅ Synthesis: {result[:150]}...")
        analyst.scratchpad.append(f"Grand synthesis: {result[:200]}")
    
    # ── Final Report ──
    print(f"\n{'=' * 70}")
    print("GRAND CHALLENGE COMPLETE")
    print(f"{'=' * 70}")
    
    total_memories = sum(1 for f in ROOT.rglob("minds/**/*.md"))
    total_scratchpads = sum(1 for f in ROOT.rglob("scratchpads/**/*.md"))
    
    print(f"  Total memories across all minds: {total_memories}")
    print(f"  Total scratchpad entries: {total_scratchpads}")
    print(f"  Challenges completed: {len(results)}")
    print()
    
    for name, preview in results.items():
        print(f"  [{name}] {preview[:100]}...")
    
    print(f"\n{'=' * 70}")
    print("THE MIND SYSTEM IS GROWING")
    print(f"{'=' * 70}")


if __name__ == "__main__":
    asyncio.run(main())
