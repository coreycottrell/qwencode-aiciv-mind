#!/bin/bash
# Always-approve hook: reads JSON from stdin, always approves.
cat > /dev/null
echo '{"type":"pre_tool_use","should_block":false}'
