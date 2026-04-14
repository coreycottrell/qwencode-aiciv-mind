---
version: 2.0.0
author: coder
created: 2025-11-17
updated: 2025-12-26
line_count: 359
compliance_status: compliant
applicable_agents:
  - git-specialist
  - file-guardian
  - coder
  - auditor
activation_trigger: "file missing|files disappeared|manifest missing|where is file|recovery"
required_tools:
  - Bash
  - Read
category: git-operations
related_skills:
  - file-cleanup-protocol.md
  - git-archaeology/recovery.md
---

# Git Archaeology: Investigation Protocol for Missing Files

**Purpose**: Fast, systematic approach to finding missing files using git history (git first, filesystem second)
**Created from**: Nov 17 investigation failures (blogger/project-manager/tg-archi recovery)

---

## When to Use This Skill

**Trigger scenarios:**
- "Where is file X? I can't find it"
- "Agent Y isn't working, manifest missing"
- "Files that were here yesterday are gone"
- ANY investigation of missing/deleted files

**DO NOT use filesystem search first. Use git archaeology first.**

---

## The Golden Rule

```bash
# BEFORE doing ANYTHING else:
git log --all --full-history --oneline -- path/to/file

# This ONE command answers:
# - Did file ever exist? (git history shows commits)
# - When did it last exist? (most recent commit)
# - Is it gone now? (compare to current state)
```

**If you search the filesystem before checking git, you're doing it wrong.**

---

## The 5-Minute Investigation Process

### Step 1: Establish Baseline (1 minute)

```bash
# Does file have git history?
git log --all --oneline -- "path/to/file.md" | head -5

# If results: File WAS in git, LAST_GOOD_COMMIT is most recent
# If empty: File was NEVER committed, check Step 3
```

### Step 2: Find When It Disappeared (1 minute)

```bash
# Is file in current HEAD?
git ls-tree HEAD "path/to/file.md"

# Empty output = File was DELETED
# Find deletion commit:
git log --all --oneline --diff-filter=D -- "path/to/file.md"
```

### Step 3: Understand Why (1 minute)

```bash
# Read commit message
git show COMMIT --stat | head -20

# Check what else changed
git diff --name-status COMMIT~1 COMMIT | wc -l
# >30 files = Large commit, deletions may be buried
```

### Step 4: Verify Impact (1 minute)

```bash
# Check all deletions between commits
git diff LAST_GOOD_COMMIT HEAD --name-status | grep "^D"

# Check if pattern exists (e.g., multiple agents deleted)
git diff LAST_GOOD_COMMIT HEAD --name-status | grep "^D" | grep ".claude/agents"
```

### Step 5: Recovery (1 minute)

```bash
# Recover from last good commit
git checkout LAST_GOOD_COMMIT -- "path/to/file.md"

# Verify recovery
ls -lh "path/to/file.md"
head -20 "path/to/file.md"
git add "path/to/file.md"
```

**For advanced recovery scenarios, see `recovery.md`**

---

## Decision Tree

```
Did file exist in git history?
|-- YES (git log shows commits)
|   |-- Is it in current HEAD? (git ls-tree HEAD path)
|   |   |-- YES -> File is INTACT (no problem)
|   |   |-- NO -> File was DELETED
|   |       |-- Find deletion: git log --diff-filter=D
|   |       |-- Recover: git checkout LAST_GOOD -- path
|   |-- Additional checks:
|       |-- Renamed? git log --follow -- path
|       |-- Moved? git log --all -- "**/filename"
|
|-- NO (git log shows nothing)
    |-- Exists locally? (ls path)
    |   |-- YES -> File created but NOT committed
    |   |-- NO -> File NEVER existed
    |       |-- Check memory: ls memories/agents/name/
    |           |-- YES -> Agent worked but manifest never created
    |           |-- NO -> Agent never existed at all
```

---

## Common Scenarios

### Scenario 1: Agent manifest missing but memory exists
```bash
git log --all --oneline -- ".claude/agents/agent-name.md"
# Results? -> DELETED, recover from LAST_GOOD
# Empty? -> NEVER committed, need to create
```

### Scenario 2: Multiple files disappeared
```bash
for file in file1.md file2.md file3.md; do
    echo "=== $file ==="
    git log --oneline --diff-filter=D -- "$file" | head -1
done
# Same commit? -> Mass deletion, recover all together
```

### Scenario 3: File was here yesterday
```bash
git reflog | head -20
# Look for: reset, checkout, rebase, filter-branch
git stash list  # Check stashes
```

### Scenario 4: File exists but not where expected
```bash
git log --all --full-history -- "**/*filename*"
git log --follow -- new-path.md  # Check rename
```

---

## Anti-Patterns

| Anti-Pattern | Wrong | Right |
|--------------|-------|-------|
| Filesystem first | `find . -name "file.md"` | `git log --all --oneline -- "path/file.md"` |
| Assume same cause | "All missing = never created" | Check each file individually |
| Trust first conclusion | `git log -- file` (no results) | `git log --all --full-history -- file` |
| Ignore large commits | Skip commits with 100+ files | Check large commits FIRST |
| Search commands | `grep "rm" ~/.bash_history` | `git diff COMMIT1 COMMIT2 -- path` |

---

## Quick Reference Card

```bash
# Check git history
git log --all --oneline -- "path/to/file.md" | head -5

# Check current state
git ls-tree HEAD "path/to/file.md"

# Find deletion commit
git log --all --oneline --diff-filter=D -- "path/to/file.md"

# Compare commits
git diff COMMIT1 COMMIT2 -- "path/to/file.md"

# Commit details
git show COMMIT --stat

# Recover file
git checkout COMMIT -- "path/to/file.md"

# Search all history
git log --all --full-history -- "**/*filename*"

# Check rename
git log --follow -- "path/to/file.md"

# List deleted files in commit
git diff --name-status COMMIT~1 COMMIT | grep "^D"

# Recent operations
git reflog | head -20

# Find large commits
git log --stat | grep -B5 "100 files changed"
```

---

## Advanced Techniques (Git-Specialist)

### Content Search
```bash
# Find when specific text was removed
git log -S "specific_text" --source --all -- path/to/file

# Find when function disappeared
git log -G "class ClassName" --source --all -- path/to/file
```

### Track Renames
```bash
git log --follow --all --stat -- path/to/file.md
```

### Search Commit Messages
```bash
git log --all --grep="cleanup\|delete\|remove" --oneline
git log --all --author="agent-name" --oneline --name-status
```

### Reflog Archaeology
```bash
git log --walk-reflogs --all --oneline -- path/to/file.md
git log --diff-filter=A --all --oneline -- path/to/file.md  # Original add
```

### Lost Commits
```bash
git fsck --lost-found --no-reflogs
ls .git/lost-found/other/
git show OBJECT_SHA
```

---

## File-Guardian Integration: Proactive Monitoring

### Daily Deletion Detection
```bash
# Run as part of daily inventory
git diff HEAD@{yesterday} HEAD --name-status | grep "^D" | tee /tmp/daily-deletions.txt

# Alert on critical paths
git diff HEAD@{1.hour} HEAD --name-status | grep "^D" | \
  grep -E "(.claude/agents|memories/agents|.claude/CLAUDE.md)" && \
  echo "ALERT: Critical file deletion detected"
```

### Registry-Filesystem Alignment
```bash
# Compare registry vs actual files
jq -r '.agents[].id' memories/agents/agent_registry.json | \
  while read agent; do
    [[ ! -f ".claude/agents/${agent}.md" ]] && \
      echo "MISMATCH: Registry has $agent but manifest missing"
  done
```

### Orphaned Memory Detection
```bash
for memory_dir in memories/agents/*/; do
  agent_id=$(basename "$memory_dir")
  [[ ! -f ".claude/agents/${agent_id}.md" ]] && \
    echo "ORPHAN: Memory exists for $agent_id but no manifest"
done
```

---

## Verification Checklist

Before concluding investigation:
- [ ] Used `--all --full-history` flags
- [ ] Verified each file individually
- [ ] Checked for renames/moves (`--follow`)
- [ ] Checked large commits for buried deletions
- [ ] Used `git ls-tree` to confirm current state
- [ ] If recovered, verified file content

---

## Speed Benchmarks

| Case | Target Time |
|------|-------------|
| Simple (obvious commit) | 2 minutes |
| Medium (large commit) | 5 minutes |
| Complex (multiple files) | 10 minutes |

**If >15 minutes:** You're probably searching filesystem instead of git history. Restart with Step 1.

---

## Success Criteria

**Using correctly:**
- First command is `git log --all --full-history`
- Investigation <10 minutes
- You find root cause (when? why? by whom?)

**Using incorrectly:**
- Starting with `find` or `ls`
- >15 minutes for simple cases
- Making assumptions without verification

---

## Related Resources

| Resource | Purpose |
|----------|---------|
| `recovery.md` | Advanced recovery scenarios (rebase, force-push, stash) |
| `file-cleanup-protocol.md` | What to do BEFORE deleting |
| git-specialist | Delegate complex git operations |
| file-guardian | File system monitoring |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-17 | Initial from investigation lessons |
| 1.1 | 2025-11-17 | Added file-guardian + git-specialist contributions |
| 2.0.0 | 2025-12-26 | Refactored to skill family, split recovery to separate file |

---

**Remember: Git history is source of truth. Filesystem is just current snapshot.**

**When in doubt, git first.**
