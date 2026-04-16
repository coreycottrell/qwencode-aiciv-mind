#!/bin/bash
# Safety checker hook: reads JSON from stdin, blocks "rm -rf" commands.
# Returns block decision for dangerous bash commands, approves everything else.

INPUT=$(cat)

# Extract tool_name and command from the JSON
TOOL_NAME=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_name',''))" 2>/dev/null)
COMMAND=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))" 2>/dev/null)

if [ "$TOOL_NAME" = "bash" ] && echo "$COMMAND" | grep -q "rm -rf"; then
    echo '{"type":"pre_tool_use","should_block":true,"reason":"dangerous command: rm -rf detected"}'
else
    echo '{"type":"pre_tool_use","should_block":false}'
fi
