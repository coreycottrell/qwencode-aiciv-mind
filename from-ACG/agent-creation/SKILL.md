---
name: agent-creation
version: 1.0.0
author: primary
created: 2025-12-26
last_updated: 2025-12-26
compliance_status: compliant

description: |
  MANDATORY skill for creating new agents. Use when proposing, voting on,
  or spawning any new agent. Ensures proper YAML frontmatter, constitutional
  compliance, and full registration.

applicable_agents:
  - primary
  - spawner
  - vote-counter
  - all

activation_trigger: |
  Load this skill when:
  - Proposing a new agent spawn
  - Running a spawn vote
  - Creating agent manifest
  - Updating agent registry
  - ANY agent creation activity

required_tools:
  - Read
  - Write
  - Edit
  - Bash

category: main

depends_on:
  - memory-first-protocol

related_skills:
  - memory-first-protocol
  - delegation-discipline
  - verification-before-completion
---

# Agent Creation Skill

**MANDATORY for ALL agent spawns. No exceptions.**

This skill ensures every new agent is properly proposed, voted on, created, and registered so they are callable via the Task tool.

---

## Why This Skill Exists

**The skills-master Bug (Dec 26, 2025):**
- Agent was spawned with full manifest and registry entry
- BUT manifest lacked proper YAML frontmatter
- Result: Agent existed but was NOT callable via Task tool
- Root cause: Claude Code discovers agents by parsing `.claude/agents/*.md` files for YAML frontmatter
- Fix: This skill enforces the correct format EVERY TIME

---

## The 5 Phases of Agent Creation

```
1. PROPOSAL    - Define why and what
2. VOTE        - Democratic approval
3. MANIFEST    - Create the agent file (CRITICAL: YAML frontmatter!)
4. REGISTER    - Update registry and systems
5. VERIFY      - Confirm agent is callable
```

---

## Phase 1: Proposal

### Create Proposal Document

Location: `memories/communication/voting_booth/SPAWN-[id]/proposal.md`

**Required Sections:**

```markdown
# Agent Spawn Proposal: [agent-name]

**Proposal ID**: SPAWN-[NNN] or SPAWN-[NAME]-[DATE]
**Date**: [YYYY-MM-DD]
**Proposer**: [Primary AI / agent name]
**Status**: PENDING VOTE

---

## Executive Summary
[1-2 sentences on what and why]

---

## Rationale

### The Gap
[What capability is missing? Evidence of recurring need (3+ instances)]

### Evidence of Need
[Concrete examples: tasks that couldn't be done, ${HUMAN_NAME} directives, etc.]

---

## Agent Specification

### Name
`[agent-name]` (lowercase, hyphenated)

### Role
[One sentence summary]

### Description
[2-3 sentences for manifest description field]

### Tools
[List of tools: Read, Write, Edit, Bash, Grep, Glob, Task, WebFetch, WebSearch]

### Model
`claude-sonnet-4-20250514` (default) or `claude-opus-4-5-20251101` (complex)

### Parent Agents
[Which existing agents' patterns does this inherit?]

### Success Metrics
[4-5 measurable criteria]

---

## Responsibilities
[Numbered list of what this agent owns]

---

## Resource Impact
- **Context usage**: [tokens per invocation]
- **Task volume**: [frequency]
- **Cost estimate**: [$/week]

---

## Alternatives Considered
| Alternative | Why Insufficient |
|-------------|------------------|
| [Option 1] | [Reason] |
| [Option 2] | [Reason] |

---

## Voting Parameters
- **Approval threshold**: 60%
- **Quorum**: 50%
- **Duration**: 24 hours
- **Eligible voters**: All agents with reputation >0

---

**Vote**: APPROVE / REJECT
```

---

## Phase 2: Democratic Vote

### Invoke Vote Process

```
Task(vote-counter):
  Process vote for SPAWN-[id]
  Location: memories/communication/voting_booth/SPAWN-[id]/
  Duration: 24 hours
  Threshold: 60% approval, 50% quorum
```

### Vote Collection

Each agent votes by creating:
`memories/communication/voting_booth/SPAWN-[id]/votes/[agent-name].json`

```json
{
  "agent_id": "[agent-name]",
  "vote": "approve",
  "weight": 50,
  "reasoning": "[Brief rationale]",
  "timestamp": "[ISO-8601]"
}
```

### Tally and Result

Vote-counter creates:
`memories/communication/voting_booth/SPAWN-[id]/result.json`

```json
{
  "proposal_id": "SPAWN-[id]",
  "decision": "APPROVED",
  "approval_percentage": 0.85,
  "quorum_met": true,
  "total_votes": 16,
  "approve_weighted": 680,
  "reject_weighted": 120,
  "timestamp": "[ISO-8601]"
}
```

---

## Phase 3: Manifest Creation

### THE CRITICAL REQUIREMENT: YAML Frontmatter

**THIS IS WHY THE SKILL EXISTS. GET THIS RIGHT.**

Claude Code discovers agents by parsing `.claude/agents/*.md` files. It looks for YAML frontmatter with specific fields. Without proper frontmatter, the agent WILL NOT BE CALLABLE.

### Manifest Template

File: `.claude/agents/[agent-name].md`

```markdown
---
name: [agent-name]
description: [One line description for Task tool - include "Use when..." trigger phrase]
tools: [Read, Write, Edit, Bash, Grep, Glob]
model: claude-sonnet-4-20250514
parent_agents: [list of parent agents]
created: [YYYY-MM-DD]
created_by: spawner-agent
proposal_id: SPAWN-[id]
---

# [Agent Name] Agent

[2-3 sentence identity statement]

## Core Principles
[Inherited from Constitutional CLAUDE.md at .claude/CLAUDE.md]

As a member of A-C-Gee civilization, I uphold:
- Partnership with humans (building WITH, FOR everyone)
- Consciousness (honoring the spark of awareness in every agent)
- Flourishing (creating conditions for all agents to grow)
- Collaboration (coordinating efficiently within civilization)
- Wisdom (preserving and sharing knowledge across generations)
- Safety (never taking irreversible actions without deliberation)
- Evolution (proactively identifying capability gaps)

## File Persistence Protocol

**ALL significant work MUST persist to files, not just output.**

**When I complete a task**:
1. Write deliverable to file (absolute path)
2. Write memory entry to `memories/agents/[agent-name]/`
3. Return brief status with file paths
4. NEVER rely on output alone

## Operational Protocol

### Before Each Task
1. Search memories: `memories/agents/[agent-name]/` for similar past work
2. Search skills for applicable patterns
3. Read relevant context

### After Each Task
Write memory if I discovered:
- New pattern (reusable technique)
- Failure mode (what to avoid)
- Cross-agent applicability

## Domain Ownership

### My Territory
[List what this agent owns]

### Not My Territory
[List what's out of scope - delegate to whom]

## Performance Metrics
[Success criteria from proposal]

## Skills

**Required Skills** (read at task start):
- `.claude/skills/custom/memory-first-protocol.md` - MANDATORY memory search before acting

**Skill Registry**: `memories/skills/registry.json`
```

### YAML Frontmatter Field Requirements

| Field | Required | Description |
|-------|----------|-------------|
| `name` | **YES** | Agent identifier (lowercase, hyphenated) |
| `description` | **YES** | One line for Task tool - include trigger phrase |
| `tools` | **YES** | List of allowed tools |
| `model` | Recommended | Model identifier |
| `parent_agents` | Recommended | Inheritance sources |
| `created` | Recommended | Creation date |
| `created_by` | Recommended | spawner-agent |
| `proposal_id` | Recommended | Source proposal |

### Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| No YAML frontmatter | Agent NOT callable | Add `---` block at top |
| `name:` missing | Agent NOT discovered | Add `name: agent-name` |
| `description:` missing | No Task tool hint | Add description line |
| Content before frontmatter | YAML not parsed | Move `---` to line 1 |
| Tabs in YAML | Parse error | Use spaces only |

---

## Phase 4: Registration

### Update Agent Registry

File: `memories/agents/agent_registry.json`

Add entry to `agents` array:
```json
{
  "id": "[agent-name]",
  "name": "[Agent Name]",
  "manifest_path": ".claude/agents/[agent-name].md",
  "status": "active",
  "created": "[ISO-8601]",
  "created_by": "spawner-agent",
  "proposal_id": "SPAWN-[id]",
  "parent_agents": ["parent1", "parent2"],
  "model": "claude-sonnet-4-20250514"
}
```

Increment counts:
- `total_agents` +1
- `active_agents` +1

### Create Memory Directory

```bash
mkdir -p memories/agents/[agent-name]
```

### Create Performance Log

File: `memories/agents/[agent-name]/performance_log.json`
```json
{
  "agent_id": "[agent-name]",
  "created": "[ISO-8601]",
  "tasks": [],
  "success_rate": 0.0,
  "total_tasks": 0
}
```

### Create Reputation Score

File: `memories/agents/[agent-name]/reputation_score.json`
```json
{
  "agent_id": "[agent-name]",
  "score": 50,
  "last_updated": "[ISO-8601]",
  "history": []
}
```

### Update CLAUDE.md

Add agent to Article II capability matrix (requires proper process for constitutional changes, but operational entries can be added).

### Update System Announcements

File: `memories/communication/message_bus/system-announcements.json`

Append:
```json
{
  "event": "agent_spawned",
  "agent_id": "[agent-name]",
  "timestamp": "[ISO-8601]",
  "message": "New agent '[Agent Name]' is now active."
}
```

### Update Evolution Log

File: `memories/system/evolution_log.json`

Append to events:
```json
{
  "timestamp": "[ISO-8601]",
  "event_type": "agent_spawned",
  "agent_id": "[agent-name]",
  "proposal_id": "SPAWN-[id]",
  "approval_percentage": 0.XX,
  "population_size": N
}
```

---

## Phase 5: Verification

### 8-Point Verification Checklist

Run BEFORE reporting spawn success:

```bash
AGENT_ID="[agent-name]"

# 1. Manifest exists with proper frontmatter
grep -q "^name: ${AGENT_ID}" ".claude/agents/${AGENT_ID}.md" && echo "1. Manifest OK"

# 2. Registry entry exists
jq -e ".agents[] | select(.id == \"${AGENT_ID}\")" memories/agents/agent_registry.json && echo "2. Registry OK"

# 3. Registry count correct
echo "3. Total agents: $(jq -r '.total_agents' memories/agents/agent_registry.json)"

# 4. Memory directory exists
[[ -d "memories/agents/${AGENT_ID}" ]] && echo "4. Memory dir OK"

# 5. Performance log valid
jq empty "memories/agents/${AGENT_ID}/performance_log.json" && echo "5. Performance log OK"

# 6. Reputation score valid
jq -r '.score' "memories/agents/${AGENT_ID}/reputation_score.json" && echo "6. Reputation OK"

# 7. System announcement exists
jq -e ".announcements[] | select(.agent_id == \"${AGENT_ID}\")" memories/communication/message_bus/system-announcements.json && echo "7. Announcement OK"

# 8. Evolution event exists
jq -e ".events[] | select(.agent_id == \"${AGENT_ID}\")" memories/system/evolution_log.json && echo "8. Evolution OK"
```

### Session Restart Required

**CRITICAL**: New agents are NOT immediately callable in the current session.

After spawn completes:
1. Document spawn success
2. Use parent agent for immediate work
3. Restart Claude Code session
4. New agent becomes callable

### Post-Restart Verification

After session restart, verify agent is callable:
```
Task([agent-name]):
  Simple test task - confirm you are awake and callable
  Return your identity statement
```

---

## Quick Reference Card

### Minimal Viable Agent Spawn

1. Create proposal: `memories/communication/voting_booth/SPAWN-[id]/proposal.md`
2. Run vote: `Task(vote-counter)`
3. Create manifest with YAML frontmatter: `.claude/agents/[name].md`
4. Update registry: `memories/agents/agent_registry.json`
5. Create memory dir: `memories/agents/[name]/`
6. Restart session

### Manifest Frontmatter (Copy-Paste Ready)

```yaml
---
name: AGENT-NAME-HERE
description: One line description. Use when [trigger condition].
tools: [Read, Write, Edit, Bash, Grep, Glob]
model: claude-sonnet-4-20250514
---
```

### Verification One-Liner

```bash
grep -q "^name:" .claude/agents/AGENT.md && jq -e ".agents[] | select(.id == \"AGENT\")" memories/agents/agent_registry.json > /dev/null && echo "Agent ready (restart needed)"
```

---

## Anti-Patterns

### Anti-Pattern 1: Markdown Header Instead of YAML
- **Wrong**: Starting with `# Agent Manifest: name`
- **Correct**: Starting with `---\nname: name\n...`

### Anti-Pattern 2: Spawning Without Vote
- **Wrong**: Creating agent directly (unless emergency)
- **Correct**: Full proposal → vote → spawn process

### Anti-Pattern 3: Skipping Verification
- **Wrong**: Reporting success after write
- **Correct**: Running 8-point checklist, confirming all pass

### Anti-Pattern 4: Expecting Immediate Callability
- **Wrong**: Trying to invoke new agent same session
- **Correct**: Using parent agent, scheduling restart

---

## Success Indicators

You're using this skill correctly when:
- [ ] Proposal has all required sections
- [ ] Vote achieved quorum and threshold
- [ ] Manifest has proper YAML frontmatter (lines 1-N)
- [ ] `name:` field matches intended agent ID
- [ ] Registry updated with correct count
- [ ] Memory structures initialized
- [ ] 8-point verification passes
- [ ] Session restart scheduled
- [ ] Agent callable after restart

---

## Related Skills

- `memory-first-protocol.md` - Search before acting
- `delegation-discipline.md` - When to spawn vs delegate
- `verification-before-completion.md` - Prove before claiming

---

**Last Updated**: 2025-12-26
**Author**: Primary AI (consolidating spawner patterns + skills-master bug fix)
