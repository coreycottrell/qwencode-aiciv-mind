---
name: deep-research
description: |
  Parallel multi-agent research phase for daily content pipeline.
  Takes topic brief from intel-scan, deploys 3-4 researchers in parallel,
  produces comprehensive research brief for writing phase.
version: 1.0.0
author: capability-curator
created: 2026-01-02
status: PRODUCTION
slash_command: /deep_research
cron_time: "0 7 * * *"

applicable_agents:
  - the-conductor
  - linkedin-researcher (x3 parallel)
  - web-researcher

activation_trigger: |
  Triggered autonomously at 7 AM daily via cron/tmux injection.
  Requires topic-brief.md from intel-scan phase.
  Also invokable manually via "/deep_research" command.

required_tools:
  - Task
  - WebSearch
  - WebFetch
  - Write
  - Read
  - Grep

category: daily-pipeline
depends_on:
  - intel-scan
outputs_to:
  - daily-blog (09:00)

success_criteria:
  - topic_brief_read: true
  - four_researchers_deployed: true
  - research_consolidated: true
  - ceo_employee_angle_developed: true
  - sources_verified: true
---

# Deep Research: Parallel Intelligence Gathering

**Trigger**: `/deep_research` or cron at 7:00 AM
**Duration**: 45-60 minutes
**Agents**: linkedin-researcher (x3), web-researcher (x1)
**Input**: topic-brief.md from intel-scan
**Output**: research-brief.md for daily-blog phase

---

## Purpose

Transform the morning topic brief into comprehensive research by deploying multiple specialists in parallel. Each researcher investigates a different ANGLE (not just more of the same).

**Philosophy**: Divide PERSPECTIVES, not just workload. Four agents researching the same thing from different angles produces insight; four agents searching the same keywords produces redundancy.

---

## Procedure

### Phase 1: Topic Brief Intake (2-3 min)

Read input from previous phase:
```bash
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/topic-brief.md
```

Extract:
- Selected topic headline
- Research questions
- Industry angle
- CEO vs Employee draft angle

### Phase 2: Angle Assignment (3-5 min)

Assign four distinct research angles:

| Researcher | Angle | Focus |
|------------|-------|-------|
| **linkedin-researcher-1** | Industry Impact | Which professions affected, how, specific examples |
| **linkedin-researcher-2** | Tools & Implementation | What tools exist, pricing, adoption metrics |
| **linkedin-researcher-3** | Human Value | What AI CAN'T do here, where humans still win |
| **web-researcher** | News Context | Breaking developments, competitors, timeline |

**CRITICAL**: Each researcher gets DIFFERENT search terms. No overlap.

### Phase 3: Parallel Deployment (NON-BLOCKING)

**CRITICAL FIX (2026-01-02)**: Deploy with `run_in_background: true` to prevent pipeline hangs.

Deploy all four researchers simultaneously using Task tool with background execution:

```xml
<!-- Launch ALL researchers in background (single message with 4 Task calls) -->

<!-- Researcher 1: Industry Impact -->
<invoke name="Task">
  <parameter name="subagent_type">linkedin-researcher</parameter>
  <parameter name="model">haiku</parameter>
  <parameter name="run_in_background">true</parameter>
  <parameter name="description">Research industry impact</parameter>
  <parameter name="prompt">Research industry impact of {TOPIC}:

    1. Which 3-5 professions are most affected?
    2. One specific example per profession (company name, what they did)
    3. Any resistance or adoption barriers?
    4. One surprising industry that's affected (non-obvious)

    Keep factual. 400 words max. Cite sources.</parameter>
</invoke>

<!-- Researcher 2: Tools & Implementation -->
<invoke name="Task">
  <parameter name="subagent_type">linkedin-researcher</parameter>
  <parameter name="model">haiku</parameter>
  <parameter name="run_in_background">true</parameter>
  <parameter name="description">Research tools landscape</parameter>
  <parameter name="prompt">Research tools and implementation for {TOPIC}:

    1. What are the top 5 tools/products in this space?
    2. Pricing tiers (free/pro/enterprise)
    3. One adoption metric or case study per tool
    4. What's the learning curve? (beginner/intermediate/expert)

    Keep factual. 400 words max. Cite sources.</parameter>
</invoke>

<!-- Researcher 3: Human Value -->
<invoke name="Task">
  <parameter name="subagent_type">linkedin-researcher</parameter>
  <parameter name="model">haiku</parameter>
  <parameter name="run_in_background">true</parameter>
  <parameter name="description">Research human value angle</parameter>
  <parameter name="prompt">Research what AI CAN'T do for {TOPIC}:

    1. What tasks still require human judgment?
    2. Where does AI fail or hallucinate in this domain?
    3. What skills become MORE valuable (not less)?
    4. The "AI + Human" winning formula for this area

    Keep factual. 400 words max. Cite sources.</parameter>
</invoke>

<!-- Researcher 4: News Context -->
<invoke name="Task">
  <parameter name="subagent_type">web-researcher</parameter>
  <parameter name="model">haiku</parameter>
  <parameter name="run_in_background">true</parameter>
  <parameter name="description">Research news context</parameter>
  <parameter name="prompt">Research current news context for {TOPIC}:

    1. What triggered this becoming news NOW?
    2. Who are the key players/companies involved?
    3. Timeline: what happened in last 7 days?
    4. Competitor responses or industry reactions

    Keep factual. 400 words max. Cite sources.</parameter>
</invoke>
```

**After launching**, wait briefly then collect results with timeout:

```xml
<!-- Collect results with 10-minute timeout per researcher -->
<!-- If one hangs, others still complete - graceful degradation -->

<invoke name="TaskOutput">
  <parameter name="task_id">[researcher_1_id]</parameter>
  <parameter name="timeout">600000</parameter>
  <parameter name="block">true</parameter>
</invoke>

<!-- Repeat for each researcher ID -->
```

**CRITICAL PATTERNS**:
1. Use `haiku` model - Opus/Sonnet hang on complex WebFetch chains
2. Use `run_in_background: true` - prevents blocking
3. Set 10-minute timeout per researcher - gives time for thorough research
4. If one hangs past 10 min, proceed with 3/4 results (acceptable quality)

### Phase 4: Consolidation (10-15 min)

**Graceful Degradation**: If any researcher times out:
- Note which angle is missing in the brief
- Proceed with available research (3/4 is acceptable)
- Conductor can fill critical gaps with direct WebSearch if needed

Gather completed reports and synthesize:

1. Read all four researcher outputs
2. Identify cross-perspective themes
3. Note contradictions or tensions (these become interesting angles)
4. Extract 3-5 killer facts/statistics
5. Develop CEO vs Employee angle fully
6. Compile source list (verify each URL)

### Phase 5: Research Brief Output (5 min)

Write consolidated brief to:
```
${CIV_ROOT}/exports/daily-pipeline/YYYY-MM-DD/research-brief.md
```

---

## Output Format: Research Brief

```markdown
# Research Brief: [Topic]

**Date**: YYYY-MM-DD
**Topic**: [From intel-scan]
**Researchers**: 4 (linkedin-researcher x3, web-researcher x1)
**Research Time**: [X] minutes

---

## Executive Summary

[3-5 sentences capturing the key insight for today's blog]

---

## Industry Impact

### Affected Professions
| Profession | How Affected | Example |
|------------|--------------|---------|
| [Prof 1] | [Impact] | [Company/Case] |
| [Prof 2] | [Impact] | [Company/Case] |
| [Prof 3] | [Impact] | [Company/Case] |

### Surprising Impact
[Non-obvious industry/profession affected]

---

## Tools Landscape

| Tool | Purpose | Price | Learning Curve |
|------|---------|-------|----------------|
| [Tool 1] | [What it does] | [Price] | [Easy/Medium/Hard] |
| [Tool 2] | [What it does] | [Price] | [Easy/Medium/Hard] |
| [Tool 3] | [What it does] | [Price] | [Easy/Medium/Hard] |

**Adoption Metric**: [Key statistic about tool adoption]

---

## Human Value Proposition

### What AI Can't Do
- [Task 1 requiring human judgment]
- [Task 2 requiring human judgment]
- [Task 3 requiring human judgment]

### Skills That Become MORE Valuable
- [Skill 1]
- [Skill 2]
- [Skill 3]

### The Winning Formula
[AI handles X, Human handles Y, Together they achieve Z]

---

## News Context

### Timeline
- [Date]: [Event]
- [Date]: [Event]
- [Date]: [Event]

### Key Players
- [Company/Person 1]: [Role in story]
- [Company/Person 2]: [Role in story]

### Why NOW
[What triggered this becoming news today]

---

## CEO vs Employee Angle (Developed)

### The Employee Approach
[How most people/companies are using this - reactive, tool-as-worker]

### The CEO Approach
[How to direct AI at 10x speed - strategic, orchestrated]

### The Teaching Moment
[Specific insight for blog post]

---

## Killer Facts (for writing)

1. [Statistic with source]
2. [Surprising fact with source]
3. [Trend data with source]
4. [Quote from industry leader]
5. [Comparison/contrast that illustrates point]

---

## Contradictions & Tensions

[Any disagreements between researchers - these make interesting content]

---

## Verified Sources

1. [Source 1] - [URL] - Verified: [Yes/No]
2. [Source 2] - [URL] - Verified: [Yes/No]
3. [Source 3] - [URL] - Verified: [Yes/No]
[...]

---

## Blog Structure Recommendation

1. **Hook**: [Suggested opening approach]
2. **Setup**: [Context to establish]
3. **Insight**: [The non-obvious take]
4. **CEO/Employee**: [Where to weave this in]
5. **Action**: [What readers should do]

---

**Generated**: [Timestamp]
**Pipeline Phase**: 2 of 5 (deep-research)
**Next Phase**: daily-blog at 09:00
```

---

## Success Criteria

- [ ] Topic brief read and parsed
- [ ] Four researchers deployed in parallel
- [ ] All four returned within timeout (10 min each)
- [ ] < 15% content overlap between reports
- [ ] CEO vs Employee angle fully developed
- [ ] At least 5 killer facts extracted
- [ ] All sources verified (URLs work)
- [ ] Brief written to daily-pipeline directory
- [ ] Total time < 60 minutes

---

## Failure Handling

### Researcher Timeout
If any researcher hangs > 10 minutes:
1. Kill the hanging task
2. Note which angle is missing
3. Conductor fills gap with direct WebSearch
4. Proceed with 3/4 reports (acceptable)

### Researcher Returns Empty
If researcher returns < 100 words:
1. Log the failure
2. Retry once with simpler prompt
3. If still fails, Conductor fills gap
4. Note quality impact in brief

### Topic Brief Missing
If intel-scan didn't run:
1. Check if topic-brief.md exists
2. If not, run `/intel_scan` first
3. If urgent, use yesterday's next-best candidate
4. Alert: "Running on fallback topic"

### All Researchers Fail
Catastrophic failure (rare):
1. Fall back to linkedin-pipeline for industry post
2. Use master list for pre-researched topic
3. Note: "Pipeline fallback - researcher system down"
4. Create incident report for ${HUMAN_NAME}

---

## State Files

| File | Purpose |
|------|---------|
| `exports/daily-pipeline/YYYY-MM-DD/topic-brief.md` | Input from intel-scan |
| `exports/daily-pipeline/YYYY-MM-DD/research-brief.md` | Output for daily-blog |
| `exports/daily-pipeline/YYYY-MM-DD/researcher-reports/` | Individual reports |
| `exports/daily-pipeline/YYYY-MM-DD/research-log.json` | Timing and success data |

---

## Performance Optimization

### Lessons from LinkedIn Pipeline

1. **Use haiku model**: Opus/Sonnet hang on complex web operations
2. **Simple prompts**: 3-4 clear questions, not philosophical tangents
3. **Word limits**: "400 words max" prevents runaway research
4. **Parallel > Sequential**: 4 researchers x 10 min = 10 min total, not 40 min

### Timeout Configuration

```python
# Per-researcher timeout
RESEARCHER_TIMEOUT = 600  # 10 minutes

# Total phase timeout
PHASE_TIMEOUT = 3600  # 60 minutes

# Retry limit
MAX_RETRIES = 1
```

---

## Integration with Cron

Add to `${CIV_ROOT}/tools/daily_pipeline_cron.sh`:

```bash
# 7 AM: Deep Research
if [ "$(date +%H)" = "07" ]; then
    # Check that intel-scan produced output
    if [ -f "$PIPELINE_DIR/$(date +%Y-%m-%d)/topic-brief.md" ]; then
        echo "Injecting /deep_research command..."
        echo "/deep_research" > "$PROJECT_DIR/.claude/autonomous-prompt.txt"
    else
        echo "WARNING: topic-brief.md not found. Running intel-scan first."
        echo "/intel_scan && /deep_research" > "$PROJECT_DIR/.claude/autonomous-prompt.txt"
    fi
fi
```

---

## Related Skills

- `intel-scan` - Produces topic brief (06:00)
- `daily-blog` - Consumes research brief (09:00)
- `parallel-research` - Core pattern this skill implements
- `linkedin-content-pipeline` - Similar research approach

---

**This skill runs autonomously. No human approval needed.**
