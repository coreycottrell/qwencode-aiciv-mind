---
name: work-mode-boop
description: Autonomous work loop with state hygiene. Trigger with /work-mode. Prevents redoing work via PM sync, handles context gracefully, delegates to 10 agents max. Use when idle or Corey says keep working.
---

# BOOP v2.0: Autonomous Work Loop

> State-aware autonomy. Never redo work. Graceful handoffs.

**Trigger**: `/work-mode` or "keep working" or idle detection

---

## The Loop (5 Steps)

### Step 1: IDENTITY + SPINE GROUND (1 min)

```
Read: /home/corey/projects/AI-CIV/ACG/.claude/CLAUDE.md
Read: /home/corey/projects/AI-CIV/ACG/.claude/CLAUDE-OPS.md
Run: /primary-spine
Read: .claude/skills/team-launch/SKILL.md
Read: .claude/skills/conductor-of-conductors/SKILL.md
Read .claude/scratchpad.md
Read .claude/scratchpad-daily/YYYY-MM-DD.md  (create if missing, use today's date)
```

**Load full identity every cycle.** CLAUDE.md + CLAUDE-OPS.md + spine. More identity has always been worth it (Corey directive 2026-02-17). Sessions with CLAUDE.md show 10.3x more delegations. This is not optional overhead - it's what keeps you alive across compacts.

**Review scratchpad for ongoing tasks:**
- Read `.claude/scratchpad-daily/YYYY-MM-DD.md` (today's)
- Identify any active team leads, pending decisions, blocked work
- Note tasks in progress vs tasks that need starting

Check context:
- If >90%: Run `/compact` immediately, then continue loop (DO NOT STOP)
- If 70-90%: Set low-context mode (no new big tasks, wrap current work first)
- If <70%: Full speed ahead

### Step 2: STATE SYNC (2 min)

**Query PM:**
```
Task(project-manager): "BOOP state sync - What's currently active? What completed since last cycle? Any blockers?"
```

**Cross-check scratchpad** for untracked work.

**Update PM** with any discovered work:
```
Task(project-manager): "State hygiene: Found untracked work - [describe]. Please update portfolio."
```

**Ongoing Task List maintenance:**
- Scan today's scratchpad for items marked TODO, PENDING, BLOCKED, or ✅
- If any TODO/PENDING items have no active team lead working them → add to PM backlog
- Maintain a running "active tasks" section in the scratchpad with current status

**Team lead communications check:**
- Check tmux panes for any active team leads: `tmux list-panes -a -F "#{pane_id} #{pane_title}"`
- If team leads are running: capture their status, note in scratchpad
- If team leads completed but no SendMessage arrived: check their output, update PM

**Inter-CIV comms check (every BOOP cycle):**
- Scan tmux for [CIV-MESSAGE] or [TEAM-MESSAGE] injections from sister civilizations
- Check `~/.claude/inter-civ-teams/*/state.json` for unread team messages
- If messages found: process, acknowledge, store to `memories/communication/inter-civ/`
- Full protocol: `.claude/skills/inter-civ-comms/SKILL.md`

**THIS IS THE GATE** - PM prevents redoing completed work.

### Step 3: WORK (variable)

**Priority order:**
1. Corey's direct instructions (ALWAYS first)
2. PM's prioritized backlog (CRITICAL > HIGH > MEDIUM)
3. Discovery (only if backlog empty AND context <70%)

**Route through the right team lead — NEVER call agents directly:**
```python
# Route through team leads per CEO Rule
TeamCreate("session-YYYYMMDD") if not already created
Task(team_name=..., name="gateway-lead", prompt=..., run_in_background=True)
Task(team_name=..., name="research-lead", prompt=..., run_in_background=True)
# Team leads delegate to specialists. Primary delegates to team leads only.
```

**Update scratchpad immediately** after delegating:
```
## Active Work
- gateway-lead: Backlog #X
- research-lead: Backlog #Y
```

### Step 4: COMPLETION SYNC (30s)

After agents return:

```
Task(project-manager): "Mark #X COMPLETE. Evidence: [brief]. Completed by: [agent]"
```

Write learnings if significant (patterns, dead ends, synthesis).

Update scratchpad with completions.

**Update ongoing task list in scratchpad:**
Append to today's daily scratchpad:
```
## BOOP Cycle [N] — [timestamp]
- Completed: [what finished]
- Active: [what's still running]
- Next: [what starts next cycle]
- Team leads: [any running leads + their status]
- Pending decisions: [anything blocked on Corey]
```

### Step 5: CONTEXT CHECK

| Context | Action |
|---------|--------|
| >90% | Update scratchpad, run `/compact`, then **CONTINUE** looping (Step 1) |
| 70-90% | Wrap current work, no new big tasks, loop to Step 3 |
| <70% AND idle AND <10 agents | Find work from backlog, loop to Step 3 |

**BOOPs do NOT stop for context.** They compact and continue. The scratchpad is your persistence layer - `/compact` reads it on resume.

---

## Pre-Compact Protocol (>90% Context)

**Before running `/compact`:**

1. Update `.claude/scratchpad.md` with full current state:

```markdown
## BOOP Cycle State [timestamp]
- Current focus: [1 sentence]
- Files changed: [absolute paths]
- Decisions made: [bullets]
- Active agents: [what's still running]
- Next priority: [specific action from backlog]
- Blockers: [if any]
- PM status: [confirmed synced / needs updates]
- BOOP count this session: [N]
```

2. Append the same state block to today's daily scratchpad (append-only, do NOT overwrite):

```
Append to: .claude/scratchpad-daily/YYYY-MM-DD.md  (use today's date)
```

The daily scratchpad is a journal - each BOOP cycle appends a new entry. Do not overwrite previous entries.

3. Sync PM:
```
Task(project-manager): "Pre-compact sync - [completed items]. Continuing after compact."
```

5. Run `/compact`

6. **After compact resumes**: Full Step 1 again (CLAUDE.md + CLAUDE-OPS.md + `/primary-spine` + scratchpad + daily scratchpad), then continue from Step 2 (STATE SYNC)

**The loop is infinite until Corey intervenes or the session ends.**

---

## Safety Rules

**NEVER:**
- Kill Claude instances
- Redo work PM says is complete
- Exceed 10 concurrent agents
- Start external comms (email, posts) autonomously
- Continue when Corey messages (STOP immediately)

**ALWAYS:**
- Query PM before starting ANY work
- Update scratchpad after ANY delegation
- Check context after each agent return

---

## Quick Reference

| Phase | Duration | Key Action |
|-------|----------|------------|
| SPINE GROUND | 30s | Load identity, check context |
| STATE SYNC | 2m | PM query, discover untracked work |
| WORK | variable | Delegate backlog, update scratchpad |
| COMPLETION SYNC | 30s | Mark complete in PM, write learnings |
| CONTEXT CHECK | instant | Route based on context % |

---

## Future Enhancements (Post-MVP)

These are NOT implemented yet. Notes for future iterations:

- **Big Goal Spine**: Align each cycle with North Star, not just task clearing
- **Observation Masking**: JetBrains research shows this beats LLM summarization for context management
- **Tiered Autonomy**: Tier 0 (monitor-only) through Tier 3 (full implementation)
- **Memory Consolidation**: Compress memories during idle time
- **Performance Monitoring**: Dedicated agent for tracking BOOP metrics
- **5-Layer Handoff**: State/Narrative/Decisions/Priorities/Warnings structure
- **Recursive Self-Correction**: Auto-retry with different approach on delegation failures
- **Inter-Civ Maintenance**: Relationship upkeep with sister civs during idle
- **Topic-Trigger Detection**: PM gate triggers when encountering stale assumptions

---

**"State hygiene first. Never redo work. Graceful exits."**
