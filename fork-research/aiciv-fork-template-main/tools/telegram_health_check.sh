#!/bin/bash
# Telegram Integration Health Check
# Used by tg-archi during wake-up to verify Telegram infrastructure
# Returns: JSON status for easy parsing + human-readable output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# CIV identity - derived at runtime for any fork
_VF="$PROJECT_ROOT/variables.template.json"
if [ -f "$_VF" ]; then
    _CIV=$(python3 -c "import json,sys; d=json.load(open('$_VF')); n=d.get('CIV_NAME','').lower().replace(' ',''); print(n) if n and n not in ('your_name','yourname','') else sys.exit(1)" 2>/dev/null)
fi
CIV_ID="${_CIV:-$(basename "$PROJECT_ROOT" | tr '[:upper:]' '[:lower:]')}"

# CIV-specific paths
LOG_FILE="/tmp/telegram_${CIV_ID}.log"
PID_FILE="/tmp/${CIV_ID}_telegram.pid"
CONFIG_FILE="$PROJECT_ROOT/config/telegram_config.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Status tracking
ISSUES=()
WARNINGS=()

echo "========================================"
echo "Telegram Integration Health Check"
echo "========================================"
echo ""

# 1. Check if config exists
echo -n "1. Config file... "
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}OK${NC}"

    # Validate JSON
    if python3 -c "import json; json.load(open('$CONFIG_FILE'))" 2>/dev/null; then
        echo "   Config JSON valid"
    else
        echo -e "   ${RED}Config JSON invalid!${NC}"
        ISSUES+=("Config JSON is invalid")
    fi

    # Check required fields
    if python3 -c "import json; c=json.load(open('$CONFIG_FILE')); assert c.get('bot_token'); assert c.get('chat_id')" 2>/dev/null; then
        echo "   Required fields present"
    else
        echo -e "   ${RED}Missing bot_token or chat_id!${NC}"
        ISSUES+=("Missing required config fields")
    fi
else
    echo -e "${RED}MISSING${NC}"
    ISSUES+=("Config file not found: $CONFIG_FILE")
fi
echo ""

# 2. Check if bot process is running
echo -n "2. Bot process... "
BOT_PID=""
if [ -f "$PID_FILE" ]; then
    BOT_PID=$(cat "$PID_FILE")
    if kill -0 "$BOT_PID" 2>/dev/null; then
        echo -e "${GREEN}RUNNING${NC} (PID: $BOT_PID)"
    else
        echo -e "${RED}DEAD${NC} (stale PID file)"
        rm -f "$PID_FILE"
        BOT_PID=""
        ISSUES+=("Bot not running (stale PID file)")
    fi
else
    # Try to find by process name
    BOT_PID=$(pgrep -f "telegram_unified.py" | head -1)
    if [ -n "$BOT_PID" ]; then
        echo -e "${GREEN}RUNNING${NC} (PID: $BOT_PID, no PID file)"
        echo "$BOT_PID" > "$PID_FILE"
        WARNINGS+=("Bot running but no PID file (fixed)")
    else
        echo -e "${RED}NOT RUNNING${NC}"
        ISSUES+=("Bot not running")
    fi
fi
echo ""

# 3. Check log file and recent activity
echo -n "3. Log file... "
if [ -f "$LOG_FILE" ]; then
    echo -e "${GREEN}EXISTS${NC}"

    # Check last modified time
    LOG_AGE=$(( $(date +%s) - $(stat -c %Y "$LOG_FILE") ))
    if [ $LOG_AGE -lt 60 ]; then
        echo "   Last activity: ${LOG_AGE}s ago (healthy)"
    elif [ $LOG_AGE -lt 300 ]; then
        echo -e "   ${YELLOW}Last activity: ${LOG_AGE}s ago (may be stale)${NC}"
        WARNINGS+=("Log not updated in ${LOG_AGE}s")
    else
        echo -e "   ${RED}Last activity: ${LOG_AGE}s ago (stale!)${NC}"
        ISSUES+=("Log very stale (${LOG_AGE}s)")
    fi

    # Check for errors in recent log
    RECENT_ERRORS=$(tail -50 "$LOG_FILE" | grep -i "error\|exception\|failed" | wc -l)
    if [ "$RECENT_ERRORS" -gt 0 ]; then
        echo -e "   ${YELLOW}Recent errors: $RECENT_ERRORS${NC}"
        WARNINGS+=("$RECENT_ERRORS errors in recent log")
    else
        echo "   No recent errors"
    fi
else
    echo -e "${YELLOW}NOT FOUND${NC} (bot may not have started)"
    if [ -n "$BOT_PID" ]; then
        WARNINGS+=("Bot running but no log file")
    fi
fi
echo ""

# 4. Check tmux session detection
echo -n "4. tmux session... "
CURRENT_SESSION=""
if [ -f "$PROJECT_ROOT/.current_session" ]; then
    CURRENT_SESSION=$(cat "$PROJECT_ROOT/.current_session")
    if tmux has-session -t "$CURRENT_SESSION" 2>/dev/null; then
        echo -e "${GREEN}OK${NC} ($CURRENT_SESSION)"
    else
        echo -e "${YELLOW}MARKER STALE${NC} ($CURRENT_SESSION not found)"

        # Try to find any ACG session
        LATEST_SESSION=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "^${CIV_ID}-primary-" | sort | tail -1)
        if [ -n "$LATEST_SESSION" ]; then
            echo "   Found alternative: $LATEST_SESSION"
            WARNINGS+=("Session marker stale, found: $LATEST_SESSION")
        else
            ISSUES+=("No ${CIV_ID} tmux session found")
        fi
    fi
else
    # Try to find session anyway
    LATEST_SESSION=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "^${CIV_ID}-primary-" | sort | tail -1)
    if [ -n "$LATEST_SESSION" ]; then
        echo -e "${GREEN}OK${NC} ($LATEST_SESSION, no marker file)"
        WARNINGS+=("No session marker file")
    else
        echo -e "${RED}NO SESSION${NC}"
        ISSUES+=("No ${CIV_ID} tmux session found")
    fi
fi
echo ""

# 5. Check Claude log session
echo -n "5. Claude session... "
CLAUDE_HISTORY="$HOME/.claude/history.jsonl"
if [ -f "$CLAUDE_HISTORY" ]; then
    CLAUDE_SESSION=$(tail -100 "$CLAUDE_HISTORY" | grep "/AI-CIV/ACG" | tail -1 | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('sessionId','')[:20])" 2>/dev/null || echo "")
    if [ -n "$CLAUDE_SESSION" ]; then
        echo -e "${GREEN}OK${NC} (${CLAUDE_SESSION}...)"
    else
        echo -e "${YELLOW}NOT FOUND${NC} (no ACG session in history)"
        WARNINGS+=("No Claude ACG session in history")
    fi
else
    echo -e "${RED}NO HISTORY${NC}"
    ISSUES+=("Claude history file not found")
fi
echo ""

# 6. Test Telegram API connectivity (quick check)
echo -n "6. Telegram API... "
if [ -f "$CONFIG_FILE" ]; then
    BOT_TOKEN=$(python3 -c "import json; print(json.load(open('$CONFIG_FILE')).get('bot_token',''))" 2>/dev/null)
    if [ -n "$BOT_TOKEN" ]; then
        API_RESULT=$(curl -s --connect-timeout 5 "https://api.telegram.org/bot${BOT_TOKEN}/getMe" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print('OK' if d.get('ok') else 'FAIL')" 2>/dev/null || echo "FAIL")
        if [ "$API_RESULT" = "OK" ]; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
            ISSUES+=("Telegram API not responding")
        fi
    else
        echo -e "${YELLOW}SKIP${NC} (no token)"
    fi
else
    echo -e "${YELLOW}SKIP${NC} (no config)"
fi
echo ""

# Summary
echo "========================================"
echo "SUMMARY"
echo "========================================"

if [ ${#ISSUES[@]} -eq 0 ] && [ ${#WARNINGS[@]} -eq 0 ]; then
    echo -e "${GREEN}All checks passed!${NC}"
    echo ""
    echo "Bot is healthy and streaming Claude responses to Telegram."
    EXIT_CODE=0
elif [ ${#ISSUES[@]} -eq 0 ]; then
    echo -e "${YELLOW}Healthy with warnings:${NC}"
    for w in "${WARNINGS[@]}"; do
        echo "  - $w"
    done
    EXIT_CODE=0
else
    echo -e "${RED}Issues found:${NC}"
    for i in "${ISSUES[@]}"; do
        echo "  - $i"
    done
    if [ ${#WARNINGS[@]} -gt 0 ]; then
        echo ""
        echo -e "${YELLOW}Warnings:${NC}"
        for w in "${WARNINGS[@]}"; do
            echo "  - $w"
        done
    fi
    EXIT_CODE=1
fi

echo ""
echo "========================================"

# Auto-fix suggestion
if [ ${#ISSUES[@]} -gt 0 ]; then
    echo ""
    echo "To fix issues, run:"
    echo "  $PROJECT_ROOT/tools/start_telegram_bot.sh restart"
    echo ""
fi

exit $EXIT_CODE
