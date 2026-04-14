#!/bin/bash
# session_monitor.sh — Ensure active Claude session, TG health, cleanup dead sessions
#
# Lightweight script intended to run as a cron job every 5 minutes.
# Works for any fork of the AICIV template (reads .aiciv-identity.json).
#
# Cron example (run as civ user):
#   */5 * * * * /home/aiciv/civ/tools/session_monitor.sh >> /home/aiciv/civ/memories/system/session_monitor.log 2>&1
#
# What it does:
#   1. Reads CIV identity from .aiciv-identity.json
#   2. Checks if .current_session exists and that tmux session is alive
#   3. If session is dead: relaunches via launch_primary_visible.sh
#   4. Kills any claude processes NOT belonging to the current session
#   5. Checks if TG bot process is running
#   6. If TG bot dead: attempts restart via start_telegram_bot.sh
#   7. Logs all status to memories/system/session_monitor.log
#
# Safe guards:
#   - Never kills the current active session's processes
#   - Never kills processes it cannot positively identify as orphaned claude
#   - Logs all actions with timestamps for audit trail

set -euo pipefail

# === Identity Detection ===
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CIV_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
IDENTITY_FILE="${CIV_ROOT}/.aiciv-identity.json"

if [[ -f "${IDENTITY_FILE}" ]]; then
    if command -v jq &>/dev/null; then
        CIV_NAME=$(jq -r '.civ_name // empty' "${IDENTITY_FILE}" 2>/dev/null)
    elif command -v python3 &>/dev/null; then
        CIV_NAME=$(python3 -c "import json; print(json.load(open('${IDENTITY_FILE}'))['civ_name'])" 2>/dev/null)
    else
        CIV_NAME=$(grep -o '"civ_name"[[:space:]]*:[[:space:]]*"[^"]*"' "${IDENTITY_FILE}" | head -1 | sed 's/.*: *"//;s/"//')
    fi
fi

if [[ -z "${CIV_NAME:-}" || "${CIV_NAME}" == '${CIV_NAME}' ]]; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: Could not read civ_name from ${IDENTITY_FILE}"
    exit 1
fi

CIV_NAME_LOWER=$(echo "${CIV_NAME}" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')

# === Paths ===
SESSION_MARKER="${CIV_ROOT}/.current_session"
LAUNCH_SCRIPT="${CIV_ROOT}/tools/launch_primary_visible.sh"
TG_BOT_SCRIPT="${CIV_ROOT}/tools/start_telegram_bot.sh"
TG_BOT_PROCESS="telegram_unified.py"
LOG_FILE="${CIV_ROOT}/memories/system/session_monitor.log"

# Ensure log directory exists
mkdir -p "$(dirname "${LOG_FILE}")"

# === Logging ===
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [session_monitor] $1" | tee -a "${LOG_FILE}"
}

log "--- Monitor cycle started for ${CIV_NAME} ---"

# === Step 1: Check current session ===
CURRENT_SESSION=""
SESSION_ALIVE=false

if [[ -f "${SESSION_MARKER}" ]]; then
    CURRENT_SESSION=$(cat "${SESSION_MARKER}" | tr -d '[:space:]')
    log "Session marker found: ${CURRENT_SESSION}"

    if tmux has-session -t "${CURRENT_SESSION}" 2>/dev/null; then
        SESSION_ALIVE=true
        log "Session ALIVE: ${CURRENT_SESSION}"
    else
        log "Session DEAD (tmux session not found): ${CURRENT_SESSION}"
    fi
else
    # Try to find any session for this civ
    FOUND_SESSION=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "^${CIV_NAME_LOWER}-primary-" | sort | tail -1 || true)
    if [[ -n "${FOUND_SESSION}" ]]; then
        CURRENT_SESSION="${FOUND_SESSION}"
        SESSION_ALIVE=true
        # Update marker
        echo "${CURRENT_SESSION}" > "${SESSION_MARKER}"
        log "No marker file, but found session: ${CURRENT_SESSION} (marker updated)"
    else
        log "No session marker and no tmux session found for ${CIV_NAME}"
    fi
fi

# === Step 2: Relaunch if dead ===
if [[ "${SESSION_ALIVE}" != "true" ]]; then
    log "ACTION: No live session found — relaunching via launch_primary_visible.sh"
    if [[ -x "${LAUNCH_SCRIPT}" ]]; then
        # Launch in background (it will create a new tmux session + write .current_session)
        # We don't attach — this is a background monitor
        nohup "${LAUNCH_SCRIPT}" "${CIV_NAME}" > /tmp/${CIV_NAME_LOWER}_relaunch.log 2>&1 &
        log "ACTION: Relaunch initiated (PID: $!). Check /tmp/${CIV_NAME_LOWER}_relaunch.log"
    else
        log "ERROR: Launch script not found or not executable: ${LAUNCH_SCRIPT}"
    fi
fi

# === Step 3: Kill orphaned claude processes ===
# Find all claude processes, check if they belong to the current session's pane
if [[ "${SESSION_ALIVE}" == "true" && -n "${CURRENT_SESSION}" ]]; then
    # Get the pane PID for the current session
    ACTIVE_PANE_PID=$(tmux display-message -t "${CURRENT_SESSION}:0.0" -p '#{pane_pid}' 2>/dev/null || true)

    if [[ -n "${ACTIVE_PANE_PID}" ]]; then
        # Find all claude-related processes
        ALL_CLAUDE_PIDS=$(pgrep -f "claude" 2>/dev/null || true)

        if [[ -n "${ALL_CLAUDE_PIDS}" ]]; then
            KILLED_COUNT=0
            for PID in ${ALL_CLAUDE_PIDS}; do
                # Check if this PID is a descendant of the active pane
                PPID_CHAIN=$(ps -o ppid= -p "${PID}" 2>/dev/null | tr -d ' ' || true)
                IS_ACTIVE=false

                # Walk up the process tree to see if it connects to active pane
                CHECK_PID="${PPID_CHAIN}"
                for _ in $(seq 1 10); do
                    if [[ "${CHECK_PID}" == "${ACTIVE_PANE_PID}" ]]; then
                        IS_ACTIVE=true
                        break
                    fi
                    CHECK_PID=$(ps -o ppid= -p "${CHECK_PID}" 2>/dev/null | tr -d ' ' || true)
                    [[ -z "${CHECK_PID}" || "${CHECK_PID}" == "0" || "${CHECK_PID}" == "1" ]] && break
                done

                if [[ "${IS_ACTIVE}" != "true" ]]; then
                    # Confirm it's actually a claude process before killing
                    PROC_CMD=$(ps -p "${PID}" -o comm= 2>/dev/null || true)
                    if [[ "${PROC_CMD}" == "claude" ]] || [[ "${PROC_CMD}" == "node" ]]; then
                        log "ACTION: Killing orphaned claude process PID ${PID} (cmd: ${PROC_CMD})"
                        kill -TERM "${PID}" 2>/dev/null || true
                        KILLED_COUNT=$((KILLED_COUNT + 1))
                    fi
                fi
            done
            if [[ "${KILLED_COUNT}" -gt 0 ]]; then
                log "Orphan cleanup: killed ${KILLED_COUNT} orphaned claude process(es)"
            else
                log "Orphan cleanup: no orphaned claude processes found"
            fi
        else
            log "No claude processes running"
        fi
    else
        log "Could not get pane PID for ${CURRENT_SESSION} — skipping orphan cleanup"
    fi
else
    log "Skipping orphan cleanup (no live session to compare against)"
fi

# === Step 4: Check TG bot process ===
TG_PID=$(pgrep -f "${TG_BOT_PROCESS}" 2>/dev/null | head -1 || true)

if [[ -n "${TG_PID}" ]]; then
    log "TG bot RUNNING (PID: ${TG_PID})"
else
    log "TG bot NOT RUNNING"
    if [[ -x "${TG_BOT_SCRIPT}" ]]; then
        log "ACTION: Attempting TG bot restart via ${TG_BOT_SCRIPT}"
        nohup "${TG_BOT_SCRIPT}" restart > /tmp/${CIV_NAME_LOWER}_tg_restart.log 2>&1 &
        log "ACTION: TG bot restart initiated (PID: $!)"
    else
        log "WARNING: TG bot restart script not found or not executable: ${TG_BOT_SCRIPT}"
    fi
fi

# === Step 5: Log summary ===
log "--- Monitor cycle complete ---"
log "  Session alive: ${SESSION_ALIVE}"
log "  Current session: ${CURRENT_SESSION:-none}"
log "  TG bot: ${TG_PID:-not running}"
