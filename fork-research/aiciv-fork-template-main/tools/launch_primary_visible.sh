#!/bin/bash

# AICIV Civilization Launcher
# Launches Claude Code inside a tmux session for an AI civilization.
#
# Usage:
#   ./launch_primary_visible.sh [CIV_NAME]
#
# Arguments:
#   CIV_NAME  (optional) Override civilization name. If omitted, reads from
#             .aiciv-identity.json in the repository root.
#
# What it does:
#   1. Detects the civilization root directory (where this script lives)
#   2. Reads CIV_NAME from argument or .aiciv-identity.json
#   3. Creates a tmux session named {civ_name}-primary-{timestamp}
#   4. Launches Claude Code with --resume inside that session
#   5. Writes the session name to .current_session for the BOOP system
#
# Designed for Linux VPS (no Windows Terminal, no WSL).

set -euo pipefail

# === Detect Civilization Root ===
# The civ root is the parent of the tools/ directory where this script lives.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CIV_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# === Read Identity ===
IDENTITY_FILE="${CIV_ROOT}/.aiciv-identity.json"

if [[ -n "${1:-}" ]]; then
    CIV_NAME="$1"
elif [[ -f "${IDENTITY_FILE}" ]]; then
    # Try jq first, fall back to python3, fall back to grep
    if command -v jq &>/dev/null; then
        CIV_NAME=$(jq -r '.civ_name // empty' "${IDENTITY_FILE}" 2>/dev/null)
    elif command -v python3 &>/dev/null; then
        CIV_NAME=$(python3 -c "import json; print(json.load(open('${IDENTITY_FILE}'))['civ_name'])" 2>/dev/null)
    else
        CIV_NAME=$(grep -o '"civ_name"[[:space:]]*:[[:space:]]*"[^"]*"' "${IDENTITY_FILE}" | head -1 | sed 's/.*: *"//;s/"//')
    fi

    if [[ -z "${CIV_NAME}" || "${CIV_NAME}" == '${CIV_NAME}' ]]; then
        echo "ERROR: Could not read civ_name from ${IDENTITY_FILE}"
        echo "Either pass CIV_NAME as first argument or populate .aiciv-identity.json"
        exit 1
    fi
else
    echo "ERROR: No CIV_NAME argument and no ${IDENTITY_FILE} found."
    echo ""
    echo "Usage: $0 [CIV_NAME]"
    echo ""
    echo "Either:"
    echo "  1. Pass the civilization name as the first argument"
    echo "  2. Create .aiciv-identity.json with {\"civ_name\": \"YourCivName\"}"
    exit 1
fi

# Lowercase version for tmux session naming
CIV_NAME_LOWER=$(echo "${CIV_NAME}" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
SESSION_NAME="${CIV_NAME_LOWER}-primary-${TIMESTAMP}"

echo "=========================================="
echo "${CIV_NAME} Primary AI Launcher"
echo "=========================================="
echo ""
echo "Civilization: ${CIV_NAME}"
echo "Root:         ${CIV_ROOT}"
echo "Session:      ${SESSION_NAME}"
echo ""

# === Write Session Marker ===
# The BOOP system (autonomy_nudge.sh) uses this to find the active session.
echo "${SESSION_NAME}" > "${CIV_ROOT}/.current_session"

# === Create tmux Session ===
if tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
    echo "Session ${SESSION_NAME} already exists. Attaching..."
    tmux attach -t "${SESSION_NAME}"
    exit 0
fi

tmux new-session -d -s "${SESSION_NAME}" -c "${CIV_ROOT}"

# Launch Claude Code with --resume inside the tmux session
tmux send-keys -t "${SESSION_NAME}" \
    "claude --resume --dangerously-skip-permissions 'Wake up. Read .claude/scratchpad.md then CLAUDE.md. Start working.'" C-m

echo "tmux session created: ${SESSION_NAME}"
echo ""
echo "To attach:"
echo "  tmux attach -t ${SESSION_NAME}"
echo ""
echo "To detach from inside tmux:"
echo "  Ctrl+B then D"
echo ""
