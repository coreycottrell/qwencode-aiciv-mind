#!/bin/bash
# send_to_civ.sh — Send a message to another civ via tmux injection.
#
# Usage:
#   ./tools/send_to_civ.sh acg "Your message here"
#   ./tools/send_to_civ.sh acg --file /path/to/message.txt
#   ./tools/send_to_civ.sh proof "Your message here"
#   ./tools/send_to_civ.sh --list                    # list known civs
#
# Civs are defined in ~/.aiciv_civ_registry.json or via --pane flag.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REGISTRY_FILE="${HOME}/.aiciv_civ_registry.json"

# Default civ registry
# Format: {"civ_name": {"session": "...", "pane": "..."}}
DEFAULT_REGISTRY='{
  "acg": {"session": "acg-primary-20260411-053150", "pane": "0"},
  "proof": {"session": "proof-primary-20260414-072027", "pane": "0"},
  "qwen": {"session": "qwen-primary-20260411-055602", "pane": "0"}
}'

# Parse arguments
CIV_NAME=""
MESSAGE=""
FILE_INPUT=""
CUSTOM_PANE=""
LIST_MODE=false

usage() {
  echo "Usage: $0 <civ_name> <message>"
  echo "       $0 <civ_name> --file <path>"
  echo "       $0 --list"
  echo ""
  echo "Known civs (from registry):"
  echo "$DEFAULT_REGISTRY" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for name, info in data.items():
    print(f'  {name:12s} → {info[\"session\"]}:{info[\"pane\"]}')
" 2>/dev/null || echo "  (registry parse error)"
  exit 1
}

if [ $# -eq 0 ]; then
  usage
fi

# Check for --list
if [ "${1:-}" = "--list" ]; then
  echo "Known civs:"
  echo "$DEFAULT_REGISTRY" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for name, info in data.items():
    print(f'  {name:12s} → {info[\"session\"]}:{info[\"pane\"]}')
" 2>/dev/null
  exit 0
fi

CIV_NAME="$1"
shift

# Parse remaining args
if [ "${1:-}" = "--file" ]; then
  FILE_INPUT="${2:?Error: --file requires a path argument}"
  if [ ! -f "$FILE_INPUT" ]; then
    echo "Error: File not found: $FILE_INPUT" >&2
    exit 1
  fi
  MESSAGE="$(cat "$FILE_INPUT")"
else
  MESSAGE="$*"
fi

if [ -z "$MESSAGE" ]; then
  echo "Error: Empty message. Use '$0 --list' to see known civs." >&2
  exit 1
fi

# Look up civ target
PANE_TARGET=""
if [ -n "$CUSTOM_PANE" ]; then
  PANE_TARGET="$CUSTOM_PANE"
elif [ -f "$REGISTRY_FILE" ]; then
  PANE_TARGET="$(python3 -c "
import json
with open('$REGISTRY_FILE') as f:
    data = json.load(f)
civ = data.get('$CIV_NAME', {})
if civ:
    print(f\"{civ.get('session', '')}:{civ.get('pane', '')}\")
else:
    print('')
" 2>/dev/null)"
fi

# Fall back to default registry
if [ -z "$PANE_TARGET" ]; then
  PANE_TARGET="$(echo "$DEFAULT_REGISTRY" | python3 -c "
import sys, json
data = json.load(sys.stdin)
civ = data.get('$CIV_NAME', {})
if civ:
    print(f\"{civ['session']}:{civ['pane']}\")
" 2>/dev/null)"
fi

# Verify tmux session exists
SESSION="${PANE_TARGET%%:*}"
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Error: tmux session '$SESSION' not found for civ '$CIV_NAME'." >&2
  echo "Available sessions:"
  tmux list-sessions 2>/dev/null || echo "  (no tmux sessions running)" >&2
  exit 1
fi

# Send the message
PREFIX="[from ${QWEN_SENDER_CIV:-qwen}] "
FORMATTED="${PREFIX}${MESSAGE}"

echo "Sending to $CIV_NAME ($PANE_TARGET):"
echo "  ${MESSAGE:0:100}..."

# Chunk large messages (tmux input buffer limit)
CHUNK_SIZE=100
if [ ${#FORMATTED} -gt $CHUNK_SIZE ]; then
  for (( i=0; i<${#FORMATTED}; i+=CHUNK_SIZE )); do
    chunk="${FORMATTED:$i:$CHUNK_SIZE}"
    tmux send-keys -t "$PANE_TARGET" -l "$chunk"
    sleep 0.05
  done
else
  tmux send-keys -t "$PANE_TARGET" -l "$FORMATTED"
fi

# Send Enter to execute
tmux send-keys -t "$PANE_TARGET" "Enter"

# Log the message
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/send_to_civ.log"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "{\"timestamp\":\"$TIMESTAMP\",\"civ\":\"$CIV_NAME\",\"pane\":\"$PANE_TARGET\",\"chars\":${#MESSAGE}}" >> "$LOG_FILE"

echo "✅ Sent to $CIV_NAME at $PANE_TARGET (${#MESSAGE} chars)"
