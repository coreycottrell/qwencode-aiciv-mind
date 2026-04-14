---
name: session-summary
description: Automated session-start context loading via git analysis, file tracking, and pattern detection
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Bash, Grep, Glob, Write]
agents-required: [code-archaeologist, pattern-detector, doc-synthesizer]
---

# Session Summary Skill

An automated context-loading flow that analyzes git commits, files changed, and patterns created to generate comprehensive session-start summaries. Enables rapid context recovery after autonomous work or session breaks.

## When to Use

**Invoke when**:
- Starting a new session (context recovery)
- After long autonomous work periods
- Before handoff to ${HUMAN_NAME} or another agent
- Context loading needed quickly
- Understanding "what happened while I was away"
- Preparing session summary for human consumption

**Do not use when**:
- No recent activity to summarize
- Already have full context (just worked on it)
- Need real-time monitoring (this is point-in-time)

## Prerequisites

**Agents Required**:
- **code-archaeologist** - Analyzes git history and file evolution
- **pattern-detector** - Identifies patterns in changes
- **doc-synthesizer** - Creates readable summary document

**Context Needed**:
- Git repository with recent commits
- File system access
- Time range to analyze (default: since last session)

**Tool Available**:
```bash
python tools/session_summary.py
```

## Procedure

### Step 1: Git Analysis
**Duration**: ~2 minutes
**Agent(s)**: code-archaeologist (or automated tool)

Analyze recent git activity:

```bash
# Get recent commits
git log --oneline -20

# Get files changed in last N hours
git diff --name-only HEAD~10

# Get commit messages for context
git log --format="%h %s" -10
```

1. Count commits in time range
2. Categorize by type (feature, fix, refactor, docs)
3. Identify major changes
4. Note any large commits

**Deliverable**: Git activity summary

---

### Step 2: File Change Analysis
**Duration**: ~3 minutes
**Agent(s)**: pattern-detector

Categorize files changed:

1. Group files by directory/domain
2. Identify new files vs. modified
3. Note deleted files
4. Calculate change velocity

```bash
# Files by category
find . -name "*.md" -mmin -360 | wc -l  # Docs in last 6h
find . -name "*.py" -mmin -360 | wc -l  # Code in last 6h
```

**Deliverable**: Categorized file changes

---

### Step 3: Pattern Detection
**Duration**: ~5 minutes
**Agent(s)**: pattern-detector

Identify patterns in recent work:

1. What themes emerge from changes?
2. Any new patterns created?
3. Infrastructure changes detected?
4. Recurring file clusters?

**Deliverable**: Detected patterns list

---

### Step 4: Summary Generation
**Duration**: ~5 minutes
**Agent(s)**: doc-synthesizer

Create human-readable summary:

1. Executive summary (1-2 sentences)
2. Key accomplishments
3. Files modified by category
4. Patterns created/modified
5. Suggested next steps
6. Open questions

**Deliverable**: Session summary document

---

### Step 5: Output
**Duration**: ~1 minute
**Agent(s)**: The Conductor

Save and share summary:

1. Write to standard location: `/to-${HUMAN_NAME_LOWER}/SESSION-SUMMARY-YYYY-MM-DD.md`
2. Update latest symlink if applicable
3. Optionally email to ${HUMAN_NAME}

**Deliverable**: Summary saved and accessible

---

## Parallelization

**Can run in parallel**:
- Steps 1-3 can all run simultaneously (independent analysis)

**Must be sequential**:
- Steps 1-3 before Step 4 (analyses needed for summary)
- Step 4 before Step 5 (summary needed before output)

## Success Indicators

- [ ] Git commits analyzed and categorized
- [ ] Files modified listed by domain/type
- [ ] Patterns created/modified identified
- [ ] Human-readable summary generated
- [ ] Inferred recent work themes captured
- [ ] Suggested next steps provided
- [ ] Summary saved to standard location

## Example

**Scenario**: Morning context recovery after 6 hours of autonomous work

```
Step 1 (Git):
  - 5 commits in last 6 hours
  - Types: 2 features, 1 fix, 2 docs
  - Largest: "Add memory system" (23 files)

Step 2 (Files):
  - 123 files modified
  - Categories:
    - .claude/: 45 files (infrastructure)
    - tools/: 12 files (Python)
    - docs/: 19 files (documentation)
    - flows/: 8 files (coordination)

Step 3 (Patterns):
  - 19 new patterns created
  - Theme: Memory infrastructure buildout
  - Recurring cluster: memory_core.py + test files

Step 4 (Summary):
  "SESSION SUMMARY - 2025-10-05

   Executive: Major memory system buildout with 19 new patterns.

   Accomplishments:
   - Memory system v1.0 complete
   - 5 flows validated
   - Agent infrastructure expanded

   Next Steps:
   - Test memory search
   - Validate flow combinations
   - Email summary to ${HUMAN_NAME}"

Step 5 (Output):
  Saved to: /to-${HUMAN_NAME_LOWER}/SESSION-SUMMARY-2025-10-05.md

Result: Comprehensive context recovered in ~15 minutes
        Ready to continue work with full awareness
```

## Automated Tool

The `session_summary.py` tool automates much of this:

```bash
# Generate summary for last 6 hours
python tools/session_summary.py --hours 6

# Generate summary since specific commit
python tools/session_summary.py --since abc123

# Generate and email to ${HUMAN_NAME}
python tools/session_summary.py --email
```

## Notes

- **Typical Duration**: 10-15 minutes
- **Error Handling**: If git history unavailable, fall back to file timestamps
- **Evolution**: Consider real-time streaming instead of point-in-time
- **Key Insight**: Context recovery is essential for session continuity
- **The "123 files, 5 commits, 19 patterns" metric**: Concrete numbers aid comprehension
- **Handoff Documentation**: Critical for human-AI partnership
- **Standard Location**: Always save to `/to-${HUMAN_NAME_LOWER}/` for discoverability

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 14: Session Summary)
**Original Status**: VALIDATED (2025-10-05)
**Conversion Date**: 2025-12-27
