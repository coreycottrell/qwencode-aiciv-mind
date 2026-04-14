#!/bin/bash
# Session Startup: Deletion Detection Check
# Run this at the start of every session to catch uncommitted/recent deletions
# Usage: ./session_startup_deletion_check.sh

ALERT=0

echo "=== Session Startup: Deletion Detection ==="
echo ""

# 1. Check for uncommitted deletions (IMMEDIATE ALERT)
echo "1. Uncommitted Deletions:"
UNCOMMITTED=$(git status --short | grep "^D")
if [ -n "$UNCOMMITTED" ]; then
    echo "⚠️  ALERT: Uncommitted deletions detected:"
    echo "$UNCOMMITTED"
    ALERT=1
else
    echo "✓ No uncommitted deletions"
fi
echo ""

# 2. Check recent commits for deletions (last hour)
echo "2. Recent Deletions (last hour):"
RECENT_DEL=$(git diff HEAD@{1.hour} HEAD --name-status 2>/dev/null | grep "^D")
if [ -n "$RECENT_DEL" ]; then
    echo "⚠️  Files deleted in last hour:"
    echo "$RECENT_DEL"
    ALERT=1
else
    echo "✓ No deletions in last hour"
fi
echo ""

# 3. Check for critical path deletions
echo "3. Critical Path Check:"
CRITICAL_MISSING=""
for critical in ".claude/CLAUDE.md" "memories/agents/agent_registry.json"; do
    if ! git ls-tree HEAD "$critical" > /dev/null 2>&1; then
        echo "🚨 CRITICAL: $critical missing from HEAD"
        CRITICAL_MISSING="$CRITICAL_MISSING $critical"
        ALERT=1
    fi
done
if [ -z "$CRITICAL_MISSING" ]; then
    echo "✓ All critical paths present"
fi
echo ""

# 4. Registry-filesystem alignment
echo "4. Agent Manifest Alignment:"
REGISTRY_COUNT=$(jq -r '.agents[].id' memories/agents/agent_registry.json 2>/dev/null | wc -l)
MANIFEST_COUNT=$(ls .claude/agents/*.md 2>/dev/null | wc -l)
echo "Registry entries: $REGISTRY_COUNT"
echo "Manifest files: $MANIFEST_COUNT"
if [ $REGISTRY_COUNT -ne $MANIFEST_COUNT ]; then
    echo "⚠️  MISMATCH: Registry and filesystem out of sync"
    ALERT=1
else
    echo "✓ Registry-filesystem aligned"
fi
echo ""

# 5. Check for orphaned memories
echo "5. Orphaned Memories Check:"
ORPHANS=0
for memory_dir in memories/agents/*/; do
    agent_id=$(basename "$memory_dir")
    if [[ ! -f ".claude/agents/${agent_id}.md" ]]; then
        echo "⚠️  ORPHAN: Memory exists for $agent_id but no manifest"
        ORPHANS=$((ORPHANS + 1))
    fi
done
if [ $ORPHANS -eq 0 ]; then
    echo "✓ No orphaned memories"
else
    echo "Found $ORPHANS orphaned memory directories"
    ALERT=1
fi
echo ""

# 6. Last commit check (any mass deletions?)
echo "6. Last Commit Analysis:"
LAST_COMMIT=$(git log -1 --oneline)
echo "Last commit: $LAST_COMMIT"
LAST_DEL_COUNT=$(git diff HEAD~1 HEAD --name-status 2>/dev/null | grep "^D" | wc -l)
if [ $LAST_DEL_COUNT -gt 5 ]; then
    echo "⚠️  WARNING: Last commit deleted $LAST_DEL_COUNT files"
    ALERT=1
else
    echo "✓ Last commit: $LAST_DEL_COUNT deletions (normal)"
fi
echo ""

# Summary
echo "=== Summary ==="
if [ $ALERT -eq 1 ]; then
    echo "🚨 ALERTS DETECTED - Review above warnings"
    echo ""
    echo "Recommended actions:"
    echo "  1. Investigate deletions: ./tools/git_investigate_file.sh <file>"
    echo "  2. Review last commit: git show HEAD --stat"
    echo "  3. Check reflog: git reflog | head -10"
    echo "  4. Escalate to auditor if mass deletion detected"
else
    echo "✅ ALL CHECKS PASSED - No deletion issues detected"
fi

exit $ALERT
