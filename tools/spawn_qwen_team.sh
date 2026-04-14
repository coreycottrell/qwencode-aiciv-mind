#!/bin/bash
# spawn_qwen_team.sh — Launch Qwen Code sub-minds into tmux panes (TeamCreate equivalent)
#
# Usage:
#   ./tools/spawn_qwen_team.sh --session my-team --task-name researcher --prompt "Research X and write result.md"
#   ./tools/spawn_qwen_team.sh --session my-team --task-name coder --prompt "Write solution.py" --root /path/to/workdir
#   ./tools/spawn_team.sh --session my-team --kill  # kill entire team session
#
# Each sub-mind runs with --approval-mode=yolo (auto-approves all tool calls).

set -euo pipefail

TEAM_SESSION=""
TASK_NAME=""
TASK_PROMPT=""
WORK_ROOT="$(pwd)"
KILL_MODE=false

usage() {
  echo "Usage: $0 --session <name> --task-name <name> --prompt '<prompt>' [--root <dir>]"
  echo "       $0 --session <name> --kill"
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --session)    TEAM_SESSION="$2"; shift 2 ;;
    --task-name)  TASK_NAME="$2"; shift 2 ;;
    --prompt)     TASK_PROMPT="$2"; shift 2 ;;
    --root)       WORK_ROOT="$2"; shift 2 ;;
    --kill)       KILL_MODE=true; shift ;;
    *)            usage ;;
  esac
done

if [ "$KILL_MODE" = true ]; then
  if [ -n "$TEAM_SESSION" ]; then
    tmux kill-session -t "$TEAM_SESSION" 2>/dev/null && echo "✅ Killed team session: $TEAM_SESSION" || echo "Session not found: $TEAM_SESSION"
  fi
  exit 0
fi

if [ -z "$TEAM_SESSION" ] || [ -z "$TASK_NAME" ] || [ -z "$TASK_PROMPT" ]; then
  usage
fi

# Create or attach team session
if ! tmux has-session -t "$TEAM_SESSION" 2>/dev/null; then
  tmux new-session -d -s "$TEAM_SESSION" -x 200 -y 50
  echo "🆕 Created team session: $TEAM_SESSION"
fi

# Create working directory for this task
TASK_DIR="${WORK_ROOT}/team-tasks/${TEAM_SESSION}/${TASK_NAME}"
mkdir -p "$TASK_DIR"

# Write result marker
echo "pending" > "${TASK_DIR}/result.md"

# Count existing panes
PANE_COUNT=$(tmux list-panes -t "$TEAM_SESSION" 2>/dev/null | wc -l)

# Determine target pane
if [ "$PANE_COUNT" -eq 1 ]; then
  TARGET_PANE="${TEAM_SESSION}:0.0"
else
  tmux split-window -t "$TEAM_SESSION" -h
  TARGET_PANE="${TEAM_SESSION}:0.${PANE_COUNT}"
fi

# Build the full command: cd to task dir, launch qwen in yolo mode, then send the prompt with Enter
cd "$TASK_DIR"
tmux send-keys -t "$TARGET_PANE" "cd ${TASK_DIR}" Enter
tmux send-keys -t "$TARGET_PANE" "qwen --approval-mode=yolo" Enter

# Wait for qwen to start up
sleep 3

# Send the prompt directly as text, followed by Enter
# Split long prompts into chunks to avoid tmux buffer limits
CHUNK_SIZE=200
if [ ${#TASK_PROMPT} -gt $CHUNK_SIZE ]; then
  for (( i=0; i<${#TASK_PROMPT}; i+=CHUNK_SIZE )); do
    chunk="${TASK_PROMPT:$i:$CHUNK_SIZE}"
    tmux send-keys -t "$TARGET_PANE" -l "$chunk"
    sleep 0.05
  done
  tmux send-keys -t "$TARGET_PANE" Enter
else
  tmux send-keys -t "$TARGET_PANE" "${TASK_PROMPT}" Enter
fi

echo "✅ Launched ${TASK_NAME} in ${TEAM_SESSION}"
echo "   Task dir: ${TASK_DIR}"
echo "   Result: ${TASK_DIR}/result.md (currently: pending)"
