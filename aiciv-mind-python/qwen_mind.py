#!/usr/bin/env python3
"""Qwen Team Lead — Persistent mind session in tmux.

Reads mandatory context files, loads memory, then loops:
- Reads input from stdin (tmux send-keys injects here)
- Thinks via Ollama API
- Writes memory and scratchpad
- Returns response

Usage: python3 qwen_mind.py
"""
import asyncio
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from mind_system import *

ROOT = Path(__file__).parent.parent / "minds"
MANDATORY_FILES = [
    ".claude/team-leads/qwen/memory.md",
    "MISSIONS.md",
    "GRAND-PLAN.md",
    "QWEN-STATUS-REPORT.md",
    "HANDOFF-RESTART.md",
]

PROJECT_ROOT = Path(__file__).parent.parent


async def main():
    print("=" * 70)
    print("QWEN MIND — Persistent Session")
    print("=" * 70)
    print()

    # ── Step 1: Read mandatory context files ──
    print("[Loading mandatory context...]")
    context = {}
    for f in MANDATORY_FILES:
        path = PROJECT_ROOT / f
        if path.exists():
            content = path.read_text()
            context[f] = content
            print(f"  ✅ {f} ({len(content)} bytes)")
        else:
            print(f"  ❌ {f} NOT FOUND")

    print()

    # ── Step 2: Load or create mind identity ──
    manifest_file = ROOT / "manifests" / "qwen-lead.json"
    if manifest_file.exists():
        manifest = Manifest.load(manifest_file)
        print(f"[Loaded manifest: {manifest.identity}, {manifest.growth_stage}, {manifest.session_count} sessions]")
    else:
        manifest = Manifest(
            identity="qwen-lead",
            role=Role.TEAM_LEAD,
            vertical="qwen",
            principles=[
                "Memory IS architecture",
                "System > Symptom",
                "Go slow to go fast",
                "I do not do things — I form orchestras that do things",
            ],
        )
        manifest.save(manifest_file)
        print(f"[Created fresh manifest]")

    # ── Step 3: Initialize mind ──
    llm = OllamaClient()
    mind = Mind(manifest, ROOT, llm)

    # ── Step 4: Load memory context ─–
    memories = mind.memory.search("Qwen mind system architecture", limit=5)
    if memories:
        print(f"[Found {len(memories)} relevant memories]")
    else:
        print("[No past memories found — starting fresh]")

    # ─– Step 5: Write session start to scratchpad ─–
    mind.scratchpad.write(
        f"# Qwen Team Lead — Session Start\n"
        f"Started: {datetime.now(timezone.utc).isoformat()}\n"
        f"Identity: {mind.manifest.identity}\n"
        f"Growth: {mind.manifest.growth_stage}\n"
        f"Sessions: {mind.manifest.session_count}\n"
        f"Fitness avg: {mind.fitness.average():.2f}\n\n"
        f"## Mandatory Context Loaded\n"
        f"{'  ✅ ' + chr(10).join(f'  ✅ ' for _ in context)}\n"
        f"{'  ✅ ' + chr(10).join(f for f in context.keys())}\n"
    )
    print(f"[Scratchpad initialized for today]")

    # ── Step 6: Write first memory ─–
    mind.memory.write(Memory(
        id=f"mem-session-start-{int(time.time())}",
        mind_id="qwen-lead",
        category=MemoryCategory.CONTEXT,
        title="Session started — mandatory context loaded",
        content=f"Loaded {len(context)} context files. Ready to receive tasks.",
        depth_score=0.5,
        tier=MemoryTier.SESSION,
    ))
    print(f"[Memory written: session start]")

    # ── Step 7: Ready for input ─–
    print()
    print("=" * 70)
    print("READY — waiting for input from ACG or Primary")
    print("=" * 70)
    print()

    # ── Step 8: Main loop ─–
    while True:
        try:
            line = input()
            if not line.strip():
                continue

            # Check for commands
            if line.strip().lower() == "quit":
                print("Saving state and exiting...")
                mind.manifest.save(manifest_file)
                break

            if line.strip().lower() == "status":
                print(f"Identity: {mind.manifest.identity}")
                print(f"Role: {mind.manifest.role.value}")
                print(f"Growth: {mind.manifest.growth_stage}")
                print(f"Sessions: {mind.manifest.session_count}")
                print(f"Fitness: {mind.fitness.average():.2f}")
                memories_count = sum(1 for _ in ROOT.rglob("minds/**/*.md"))
                print(f"Total memories: {memories_count}")
                print()
                continue

            print(f"\n🧠 Processing: {line[:80]}...")

            # Think about the input
            result = await mind.think(line)

            print(f"\n{'=' * 60}")
            print(result[:3000])
            if len(result) > 3000:
                print(f"\n... ({len(result)} chars total)")
            print(f"{'=' * 60}")
            print()
            print("READY — waiting for input...")
            print()

        except EOFError:
            break
        except KeyboardInterrupt:
            print("\nInterrupted. Saving state...")
            mind.manifest.save(manifest_file)
            break
        except Exception as e:
            print(f"\nError: {e}")
            import traceback
            traceback.print_exc()
            print()
            print("READY — waiting for input...")
            print()

    # Final save
    mind.manifest.save(manifest_file)
    print(f"\nSession ended. Manifest saved.")
    print(f"Sessions: {mind.manifest.session_count}")
    print(f"Fitness avg: {mind.fitness.average():.2f}")


if __name__ == "__main__":
    asyncio.run(main())
