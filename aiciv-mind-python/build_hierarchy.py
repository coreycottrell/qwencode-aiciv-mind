#!/usr/bin/env python3
"""Build the full mind hierarchy: 3 Team Leads, 10 agents, some shared.

Architecture:
  Primary (conductor of conductors)
  ├── research-lead (coordinates research vertical)
  │   ├── researcher       (gathers information)
  │   ├── analyst          (analyzes data, patterns)
  │   └── hypothesis-tester (designs experiments)
  ├── code-lead (coordinates code/engineering)
  │   ├── developer        (writes code)
  │   ├── tester           (writes tests, verifies)
  │   ├── reviewer         (code review, quality)
  │   └── researcher       (SHARED — research technical topics)
  └── ops-lead (coordinates operations/infrastructure)
      ├── deployer         (deployments, config)
      ├── monitor          (health, alerts)
      ├── analyst          (SHARED — analyze metrics/logs)
      └── researcher       (SHARED — research ops topics)

Shared agents:
  - researcher → used by research, code, ops (different vertical context)
  - analyst → used by research, ops
"""
import asyncio
import shutil
import time
from pathlib import Path

from mind_system import *


async def main():
    root = Path("/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds")
    # Clean previous demo runs
    for d in ["primary", "research-lead", "code-lead", "ops-lead"]:
        if (root / "manifests" / d).exists():
            pass  # keep manifests from previous runs

    llm = OllamaClient()

    print("=" * 70)
    print("FULL MIND HIERARCHY — 3 Team Leads, 10 Agents (with sharing)")
    print("=" * 70)

    # ── Primary ──
    primary = Primary(root, llm)
    print(f"\n1. PRIMARY: {primary.manifest.identity}")
    print(f"   growth: {primary.manifest.growth_stage}, sessions: {primary.manifest.session_count}")

    # ── Team Leads ──
    research_lead = primary.spawn_team_lead("research")
    code_lead = primary.spawn_team_lead("code")
    ops_lead = primary.spawn_team_lead("ops")

    print(f"\n2. TEAM LEADS:")
    for lead in [research_lead, code_lead, ops_lead]:
        print(f"   {lead.manifest.identity}: vertical={lead.manifest.vertical}, "
              f"principles={len(lead.manifest.principles)}, anti-patterns={len(lead.manifest.anti_patterns)}")

    # ── Agents ──
    print(f"\n3. AGENTS:")

    # Research team
    r_researcher = research_lead.spawn_agent("researcher")
    r_analyst = research_lead.spawn_agent("analyst")
    r_hypothesis = research_lead.spawn_agent("hypothesis-tester")

    # Code team (developer, tester, reviewer + SHARED researcher)
    c_developer = code_lead.spawn_agent("developer")
    c_tester = code_lead.spawn_agent("tester")
    c_reviewer = code_lead.spawn_agent("reviewer")
    c_researcher = code_lead.spawn_agent("researcher")  # shared domain

    # Ops team (deployer, monitor + SHARED analyst + SHARED researcher)
    o_deployer = ops_lead.spawn_agent("deployer")
    o_monitor = ops_lead.spawn_agent("monitor")
    o_analyst = ops_lead.spawn_agent("analyst")  # shared domain
    o_researcher = ops_lead.spawn_agent("researcher")  # shared domain

    agents = {
        "research/researcher": r_researcher,
        "research/analyst": r_analyst,
        "research/hypothesis-tester": r_hypothesis,
        "code/developer": c_developer,
        "code/tester": c_tester,
        "code/reviewer": c_reviewer,
        "code/researcher": c_researcher,
        "ops/deployer": o_deployer,
        "ops/monitor": o_monitor,
        "ops/analyst": o_analyst,
        "ops/researcher": o_researcher,
    }

    for name, agent in agents.items():
        print(f"   {name}: specialty={agent.manifest.specialty}, "
              f"tools={len(agent.allowed_tools)}, growth={agent.manifest.growth_stage}")

    # ── Shared agent verification ──
    print(f"\n4. SHARED AGENTS:")
    shared = [
        ("researcher", ["research", "code", "ops"]),
        ("analyst", ["research", "ops"]),
    ]
    for specialty, verticals in shared:
        instances = [agents[f"{v}/{specialty}"] for v in verticals]
        print(f"   {specialty}: {len(instances)} instances")
        for inst in instances:
            print(f"     - {inst.manifest.identity} (vertical={inst.manifest.vertical})")

    # ── Hard rule verification ──
    print(f"\n5. HARD RULE VERIFICATION:")

    # Primary can only delegate to Team Leads
    assert primary.can_delegate_to(research_lead)
    assert primary.can_delegate_to(code_lead)
    assert primary.can_delegate_to(ops_lead)
    assert not primary.can_delegate_to(r_researcher)
    assert not primary.can_delegate_to(c_developer)
    print(f"   ✅ Primary → TeamLeads only (Agents blocked)")

    # Team Leads can only delegate to same-vertical Agents
    assert research_lead.can_delegate_to(r_researcher)
    assert research_lead.can_delegate_to(r_analyst)
    assert not research_lead.can_delegate_to(c_developer)
    assert not research_lead.can_delegate_to(o_deployer)
    print(f"   ✅ research-lead → research/* only")

    assert code_lead.can_delegate_to(c_developer)
    assert code_lead.can_delegate_to(c_researcher)
    assert not code_lead.can_delegate_to(r_researcher)  # different vertical
    print(f"   ✅ code-lead → code/* only")

    assert ops_lead.can_delegate_to(o_deployer)
    assert ops_lead.can_delegate_to(o_analyst)
    assert not ops_lead.can_delegate_to(r_analyst)
    print(f"   ✅ ops-lead → ops/* only")

    # Agents cannot spawn or delegate
    for name, agent in agents.items():
        try:
            agent.spawn_child("helper")
            raise AssertionError(f"{name} spawned child!")
        except DelegationError:
            pass
        try:
            agent.delegate(agent, "test")
            raise AssertionError(f"{name} delegated!")
        except DelegationError:
            pass
    print(f"   ✅ All {len(agents)} agents: cannot spawn, cannot delegate")

    # ── File structure ──
    print(f"\n6. MIND FILE STRUCTURE:")
    manifest_count = sum(1 for f in (root / "manifests").rglob("*.json"))
    mind_dir_count = sum(1 for f in root.rglob("minds/**/*") if f.is_file())
    print(f"   manifests: {manifest_count} files")
    print(f"   minds/: {mind_dir_count} edge index files")
    print(f"   Total minds: 1 Primary + 3 TeamLeads + {len(agents)} Agents = {1 + 3 + len(agents)}")

    print(f"\n{'=' * 70}")
    print(f"HIERARCHY COMPLETE — {1 + 3 + len(agents)} minds, all rules enforced")
    print(f"{'=' * 70}")


if __name__ == "__main__":
    asyncio.run(main())
