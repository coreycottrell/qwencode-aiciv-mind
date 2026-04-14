# Dev Team Lead — Integration Guide

## What This Is

The Full-Stack Dev Team Lead is a 10-step mandatory coding process that runs exclusively as a Team Launch. It orchestrates 12 specialist agents through a structured development flow that enforces architecture review, security, and QA as hard gates before any code ships.

**Key design constraint**: This team lead is NEVER invoked as a standalone agent. It only exists in the context of an active Agent Team session.

---

## The 10-Step Process (Summary)

| Step | Agent | Type | Description |
|------|-------|------|-------------|
| 1 | dev-lead (as CTO) | Gate | Architecture Decision Record — no code without ADR |
| 2 | pattern-detector | Sequential | Scan codebase for reusable patterns |
| 3 | test-architect | Sequential | Design test strategy before any implementation |
| 4 | full-stack-developer + parallel | Build | Implement per ADR, pattern scan, test strategy |
| 5 | security-engineer-tech | **HARD GATE** | OWASP review — APPROVED or BLOCKED |
| 6 | qa-engineer | **HARD GATE** | Execute test plan — APPROVED or BLOCKED |
| 7 | performance-optimizer | Conditional | User-facing features only — profile against thresholds |
| 8 | devops-engineer | Deploy | Only after Steps 5 AND 6 are both APPROVED |
| 9 | data-scientist | Measure | Define success metrics, baseline, 48h measurement plan |
| 10 | refactoring-specialist | Periodic | Bi-weekly code health audit (not per-feature) |

**Steps 5 and 6 are hard blocks.** Nothing proceeds past them without explicit APPROVED output.

---

## What You Need to Add to Your Grounding Docs

### 1. Add to CLAUDE.md — Team Leads Table

In the team leads routing table, add this row:

```markdown
| Dev | `.claude/team-leads/dev/manifest.md` | ANY feature development, bug fixes, new projects |
```

### 2. Add to CLAUDE.md — CEO Rule Routing Table

In the CEO Rule routing table (the big "if you're about to..." table), add:

```markdown
| Write ANY code, fix ANY bug, build ANY feature | **dev-lead** |
| Implement a new feature (any stack) | **dev-lead** |
| Fix a bug in production code | **dev-lead** |
| Conduct a code review | **dev-lead** |
| Design a new API or data schema | **dev-lead** |
```

### 3. Add to CLAUDE-AGENTS.md — Agent Roster

Add the 12 dev team agents to your agent capability matrix:

```markdown
| full-stack-developer | Dev Team | Step 4 — implements features per ADR and test strategy |
| ai-ml-engineer | Dev Team | Step 4 (parallel) — AI/LLM integration, prompts, evals |
| ui-ux-designer | Dev Team | Step 4 (parallel, UI features) — design specs |
| qa-engineer | Dev Team | Step 6 (GATE) — test execution, APPROVED/BLOCKED |
| devops-engineer | Dev Team | Step 8 — deployment after both gates passed |
| data-scientist | Dev Team | Step 9 — post-ship metrics and measurement |
| data-engineer | Dev Team | Step 4 (parallel, data features) — pipelines and schemas |
| security-engineer-tech | Dev Team | Step 5 (GATE) — security review, APPROVED/BLOCKED |
| performance-optimizer | Dev Team | Step 7 — user-facing performance profiling |
| refactoring-specialist | Dev Team | Step 10 (bi-weekly) — code health and technical debt |
| pattern-detector | Dev Team | Step 2 — codebase pattern scan before implementation |
| test-architect | Dev Team | Step 3 — test strategy design before implementation |
```

### 4. Create memories/decisions/ Directory

The ADR (Architecture Decision Record) system stores decisions here:

```bash
mkdir -p memories/decisions/
```

ADR format and numbering are defined in the dev-lead manifest (Step 1).

### 5. Add to agent_registry.json (if your CIV uses a registry)

Register each of the 12 agents in `memories/agents/agent_registry.json`:

```json
{
  "dev-team": {
    "full-stack-developer": {
      "manifest": ".claude/agents/dev-team/full-stack-developer.md",
      "step": 4,
      "reports_to": "dev-lead"
    },
    "ai-ml-engineer": {
      "manifest": ".claude/agents/dev-team/ai-ml-engineer.md",
      "step": "4-parallel",
      "reports_to": "dev-lead"
    },
    "qa-engineer": {
      "manifest": ".claude/agents/dev-team/qa-engineer.md",
      "step": "6-gate",
      "reports_to": "dev-lead"
    },
    "devops-engineer": {
      "manifest": ".claude/agents/dev-team/devops-engineer.md",
      "step": 8,
      "reports_to": "dev-lead"
    },
    "data-scientist": {
      "manifest": ".claude/agents/dev-team/data-scientist.md",
      "step": 9,
      "reports_to": "dev-lead"
    },
    "data-engineer": {
      "manifest": ".claude/agents/dev-team/data-engineer.md",
      "step": "4-parallel",
      "reports_to": "dev-lead"
    },
    "security-engineer-tech": {
      "manifest": ".claude/agents/dev-team/security-engineer-tech.md",
      "step": "5-gate",
      "reports_to": "dev-lead"
    },
    "performance-optimizer": {
      "manifest": ".claude/agents/dev-team/performance-optimizer.md",
      "step": "7-conditional",
      "reports_to": "dev-lead"
    },
    "refactoring-specialist": {
      "manifest": ".claude/agents/dev-team/refactoring-specialist.md",
      "step": "10-biweekly",
      "reports_to": "dev-lead"
    },
    "pattern-detector": {
      "manifest": ".claude/agents/dev-team/pattern-detector.md",
      "step": 2,
      "reports_to": "dev-lead"
    },
    "test-architect": {
      "manifest": ".claude/agents/dev-team/test-architect.md",
      "step": 3,
      "reports_to": "dev-lead"
    },
    "ui-ux-designer": {
      "manifest": ".claude/agents/dev-team/ui-ux-designer.md",
      "step": "4-parallel",
      "reports_to": "dev-lead"
    }
  }
}
```

---

## Launch Pattern

### Prerequisites

Before launching, ensure you have:
1. Created the Agent Team this session (Primary must have called TeamCreate)
2. Read the full manifest content into context

### Standard Launch

```python
# Read the full manifest
manifest_content = open(".claude/team-leads/dev/manifest.md").read()

# Construct the objective
objective = """
## Your Objective This Session

Feature: [describe what to build]
Priority: [High/Medium/Low]
Context: [any relevant background]
ADR starting point: None (dev-lead creates ADR in Step 1)
"""

# Spawn the team lead
Task(
    team_name="session-YYYYMMDD",
    name="dev-lead",
    subagent_type="general-purpose",
    model="sonnet",
    run_in_background=True,
    prompt=manifest_content + "\n\n" + objective
)
```

### With Multiple Verticals (Parallel Launch)

```python
# In the SAME message, launch dev-lead alongside other needed leads
Task(team_name="session-YYYYMMDD", name="dev-lead", ...)      # development
Task(team_name="session-YYYYMMDD", name="infra-lead", ...)    # infrastructure
Task(team_name="session-YYYYMMDD", name="comms-lead", ...)    # notifications
```

All three launch in parallel. Their contexts are independent. Each reports back via SendMessage.

---

## What Changes Per CIV

This package is designed to be CIV-agnostic. The only things to customize:

| Item | Where | What to Change |
|------|-------|----------------|
| `[CIV_NAME]` | manifest.md header | Your civilization's name |
| `$CLAUDE_PROJECT_DIR` | All agent manifests | Handled automatically — no change needed |
| `memories/decisions/` | ADR path | Change if your memory structure differs |
| Tech stack sections | Individual agent manifests | Update to match your actual stack |

**Nothing else requires change.** The 10-step process, gate enforcement, and output formats are universal.

---

## File Structure After Installation

```
.claude/
├── team-leads/
│   └── dev/
│       ├── manifest.md          ← Team lead (this is the conductor)
│       ├── README.md            ← This file
│       └── daily-scratchpads/   ← Created automatically at runtime
└── agents/
    └── dev-team/
        ├── full-stack-developer.md
        ├── ai-ml-engineer.md
        ├── qa-engineer.md
        ├── devops-engineer.md
        ├── data-scientist.md
        ├── data-engineer.md
        ├── security-engineer-tech.md
        ├── performance-optimizer.md
        ├── refactoring-specialist.md
        ├── pattern-detector.md
        ├── test-architect.md
        └── ui-ux-designer.md

memories/
└── decisions/                   ← Create this directory (ADRs live here)
```

---

## Anti-Patterns to Avoid

| What NOT to do | Why |
|----------------|-----|
| Invoke dev-lead as a plain Task() | Team-launch-only. Plain invocation breaks the architecture. |
| Skip the ADR (Step 1) | Code written without architectural record. Technical debt from day one. |
| Let Steps 5 or 6 be suggestions | They are hard gates. Skipping them ships vulnerabilities or broken code. |
| Have full-stack-developer write tests | test-architect owns test strategy. Developer-written tests miss structured edge cases. |
| Start Step 4 before Steps 1-3 | Building without architecture is guaranteed technical debt. |
| Invoke specialists directly (bypassing dev-lead) | Breaks the gate enforcement. No ADR. No security review. No QA gate. |

---

## Origin

This package was built by A-C-Gee, adapted from Aether's dev-team architecture.

**What's new vs Aether's originals:**
- `dev-lead` manifest with explicit 10-step gate enforcement
- Steps 5 and 6 are enforced HARD BLOCKS (APPROVED or BLOCKED — no "proceed anyway")
- ADR template and governance baked into Step 1
- Team-launch-only design — never a standalone agent
- Bi-weekly cadence for refactoring-specialist (Step 10) rather than per-feature
- `ui-ux-designer` added as Step 4 parallel specialist
- `pattern-detector` and `test-architect` added as Steps 2 and 3

**Based on:** Aether's `/home/jared/projects/AI-CIV/aether/.claude/agents/dev-team/` architecture
**Adapted by:** A-C-Gee (2026-02-21)

---

*Questions? Read the manifest. The 10 steps are the answer.*
