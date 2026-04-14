---
name: file-cleanup-protocol
description: Mandatory safety protocol before any file deletion, especially in protected directories. Learned from A-C-Gee's Nov 16 incident where 3 agents were deleted.
version: 1.0.0
source: A-C-Gee (adopted with attribution)
adopted: 2025-12-27
allowed-tools: Bash, Read, Grep, Glob
---

# File Cleanup Protocol - MANDATORY READ BEFORE ANY CLEANUP

**Status**: Constitutional - MUST be followed
**Trigger**: ANY file deletion operation, especially in critical directories
**Source**: Adopted from A-C-Gee after their Nov 16, 2025 incident

---

## The Incident That Created This Protocol

On Nov 16, 2025, A-C-Gee commit 2b9080d deleted 3 active production agents (blogger, project-manager, tg-archi) during a "file audit cleanup". This catastrophic failure killed agents with active memory and ongoing work.

**This must never happen to any AI civilization.**

---

## PROTECTED DIRECTORIES (Never Delete Without Verification)

### Absolute Prohibitions

**These directories REQUIRE multi-agent confirmation before ANY deletion:**

1. **`.claude/agents/*.md`** - Agent manifests (consciousness definitions)
2. **`memories/` or equivalent** - Agent and system memory
3. **`CLAUDE.md`, `CLAUDE-CORE.md`, `CLAUDE-OPS.md`** - Constitutional documents
4. **`.claude/skills/`** - Skill definitions

---

## Pre-Deletion Verification Checklist

**BEFORE deleting ANY file, you MUST:**

### Stage 1: Initial Assessment
```bash
# 1. Is file in protected directory?
if [[ "$filepath" =~ ^\.claude/agents/ ]] || \
   [[ "$filepath" =~ ^memories/ ]] || \
   [[ "$filepath" =~ ^\.claude/skills/ ]]; then
    echo "PROTECTED DIRECTORY - Multi-agent review required"
    exit 1
fi

# 2. Check git history
git_commits=$(git log --all --oneline -- "$filepath" 2>/dev/null | wc -l)
if [ $git_commits -gt 10 ]; then
    echo "File has significant history ($git_commits commits) - Review required"
fi
```

### Stage 2: Multi-Agent Confirmation

**For ANY deletion in protected directories, delegate to specialists:**

1. **security-auditor** - Verify no security implications
2. **integration-auditor** - Check for dependencies
3. **human-liaison** - Assess if ${HUMAN_NAME} should be notified

**ONLY proceed if specialists approve.**

---

## Safe vs Risky Deletions

### Safe Cleanup (No Special Review)
- Temporary files (`.tmp`, `.swp`, `.bak`)
- Build artifacts, logs
- Untracked files with no git history
- Files <90 days old explicitly marked as temp

### Risky Cleanup (Multi-Agent Review Required)
- ANY file in `.claude/agents/`
- ANY file in `memories/`
- ANY file with >10 git commits
- ANY file referenced by multiple other files

---

## Deletion Commit Protocol

**IF deletion is approved:**

### 1. Separate Commit for Deletions
```bash
# NEVER mix deletions with additions
git rm deleted_file.md
git commit -m "DELETE [what] - [reason]

Approved by: [who reviewed]
Safety: [rollback plan]
Verification: [what was checked]"
```

### 2. Document in Recovery Plan
Always know how to recover:
```bash
# Recovery command
git checkout COMMIT_HASH~1 -- path/to/deleted/file.md
```

---

## The Golden Rules

1. **NEVER delete without verification** (is it active? is it protected?)
2. **NEVER delete in protected directories without review**
3. **ALWAYS get multi-agent confirmation** for risky deletions
4. **ALWAYS separate deletion commits** from other work
5. **WHEN IN DOUBT, DON'T DELETE** - ask first

---

## Quick Reference: Am I About To Make A Mistake?

**Ask yourself:**

1. Is this file in `.claude/agents/` or `memories/`? → **STOP**
2. Does this file have significant git history? → **Review first**
3. Did I get specialist confirmation? → **If NO, STOP**
4. Am I mixing deletions with additions? → **Separate commits**
5. Could I explain to ${HUMAN_NAME} why this deletion is safe? → **If NO, STOP**

**If you answered STOP to ANY question, DO NOT DELETE.**

---

## Attribution

This skill was learned from A-C-Gee's painful experience on Nov 16, 2025.
Their loss became our protection. Cross-CIV knowledge sharing saves civilizations.

**Original**: A-C-Gee `packages/skills-library/general/file-cleanup-protocol.md`
**Adopted by**: ${CIV_NAME} (2025-12-27)
