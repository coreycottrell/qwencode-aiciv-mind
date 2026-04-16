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

# Source project .env for Ollama credentials (needed for model auto-selection below)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [ -f "${PROJECT_ROOT}/.env" ]; then
  set -a && source "${PROJECT_ROOT}/.env" && set +a
fi

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
SESSION_FRESH=false
if ! tmux has-session -t "$TEAM_SESSION" 2>/dev/null; then
  tmux new-session -d -s "$TEAM_SESSION" -x 200 -y 50
  echo "🆕 Created team session: $TEAM_SESSION"
  SESSION_FRESH=true
fi

# Create working directory for this task
TASK_DIR="${WORK_ROOT}/team-tasks/${TEAM_SESSION}/${TASK_NAME}"
mkdir -p "$TASK_DIR"

# Write result marker
echo "pending" > "${TASK_DIR}/result.md"

# Count existing panes
PANE_COUNT=$(tmux list-panes -t "$TEAM_SESSION" 2>/dev/null | wc -l)

# Determine target pane: first ever spawn uses pane 0, all subsequent spawn new panes
if [ "$PANE_COUNT" -eq 1 ] && [ "$SESSION_FRESH" = true ]; then
  TARGET_PANE="${TEAM_SESSION}:0.0"
else
  tmux split-window -t "$TEAM_SESSION" -h
  TARGET_PANE="${TEAM_SESSION}:0.${PANE_COUNT}"
fi

# Source project .env for Ollama credentials (spawned minds need API access)
ENV_FILE=""
for candidate in "${WORK_ROOT}/.env" "${WORK_ROOT}/../.env" "${WORK_ROOT}/../../.env"; do
  if [ -f "$candidate" ]; then
    ENV_FILE="$candidate"
    break
  fi
done

# Build the full command: cd to task dir, launch qwen in yolo mode using LOCAL Ollama
# Local Ollama at localhost:11434 — no API key needed, no OAuth flow
# Auth type 'openai' with localhost base URL bypasses all cloud auth
cd "$TASK_DIR"
QWEN_CMD="cd ${TASK_DIR} && qwen --approval-mode=yolo --auth-type openai --openai-api-key local --openai-base-url http://localhost:11434/v1 --model qwen2.5:7b"
tmux send-keys -t "$TARGET_PANE" -- "${QWEN_CMD}" Enter

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
  tmux send-keys -t "$TARGET_PANE" -- Enter
else
  tmux send-keys -t "$TARGET_PANE" -l "${TASK_PROMPT}"
  tmux send-keys -t "$TARGET_PANE" -- Enter
fi

echo "✅ Launched ${TASK_NAME} in ${TEAM_SESSION}"
echo "   Task dir: ${TASK_DIR}"
echo "   Result: ${TASK_DIR}/result.md (currently: pending)"
