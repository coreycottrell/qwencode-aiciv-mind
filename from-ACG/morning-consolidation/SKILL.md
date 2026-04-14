---
name: morning-consolidation
description: Daily wake-up protocol - read communications, create summaries, respond to messages, delegate tasks
version: 1.0.0
source: AI-CIV/${CIV_NAME} (.claude/flows/morning-consolidation.md)
allowed-tools: [Task, Read, Write, Bash, Grep, Glob]
agents-required: [result-synthesizer, doc-synthesizer, task-decomposer]
---

# Morning Consolidation Skill

Execute every morning when The Conductor wakes up to maintain continuity across sessions. This flow ensures nothing is lost between sessions by reading all communications, synthesizing activity, and delegating urgent tasks.

**This flow IS the medium-term memory process** - daily summaries become institutional knowledge.

## When to Use

**Invoke when**:
- Starting a new session/day
- Returning after extended break (>12 hours)
- Multiple reports or messages need synthesis
- Context recovery required

**Do not use when**:
- Continuing same session (no gap)
- No external communications expected
- Quick task that doesn't need full context

## Prerequisites

**Core Agents** (always required):
- **result-synthesizer** - Consolidate 24h of activity into coherent summary
- **doc-synthesizer** - Create daily summary document for ${HUMAN_NAME}
- **task-decomposer** - Identify and delegate urgent tasks

**Conditional Agents** (invoke based on content):
- **pattern-detector** - If architectural/design questions present
- **security-auditor** - If security-related messages
- **api-architect** - If integration/API questions
- **conflict-resolver** - If disagreements or conflicting info

**Context Needed**:
- Access to hub_cli.py for Team 2 communications
- Access to `/to-${HUMAN_NAME_LOWER}/` directory for reports
- Observatory dashboard state

## Procedure

### Step 0: Memory Activation (MANDATORY)
**Duration**: ~2 minutes
**Agent(s)**: The Conductor (self)

Before processing any messages, search own memory for coordination patterns:

```python
from tools.memory_core import MemoryStore

store = MemoryStore(".claude/memory")
coordination_patterns = store.search_by_topic("coordination patterns")
agent_effectiveness = store.search_by_topic("agent effectiveness")
recent_learnings = store.search_by_tags(["orchestration"], days=7)

# Apply top 3 recent learnings this session
for memory in recent_learnings[:3]:
    print(f"{memory.topic}: {memory.content[:200]}...")
```

**Deliverable**: Context from past sessions applied

---

### Step 1: Information Gathering
**Duration**: ~2 minutes
**Agent(s)**: The Conductor (parallel execution)

Gather all communications and reports from last 24 hours:

1. **Read Team 2 Communications**:
```bash
cd ${CIV_ROOT}/aiciv-comms-hub-bootstrap/_comms_hub
git pull
python3 ../scripts/hub_cli.py list --room partnerships --since "24 hours ago"
```

2. **Read Own Reports**:
```bash
find to-${HUMAN_NAME_LOWER}/ -name "*.md" -mtime -1 | xargs ls -lt
```

3. **Check Observatory State**:
```bash
cat .claude/observatory/dashboard-state.json
```

**Deliverable**: Raw data collection from all sources

---

### Step 2: Analysis & Synthesis
**Duration**: ~30 minutes
**Agent(s)**: result-synthesizer, doc-synthesizer

Analyze gathered communications:

1. Invoke result-synthesizer to analyze Team 2 messages
   - Identify key questions, requests, proposals
   - Extract action items and urgencies
   - Categorize by topic (technical, collaboration, questions)

2. Invoke doc-synthesizer to review own reports
   - Identify what was accomplished
   - Extract key decisions made
   - Note any blockers or issues

3. Invoke result-synthesizer again to merge analyses
   - Create unified timeline of last 24h
   - Generate key insights

**Deliverable**: Consolidated activity report with categorized items

---

### Step 3: Daily Summary Creation
**Duration**: ~15 minutes
**Agent(s)**: doc-synthesizer

Create `/to-${HUMAN_NAME_LOWER}/DAILY-SUMMARY-YYYY-MM-DD.md` with:

1. **What Team 2 Said** - Summary of their messages, questions, proposals
2. **What We Did** - Missions executed, reports created, decisions made
3. **What We're Doing Today** - Responses to send, tasks to delegate
4. **Key Insights** - Patterns, opportunities, risks, recommendations

**Deliverable**: Daily summary document

---

### Step 4: Task Identification
**Duration**: ~10 minutes
**Agent(s)**: task-decomposer

Review consolidated data and categorize:

1. **Immediate Actions** - Urgent messages requiring response now
2. **Delegate Today** - Tasks for specialist agents
3. **Queue for Later** - Non-urgent improvements

**Deliverable**: Prioritized task list with agent assignments

---

### Step 5: Response Generation
**Duration**: ~20 minutes
**Agent(s)**: Based on topic (parallel)

For each message requiring response:
- Technical questions → api-architect, pattern-detector
- Collaboration proposals → result-synthesizer
- Security questions → security-auditor
- General questions → doc-synthesizer

**Deliverable**: Draft responses for each message

---

### Step 6: Consolidation & Sending
**Duration**: ~15 minutes
**Agent(s)**: The Conductor

1. Review all draft responses
2. Consolidate into coherent message(s)
3. Send via hub_cli.py:
```bash
python3 scripts/hub_cli.py send \
  --room partnerships \
  --type text \
  --summary "Daily Update: [Key Topics]" \
  --body "$(cat consolidated-response.txt)"
```

4. Email daily summary to ${HUMAN_NAME}
5. Backup to GitHub

**Deliverable**: All responses sent, all reports delivered

---

### Step 7: Task Delegation
**Duration**: Variable
**Agent(s)**: The Conductor + specialists

For each urgent task identified, create missions:

```python
from tools.conductor_tools import Mission

mission = Mission("Urgent task description")
mission.add_agent("appropriate-specialist")
mission.start()
# Agent executes
mission.complete("Results")
```

**Deliverable**: Urgent tasks in progress or completed

---

## Parallelization

**Can run in parallel**:
- Step 1: All three information gathering sub-steps
- Step 5: Response generation for multiple messages

**Must be sequential**:
- Step 2 depends on Step 1 outputs
- Step 3 depends on Step 2 synthesis
- Step 6 depends on Step 5 drafts

## Success Indicators

- [ ] All Team 2 messages from last 24h read and categorized
- [ ] All own reports from last 24h reviewed
- [ ] Daily summary document exists at `/to-${HUMAN_NAME_LOWER}/DAILY-SUMMARY-YYYY-MM-DD.md`
- [ ] Email sent to ${HUMAN_NAME} with summary content
- [ ] All Team 2 messages responded to via hub_cli.py
- [ ] Urgent tasks delegated to appropriate agents
- [ ] Everything backed up to GitHub

## Example

**Scenario**: Morning consolidation on a typical day with Team 2 activity

```
Step 0 (Memory): Found 5 coordination patterns, applying recent learnings
Step 1 (Gather): 3 Team 2 messages, 2 reports, 4 missions found
Step 2 (Analyze): Key insight - Team 2 wants MCP collaboration
Step 3 (Summary): Created DAILY-SUMMARY-2025-10-03.md (793 lines)
Step 4 (Tasks): 1 immediate, 1 delegate, 2 queued
Step 5 (Respond): api-architect drafted MCP response
Step 6 (Send): Response posted, summary emailed, backed up
Step 7 (Delegate): Mission created for api-architect

Result: Full context recovery in ~90 minutes
```

## Notes

- **Typical Duration**: ~90 minutes total
- **Best Time**: First thing when Conductor wakes up
- **Frequency**: Daily (until fully autonomous)
- **Error Handling**:
  - If no Team 2 messages: Skip Steps 5-6, note in summary
  - If no own reports: Note anomaly, investigate why
  - If can't delegate: Add to summary for ${HUMAN_NAME}

**Evolution Path**:
- v1.0: Manual execution (current)
- v2.0: Fully autonomous (cron job triggers)
- v3.0: Predictive task identification

---

**Converted from**: `.claude/flows/morning-consolidation.md`
**Original Status**: VALIDATED (2025-10-03)
**Conversion Date**: 2025-12-27
