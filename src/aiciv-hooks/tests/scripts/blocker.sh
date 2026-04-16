#!/bin/bash
# Always-block hook: reads JSON from stdin, always blocks.
cat > /dev/null
echo '{"type":"pre_tool_use","should_block":true,"reason":"blocked by chain hook 3"}'
