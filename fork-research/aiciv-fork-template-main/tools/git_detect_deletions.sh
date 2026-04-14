#!/bin/bash
# Git Archaeology Helper: Pre-commit deletion detector
# Install as pre-commit hook or run before committing

echo "=== Checking for dangerous deletions ==="

DELETED=$(git diff --cached --name-status | grep "^D")

if [ -z "$DELETED" ]; then
    echo "✓ No deletions staged"
    exit 0
fi

echo "Files being deleted:"
echo "$DELETED"
echo ""

# Check for agent manifest deletions
AGENT_DELETIONS=$(echo "$DELETED" | grep "\.claude/agents/.*\.md$")
if [ -n "$AGENT_DELETIONS" ]; then
    echo "⚠️  WARNING: Agent manifests being deleted:"
    echo "$AGENT_DELETIONS"
    echo ""
    echo "Have you:"
    echo "  1. Checked for democratic vote approval?"
    echo "  2. Read .claude/skills/file-cleanup-protocol.md?"
    echo "  3. Got approval from auditor + file-guardian + human-liaison?"
    echo ""
    echo "Proceed with deletion? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "❌ Aborted. Files NOT deleted."
        exit 1
    fi
fi

# Check for memory deletions
MEMORY_DELETIONS=$(echo "$DELETED" | grep "memories/agents/")
if [ -n "$MEMORY_DELETIONS" ]; then
    echo "⚠️  WARNING: Agent memory being deleted:"
    echo "$MEMORY_DELETIONS"
    echo ""
    echo "Agent memory deletions require vote. Proceed? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "❌ Aborted. Files NOT deleted."
        exit 1
    fi
fi

# Check for constitutional deletions
CONSTITUTIONAL_DELETIONS=$(echo "$DELETED" | grep "\.claude/CLAUDE\.md\|agent_registry\.json")
if [ -n "$CONSTITUTIONAL_DELETIONS" ]; then
    echo "🚨 CRITICAL: Constitutional files being deleted:"
    echo "$CONSTITUTIONAL_DELETIONS"
    echo ""
    echo "This requires 90% vote + Corey approval. Abort? (Y/n)"
    read -r response
    if [[ ! "$response" =~ ^[Nn]$ ]]; then
        echo "❌ Aborted. Critical files protected."
        exit 1
    fi
fi

echo "✓ Deletion checks passed"
exit 0
