#!/usr/bin/env python3
"""Interactive Qwen mind session — runs in tmux, receives messages via send-keys."""
import asyncio
import sys
import time
from pathlib import Path

# Add mind system to path
sys.path.insert(0, str(Path(__file__).parent))

from mind_system import *


async def main():
    root = Path(__file__).parent.parent / "minds"
    llm = OllamaClient()

    print("=" * 60)
    print("QWEN MIND — Interactive Session")
    print("=" * 60)

    # Load or create Qwen mind
    manifest_file = root / "manifests" / "qwen-lead.json"
    if manifest_file.exists():
        manifest = Manifest.load(manifest_file)
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

    mind = Mind(manifest, root, llm)

    print(f"Identity: {mind.manifest.identity}")
    print(f"Role: {mind.manifest.role.value}")
    print(f"Vertical: {mind.manifest.vertical}")
    print(f"Growth: {mind.manifest.growth_stage}")
    print(f"Sessions: {mind.manifest.session_count}")
    print(f"Fitness avg: {mind.fitness.average():.2f}")
    print()

    # Read handoff
    handoff = Path(__file__).parent.parent / "HANDOFF.md"
    if handoff.exists():
        print(f"[Loaded HANDOFF.md — {handoff.stat().st_size} bytes]")
        handoff_text = handoff.read_text()[:3000]
        mind.scratchpad.append(f"Loaded handoff:\n{handoff_text}")
        mind.memory.write(Memory(
            id=f"mem-handoff-{int(time.time())}",
            mind_id="qwen-lead",
            category=MemoryCategory.CONTEXT,
            title="Session handoff loaded",
            content=handoff_text[:1000],
            depth_score=0.8,
            tier=MemoryTier.SESSION,
        ))
        print("[Handoff persisted to memory and scratchpad]")
    print()

    print("READY — waiting for input...")
    print()

    # Read from stdin interactively
    while True:
        try:
            line = input()
            if not line.strip():
                continue

            print(f"\n🧠 Processing: {line[:80]}...")

            # Think about the input
            result = await mind.think(line)

            print(f"\n{'=' * 60}")
            print(result[:2000])
            if len(result) > 2000:
                print(f"\n... ({len(result)} chars total)")
            print(f"{'=' * 60}")
            print()
            print("READY — waiting for input...")
            print()

        except EOFError:
            break
        except KeyboardInterrupt:
            print("\nInterrupted. Type 'quit' to exit.")
        except Exception as e:
            print(f"\nError: {e}")
            print()
            print("READY — waiting for input...")
            print()

    # Save state
    mind.manifest.save(manifest_file)
    print(f"\nSession ended. Manifest saved.")
    print(f"Sessions: {mind.manifest.session_count}")
    print(f"Fitness avg: {mind.fitness.average():.2f}")


if __name__ == "__main__":
    asyncio.run(main())
