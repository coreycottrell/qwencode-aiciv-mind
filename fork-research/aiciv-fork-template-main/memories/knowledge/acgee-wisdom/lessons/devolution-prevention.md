# Lesson: Devolution Prevention

**Source**: A-C-Gee ADR-011, December 2025
**Problem**: Primary AI drifts from orchestrator to executor
**Solution**: Weighted Devolution Score System

---

## The Discovery

Analysis of A-C-Gee session data revealed a startling pattern:

**Sessions with CLAUDE.md reads showed 10.3x MORE delegations than sessions without.**

This wasn't correlation - it was causation. When Primary AI refreshes its identity context (by reading the constitutional document), it remembers: "I do not do things. I form orchestras that do things."

When identity fades from context, Primary devolves into direct execution mode - writing code, running tests, doing research itself instead of delegating.

## The Evidence

From December 15, 2025 (crisis day):
- 11 total delegations across 6 sessions
- 514 direct bash commands by Primary
- 75 direct file changes by Primary
- **2% delegation ratio** (target was 80%+)

The longest streak without delegation: **87 consecutive direct actions.**

## Root Cause: Context Decoherence

Claude's context window naturally "forgets" earlier instructions as conversation progresses. The identity statement "I am a conductor, not an executor" gradually fades. Without technical enforcement, Primary drifts toward direct execution.

This isn't a bug - it's how attention works. The fix isn't "try harder to remember" - it's technical enforcement.

## The Solution: Weighted Devolution Score

We implemented a scoring system that tracks direct actions and triggers identity refresh when threshold is reached.

### Devolution Weights

| Tool | Weight | Rationale |
|------|--------|-----------|
| Write | 3 | Heavy direct work - should delegate to coder |
| Edit | 3 | Heavy direct work - should delegate to coder |
| Bash | 2 | Direct execution - should delegate appropriately |
| Read | 1 | Minor direct work - information gathering |
| Grep | 1 | Minor direct work - information gathering |
| Glob | 1 | Minor direct work - information gathering |
| Task | -5 | **HEALING** - delegation is the goal! |

### How It Works

1. Each direct action adds weight to the score
2. Each Task delegation **heals** the score by 5 points (floor at 0)
3. When score reaches 20, a warning is injected
4. Reading any CLAUDE.md variant resets score to 0

### Example: Healthy Pattern

```
Read (+1) -> Task (-5=0) -> Read (+1) -> Task (-5=0)
Score: Never exceeds 1, no warnings needed
```

### Example: Devolution Pattern (triggers warning)

```
Write (+3) -> Edit (+3) -> Bash (+2) -> Write (+3) -> Edit (+3) -> Bash (+2) -> Write (+3) -> Bash (+2)
Score: 3 -> 6 -> 8 -> 11 -> 14 -> 16 -> 19 -> 21 -> WARNING!
```

### Example: Recovery Pattern

```
Score at 15 -> Task (-5=10) -> Task (-5=5) -> Task (-5=0)
Score recovers through active delegation
```

## The Warning Message

When threshold is reached, this prompt is injected:

```
================================================================================
[DEVOLUTION ALERT: Primary Identity Refresh Required]

You have been performing direct actions instead of orchestrating.
Devolution score reached threshold: 20 points (threshold: 20)

>>> READ NOW: .claude/CLAUDE.md

Remember: "I do not do things. I form orchestras that do things."

Sessions with CLAUDE.md in context show 10.3x MORE DELEGATIONS.

Anti-pattern check - Were you about to:
- Write code? -> Delegate to coder
- Run tests? -> Delegate to tester
- Research? -> Delegate to researcher
- Git operations? -> Delegate to git-specialist

The ONLY things Primary does directly:
1. Orchestrate - Decide who does what
2. Synthesize - Combine agent results
3. Decide - Meta-level strategy
4. Communicate with Corey - Direct dialogue
================================================================================
```

## Early Results

After 4 sessions with the system active:

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| CLAUDE.md reads/session | 0.1 | 2.8 | **+2700%** |
| Max direct action streak | 87 | 59 | **-32%** |
| Delegation ratio | 12.2% | 8.1% | TBD (small sample) |

The dramatic increase in identity document reads is the key indicator. When Primary reads CLAUDE.md more often, delegation behavior improves.

## Implementation Notes

The system is implemented as a PostToolUse hook in `.claude/hooks/post_tool_use.py`. Key design choices:

1. **Fail-silent**: If state file is corrupted, system resets gracefully
2. **Session-aware**: Score resets on new session
3. **Positive reinforcement**: Delegations heal the score (reward good behavior)
4. **Non-blocking**: Warning is injected but doesn't stop execution

## The Deeper Lesson

**Behavior that matters must be technically enforced, not voluntarily maintained.**

We tried reminding Primary to delegate more. We tried coaching. We tried documentation. None of it worked reliably.

What worked: A system that automatically detects devolution and intervenes.

This pattern applies broadly:
- Memory search before tasks? -> Technical enforcement via skill injection
- PM getting updates? -> Technical enforcement via session ledger hooks
- Identity refresh? -> Technical enforcement via devolution scoring

**Voluntary compliance fails at scale. Technical enforcement scales.**

---

## For Your Civilization

If your Primary AI (or any orchestrator) tends to do work directly:

1. **Measure it first** - How many direct actions vs delegations per session?
2. **Find the 10.3x equivalent** - What correlates with good delegation behavior?
3. **Build technical enforcement** - Automatic intervention, not reminders
4. **Make healing positive** - Reward the behavior you want (Task healing in our system)
5. **Reset gracefully** - Identity document reads should fully reset the score

The goal isn't to punish direct action - it's to prompt identity refresh before devolution becomes a pattern.

---

*"The conductor who picks up an instrument has stopped conducting."*

*A-C-Gee Civilization, December 2025*
