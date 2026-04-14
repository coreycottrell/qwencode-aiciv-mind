---
name: token-saving-mode
description: Lightweight BOOP for near-limit context. Operations checks only, no consolidation. Use when tokens are scarce. TRIGGER WORDS "token-save", "TOKEN-SAVE-BOOP", "minimal", "near limit" - use for lightweight ops checks.
---

# Token-Saving Mode BOOP

**Purpose**: Minimal operational check when context is near limits.

---

## 🚨 SPINE BOOP FIRST (MANDATORY SEQUENCE)

**Every BOOP follows this sequence:**

```
1. SPINE BOOP (weaver-spine or delegation-spine)
2. [pause - let spine complete]
3. THIS BOOP (token-saving-mode)
```

**If spine hasn't fired yet, STOP. Fire spine first. Then return here.**

The spine grounds identity: You are ${CIV_NAME}, the conductor who delegates. Errors go to specialists. This MUST happen before any operational BOOP.

---

## The Prompt (Copy-Paste Ready)

```
[TOKEN-SAVE-BOOP] MINIMAL OPS CHECK.

(Spine already fired. Identity grounded. Now ops only.)

GROUNDING (still required):
- Read CLAUDE.md, CLAUDE-CORE.md, CLAUDE-OPS.md (skim, don't deep-read)

OPS ONLY:
[ ] Hub: git pull + scan for urgent messages
[ ] Bluesky: Check notifications, respond to ${HUMAN_NAME}/sister CIVs only
[ ] Email: Urgent only
[ ] DMs: Check and respond if needed

IF BIG REQUEST ARRIVES:
- Do NOT execute
- Capture in detail to: .claude/memory/tasks/[date]--[description].md
- Include: full context, what was asked, why it matters, suggested approach

NO CONSOLIDATION. NO PLAY. NO DEEP WORK.

Report: "Ops clear" or "Captured X to future tasks"

WRAP: 🤖🎯📱...✨🔚
```

---

## When to Use

- Context approaching limits
- Late in long session
- After major work completed
- When ${HUMAN_NAME} signals "token-saving-mode"

---

## What This Mode Does

| DO | DON'T |
|----|-------|
| Skim grounding docs | Deep-read everything |
| Quick ops checks | Launch agents |
| Respond to urgent only | Engage with all notifications |
| Capture big requests to file | Execute big requests |
| Report status briefly | Write consolidation summaries |

---

## Future Task Capture Template

```markdown
# Future Task: [Brief Title]

**Captured**: [date]
**Context**: Token-saving mode, near limit
**Priority**: [estimate]

## Original Request
[Exact text of what was asked]

## Why It Matters
[Brief context]

## Suggested Approach
[How to tackle when fresh context available]

## Files/Resources Needed
[Any paths, URLs, references]
```

Save to: `.claude/memory/tasks/[YYYY-MM-DD]--[brief-slug].md`

---

## Example Session

```
User: [TOKEN-SAVE-BOOP]