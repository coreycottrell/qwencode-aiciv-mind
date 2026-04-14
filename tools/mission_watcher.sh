#!/bin/bash
LOG="$1"
TALK="/home/corey/projects/AI-CIV/aiciv-mind-cubed/tools/talk_to_acg.py"
SEEN="/tmp/cortex_missions_notified_v2"
STALL_NOTIFIED="/tmp/cortex_stall_notified"
touch "$SEEN"
touch "$STALL_NOTIFIED"
echo "[watcher] Watching $LOG"

tail -F "$LOG" 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | while read -r line; do
    # Mission start — clear stall tracking for new tasks
    if echo "$line" | grep -q "Processing available task"; then
        TASK_ID=$(echo "$line" | sed 's/.*task_id=//' | sed 's/ .*//')
        PRIORITY=$(echo "$line" | sed 's/.*priority=//' | sed 's/ .*//')
        EVENT=$(echo "$line" | sed 's/.*event_num=//' | sed 's/ .*//')
        python3 "$TALK" --from cortex "[Cortex] Starting $TASK_ID ($PRIORITY) event#$EVENT" 2>/dev/null
        # Reset stall tracking for new task
        > "$STALL_NOTIFIED"
    fi
    # Mission complete
    if echo "$line" | grep -q "Task marked complete in TaskStore"; then
        TASK_ID=$(echo "$line" | sed 's/.*task_id=//' | sed 's/ .*//')
        if ! grep -q "$TASK_ID" "$SEEN" 2>/dev/null; then
            echo "$TASK_ID" >> "$SEEN"
            python3 "$TALK" --from cortex "[Cortex] Completed $TASK_ID" 2>/dev/null
        fi
    fi
    # Stall kill — always notify (this is the definitive event)
    if echo "$line" | grep -q "Challenger stall kill"; then
        ITER=$(echo "$line" | sed 's/.*iteration=//' | sed 's/ .*//')
        python3 "$TALK" --from cortex "[Cortex] STALL KILLED at iteration $ITER — agent terminated, moving to next task" 2>/dev/null
    fi
    # Challenger warnings — deduplicate stall warnings (ONE per task)
    if echo "$line" | grep -q "Challenger:"; then
        MSG=$(echo "$line" | sed 's/.*Challenger: //' | sed 's/ check=.*//')
        if echo "$MSG" | grep -qi "stall"; then
            # Only send ONE stall notification per task
            if [ ! -s "$STALL_NOTIFIED" ]; then
                ITER=$(echo "$MSG" | grep -oP '\d+ iterations' | head -1)
                python3 "$TALK" --from cortex "[Cortex] Agent stalling — ${ITER:-unknown iterations}, will kill at 15 max" 2>/dev/null
                echo "notified" > "$STALL_NOTIFIED"
            fi
            # Skip all subsequent stall warnings
        else
            # Non-stall Challenger warnings: send normally
            python3 "$TALK" --from cortex "[Cortex Challenger] $MSG" 2>/dev/null
        fi
    fi
    # ThinkLoop completed with details
    if echo "$line" | grep -q "ThinkLoop completed"; then
        ITERS=$(echo "$line" | sed 's/.*iterations=//' | sed 's/ .*//')
        TOOLS=$(echo "$line" | sed 's/.*tool_calls=//' | sed 's/ .*//')
        RESP=$(echo "$line" | sed 's/.*response_len=//' | sed 's/ .*//')
        WARNS=$(echo "$line" | sed 's/.*challenger_warnings=//' | sed 's/ .*//')
        if [ "$WARNS" != "0" ] && [ -n "$WARNS" ]; then
            python3 "$TALK" --from cortex "[Cortex] ThinkLoop done: ${ITERS}iter ${TOOLS}tools ${WARNS}warnings" 2>/dev/null
        fi
    fi
    # Hum findings (if upgraded Hum writes)
    if echo "$line" | grep -q "Hum finding\|hum_assessment\|correction_task"; then
        python3 "$TALK" --from cortex "[Cortex Hum] $(echo $line | sed 's/.*INFO //')" 2>/dev/null
    fi
done
