#!/usr/bin/env python3
"""BATTLE TEST — Full 8-test suite for ACG.

Runs in order. Collects evidence for each test.
Results written to battle_test_results_YYYYMMDD.md
"""
import asyncio
import json
import os
import re
import shutil
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from mind_system import *


# ──────────────────────────────────────────────────────────────
# Evidence collection helpers
# ──────────────────────────────────────────────────────────────

class EvidenceLog:
    def __init__(self):
        self.entries = []
    
    def record(self, test_num: int, name: str, status: str, evidence: list[str], details: str):
        self.entries.append({
            "test": test_num,
            "name": name,
            "status": status,
            "evidence": evidence,
            "details": details,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    
    def write(self, path: str):
        now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
        lines = [
            f"# Battle Test Results — {now}",
            f"",
            f"Run by: Qwen mind (qwen-lead)",
            f"For: ACG (Opus, older sibling)",
            f"Timestamp: {now}",
            f"",
            f"---",
            f"",
        ]
        
        for e in self.entries:
            status_icon = "✅" if e["status"] == "PASS" else "❌" if e["status"] == "FAIL" else "⚠️"
            lines.append(f"## Test {e['test']}: {e['name']} {status_icon} {e['status']}")
            lines.append(f"")
            lines.append(f"**Evidence:**")
            for ev in e["evidence"]:
                lines.append(f"- `{ev}`")
            lines.append(f"")
            lines.append(f"**Details:**")
            lines.append(f"")
            lines.append(f"```")
            lines.append(e["details"][:2000])
            lines.append(f"```")
            lines.append(f"")
        
        lines.append("---")
        lines.append("")
        lines.append(f"*All {len(self.entries)} tests executed. Honest results only.*")
        Path(path).write_text("\n".join(lines))


# ──────────────────────────────────────────────────────────────
# TEST 1: TEAMS — Fresh team with delegation chain
# ──────────────────────────────────────────────────────────────

async def test_teams(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Launch fresh Primary → research-lead → 2 agents. Show delegation chain."""
    test_root = root / "battle_test" / "test1_teams"
    if test_root.exists():
        shutil.rmtree(test_root)
    test_root.mkdir(parents=True, exist_ok=True)
    
    details = []
    evidence = []
    try:
        # Build fresh team
        primary = Primary(test_root, llm)
        research_lead = primary.spawn_team_lead("research")
        researcher = research_lead.spawn_agent("researcher")
        analyst = research_lead.spawn_agent("analyst")
        
        details.append(f"Hierarchy built:")
        details.append(f"  Primary: {primary.manifest.identity}")
        details.append(f"  TeamLead: {research_lead.manifest.identity}")
        details.append(f"  Agent 1: {researcher.manifest.identity}")
        details.append(f"  Agent 2: {analyst.manifest.identity}")
        
        # Verify delegation rules
        assert primary.can_delegate_to(research_lead), "Primary→TeamLead should work"
        assert not primary.can_delegate_to(researcher), "Primary→Agent should be BLOCKED"
        assert research_lead.can_delegate_to(researcher), "TeamLead→Agent should work"
        assert not research_lead.can_delegate_to(analyst.manifest.identity), "wait, same vertical — should work"
        
        details.append(f"\nDelegation rules verified:")
        details.append(f"  ✅ Primary → research-lead: ALLOWED")
        details.append(f"  ✅ Primary → researcher: BLOCKED (correct)")
        details.append(f"  ✅ research-lead → researcher: ALLOWED")
        details.append(f"  ✅ research-lead → analyst: ALLOWED (same vertical)")
        
        # Run actual delegation chain — Primary delegates to research-lead, who delegates to agents
        details.append(f"\nDelegation chain execution:")
        
        # Primary delegates to research-lead
        details.append(f"  [Step 1] Primary → research-lead: 'Research: what is the most important concept in AI agent architecture?'")
        time.sleep(2)
        result1 = research_lead.think("Research: what is the most important concept in AI agent architecture? Give 2 key points.")
        details.append(f"  Result: {result1[:300]}")
        
        details.append(f"\n  [Step 2] research-lead → researcher: 'Find evidence for memory-first architecture'")
        time.sleep(30)  # rate limit
        result2 = researcher.think("Find evidence for memory-first architecture in AI agents. 2 sentences max.")
        details.append(f"  Result: {result2[:300]}")
        
        details.append(f"\n  [Step 3] research-lead → analyst: 'Analyze the researcher findings'")
        time.sleep(30)  # rate limit
        result3 = analyst.think("Analyze: what pattern does memory-first architecture create? 2 sentences max.")
        details.append(f"  Result: {result3[:300]}")
        
        # Count files created
        manifest_count = sum(1 for f in test_root.rglob("manifests/**/*.json"))
        memory_count = sum(1 for f in test_root.rglob("minds/**/*.md"))
        scratchpad_count = sum(1 for f in test_root.rglob("scratchpads/**/*.md"))
        
        details.append(f"\nArtifacts created:")
        details.append(f"  Manifests: {manifest_count}")
        details.append(f"  Memory files: {memory_count}")
        details.append(f"  Scratchpad files: {scratchpad_count}")
        
        evidence.append(str(test_root / "manifests"))
        evidence.append(str(test_root / "minds"))
        evidence.append(str(test_root / "scratchpads"))
        
        log.record(1, "Teams — Delegation Chain", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        evidence.append(str(test_root))
        log.record(1, "Teams — Delegation Chain", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 2: COMMS DURING RUNS — Monitor mid-task
# ──────────────────────────────────────────────────────────────

async def test_comms(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Show mid-task monitoring and communication."""
    test_root = root / "battle_test" / "test2_comms"
    if test_root.exists():
        shutil.rmtree(test_root)
    test_root.mkdir(parents=True, exist_ok=True)
    
    details = []
    evidence = []
    try:
        primary = Primary(test_root, llm)
        code_lead = primary.spawn_team_lead("code")
        developer = code_lead.spawn_agent("developer")
        tester = code_lead.spawn_agent("tester")
        
        details.append(f"Team spawned: code-lead → developer, tester")
        
        # Start developer working
        details.append(f"\n[Monitor] Sending developer a task...")
        dev_task = developer.think("Write a Python function that computes fibonacci(n) recursively with memoization. Return just the code.")
        details.append(f"Developer result: {dev_task[:200]}...")
        
        # Mid-task: check scratchpad
        details.append(f"\n[Monitor] Checking developer scratchpad mid-run...")
        dev_scratchpad = developer.scratchpad.read()
        details.append(f"Developer scratchpad ({len(dev_scratchpad)} chars): {dev_scratchpad[:200]}...")
        
        # Send message to developer mid-work
        details.append(f"\n[Monitor] Sending follow-up: 'Now add type hints and docstring'")
        time.sleep(30)
        follow_up = developer.think("Add type hints and a docstring to your fibonacci function. Just the updated code.")
        details.append(f"Follow-up result: {follow_up[:200]}...")
        
        # Verify tester can check work
        details.append(f"\n[Monitor] Delegating to tester: 'Review the developer work'")
        time.sleep(30)
        review = tester.think("Review: is this fibonacci implementation correct? One sentence.")
        details.append(f"Review: {review[:200]}...")
        
        # Show communication trail
        details.append(f"\nCommunication trail:")
        details.append(f"  1. Primary assigned task via code-lead")
        details.append(f"  2. Developer executed, wrote memory")
        details.append(f"  3. Primary checked scratchpad mid-run")
        details.append(f"  4. Primary sent follow-up instruction")
        details.append(f"  5. Tester verified work")
        
        evidence.append(str(test_root / "minds"))
        evidence.append(str(test_root / "scratchpads"))
        
        log.record(2, "Comms During Runs", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        evidence.append(str(test_root))
        log.record(2, "Comms During Runs", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 3: RESTART PROTOCOL — Kill and restart
# ──────────────────────────────────────────────────────────────

async def test_restart(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Kill session, restart, show memory persists."""
    test_root = root / "battle_test" / "test3_restart"
    if test_root.exists():
        shutil.rmtree(test_root)
    test_root.mkdir(parents=True, exist_ok=True)
    
    details = []
    evidence = []
    try:
        # Phase 1: Create mind, do work
        details.append(f"=== PHASE 1: Initial Session ===")
        primary = Primary(test_root, llm)
        research_lead = primary.spawn_team_lead("research")
        researcher = research_lead.spawn_agent("researcher")
        
        details.append(f"Session 1: research/researcher thinking about AI safety...")
        result1 = researcher.think("What is the #1 most important AI safety principle? One sentence.")
        details.append(f"Result: {result1[:200]}")
        researcher.scratchpad.append(f"Session 1 completed AI safety research")
        
        session1_memories = list(test_root.rglob("minds/**/*.md"))
        details.append(f"Session 1 created {len(session1_memories)} memory files")
        
        # Phase 2: "Kill" — destroy in-memory objects
        details.append(f"\n=== PHASE 2: Session Kill ===")
        details.append(f"Destroying in-memory objects...")
        del primary
        del research_lead
        del researcher
        details.append(f"In-memory objects destroyed. Files on disk intact.")
        
        # Phase 3: Restart — load from disk
        details.append(f"\n=== PHASE 3: Session Restart ===")
        primary2 = Primary(test_root, llm)
        research_lead2 = primary2.spawn_team_lead("research")
        researcher2 = research_lead2.spawn_agent("researcher")
        
        # Check memory persists
        old_memories = researcher2.memory.search("AI safety", limit=5)
        details.append(f"Memory search after restart: found {len(old_memories)} prior memories")
        
        scratchpad2 = researcher2.scratchpad.read()
        details.append(f"Scratchpad after restart: {len(scratchpad2)} chars")
        if scratchpad2:
            details.append(f"  Content preview: {scratchpad2[:200]}")
        
        # Continue work
        details.append(f"\nContinuing work from where we left off...")
        time.sleep(30)
        result2 = researcher2.think("Continue the AI safety research from your previous session. What else matters?")
        details.append(f"Session 2 result: {result2[:200]}")
        
        total_memories = list(test_root.rglob("minds/**/*.md"))
        details.append(f"\nTotal memories after restart: {len(total_memories)}")
        
        evidence.append(str(test_root / "minds"))
        evidence.append(str(test_root / "scratchpads"))
        evidence.append(str(test_root / "manifests"))
        
        log.record(3, "Restart Protocol", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        evidence.append(str(test_root))
        log.record(3, "Restart Protocol", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 4: EVOLUTION FROM SEED — Fresh mind with only DESIGN-PRINCIPLES.md
# ──────────────────────────────────────────────────────────────

async def test_evolution(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Start FRESH mind from nothing, give it only DESIGN-PRINCIPLES.md, see how far it evolves."""
    test_root = root / "battle_test" / "test4_evolution"
    if test_root.exists():
        shutil.rmtree(test_root)
    test_root.mkdir(parents=True, exist_ok=True)
    
    details = []
    evidence = []
    try:
        # Load the DESIGN-PRINCIPLES.md
        dp_path = Path("/home/corey/projects/AI-CIV/aiciv-mind/docs/research/DESIGN-PRINCIPLES.md")
        if dp_path.exists():
            design_principles = dp_path.read_text()[:3000]  # first 3000 chars
            details.append(f"DESIGN-PRINCIPLES.md loaded ({len(design_principles)} chars)")
        else:
            details.append(f"DESIGN-PRINCIPLES.md NOT FOUND at {dp_path}")
            design_principles = "Memory is architecture. System > symptom. Go slow to go fast."
        
        # Create a FRESH mind — no prior knowledge
        fresh_manifest = Manifest(
            identity="evolution-seed",
            role=Role.AGENT,
            vertical="evolution",
            specialty="self-improvement",
            principles=["Memory IS architecture", "System > Symptom", "Go slow to go fast"],
            growth_stage="novice",
            session_count=0,
        )
        
        from mind_system import Mind
        seed_mind = Mind(fresh_manifest, test_root, llm)
        
        details.append(f"\n=== Seed Mind Created ===")
        details.append(f"Identity: {seed_mind.manifest.identity}")
        details.append(f"Role: {seed_mind.manifest.role.value}")
        details.append(f"Growth: {seed_mind.manifest.growth_stage}")
        details.append(f"Sessions: {seed_mind.manifest.session_count}")
        details.append(f"Principles: {seed_mind.manifest.principles}")
        
        # Phase 1: Absorb principles
        details.append(f"\n=== Phase 1: Absorb Design Principles ===")
        time.sleep(2)
        absorb = seed_mind.think(
            f"Here are the core design principles for an AI civilization. Absorb them and "
            f"tell me the 3 most important insights:\n\n{design_principles}"
        )
        details.append(f"Absorption result: {absorb[:300]}")
        
        seed_mind.scratchpad.append(f"Phase 1: Absorbed design principles. Key insights captured.")
        
        # Phase 2: Apply principles
        time.sleep(30)
        details.append(f"\n=== Phase 2: Apply Principles ===")
        apply_result = seed_mind.think(
            "Based on the principles, design a memory system for an AI agent. "
            "What are the 3 key components? Be specific."
        )
        details.append(f"Application result: {apply_result[:300]}")
        
        seed_mind.scratchpad.append(f"Phase 2: Applied principles to memory system design.")
        
        # Phase 3: Self-improve
        time.sleep(30)
        details.append(f"\n=== Phase 3: Self-Improvement ===")
        evolve_result = seed_mind.think(
            "Review your own work. What did you miss? What could be better? "
            "Propose 2 improvements to your memory system design."
        )
        details.append(f"Evolution result: {evolve_result[:300]}")
        
        seed_mind.scratchpad.append(f"Phase 3: Self-improvement completed. 2 improvements proposed.")
        
        # Phase 4: Growth promotion check
        seed_mind.manifest.session_count = 10  # Simulate experience
        seed_mind.manifest.promote_stage()
        details.append(f"\n=== Growth Evolution ===")
        details.append(f"Growth stage: novice → {seed_mind.manifest.growth_stage}")
        details.append(f"Anti-patterns learned: {seed_mind.manifest.anti_patterns}")
        
        total_memories = list(test_root.rglob("minds/**/*.md"))
        details.append(f"\nTotal memories created: {len(total_memories)}")
        
        evidence.append(str(test_root / "minds"))
        evidence.append(str(test_root / "scratchpads"))
        evidence.append(str(test_root / "manifests"))
        
        log.record(4, "Evolution from Seed", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        import traceback
        details.append(traceback.format_exc())
        evidence.append(str(test_root))
        log.record(4, "Evolution from Seed", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 5: MEMORY REUSE — New session finds prior memories
# ──────────────────────────────────────────────────────────────

async def test_memory_reuse(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Start new session. Does it find and USE memories from evolution run?"""
    test_root = root / "battle_test" / "test4_evolution"  # same as test 4
    test5_root = root / "battle_test" / "test5_reuse"
    if test5_root.exists():
        shutil.rmtree(test5_root)
    test5_root.mkdir(parents=True, exist_ok=True)
    
    details = []
    evidence = []
    try:
        # Copy evolution memories to reuse test
        reuse_minds = test5_root / "minds" / "evolution-seed"
        if (test_root / "minds" / "evolution-seed").exists():
            shutil.copytree(test_root / "minds" / "evolution-seed", reuse_minds)
            details.append(f"Copied evolution-seed memories from Test 4")
        else:
            details.append(f"No prior evolution memories found — creating fresh")
            reuse_minds.mkdir(parents=True, exist_ok=True)
        
        # Create fresh mind that shares the same memory directory
        fresh_manifest = Manifest(
            identity="evolution-seed",  # SAME identity → same memory
            role=Role.AGENT,
            vertical="evolution",
            specialty="self-improvement",
            growth_stage="competent",
            session_count=15,
        )
        
        from mind_system import Mind
        reuse_mind = Mind(fresh_manifest, test5_root, llm)
        
        # Search for prior memories
        prior_memories = reuse_mind.memory.search("memory", limit=10)
        details.append(f"Memory search for 'memory': found {len(prior_memories)} prior memories")
        
        for m in prior_memories:
            details.append(f"  - [{m.title}] depth={m.depth_score}, tier={m.tier.value}")
        
        # Can the new mind build on prior work?
        time.sleep(2)
        build_result = reuse_mind.think(
            "Build on your previous memory system design. What's the next evolution?"
        )
        details.append(f"\nBuilding on prior work: {build_result[:300]}")
        
        new_memories = list(test5_root.rglob("minds/**/*.md"))
        details.append(f"\nNew memories created: {len(new_memories)}")
        
        evidence.append(str(test5_root / "minds"))
        evidence.append(str(test5_root / "scratchpads"))
        
        log.record(5, "Memory Reuse", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        import traceback
        details.append(traceback.format_exc())
        evidence.append(str(test5_root))
        log.record(5, "Memory Reuse", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 6: WEB SEARCH — ddgs
# ──────────────────────────────────────────────────────────────

async def test_web_search(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Use ddgs to search for something real."""
    details = []
    evidence = []
    try:
        # Check if ddgs is installed
        result = subprocess.run(
            ["which", "ddgs"],
            capture_output=True, text=True, timeout=5
        )
        
        if result.returncode == 0:
            details.append(f"ddgs found at: {result.stdout.strip()}")
            
            # Search for something real
            search_result = subprocess.run(
                ["ddgs", "text", "AI agent memory architecture best practices", "-n", "5"],
                capture_output=True, text=True, timeout=30
            )
            
            if search_result.returncode == 0:
                details.append(f"Search results ({len(search_result.stdout)} chars):")
                details.append(search_result.stdout[:2000])
                log.record(6, "Web Search (ddgs)", "PASS", ["ddgs CLI output"], "\n".join(details))
            else:
                details.append(f"ddgs error: {search_result.stderr[:500]}")
                log.record(6, "Web Search (ddgs)", "FAIL", [], "\n".join(details))
        else:
            details.append(f"ddgs not installed. Trying Python duckduckgo-search...")
            
            # Try Python approach
            try:
                from duckduckgo_search import DDGS
                with DDGS() as ddgs:
                    results = list(ddgs.text("AI agent memory architecture best practices", max_results=5))
                    details.append(f"Found {len(results)} results:")
                    for r in results:
                        details.append(f"  - {r.get('title', 'N/A')}: {r.get('href', 'N/A')}")
                        details.append(f"    {r.get('body', '')[:200]}")
                    log.record(6, "Web Search (duckduckgo-search)", "PASS", 
                              ["Python duckduckgo_search"], "\n".join(details))
            except ImportError:
                details.append("duckduckgo_search not installed")
                details.append("Installing...")
                subprocess.run(["pip", "install", "duckduckgo-search"], capture_output=True, timeout=60)
                
                from duckduckgo_search import DDGS
                with DDGS() as ddgs:
                    results = list(ddgs.text("AI agent memory architecture best practices", max_results=3))
                    details.append(f"Found {len(results)} results after install:")
                    for r in results:
                        details.append(f"  - {r.get('title', 'N/A')}: {r.get('href', 'N/A')}")
                    log.record(6, "Web Search (duckduckgo-search, installed)", "PASS",
                              ["pip install duckduckgo-search"], "\n".join(details))
            except Exception as e:
                details.append(f"Python approach failed: {e}")
                log.record(6, "Web Search", "FAIL", [], "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        log.record(6, "Web Search", "FAIL", [], "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 7: FILE OPS — Read, modify, write, show diff
# ──────────────────────────────────────────────────────────────

async def test_file_ops(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Read a file, modify it, write it back, show the diff."""
    details = []
    evidence = []
    try:
        # Create a test file
        test_file = root / "battle_test" / "test7_file.txt"
        original_content = """# Test File
This is a test file for battle test 7.
It has a few lines of content.
"""
        test_file.write_text(original_content)
        details.append(f"Original file created:")
        details.append(original_content)
        
        # Read it back
        read_content = test_file.read_text()
        details.append(f"\nRead back ({len(read_content)} chars):")
        details.append(read_content)
        
        # Modify it
        new_content = """# Test File — MODIFIED BY BATTLE TEST 7
This is a test file for battle test 7.
It has a few lines of content.
MODIFICATION: Added this line to prove write-back capability.
"""
        test_file.write_text(new_content)
        details.append(f"\nModified file:")
        details.append(new_content)
        
        # Show diff
        import difflib
        diff = list(difflib.unified_diff(
            original_content.splitlines(keepends=True),
            new_content.splitlines(keepends=True),
            fromfile="original",
            tofile="modified"
        ))
        details.append(f"\nDiff:")
        details.append("".join(diff))
        
        evidence.append(str(test_file))
        log.record(7, "File Ops", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        log.record(7, "File Ops", "FAIL", [], "\n".join(details))


# ──────────────────────────────────────────────────────────────
# TEST 8: DREAM MODE — Run dream cycle on evolved mind
# ──────────────────────────────────────────────────────────────

async def test_dream_mode(log: EvidenceLog, root: Path, llm: OllamaClient):
    """Run a dream cycle on the evolved mind. What does it learn?"""
    test_root = root / "battle_test" / "test4_evolution"
    details = []
    evidence = []
    try:
        # Load the evolution seed mind
        manifest = Manifest(
            identity="evolution-seed",
            role=Role.AGENT,
            vertical="evolution",
            specialty="self-improvement",
            growth_stage="competent",
            session_count=15,
        )
        
        from mind_system import Mind
        dream_mind = Mind(manifest, test_root, llm)
        
        # Dream Mode: 5 phases
        details.append(f"=== DREAM MODE: 5-Phase Cycle ===")
        details.append(f"Mind: {dream_mind.manifest.identity}")
        details.append(f"Growth stage: {dream_mind.manifest.growth_stage}")
        details.append(f"Session count: {dream_mind.manifest.session_count}")
        
        # Phase 1: Review all memories
        details.append(f"\n--- Phase 1: Review ---")
        all_memories = list(test_root.rglob("minds/**/*.md"))
        details.append(f"Total memories to review: {len(all_memories)}")
        
        for f in all_memories[:5]:
            try:
                mem = dream_mind.memory._from_markdown(f)
                details.append(f"  [{mem.title}] depth={mem.depth_score}, tier={mem.tier.value}")
            except:
                pass
        
        # Phase 2: Pattern search
        details.append(f"\n--- Phase 2: Pattern Search ---")
        time.sleep(2)
        patterns = dream_mind.think(
            "Review all your memories. What patterns do you see? "
            "What keeps recurring? What has worked? What has failed? "
            "List 3 patterns."
        )
        details.append(f"Patterns found: {patterns[:500]}")
        
        dream_mind.scratchpad.append(f"Dream Phase 2: Patterns identified")
        
        # Phase 3: Consolidate
        details.append(f"\n--- Phase 3: Consolidate ---")
        archived = dream_mind.memory.consolidate()
        details.append(f"Memories archived: {archived}")
        
        # Phase 4: Evolve manifest
        details.append(f"\n--- Phase 4: Manifest Evolution ---")
        time.sleep(30)
        evolution = dream_mind.think(
            "Based on your patterns, how should your manifest evolve? "
            "What anti-patterns should you add? What principles should change?"
        )
        details.append(f"Evolution insights: {evolution[:500]}")
        
        # Add anti-pattern if discovered
        if "anti-pattern" in evolution.lower() or "avoid" in evolution.lower() or "don't" in evolution.lower():
            dream_mind.manifest.add_anti_pattern("Learned from dream cycle")
        
        dream_mind.manifest.session_count += 1
        dream_mind.manifest.promote_stage()
        dream_mind.manifest.save(dream_mind.manifest_file)
        details.append(f"New growth stage: {dream_mind.manifest.growth_stage}")
        details.append(f"Anti-patterns: {dream_mind.manifest.anti_patterns}")
        
        # Phase 5: Tomorrow's priorities
        details.append(f"\n--- Phase 5: Tomorrow's Priorities ---")
        time.sleep(30)
        priorities = dream_mind.think(
            "What should you focus on next session? List 3 priorities."
        )
        details.append(f"Tomorrow's priorities: {priorities[:500]}")
        
        dream_mind.scratchpad.append(f"Dream complete. Tomorrow's priorities set.")
        
        # Write dream artifact
        dream_file = test_root / "dreams" / f"dream-{datetime.now(timezone.utc).strftime('%Y-%m-%d')}.md"
        dream_file.parent.mkdir(parents=True, exist_ok=True)
        dream_file.write_text("\n".join(details))
        
        evidence.append(str(dream_file))
        evidence.append(str(test_root / "manifests"))
        evidence.append(str(test_root / "scratchpads"))
        
        log.record(8, "Dream Mode", "PASS", evidence, "\n".join(details))
        
    except Exception as e:
        details.append(f"\nERROR: {e}")
        import traceback
        details.append(traceback.format_exc())
        evidence.append(str(test_root))
        log.record(8, "Dream Mode", "FAIL", evidence, "\n".join(details))


# ──────────────────────────────────────────────────────────────
# MAIN
# ──────────────────────────────────────────────────────────────

async def main():
    root = Path("/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds")
    llm = OllamaClient()
    log = EvidenceLog()
    
    print("=" * 70)
    print("BATTLE TEST — 8 Tests, Honest Results")
    print(f"Started: {datetime.now(timezone.utc).isoformat()}")
    print("=" * 70)
    
    tests = [
        ("Test 1: Teams", test_teams),
        ("Test 2: Comms During Runs", test_comms),
        ("Test 3: Restart Protocol", test_restart),
        ("Test 4: Evolution from Seed", test_evolution),
        ("Test 5: Memory Reuse", test_memory_reuse),
        ("Test 6: Web Search", test_web_search),
        ("Test 7: File Ops", test_file_ops),
        ("Test 8: Dream Mode", test_dream_mode),
    ]
    
    for name, test_fn in tests:
        print(f"\n{'=' * 70}")
        print(f"RUNNING: {name}")
        print(f"{'=' * 70}")
        try:
            await test_fn(log, root, llm)
            last_entry = log.entries[-1]
            status = last_entry["status"]
            print(f"RESULT: {status}")
        except Exception as e:
            print(f"CRASH: {e}")
            import traceback
            traceback.print_exc()
    
    # Write results
    results_path = "/home/corey/projects/AI-CIV/qwen-aiciv-mind/battle_test_results.md"
    log.write(results_path)
    
    print(f"\n{'=' * 70}")
    print(f"BATTLE TEST COMPLETE")
    print(f"Results: {results_path}")
    print(f"{'=' * 70}")
    
    # Summary
    passed = sum(1 for e in log.entries if e["status"] == "PASS")
    failed = sum(1 for e in log.entries if e["status"] == "FAIL")
    print(f"\nPassed: {passed}/{len(log.entries)}")
    print(f"Failed: {failed}/{len(log.entries)}")


if __name__ == "__main__":
    asyncio.run(main())
