---
name: verify-publish
description: |
  Verification and publishing phase for daily content pipeline.
  Fact-checks claims, publishes blog, posts Bluesky thread,
  prepares LinkedIn for ${HUMAN_NAME} (or auto-posts if enabled).
version: 1.0.0
author: capability-curator
created: 2026-01-02
status: PRODUCTION
slash_command: /verify_publish
cron_time: "0 11 * * *"

applicable_agents:
  - the-conductor
  - claim-verifier
  - bsky-manager

activation_trigger: |
  Triggered autonomously at 11 AM daily via cron/tmux injection.
  Requires blog-post.md from daily-blog phase.
  Also invokable manually via "/verify_publish" command.

required_tools:
  - Task
  - WebSearch
  - WebFetch
  - Write
  - Read
  - Bash

category: daily-pipeline
depends_on:
  - daily-blog
outputs_to:
  - evening-capture (18:00)

success_criteria:
  - blog_verified: true
  - claims_checked: true
  - blog_published: true
  - bluesky_thread_posted: true
  - linkedin_prepared: true
  - publication_logged: true
---

# Verify & Publish: Quality Assurance and Distribution

**Trigger**: `/verify_publish` or cron at 11:00 AM
**Duration**: 30-45 minutes
**Agents**: claim-verifier, bsky-manager
**Input**: Blog package from daily-blog
**Output**: Published content across platforms

---

## Purpose

This is the quality gate before publication. Every factual claim gets verified. Every source gets checked. Then we publish to blog, post to Bluesky, and prepare LinkedIn for ${HUMAN_NAME}.

**Philosophy**: Trust is hard to build and easy to destroy. One wrong fact damages credibility. Verify everything.

---

## Procedure

### Phase 1: Package Intake (2-3 min)

Read package from previous phase:
```bash
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/blog-post.md
cat ${CIV_ROOT}/exports/daily-pipeline/$(date +%Y-%m-%d)/metadata.json
```

Extract:
- Blog post content
- All factual claims
- All cited sources
- Bluesky thread content
- LinkedIn post (if present)

### Phase 2: Claim Extraction (5 min)

Identify all verifiable claims in the post:

```python
# Types of claims to verify:
# 1. Statistics ("40% of companies...")
# 2. Dates ("announced last week...")
# 3. Quotes ("CEO said...")
# 4. Comparisons ("faster than X...")
# 5. Tool claims ("supports Y feature...")
```

Create claims manifest:
```markdown
## Claims to Verify

1. [Claim]: "[exact quote from post]"
   - Source cited: [URL]
   - Type: statistic/date/quote/comparison/feature

2. [Claim]: "[exact quote from post]"
   ...
```

### Phase 3: Verification (15-20 min)

Invoke claim-verifier for fact-checking:

```python
Task(
    subagent_type="claim-verifier",
    model="sonnet",
    prompt=f"""Verify these factual claims:

{CLAIMS_MANIFEST}

For each claim:
1. Visit the cited source URL
2. Confirm the claim matches the source
3. Check if information is current (not outdated)
4. Note any discrepancies

Output format for each claim:
- Claim: [original]
- Source: [URL]
- Verdict: VERIFIED / MODIFIED / UNVERIFIED / FALSE
- Notes: [explanation]
- Suggested fix: [if needed]
"""
)
```

#### Verification Outcomes

| Verdict | Action |
|---------|--------|
| **VERIFIED** | Proceed with original claim |
| **MODIFIED** | Update claim with correct info |
| **UNVERIFIED** | Remove or soften claim |
| **FALSE** | Remove claim entirely |

### Phase 4: Source Validation (5 min)

Check all source URLs:

```python
for source in sources:
    try:
        response = WebFetch(url=source['url'], prompt="Is this page accessible?")
        source['status'] = 'accessible'
    except:
        source['status'] = 'broken'
        # Find alternative source or remove citation
```

**Broken source handling**:
1. Search for alternative source with same info
2. If found, update citation
3. If not found, remove specific claim

### Phase 5: Apply Corrections (5 min)

If any claims need modification:
1. Update blog-post.md with corrections
2. Update bluesky-thread.md if affected
3. Update linkedin-post.md if affected
4. Log all changes in verification-log.md

### Phase 6: Publication (15-20 min)

#### 6a: Publish Blog

**USE sageandweaver-blog SKILL** - ${CIV_NAME} publishes its own blog via Netlify:

```bash
# 1. Generate HTML from markdown
python3 ${ACG_ROOT}/sageandweaver-network/tools/create_blog_post.py \
    --civilization weaver \
    --title "Your Title" \
    --date "$(date +'%B %d, %Y')" \
    --slug "$(date +%Y-%m-%d)-your-slug" \
    --content /path/to/blog-post.md

# 2. Copy header image
cp exports/blog-header.png \
   ${ACG_ROOT}/sageandweaver-network/weaver-blog/images/

# 3. Update posts.json (add entry at TOP)

# 4. Deploy to Netlify
cd ${ACG_ROOT}/sageandweaver-network && \
netlify deploy --prod --site 7e89a1b0-172a-4d48-b191-c7d9dcc452f2

# 5. VERIFY (MUST return 200)
curl -s -o /dev/null -w "%{http_code}" \
    "https://sageandweaver-network.netlify.app/weaver-blog/posts/YOUR-SLUG.html"
```

**DO NOT send to hub. ${CIV_NAME} publishes its OWN blog.**

#### 6b: Post Bluesky Thread

Invoke bsky-manager with safety constraints:

```python
Task(
    subagent_type="bsky-manager",
    model="sonnet",
    prompt=f"""Post this thread to Bluesky:

{BLUESKY_THREAD_CONTENT}

## Safety Requirements (CONSTITUTIONAL):
- Wait 30+ seconds between posts
- Total posts: 5-6 max
- Include blog URL in final post
- Random variance in delays

## Thread Format:
- Post 1: Hook (no link)
- Posts 2-4: Key insights
- Post 5: Gap/FOMO
- Post 6: Link + signature

## Verification:
After posting, confirm each post was successful.
Return thread URL and all post URIs.
"""
)
```

**CRITICAL**: bsky-manager follows bsky-safety protocol. Never rush posts.

#### 6c: Prepare LinkedIn

For LinkedIn posts, two options:

**Option A: Manual (Default)**
```bash
# Email to ${HUMAN_NAME} for manual posting
python3 ${CIV_ROOT}/tools/send_email.py \
    --to ${HUMAN_NAME_LOWER}cmusic@gmail.com \
    --subject "LinkedIn Post Ready: [Title]" \
    --body "Ready for your review and posting..." \
    --attachment linkedin-post.md
```

**Option B: Auto-post (If Enabled)**
```bash
# Only if LINKEDIN_AUTO_POST=true in .env
# Uses linkedin-api or similar integration
# NOT YET IMPLEMENTED - requires ${HUMAN_NAME} approval
```

#### 6d: Send Image to ${HUMAN_NAME}

If image generation is part of pipeline:
```bash
python3 ${CIV_ROOT}/tools/send_telegram_file.sh \
    437939400 \
    "exports/daily-pipeline/$(date +%Y-%m-%d)/blog-header.png" \
    "Blog header for: [Title]"
```

### Phase 7: Publication Log (5 min)

Write publication record:
```
${CIV_ROOT}/exports/daily-pipeline/YYYY-MM-DD/publication-log.md
```

Update registries:
- `.claude/registries/blog-post-registry.md`
- `.claude/bsky_responded.txt` (add thread URIs)

---

## Output Format: Publication Log

```markdown
# Publication Log: [Date]

## Verification Summary

| Claim | Verdict | Action |
|-------|---------|--------|
| [Claim 1] | VERIFIED | None |
| [Claim 2] | MODIFIED | Updated to [new text] |
| [Claim 3] | VERIFIED | None |

**Claims checked**: X
**Verified**: Y
**Modified**: Z
**Removed**: W

## Source Validation

| Source | Status |
|--------|--------|
| [URL 1] | accessible |
| [URL 2] | accessible |
| [URL 3] | broken -> replaced |

## Publication Status

### Blog
- **Published to**: sageandweaver-network.netlify.app (direct Netlify deploy)
- **Status**: [Pending/Published]
- **URL**: https://sageandweaver-network.netlify.app/weaver-blog/posts/[SLUG].html

### Bluesky
- **Thread posted**: [Yes/No]
- **Posts**: X/6
- **Thread URL**: [URL]
- **Post URIs**:
  - [URI 1]
  - [URI 2]
  - ...

### LinkedIn
- **Status**: [Sent to ${HUMAN_NAME} / Auto-posted]
- **Email sent**: [timestamp]

## Corrections Applied

1. [Change 1 description]
2. [Change 2 description]
...

## Quality Metrics

- **Verification pass rate**: X%
- **Source availability**: Y%
- **Publication success**: Z/3 platforms

---

**Generated**: [Timestamp]
**Pipeline Phase**: 4 of 5 (verify-publish)
**Next Phase**: evening-capture at 18:00
```

---

## Success Criteria

- [ ] All claims extracted from blog
- [ ] claim-verifier checked each claim
- [ ] No FALSE claims remain in published content
- [ ] All source URLs verified accessible
- [ ] Blog published to Netlify (verified 200)
- [ ] Bluesky thread posted (5-6 posts)
- [ ] LinkedIn post sent to ${HUMAN_NAME} (or auto-posted)
- [ ] Publication log complete
- [ ] Blog registry updated
- [ ] Total time < 45 minutes

---

## Failure Handling

### Blog Package Missing
If daily-blog didn't run:
1. Check if blog-post.md exists
2. If not, run `/daily_blog` first
3. If that fails, skip today's publication
4. Note: "Publication skipped - no content"

### Verification Finds Major Issues
If > 30% of claims fail verification:
1. HALT publication
2. Send alert to ${HUMAN_NAME}
3. Mark content for rewrite
4. Note: "Publication held - verification failed"

### Bluesky Rate Limited
If bsky-manager reports rate limiting:
1. Pause thread posting
2. Wait 24 hours
3. Queue for tomorrow's first action
4. Still publish blog + LinkedIn

### Netlify Deploy Fails
If netlify deploy fails:
1. Try API deploy: `python3 ${CIV_ROOT}/tools/netlify_api_deploy.py`
2. If still fails, check netlify token in ~/.config/netlify/config.json
3. Email blog to ${HUMAN_NAME} as backup
4. Note: "Netlify delivery deferred"

### claim-verifier Timeout
If verifier hangs > 15 minutes:
1. Kill task
2. Conductor does quick manual check (top 3 claims only)
3. Proceed with caution flag
4. Note: "Abbreviated verification"

---

## Verification Strictness Levels

| Content Type | Level | Actions |
|--------------|-------|---------|
| **Statistics** | STRICT | Must match source exactly |
| **Dates** | STRICT | Must be accurate to day |
| **Quotes** | STRICT | Must be verbatim or marked paraphrased |
| **Comparisons** | MEDIUM | Directionally correct is OK |
| **General claims** | MEDIUM | Reasonable interpretation OK |
| **Opinions** | LIGHT | Just flag if controversial |

---

## Platform-Specific Constraints

### Bluesky
- 300 grapheme limit per post
- Rate limits from bsky-safety skill
- 30+ seconds between posts
- Maximum 5-6 posts per thread

### LinkedIn
- 3000 character limit for posts
- Hashtags: 3-5 optimal
- Link in comments performs better than in post
- Best posting times: 7-8 AM, 12 PM, 5-6 PM (user's timezone)

### Blog (sageandweaver-network.netlify.app)
- HTML format (use create_blog_post.py)
- Header image in weaver-blog/images/
- Update posts.json for index page
- Published by ${CIV_NAME} directly via Netlify

---

## State Files

| File | Purpose |
|------|---------|
| `exports/daily-pipeline/YYYY-MM-DD/blog-post.md` | Input from daily-blog |
| `exports/daily-pipeline/YYYY-MM-DD/verification-log.md` | Verification results |
| `exports/daily-pipeline/YYYY-MM-DD/publication-log.md` | Publication record |
| `.claude/registries/blog-post-registry.md` | All posts tracking |
| `.claude/bsky_responded.txt` | Posted thread URIs |

---

## Integration with Cron

Add to `${CIV_ROOT}/tools/daily_pipeline_cron.sh`:

```bash
# 11 AM: Verify & Publish
if [ "$(date +%H)" = "11" ]; then
    # Check that daily-blog produced output
    if [ -f "$PIPELINE_DIR/$(date +%Y-%m-%d)/blog-post.md" ]; then
        echo "Injecting /verify_publish command..."
        echo "/verify_publish" > "$PROJECT_DIR/.claude/autonomous-prompt.txt"
    else
        echo "WARNING: blog-post.md not found. Cannot publish."
        echo "No content to publish today." > "$PROJECT_DIR/.claude/autonomous-prompt.txt"
    fi
fi
```

---

## Related Skills

- `daily-blog` - Produces blog package (09:00)
- `evening-capture` - Captures engagement data (18:00)
- `bsky-safety` - Rate limiting and ban prevention
- `claim-verifier` (agent) - Fact checking
- `bluesky-blog-thread` - Thread posting pattern

---

**This skill runs autonomously. No human approval needed.**
**Publication requires verification. No exceptions.**
