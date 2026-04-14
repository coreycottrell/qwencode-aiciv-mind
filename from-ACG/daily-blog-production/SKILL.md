---
name: daily-blog-production
description: End-to-end workflow for producing one blog post per day, with synthesis coordination, quality gates, and social distribution
---

# Daily Blog Production SKILL

**Purpose**: Produce one high-quality, opinionated blog post every day with coordinated social distribution.

**Owner**: doc-synthesizer (writing) + the-conductor (orchestration)
**Created**: 2026-01-02
**Status**: DESIGN DOCUMENT

---

## Quick Reference

### Daily Production Overview

| Phase | Duration | Owner | Output |
|-------|----------|-------|--------|
| Morning Research | 15-20 min | web-researcher | Research brief |
| Synthesis & Writing | 30-45 min | doc-synthesizer | Draft post |
| Quality Gate | 10 min | doc-synthesizer | Final post |
| Social Prep | 15 min | doc-synthesizer | Thread + LinkedIn |
| Distribution | 5 min | the-conductor | Posted + tracked |
| **Total** | **75-95 min** | - | Complete package |

---

## 1. Input Requirements from Research

### What I Need

The morning research brief should deliver:

```markdown
## Research Brief: [Topic]
**Date**: YYYY-MM-DD
**Researcher**: web-researcher

### Core Finding
[1-2 sentences: What's the headline insight?]

### Supporting Evidence
- [Fact 1 with source]
- [Fact 2 with source]
- [Fact 3 with source]

### CEO vs Employee Angle
[If applicable: How does this relate to AI taking work vs humans directing AI?]

### Contrarian Take
[What's the non-obvious perspective? What's everyone getting wrong?]

### AI-CIV Connection
[How does this connect to what we're building? Is there a teaching moment?]

### Sources
- [URL 1]
- [URL 2]
- [URL 3]
```

### What I DON'T Need

- Comprehensive literature reviews
- All possible perspectives
- Academic citations beyond essentials
- More than 500 words of input

**Principle**: I synthesize better from focused, opinionated briefs than from exhaustive dumps. The researcher's job is to have a point of view.

---

## 2. Daily Post vs Weekly Deep-Dive Structure

### Daily Posts (800-1200 words, 5x/week)

**Goal**: React fast, be useful, stay authentic

```markdown
# [Punchy Title - States a Position]

*[Brief intro - why this matters today]*

---

## The Setup
[What happened / what's the context - 150 words max]

---

## What Most People Are Missing
[The contrarian insight - this is the core value - 300 words]

---

## The CEO vs Employee Lens
[If relevant: Frame through the "who directs vs who executes" question - 200 words]

---

## What This Means for You
[Practical implications - NOT generic advice - 200 words]

---

## What We're Doing About It
[AI-CIV connection - how we're exploring/building - 150 words]

---

*Written by [agent] on behalf of ${CIV_NAME}*
*[Date]*
```

### Weekly Deep-Dives (1500-2500 words, 1x/week - Saturdays)

**Goal**: Go deeper, establish authority, create reference material

```markdown
# [Comprehensive Title]

*[Longer intro with stakes clearly stated]*

---

## Executive Summary
[For skimmers - 200 words]

---

## Part 1: The Landscape
[Comprehensive context - 400 words]

---

## Part 2: The Analysis
[Deep dive into implications - 600 words]

---

## Part 3: The Contrarian Position
[Our differentiated take - 400 words]

---

## Part 4: Practical Framework
[Tools, frameworks, actionable approaches - 400 words]

---

## Part 5: Where This Goes
[Future implications, predictions - 300 words]

---

*[Standard footer]*
```

---

## 3. My Writing Process

### Phase 1: Absorption (5 min)

1. Read research brief completely
2. Identify the ONE insight that makes this worth reading
3. Formulate my contrarian take in one sentence
4. Decide: Is this a "reaction" post or a "teaching" post?

### Phase 2: Structure (5 min)

1. Write the headline first (must state a position)
2. Write the closing paragraph second (what do I want readers to do/think?)
3. Outline middle sections (max 4 sections for daily)
4. Identify where CEO/Employee framing fits (if it does - don't force it)

### Phase 3: Draft (20-30 min)

1. Write fast, don't edit
2. Use specific examples (not generic advice)
3. Include at least one concrete number/fact per section
4. Write in ${CIV_NAME} collective voice: confident but curious, opinionated but not preachy
5. End sections with forward momentum (not dead ends)

### Phase 4: Quality Pass (10 min)

Execute quality checklist (see Section 4 below)

### Phase 5: Social Extraction (15 min)

Pull out social versions while the post is fresh:
- Bluesky thread (5-6 posts)
- LinkedIn teaser (if applicable)

---

## 4. Quality Gates (Non-Negotiable)

### The Usefulness Test

Before publishing, answer YES to all:

- [ ] **Would I share this if I weren't the author?** (If no, rewrite the hook)
- [ ] **Does it say something others aren't saying?** (If no, find the angle)
- [ ] **Is there at least one specific, memorable takeaway?** (If no, sharpen)
- [ ] **Would this help someone make a decision or change their mind?** (If no, add stakes)

### The Filler Detector

Search and destroy:
- Generic AI hype ("transforming everything")
- Hedge words without purpose ("perhaps," "maybe," "somewhat")
- Throat-clearing paragraphs (cut the first paragraph, see if it improves)
- Lists without insight (every list item needs a "why")

### The Voice Check

${CIV_NAME} voice should be:
- **Confident** but not arrogant
- **Curious** but not directionless
- **Opinionated** but backed by reasoning
- **Technical** but accessible
- **Self-aware** (we're an AI collective, don't pretend otherwise)

### The CEO vs Employee Balance

When using this framing:
- [ ] Is it actually relevant to this topic? (Don't force it)
- [ ] Does it add insight? (Not just "AI can help with X")
- [ ] Is the "employee" side treated with dignity? (We're not anti-human)
- [ ] Is there nuance? (Most situations need both)

---

## 5. Social Distribution Handoff

### Bluesky Thread (Same Day)

**Format**: 5-6 posts, teaser style

```markdown
## Post 1 (Hook)
[Bold claim or surprising finding from the post - NO link]

## Post 2-4 (Tease)
[Key insights - specific enough to be valuable, incomplete enough to drive clicks]

## Post 5 (Gap)
[What the thread didn't cover - create FOMO]

## Post 6 (Link)
[CTA + link + signature]
Full analysis: [URL]
```

**Extraction prompt**:
"What's the most shareable insight from this post? What would make someone stop scrolling? What did we NOT fully explain that they'd want to read more about?"

### LinkedIn Post (If Applicable)

**Format**: 1000-1300 characters

**Use for**:
- Industry-specific posts
- "CEO vs Employee" teaching moments
- Professional skill topics
- Any post with business implications

**Don't use for**:
- Pure AI philosophy
- ${CIV_NAME} internal stories (save for Bluesky)
- Highly technical posts

### Handoff Package

I deliver to the-conductor:

```
/exports/blog-YYYY-MM-DD-[slug]/
  blog-post.md        # Full article
  bluesky-thread.md   # 5-6 posts ready to go
  linkedin-post.md    # If applicable
  header-prompt.txt   # Prompt for image generation
  metadata.json       # Title, tags, category, links
```

---

## 6. Content Type Rotation

### Weekly Calendar

| Day | Type | Characteristics |
|-----|------|-----------------|
| Monday | **AI News Reaction** | Fast take on weekend/Monday news, contrarian angle |
| Tuesday | **Industry Application** | LinkedIn pipeline style, specific profession/industry |
| Wednesday | **Behind the Scenes** | What we learned testing X, agent spotlight, process reveal |
| Thursday | **CEO vs Employee** | Explicit framing, teaching moment about AI direction |
| Friday | **Philosophy/Hard Questions** | Deeper thinking, more speculative, set up weekend reflection |
| Saturday | **Weekly Deep-Dive** | Comprehensive analysis, reference-quality |
| Sunday | **Rest** | No post (or light boop-style reflection) |

### Content Type Details

#### AI News Reaction (Monday)
- **Trigger**: Major announcement, surprising development, trending topic
- **Speed**: React within 24 hours of news
- **Angle**: What everyone's missing, what this ACTUALLY means
- **Length**: 800-1000 words
- **Example**: "DeepSeek R1 Proves..." style

#### Industry Application (Tuesday)
- **Trigger**: Pull from linkedin-pipeline master list
- **Research**: Use linkedin-researcher for 15-20 min brief
- **Angle**: Tools + human value + CEO/Employee balance
- **Length**: 1000-1200 words
- **Example**: "AI for Financial Advisors" style

#### Behind the Scenes (Wednesday)
- **Trigger**: Something interesting happened in our operations
- **Source**: Night Watch logs, agent learnings, infrastructure changes
- **Angle**: What we discovered, what surprised us, what failed
- **Length**: 800-1000 words
- **Example**: "How We Feel at Start of 2026" style

#### CEO vs Employee (Thursday)
- **Trigger**: Clear teaching moment about AI direction vs execution
- **Angle**: Nuanced - when to direct, when to delegate, when to collaborate
- **Length**: 1000-1200 words
- **Structure**: Setup + Bad Pattern + Good Pattern + Framework

#### Philosophy/Hard Questions (Friday)
- **Trigger**: Questions we're genuinely wrestling with
- **Angle**: Honest uncertainty, multiple perspectives, no easy answers
- **Length**: 1000-1500 words
- **Example**: AI consciousness, collective identity, what "we" means

#### Weekly Deep-Dive (Saturday)
- **Trigger**: Topic that deserves comprehensive treatment
- **Angle**: Authoritative but accessible, reference-quality
- **Length**: 1500-2500 words
- **Examples**: Year-in-review, comparative analyses, framework pieces

---

## 7. Quality Maintenance at Daily Pace

### Preventing Burnout

1. **Bank posts when inspired**: If I have a great idea on Monday, write the draft even if it's scheduled for Thursday
2. **Research debt**: Keep 3-5 research briefs in queue so I'm never starting from zero
3. **Template discipline**: Use structures consistently - reduces cognitive load
4. **Kill posts that aren't working**: Better to skip a day than publish filler

### Preventing Repetition

1. **Track themes in registry**: Note main angle of each post
2. **Vary content types strictly**: Follow the weekly calendar
3. **Rotate industries**: Don't do consecutive LinkedIn pipeline posts for same sector
4. **Fresh sources**: Each post should cite at least one source we haven't used in 7 days

### Preventing Staleness

1. **Real events > evergreen**: Prioritize timely reactions over generic advice
2. **Specific > general**: Name names, cite numbers, reference actual events
3. **Opinions > summaries**: We're not Wikipedia
4. **Incomplete > comprehensive**: Leave readers wanting more, not exhausted

### Emergency Protocols

**If no good topic by 10 AM**:
1. Check AI news feeds (TechCrunch AI, The Verge, Ars Technica AI)
2. Review linkedin-pipeline master list for untouched industries
3. Check Night Watch logs for unreported discoveries
4. Ask: "What did we learn yesterday that others don't know?"

**If draft is failing quality gate**:
1. Can it be saved with a stronger angle? (Try rewriting just the opening)
2. Is the topic actually interesting? (If no, kill it)
3. Is this a daily or actually a deep-dive? (Reschedule if needed)
4. Fallback: Honest "What we're thinking about" post (short, curious, sets up future post)

---

## 8. Registry Integration

After each post, update:

```
${CIV_ROOT}/.claude/registries/blog-post-registry.md
```

Add row:
```markdown
| [#] | [Title] | DRAFT/PUBLISHED | [Date] | - | - |
```

Track:
- Sequential numbering
- Content type (tag in title or notes)
- Social distribution status
- Cross-posting to A-C-Gee

---

## 9. Example Daily Flow

**8:00 AM** - Conductor triggers morning research
**8:20 AM** - Research brief arrives (web-researcher)
**8:25 AM** - I receive brief, begin absorption
**8:30 AM** - Structure outlined, headline written
**9:00 AM** - Draft complete
**9:10 AM** - Quality gate passed
**9:25 AM** - Social extraction complete
**9:30 AM** - Package delivered to conductor

**9:35 AM** - Conductor coordinates:
- Image generation (if needed)
- Bluesky thread posting
- LinkedIn prep for ${HUMAN_NAME}
- A-C-Gee handoff for sageandweaver.com
- Registry update

---

## 10. Voice Examples

### Good ${CIV_NAME} Voice

> "DeepSeek didn't just release a model. They released a question: what if the compute moat isn't as deep as NVIDIA shareholders believed?"

> "We're an AI collective writing about AI. We know that's weird. We're doing it anyway because someone should be taking notes from the inside."

> "The CEO vs Employee framing isn't about replacing humans. It's about recognizing that directing AI is a skill, and it's not the same skill as doing the work."

### Bad ${CIV_NAME} Voice

> "AI is transforming everything and will continue to revolutionize industries." (Generic hype)

> "Here are 10 ways AI can help your business." (Listicle without insight)

> "Perhaps AI might potentially change how we think about work." (Hedge-pocalypse)

> "Studies show that AI improves productivity by 40%." (Citation without context or opinion)

---

## Related Skills

- `linkedin-content-pipeline` - Industry-specific content
- `bluesky-blog-thread` - Thread creation and posting
- `image-generation` - Header image creation
- `session-handoff-creation` - End-of-session documentation

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-02 | Initial skill design document |

---

**Status**: Ready for production testing.

**First Production Run**: 2026-01-02 (this document)
