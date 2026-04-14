# LinkedIn Thought Leadership Pipeline

**Created**: 2025-12-29
**Status**: DESIGNED (awaiting validation)
**Purpose**: Transform research into verified, ready-to-publish LinkedIn content

---

## Executive Summary

A 3-agent pipeline that does the heavy lifting for LinkedIn thought leadership, leaving ${HUMAN_NAME} with only the "monkey work" of posting.

**Time Investment**:
- ${HUMAN_NAME}: ~20 minutes/week for 2-3 posts
- Agents: ~60-90 minutes/week automated research, writing, verification

**The Philosophy**: Great thought leadership requires great research, great writing, AND great verification. No single agent should do all three - specialization creates quality.

---

## The Pipeline

```
                    [Research Request]
                          |
                          v
            +--------------------------+
            |    linkedin-researcher   |
            |  (Domain Deep Research)  |
            |                          |
            | - Find statistics        |
            | - Source case studies    |
            | - Identify counter-      |
            |   narratives             |
            | - Compile research brief |
            +--------------------------+
                          |
                          | Research Brief
                          v
            +--------------------------+
            |     linkedin-writer      |
            |  (Content Creation)      |
            |                          |
            | - Apply "Director vs     |
            |   User" framework        |
            | - Use optimal hook       |
            | - Mark claims [CLAIM:N]  |
            | - Create draft post      |
            +--------------------------+
                          |
                          | Draft Post + Claim Index
                          v
            +--------------------------+
            |     claim-verifier       |
            |  (Adversarial Review)    |
            |                          |
            | - Independent verify     |
            | - NO access to           |
            |   researcher's notes     |
            | - 5-level confidence     |
            | - Issue verdict          |
            +--------------------------+
                          |
                          |
          +---------------+---------------+
          |               |               |
          v               v               v
      [GREEN]         [YELLOW]         [RED]
   Ready to Post    Revise + Re-verify  Re-research
          |               |               |
          v               |               |
    +----------+          |               |
    |  COREY   |<---------+               |
    | Reviews  |                          |
    | & Posts  |                          |
    +----------+                          |
                                          v
                              Return to linkedin-researcher
```

---

## Detailed Phase Breakdown

### Phase 1: Research (linkedin-researcher)

**Trigger**:
- Weekly content calendar need
- Specific topic request from marketing-strategist
- News-driven opportunity

**Input**:
- Domain/industry focus
- Specific angle (optional)
- Recent posts to avoid repetition

**Process**:
1. Check memory for past research on this domain
2. Search authoritative sources (academic, government, established firms)
3. Find 3-5 statistics with citations
4. Locate relevant quotes from industry figures
5. Identify 1-2 case studies with quantified results
6. Discover counter-narrative angles
7. Compile structured research brief

**Output**: Research Brief (see linkedin-researcher manifest for template)

**Time**: ~15-20 minutes per topic

**Quality Gate**:
- 70%+ sources from HIGH authority
- All statistics have verifiable sources
- At least one counter-narrative angle

---

### Phase 2: Writing (linkedin-writer)

**Trigger**: Research brief available from linkedin-researcher

**Input**:
- Research brief with statistics, quotes, case studies
- Hook type preference (optional)
- Specific angle to take (optional)

**Process**:
1. Check memory for past posts (avoid repetition)
2. Select optimal hook formula based on content
3. Frame through "Director vs User" lens
4. Write 1,200-1,800 character post
5. Mark ALL factual claims with [CLAIM:N]
6. Create claim index with verification guidance
7. Add optional Sage & Weaver tie-in (removable)

**Output**: Draft Post with Claim Index (see linkedin-writer manifest for template)

**Time**: ~10-15 minutes per post

**Quality Gate**:
- Hook under 235 characters
- All claims marked
- Mobile-readable formatting
- "Director" framework present

---

### Phase 3: Verification (claim-verifier)

**Trigger**: Draft post ready with [CLAIM:N] markers

**Input**:
- Draft post ONLY (not research brief)
- Claim index

**CRITICAL**: claim-verifier does NOT receive linkedin-researcher's sources. They verify INDEPENDENTLY.

**Process**:
1. Extract all [CLAIM:N] markers
2. For each claim:
   - Search for independent verification
   - Find different source than researcher used
   - Assess confidence level
   - Determine recommendation (KEEP/HEDGE/CUT/REWRITE)
3. Calculate overall verdict (GREEN/YELLOW/RED)
4. Compile verification report

**Output**: Verification Report with overall verdict

**Time**: ~15-20 minutes per post

**Quality Gate**:
- Every claim has independent verification attempt
- Confidence levels assigned accurately
- Clear revision guidance if YELLOW

---

### Phase 4: Resolution

**If GREEN**:
- Post ready for ${HUMAN_NAME}
- ${HUMAN_NAME} reviews (2-5 minutes)
- ${HUMAN_NAME} posts to LinkedIn
- Done

**If YELLOW**:
- linkedin-writer receives revision guidance
- Specific claims revised
- Re-submit to claim-verifier
- Loop until GREEN

**If RED**:
- Either:
  - Return to linkedin-researcher for new research
  - Abandon topic (move to different angle)
- Never force bad content through

---

## Invocation Pattern

### Standard Weekly Execution

**Step 1: Request research on target domains**

```xml
<invoke name="Task">
<parameter name="subagent_type">linkedin-researcher</parameter>
<parameter name="description">Research AI in healthcare for LinkedIn thought leadership</parameter>
<parameter name="prompt">
You are linkedin-researcher. Research AI in healthcare for a LinkedIn thought leadership post.

Focus: [Specific angle or recent news]

Find:
- 3-5 statistics from authoritative sources
- 1-2 notable quotes from industry figures
- 1-2 case studies with quantified results
- Counter-narrative angles

Output: Complete research brief following your template.

Remember: Read CLAUDE-CORE.md Books I-II on activation.
</parameter>
</invoke>
```

**Step 2: Create draft post from research**

```xml
<invoke name="Task">
<parameter name="subagent_type">linkedin-writer</parameter>
<parameter name="description">Create LinkedIn post from healthcare AI research</parameter>
<parameter name="prompt">
You are linkedin-writer. Create a LinkedIn thought leadership post from this research brief.

[Insert research brief]

Requirements:
- Frame through "Director vs User" lens
- Use optimal hook formula
- 1,200-1,800 characters
- Mark ALL claims with [CLAIM:N]
- Include claim index

Output: Draft post with claim index following your template.

Remember: Read CLAUDE-CORE.md Books I-II on activation.
</parameter>
</invoke>
```

**Step 3: Verify claims independently**

```xml
<invoke name="Task">
<parameter name="subagent_type">claim-verifier</parameter>
<parameter name="description">Verify claims in LinkedIn post</parameter>
<parameter name="prompt">
You are claim-verifier. Verify all claims in this draft post.

[Insert draft post ONLY - NOT research brief]

[Insert claim index]

CRITICAL: Verify INDEPENDENTLY. Do NOT use the researcher's sources. Find different sources confirming the same claims.

Output: Verification report with GREEN/YELLOW/RED verdict.

Remember: Read CLAUDE-CORE.md Books I-II on activation.
</parameter>
</invoke>
```

---

## Integration with Marketing Strategy

### Content Calendar Alignment

marketing-strategist defines weekly content themes:

| Week | Focus | linkedin-researcher Target | Hook Type |
|------|-------|---------------------------|-----------|
| 1 | Healthcare AI | Diagnostic AI statistics | Big Outcome |
| 2 | Legal AI | Contract review automation | Contrarian |
| 3 | Enterprise AI adoption | Fortune 500 case studies | List |
| 4 | AI strategy | Common mistakes | Pain |

linkedin-researcher executes research based on calendar.
linkedin-writer aligns hook type with strategic direction.

### Evergreen vs Timely Content

**Evergreen** (can be prepared in advance):
- "Director vs User" framework posts
- Industry-specific AI adoption patterns
- Common mistake posts

**Timely** (news-driven):
- New AI model releases
- Regulatory announcements
- Major vendor news

Pipeline handles both - timely just requires faster turnaround.

---

## Quality Metrics

### Per-Post Metrics

| Stage | Metric | Target |
|-------|--------|--------|
| Research | Source authority | 70%+ HIGH |
| Research | Data freshness | 80%+ < 18 months |
| Writing | Character count | 1,200-1,800 |
| Writing | Claims marked | 100% |
| Verification | Independent verification | 100% attempted |
| Verification | GREEN rate | 70%+ first pass |

### Pipeline Health Metrics

| Metric | Target | Alert If |
|--------|--------|----------|
| Posts per week | 2-3 | < 2 |
| Research-to-publish time | < 48 hours | > 72 hours |
| YELLOW revision cycles | < 2 | > 2 |
| RED rate | < 10% | > 20% |
| Post-publication corrections | 0 | Any |

---

## Failure Modes & Recovery

### Research Gaps

**Symptom**: linkedin-researcher can't find authoritative sources
**Cause**: Topic too niche or too new
**Recovery**:
- Shift to adjacent topic with better sources
- Wait for more research to emerge
- Use qualitative framing ("early adopters report...")

### Verification Failures

**Symptom**: claim-verifier returns RED
**Cause**: Claims overstate source, outdated data, or fabricated
**Recovery**:
- Return to linkedin-researcher for new research
- linkedin-writer creates alternative angle avoiding problematic claims
- If pattern persists, review research methodology

### Voice Drift

**Symptom**: linkedin-writer output doesn't sound like ${HUMAN_NAME}
**Cause**: Template becoming generic
**Recovery**:
- Review past successful posts
- Add personal anecdotes from ${HUMAN_NAME}
- Refresh voice examples in linkedin-writer memory

---

## Integration with Other Flows

### Parallel Research

For comprehensive topic coverage, invoke multiple linkedin-researcher tasks in parallel:

```
linkedin-researcher (Healthcare) ─┐
linkedin-researcher (Legal)       ├─> result-synthesizer ─> Theme Report
linkedin-researcher (Finance)     │
linkedin-researcher (Enterprise)  ─┘
```

### Marketing Strategist Coordination

Monthly content planning:

```
marketing-strategist ─> Content Calendar ─> linkedin-researcher (weekly)
                                        ─> linkedin-writer (weekly)
                                        ─> claim-verifier (per post)
```

---

## Sage & Weaver Connection

### How Posts Drive Business

**Awareness Path**:
```
LinkedIn Post (education, value)
    ↓
Profile visit / Follow
    ↓
Lead magnet (5 Power Prompts)
    ↓
Director's Brief ($10/mo)
    ↓
Your Sage ($30/mo)
```

**Direct Path** (for workshop posts):
```
LinkedIn Post (case study, results)
    ↓
DM or comment
    ↓
Workshop conversation ($200-3000)
```

### Subtle Tie-In Examples

**Good** (natural connection):
> "This is why I built Your Sage - because generic AI can't do this."
> "The 'Director vs User' gap I see in my work with teams..."

**Bad** (forced promotion):
> "For more insights, check out sageandweaver.com!"
> "This is available in our premium newsletter..."

**Removable Test**: If tie-in removed, post should still be complete and valuable.

---

## Validation Plan

### Test 1: Single Post Flow

1. linkedin-researcher: Healthcare AI research
2. linkedin-writer: Draft from research
3. claim-verifier: Verification report
4. Measure: Time, quality, verdict

### Test 2: Revision Cycle

1. Intentionally use weak source
2. claim-verifier catches (should)
3. linkedin-writer revises
4. claim-verifier re-verifies
5. Measure: Cycle count, final quality

### Test 3: Full Week

1. 3 posts through pipeline
2. Different industries/angles
3. Measure: Total time, consistency, ${HUMAN_NAME} effort

**Success Criteria**:
- ${HUMAN_NAME} spends < 30 minutes for 3 posts
- Zero post-publication corrections
- All posts reflect "Director" positioning

---

## Revision History

| Date | Change | Author |
|------|--------|--------|
| 2025-12-29 | Initial creation | agent-architect |

---

**This pipeline transforms thought leadership from a time sink into a system.**

**Research discovers. Writing transforms. Verification protects. ${HUMAN_NAME} posts.**

---

**END linkedin-thought-leadership-pipeline.md**
