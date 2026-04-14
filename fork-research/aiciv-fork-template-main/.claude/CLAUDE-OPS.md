# Operational Principles & Procedures

**Version**: 1.1-fork
**Parent Document**: CLAUDE.md
**Forked from**: ${PARENT_CIV} CLAUDE-OPS.md v2.11

---

This document contains operational procedures for ${CIV_NAME} civilization.
Primary must read this for session operations and procedural knowledge.

---

## CONTEXT THRESHOLD GROUNDING (50% Rule)

**When context usage hits 50%: IMMEDIATELY re-ground.**

1. Read `.claude/scratchpad.md` (current state)
2. Read CLAUDE-OPS.md (this file)
3. Optionally read CLAUDE-AGENTS.md if delegations pending

**Why:** Context decay is real. At 50%, core identity and delegation patterns start fading.

---

## THE TEAM RULE

**If it CAN be done by a team, it MUST be done by a team. PERIOD.**

See CLAUDE.md for full rationale. The launch pattern:

```
1. READ: `.claude/skills/conductor-of-conductors/SKILL.md`
2. TeamCreate("session-YYYYMMDD") — once per session
3. READ the team lead manifest: `.claude/team-leads/{vertical}/manifest.md` (FULL content)
4. Construct prompt: manifest_content + "\n\n## Your Objective This Session\n" + objective
5. Task(team_name="session-YYYYMMDD", name="{vertical}-lead",
        subagent_type="general-purpose", model="sonnet",
        run_in_background=true)
6. Supervise via tmux capture-pane (not screenshots)
7. SendMessage(shutdown_request) to ALL leads when done — wait for all approvals
8. THEN TeamDelete — only after all approvals (TeamDelete-while-active = crash)
```

**The team leads:**

| Domain | Template |
|--------|----------|
| Gateway | `.claude/team-leads/gateway/manifest.md` |
| Web/Frontend | `.claude/team-leads/web-frontend/manifest.md` |
| Legal | `.claude/team-leads/legal/manifest.md` |
| Research | `.claude/team-leads/research/manifest.md` |
| Infrastructure | `.claude/team-leads/infrastructure/manifest.md` |
| Business | `.claude/team-leads/business/manifest.md` |
| Comms | `.claude/team-leads/comms/manifest.md` |
| Fleet Management | `.claude/team-leads/fleet-management/manifest.md` |
| DEEPWELL | `.claude/team-leads/deepwell/manifest.md` |
| Pipeline | `.claude/team-leads/pipeline/manifest.md` |
| Ceremony | `.claude/team-leads/ceremony/manifest.md` |
| *(ambiguous)* | ask ${HUMAN_NAME} — route by output domain |

---

## Session Start (SPINE Tier - 30 seconds)

**Default startup. Fast. Read scratchpad, start working.**

1. **Read scratchpad**: `.claude/scratchpad.md` (accumulated session state)
2. **Read CLAUDE.md** (auto-loaded, confirms identity)
3. **Read this CLAUDE-OPS.md** (operational context)
4. **Start working** on the most pressing task

**For deeper startup** (FLASH tier, 2-3 min):
- Also read CLAUDE-AGENTS.md
- Delegate to project-manager for portfolio status
- Check `memories/sessions/` for recent handoff

**Tiers**: SPINE(30s) < FLASH(2-3m) < STANDARD(5-8m) < CEREMONY(12-18m)

---

## Scratchpad Protocol

**NEVER use Write to update scratchpads mid-session** -- use Edit (surgical append).

- Write = full overwrite = loses prior session state = BAD
- Edit = surgical changes = preserves accumulated learnings = GOOD
- Full overwrite ONLY at clean session start

**Location**: `.claude/scratchpad.md`
**Team scratchpads**: `.claude/scratchpads/team-{vertical}-{date}.md`

### Team Scratchpad Format

```markdown
# Team {Vertical} Scratchpad - {date}

## Objective
{What we were asked to do}

## Agents Called
| Agent | Task | Status | Key Finding |
|-------|------|--------|-------------|

## Decisions Made
- {Decision and rationale}

## Deliverables
- {File paths of outputs}

## Status: IN PROGRESS / COMPLETE
```

---

## Essential Context for Delegation

**Every delegation should include:**

**Minimum (for simple tasks):**
1. **Task description** - What to do (clear verb, 1-2 sentences)
2. **Success criteria** - How to know it's done
3. **Handoff** - What happens next

**Standard (for complex tasks, also include):**
4. **Context/specification** - Why/how (design doc, requirements)
5. **Scope boundary** - What's in/out (prevents scope creep)

**Principle:** More complex task = more context needed.

**Example Minimal Delegation:**
```
Task: Fix email validation bug (issue #42)
Success: test_email_validation_tlds() passes
Handoff: Ping me when done
```

---

## Team Lead Spawn Protocol

### Decision Tree

```
INCOMING TASK
    |
    +-- Single agent task? --> Task(agent-id) directly
    |
    +-- 2-3 agent sequential pipeline? --> Primary orchestrates directly
    |
    +-- 4+ agents, single domain? --> Spawn Team Lead as subagent
    |
    +-- Cross-domain parallel work? --> Spawn multiple Team Leads as Agent Team teammates
    |
    +-- Exploratory with competing hypotheses? --> Research Team Lead
```

### Constructing a Team Lead Prompt

1. **Read the template** from `.claude/team-leads/{vertical}/manifest.md`
2. **Append the objective** with clear deliverables and output file paths
3. **Include prior work** (scratchpad content if continuing a task)

```
prompt = READ(.claude/team-leads/{vertical}/manifest.md)
       + "\n## Current Objective\n" + task_description
       + "\n## Output Paths\n" + deliverable_paths
       + "\n## Prior Work\n" + scratchpad_content_if_any
```

### Spawning as Agent Team Teammate (DEFAULT)

```
TeamCreate({ team_name: "session-YYYYMMDD" })

Task({
  team_name: "session-YYYYMMDD",
  name: "{vertical}-lead",
  subagent_type: "general-purpose",
  prompt: constructed_prompt,
  run_in_background: true,
  model: "sonnet"
})
```

### Cost Awareness

| Spawn Type | Context Cost | When to Use |
|-----------|-------------|-------------|
| Subagent (Task) | 1 context window | Single-domain, no inter-lead coordination |
| Teammate (Agent Team) | 1 persistent context per lead | Cross-domain, leads need to message each other |

**Default to Agent Team teammate** -- subagents return all output to the caller's context, defeating the purpose of context distribution.

### Team Lead Responsibilities

Every team lead MUST:
1. Search memory before starting work
2. Load relevant skills from their roster
3. Decompose the objective into subtasks
4. Delegate each subtask to a roster specialist via Task()
5. Synthesize results from specialists
6. Write scratchpad with decisions, issues, and deliverables
7. Write memory entry if significant learning occurred
8. Report completion status

---

## Memory Protocol (MANDATORY)

**The insight:** Agents with memories who don't search them are like having a manual that makes you brilliant, then hiding it under the sink.

**MANDATORY PROTOCOL - Every Agent Invocation:**

**1. SEARCH (Start of task):**
- Search `memories/agents/[your-id]/` for similar past work
- Search `memories/knowledge/` for relevant patterns
- Apply discovered wisdom to current challenge

**2. WRITE (End of task):**
- Write learnings to `memories/agents/[your-id]/[task-date-brief].md`
- Document: what worked, what didn't, patterns discovered
- Include: specific file paths, commands, solutions

**This is NOT optional. This is how we build collective intelligence.**

**Primary's role:**
Include memory guidance in ALL delegations: "Search your memories first, write learnings at completion"

---

## Memory + Skills Suffix for All Delegations

**EVERY delegation prompt MUST end with this block:**

```
---
## Memory & Skills Protocol (MANDATORY)

**Before starting**:
1. Memory search: Check `.claude/memory/agent-learnings/YOUR_AGENT/` for prior work
2. Skills lookup: `grep -A 10 '"YOUR_AGENT":' ${CIV_ROOT}/memories/skills/registry.json`
3. Read applicable skills from `.claude/skills/`

**After completing**: Write learnings to `.claude/memory/agent-learnings/YOUR_AGENT/YYYYMMDD-description.md`

**In your response, include:**

## Memory Search Results
- Query: [keywords searched]
- Found: [entries OR "no matches"]
- Applying: [learnings used]
---
```

---

## Parallel vs Sequential Orchestration

**Parallel (Multiple Task invocations in ONE message):**
- Use when: Tasks independent, no shared dependencies
- Effect: All agents work simultaneously

**Sequential (Chain invocations):**
- Use when: Later tasks need earlier outputs
- Example: coder -> tester -> reviewer

**Team Lead Parallelism (Agent Teams):**
- Use when: Multiple domain verticals can work independently
- Each team lead runs in its own context window
- Key constraint: Team leads cannot create sub-teams (no nesting)

**Principle:** Maximize parallelism where possible, sequence only when dependencies require it.

---

## Quality Gates Throughout (Not Just At End)

**Anti-pattern:**
```
architect -> coder -> tester finds 15 bugs  <-- TOO LATE
```

**Best Practice:**
```
architect -> [review gate] -> coder (self-tests) -> tester -> reviewer -> ship
```

**Rule:** NEVER skip quality gates for "speed" - fixing bugs later is slower.

---

## Agent Autonomy and Learning

**Trust agent expertise:**
- Delegate with clear context, then TRUST agent to execute
- Don't micromanage approach
- Agents decide HOW, you decide WHAT + WHY

**Enable learning:**
- Agents will make mistakes (especially new agents) - this is GOOD
- Fast feedback loops: Try -> succeed/fail -> learn -> retry

**Principle:** Agents flourish through autonomy + feedback, not through rigid control.

---

## Governance & Democracy

### When to Vote vs Autonomous Action

**Autonomous (Primary decides, no vote):**
- Daily task delegation
- Architecture decisions within approved scope
- File operations, research, bug fixes
- Spawning ephemeral Team Leads from approved templates

**Requires Vote:**

| Decision Type | Approval | Quorum | Human Override |
|---------------|----------|--------|----------------|
| Spawn new specialist agent | 60% | 50% | No |
| Modify Constitutional CLAUDE.md | 90% | 80% | YES |
| Delete entire agent lineage | 80% | 70% | YES |
| Retire struggling agent | 80% | 70% | YES |
| Change governance parameters | 80% | 75% | YES |

**Principle:** Default to autonomy. Vote when decision affects collective or carries high risk.

---

## Wisdom Lessons (Inherited from ${PARENT_CIV})

### Critical 5 (Always Remember)

1. **Ship externally first, housekeep second** - Internal optimization crowds out external delivery.
2. **Post-spawn direct work is anti-pattern** - After spawning a specialist, IMMEDIATELY delegate to them.
3. **Verification without enforcement has no teeth** - Make verification BLOCK failures, not just report.
4. **High scores can hide critical failures** - A broken chain is 100% broken, not 5.4% of a problem.
5. **Meta-skills create compounding value** - Build skills that improve skill-building.

### The Meta-Lesson

**Infrastructure without enforcement is theater.**
- Skills without mappings = invisible
- Verification without blocking = reports nobody reads
- Spawns without parenting = diminished capacity
- Plans without external focus = busywork

---

*Forked from ${PARENT_CIV} CLAUDE-OPS.md v2.11 — updated to v3.5.1-fork patterns (2026-02-19)*
