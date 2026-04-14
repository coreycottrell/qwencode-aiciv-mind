#!/bin/bash

# AICIV Civilization Launcher - Linux Tower (Generic Fork Template Version)
# Runs natively on Linux (no WSL, no wt.exe)
# Reads civilization identity from .aiciv-identity.json
#
# Version: 1.0 (2026-02-21)
# Usage: ./launch_civ_tower.sh
#   Attach manually: tmux attach -t <session-name>
#
# IMPORTANT: Ensure ANTHROPIC_API_KEY is NOT set in environment.
# Web auth (claude /login → option 1) is the correct auth method.
# This script explicitly unsets ANTHROPIC_API_KEY before launching Claude.

set -euo pipefail

# === Detect Civilization Root ===
# The civ root is the parent of the tools/ directory where this script lives.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# === Read Identity ===
IDENTITY_FILE="${PROJECT_DIR}/.aiciv-identity.json"

if [[ -f "${IDENTITY_FILE}" ]]; then
    # Try jq first, fall back to python3, fall back to grep
    if command -v jq &>/dev/null; then
        CIV_NAME=$(jq -r '.civ_name // empty' "${IDENTITY_FILE}" 2>/dev/null)
    elif command -v python3 &>/dev/null; then
        CIV_NAME=$(python3 -c "import json; print(json.load(open('${IDENTITY_FILE}'))['civ_name'])" 2>/dev/null)
    else
        CIV_NAME=$(grep -o '"civ_name"[[:space:]]*:[[:space:]]*"[^"]*"' "${IDENTITY_FILE}" | head -1 | sed 's/.*: *"//;s/"//')
    fi

    if [[ -z "${CIV_NAME:-}" || "${CIV_NAME}" == '${CIV_NAME}' ]]; then
        echo "ERROR: Could not read civ_name from ${IDENTITY_FILE}"
        echo "Ensure .aiciv-identity.json contains a valid civ_name."
        exit 1
    fi
else
    echo "ERROR: Identity file not found: ${IDENTITY_FILE}"
    echo "Create .aiciv-identity.json with {\"civ_name\": \"YourCivName\"}"
    exit 1
fi

# Lowercase version for tmux session naming
CIV_NAME_LOWER=$(echo "${CIV_NAME}" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
SESSION_NAME="${CIV_NAME_LOWER}-primary-${TIMESTAMP}"

CLAUDE_PROMPT="You are PRIMARY AI — CONDUCTOR OF CONDUCTORS for ${CIV_NAME}. Execute the wake-up protocol skill at .claude/skills/wake-up-protocol/SKILL.md - all 7 steps. Daily scratchpad: .claude/scratchpad-daily/$(date +%Y-%m-%d).md. TEAM RULE: If it can be done by a team, it MUST be done by a team. Then immediately enter work-mode BOOPs using the work-mode-boop skill at .claude/skills/work-mode-boop/SKILL.md. Run autonomously until context is exhausted. Stop immediately if ${CIV_NAME} human partner messages."

echo "=========================================="
echo "   ${CIV_NAME} Primary AI Launcher (Tower)"
echo "=========================================="
echo ""
echo "Civilization: ${CIV_NAME}"
echo "Root:         ${PROJECT_DIR}"
echo ""

# Check if any existing session for this civ already exists
EXISTING_SESSION=$(tmux list-sessions 2>/dev/null | grep "^${CIV_NAME_LOWER}-primary-" | head -1 | cut -d: -f1)

if [ -n "${EXISTING_SESSION}" ]; then
    echo "Existing session found: ${EXISTING_SESSION}"
    echo ""
    echo "Options:"
    echo "  1) Attach to existing session: ${EXISTING_SESSION}"
    echo "  2) Create a new session"
    echo ""
    read -r -p "Choice [1/2]: " CHOICE

    if [ "${CHOICE}" = "1" ]; then
        echo ""
        echo "Attaching to: ${EXISTING_SESSION}"
        tmux attach -t "${EXISTING_SESSION}"
        exit 0
    fi
    echo ""
    echo "Creating new session..."
fi

echo "Session: ${SESSION_NAME}"
echo ""

# Write session name to file for reference (used by autonomy_nudge.sh BOOP system)
echo "${SESSION_NAME}" > "${PROJECT_DIR}/.current_session"

# Create tmux session in detached mode
tmux new-session -d -s "${SESSION_NAME}" -c "${PROJECT_DIR}"

# CRITICAL: Unset ANTHROPIC_API_KEY before launching Claude.
# Web auth (claude /login → option 1) must be used, not API key auth.
# If ANTHROPIC_API_KEY is set in the environment, it overrides web auth
# and causes "insufficient credits" or wrong-account errors.
tmux send-keys -t "${SESSION_NAME}" "unset ANTHROPIC_API_KEY" C-m

# Send claude command into the session
tmux send-keys -t "${SESSION_NAME}" "claude --model claude-sonnet-4-6 --dangerously-skip-permissions '${CLAUDE_PROMPT}'" C-m

echo "Tmux session created: ${SESSION_NAME}"
echo "Working directory: ${PROJECT_DIR}"
echo ""
echo "Attaching now... (Ctrl-b d to detach)"
echo ""

# Attach to the newly created session in this terminal
tmux attach -t "${SESSION_NAME}"
