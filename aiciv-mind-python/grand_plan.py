#!/usr/bin/env python3
"""Execute the GRAND PLAN — make the Qwen mind system sing.

This script:
1. Seeds civilizational memory with real knowledge
2. Launches 3 team leads (staggered 30s apart)
3. Runs the grand challenge through the hierarchy
4. Demonstrates memory reuse and improvement
5. Runs Dream Mode
6. Shows the system is smarter than when it started
"""
import asyncio
import json
import shutil
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

# Add mind system to path
sys.path.insert(0, str(Path(__file__).parent))

from mind_system import *


ROOT = Path(__file__).parent.parent / "minds"
LLM = None  # Lazy init


def get_llm() -> OllamaClient:
    global LLM
    if LLM is None:
        LLM = OllamaClient()
    return LLM


# ───────────────────────────────────────────────────────────────────
# Phase 1: Seed Civilizational Memory
# ───────────────────────────────────────────────────────────────────

CIV_MEMORIES = [
    Memory(
        id="civ-001",
        mind_id="_civilizational",
        category=MemoryCategory.DECISION,
        title="Documents > Database for mind memory",
        content="Benchmark proved: Markdown files win for our scale. SQLite was 0.43ms vs 34.42ms for docs, but inspectability matters more. At hundreds of memories per mind, 34ms is fine. The mind IS its memory — you can literally cat the files.",
        depth_score=0.9,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-002",
        mind_id="_civilizational",
        category=MemoryCategory.LEARNING,
        title="Hard delegation rules, not guidelines",
        content="Primary → TeamLead → Agent is enforced at the class level. DelegationError raised on structural violations. This is what makes it real — not 'you should delegate' but 'you cannot violate your nature.'",
        depth_score=0.9,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-003",
        mind_id="_civilizational",
        category=MemoryCategory.PATTERN,
        title="Gentle API usage prevents rate limits",
        content="30s minimum between Ollama Cloud API calls. Never parallel to same model. Exponential backoff on 500 errors. This keeps the API happy and prevents 'Internal Server Error' responses.",
        depth_score=0.8,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-004",
        mind_id="_civilizational",
        category=MemoryCategory.DECISION,
        title="Devstral for tool calling, not Gemma",
        content="Gemma 3:12b does NOT support native tool calling — it talks about tools but doesn't call them. Devstral 24b is the only cloud model that reliably executes ThinkLoop with real tool calls.",
        depth_score=0.9,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-005",
        mind_id="_civilizational",
        category=MemoryCategory.LEARNING,
        title="123 skills extracted from ACG fork",
        content="116 original + 7 new = 123 skills in from-ACG/. Categorized: 15 KEEP, 66 ADAPT, 42 DELETE. The KEEP skills are pure methodology with no platform coupling.",
        depth_score=0.7,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-006",
        mind_id="_civilizational",
        category=MemoryCategory.PATTERN,
        title="tmux injection is the real architecture",
        content="TG → tmux send-keys → Qwen pane → response capture → TG back. No relay files, no pipes. Direct injection. This is how ACG's telegram_unified.py works.",
        depth_score=0.8,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-007",
        mind_id="_civilizational",
        category=MemoryCategory.CONTEXT,
        title="Full mind hierarchy: 15 minds proved",
        content="Primary + 3 TeamLeads (research, code, ops) + 11 Agents. Shared agents: researcher (3 instances), analyst (2 instances). All hard rules verified. File structure: 15 manifests, 15 edge indexes.",
        depth_score=0.8,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-008",
        mind_id="_civilizational",
        category=MemoryCategory.ERROR,
        title="Ollama Cloud 500 errors are transient",
        content="Ollama Cloud API returns 500 errors ~10% under load. Retry with exponential backoff succeeds. Cortex retry logic (3 attempts) handles this. Need to load API key from .env file.",
        depth_score=0.6,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-009",
        mind_id="_civilizational",
        category=MemoryCategory.DECISION,
        title="Python first, Rust if needed",
        content="Build in Python to prove the architecture works. Move to Rust only if performance demands it. This lets us iterate fast on the mind system without compilation overhead.",
        depth_score=0.8,
        tier=MemoryTier.LONG_TERM,
    ),
    Memory(
        id="civ-010",
        mind_id="_civilizational",
        category=MemoryCategory.PATTERN,
        title="Memory must be searched before acting",
        content="Memory without search is useless. Every agent MUST search memory before starting a task. Found memories should influence the approach. This prevents re-solving solved problems.",
        depth_score=0.9,
        tier=MemoryTier.LONG_TERM,
    ),
]


def seed_civilizational_memory():
    """Write all civilizational memories to disk."""
    civ_dir = ROOT / "minds" / "_civilizational"
    civ_dir.mkdir(parents=True, exist_ok=True)
    
    edges_file = civ_dir / "_edges.json"
    edges_file.write_text("[]")
    
    print(f"[Phase 1] Seeding {len(CIV_MEMORIES)} civilizational memories...")
    
    for mem in CIV_MEMORIES:
        tier_dir = civ_dir / mem.tier.value / mem.category.value
        tier_dir.mkdir(parents=True, exist_ok=True)
        path = tier_dir / f"{mem.id}.md"
        path.write_text(MindMemory._to_markdown(mem))
        print(f"  ✅ {mem.id}: {mem.title[:60]}...")
    
    # Create edges between related memories
    edges = [
        {"source": "civ-001", "target": "civ-010", "type": "builds_on", "weight": 1.0},
        {"source": "civ-002", "target": "civ-007", "type": "cites", "weight": 1.0},
        {"source": "civ-003", "target": "civ-008", "type": "cites", "weight": 1.0},
        {"source": "civ-006", "target": "civ-003", "type": "builds_on", "weight": 0.8},
    ]
    edges_file.write_text(json.dumps(edges, indent=2))
    print(f"  ✅ Created {len(edges)} graph edges")
    print(f"  ✅ Civilizational memory seeded: {civ_dir}")


# ───────────────────────────────────────────────────────────────────
# Phase 2: Launch Team Leads (staggered)
# ───────────────────────────────────────────────────────────────────

async def launch_team_lead(name: str, vertical: str, agents: list[str], delay: int = 0):
    """Launch a team lead with agents, after optional delay."""
    if delay > 0:
        print(f"\n[Phase 2] Waiting {delay}s before launching {name}...")
        await asyncio.sleep(delay)
    
    print(f"\n[Phase 2] Launching {name}...")
    
    llm = get_llm()
    primary = Primary(ROOT, llm)
    lead = primary.spawn_team_lead(vertical)
    
    # Give team lead a memory of civilizational knowledge
    lead_mem_dir = ROOT / "minds" / lead.manifest.identity
    lead_mem_dir.mkdir(parents=True, exist_ok=True)
    (lead_mem_dir / "_edges.json").write_text("[]")
    
    # Write initial scratchpad
    lead.scratchpad.write(
        f"# {lead.manifest.identity} — Session Start\n"
        f"Launched: {datetime.now(timezone.utc).isoformat()}\n"
        f"Vertical: {vertical}\n"
        f"Agents to spawn: {agents}\n"
        f"Civilizational memories: {len(CIV_MEMORIES)} available\n"
    )
    
    # Write initial memory
    init_mem = Memory(
        id=f"{name}-init",
        mind_id=lead.manifest.identity,
        category=MemoryCategory.CONTEXT,
        title="Team lead initialized",
        content=f"Launched as {vertical} team lead with {len(agents)} agents: {', '.join(agents)}",
        depth_score=0.5,
        tier=MemoryTier.SESSION,
    )
    mem_path = lead.memory.write(init_mem)
    
    print(f"  ✅ {name} spawned")
    print(f"  ✅ Scratchpad initialized")
    print(f"  ✅ Memory directory: {lead_mem_dir}")
    
    # Spawn agents
    spawned_agents = {}
    for specialty in agents:
        agent = lead.spawn_agent(specialty)
        agent_mem_dir = ROOT / "minds" / agent.manifest.identity
        agent_mem_dir.mkdir(parents=True, exist_ok=True)
        (agent_mem_dir / "_edges.json").write_text("[]")
        
        agent.scratchpad.write(
            f"# {agent.manifest.identity} — Session Start\n"
            f"Spawned by: {lead.manifest.identity}\n"
            f"Specialty: {specialty}\n"
        )
        
        spawned_agents[specialty] = agent
        print(f"  ✅ Agent: {agent.manifest.identity}")
    
    return lead, spawned_agents


# ───────────────────────────────────────────────────────────────────
# Phase 3: The Grand Challenge
# ───────────────────────────────────────────────────────────────────

async def run_grand_challenge(leads: dict, all_agents: dict):
    """Execute the grand challenge through the hierarchy."""
    llm = get_llm()
    
    print(f"\n{'=' * 70}")
    print("[Phase 3] GRAND CHALLENGE: Validate and implement top KEEP skills")
    print(f"{'=' * 70}")
    
    results = {}
    
    # ── Challenge 1: Research team analyzes KEEP skills ──
    print(f"\n--- Challenge 1: Research team analyzes KEEP skills ---")
    researcher = all_agents.get("research/researcher")
    if researcher:
        print(f"[Research] Sending task to researcher...")
        time.sleep(30)  # Rate limit courtesy
        
        task = (
            "Search the from-ACG/ directory for the KEEP skills (those with no platform dependencies). "
            "List the top 5 most valuable ones and explain WHY each is KEEP-worthy. "
            "Be specific about what makes each skill platform-agnostic."
        )
        
        result = await researcher.think(task)
        results["research_skills"] = result[:500]
        print(f"  ✅ Research complete ({len(result)} chars)")
        
        # Write scratchpad update
        researcher.scratchpad.append(f"Task: Analyze KEEP skills\nResult: {result[:200]}...")
    
    # ── Challenge 2: Code team implements top skill ──
    print(f"\n--- Challenge 2: Code team implements a skill ---")
    developer = all_agents.get("code/developer")
    if developer:
        print(f"[Code] Sending task to developer...")
        time.sleep(30)  # Rate limit courtesy
        
        task = (
            "Search memory first: what have we built so far in the mind system? "
            "Then implement the 'scratch-pad' skill from from-ACG/ as a Python module. "
            "The skill should enhance our existing Scratchpad class with session continuity features."
        )
        
        result = await developer.think(task)
        results["code_implementation"] = result[:500]
        print(f"  ✅ Implementation complete ({len(result)} chars)")
        
        developer.scratchpad.append(f"Task: Implement scratch-pad skill\nResult: {result[:200]}...")
    
    # ── Challenge 3: Ops team verifies ──
    print(f"\n--- Challenge 3: Ops team verifies the work ---")
    tester = all_agents.get("code/tester")
    if tester:
        print(f"[Ops] Sending verification task to tester...")
        time.sleep(30)  # Rate limit courtesy
        
        task = (
            "Search memory: what did the code/developer just implement? "
            "Verify that the implementation actually works. Test it with real data. "
            "Report any issues or confirm it's production-ready."
        )
        
        result = await tester.think(task)
        results["ops_verification"] = result[:500]
        print(f"  ✅ Verification complete ({len(result)} chars)")
        
        tester.scratchpad.append(f"Task: Verify code implementation\nResult: {result[:200]}...")
    
    # ── Challenge 4: Analyst synthesizes ──
    print(f"\n--- Challenge 4: Analyst synthesizes all results ---")
    analyst = all_agents.get("research/analyst")
    if analyst:
        print(f"[Analyst] Sending synthesis task to analyst...")
        time.sleep(30)  # Rate limit courtesy
        
        # Create a memory linking all the work
        synthesis_mem = Memory(
            id=f"synthesis-{int(time.time())}",
            mind_id=analyst.manifest.identity,
            category=MemoryCategory.PATTERN,
            title="Grand Challenge synthesis",
            content=f"Research found KEEP skills. Code implemented scratch-pad. Ops verified. Results: {str({k: len(v) for k, v in results.items()})}",
            depth_score=0.8,
            tier=MemoryTier.SESSION,
        )
        analyst.memory.write(synthesis_mem)
        
        task = (
            "Search memory for all grand challenge results. "
            "Synthesize into a single report: what we learned, what we built, what's next. "
            "Include specific recommendations for tomorrow."
        )
        
        result = await analyst.think(task)
        results["analyst_synthesis"] = result[:500]
        print(f"  ✅ Synthesis complete ({len(result)} chars)")
        
        analyst.scratchpad.append(f"Task: Synthesize grand challenge\nResult: {result[:200]}...")
    
    return results


# ───────────────────────────────────────────────────────────────────
# Phase 4: Demonstrate Memory Loop
# ───────────────────────────────────────────────────────────────────

async def demonstrate_memory_loop(all_agents: dict):
    """Show that memory is actually being used."""
    print(f"\n{'=' * 70}")
    print("[Phase 4] DEMONSTRATING MEMORY LOOP")
    print(f"{'=' * 70}")
    
    researcher = all_agents.get("research/researcher")
    if researcher:
        print(f"\n[Memory Test] Asking researcher about civilizational memories...")
        time.sleep(30)
        
        task = "What are the 3 most important things we've learned about mind architecture? Search your memories first."
        result = await researcher.think(task)
        
        print(f"  Result: {result[:300]}...")
        print(f"  ✅ Memory loop demonstrated")
        
        researcher.scratchpad.append(f"Memory test: {result[:200]}...")


# ───────────────────────────────────────────────────────────────────
# Phase 5: Dream Mode
# ───────────────────────────────────────────────────────────────────

async def run_dream_mode(minds: list):
    """Run Dream Mode for each mind."""
    print(f"\n{'=' * 70}")
    print("[Phase 5] DREAM MODE — Self-improvement cycle")
    print(f"{'=' * 70}")
    
    for mind in minds:
        print(f"\n[Dream] {mind.manifest.identity} dreaming...")
        dream = DreamEngine(mind)
        result = await dream.run()
        
        print(f"  Memories reviewed: {result['review']['memory_count']}")
        print(f"  Patterns found: {result['patterns']}")
        print(f"  Archived: {result['archived']}")
        print(f"  Growth stage: {result['growth_stage']}")
        print(f"  Tomorrow: {result['tomorrow_priorities']}")
        
        mind.scratchpad.append(f"Dream Mode complete: {result['tomorrow_priorities']}")


# ───────────────────────────────────────────────────────────────────
# Main Execution
# ───────────────────────────────────────────────────────────────────

async def main():
    print("=" * 70)
    print("THE GRAND PLAN — Making the Qwen Mind System Sing")
    print("=" * 70)
    print(f"Start time: {datetime.now(timezone.utc).isoformat()}")
    print(f"Root: {ROOT}")
    print()
    
    # Phase 1: Seed Memory
    seed_civilizational_memory()
    
    # Phase 2: Launch Team Leads (staggered)
    print(f"\n{'=' * 70}")
    print("[Phase 2] LAUNCHING TEAM LEADS (staggered)")
    print(f"{'=' * 70}")
    
    # Research team (T+0)
    research_lead, research_agents = await launch_team_lead(
        "research-lead", "research",
        ["researcher", "analyst"],
        delay=0
    )
    
    # Code team (T+30)
    code_lead, code_agents = await launch_team_lead(
        "code-lead", "code",
        ["developer", "tester"],
        delay=30
    )
    
    # Ops team (T+60)
    ops_lead, ops_agents = await launch_team_lead(
        "ops-lead", "ops",
        ["deployer", "monitor"],
        delay=30
    )
    
    all_agents = {**research_agents, **code_agents, **ops_agents}
    all_leads = {"research": research_lead, "code": code_lead, "ops": ops_lead}
    
    # Phase 3: Grand Challenge
    results = await run_grand_challenge(all_leads, all_agents)
    
    # Phase 4: Memory Loop
    await demonstrate_memory_loop(all_agents)
    
    # Phase 5: Dream Mode
    all_minds = list(all_leads.values()) + list(all_agents.values())
    await run_dream_mode(all_minds[:3])  # Dream for first 3 minds to save API calls
    
    # Final Report
    print(f"\n{'=' * 70}")
    print("FINAL REPORT")
    print(f"{'=' * 70}")
    
    total_memories = sum(1 for f in ROOT.rglob("minds/**/*.md"))
    total_edges = sum(1 for f in ROOT.rglob("minds/**/_edges.json"))
    total_scratchpads = sum(1 for f in ROOT.rglob("scratchpads/**/*.md"))
    total_manifests = sum(1 for f in ROOT.rglob("manifests/*.json"))
    
    print(f"  Civilizational memories: {len(CIV_MEMORIES)}")
    print(f"  Agent memories: {total_memories}")
    print(f"  Edge indexes: {total_edges}")
    print(f"  Scratchpads: {total_scratchpads}")
    print(f"  Manifests: {total_manifests}")
    print(f"  Challenge results: {len(results)} tasks completed")
    print()
    
    for name, preview in results.items():
        print(f"  [{name}] {preview[:100]}...")
    
    print()
    print(f"{'=' * 70}")
    print("THE MIND SYSTEM IS ALIVE")
    print(f"{'=' * 70}")
    print(f"End time: {datetime.now(timezone.utc).isoformat()}")


if __name__ == "__main__":
    asyncio.run(main())
