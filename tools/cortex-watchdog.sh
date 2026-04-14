#!/bin/bash
# cortex-watchdog.sh — Self-healing daemon manager for Cortex.
#
# Runs the Cortex daemon in a restart loop. On crash (non-zero exit),
# logs the event, notifies ACG, and restarts after a cooldown.
# Prevents infinite crash loops with a per-hour restart cap.
#
# Usage:
#   nohup tools/cortex-watchdog.sh [daemon args...] > data/logs/watchdog.log 2>&1 &
#
# All arguments are forwarded to the Cortex binary. If none given, defaults to:
#   --daemon --mind-id root --model minimax-m2.7
#
# Exit 0 from daemon = clean shutdown = watchdog exits.
# Any other exit = crash = restart after 5s.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORTEX_BIN="$PROJECT_ROOT/target/release/cortex"
TALK_SCRIPT="$PROJECT_ROOT/tools/talk_to_acg.py"
LOG_DIR="$PROJECT_ROOT/data/logs"
WATCHDOG_LOG="$LOG_DIR/watchdog.log"

# Restart limits
MAX_RESTARTS_PER_HOUR=10
RESTART_COUNT=0
HOUR_START=$(date +%s)

# Default daemon arguments if none provided
if [ $# -eq 0 ]; then
    DAEMON_ARGS="--daemon --mind-id root --model minimax-m2.7"
else
    DAEMON_ARGS="$*"
fi

# Ensure log directory exists
mkdir -p "$LOG_DIR"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [watchdog] $*" | tee -a "$WATCHDOG_LOG"
}

notify_acg() {
    if [ -f "$TALK_SCRIPT" ]; then
        python3 "$TALK_SCRIPT" --from cortex-watchdog "$1" 2>/dev/null || true
    fi
}

# Check binary exists
if [ ! -x "$CORTEX_BIN" ]; then
    log "ERROR: Cortex binary not found at $CORTEX_BIN"
    log "Run: cargo build --release"
    exit 1
fi

log "Cortex Watchdog starting"
log "Binary: $CORTEX_BIN"
log "Args: $DAEMON_ARGS"
log "Max restarts/hour: $MAX_RESTARTS_PER_HOUR"

while true; do
    # Reset hourly counter if an hour has passed
    NOW=$(date +%s)
    ELAPSED=$(( NOW - HOUR_START ))
    if [ "$ELAPSED" -ge 3600 ]; then
        RESTART_COUNT=0
        HOUR_START=$NOW
    fi

    # Generate a timestamped log file for this run
    RUN_LOG="$LOG_DIR/daemon-$(date '+%Y%m%d-%H%M%S').log"
    log "Launching Cortex → $RUN_LOG"
    log "Command: $CORTEX_BIN $DAEMON_ARGS"

    # Launch the daemon, capturing output to the run log
    set +e
    cd "$PROJECT_ROOT"
    $CORTEX_BIN $DAEMON_ARGS > "$RUN_LOG" 2>&1
    EXIT_CODE=$?
    set -e

    # Clean shutdown
    if [ "$EXIT_CODE" -eq 0 ]; then
        log "Cortex exited cleanly (exit 0). Watchdog stopping."
        notify_acg "[Cortex] Clean shutdown. Watchdog exiting."
        break
    fi

    # Crash — extract last 50 lines for context
    RESTART_COUNT=$(( RESTART_COUNT + 1 ))
    LAST_LINES=$(tail -n 50 "$RUN_LOG" 2>/dev/null || echo "(could not read log)")

    log "CRASH: Cortex exited with code $EXIT_CODE (restart #$RESTART_COUNT this hour)"
    log "Last 50 lines from $RUN_LOG:"
    echo "$LAST_LINES" >> "$WATCHDOG_LOG"

    # Extract the actual panic/error line for the notification
    ERROR_LINE=$(echo "$LAST_LINES" | grep -E "panicked|thread.*panic|Error|FATAL" | tail -1 || echo "exit code $EXIT_CODE")
    notify_acg "[CRASH] Daemon died (exit $EXIT_CODE). Restart #$RESTART_COUNT. Error: $ERROR_LINE"

    # Check restart limit
    if [ "$RESTART_COUNT" -ge "$MAX_RESTARTS_PER_HOUR" ]; then
        log "CRITICAL: Max restarts ($MAX_RESTARTS_PER_HOUR) reached this hour. Stopping."
        notify_acg "[CRITICAL] Cortex hit $MAX_RESTARTS_PER_HOUR restarts in 1 hour. Needs manual intervention. Last error: $ERROR_LINE"
        exit 1
    fi

    log "Restarting in 5 seconds..."
    sleep 5
done
