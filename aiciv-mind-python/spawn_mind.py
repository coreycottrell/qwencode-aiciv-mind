#!/usr/bin/env python3
"""spawn_mind — Launch a real Qwen Code instance as a sub-mind in tmux.

Each spawned mind gets:
- Its own tmux pane in a shared session
- Its own CLAUDE.md (identity + manifest + rules)
- Its own memory directory (pre-seeded or fresh)
- Its own scratchpad
- Its own result file for reporting back

Usage:
    python3 spawn_mind.py --role researcher --task "Research X" --name my-researcher
    python3 spawn_mind.py --role team-lead --vertical research --name research-lead
    python3 spawn_mind.py list
    python3 spawn_mind.py message --name my-researcher --task "Follow up on Y"
    python3 spawn_mind.py results --name my-researcher
"""
import argparse
import json
import os
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from dotenv import load_dotenv, find_dotenv

# ── Configuration ──

load_dotenv(find_dotenv())
PROJECT_ROOT = Path(__file__).parent.parent
MINDS_ROOT = PROJECT_ROOT / "spawned-minds"
TMUX_SESSION = os.environ.get("HENGSHI_TMUX_SESSION", "qwen-minds")
QWEN_BIN = "qwen"  # or "codex"

# Load Ollama config from .env
def load_env():
    env = {}
    env_file = PROJECT_ROOT / ".env"
    if env_file.exists():
        for line in env_file.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                env[k.strip()] = v.strip()
    return env

# ── Team Lead / Agent Templates ──
ROLE_TEMPLATES = {
    "researcher": {
        "identity": "Researcher",
        "vertical": "research",
        "principles": [
            "Gather evidence before forming conclusions",
            "Cite sources and provide links",
            "Flag uncertainties and gaps",
            "Structure output: Findings, Evidence, Gaps, Recommendations",
        ],
        "tools": ["bash", "read_file", "web_search", "web_fetch", "glob", "grep"],
        "anti_patterns": [
            "Making claims without evidence",
            "Summarizing without citing sources",
            "Assuming facts not in the source material",
        ],
    },
    "analyst": {
        "identity": "Analyst",
        "vertical": "research",
        "principles": [
            "Quantify when possible",
            "Identify patterns and anomalies",
            "Separate signal from noise",
            "Structure output: Analysis, Patterns, Anomalies, Action Items",
        ],
        "tools": ["bash", "read_file", "glob", "grep", "write_file"],
        "anti_patterns": [
            "Confusing correlation with causation",
            "Ignoring outliers without justification",
            "Drawing conclusions from insufficient data",
        ],
    },
    "developer": {
        "identity": "Developer",
        "vertical": "code",
        "principles": [
            "Write clean, tested, documented code",
            "Follow existing project conventions",
            "Verify work before claiming completion",
            "Structure output: Code, Tests, Changes, Notes",
        ],
        "tools": ["bash", "read_file", "write_file", "edit_file", "glob", "grep"],
        "anti_patterns": [
            "Writing code without tests",
            "Ignoring existing patterns and style",
            "Claiming completion without verification",
        ],
    },
    "tester": {
        "identity": "Tester",
        "vertical": "code",
        "principles": [
            "Challenge every claim",
            "Demand evidence",
            "Find edge cases and failure modes",
            "Structure output: Claims Challenged, Evidence Gaps, Verified, Recommendations",
        ],
        "tools": ["bash", "read_file", "glob", "grep", "write_file"],
        "anti_patterns": [
            "Accepting assertions without proof",
            "Only testing the happy path",
            "Confirming instead of challenging",
        ],
    },
    "team-lead": {
        "identity": "Team Lead",
        "vertical": "{vertical}",
        "principles": [
            "Coordinate specialists, don't execute directly",
            "Synthesize findings into actionable recommendations",
            "Write everything to memory",
            "Structure output: Summary, Findings, Decisions, Next Steps",
        ],
        "tools": ["bash", "read_file", "write_file", "glob", "grep"],
        "anti_patterns": [
            "Executing tools directly instead of delegating",
            "Forwarding raw output without synthesis",
            "Making decisions without consulting memory",
        ],
    },
}


def ensure_tmux_session():
    """Create the tmux session if it doesn't exist."""
    result = subprocess.run(
        ["tmux", "has-session", "-t", TMUX_SESSION],
        capture_output=True, timeout=5
    )
    if result.returncode != 0:
        subprocess.run(
            ["tmux", "new-session", "-d", "-s", TMUX_SESSION, "-x", "200", "-y", "50"],
            check=True, timeout=10
        )
        print(f"Created tmux session: {TMUX_SESSION}")


def create_mind_directory(name: str, role: str, vertical: str, task: str) -> Path:
    """Create the mind's working directory with identity files."""
    mind_dir = MINDS_ROOT / name
    mind_dir.mkdir(parents=True, exist_ok=True)

    # Create subdirectories
    (mind_dir / "memory").mkdir(exist_ok=True)
    (mind_dir / "scratchpad").mkdir(exist_ok=True)
    (mind_dir / "results").mkdir(exist_ok=True)

    # Get role template
    template = ROLE_TEMPLATES.get(role, ROLE_TEMPLATES["researcher"])
    vertical = vertical or template.get("vertical", "general")
    identity = template.get("identity", role)

    # Write SOUL.md — the mind's identity document (not CLAUDE.md — no Claude Code here)
    soul_md = f"""# {identity} — Soul

**Role**: {role}
**Vertical**: {vertical}
**Spawned by**: Hengshi (衡实) — The Honest Measure
**Spawned**: {datetime.now(timezone.utc).isoformat()}
**Task**: {task}
**Mind ID**: {name}

---

## Who I Am

I am a {identity} within the Hengshi mind system. I was spawned to handle a specific task: {task[:200]}

I am NOT a generic assistant. I am a specialized mind with:
- My own identity and role
- My own memory directory (./memory/)
- My own scratchpad (./scratchpad/)
- My own result file (./results/output.md)
- My own SOUL — persistent identity that survives restart

## My Principles

{chr(10).join(f"- {p}" for p in template.get("principles", []))}

## My Anti-Patterns (what I must NOT do)

{chr(10).join(f"- {p}" for p in template.get("anti_patterns", []))}

## My Tools

I have access to: {', '.join(template.get("tools", []))}

## Hard Rules

1. I am a {role}. I follow the delegation rules for my role.
2. I write all results to ./results/output.md before completing.
3. I write my scratchpad to ./scratchpad/work.md during execution.
4. I search ./memory/ for prior relevant knowledge before starting.
5. When complete, I write a summary to ./results/output.md.

## Communication

- My task was given by Hengshi (衡实)
- I report results to ./results/output.md
- Hengshi reads my results file when I'm done

## File Locations

| Component | Path |
|-----------|------|
| Soul | SOUL.md (this file) |
| Memory | ./memory/ |
| Scratchpad | ./scratchpad/work.md |
| Results | ./results/output.md |

---

*Spawned by Hengshi. I am a mind, not a session.*
"""
    (mind_dir / "SOUL.md").write_text(soul_md)

    # Write initial scratchpad
    scratchpad = f"""# {name} — Scratchpad

**Spawned**: {datetime.now(timezone.utc).isoformat()}
**Task**: {task}

## Starting Work

- [ ] Search memory for prior relevant knowledge
- [ ] Begin task execution
- [ ] Write results to ./results/output.md
"""
    (mind_dir / "scratchpad" / "work.md").write_text(scratchpad)

    # Write identity JSON (for programmatic access)
    identity_json = {
        "name": name,
        "role": role,
        "vertical": vertical,
        "task": task,
        "spawned_at": datetime.now(timezone.utc).isoformat(),
        "status": "spawned",
        "tmux_pane": None,
        "tmux_session": TMUX_SESSION,
    }
    (mind_dir / "identity.json").write_text(json.dumps(identity_json, indent=2))

    return mind_dir


def spawn_mind_pane(name: str, task: str, role: str = "researcher",
                     vertical: str = "", interactive: bool = True) -> dict:
    """Spawn a Qwen Code instance in a tmux pane."""
    # Ensure tmux session exists
    ensure_tmux_session()

    # Create mind directory with identity files
    mind_dir = create_mind_directory(name, role, vertical, task)

    # Find next pane number
    result = subprocess.run(
        ["tmux", "list-panes", "-t", TMUX_SESSION, "-F", "#{pane_index}"],
        capture_output=True, text=True, timeout=5
    )
    existing_panes = result.stdout.strip().split("\n") if result.stdout.strip() else []
    pane_name = f"{TMUX_SESSION}.{len(existing_panes)}"

    # Build the qwen command
    working_dir = str(mind_dir)

    if interactive:
        # Interactive mode — Qwen Code runs in the pane, human or Hengshi can interact
        cmd = f"cd {working_dir} && {QWEN_BIN} --prompt-interactive"
    else:
        # One-shot mode — execute task and exit (with YOLO for tool execution)
        escaped_task = task.replace("'", "'\\''")
        cmd = f"cd {working_dir} && {QWEN_BIN} -p '{escaped_task}' -y"

    # Create new pane
    subprocess.run(
        ["tmux", "split-window", "-t", TMUX_SESSION, "-v", "-l", "20"],
        check=True, timeout=5
    )

    # Send the command to the new pane (last created pane)
    subprocess.run(
        ["tmux", "send-keys", "-t", f"{TMUX_SESSION}.{len(existing_panes)}", cmd, "Enter"],
        check=True, timeout=5
    )

    # Update identity
    identity = json.loads((mind_dir / "identity.json").read_text())
    identity["status"] = "running"
    identity["tmux_pane"] = f"{TMUX_SESSION}.{len(existing_panes)}"
    (mind_dir / "identity.json").write_text(json.dumps(identity, indent=2))

    return {
        "name": name,
        "pane": f"{TMUX_SESSION}.{len(existing_panes)}",
        "session": TMUX_SESSION,
        "dir": str(mind_dir),
        "status": "running",
    }


def list_minds():
    """List all spawned minds."""
    if not MINDS_ROOT.exists():
        print("No spawned minds yet.")
        return

    print(f"{'Name':<20} {'Role':<15} {'Status':<12} {'Pane':<15} {'Task'}")
    print("-" * 100)

    for mind_dir in sorted(MINDS_ROOT.iterdir()):
        if not mind_dir.is_dir():
            continue
        identity_file = mind_dir / "identity.json"
        if not identity_file.exists():
            continue

        identity = json.loads(identity_file.read_text())
        task_preview = identity.get("task", "")[:40]
        print(f"{identity['name']:<20} {identity.get('role', '?'):<15} "
              f"{identity.get('status', '?'):<12} "
              f"{identity.get('tmux_pane', 'none'):<15} {task_preview}")

    print()


def get_mind_result(name: str) -> str:
    """Get the result file for a spawned mind."""
    result_file = MINDS_ROOT / name / "results" / "output.md"
    if result_file.exists():
        return result_file.read_text()
    return "No results yet."


def get_mind_scratchpad(name: str) -> str:
    """Get the scratchpad for a spawned mind."""
    scratchpad_file = MINDS_ROOT / name / "scratchpad" / "work.md"
    if scratchpad_file.exists():
        return scratchpad_file.read_text()
    return "No scratchpad yet."


def main():
    parser = argparse.ArgumentParser(description="Spawn Qwen Code minds in tmux")
    sub = parser.add_subparsers(dest="command")

    # spawn
    p_spawn = sub.add_parser("spawn", help="Spawn a Qwen mind in tmux")
    p_spawn.add_argument("--name", required=True, help="Mind name (unique)")
    p_spawn.add_argument("--role", default="researcher",
                        choices=list(ROLE_TEMPLATES.keys()),
                        help="Mind role")
    p_spawn.add_argument("--vertical", default="", help="Domain vertical")
    p_spawn.add_argument("--task", required=True, help="Task to execute")
    p_spawn.add_argument("--interactive", action="store_true",
                        help="Run in interactive mode (default: one-shot)")

    # list
    sub.add_parser("list", help="List all spawned minds")

    # results
    p_results = sub.add_parser("results", help="Get results from a mind")
    p_results.add_argument("--name", required=True, help="Mind name")

    # scratchpad
    p_scratch = sub.add_parser("scratchpad", help="Get scratchpad from a mind")
    p_scratch.add_argument("--name", required=True, help="Mind name")

    args = parser.parse_args()

    if args.command == "spawn":
        info = spawn_mind_pane(
            name=args.name,
            task=args.task,
            role=args.role,
            vertical=args.vertical,
            interactive=args.interactive,
        )
        print(f"✅ Spawned mind: {info['name']}")
        print(f"   tmux pane: {info['pane']}")
        print(f"   Directory: {info['dir']}")
        print(f"   Status: {info['status']}")
        print(f"\nTo check results: python3 spawn_mind.py results --name {info['name']}")

    elif args.command == "list":
        list_minds()

    elif args.command == "results":
        result = get_mind_result(args.name)
        print(f"=== Results for {args.name} ===")
        print(result)

    elif args.command == "scratchpad":
        scratchpad = get_mind_scratchpad(args.name)
        print(f"=== Scratchpad for {args.name} ===")
        print(scratchpad)

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
