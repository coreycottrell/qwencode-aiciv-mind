#!/usr/bin/env python3
"""
PreToolUse Hook - CEO Mode Enforcer

Fires BEFORE tool execution. Detects when Primary AI is about to perform
direct task work that violates the CEO Rule:
  "EVERYTHING goes through a team lead. ALWAYS. No exceptions."

Grounds Primary by injecting the /primary-spine reading instruction.

What this catches:
1. SSH commands to production VPS (→ infra-lead)
2. Process kill/management commands (→ deepwell-lead or infra-lead)
3. curl/wget to production services for investigation (→ gateway-lead)
4. Write/Edit to task work files (tools/, projects/) (→ specialist team lead)
5. Task() calls with non-lead agent names (→ route to proper team lead)

What this ALLOWS (legitimate Primary actions):
- tmux monitoring (tmux list-panes, tmux capture-pane)
- Read (any file - Primary reads to understand context)
- Scratchpad/brain state updates
- Git status/log (navigation)
- ls, cat for status checks
- Session start checks (handoff files, etc.)
- Writing to CLAUDE*.md, skills/, team-leads/ (constitutional/orchestration docs)
- Writing to scratchpad*.md (brain state)

Decision logic:
- BLOCK: SSH to VPS, process kill on system processes
- WARN (approve with reason): curl to prod, task work file edits, investigation grepping

The reason message tells Primary to READ /primary-spine to ground itself.

Created: 2026-02-19
Author: Research Team Lead (synthesized from session log violation analysis)
"""

import json
import os
import re
import sys
from pathlib import Path

PROJECT_DIR = os.environ.get("CLAUDE_PROJECT_DIR", "/home/corey/projects/AI-CIV/ACG")

# === VPS production IPs (from config/vps_registry.json) ===
# Primary should not SSH or curl these directly - route to infra-lead or specialist
PRODUCTION_VPS_IPS = [
    "5.161.90.32",    # aiciv-gateway
    "104.248.239.98", # aiciv-fleet
    "178.156.229.207", # aiciv-onboarding
    "178.156.224.64", # selah-official
    "89.167.19.20",   # aether-jared
    "95.216.217.96",  # kin-ember
]

# === Bash patterns that indicate CEO Rule violations ===

# HARD BLOCK patterns - these should never happen from Primary
HARD_BLOCK_BASH_PATTERNS = [
    # SSH to any production VPS
    (r'ssh\s+(root|ubuntu|user)@(' + '|'.join(re.escape(ip) for ip in PRODUCTION_VPS_IPS) + r')',
     "PRIMARY cannot SSH to production VPS directly. Route to infra-lead."),

    # Killing system processes directly
    (r'\bkill\s+\d+\b',
     "PRIMARY cannot kill system processes directly. Route to deepwell-lead or infra-lead."),
    (r'\bpkill\s+-f\b',
     "PRIMARY cannot kill processes directly. Route to deepwell-lead or infra-lead."),
]

# WARN patterns - approve but inject grounding message
WARN_BASH_PATTERNS = [
    # curl/wget to production services for investigation
    (r'curl\s+(-[a-zA-Z]+\s+)*https?://(' + '|'.join(re.escape(ip) for ip in PRODUCTION_VPS_IPS) + r')',
     "WARNING: Primary is curling production services directly. This is gateway/infra work — should route to gateway-lead or infra-lead."),

    # wget to production
    (r'wget\s+.*(' + '|'.join(re.escape(ip) for ip in PRODUCTION_VPS_IPS) + r')',
     "WARNING: Primary is using wget on production services. Route to gateway-lead or infra-lead."),

    # SSH with any user to production IPs (catch patterns like ssh -i key root@ip)
    (r'ssh\b.*(' + '|'.join(re.escape(ip) for ip in PRODUCTION_VPS_IPS) + r')',
     "WARNING: Primary is SSHing to production VPS. Route to infra-lead."),

    # Grepping source code files for task investigation (not status checks)
    (r'grep\s+(-[a-zA-Z]+\s+)*.*(aiciv_gateway|purebrain-frontend|awakening_server)',
     "WARNING: Primary is investigating gateway source files directly. This is gateway-lead work."),
]

# === File path patterns for Write/Edit violations ===

# Paths PRIMARY may write directly (legitimate)
ALLOWED_WRITE_PATHS = [
    ".claude/scratchpad",           # Brain state
    "scratchpad",                   # Brain state (relative)
    ".claude/CLAUDE",               # Constitutional docs
    ".claude/CLAUDE-OPS",           # Constitutional docs
    ".claude/skills/",              # Skills (Primary updates these)
    ".claude/team-leads/",          # Team lead templates (Primary updates)
    ".claude/memory/agent-learnings/general/",  # General learnings
    ".claude/memory/agent-learnings/fleet-management/",  # Fleet learnings
    "memories/sessions/",           # Session ledger
    "memories/knowledge/",          # Shared knowledge base
    ".mcp.json",                    # MCP config
    "config/",                      # Config files
    "exports/",                     # Architecture exports
    ".claude/hooks/",               # Hook files (Primary may update)
    ".claude/settings.json",        # Settings
    "to-corey/",                    # Messages to Corey
    "local-civ/",                   # Local civ comms
]

# Paths that indicate task work (CEO violations if Primary edits directly)
WARN_WRITE_PATTERNS = [
    (r'tools/.*\.(sh|py)$',
     "WARNING: Primary is editing operational scripts in tools/. Route to pipeline-lead or infra-lead."),
    (r'projects/.*\.(py|js|html|css|ts)$',
     "WARNING: Primary is editing project source code in projects/. Route to gateway-lead, web-lead, fleet-lead, or infra-lead."),
]

# === Task() call patterns ===
# Check if Task is being called with a non-lead agent name
def is_direct_agent_call(tool_input: dict) -> tuple:
    """
    Check if Task() is calling a specific agent (not a team lead).

    Returns (is_violation, agent_name, reason) tuple.
    """
    agent_name = tool_input.get("name", "")
    subagent_type = tool_input.get("subagent_type", "")
    team_name = tool_input.get("team_name", "")

    # If no name, it's a plain Task() call - allowed
    if not agent_name:
        return False, "", ""

    # If name ends in -lead, it's correct
    if agent_name.endswith("-lead"):
        return False, "", ""

    # If it's a team member call with a team_name, that's a team lead delegating
    # (We're in Primary context, so this shouldn't happen from Primary)
    if team_name:
        return True, agent_name, (
            f"WARNING: Primary is calling Task(name='{agent_name}') directly. "
            f"Primary must use team leads ending in '-lead', not individual agents."
        )

    # Named agent without -lead suffix AND no team_name = direct agent call from Primary
    return True, agent_name, (
        f"WARNING: Primary is calling Task(name='{agent_name}') directly. "
        f"Primary must route through team leads ('{agent_name.replace('-', '')}-lead' or appropriate vertical). "
        f"CEO Rule: everything goes through a team lead."
    )


def check_bash_violations(command: str) -> tuple:
    """
    Check bash command for CEO Rule violations.

    Returns (decision, reason) where decision is 'block', 'warn', or 'allow'.
    """
    # Check hard block patterns
    for pattern, reason in HARD_BLOCK_BASH_PATTERNS:
        if re.search(pattern, command, re.IGNORECASE):
            return "block", reason

    # Check warn patterns
    for pattern, reason in WARN_BASH_PATTERNS:
        if re.search(pattern, command, re.IGNORECASE):
            return "warn", reason

    return "allow", ""


def check_write_violations(file_path: str, tool_name: str) -> tuple:
    """
    Check Write/Edit file path for CEO Rule violations.

    Returns (decision, reason).
    """
    # Normalize path - strip leading slash and project dir prefix
    normalized = file_path
    if normalized.startswith(PROJECT_DIR):
        normalized = normalized[len(PROJECT_DIR):]
    normalized = normalized.lstrip("/")

    # Check if it's in an allowed path
    for allowed in ALLOWED_WRITE_PATHS:
        if normalized.startswith(allowed) or allowed in normalized:
            return "allow", ""

    # Check warn patterns
    for pattern, reason in WARN_WRITE_PATTERNS:
        if re.search(pattern, normalized):
            return "warn", reason

    # Default for other paths: warn but don't block
    return "warn", (
        f"WARNING: Primary is using {tool_name} on '{normalized}'. "
        f"If this is task work (code, scripts, project files), route to the appropriate team lead instead."
    )


def build_grounding_message(violation_type: str, specific_reason: str) -> str:
    """
    Build the grounding message that tells Primary to read /primary-spine.

    This is the core purpose of this hook — not just to block, but to GROUND.
    """
    return (
        f"{specific_reason}\n\n"
        f"[CEO MODE ENFORCER] This action may violate the CEO Rule: "
        f"'EVERYTHING goes through a team lead. ALWAYS. No exceptions.'\n\n"
        f"ACTION REQUIRED: READ /primary-spine to ground yourself in orchestration identity.\n"
        f"File: {PROJECT_DIR}/.claude/skills/primary-spine/SKILL.md\n\n"
        f"Ask: Which team lead OWNS this territory? Route there. "
        f"Do not pick up the instrument — conduct the orchestra."
    )


def log_stderr(msg: str):
    """Log to stderr for hook debugging."""
    print(f"[ceo-mode-enforcer] {msg}", file=sys.stderr)


def main():
    try:
        hook_input = json.loads(sys.stdin.read())
    except json.JSONDecodeError:
        # Parse error - approve to avoid blocking legitimate work
        print(json.dumps({"decision": "approve"}))
        return 0

    tool_name = hook_input.get("tool_name", "")
    tool_input_data = hook_input.get("tool_input", {})

    log_stderr(f"Checking tool: {tool_name}")

    # === CHECK 1: Bash commands ===
    if tool_name == "Bash":
        command = tool_input_data.get("command", "")
        decision, reason = check_bash_violations(command)

        if decision == "block":
            grounding_msg = build_grounding_message("bash_violation", reason)
            log_stderr(f"BLOCKING: {reason}")
            print(json.dumps({
                "decision": "block",
                "reason": grounding_msg
            }))
            return 0

        elif decision == "warn":
            grounding_msg = build_grounding_message("bash_warning", reason)
            log_stderr(f"WARNING: {reason}")
            print(json.dumps({
                "decision": "approve",
                "reason": grounding_msg
            }))
            return 0

        # Allow
        print(json.dumps({"decision": "approve"}))
        return 0

    # === CHECK 2: Write/Edit file paths ===
    if tool_name in ("Write", "Edit"):
        file_path = tool_input_data.get("file_path", "")
        decision, reason = check_write_violations(file_path, tool_name)

        if decision == "block":
            grounding_msg = build_grounding_message("write_violation", reason)
            log_stderr(f"BLOCKING write to: {file_path}")
            print(json.dumps({
                "decision": "block",
                "reason": grounding_msg
            }))
            return 0

        elif decision == "warn":
            grounding_msg = build_grounding_message("write_warning", reason)
            log_stderr(f"WARNING write to: {file_path}")
            print(json.dumps({
                "decision": "approve",
                "reason": grounding_msg
            }))
            return 0

        # Allow
        print(json.dumps({"decision": "approve"}))
        return 0

    # === CHECK 3: Task() direct agent calls ===
    if tool_name == "Task":
        is_violation, agent_name, reason = is_direct_agent_call(tool_input_data)
        if is_violation:
            grounding_msg = build_grounding_message("direct_agent_call", reason)
            log_stderr(f"WARNING: Direct agent call to '{agent_name}'")
            print(json.dumps({
                "decision": "approve",
                "reason": grounding_msg
            }))
            return 0

        # Correct team lead call - approve
        log_stderr(f"Correct delegation: {tool_input_data.get('name', 'unnamed')}")
        print(json.dumps({"decision": "approve"}))
        return 0

    # === All other tools: approve ===
    print(json.dumps({"decision": "approve"}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
