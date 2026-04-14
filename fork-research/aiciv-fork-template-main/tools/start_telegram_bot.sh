#!/bin/bash
# Start the A-C-Gee Unified Telegram Bot
# Usage: ./start_telegram_bot.sh [start|stop|status|restart|logs|health|recover]
#
# CRITICAL: Uses ACG-specific process detection to avoid cross-CIV conflicts.
# - PID file: /tmp/acg_telegram.pid
# - Log file: /tmp/telegram_acg.log
# - Process match: AI-CIV/ACG.*telegram_unified.py

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BOT_SCRIPT="$PROJECT_ROOT/tools/telegram_unified.py"

# CIV identity - derived at runtime for any fork (reads variables.template.json or uses directory name)
_VF="$PROJECT_ROOT/variables.template.json"
if [ -f "$_VF" ]; then
    _CIV=$(python3 -c "import json,sys; d=json.load(open('$_VF')); n=d.get('CIV_NAME','').lower().replace(' ',''); print(n) if n and n not in ('your_name','yourname','') else sys.exit(1)" 2>/dev/null)
fi
CIV_ID="${_CIV:-$(basename "$PROJECT_ROOT" | tr '[:upper:]' '[:lower:]')}"
LOG_FILE="/tmp/telegram_${CIV_ID}.log"
PID_FILE="/tmp/${CIV_ID}_telegram.pid"

# CIV-specific process detection (matches this civ's bot only)
get_civ_pid() {
    pgrep -f "python3.*${PROJECT_ROOT}.*telegram_unified.py" 2>/dev/null | head -1
}

start_bot() {
    # Check for running ACG-specific process
    existing_pid=$(get_civ_pid)
    if [ -n "$existing_pid" ]; then
        echo "ACG Bot already running (PID: $existing_pid)"
        return 1
    fi

    # Also check PID file for stale entries and clean up
    if [ -f "$PID_FILE" ]; then
        pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "ACG Bot already running (PID: $pid)"
            return 1
        else
            # Stale PID file, clean it up
            rm -f "$PID_FILE"
        fi
    fi

    echo "Starting ACG Telegram bot..."
    cd "$PROJECT_ROOT"
    nohup python3 "$BOT_SCRIPT" > "$LOG_FILE" 2>&1 &
    echo $! > "$PID_FILE"
    sleep 2

    if kill -0 "$(cat $PID_FILE)" 2>/dev/null; then
        echo "ACG Bot started (PID: $(cat $PID_FILE))"
        echo "Log: $LOG_FILE"
    else
        echo "Failed to start ACG bot"
        rm -f "$PID_FILE"
        return 1
    fi
}

stop_bot() {
    if [ -f "$PID_FILE" ]; then
        pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "Stopping ACG bot (PID: $pid)..."
            kill "$pid"
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid"
            fi
            rm -f "$PID_FILE"
            echo "ACG Bot stopped"
        else
            echo "ACG Bot not running (stale PID file)"
            rm -f "$PID_FILE"
        fi
    else
        # Try to find and kill ACG-specific process only
        acg_pid=$(get_civ_pid)
        if [ -n "$acg_pid" ]; then
            kill "$acg_pid" 2>/dev/null
            echo "ACG Bot stopped (PID: $acg_pid)"
        else
            echo "ACG Bot not running"
        fi
    fi
}

status_bot() {
    if [ -f "$PID_FILE" ]; then
        pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "ACG Bot running (PID: $pid)"
            echo ""
            echo "Recent log:"
            tail -10 "$LOG_FILE"
        else
            echo "ACG Bot not running (stale PID file)"
        fi
    else
        # Check for running ACG-specific process
        pid=$(get_civ_pid)
        if [ -n "$pid" ]; then
            echo "ACG Bot running (PID: $pid) - no PID file"
        else
            echo "ACG Bot not running"
        fi
    fi
}

case "${1:-status}" in
    start)
        start_bot
        ;;
    stop)
        stop_bot
        ;;
    restart)
        stop_bot
        sleep 1
        start_bot
        ;;
    status)
        status_bot
        ;;
    logs)
        tail -f "$LOG_FILE"
        ;;
    health)
        # Use the full service script for health
        "$SCRIPT_DIR/telegram_service.sh" health
        ;;
    recover)
        # Use the full service script for recovery
        "$SCRIPT_DIR/telegram_service.sh" recover
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|health|recover}"
        exit 1
        ;;
esac
