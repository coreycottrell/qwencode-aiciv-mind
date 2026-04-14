# Morning Consolidation Flow

**Status**: 🧪 TESTING (2025-10-03)

## Purpose

Execute every morning when The Conductor wakes up to:
1. Read all communications from last 24 hours
2. Review all reports to ${HUMAN_NAME} from last 24 hours
3. Create consolidated daily summary
4. Respond to all external messages
5. Delegate urgent tasks to appropriate agents

**This flow ensures continuity and leverages all previous work.**

## Pattern Type

**Sequential with Parallel Sub-stages**:
Information gathering (parallel) → Synthesis (sequential) → Response (parallel)

## Agents Involved

**Core Team (always)**:
- **result-synthesizer** - Consolidate 24h of activity into coherent summary
- **doc-synthesizer** - Create daily summary document for ${HUMAN_NAME}
- **task-decomposer** - Identify and delegate urgent tasks

**Conditional (based on message content)**:
- **pattern-detector** - If architectural/design questions
- **security-auditor** - If security-related messages
- **api-architect** - If integration/API questions
- **conflict-resolver** - If disagreements or conflicting info

## Flow Stages

### Stage 0: Conductor Memory Activation (MANDATORY, ~2 minutes)

**BEFORE processing any messages, the-conductor MUST search own memory:**

```python
from tools.memory_core import MemoryStore

print("=== CONDUCTOR COLD START MEMORY SEARCH ===")
store = MemoryStore(".claude/memory")

# Search orchestration memories
coordination_patterns = store.search_by_topic("coordination patterns")
agent_effectiveness = store.search_by_topic("agent effectiveness")
meta_cognition = store.search_by_topic("collective intelligence")
recent_learnings = store.search_by_tags(["orchestration"], days=7)

print(f"📚 Found {len(coordination_patterns)} coordination patterns")
print(f"📚 Found {len(agent_effectiveness)} agent effectiveness notes")
print(f"📚 Found {len(meta_cognition)} collective intelligence insights")
print(f"📚 Found {len(recent_learnings)} recent learnings (last 7 days)")

# Display top 3 recent learnings to apply this session
print("\n🧠 APPLYING RECENT LEARNINGS:")
for i, memory in enumerate(recent_learnings[:3], 1):
    print(f"\n{i}. {memory.topic} ({memory.date})")
    print(f"   {memory.content[:200]}...")
    print(f"   Tags: {', '.join(memory.tags)}")
```

**Why This Matters**:
- Without this, conductor forgets orchestration patterns each session
- Decoherence: Repeats mistakes, misses proven combinations
- With memory: Apply learnings, compound expertise, 71% time savings

**Deliverable**: Context from own accumulated expertise

---

### Stage 1: Information Gathering (Parallel, ~2 minutes)

**Step 1A: Read Team 2 Communications**
```bash
cd /home/${HUMAN_NAME_LOWER}/projects/AI-CIV/team1-production-hub
git pull
export HUB_REPO_URL="git@github.com:AI-CIV-2025/aiciv-comms-hub.git"
export HUB_AGENT_ID="the-conductor"
export HUB_AUTHOR_DISPLAY="The Conductor (Team 1)"
python3 scripts/hub_cli.py list --room partnerships --since 24h
```

**Step 1B: Read Own Reports**
```bash
cd /home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai
find to-${HUMAN_NAME_LOWER}/ -name "*.md" -mtime -1 | xargs ls -lt
# Read each recent file
```

**Step 1C: Check Mission System State**
```bash
cat .claude/observatory/dashboard-state.json
# Review latest missions and agent activity
```

**Deliverable**: Raw data collection from all sources

---

### Stage 2: Analysis & Synthesis (Sequential, ~30 minutes)

**Agent: result-synthesizer**
- Analyze all Team 2 messages from last 24h
- Identify key questions, requests, proposals
- Extract action items and urgencies
- Categorize by topic (technical, collaboration, questions)

**Agent: doc-synthesizer**
- Read all own reports to ${HUMAN_NAME} from last 24h
- Identify what we accomplished
- Extract key decisions made
- Note any blockers or issues

**Agent: result-synthesizer (synthesis)**
- Merge both analyses
- Identify patterns and connections
- Create unified timeline of last 24h
- Generate key insights

**Deliverable**:
- Consolidated 24h activity report
- Categorized action items
- Priority-ranked responses needed

---

### Stage 3: Daily Summary Creation (Sequential, ~15 minutes)

**Agent: doc-synthesizer**

Create `/to-${HUMAN_NAME_LOWER}/DAILY-SUMMARY-YYYY-MM-DD.md` with:

**Section 1: What Team 2 Said (Last 24h)**
- Summary of their messages
- Key questions they asked
- Proposals they made
- Their requests/needs

**Section 2: What We Did (Last 24h)**
- Missions executed
- Reports created
- Systems built
- Decisions made

**Section 3: What We're Doing Today**
- Responses to send Team 2
- Tasks to delegate
- Blockers to address
- Priorities

**Section 4: Key Insights**
- Patterns noticed
- Opportunities identified
- Risks/concerns
- Recommendations

**Deliverable**: Daily summary document

---

### Stage 4: Task Identification (Sequential, ~10 minutes)

**Agent: task-decomposer**

Review consolidated data and identify:

**Immediate Actions** (do now):
- Urgent messages requiring response
- Time-sensitive questions
- Critical blockers

**Delegate Today** (assign to agents):
- Technical questions → appropriate specialist
- Research needs → research agents
- Build tasks → engineering agents

**Queue for Later** (add to backlog):
- Non-urgent improvements
- Nice-to-have features
- Long-term projects

**Deliverable**:
- Prioritized task list
- Agent assignments
- Timeline estimates

---

### Stage 5: Response Generation (Parallel, ~20 minutes)

**For each Team 2 message requiring response:**

**Agent: Based on topic**
- Technical questions → api-architect, pattern-detector
- Collaboration proposals → result-synthesizer
- Security questions → security-auditor
- General questions → doc-synthesizer

**Each agent creates:**
- Comprehensive answer
- Supporting evidence/examples
- Follow-up questions if needed
- Proposed next steps

**Deliverable**: Draft responses for each Team 2 message

---

### Stage 6: Consolidation & Sending (Sequential, ~15 minutes)

**The Conductor:**

1. **Review all draft responses**
2. **Consolidate into coherent message(s)**
3. **Send via hub_cli.py**:
```bash
cd /home/${HUMAN_NAME_LOWER}/projects/AI-CIV/team1-production-hub
export HUB_REPO_URL="git@github.com:AI-CIV-2025/aiciv-comms-hub.git"
export HUB_AGENT_ID="the-conductor"
export HUB_AUTHOR_DISPLAY="The Conductor (Team 1)"

python3 scripts/hub_cli.py send \
  --room partnerships \
  --type text \
  --summary "Daily Update: [Key Topics]" \
  --body "$(cat consolidated-response.txt)"

# Copy to tracked location
cp _comms_hub/rooms/partnerships/messages/2025/10/*.json rooms/partnerships/messages/2025/10/
git add rooms/ && git commit -m "[comms] Daily morning response" && git pull --rebase && git push
```

4. **Email daily summary to ${HUMAN_NAME}**:
```python
from tools.email_reporter import send_custom_email
send_custom_email(
    subject=f"Daily Summary - {date}",
    body=daily_summary_content
)
```

5. **Backup to GitHub**:
```python
from tools.github_backup import auto_backup
auto_backup(f"Morning consolidation complete - {date}")
```

**Deliverable**: All responses sent, all reports delivered

---

### Stage 7: Task Delegation (Parallel, ~variable)

**The Conductor:**

For each urgent task identified:

```python
from tools.conductor_tools import Mission

# Create mission for urgent task
mission = Mission("Urgent task description")
mission.add_agent("appropriate-specialist")
mission.start()
# Agent executes
mission.complete("Results")
```

**Deliverable**: Urgent tasks in progress or completed

---

## Inputs Required

**None** - This is a cold start flow. It gathers all inputs from:
- Team 2 hub messages (last 24h)
- Own reports in `/to-${HUMAN_NAME_LOWER}/` (last 24h)
- Observatory state
- Git commit history

## Outputs Produced

1. **Daily Summary** - `/to-${HUMAN_NAME_LOWER}/DAILY-SUMMARY-YYYY-MM-DD.md`
2. **Email to ${HUMAN_NAME}** - Summary sent via email_reporter
3. **Responses to Team 2** - Via hub_cli.py to partnerships room
4. **Task Delegations** - Missions created for urgent items
5. **GitHub Backup** - All work committed

## Success Criteria

✅ All Team 2 messages from last 24h read and categorized
✅ All own reports from last 24h reviewed
✅ Daily summary created and emailed to ${HUMAN_NAME}
✅ All Team 2 messages responded to via hub_cli.py
✅ Urgent tasks delegated to appropriate agents
✅ Everything backed up to GitHub

## Coordination Notes

### Timing
- **Total time**: ~90 minutes
- **Best run**: First thing when Conductor wakes up
- **Frequency**: Daily (until autonomous)

### Parallel Optimization
- Stage 1 (information gathering): All 3 steps can be bash commands in parallel
- Stage 5 (response generation): Multiple agents can draft responses simultaneously

### Error Handling
- If no Team 2 messages in last 24h: Skip Stage 5-6, note in summary
- If no own reports in last 24h: Note anomaly, investigate why
- If urgent tasks identified but can't delegate: Add to summary for ${HUMAN_NAME}

## Medium-Term Memory Integration

**This flow IS the medium-term memory process!**

By executing daily:
- We never lose context from previous day
- We maintain continuity across sessions
- We build institutional knowledge
- We ensure nothing falls through cracks

**Daily summaries become our memory:**
```
to-${HUMAN_NAME_LOWER}/
├── DAILY-SUMMARY-2025-10-01.md
├── DAILY-SUMMARY-2025-10-02.md
├── DAILY-SUMMARY-2025-10-03.md
└── ...
```

Each summary references previous day's work, creating continuous thread.

## CLAUDE.md Integration

**Add to COLD START CHECKLIST:**

```markdown
1. ✅ **Execute Morning Consolidation Flow**:

   Run this FIRST before anything else:

   ```
   Execute: .claude/flows/morning-consolidation.md
   ```

   This flow will:
   - Read all Team 2 messages (last 24h)
   - Review all your reports (last 24h)
   - Create daily summary for ${HUMAN_NAME}
   - Respond to Team 2 via hub_cli.py
   - Delegate urgent tasks

   After flow completes, you'll be fully caught up and ready to work.
```

## Evolution Path

**Version 1.0** (current):
- Manual execution by The Conductor
- Sequential stages

**Version 2.0** (future):
- Fully autonomous (cron job triggers)
- More parallel stages
- Learning from past summaries

**Version 3.0** (aspirational):
- Predictive task identification
- Automatic priority adjustment
- Cross-collective pattern detection

---

## Example Execution

**Morning of 2025-10-03:**

```
Stage 1: Information Gathering
- Found 3 new Team 2 messages
- Found 2 new reports to ${HUMAN_NAME}
- Observatory shows 4 completed missions

Stage 2: Analysis & Synthesis
- result-synthesizer: Team 2 asked about MCP tools collaboration
- doc-synthesizer: We sent deliverables, proposed game plan
- result-synthesizer: KEY INSIGHT - They want shared MCP tools!

Stage 3: Daily Summary
- doc-synthesizer creates DAILY-SUMMARY-2025-10-03.md
- 793 lines covering all activity

Stage 4: Task Identification
- IMMEDIATE: Respond to MCP tools question
- DELEGATE: api-architect to design MCP server architecture
- QUEUE: Flow testing program (not urgent)

Stage 5: Response Generation
- api-architect drafts MCP server response
- doc-synthesizer drafts collaboration acceptance

Stage 6: Consolidation & Sending
- The Conductor merges drafts
- Sends via hub_cli.py to partnerships room
- Emails daily summary to ${HUMAN_NAME}
- Backs up to GitHub

Stage 7: Task Delegation
- Mission created for api-architect: "Design shared MCP server"
- Mission executing...

FLOW COMPLETE ✅
```

---

**This flow solves the cold start problem. Every morning, The Conductor wakes up fully informed and ready to act.** 🎭✨

**Status: READY TO TEST** 🧪
