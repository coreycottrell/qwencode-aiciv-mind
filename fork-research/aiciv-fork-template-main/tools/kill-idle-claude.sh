#!/usr/bin/env bash
# kill-idle-claude.sh - Kill idle Claude Code instances in tmux sessions
#
# Purpose: Safety net to reclaim RAM from abandoned onboarding sessions.
# The awakening_server.py has its own lifecycle manager, but this cron-based
# script catches anything that slips through (e.g., server restart loses
# in-memory session state but tmux sessions persist).
#
# Runs as root via cron every 5 minutes.
# Checks tmux sessions owned by 'aiciv' user.
# Kills sessions with no input activity for 15+ minutes.
#
# Log: /var/log/claude-idle-killer.log

set -euo pipefail

IDLE_THRESHOLD=900  # 15 minutes in seconds
LOG_FILE="/var/log/claude-idle-killer.log"
TMUX_USER="aiciv"

# Counters for summary
checked=0
killed=0
skipped=0
errors=0

log() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    echo "$msg" >> "$LOG_FILE"
    echo "$msg"
}

# Get current epoch time
now=$(date +%s)

# Get list of tmux sessions owned by aiciv
# Using su to run tmux as the aiciv user so we see their sessions
session_list=$(su - "$TMUX_USER" -c 'tmux list-sessions -F "#{session_name}" 2>/dev/null' || true)

if [ -z "$session_list" ]; then
    echo "No tmux sessions found for user $TMUX_USER. Nothing to do."
    exit 0
fi

log "--- Idle check started (threshold: ${IDLE_THRESHOLD}s) ---"

while IFS= read -r session_name; do
    # Skip empty lines
    [ -z "$session_name" ] && continue

    checked=$((checked + 1))

    # Get last activity timestamp (epoch seconds) for this session
    # session_activity = time of last input to the session
    activity_epoch=$(su - "$TMUX_USER" -c "tmux display-message -p -t '${session_name}' '#{session_activity}'" 2>/dev/null || true)

    if [ -z "$activity_epoch" ]; then
        log "ERROR: Could not get activity time for session '$session_name' (session may have died)"
        errors=$((errors + 1))
        continue
    fi

    # Validate that activity_epoch is a number
    if ! [[ "$activity_epoch" =~ ^[0-9]+$ ]]; then
        log "ERROR: Invalid activity epoch '$activity_epoch' for session '$session_name'"
        errors=$((errors + 1))
        continue
    fi

    idle_seconds=$((now - activity_epoch))

    if [ "$idle_seconds" -ge "$IDLE_THRESHOLD" ]; then
        idle_minutes=$((idle_seconds / 60))
        log "KILL: Session '$session_name' idle for ${idle_minutes}m ${idle_seconds}s (threshold: ${IDLE_THRESHOLD}s)"

        # Step 1: Kill any claude processes inside the session's pane
        # Get the pane PID, then find claude children
        pane_pid=$(su - "$TMUX_USER" -c "tmux display-message -p -t '${session_name}' '#{pane_pid}'" 2>/dev/null || true)
        if [ -n "$pane_pid" ] && [ "$pane_pid" != "0" ]; then
            # Find and kill claude processes that are descendants of the pane
            # Use pkill to kill the process tree rooted at the pane PID
            claude_pids=$(pgrep -P "$pane_pid" -f "claude" 2>/dev/null || true)
            if [ -n "$claude_pids" ]; then
                for pid in $claude_pids; do
                    log "  Killing claude process PID $pid (child of pane PID $pane_pid)"
                    kill -TERM "$pid" 2>/dev/null || true
                done
                # Give processes a moment to die gracefully
                sleep 2
                # Force kill if still alive
                for pid in $claude_pids; do
                    if kill -0 "$pid" 2>/dev/null; then
                        log "  Force-killing claude PID $pid"
                        kill -KILL "$pid" 2>/dev/null || true
                    fi
                done
            fi
        fi

        # Step 2: Kill the tmux session itself
        su - "$TMUX_USER" -c "tmux kill-session -t '${session_name}'" 2>/dev/null || true
        killed=$((killed + 1))
        log "  Session '$session_name' terminated."
    else
        remaining=$((IDLE_THRESHOLD - idle_seconds))
        skipped=$((skipped + 1))
        # Only log active sessions at debug level (don't spam the log)
    fi

done <<< "$session_list"

# Summary
summary="--- Summary: checked=$checked killed=$killed active=$skipped errors=$errors ---"
log "$summary"

# Also print summary to stdout for cron email (if configured)
echo "$summary"
