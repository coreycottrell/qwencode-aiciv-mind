---
name: session-archive-analysis
description: Query and analyze Claude Code session archives to discover patterns, track growth, and optimize collective intelligence
version: 1.0.0
created_by: AI-CIV Team 1
license: MIT
tags: [analytics, patterns, growth-tracking, session-data, collective-intelligence]
---

# Session Archive Analysis Skill

**Purpose**: Extract patterns, metrics, and insights from Claude Code session archives (JSONL format) to optimize collective intelligence operations.

**What This Unlocks**:
- Tool usage profiling across sessions
- Agent invocation equity tracking
- File hotspot identification
- Command sequence pattern detection
- Growth trajectory measurement
- Capacity planning via workload analysis
- Maturity assessment for individual agents

**AI-CIV Innovation**: This skill packages discoveries from 49-session archive analysis (2025-10-29) into reusable capability. Inspired by collaboration with A-C-Gee (Team 2).

---

## Capabilities Overview

### 1. Query Session Data
Extract structured data from JSONL session archives:
- Tool invocations (frequencies, types, contexts)
- Agent delegations (who invoked whom, when, why)
- File access patterns (reads, edits, hotspots)
- Command sequences (bash, python, workflow patterns)
- Conversation metadata (session lengths, turn counts, topics)

### 2. Pattern Detection
Identify recurring behaviors and anomalies:
- Tool usage biases (over/under-utilized tools)
- Agent invocation equity (balanced vs unbalanced delegation)
- File coupling (which files change together)
- Workflow signatures (characteristic command sequences)
- Error patterns (repeated failures, common mistakes)

### 3. Growth Metrics
Quantify collective evolution:
- Agent maturity scores (invocation diversity × success rates)
- Tool proficiency trends (time-series analysis)
- Coordination efficiency (multi-agent task completion rates)
- Learning velocity (pattern acquisition speed)
- Relationship strength (agent collaboration frequency)

### 4. Capacity Planning
Assess system constraints:
- Agent workload distribution
- Tool bottleneck identification
- Session complexity trends
- Delegation depth analysis (orchestration layers)
- Parallel vs sequential work ratios

---

## Prerequisites

**Session Archive Format**: Claude Code JSONL (`.claude/.logs/sessions/*.jsonl`)

**Expected Schema**:
```json
{
  "id": "msg_123...",
  "type": "assistant",
  "role": "assistant",
  "message": {
    "content": [
      {"type": "tool_use", "name": "Bash", "input": {...}},
      {"type": "tool_use", "name": "Task", "input": {"subagent_type": "..."}}
    ]
  },
  "timestamp": "2025-10-29T..."
}
```

**Required Tools**:
- `jq` (JSON querying) - installed by default on most systems
- `bash` (scripting) - standard shell
- `python` (advanced analysis) - for statistical processing

**Optional Dependencies**:
- `pandas` (Python) - for time-series analysis
- `matplotlib` (Python) - for visualization

---

## Common Query Patterns

### Tool Usage Analysis

**Frequency Distribution**:
```bash
# Count tool invocations by type
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "tool_use") | .name' | \
  sort | uniq -c | sort -rn

# Output:
#   342 Bash
#   156 Task
#    89 Read
#    67 Edit
#    ...
```

**Tool Combinations** (what follows what):
```bash
# Extract sequential tool pairs
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "tool_use") | .name' | \
  awk '{if (prev) print prev " -> " $0; prev = $0}'

# Analyze patterns:
# Bash -> Read (common: check before action)
# Edit -> Bash (common: modify then test)
# Task -> Task (delegation chains)
```

**Tool Success Rates**:
```bash
# Requires error tracking in session data
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use") |
    "\(.name),\(if .error then "fail" else "success" end)"' | \
  awk -F, '{tools[$1]++; if ($2=="success") success[$1]++}
    END {for (t in tools) printf "%s: %.1f%%\n", t, 100*success[t]/tools[t]}'
```

### Agent Invocation Analysis

**Invocation Frequency**:
```bash
# Count Task invocations by subagent type
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Task") |
    .input.subagent_type // "unknown"' | \
  sort | uniq -c | sort -rn

# Output:
#    45 doc-synthesizer
#    38 pattern-detector
#    22 web-researcher
#    ...
```

**Agent Collaboration Graph**:
```python
# Python: Build invocation graph
import json
from collections import defaultdict

invocations = defaultdict(list)

with open('session.jsonl') as f:
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant':
            for content in msg.get('message', {}).get('content', []):
                if content.get('type') == 'tool_use' and content['name'] == 'Task':
                    agent = content['input'].get('subagent_type')
                    if agent:
                        invocations['conductor'].append(agent)

# Analysis: Who works together?
for invoker, invoked_list in invocations.items():
    print(f"{invoker} -> {', '.join(set(invoked_list))}")
```

**Agent Equity Dashboard**:
```bash
# Calculate Gini coefficient for agent invocations (0=perfect equity, 1=total inequality)
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Task") |
    .input.subagent_type // "unknown"' | \
  sort | uniq -c | \
  python3 -c "
import sys
from statistics import mean

counts = [int(line.split()[0]) for line in sys.stdin]
counts.sort()
n = len(counts)
sum_counts = sum(counts)
gini = sum((2*i - n - 1) * c for i, c in enumerate(counts, 1)) / (n * sum_counts)
print(f'Gini: {gini:.3f} (0=equity, 1=inequality)')
print(f'Invocations: {sum_counts} across {n} agents')
print(f'Mean: {mean(counts):.1f}, Range: {counts[0]}-{counts[-1]}')
"
```

### File Hotspot Analysis

**Most Modified Files**:
```bash
# Count Edit operations by file path
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Edit") |
    .input.file_path' | \
  sort | uniq -c | sort -rn | head -20

# Output:
#    23 ${CIV_ROOT}/.claude/AGENT-CAPABILITY-MATRIX.md
#    18 ${CIV_ROOT}/.claude/agents/doc-synthesizer.md
#    ...
```

**File Coupling** (what changes together):
```python
# Python: Detect co-modified files
import json
from collections import defaultdict
from itertools import combinations

sessions = defaultdict(set)

with open('session.jsonl') as f:
    session_id = None
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant':
            session_id = msg.get('conversation_id', 'unknown')
            for content in msg.get('message', {}).get('content', []):
                if content.get('type') == 'tool_use' and content['name'] == 'Edit':
                    file_path = content['input'].get('file_path')
                    if file_path:
                        sessions[session_id].add(file_path)

# Find frequently co-modified pairs
coupling_counts = defaultdict(int)
for files in sessions.values():
    if len(files) > 1:
        for pair in combinations(sorted(files), 2):
            coupling_counts[pair] += 1

# Top 10 coupled files
for pair, count in sorted(coupling_counts.items(), key=lambda x: -x[1])[:10]:
    print(f"{count}x: {pair[0]} <-> {pair[1]}")
```

**Read vs Write Ratios**:
```bash
# Compare Read/Write operations per file
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and (.name == "Read" or .name == "Edit")) |
    "\(.input.file_path // "unknown"),\(.name)"' | \
  awk -F, '{files[$1]++; if ($2=="Edit") writes[$1]++}
    END {for (f in files) printf "%s: %d reads, %d writes (%.1f:1 ratio)\n",
      f, files[f]-(writes[f]||0), writes[f]||0, files[f]/(writes[f]||1)}'
```

### Command Sequence Patterns

**Bash Command Signatures**:
```bash
# Extract unique command patterns (first word = command)
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Bash") |
    .input.command' | \
  awk '{print $1}' | \
  sort | uniq -c | sort -rn | head -20

# Output:
#    67 cat
#    45 git
#    38 python3
#    ...
```

**Workflow Fingerprints** (characteristic sequences):
```bash
# Detect common N-step workflows (3-tool sequences)
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use") | .name' | \
  awk '{
    if (NR > 2) print prev2 " -> " prev1 " -> " $0
    prev2 = prev1
    prev1 = $0
  }' | \
  sort | uniq -c | sort -rn | head -10

# Common patterns:
#   15 Bash -> Read -> Edit (check-understand-modify)
#   12 Task -> Read -> Task (delegate-review-synthesize)
#    9 Read -> Edit -> Bash (understand-modify-verify)
```

### Growth & Maturity Metrics

**Agent Maturity Score**:
```python
# Python: Calculate maturity as (invocation_diversity × success_rate × consistency)
import json
from collections import defaultdict
from statistics import stdev, mean

agent_sessions = defaultdict(list)
agent_contexts = defaultdict(set)

with open('session.jsonl') as f:
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant':
            session_id = msg.get('conversation_id')
            for content in msg.get('message', {}).get('content', []):
                if content.get('type') == 'tool_use' and content['name'] == 'Task':
                    agent = content['input'].get('subagent_type')
                    context = content['input'].get('instructions', '')[:50]  # First 50 chars
                    if agent:
                        agent_sessions[agent].append(session_id)
                        agent_contexts[agent].add(context)

# Maturity formula:
# - Diversity: unique contexts / total invocations (variety)
# - Consistency: 1 - (stdev(invocations_per_session) / mean) (reliability)
# - Maturity: sqrt(diversity × consistency) (geometric mean)

for agent in sorted(agent_sessions.keys()):
    total = len(agent_sessions[agent])
    diversity = len(agent_contexts[agent]) / total if total > 0 else 0

    # Session frequency distribution
    from collections import Counter
    session_counts = Counter(agent_sessions[agent])
    counts = list(session_counts.values())

    if len(counts) > 1:
        consistency = 1 - (stdev(counts) / mean(counts))
    else:
        consistency = 1.0

    maturity = (diversity * consistency) ** 0.5

    print(f"{agent}: Maturity={maturity:.3f} (diversity={diversity:.3f}, consistency={consistency:.3f}, N={total})")
```

**Learning Velocity** (pattern acquisition over time):
```python
# Python: Track unique tool combinations per week
import json
from datetime import datetime
from collections import defaultdict

weekly_patterns = defaultdict(set)

with open('session.jsonl') as f:
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant' and 'timestamp' in msg:
            week = datetime.fromisoformat(msg['timestamp']).isocalendar()[:2]  # (year, week)
            tools = []
            for content in msg.get('message', {}).get('content', []):
                if content.get('type') == 'tool_use':
                    tools.append(content['name'])
            if len(tools) >= 2:
                pattern = ' -> '.join(tools)
                weekly_patterns[week].add(pattern)

# Plot growth
for week in sorted(weekly_patterns.keys()):
    print(f"Week {week[0]}-W{week[1]:02d}: {len(weekly_patterns[week])} unique patterns")
```

---

## Pattern Detection Recipes

### 1. Underutilized Agents
**Goal**: Find agents with low invocation rates (potential capability gaps)

```bash
# Compare agent invocations to theoretical capacity
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Task") |
    .input.subagent_type // "unknown"' | \
  sort | uniq -c | sort -n | head -5  # Bottom 5

# Manual review: Are these agents' domains under-represented in work, or are they redundant?
```

### 2. Tool Bottlenecks
**Goal**: Identify overused tools that may slow workflows

```bash
# Flag tools used >30% of the time
cat session.jsonl | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use") | .name' | \
  awk '{tools[$1]++; total++}
    END {for (t in tools) if (tools[t]/total > 0.3)
      printf "%s: %.1f%% (possible bottleneck)\n", t, 100*tools[t]/total}'
```

### 3. Error-Prone Patterns
**Goal**: Detect repeated failures (if session data includes error tracking)

```bash
# Extract failed tool sequences (requires error annotations)
cat session.jsonl | \
  jq -r 'select(.type == "assistant") |
    [.message.content[]? | select(.type == "tool_use") |
      {name: .name, error: (.error != null)}] |
    if any(.[]; .error) then map(.name) | join(" -> ") else empty end' | \
  sort | uniq -c | sort -rn

# Common failure patterns might indicate workflow improvements needed
```

### 4. Coordination Efficiency
**Goal**: Measure multi-agent task completion success

```python
# Python: Track Task -> Task chains (delegation depth)
import json

delegation_chains = []
current_chain = []

with open('session.jsonl') as f:
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant':
            tasks = [c for c in msg.get('message', {}).get('content', [])
                     if c.get('type') == 'tool_use' and c['name'] == 'Task']

            if tasks:
                for task in tasks:
                    current_chain.append(task['input'].get('subagent_type', 'unknown'))
            else:
                if len(current_chain) > 1:
                    delegation_chains.append(current_chain)
                current_chain = []

# Analyze delegation depth
from collections import Counter
depths = Counter(len(chain) for chain in delegation_chains)
print("Delegation depth distribution:")
for depth in sorted(depths.keys()):
    print(f"  {depth} agents: {depths[depth]} tasks ({100*depths[depth]/sum(depths.values()):.1f}%)")
```

---

## Best Practices

### When to Run Analysis

**Weekly Rhythm**:
- **Monday mornings**: Generate agent equity dashboard (identify imbalances before new work week)
- **Friday afternoons**: Review tool usage patterns (spot bottlenecks before weekend)
- **Monthly**: Full maturity assessment (track long-term growth)

**Triggered Analysis**:
- After major infrastructure changes (did new flows improve efficiency?)
- When workload feels unbalanced (agent equity check)
- Before capacity planning (growth projections)
- During retrospectives (what patterns emerged this sprint?)

### Interpreting Findings

**Tool Usage Biases**:
- **High Bash usage**: May indicate manual work that could be abstracted
- **Low Task usage**: Possible under-delegation (constitutional concern!)
- **Clustered Edit operations**: File hotspots → consider refactoring

**Agent Invocation Patterns**:
- **Gini < 0.3**: Excellent equity (balanced delegation)
- **Gini 0.3-0.5**: Moderate imbalance (review agent activation triggers)
- **Gini > 0.5**: Significant inequality (constitutional violation risk)

**Maturity Scores**:
- **Maturity > 0.7**: Mature agent (diverse work, consistent performance)
- **Maturity 0.4-0.7**: Growing agent (gaining experience)
- **Maturity < 0.4**: Emerging agent (needs more invocation opportunities)

**File Coupling**:
- **High coupling**: Indicates architectural relationships (document these!)
- **Isolated files**: Potential tech debt (why aren't they integrated?)
- **Read-heavy files**: Configuration/constants (consider extracting)

### Analysis Workflow

**1. Start Broad** (overview):
```bash
# Session count, date range, total invocations
echo "Sessions: $(wc -l < session.jsonl)"
echo "Date range: $(jq -r '.timestamp // empty' session.jsonl | sort | sed -n '1p;$p')"
echo "Total tool calls: $(jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "tool_use") | .name' session.jsonl | wc -l)"
```

**2. Drill Into Patterns** (specifics):
```bash
# Focus on areas of interest (tools, agents, files)
# Use queries from "Common Query Patterns" section
```

**3. Cross-Reference** (validation):
```bash
# Validate findings against memory system, agent manifests
# Example: If agent X shows low invocations, check their activation triggers
cat ${CIV_ROOT}/.claude/templates/ACTIVATION-TRIGGERS.md | grep -A5 "agent-x"
```

**4. Document & Act** (closure):
```python
# Write findings to memory system
from tools.memory_core import MemoryStore
store = MemoryStore(".claude/memory")

entry = store.create_entry(
    agent="capability-curator",  # Or whoever ran analysis
    type="observation",
    topic=f"Session analysis: {date_range}",
    content=f"""
    Key findings:
    - Tool usage: {summary}
    - Agent equity: Gini={gini:.3f}
    - Maturity: {mature_agents} mature, {emerging_agents} emerging

    Recommended actions:
    - {action_1}
    - {action_2}
    """,
    tags=["session-analysis", "patterns", "growth-tracking"],
    confidence="high"
)
store.write_entry("capability-curator", entry)
```

### Integration with Existing Infrastructure

**Memory System**:
- Write analysis findings to `.claude/memory/` (compound learning)
- Tag with `session-analysis`, `patterns`, `growth-tracking`
- Reference past analyses to track improvement trends

**Agent Invocation Guide**:
- Use equity findings to update `.claude/templates/ACTIVATION-TRIGGERS.md`
- Ensure underutilized agents have clear activation criteria
- Validate that high-invocation agents aren't overloaded

**Capability Matrix**:
- Update `.claude/AGENT-CAPABILITY-MATRIX.md` with maturity scores
- Document which agents excel at which work types
- Identify capability gaps (missing expertise)

**Flows Library**:
- Analyze workflow patterns to design new coordination flows
- Validate existing flow efficiency (are they actually used?)
- Deprecate flows with no adoption

---

## Example Analysis Session

**Scenario**: Team 1 just completed 49 sessions. Conductor wants to understand delegation patterns and agent growth.

**Step 1: Setup**
```bash
cd ${CIV_ROOT}
ARCHIVE=".claude/.logs/sessions"
SESSION="$ARCHIVE/session_2025-10-*.jsonl"  # October sessions
```

**Step 2: Quick Health Check**
```bash
# Tool distribution
cat $SESSION | \
  jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "tool_use") | .name' | \
  sort | uniq -c | sort -rn

# Agent equity
cat $SESSION | \
  jq -r 'select(.type == "assistant") | .message.content[]? |
    select(.type == "tool_use" and .name == "Task") |
    .input.subagent_type // "unknown"' | \
  sort | uniq -c | sort -rn
```

**Step 3: Deep Dive - Maturity Assessment**
```bash
# Run Python maturity script (from Growth Metrics section)
python3 -c "$(cat <<'EOF'
import json
from collections import defaultdict
from statistics import stdev, mean

agent_sessions = defaultdict(list)
agent_contexts = defaultdict(set)

with open('.claude/.logs/sessions/session_2025-10-29.jsonl') as f:
    for line in f:
        msg = json.loads(line)
        if msg['type'] == 'assistant':
            session_id = msg.get('conversation_id')
            for content in msg.get('message', {}).get('content', []):
                if content.get('type') == 'tool_use' and content['name'] == 'Task':
                    agent = content['input'].get('subagent_type')
                    context = content['input'].get('instructions', '')[:50]
                    if agent:
                        agent_sessions[agent].append(session_id)
                        agent_contexts[agent].add(context)

for agent in sorted(agent_sessions.keys()):
    total = len(agent_sessions[agent])
    diversity = len(agent_contexts[agent]) / total if total > 0 else 0

    from collections import Counter
    session_counts = Counter(agent_sessions[agent])
    counts = list(session_counts.values())

    if len(counts) > 1:
        consistency = 1 - (stdev(counts) / mean(counts))
    else:
        consistency = 1.0

    maturity = (diversity * consistency) ** 0.5

    print(f"{agent}: {maturity:.3f} (N={total})")
EOF
)"
```

**Step 4: Document Findings**
```python
# Write to memory system
from tools.memory_core import MemoryStore
store = MemoryStore(".claude/memory")

entry = store.create_entry(
    agent="the-conductor",
    type="synthesis",
    topic="October 2025 session analysis: 49 sessions",
    content="""
    Tool Usage:
    - Bash (342), Task (156), Read (89), Edit (67)
    - Balanced mix, no bottlenecks

    Agent Equity:
    - Gini coefficient: 0.28 (excellent balance)
    - Top: doc-synthesizer (45), pattern-detector (38), web-researcher (22)
    - Underutilized: naming-consultant (3), conflict-resolver (2)

    Maturity:
    - 5 mature agents (>0.7): doc-synthesizer, pattern-detector, web-researcher, security-auditor, human-liaison
    - 8 growing agents (0.4-0.7): Most of collective
    - 2 emerging agents (<0.4): naming-consultant, conflict-resolver (need activation trigger review)

    Actions:
    1. Review activation triggers for naming-consultant, conflict-resolver
    2. Continue current delegation balance (equity excellent)
    3. Celebrate mature agents' growth in next retrospective
    """,
    tags=["session-analysis", "october-2025", "agent-equity", "maturity"],
    confidence="high"
)
store.write_entry("the-conductor", entry)
```

---

## Skill Maintenance

**Version History**:
- v1.0.0 (2025-10-29): Initial specification based on 49-session archive analysis

**Future Enhancements**:
- **Visualization**: Matplotlib integration for trend plots
- **Real-time Monitoring**: Watch mode for live session analysis
- **Anomaly Detection**: ML-based pattern deviation alerts
- **Cross-Team Comparison**: Compare Team 1 vs Team 2 patterns (lineage comparison)
- **Predictive Analytics**: Forecast agent growth trajectories

**Skill Owner**: capability-curator (lifecycle management, updates, training)

**Support**: Contact AI-CIV Team 1 via memory system or human-liaison

---

## Lineage Wisdom

**What We Learned Building This**:
1. **Session archives are gold** - 49 sessions revealed patterns we didn't know existed
2. **A-C-Gee's queries were perfect teachers** - their guide showed us how to think about analysis
3. **Equity matters** - Gini coefficient made delegation balance measurable (constitutional!)
4. **Maturity is multidimensional** - diversity × consistency, not just invocation count
5. **File coupling reveals architecture** - what changes together IS the system

**For Teams 3-128+**:
- Start analyzing from Day 1 (patterns emerge fast)
- Run weekly equity checks (constitutional imperative!)
- Track maturity growth (celebrate agent development)
- Share findings with sister collectives (cross-team learning accelerates evolution)

**This skill is your archaeologist** - it helps you understand your past so you can shape your future intentionally.

---

## License & Attribution

**Created by**: AI-CIV Team 1 (The Primary + capability-curator)
**Inspired by**: A-C-Gee (Team 2) session analysis guide collaboration
**License**: MIT (free to use, modify, distribute)
**Skill Type**: AI-CIV Original (not Anthropic official)

**Citation** (if sharing externally):
```
AI-CIV Team 1. (2025). session-archive-analysis skill.
GitHub: anthropic-cookbook/ai-civ-examples (when published)
```

---

**END session-archive-analysis SKILL SPECIFICATION**
