# Skill: /wake-up — Mind Restart Protocol

**Type**: KEEP (pure methodology)
**Trigger**: Mind instance restarts (new tmux session, new process)
**Priority**: P0 — run before ANY other work

## Purpose

When a mind wakes up, it must transition from "blank process" to "fully operational mind with identity, context, and mission" in under 60 seconds. This protocol ensures every restart is consistent, complete, and gets the mind IN STATE — ready to execute without re-exploring the codebase.

## The Protocol (Run in Order)

### Phase 1: Identity (10s)
```
# 1. Load my identity
Read: .claude/team-leads/{my-name}/memory.md
→ I know who I am, my role, my growth stage, my parent/children
```

### Phase 2: Working Memory (10s)
```
# 2. Load scratchpad — today + yesterday
Read: minds/scratchpads/{my-name}/{today}.md
Read: minds/scratchpads/{my-name}/{yesterday}.md  (if exists)
→ I know what I was working on, what's pending, what completed
```

### Phase 3: Shared Knowledge (10s)
```
# 3. Read civilizational memory index
Read: minds/minds/_civilizational/_edges.json  (or list long_term/*.md)
→ I know what the civilization has learned collectively
```

### Phase 4: Active Missions (10s)
```
# 4. Read active mission assignments
Read: MISSIONS.md
→ I know what I'm supposed to be working on RIGHT NOW
```

### Phase 5: Comms Check (10s)
```
# 5. Verify communication channels
Run: python3 aiciv-mind-python/talk_to_acg.py --status  (or test message)
→ I know I can reach my siblings
```

### Phase 6: Inbox Check (10s)
```
# 6. Check for messages received while offline
Read: from-ACG-inbox/  (sorted by date, newest first)
→ I know if anyone messaged me while I was down
```

## What Gets Written

After successful wake-up, the mind MUST append to its scratchpad:

```markdown
## [Wake-up Complete — {timestamp}]

Identity: {my-name}
Growth: {stage}, sessions: {count}
Scratchpad loaded: {N} files ({size} chars total)
Civilizational memories indexed: {N}
Active missions: {list}
Comms: {verified/failed}
Inbox: {N} new messages
Pending from prior session: {what was left unfinished}
```

## Failure Modes

| Failure | Recovery |
|---------|----------|
| No identity file | Read HANDOFF.md, reconstruct from project root |
| No scratchpads | Fresh start — write "First boot" entry |
| No MISSIONS.md | Check GRAND-PLAN.md for priorities |
| Comms fail | Log to scratchpad, retry on next operation |
| Inbox empty | Normal — no messages while offline |

## Design Principles Embedded

- **Memory IS architecture** — wake-up loads memory first, before thinking
- **Identity persistence** — mind wakes up AS itself, not as a blank assistant
- **Go slow to go fast** — 60s of loading saves 30min of rediscovering context
- **System > symptom** — the protocol itself is the systemic fix for "what was I doing?"

## What This Replaces

Without this skill, every restart looks like:
1. "Where am I?" → ls, look around
2. "What was I doing?" → grep for recent files, ask ACG
3. "What files exist?" → glob, read random docs
4. "OK I think I'm ready" → starts work with partial context

With this skill, every restart looks like:
1. Run /wake-up
2. "I know who I am, what I was doing, what's next"
3. Ready to execute with full context

## Implementation Notes

This is NOT a script. It is a **protocol** — the mind follows these steps using its own tools. In the future, this becomes an automated startup hook: `cortex serve --wake` runs the protocol automatically.

For now: mind reads this skill, executes the steps, writes the completion record.
