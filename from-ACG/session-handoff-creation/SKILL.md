---
name: session-handoff-creation
version: 1.0.0
author: skills-master
created: 2025-12-15
last_updated: 2025-12-26
line_count: 190
compliance_status: compliant

description: End-of-session protocol for creating proper handoff documents. Ensures continuity across sessions by documenting achievements, blockers, next steps, and critical context that future iterations need to resume work effectively.

applicable_agents:
  - primary

activation_trigger: |
  Load this skill when:
  - End of every work session (MANDATORY)
  - Before context compaction
  - When switching major focus areas
  - After major achievements

required_tools:
  - Write
  - Read

category: custom

depends_on: []
---

# Session Handoff Creation

## Purpose

Enable seamless session continuity by creating comprehensive handoff documents that allow the next Primary AI iteration to wake up oriented and productive. Handoffs are civilization memory - they preserve context that would otherwise be lost.

## When to Use

- End of every work session (MANDATORY)
- Before context compaction
- When switching major focus areas
- Before extended breaks (creator unavailable)
- After major achievements (capture while fresh)
- When blockers require external resolution

## Procedure

### 1. Gather Session Information

Before writing, collect:
- What was accomplished (deliverables, files modified)
- What was attempted but blocked (and why)
- What's next (priorities, pending decisions)
- What's critical (context that would be lost)

### 2. Choose Handoff Type

**Standard Session Handoff:**
- Normal end-of-session
- Filename: `HANDOFF-[FOCUS]-[YYYYMMDD].md`
- Location: `.claude/memory/handoffs/`

**Interim Handoff:**
- Mid-session checkpoint (before risky operation)
- Filename: `INTERIM-HANDOFF-[YYYYMMDD]-[TOPIC].md`
- Location: `.claude/memory/handoffs/`

### 3. Write Handoff Document

**Required sections:**

```markdown
# HANDOFF: [Brief Title]

**Date:** YYYY-MM-DD [Time Period]
**Status:** [Complete/Partial/Blocked]
**Trigger:** [Why session ended - normal end, compaction, blocker]

---

## Summary of Achievements

[1-3 sentences describing what was accomplished]

### Deliverable 1
**Status:** [BUILT/COMPLETE/PARTIAL/BLOCKED]
**Files:**
- `/absolute/path/to/file1.py` - [brief description]
- `/absolute/path/to/file2.md` - [brief description]

**Key Details:**
[Specific information next iteration needs]

### Deliverable 2
[Same format...]

---

## Critical Notes

### [Topic 1]
- [Important context that would be lost]
- [Why it matters]

### [Topic 2]
- [More critical context]

---

## Testing/Verification Needed

1. **[Test Name]** - [What to verify]
2. **[Test Name]** - [What to verify]

---

## Quick Commands

```bash
# [Useful command 1]
[command]

# [Useful command 2]
[command]
```

---

## Files Modified This Session

| File | Change |
|------|--------|
| `/path/to/file` | [Brief description] |

---

## Pending Work

| Task | Priority | Notes |
|------|----------|-------|
| [Task] | HIGH/MEDIUM/LOW | [Context] |

---

## Blockers (if any)

| Blocker | Resolution Path | Owner |
|---------|-----------------|-------|
| [Blocker] | [How to unblock] | [Who] |

---

*Handoff written by [Agent] - YYYY-MM-DD*
```

### 4. Update Handoff Registry

After writing handoff, update the registry:

**Location:** `.claude/memory/system/HANDOFF_REGISTRY.json`

**Update the `most_recent` field:**
```json
{
  "most_recent": "HANDOFF-[YOUR-NEW-FILE].md",
  "handoffs": [
    {
      "path": ".claude/memory/handoffs/HANDOFF-[YOUR-NEW-FILE].md",
      "date": "YYYY-MM-DD",
      "time": "HH:MM",
      "duration_hours": X,
      "focus": "[Brief focus description]",
      "status": "COMPLETE/PARTIAL/BLOCKED",
      "key_deliverables": [
        "[Deliverable 1]",
        "[Deliverable 2]"
      ]
    }
  ]
}
```

### 5. Verify Handoff Quality

Before marking session complete:
- [ ] All file paths are absolute
- [ ] Status accurately reflects completion
- [ ] Critical context is preserved (not just "what" but "why")
- [ ] Next steps are actionable
- [ ] Registry updated with new entry
- [ ] `most_recent` field points to new handoff

## Anti-Patterns

### 1. Missing File Paths
```
BAD:  "Updated the config file"
GOOD: "Updated `/absolute/path/to/.mcp.json` - added new server"
```
**Impact:** Next iteration cannot find modified files

### 2. Vague Status
```
BAD:  "Made progress on the feature"
GOOD: "Implemented 6/8 MCP tools (75%), remaining: skill_execute, skill_compose"
```
**Impact:** Next iteration doesn't know what's done

### 3. Missing "Why" Context
```
BAD:  "Decided not to use library X"
GOOD: "Decided not to use library X because it conflicts with MCP server pattern"
```
**Impact:** Next iteration may re-investigate same dead end

### 4. No Verification Steps
```
BAD:  "Feature is ready"
GOOD: "Feature ready - verify with: `python3 test_feature.py`"
```
**Impact:** Next iteration doesn't know how to verify

### 5. Relative Paths
```
BAD:  "Check ./memories/handoffs/..."
GOOD: "Check /absolute/path/to/memories/handoffs/..."
```
**Impact:** Paths break when working directory changes

## Success Indicators

- **Wake-up time <15 minutes** - Next iteration orients quickly
- **No re-investigation** - Critical context preserved
- **Clear next steps** - No ambiguity about priorities
- **Verifiable claims** - Test commands provided for achievements
- **Registry current** - `most_recent` always points to latest handoff
- **Absolute paths only** - All file references are portable

## Related

- `${CIV_ROOT}/.claude/CLAUDE.md` Article III - Session start principles
- `.claude/memory/system/HANDOFF_REGISTRY.json` - Registry file
- `${CIV_ROOT}/.claude/skills/custom/agent-delegation-patterns.md` - Delegation patterns
- `.claude/memory/handoffs/` - Example handoff documents
