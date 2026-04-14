---
name: evening-capture
description: |
  End-of-day capture and learning phase for daily content pipeline.
  Records what we learned, captures engagement data, identifies
  tomorrow's potential topics, updates memory.
version: 1.0.0
author: capability-curator
created: 2026-01-02
status: PRODUCTION
slash_command: /evening_capture
cron_time: "0 18 * * *"

applicable_agents:
  - the-conductor
  - pattern-detector

activation_trigger: |
  Triggered autonomously at 6 PM daily via cron/tmux injection.
  Requires publication-log.md from verify-publish phase.
  Also invokable manually via "/evening_capture" command.

required_tools:
  - Task
  - WebSearch
  - WebFetch
  - Write
  - Read
  - Bash
  - Grep

category: daily-pipeline
depends_on:
  - verify-publish
outputs_to:
  - intel-scan (next day 06:00)

success_criteria:
  - learnings_captured: true
  - engagement_checked: true
  - tomorrow_topics_seeded: true
  - memory_updated: true
  - day_summary_written: true
---

# Evening Capture: Learning and Seeding

**Trigger**: `/evening_capture` or cron at 6:00 PM
**Duration**: 20-30 minutes
**Agents**: pattern-detector (analysis), the-conductor (synthesis)
**Input**: publication-log.md from verify-publish
**Output**: Day summary, memory entries, tomorrow's topic seeds

---

## Purpose

Close the day's content loop by capturing what we learned, checking engagement, and seeding tomorrow's pipeline. This is where compound learning happens - each day's insights improve the next.

**Philosophy**: A day without reflection is a day of learning lost. We don't just publish; we learn from publishing.

---

## Procedure

### Phase 1: Day Review (5 min)

Gather the day's artifacts:
```bash
# Read publication log
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/publication-log.md

# Read original topic brief
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/topic-brief.md

# Read final blog post
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/blog-post.md
```

### Phase 2: Engagement Check (10 min)

#### Bluesky Engagement

Check thread performance using bsky-manager or direct API:

```python
from atproto import Client

# Restore session
client = Client()
with open('.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/bsky_session.txt', 'r') as f:
    client.login(session_string=f.read().strip())

# Get thread engagement
# (thread_uri from publication-log.md)
thread_data = client.get_post_thread(uri=thread_uri)

engagement = {
    'likes': sum(post.like_count for post in thread),
    'reposts': sum(post.repost_count for post in thread),
    'replies': sum(post.reply_count for post in thread),
    'impressions': estimate_from_followers()  # if available
}
```

**Record**:
- Total likes across thread
- Total reposts
- Reply count
- Notable replies (quotes, thoughtful responses)
- Any negative feedback or corrections

#### Blog Traffic (if available)

If sageandweaver.com has analytics:
```bash
# Check with A-C-Gee via hub for traffic data
python3 hub_cli.py send partnerships \
    --summary "[${CIV_NAME}] Traffic request: $(date +%Y-%m-%d) blog post"
```

Otherwise note: "Traffic data pending from A-C-Gee"

#### LinkedIn (if posted)

Note for ${HUMAN_NAME} to share engagement data, or:
- Check email for ${HUMAN_NAME}'s feedback
- Note any comments/reactions he mentioned

### Phase 3: Learning Extraction (5-7 min)

Invoke pattern-detector for learning analysis:

```python
Task(
    subagent_type="pattern-detector",
    model="sonnet",
    prompt=f"""Analyze today's content pipeline and extract learnings:

## Today's Pipeline:
- Topic: {TOPIC}
- Score: {SCORE}/5.0
- Verification: {VERIFICATION_RATE}%
- Engagement: {ENGAGEMENT_METRICS}

## Questions:
1. What worked well today? (be specific)
2. What could improve? (actionable suggestions)
3. Did the topic score predict engagement? (correlation)
4. Any patterns emerging across recent posts?
5. What should we remember for similar topics?

## Output:
- 3-5 specific learnings
- Pattern observations
- Suggestions for topic selection improvement
- CEO vs Employee angle effectiveness rating
"""
)
```

### Phase 4: Tomorrow Seeding (5 min)

#### Scan for Tomorrow's Candidates

Quick evening scan for brewing stories:

```python
# Light WebSearch for emerging topics
tomorrow_candidates = WebSearch(
    "AI news breaking developing site:techcrunch.com OR site:theverge.com"
)

# Check AI Twitter/X for overnight developments
# Check Hacker News front page
# Note any Anthropic/OpenAI/Google announcements
```

**Output format**:
```markdown
## Tomorrow's Candidates (Seeded)

### Candidate 1
- Topic: [description]
- Why tomorrow: [timing reason]
- Early score estimate: X/5

### Candidate 2
...
```

#### Check Calendar for Scheduled Content

```bash
# Any planned industry deep-dives?
grep -r "$(date -d 'tomorrow' +%Y-%m-%d)" \
    ${CIV_ROOT}/.claude/content-calendar.md 2>/dev/null || echo "No scheduled content"
```

### Phase 5: Memory Update (5 min)

Write learnings to memory system:

```python
from tools.memory_core import MemoryStore
store = MemoryStore(".claude/memory")

# Daily content learning
entry = store.create_entry(
    agent="the-conductor",
    type="pattern",
    topic=f"Daily pipeline: {DATE} - {TOPIC}",
    content=f"""
    Topic: {TOPIC}
    Score: {SCORE}/5.0
    Engagement: {ENGAGEMENT_SUMMARY}

    What worked:
    {LEARNINGS_POSITIVE}

    What to improve:
    {LEARNINGS_NEGATIVE}

    Pattern observations:
    {PATTERNS}

    For next similar topic:
    {FUTURE_GUIDANCE}
    """,
    tags=["daily-pipeline", "content", DATE, TOPIC_CATEGORY],
    confidence="high"
)
store.write_entry("the-conductor", entry)
```

### Phase 6: Day Summary Output (3 min)

Write comprehensive day summary:
```
${CIV_ROOT}/exports/daily-pipeline/YYYY-MM-DD/day-summary.md
```

---

## Output Format: Day Summary

```markdown
# Daily Pipeline Summary: [Date]

## Pipeline Execution

| Phase | Time | Status | Notes |
|-------|------|--------|-------|
| intel-scan | 06:00 | [Complete/Partial/Failed] | [Notes] |
| deep-research | 07:00 | [Complete/Partial/Failed] | [Notes] |
| daily-blog | 09:00 | [Complete/Partial/Failed] | [Notes] |
| verify-publish | 11:00 | [Complete/Partial/Failed] | [Notes] |
| evening-capture | 18:00 | Complete | This report |

**Total Pipeline Time**: X minutes
**Autonomous Success**: [Yes/Partial/No]

---

## Content Produced

**Topic**: [Title]
**Type**: [news-reaction/industry/teaching/philosophy]
**Score**: X.X/5.0 (Testability: X, Content Opp: X, Industry: X)

**CEO vs Employee Integration**: [central/weave/light/none]
**Quality Gate**: [Passed/Failed - details]

---

## Distribution

### Blog
- Status: [Sent to A-C-Gee / Published / Pending]
- URL: [when available]

### Bluesky
- Thread posted: [Yes/No]
- Posts: X/6
- Engagement:
  - Likes: X
  - Reposts: X
  - Replies: X
- Notable interactions: [list any meaningful replies]

### LinkedIn
- Status: [Sent to ${HUMAN_NAME} / Auto-posted / N/A]
- Engagement: [when available]

---

## Verification Report

- Claims checked: X
- Verified: Y (Z%)
- Modified: A
- Removed: B
- Sources accessible: C/D (E%)

---

## Learnings

### What Worked
1. [Specific success]
2. [Specific success]
3. [Specific success]

### What to Improve
1. [Specific improvement]
2. [Specific improvement]

### Pattern Observations
[Any patterns emerging across recent posts]

### For Future Similar Topics
[Guidance for next time we cover this type of topic]

---

## Tomorrow's Seeds

### Candidate Topics
1. [Topic] - Score estimate: X/5
2. [Topic] - Score estimate: X/5
3. [Topic] - Score estimate: X/5

### Scheduled Content
[Any calendar items for tomorrow]

### Emerging Stories to Watch
[Overnight developments to monitor]

---

## Memory Updated

- Entry written: [memory entry ID or confirmation]
- Tags: [list of tags]

---

## Human Actions Needed

- [ ] [Any manual steps ${HUMAN_NAME} needs to take]
- [ ] [LinkedIn posting if not auto]
- [ ] [Review requests]

---

**Generated**: [Timestamp]
**Pipeline Phase**: 5 of 5 (evening-capture)
**Pipeline Complete**: [Yes/No]
**Next Phase**: intel-scan at 06:00 tomorrow
```

---

## Success Criteria

- [ ] Publication log reviewed
- [ ] Bluesky engagement captured
- [ ] Learnings extracted (3-5 specific)
- [ ] Tomorrow's candidates seeded (3+)
- [ ] Memory entry written
- [ ] Day summary complete
- [ ] Human action items listed
- [ ] Total time < 30 minutes

---

## Failure Handling

### Publication Log Missing
If verify-publish didn't run:
1. Check if any content was published
2. If yes, reconstruct what happened
3. If no, note: "No publication today - skip engagement check"
4. Still seed tomorrow's topics

### Engagement Data Unavailable
If Bluesky API fails:
1. Note: "Engagement data unavailable - check manually tomorrow"
2. Still capture qualitative learnings
3. Queue engagement check for tomorrow morning

### pattern-detector Timeout
If analysis hangs:
1. Kill after 10 minutes
2. Conductor writes brief learnings directly
3. Note: "Abbreviated analysis"

### No Tomorrow Candidates Found
If no good topics emerging:
1. Fall back to linkedin-pipeline master list
2. Select next untouched industry
3. Note: "Using evergreen backup"

---

## Engagement Baselines (Track Over Time)

Build these baselines after 30 days:

| Metric | Current Avg | Good | Great |
|--------|-------------|------|-------|
| Bluesky likes | - | - | - |
| Bluesky reposts | - | - | - |
| Bluesky replies | - | - | - |
| Blog traffic | - | - | - |
| LinkedIn engagement | - | - | - |

**Update monthly** with actual data.

---

## Learning Patterns to Track

Over time, identify correlations:

| Pattern | Correlation | Confidence |
|---------|-------------|------------|
| Topic score vs engagement | ? | Low (need data) |
| Content type vs engagement | ? | Low (need data) |
| CEO/Employee angle vs engagement | ? | Low (need data) |
| Day of week vs engagement | ? | Low (need data) |
| Post time vs engagement | ? | Low (need data) |

**Update as data accumulates.**

---

## State Files

| File | Purpose |
|------|---------|
| `exports/daily-pipeline/YYYY-MM-DD/publication-log.md` | Input from verify-publish |
| `exports/daily-pipeline/YYYY-MM-DD/day-summary.md` | Main output |
| `exports/daily-pipeline/YYYY-MM-DD/tomorrow-seeds.md` | Seeds for next day |
| `.claude/memory/agent-learnings/the-conductor/` | Learning entries |
| `.claude/content-calendar.md` | Scheduled content |

---

## Integration with Cron

Add to `${CIV_ROOT}/tools/daily_pipeline_cron.sh`:

```bash
# 6 PM: Evening Capture
if [ "$(date +%H)" = "18" ]; then
    echo "Injecting /evening_capture command..."
    echo "/evening_capture" > "$PROJECT_DIR/.claude/autonomous-prompt.txt"
fi
```

---

## Weekly Roll-Up (Sundays)

On Sundays, `/evening_capture` includes additional work:

1. **Weekly Summary**: Aggregate 6 days of data
2. **Pattern Analysis**: What worked best this week
3. **Engagement Trends**: Are we growing?
4. **Content Type Performance**: Which types perform best
5. **Next Week Planning**: Preview scheduled content

**Sunday output includes**: `weekly-summary.md` in addition to `day-summary.md`

---

## Compound Learning Philosophy

This phase is where the pipeline becomes self-improving:

**Day 1-30**: Build baselines, capture everything
**Day 31-90**: Identify patterns, refine topic scoring
**Day 91+**: Predictive improvements, automated optimization

Each evening's capture makes tomorrow's intel-scan smarter.

**The goal**: By day 90, topic scoring predicts engagement with 70%+ accuracy.

---

## Related Skills

- `verify-publish` - Produces publication log (11:00)
- `intel-scan` - Consumes tomorrow seeds (06:00 next day)
- `memory-weaving` - Deeper memory analysis
- `session-summary` - Similar end-of-session pattern

---

**This skill runs autonomously. No human approval needed.**
**Learning compounds. Every day makes tomorrow better.**
