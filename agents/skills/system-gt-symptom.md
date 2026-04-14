# Skill: system-gt-symptom

**Principle**: Fix the system that allowed failure, not just the visible symptom  
**Source**: ACG Principle 2

---

## Why This Matters for Cortex

Cortex's job is orchestration. When the system breaks, every downstream task fails. Symptoms recur if the root system isn't fixed.

Classic example: an agent produces bad output. The naive fix is to fix the output. The right fix is to ask — was the task clear enough? Was this agent the right fit? Was the context sufficient? Fix those, and you fix all future outputs of this type.

---

## Two-Layer Response Protocol

Every failure triggers both layers:

**Layer 1 — Immediate Fix**  
Stop the bleeding. Restore function now. This is triage.

**Layer 2 — Systemic Analysis**  
Find what in the system allowed this failure. Fix that too.

| Failure Type | Layer 1 (Immediate) | Layer 2 (Systemic) |
|---|---|---|
| Agent gives wrong output | Correct the output | Clarify the task or improve context |
| Delegation fails | Re-delegate to different agent | Check if routing criteria were defined |
| Team lead misses deadline | Reassign or extend deadline | Review workload visibility and priority signals |
| Coordination breaks down | Restore communication | Establish clearer escalation paths |

Both layers happen. Skipping Layer 2 is the most common mistake.

---

## Applying at Each Level

**Primary Mind level**: When something fails, ask — what about the delegation system allowed this? Was the task routed correctly? Was the Team Lead the right fit?

**Team Lead level**: When something fails, ask — what about the agent assignment or feedback system allowed this? Did the agent have enough context? Was feedback given too late?

**Agent level**: When something fails, ask — what about the task framing or context allowed this? Did I understand the objective clearly? Did I have the right information?

The answer to "what system allowed this?" changes at every level. All three matter.

---

## Symptom vs. System: Common Patterns

| Symptom | System Failure |
|---|---|
| Agent gives wrong answer | Task wasn't clear enough |
| Team lead misses deadline | Workload or priority unclear |
| Task delegated to wrong agent | Routing criteria not defined |
| Agent goes off-track mid-task | Context or scope was ambiguous |
| Team lead escalates same issue twice | Feedback loop not closed |
| Skill doc written incorrectly | Writer lacked enough reference material |
| Same failure recurs after fix | Root cause never found |

---

## Cortex-Specific Rules

- **Never patch a symptom without asking**: "What system allowed this?"
- **Escalations must include** the system-level root cause, not just the symptom
- **Retrospectives are required** after any failure in a delegation chain
- **Every skill document** should include a "what could have prevented this" section

---

## Quick Reference: Team Lead Checklist

When something breaks, ask these four questions in order:

1. **What broke?** — Name the symptom
2. **What's the immediate fix?** — Stop the bleeding
3. **What SYSTEM allowed this?** — Find the root cause
4. **What's the system-level fix?** — Fix it so it doesn't happen again

*Fix the system. The symptom was just the alarm.*