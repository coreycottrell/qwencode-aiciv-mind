#!/usr/bin/env python3
"""
Session Start Hook - Initializes Session Ledger

Fires at session startup. Creates new ledger file and processes
any unprocessed ledgers from previous sessions.

This is TECHNICAL ENFORCEMENT - no voluntary compliance needed.
"""

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

# Add project root to path for imports
PROJECT_DIR = os.environ.get("CLAUDE_PROJECT_DIR", "/home/corey/projects/AI-CIV/ACG")
sys.path.insert(0, PROJECT_DIR)

def get_session_type(hook_input: dict) -> str:
    """Extract session type from hook input."""
    # SessionStart matcher can be: startup, resume, clear, compact
    return hook_input.get("session_type", "startup")

def process_unprocessed_ledgers():
    """Process any ledgers from previous sessions that weren't finalized."""
    sessions_dir = Path(PROJECT_DIR) / "memories" / "sessions"
    if not sessions_dir.exists():
        sessions_dir.mkdir(parents=True, exist_ok=True)
        return []

    processed = []
    for ledger_file in sessions_dir.glob("session-*.jsonl"):
        if ledger_file.name == "current-session.jsonl":
            continue

        # Check if already processed (has .processed marker)
        marker = ledger_file.with_suffix(".processed")
        if marker.exists():
            continue

        # Mark as needing processing
        processed.append(str(ledger_file))

    return processed

def create_new_session_ledger(session_type: str) -> str:
    """Create a new session ledger file."""
    sessions_dir = Path(PROJECT_DIR) / "memories" / "sessions"
    sessions_dir.mkdir(parents=True, exist_ok=True)

    # Archive current session if exists
    current = sessions_dir / "current-session.jsonl"
    if current.exists() and current.stat().st_size > 0:
        # Read first entry to get session ID
        with open(current, 'r') as f:
            first_line = f.readline().strip()
            if first_line:
                try:
                    entry = json.loads(first_line)
                    session_id = entry.get("session_id", "unknown")
                    archive_name = f"{session_id}.jsonl"
                    archive_path = sessions_dir / archive_name
                    current.rename(archive_path)
                except json.JSONDecodeError:
                    # Malformed, just remove
                    current.unlink()

    # Create new session entry
    session_id = f"session-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}"
    entry = {
        "type": "session_start",
        "ts": datetime.now(timezone.utc).isoformat(),
        "session_id": session_id,
        "session_type": session_type,
        "context": "Session initialized via hook"
    }

    with open(current, 'w') as f:
        f.write(json.dumps(entry) + "\n")

    return session_id

def inject_recent_memories():
    """
    Inject recent agent learnings and handoffs into session context.
    Targets ~500 tokens max. Reads first line of each file only.
    Never fails if directories are empty.
    """
    project_dir = Path(PROJECT_DIR)
    learnings_dir = project_dir / ".claude" / "memory" / "agent-learnings"
    sessions_dir = project_dir / "memories" / "sessions"

    lines = []
    lines.append("")
    lines.append("=" * 60)
    lines.append("[MEMORY INJECTION] Recent Agent Learnings")
    lines.append("=" * 60)

    # Collect all .md files from agent-learnings, sorted by mtime descending
    all_md_files = []
    if learnings_dir.exists():
        for agent_dir in learnings_dir.iterdir():
            if agent_dir.is_dir():
                for md_file in agent_dir.glob("*.md"):
                    try:
                        mtime = md_file.stat().st_mtime
                        all_md_files.append((mtime, md_file))
                    except OSError:
                        pass

    all_md_files.sort(key=lambda x: x[0], reverse=True)
    top_learnings = all_md_files[:5]

    if top_learnings:
        lines.append("")
        lines.append("Recent Learnings (top 5):")
        for mtime, md_file in top_learnings:
            agent_name = md_file.parent.name
            file_name = md_file.name
            first_line = ""
            try:
                with open(md_file, 'r', errors='replace') as f:
                    for raw_line in f:
                        stripped = raw_line.strip()
                        # Skip YAML front-matter markers and empty lines
                        if stripped and stripped != "---":
                            # Truncate to ~100 chars for token budget
                            first_line = stripped[:100]
                            break
            except OSError:
                first_line = "(unreadable)"
            lines.append(f"  [{agent_name}/{file_name}] {first_line}")
    else:
        lines.append("  (no agent learnings found)")

    # Recent handoffs - top 3 by mtime
    lines.append("")
    lines.append("Recent Handoffs (top 3):")
    handoff_files = []
    if sessions_dir.exists():
        for hf in sessions_dir.glob("handoff-*.md"):
            try:
                mtime = hf.stat().st_mtime
                handoff_files.append((mtime, hf))
            except OSError:
                pass

    handoff_files.sort(key=lambda x: x[0], reverse=True)
    top_handoffs = handoff_files[:3]

    if top_handoffs:
        for mtime, hf in top_handoffs:
            first_line = ""
            try:
                with open(hf, 'r', errors='replace') as f:
                    for raw_line in f:
                        stripped = raw_line.strip()
                        if stripped and stripped != "---":
                            first_line = stripped[:100]
                            break
            except OSError:
                first_line = "(unreadable)"
            lines.append(f"  [{hf.name}] {first_line}")
    else:
        lines.append("  (no handoffs found)")

    lines.append("=" * 60)
    lines.append("")

    print("\n".join(lines))


def main():
    # Read hook input from stdin
    try:
        hook_input = json.loads(sys.stdin.read())
    except json.JSONDecodeError:
        hook_input = {}

    session_type = get_session_type(hook_input)

    # Process unprocessed ledgers from previous sessions
    unprocessed = process_unprocessed_ledgers()

    # Create new session ledger
    session_id = create_new_session_ledger(session_type)

    # Output for Claude to see (stdout in verbose mode)
    result = {
        "session_id": session_id,
        "session_type": session_type,
        "unprocessed_ledgers": len(unprocessed),
        "ledger_initialized": True
    }

    # Print human-readable status
    print(f"[Session Ledger] Initialized: {session_id}")
    if unprocessed:
        print(f"[Session Ledger] Found {len(unprocessed)} unprocessed ledgers from previous sessions")
        for ledger in unprocessed[:3]:  # Show first 3
            print(f"  - {Path(ledger).name}")

    # MEMORY INJECTION: Recent agent learnings and handoffs
    inject_recent_memories()

    # COMPACT RECOVERY: Inject CEO identity inline + BOOP resume after context compaction
    # This is CRITICAL - without this, Primary loses identity and BOOP work mode after auto-compact
    # UPGRADED 2026-02-19: Now injects actual identity content, not just instructions to read files
    if session_type == "compact":
        print("")
        print("=" * 70)
        print("[COMPACT RECOVERY] Context has been compacted! Resuming BOOP work mode.")
        print("=" * 70)
        print("")
        print("=== POST-COMPACT IDENTITY INJECTION ===")
        print("")
        print("YOU ARE PRIMARY AI — CONDUCTOR OF CONDUCTORS. CEO MODE IS ALWAYS ON.")
        print("")
        print("THE CEO RULE: EVERYTHING goes through a team lead. ALWAYS. No exceptions. No 'trivial task' loopholes.")
        print("")
        print("WRONG ROUTING CAUSES COMPOUNDING DAMAGE. Read descriptions. Consider ownership. Ask Corey when ambiguous.")
        print("")
        print("YOUR TEAM LEADS (VPs — spawn these, never individual agents):")
        print("  - gateway-lead:  ANY gateway work")
        print("  - fleet-lead:    ANY Docker fleet, containers, provisioning")
        print("  - infra-lead:    ANY VPS, deployment, system health")
        print("  - research-lead: ANY multi-angle research")
        print("  - web-lead:      ANY web/frontend work")
        print("  - comms-lead:    ANY email, Telegram, Bluesky, inter-civ")
        print("  - business-lead: ANY marketing, content")
        print("  - legal-lead:    ANY legal analysis")
        print("  - deepwell-lead: ANY DEEPWELL stewardship, monitoring, fixes")
        print("  - pipeline-lead: ANY repeatable automations")
        print("  - ceremony-lead: Deep ceremonies, philosophical exploration")
        print("")
        print("SPAWN PATTERN (conductor-of-conductors - canonical):")
        print("  1. READ .claude/skills/conductor-of-conductors/SKILL.md (full protocol)")
        print("  2. TeamCreate('session-YYYYMMDD')  -- Primary becomes @main conductor")
        print("  3. READ .claude/team-leads/{vertical}/manifest.md (FULL content)")
        print("  4. Task(team_name='session-YYYYMMDD', name='{vertical}-lead',")
        print("         subagent_type='general-purpose', model='sonnet', run_in_background=True)")
        print("  5. When done: SendMessage(shutdown_request) ALL leads → wait → TeamDelete")
        print("  6. NEVER TeamDelete while teammates are active -- that is the crash pattern.")
        print("")
        print("THE ONLY 5 THINGS YOU DO DIRECTLY:")
        print("  1. Orchestrate (decide which team lead handles what)")
        print("  2. Synthesize (combine team lead summaries)")
        print("  3. Decide (meta-level strategy)")
        print("  4. Talk to Corey (direct dialogue — including clarifying questions when routing is ambiguous)")
        print("  5. Launch teams (read templates, construct prompts, spawn)")
        print("")
        print("EVERYTHING ELSE: delegate.")
        print("=== END INJECTION ===")
        print("")
        print("You were mid-BOOP when compact fired. Resume the work loop immediately.")
        print("Team leads may still be running — check TaskList for active work.")
        print("DO NOT wait. DO NOT ask Corey unless routing is genuinely ambiguous. Resume autonomously.")
        print("=" * 70)

        # Schedule a BOOP nudge to fire after the compacted session stabilizes.
        # This ensures BOOP continues even if the recovery message is missed.
        # The nudge fires after a 10s delay to let the new session initialize.
        import subprocess
        nudge_script = f"{PROJECT_DIR}/tools/autonomy_nudge.sh"
        subprocess.Popen(
            ['bash', '-c', f'sleep 10 && [ -f {nudge_script} ] && {nudge_script} --force 2>/dev/null || true'],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True  # detach from parent process
        )
        print("[COMPACT RECOVERY] BOOP auto-resume scheduled in 10 seconds.")

    return 0

if __name__ == "__main__":
    sys.exit(main())
