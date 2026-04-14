#!/bin/bash
# ACG Telegram Health Check - Run via cron
# Add to crontab: */5 * * * * /home/corey/projects/AI-CIV/ACG/tools/telegram_health_cron.sh
#
# This script checks if the ACG Telegram bot is healthy and restarts it if needed.
# CRITICAL: Uses ACG-specific process detection to avoid cross-CIV conflicts.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# CIV identity - derived at runtime for any fork
_VF="$PROJECT_ROOT/variables.template.json"
if [ -f "$_VF" ]; then
    _CIV=$(python3 -c "import json,sys; d=json.load(open('$_VF')); n=d.get('CIV_NAME','').lower().replace(' ',''); print(n) if n and n not in ('your_name','yourname','') else sys.exit(1)" 2>/dev/null)
fi
CIV_ID="${_CIV:-$(basename "$PROJECT_ROOT" | tr '[:upper:]' '[:lower:]')}"

LOG_FILE="/tmp/telegram_${CIV_ID}_health_cron.log"
BOT_LOG="/tmp/telegram_${CIV_ID}.log"
MAX_LOG_AGE=300  # 5 minutes - restart if log is older

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# CIV-specific process detection
get_civ_pid() {
    pgrep -f "python3.*${PROJECT_ROOT}.*telegram_unified.py" 2>/dev/null | head -1
}

# Check if CIV bot process is running
is_running() {
    local pid=$(get_civ_pid)
    [ -n "$pid" ]
}

# Check if log file is fresh (updated recently)
is_log_fresh() {
    if [ ! -f "$BOT_LOG" ]; then
        return 1
    fi
    local age=$(($(date +%s) - $(stat -c %Y "$BOT_LOG")))
    [ "$age" -lt "$MAX_LOG_AGE" ]
}

# Check if CIV tmux session exists
has_tmux_session() {
    tmux list-sessions -F "#{session_name}" 2>/dev/null | grep -q "${CIV_ID}-primary-"
}

# Start the CIV bot
start_bot() {
    log "Starting ${CIV_ID} bot..."
    cd "$PROJECT_ROOT"

    # Kill any stale bot process
    local old_pid=$(get_civ_pid)
    if [ -n "$old_pid" ]; then
        log "Killing stale bot (PID: $old_pid)"
        kill "$old_pid" 2>/dev/null
        sleep 1
    fi

    # Clean up PID file
    rm -f "/tmp/${CIV_ID}_telegram.pid"

    # Start fresh
    nohup python3 "$PROJECT_ROOT/tools/telegram_unified.py" > "$BOT_LOG" 2>&1 &
    local new_pid=$!
    echo $new_pid > "/tmp/${CIV_ID}_telegram.pid"

    sleep 2
    if is_running; then
        log "ACG Bot started successfully (PID: $(get_acg_pid))"
        return 0
    else
        log "ERROR: Failed to start ACG bot"
        return 1
    fi
}

# Main health check
main() {
    local needs_restart=0
    local reason=""

    # Check 1: Is ACG process running?
    if ! is_running; then
        needs_restart=1
        reason="ACG process not running"
    fi

    # Check 2: Is log fresh? (only if process is "running")
    if [ "$needs_restart" -eq 0 ] && ! is_log_fresh; then
        needs_restart=1
        reason="Log file stale (possible hang)"
    fi

    # Check 3: Does ACG tmux session exist? (if not, bot can't inject)
    if ! has_tmux_session; then
        # Don't restart, but log warning
        log "WARNING: No ACG tmux session found - bot will wait"
    fi

    # Restart if needed
    if [ "$needs_restart" -eq 1 ]; then
        log "Health check FAILED: $reason"
        start_bot
    fi
}

# Run
main
