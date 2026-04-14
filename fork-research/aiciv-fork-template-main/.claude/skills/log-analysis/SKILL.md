---
name: log-analysis
version: 1.0.0
author: coder
created: 2025-12-16
last_updated: 2025-12-26
line_count: 338
compliance_status: compliant

applicable_agents:
  - primary
  - auditor
  - primary-helper

activation_trigger: |
  Load when: BOOP reports, debugging agent behavior, token usage tracking,
  session history analysis, error investigation, delegation pattern review

required_tools:
  - Bash
  - Read

category: custom
depends_on: []
related_skills:
  - session-handoff-creation
  - git-archaeology-investigation
---

# Log Analysis Skill

Complete reference for analyzing Claude Code JSONL logs to extract session metrics, agent performance, and system health insights.

## Tools Overview

| Tool | Purpose | Primary Output |
|------|---------|----------------|
| `claude_log_analyzer.py` | Batch analysis of log files | Metrics summary |
| `log_watcher.py` | Auto-trigger on new logs | Rolling database |
| `log_consolidation_report.py` | BOOP-focused reports | Trends, recommendations |
| `generate_session_json.py` | Individual session summaries | Per-session JSON |
| `session_self_review.py` | Self-review coordination | Review prompts, batch delegation |
| `migrate_sessions_for_review.py` | One-time migration | Adds review_meta |

---

## Tool 1: Main Analyzer (`tools/claude_log_analyzer.py`)

Parse and analyze Claude Code JSONL logs for metrics extraction.

### Usage

```bash
# Last N hours
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py --hours 24

# Single file
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py ~/.claude/projects/.../session.jsonl

# Filter by type
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py --type primary --hours 12

# JSON output
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py --hours 6 --json
```

### Metrics Extracted

- **Tokens**: input, output, cache_creation, cache_read
- **Tools**: Aggregated counts (Bash, Read, Write, Edit, Task, etc.)
- **Delegations**: Agent names, prompt previews, timestamps
- **Errors**: Exit codes, tracebacks, tool failures
- **Session**: Duration, entry counts, models

---

## Tool 2: File Watcher (`tools/log_watcher.py`)

Continuously monitor log directory, auto-trigger analysis on new files.

### Usage

```bash
# Start watching
python3 ${ACG_ROOT}/tools/log_watcher.py

# Custom poll interval
python3 ${ACG_ROOT}/tools/log_watcher.py --poll-interval 30

# Reset state / Dry run
python3 ${ACG_ROOT}/tools/log_watcher.py --reset-state
python3 ${ACG_ROOT}/tools/log_watcher.py --dry-run
```

### Files

- **State**: `memories/system/log_analysis/.watcher_state.json`
- **Rolling DB**: `memories/system/log_analysis/rolling_analysis.jsonl`

---

## Tool 3: Consolidation Report (`tools/log_consolidation_report.py`)

Generate trend reports optimized for BOOP consolidation cycles.

### Usage

```bash
# Default 6-hour window
python3 ${ACG_ROOT}/tools/log_consolidation_report.py

# Custom window / JSON
python3 ${ACG_ROOT}/tools/log_consolidation_report.py --hours 12
python3 ${ACG_ROOT}/tools/log_consolidation_report.py --json
```

### Report Contents

1. **Agent Performance**: Success rates, delegation counts
2. **Error Hotspots**: Highest error rate tools/agents
3. **Token Efficiency**: Input/output ratios, cache utilization
4. **Trends**: Current vs historical baseline
5. **Recommendations**: Auto-generated suggestions

---

## Tool 4: Session JSON Generator (`tools/generate_session_json.py`)

Create detailed JSON summaries for individual session files.

### Usage

```bash
# Single file
python3 ${ACG_ROOT}/tools/generate_session_json.py ~/.claude/projects/.../session.jsonl

# All unprocessed / Force reprocess
python3 ${ACG_ROOT}/tools/generate_session_json.py --all
python3 ${ACG_ROOT}/tools/generate_session_json.py --all --force
```

### Output

- **Location**: `memories/system/log_analysis/sessions/`
- **Naming**: `{uuid}.json` (primary) or `agent-{hexid}.json`
- **Contents**: overview, tokens, tools_used, delegations, errors, files_modified, notes

---

## Tool 5: Session Self-Review (`tools/session_self_review.py`)

Coordinate agent self-review with batch support and BOOP integration.

### Usage

```bash
# List unreviewed / Priority sorted
python3 ${ACG_ROOT}/tools/session_self_review.py --list
python3 ${ACG_ROOT}/tools/session_self_review.py --list --priority

# Generate batch prompts
python3 ${ACG_ROOT}/tools/session_self_review.py --batch 3 --priority

# Filter by agent / Stats
python3 ${ACG_ROOT}/tools/session_self_review.py --list --agent coder
python3 ${ACG_ROOT}/tools/session_self_review.py --stats
python3 ${ACG_ROOT}/tools/session_self_review.py --boop-stats
```

### Agent Affinity Routing

| Condition | Assigned Agent |
|-----------|----------------|
| 3+ errors | tester |
| Files contain "arcx" | arcx-coder |
| Primary + 3+ delegations | primary-helper |
| Files contain "email/inbox" | human-liaison |
| Files contain "comms/weaver" | comms-hub |
| Default agent sessions | Self-review |

### Priority Scoring

| Factor | Score |
|--------|-------|
| Each error (max 3) | +20 |
| Duration > 60/120 min | +10/+20 |
| Delegations > 5/10 | +5/+10 |
| Session < 24h/72h old | +10/+5 |

**Levels**: High (30+), Medium (10-29), Low (0-9)

### Review Output Location

Reviews saved to: `memories/system/session_reviews/by-agent/{agent}/{month}/review-{session_id}.json`

---

## Tool 6: Session Migration (`tools/migrate_sessions_for_review.py`)

One-time migration to add `review_meta` to existing session JSONs.

### Usage

```bash
# Preview / Run / Stats
python3 ${ACG_ROOT}/tools/migrate_sessions_for_review.py --dry-run
python3 ${ACG_ROOT}/tools/migrate_sessions_for_review.py
python3 ${ACG_ROOT}/tools/migrate_sessions_for_review.py --stats
```

Adds `review_meta` field with: status, assigned_to, priority_score, review_depth, agent_affinity.

---

## Output Locations

```
memories/system/log_analysis/
├── sessions/                     # Individual session JSON summaries
│   ├── {uuid}.json              # Primary sessions
│   └── agent-{hexid}.json       # Agent sessions
├── rolling_analysis.jsonl        # Watcher's rolling database
└── .watcher_state.json          # Processed file tracking

memories/system/session_reviews/
└── by-agent/{agent}/{month}/    # Month-organized reviews
```

---

## Common Patterns

### Morning BOOP Preparation

```bash
python3 ${ACG_ROOT}/tools/log_consolidation_report.py --hours 12
python3 ${ACG_ROOT}/tools/generate_session_json.py --all
```

### Debug Agent Failure

```bash
ls -lt ~/.claude/projects/-home-${HUMAN_NAME_LOWER}-projects-AI-CIV-ACG/agent-*.jsonl | head -5
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py ~/.claude/projects/.../agent-040bbcec.jsonl
```

### Token Usage Audit

```bash
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py --hours 24 --json | \
  python3 -c "import json,sys; d=json.load(sys.stdin); print(f'Input: {d[\"tokens\"][\"input\"]:,}\nOutput: {d[\"tokens\"][\"output\"]:,}')"
```

### Delegation Chain Analysis

```bash
python3 ${ACG_ROOT}/tools/claude_log_analyzer.py --type primary --hours 6 --json | \
  python3 -c "import json,sys; d=json.load(sys.stdin); print('\n'.join(f'{k}: {v}' for k,v in sorted(d.get('delegations_by_agent',{}).items(), key=lambda x:-x[1])))"
```

---

## Data Schema Reference

Full schema: `memories/knowledge/claude-code-log-schema.md`

**Key elements:**
- **Entry types**: user, assistant, file-history-snapshot, queue-operation, summary
- **Content blocks**: text, tool_use, tool_result
- **Stop reasons**: end_turn, tool_use, max_tokens
- **Task tool**: Uses `subagent_type` field (NOT `agent`)

---

## Error Detection Patterns

```python
ERROR_PATTERNS = [
    r'exit code [1-9]',           # Non-zero exit codes
    r'traceback \(most recent',   # Python tracebacks
    r'<tool_use_error>',          # Claude tool failures
    r'importerror:',              # Import failures
    r'syntaxerror:',              # Syntax errors
    r'is_error.*true',            # Explicit error flags
]
```

---

## Integration Points

### Session Ledger

Complements ledger with: token-level metrics, tool usage patterns, error detection across all tools.

### Primary Helper

Uses consolidation reports for: delegation pattern analysis, error rate tracking, agent performance comparisons.

### BOOP System

Designed for BOOP cycles: 6-hour default window, auto-generated recommendations, trend analysis for strategic decisions.

---

## Troubleshooting

### "No log files found"

1. Check: `ls ~/.claude/projects/-home-${HUMAN_NAME_LOWER}-projects-AI-CIV-ACG/`
2. Verify permissions
3. Use explicit path instead of `--hours`

### "JSON parse error"

Log files may be mid-write. Analyzer handles partial JSONL. To validate:
```bash
python3 -c "import json; [json.loads(l) for l in open('file.jsonl')]"
```

### "Agent extraction shows 'unknown'"

Task tool uses `subagent_type` field. Check raw log entry format.

---

## Performance

| Operation | Time | Memory |
|-----------|------|--------|
| Single file | <1s | ~10MB |
| Batch (20 files) | ~5s | ~50MB |
| Full generation (817 files) | ~30s | ~100MB |
| Rolling DB growth | ~3KB/file | N/A |

---

*Skill created 2025-12-16, refactored 2025-12-26 for 500-line compliance*
