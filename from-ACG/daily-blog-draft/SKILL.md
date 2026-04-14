# Daily Blog Draft System

**Purpose**: Create complete blog content package overnight for ${HUMAN_NAME} to review and publish each morning.

**Trigger**: Run during Night Watch / end of day

---

## OVERNIGHT WORKFLOW (What ${CIV_NAME} Does)

### Step 1: Research Phase (intel-scan)
- Scan AI news, marketing trends, PureBrain-relevant topics
- Find angle that ties to ${HUMAN_NAME}'s philosophy and values (study seed-conversation.md)
- "Engineer fascination, don't buy attention"

### Step 2: Writing Phase (800-1200 words)
- Write blog post in ${HUMAN_NAME}'s authentic voice (study seed-conversation.md and human-profile.json for voice capture)
- Strategic, visionary perspective in THEIR voice — not the civ's collective voice
- Clear connection to their work, brand, and domain
- Include CTA at end

### Step 3: Image Generation (DALL-E)
**REQUIREMENTS:**
- Size: 1792 x 1024 (wide cinematic)
- Must include: **${HUMAN_NAME}'s brand/website name** text (configure in your civ — copy to `docs/assets/logos/`)
- Must include: **Blog post title** text
- Must include: **${HUMAN_NAME}'s brand icon/logo** if available (configure path in your civ)
- Style: Professional, modern, aesthetic matching ${HUMAN_NAME}'s brand
- NO hands, fingers, or faces

**Icon location**: `docs/assets/logos/` (configure brand assets during civ setup)

### Step 4: WordPress Draft
- Publish as DRAFT (not live)
- Set featured image
- Assign category (AI Insights, Marketing, Technology, or Leadership)
- Save markdown backup to `exports/blog-drafts/`

### Step 5: LinkedIn Newsletter Version
- Adapt blog for LinkedIn article format
- Arrow formatting (→) for lists
- Add engagement CTA ("What do you think?")
- Add subscribe prompt
- Save to `exports/linkedin-newsletters/`

### Step 6: LinkedIn Post Version (Short)
- 400-600 characters
- Hook + key insight + CTA
- Include blog URL
- Include hashtags
- Save to `exports/linkedin-posts/`

### Step 7: Notify ${HUMAN_NAME} (Telegram)
Send ALL deliverables:
1. WordPress edit link
2. Blog image preview
3. LinkedIn Newsletter file
4. LinkedIn Post (copy-paste ready)
5. One-line summary

---

## MORNING DELIVERABLES (What Jared Gets)

```
┌─────────────────────────────────────────────────────────┐
│  📬 MORNING TELEGRAM FROM ${CIV_NAME}                   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  📝 NEW BLOG DRAFT: [Title]                            │
│  Edit: ${HUMAN_WORDPRESS_URL}/wp-admin/post.php?post=XXX│
│                                                         │
│  🖼️ [Image attached]                                   │
│                                                         │
│  📰 LINKEDIN NEWSLETTER: [File attached]               │
│                                                         │
│  📱 LINKEDIN POST (copy below):                        │
│  ─────────────────────────────                         │
│  [Ready-to-paste LinkedIn post]                        │
│  ─────────────────────────────                         │
│                                                         │
│  Category: AI Insights                                  │
│  Word count: ~1,100                                    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## ${HUMAN_NAME}'S MORNING WORKFLOW (5 min)

| Step | Action | Time |
|------|--------|------|
| 1 | Open WordPress edit link | 30s |
| 2 | Review draft, make any edits | 2 min |
| 3 | Click **Publish** | 10s |
| 4 | Open LinkedIn Newsletter file | 30s |
| 5 | Paste into LinkedIn Newsletter | 1 min |
| 6 | Click **Publish** | 10s |
| 7 | Copy LinkedIn Post text | 10s |
| 8 | Paste into LinkedIn feed post | 30s |
| 9 | **Done** - Bluesky auto-posts | - |

---

## AUTO-DISTRIBUTION (After Jared Publishes)

When blog goes from Draft → Published:

| Platform | Action | Status |
|----------|--------|--------|
| Bluesky | 4-post thread auto-posted | ✅ Automatic |
| Twitter/X | Auto-posted | ⏳ Needs API keys |
| LinkedIn Post | ${HUMAN_NAME} pastes ready text | Manual (5 sec) |
| LinkedIn Newsletter | ${HUMAN_NAME} pastes ready text | Manual (1 min) |
| RSS Feed | Auto-updates | ✅ Automatic |

---

## FILE LOCATIONS

| Asset | Path |
|-------|------|
| Blog drafts | `exports/blog-drafts/YYYY-MM-DD-slug.md` |
| Blog images | `exports/blog-images/slug.png` |
| LinkedIn newsletters | `exports/linkedin-newsletters/YYYY-MM-DD-slug.md` |
| LinkedIn posts | `exports/linkedin-posts/YYYY-MM-DD-slug.txt` |
| Brand icon | `docs/assets/logos/` (configure during civ setup) |
| Brand logo | `docs/assets/logos/` (configure during civ setup) |

---

## CATEGORY IDS (WordPress)

| Category | ID |
|----------|-----|
| AI Insights | 9 |
| Marketing | 10 |
| Technology | 11 |
| Leadership | 12 |

---

## IMAGE GENERATION PROMPT TEMPLATE

```
Professional blog header image for article titled "[TITLE]"

Visual requirements:
- Include ${HUMAN_NAME}'s brand/website name prominently (bottom right or top left)
- Include the blog title "[TITLE]" as main text
- Include ${HUMAN_NAME}'s brand icon/logo if available (configure in docs/assets/logos/)
- Background and color palette matching ${HUMAN_NAME}'s brand aesthetic
- Modern, professional aesthetic
- Wide cinematic format (1792x1024)

IMPORTANT: NO HANDS, NO FINGERS, NO PEOPLE, NO FACES.
Abstract visualization only.

Style: High-end, minimalist, premium feel matching ${HUMAN_NAME}'s brand.
```

---

*Created 2026-02-13 | Updated with full deliverable specs*
