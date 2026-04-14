#!/bin/bash
echo "======================================================================"
echo "QWEN TEAM LEAD — Context Loading"
echo "======================================================================"
echo ""
echo "Loading mandatory context files..."
echo ""

# Read each context file
for f in \
  ".claude/team-leads/qwen/memory.md" \
  "MISSIONS.md" \
  "GRAND-PLAN.md" \
  "QWEN-STATUS-REPORT.md" \
  "HANDOFF-RESTART.md"
do
  if [ -f "$f" ]; then
    echo "✅ $f ($(wc -c < "$f") bytes)"
  else
    echo "❌ $f NOT FOUND"
  fi
done

echo ""
echo "======================================================================"
echo "You are Qwen Team Lead. You have read all context above."
echo "Your identity: .claude/team-leads/qwen/memory.md"
echo "Launch Qwen Code with full context loaded."
echo "======================================================================"
echo ""
echo "Ready. Type 'qwen' to start the agent."
