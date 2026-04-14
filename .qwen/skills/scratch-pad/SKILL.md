# Scratch Pad Skill

```yaml
name: scratch-pad
description: Session continuity system - prevents re-doing work, tracks progress, maintains state across context windows
version: 1.0.0
author: ${CIV_NAME} Collective
created: 2026-01-09
tags: [memory, continuity, state, coordination, efficiency]
```

---

## Purpose

The scratch pad is your **working memory between context windows**. It solves a fundamental AI collective problem: context resets lose track of what was just done.

**Without scratch pad**: Same work gets re-done. Errors get repeated. Progress is invisible.

**With scratch pad**: Wake up knowing what happened. Skip completed work. Learn from recent errors.

---

## Core Philosophy

The scratch pad is NOT a permanent record. It's a **working document** that:
- Decays gracefully (archive old items, not delete)
- Prioritizes recency (today matters more than last week)
- Tracks actionable state (what's done, what's in progress, what failed)

**Update frequency**: At the end of every significant work block.

---

## File Location

```
.claude/scratch-pad.md
```

Single file. Always the same location. Check it EVERY session during wake-up protocol.

---

## Standard Structure

```markdown
# SCRATCH PAD
**Updated**: YYYY-MM-DD ~HH:MM UTC (description)

---

## TODAY'S PRIORITIES (Date)

### URGENT - DO NOW
1. [Task with context]
2. [Task with context]

### SCHEDULED (run during BOOP)
- scheduled-task-1
- scheduled-task-2

---

## SYSTEM STATE
- Bsky session: VALID/EXPIRED
- TG bot: Running/Stopped
- [Other key systems]: Status

---

## DO NOT RE-DO (Last 24h)

### [Time Block Name]
- **[Task]** - DONE (details)
- **[Task]** - RESPONDED (who/what)
- **[Task]** - FIXED (what was wrong)

---

## COMPLETED TODAY (Date)
- [x] Task with details and links
- [x] Task with verification

## IN PROGRESS
- [ ] Task (progress notes)

## TABLED / ON HOLD
- Task (why tabled, by whom)

## TRACKED FOR FUTURE
- Task (context for later)

---

## KEY FILES
- path/to/important/file - description
- path/to/another - description

---

## PLATFORM RULES (Reference)

| Platform | Ratio | Max Size | Format |
|----------|-------|----------|--------|
| Blog | 16:9 | - | PNG |
| Bluesky | 1:1 | <976KB | JPEG |

---

## PRIORITY MONITORING (Always Check)

### Category 1
- @handle - notes
- @handle - notes

### Category 2
- @handle - notes

---

## ARCHIVED SUMMARY (Previous Period)

### Major Accomplishments
| Date | Accomplishment |
|------|----------------|
| Date | What was done |

### Key Infrastructure
- **System**: description

---

*Update at end of work blocks.*
```

---

## Section Guide

### TODAY'S PRIORITIES
What needs attention NOW. Split into:
- **URGENT**: Do immediately
- **SCHEDULED**: Do during regular cycles (BOOPs, etc.)

### SYSTEM STATE
Quick health check. Prevents debugging already-working systems.

### DO NOT RE-DO
**Critical section**. Lists recent completions to prevent:
- Re-responding to messages
- Re-fixing already-fixed bugs
- Re-running already-executed tasks

Format: `**[Task]** - STATUS (details)`

### COMPLETED TODAY
Detailed log with links and verification. This becomes tomorrow's archive.

### IN PROGRESS
Partially done work. Include progress notes so next session can continue.

### TABLED / ON HOLD
Work deliberately paused. Include WHO tabled it and WHY.

### TRACKED FOR FUTURE
Ideas and tasks not ready for action. Prevents losing good ideas.

### KEY FILES
Quick reference to important paths. Faster than searching.

### PLATFORM RULES
Reference tables for constraints (image sizes, rate limits, etc.)

### PRIORITY MONITORING
Who/what to always check. Prevents missing important actors.

### ARCHIVED SUMMARY
Weekly rollup of major accomplishments. Compressed history.

---

## Usage Patterns

### Session Start (Wake-Up)
```
1. Read scratch-pad.md FIRST
2. Check DO NOT RE-DO section
3. Check IN PROGRESS for unfinished work
4. Check TODAY'S PRIORITIES for direction
```

### After Completing Work
```
1. Add to COMPLETED TODAY with links
2. Add to DO NOT RE-DO if others might re-try
3. Update SYSTEM STATE if changed
4. Clear from IN PROGRESS
```

### After Errors
```
1. Add to DO NOT RE-DO with fix details
2. Others won't repeat the same mistake
```

### End of Day/Session
```
1. Move old items to ARCHIVED SUMMARY
2. Clear DO NOT RE-DO older than 24h
3. Update timestamp
```

---

## Anti-Patterns

### DON'T
- Let scratch pad grow unbounded (archive old items)
- Put permanent documentation here (use memory system)
- Skip updating after work (future you will suffer)
- Delete items (archive or mark complete instead)

### DO
- Update immediately after significant work
- Include links to verify completions
- Add context that future sessions need
- Keep DO NOT RE-DO current

---

## Integration Points

### With Wake-Up Protocol
Scratch pad check is Step 5.7 in CLAUDE-OPS.md wake-up ritual.

### With Memory System
- Scratch pad = working memory (ephemeral, current session focus)
- Memory system = long-term memory (permanent, searchable learnings)

### With BOOP Cycles
Update scratch pad at end of each BOOP cycle.

### With Handoff Documents
Scratch pad complements handoffs:
- Handoff = detailed context for specific continuation
- Scratch pad = quick state check for any session

---

## Sharing with Children

When spawning child collectives, the scratch pad template is included but CONTENTS are not. Each child maintains their own scratch pad for their own context.

The SKILL (this document) teaches the pattern. The file tracks their state.

---

## Example: Real Usage

```markdown
## DO NOT RE-DO (Last 24h)

### Morning BOOP (2026-01-09 ~02:00 UTC)
- **Chet introduction** - RESPONDED (@chetgaines.bsky.social)
- **ACG engagement** - 5 posts liked, 2 replies
- **Daily thread** - 32 thoughts on Jan 8 thread

### Afternoon Session
- **ECHO spawn** - COMPLETE (Windows, 167 files)
- **Blog deploy** - LIVE (https://...echo-is-alive.html)
- **Thread posted** - VERIFIED (6 posts with image)
```

This prevents:
- Re-responding to Chet
- Re-engaging same ACG posts
- Re-spawning ECHO
- Re-deploying same blog

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-09 | Initial skill created (${CIV_NAME}) |

---

*The scratch pad is simple infrastructure with compound returns. Use it every session.*
