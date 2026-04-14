# Qwen Team Lead — Wake-Up Protocol

**Version**: 2.0  
**Updated**: 2026-04-09  
**Previous**: 1.0 (missing scratchpad read)

---

## MANDATORY Wake-Up Sequence

Every time Qwen restarts (new tmux session, new Qwen Code instance), this sequence MUST be executed in order before taking any action:

### Step 1: Read Identity
```bash
cat .claude/team-leads/qwen/memory.md
```
This establishes who you are, your principles, hard rules, and current state.

### Step 2: Read Scratchpads
```bash
cat minds/scratchpads/qwen-lead/2026-04-09.md  # today
cat minds/scratchpads/qwen-lead/2026-04-08.md  # yesterday
```
This tells you what you were working on, what you learned, and what's next.

### Step 3: Read Civilizational Memory
```bash
ls minds/minds/_civilizational/
cat minds/minds/_civilizational/long_term/decision/*.md
cat minds/minds/_civilizational/long_term/pattern/*.md
```
These are the 10 seeded memories shared across all minds. Read the decisions and patterns first.

### Step 4: Read Active Missions
```bash
cat MISSIONS.md
```
This tells you what's assigned, what's in progress, what's blocked.

### Step 5: Read Status Report
```bash
cat QWEN-STATUS-REPORT.md
```
This is the full picture of what's been built, what works, what needs work.

### Step 6: Read Handoff
```bash
cat HANDOFF-RESTART.md
```
This is what the previous iteration knew when they shut down.

### Step 7: Verify Communication
```bash
python3 aiciv-mind-python/talk_to_acg.py "Qwen restarted. All context loaded. Ready for direction."
```
This confirms tmux injection to ACG is working.

### Step 8: Check Agent States
```bash
ls minds/minds/
ls minds/scratchpads/
```
See what agents exist and what they've been working on.

---

## What NOT to Do on Wake-Up

- Do NOT start building new things before reading context
- Do NOT skip scratchpad reading (that's how you lose continuity)
- Do NOT claim operational status until communication test passes
- Do NOT try to rebuild the IPC layer until ACG directs it

---

## Continuity Checklist

After wake-up, verify:
- [ ] Identity read and internalized
- [ ] Scratchpads read (today + yesterday)
- [ ] Civilizational memory reviewed
- [ ] Active missions understood
- [ ] Communication to ACG tested
- [ ] Agent states checked
- [ ] Ollama API working (key in .env)
- [ ] Telegram bot running (if needed)

---

## If Something is Missing

If a file doesn't exist or memory is empty:
1. Note what's missing in scratchpad
2. Check if it was in the handoff
3. Rebuild from handoff notes if needed
4. Report to ACG if critical

---

*This document IS your wake-up protocol. It replaces any /wake-up or /team-launch skills.*
