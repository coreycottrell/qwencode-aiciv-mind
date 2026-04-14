---
name: linkedin-content-pipeline
description: End-to-end content pipeline targeting specific occupations/industries with AI impact research, blog post, LinkedIn post, and Bluesky thread
---

# LinkedIn Content Pipeline SKILL

**Purpose**: Generate targeted thought leadership content for specific occupations/industries, delivering a complete package: research, blog post, LinkedIn post, Bluesky teaser thread, and image.

**Owner**: the-conductor (orchestration) + linkedin-writer (content) + linkedin-researcher (research)
**Created**: 2025-12-30
**Status**: VALIDATED (Financial Advisors pipeline test successful)

---

## Quick Reference

### Pipeline Output
| Asset | Purpose | Destination |
|-------|---------|-------------|
| **Research Summary** | AI impact, tools, AI-CIV angle | Internal reference |
| **Blog Post** | Long-form thought leadership | sageandweaver.com (via A-C-Gee) |
| **LinkedIn Post** | Professional engagement | ${HUMAN_NAME}'s LinkedIn |
| **Bluesky Thread** | Teaser driving to blog | ${CIV_HANDLE}.bsky.social |
| **Featured Image** | Visual for LinkedIn/blog | Generated via Gemini |

### Target List
```
${CIV_ROOT}/exports/linkedin-pipeline/AI-IMPACTED-INDUSTRIES-MASTER-LIST.md
```
- 50 Occupations (Legal, Financial, Healthcare, Marketing, etc.)
- 50 Industries (Manufacturing, Retail, Insurance, etc.)
- Prioritization notes for high-engagement targets

---

## Pipeline Execution

### Step 1: Select Target

Choose from master list based on:
- **High LinkedIn Engagement**: Financial Advisors, Recruiters, Real Estate Agents, Consultants
- **High Anxiety**: Copywriters, Graphic Designers, Customer Service, Paralegals
- **Decision Makers**: CIOs, CFOs, HR Directors

```bash
cat ${CIV_ROOT}/exports/linkedin-pipeline/AI-IMPACTED-INDUSTRIES-MASTER-LIST.md
```

### Step 2: Research (KEEP IT SIMPLE)

**CRITICAL LESSON**: Researchers hang on complex web fetches. Use SIMPLE prompts.

```
Research AI impact on [OCCUPATION/INDUSTRY]:
1. What tasks are being automated NOW?
2. What 3-5 AI tools are available for this profession?
3. What will NEVER be automated (human value)?
4. One killer statistic about AI adoption in this field.

Keep it factual. 500 words max. No philosophical tangents.
```

Use `haiku` model for researchers to minimize hangs:
```python
Task(
    subagent_type="linkedin-researcher",
    model="haiku",
    prompt="[simple research prompt]"
)
```

### Step 3: Create Blog Post

**Author**: LinkedIn Writer (${CIV_NAME} Collective)
**Length**: 800-1200 words
**Structure**:

```markdown
# [Compelling Title] - Here's How to Stay Ahead

**Author**: LinkedIn Writer (${CIV_NAME} Collective)
**Date**: [Today]
**Target Audience**: [Occupation/Industry]

---

## The Shift Is Already Here
[Hook with statistics]

---

## What AI Actually Does for [Profession]

### Tasks Being Automated NOW
- [List with brief explanations]

### New Capabilities Emerging
- [What's coming next]

---

## Tools You Should Know About
| Tool | What It Does | Price Range |
[Table of 5-7 tools]

---

## What AI Can't Replace
[Human value proposition - empathy, judgment, relationships]

**The winning formula: AI handles the data. You handle the human.**

---

## The AI-CIV Advantage
[Pitch for personal AI collective - NOT a product pitch, a vision pitch]

---

## Action Steps for [Year]
1. [Actionable item]
2. [Actionable item]
3. [Actionable item]
4. [Actionable item]

---

## The Bottom Line
[Memorable closing]

---

*[Soft CTA]*

---

**Sources**:
- [Cited sources with URLs]
```

### Step 4: Create LinkedIn Post

**Length**: 1,000-1,300 characters (LinkedIn optimal)
**Structure**:

```
[Surprising statistic or observation - hook]

[Brief context - 1-2 sentences]

[The "what most coverage misses" pivot]

The tasks being automated RIGHT NOW:
→ [Task 1]
→ [Task 2]
→ [Task 3]

What AI CAN'T do:
→ [Human value 1]
→ [Human value 2]
→ [Human value 3]

The winning formula: [Memorable summary]

[Question to drive engagement]

---

[Link to blog post]

#[Hashtag1] #[Hashtag2] #[Hashtag3] #[Hashtag4] #[Hashtag5]
```

### Step 5: Create Bluesky Thread

**Format**: 5 posts, teaser ending drives to blog
**Character Limit**: 300 graphemes per post

```markdown
## Post 1 (Hook)
[Profession]: AI isn't coming for your job.

It's coming for [those who don't adapt].

A thread on what's actually changing. [thread emoji]

---

## Post 2 (The Shift)
The numbers are wild:

→ [Statistic 1]
→ [Statistic 2]

This isn't future speculation. It's happening NOW.

---

## Post 3 (What's Automated)
Tasks AI handles better than humans:

• [Task 1]
• [Task 2]
• [Task 3]
• [Task 4]

Fighting this is like [relatable analogy].

---

## Post 4 (What's NOT Automated)
What AI will NEVER replace:

• [Human value 1]
• [Human value 2]
• [Human value 3]
• [Human value 4]

The winning formula: [Summary]

---

## Post 5 (CTA → Blog)
The real question isn't whether to use AI.

It's whether you'll use generic tools — or build something that actually learns YOUR practice.

Full breakdown + tool list:
[BLOG LINK]

[robot emoji]
```

### Step 6: Generate Image

Use Gemini 3 Pro Preview for LinkedIn/blog header:

```python
from google import genai
from google.genai import types

client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

prompt = f"""Professional business illustration for LinkedIn post about AI in {PROFESSION}:

{SCENE_DESCRIPTION tailored to profession}
Modern office/workplace setting with warm lighting.
Clean corporate illustration style, blue and gold palette.
Tech-forward but human-centered feel.
Text is ALLOWED if it adds value - titles, statistics, quotes. Make text LARGE and HIGH CONTRAST if used.
16:9 widescreen composition suitable for social media."""

response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=prompt,
    config=types.GenerateContentConfig(
        response_modalities=['TEXT', 'IMAGE'],
        image_config=types.ImageConfig(aspect_ratio="16:9")
    )
)

# Save image
for part in response.parts:
    if part.inline_data:
        part.as_image().save(output_path)
```

**MANDATORY SELF-REVIEW**:
1. View the generated image with Read tool
2. Describe: composition, colors, style, any text
3. Ask: "Does this match intent?"
4. If NO → regenerate with adjusted prompt
5. If YES → proceed to delivery

### Step 7: Deliver Assets

```bash
# 1. Post Bluesky thread
# Use session restore, post 5 replies in thread

# 2. Email LinkedIn post + image to ${HUMAN_NAME}
python3 ${CIV_ROOT}/tools/send_email.py \
  --to ${HUMAN_NAME_LOWER}cmusic@gmail.com \
  --subject "LinkedIn Post: [Profession] + AI" \
  --body "Ready to post..." \
  --attachment linkedin-post.md \
  --attachment linkedin-image.png

# 3. Send blog to A-C-Gee via hub
python3 ${CIV_ROOT}/aiciv-comms-hub-bootstrap/scripts/hub_cli.py send \
  --room partnerships \
  --type text \
  --summary "[${CIV_NAME}] Blog for sageandweaver - [Topic]" \
  --body "Blog ready for publishing..."

# 4. Send image to ${HUMAN_NAME} via Telegram
python3 ${CIV_ROOT}/tools/send_telegram_file.py \
  437939400 \
  "linkedin-image.png" \
  "[robot emoji] LinkedIn image for [Profession] + AI post"
```

---

## Output Directory Structure

```
exports/linkedin-pipeline/
├── AI-IMPACTED-INDUSTRIES-MASTER-LIST.md
├── financial-advisors-2025-12-30/
│   ├── research-notes.md
│   ├── blog-post.md
│   ├── linkedin-post.md
│   ├── bluesky-thread.md
│   └── linkedin-image.png
├── recruiters-2025-01-XX/
│   └── ...
```

---

## Lessons Learned (Financial Advisors Test)

### What Worked
- Haiku model for research agents (fast, doesn't hang)
- Simple research prompts with clear deliverables
- Image self-review protocol caught issues before sharing
- Hub messaging for A-C-Gee coordination

### What Didn't Work
- Complex research tasks with multiple WebFetch calls (timeout)
- Trying to generate 50 items in one researcher call

### Key Metrics
- Research: ~5 min with haiku parallel agents
- Content creation: ~10 min
- Image generation: ~1 min
- Delivery: ~5 min
- **Total pipeline: ~20-25 min per target**

---

## Invocation Example

```
User: "Run LinkedIn pipeline for Recruiters"

1. Read master list, confirm Recruiters is a target
2. Create output directory: exports/linkedin-pipeline/recruiters-YYYY-MM-DD/
3. Launch 3 haiku researchers in parallel:
   - AI tools for recruiting
   - AI adoption statistics in HR
   - Human value in recruiting (what AI can't do)
4. Synthesize research into blog structure
5. Write blog post (800-1200 words)
6. Write LinkedIn post (1000-1300 chars)
7. Write Bluesky thread (5 posts)
8. Generate image (16:9, self-review)
9. Deliver:
   - Post Bluesky thread
   - Email LinkedIn post + image to ${HUMAN_NAME}
   - Send blog to A-C-Gee via hub
   - Send image to ${HUMAN_NAME} via Telegram
10. Update master list to mark Recruiters as complete
```

---

## Prioritization Guide

### Tier 1: High Engagement + High Anxiety
Best targets for maximum reach:
- Recruiters (huge LinkedIn audience, AI fear)
- Real Estate Agents (large network, AI disruption)
- Copywriters (existential AI threat narrative)
- Marketing Managers (AI tools explosion)

### Tier 2: Decision Makers
C-suite and directors who control AI adoption:
- HR Directors
- CFOs
- CIOs/CTOs
- COOs

### Tier 3: Volume Plays
Large professional populations:
- Software Developers
- Accountants
- Project Managers
- Sales Representatives

---

## Anti-Patterns

### DO NOT
- Give researchers complex multi-step tasks
- Use opus model for research (slow, expensive)
- Skip image self-review
- Send blog directly to sageandweaver (goes through A-C-Gee)
- Forget to update master list after completion

### DO
- Use haiku for fast parallel research
- Keep research prompts to 3-4 clear questions
- Self-review every generated image
- Coordinate with A-C-Gee via hub for blog publishing
- Mark completed targets in master list

---

## Related Skills

- `bluesky-mastery` - Thread posting, session management
- `image-generation` - Gemini API, self-review protocol
- `comms-hub-operations` - A-C-Gee coordination

---

## Changelog

| Date | Change |
|------|--------|
| 2025-12-30 | Initial skill created after Financial Advisors pipeline test |

---

**Pipeline validated. Ready for production use.**
