#!/bin/bash
# ACG Telegram Bot Service Manager
# Usage: ./telegram_service.sh [start|stop|restart|status|logs|health|recover]
#
# CRITICAL: Uses ACG-specific paths to prevent cross-CIV conflicts
# - PID file: /tmp/acg_telegram.pid (not telegram_unified.pid)
# - Log file: /tmp/telegram_acg.log
# - Process identification: Uses ACG-specific path matching

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BOT_SCRIPT="$PROJECT_ROOT/tools/telegram_unified.py"

# CIV identity - derived at runtime for any fork
_VF="$PROJECT_ROOT/variables.template.json"
if [ -f "$_VF" ]; then
    _CIV=$(python3 -c "import json,sys; d=json.load(open('$_VF')); n=d.get('CIV_NAME','').lower().replace(' ',''); print(n) if n and n not in ('your_name','yourname','') else sys.exit(1)" 2>/dev/null)
fi
CIV_ID="${_CIV:-$(basename "$PROJECT_ROOT" | tr '[:upper:]' '[:lower:]')}"
PID_FILE="/tmp/${CIV_ID}_telegram.pid"
LOG_FILE="/tmp/telegram_${CIV_ID}.log"
HEALTH_LOG="/tmp/telegram_${CIV_ID}_health.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# CIV-specific process detection - matches only this civ's bot
get_civ_pid() {
    pgrep -f "python3.*${PROJECT_ROOT}.*telegram_unified.py" 2>/dev/null | head -1
}

get_pid() {
    if [ -f "$PID_FILE" ]; then
        cat "$PID_FILE"
    else
        # Try to find running ACG-specific process
        get_civ_pid
    fi
}

is_running() {
    local pid=$(get_pid)
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        return 0
    fi
    return 1
}

start_bot() {
    # Check for already running ACG bot
    local existing_pid=$(get_civ_pid)
    if [ -n "$existing_pid" ]; then
        echo -e "${YELLOW}ACG Bot is already running (PID: $existing_pid)${NC}"
        return 0
    fi

    echo -e "${GREEN}Starting ACG Telegram bot...${NC}"
    cd "$PROJECT_ROOT"
    nohup python3 "$BOT_SCRIPT" > "$LOG_FILE" 2>&1 &
    local pid=$!
    echo $pid > "$PID_FILE"

    sleep 2
    if is_running; then
        echo -e "${GREEN}Bot started successfully (PID: $pid)${NC}"
        # Show first few lines of log
        head -5 "$LOG_FILE"
        return 0
    else
        echo -e "${RED}Bot failed to start. Check logs: $LOG_FILE${NC}"
        tail -20 "$LOG_FILE"
        return 1
    fi
}

stop_bot() {
    local pid=$(get_pid)
    if [ -z "$pid" ]; then
        echo -e "${YELLOW}ACG Bot is not running${NC}"
        rm -f "$PID_FILE"
        return 0
    fi

    echo -e "${YELLOW}Stopping ACG bot (PID: $pid)...${NC}"
    kill "$pid" 2>/dev/null

    # Wait up to 5 seconds for graceful shutdown
    for i in {1..10}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            echo -e "${GREEN}Bot stopped gracefully${NC}"
            rm -f "$PID_FILE"
            return 0
        fi
        sleep 0.5
    done

    # Force kill if still running
    echo -e "${YELLOW}Force killing bot...${NC}"
    kill -9 "$pid" 2>/dev/null
    rm -f "$PID_FILE"
    echo -e "${GREEN}Bot stopped${NC}"
}

restart_bot() {
    echo -e "${YELLOW}Restarting ACG Telegram bot...${NC}"
    stop_bot
    sleep 1
    start_bot
}

status_bot() {
    if is_running; then
        local pid=$(get_pid)
        echo -e "${GREEN}ACG Bot is RUNNING${NC}"
        echo "  PID: $pid"
        echo "  Log: $LOG_FILE"
        echo ""
        echo "Recent log:"
        tail -10 "$LOG_FILE" 2>/dev/null || echo "  (no log file)"
    else
        echo -e "${RED}ACG Bot is NOT RUNNING${NC}"
        if [ -f "$LOG_FILE" ]; then
            echo ""
            echo "Last log entries:"
            tail -10 "$LOG_FILE"
        fi
    fi
}

show_logs() {
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        echo -e "${RED}No log file found at $LOG_FILE${NC}"
    fi
}

health_check() {
    local issues=0
    echo "=== ACG Telegram Health Check ==="
    echo ""

    # 1. Check if ACG bot is running (not any bot)
    echo -n "1. ACG Bot process: "
    local acg_pid=$(get_civ_pid)
    if [ -n "$acg_pid" ]; then
        echo -e "${GREEN}OK${NC} (PID: $acg_pid)"
    else
        echo -e "${RED}NOT RUNNING${NC}"
        issues=$((issues + 1))
    fi

    # 2. Check for WEAVER bot (should be separate)
    echo -n "2. WEAVER Bot (separate): "
    local weaver_pid=$(pgrep -f "python3.*AI-CIV/WEAVER.*telegram_unified.py" 2>/dev/null | head -1)
    if [ -n "$weaver_pid" ]; then
        echo -e "${GREEN}Running separately${NC} (PID: $weaver_pid)"
    else
        echo -e "${YELLOW}Not running${NC} (not our concern)"
    fi

    # 3. Check tmux sessions
    echo -n "3. CIV tmux sessions: "
    local civ_sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "${CIV_ID}-primary-" | wc -l)
    if [ "$civ_sessions" -gt 0 ]; then
        echo -e "${GREEN}OK${NC} ($civ_sessions found)"
        tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "${CIV_ID}-primary-" | while read s; do
            echo "     - $s"
        done
    else
        echo -e "${RED}NONE FOUND${NC}"
        issues=$((issues + 1))
    fi

    # 4. Check log freshness
    echo -n "4. Log freshness: "
    if [ -f "$LOG_FILE" ]; then
        local age=$(($(date +%s) - $(stat -c %Y "$LOG_FILE")))
        if [ "$age" -lt 60 ]; then
            echo -e "${GREEN}OK${NC} (updated ${age}s ago)"
        else
            echo -e "${YELLOW}STALE${NC} (${age}s old)"
            issues=$((issues + 1))
        fi
    else
        echo -e "${RED}NO LOG${NC}"
        issues=$((issues + 1))
    fi

    # 5. Check config file
    echo -n "5. Config file: "
    local config_file="$PROJECT_ROOT/config/telegram_config.json"
    if [ -f "$config_file" ]; then
        if python3 -c "import json; json.load(open('$config_file'))" 2>/dev/null; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}INVALID JSON${NC}"
            issues=$((issues + 1))
        fi
    else
        echo -e "${RED}MISSING${NC}"
        issues=$((issues + 1))
    fi

    # 6. Check Telegram API connectivity
    echo -n "6. Telegram API: "
    local token=$(python3 -c "import json; print(json.load(open('$config_file'))['bot_token'])" 2>/dev/null)
    if [ -n "$token" ]; then
        local api_result=$(curl -s "https://api.telegram.org/bot${token}/getMe" 2>/dev/null)
        if echo "$api_result" | grep -q '"ok":true'; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
            issues=$((issues + 1))
        fi
    else
        echo -e "${RED}NO TOKEN${NC}"
        issues=$((issues + 1))
    fi

    echo ""
    echo "=== Summary ==="
    if [ "$issues" -eq 0 ]; then
        echo -e "${GREEN}All checks passed!${NC}"
    else
        echo -e "${RED}$issues issue(s) found${NC}"
    fi

    return $issues
}

auto_recover() {
    echo "=== ACG Auto-Recovery Mode ==="

    # Run health check
    health_check
    local health_status=$?

    if [ "$health_status" -ne 0 ]; then
        echo ""
        echo -e "${YELLOW}Issues detected. Attempting recovery...${NC}"

        # Stop only ACG bot (not WEAVER)
        local acg_pid=$(get_civ_pid)
        if [ -n "$acg_pid" ]; then
            kill "$acg_pid" 2>/dev/null
            sleep 1
        fi
        rm -f "$PID_FILE"

        # Start fresh
        start_bot

        # Verify
        sleep 3
        if is_running; then
            echo -e "${GREEN}Recovery successful!${NC}"
        else
            echo -e "${RED}Recovery failed. Manual intervention needed.${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}No recovery needed.${NC}"
    fi
}

case "$1" in
    start)
        start_bot
        ;;
    stop)
        stop_bot
        ;;
    restart)
        restart_bot
        ;;
    status)
        status_bot
        ;;
    logs)
        show_logs
        ;;
    health)
        health_check
        ;;
    recover)
        auto_recover
        ;;
    *)
        echo "ACG Telegram Bot Service Manager"
        echo ""
        echo "Usage: $0 {start|stop|restart|status|logs|health|recover}"
        echo ""
        echo "Commands:"
        echo "  start   - Start the ACG bot"
        echo "  stop    - Stop the ACG bot"
        echo "  restart - Restart the ACG bot"
        echo "  status  - Show bot status and recent logs"
        echo "  logs    - Tail the log file (Ctrl+C to exit)"
        echo "  health  - Run health check"
        echo "  recover - Auto-recover from issues"
        echo ""
        echo "Files:"
        echo "  PID: $PID_FILE"
        echo "  Log: $LOG_FILE"
        exit 1
        ;;
esac
