# Blog Banner Creation Skill

**Skill ID**: `blog-banner-creation`
**Version**: 1.0.0
**Created**: 2026-02-16
**Author**: Aether (locked in per Jared's directive)

## Purpose

Create branded blog header images for PureBrain.ai that follow strict layout and branding rules to ensure consistency and mobile-safe rendering.

## LOCKED RULES (Non-Negotiable)

### 1. Logo Format: PUREBRAIN.ai (LOCKED - NON-NEGOTIABLE)

**This format applies EVERYWHERE on ANYTHING we create.**

The logo text MUST be rendered in this exact format:

| Segment | Color | Case | Notes |
|---------|-------|------|-------|
| PUREBR | PT Blue (#2a93c1) | UPPERCASE | First 6 letters |
| AI | PT Orange (#f1420b) | UPPERCASE | The "AI" inside BRAIN |
| N | PT Blue (#2a93c1) | UPPERCASE | Last letter of BRAIN |
| .ai | WHITE (#ffffff) | lowercase | Domain extension |

**Visual**: `PUREBR`<span style="color:#f1420b">`AI`</span>`N`<span style="color:#fff">`.ai`</span>

**HTML Example**:
```html
<span class="purebr">PUREBR</span><span class="ai-orange">AI</span><span class="purebr">N</span><span class="ai-white">.ai</span>
```

**CSS**:
```css
.purebr { color: #2a93c1; }      /* PT Blue */
.ai-orange { color: #f1420b; }   /* PT Orange */
.ai-white { color: #ffffff; }    /* White */
```

Always include the hexagon icon to the LEFT of the text when space permits.

### 2. The 75% Safe Zone Rule

**ALL important content MUST be within the center 75% of the image.**

```
+--------------------------------------------------+
|                    12.5% margin                   |
|  +--------------------------------------------+  |
|  |                                            |  |
|  |           75% SAFE ZONE                    |  |
|  |    (All logos, text, important content)    |  |
|  |                                            |  |
|  +--------------------------------------------+  |
|                    12.5% margin                   |
+--------------------------------------------------+
```

- **Horizontal margins**: 12.5% on left and right
- **Vertical margins**: 12.5% on top and bottom
- **Why**: Prevents cutoff on mobile devices and social media previews

### 3. Layout Hierarchy (Top to Bottom)

Within the 75% safe zone, content is arranged:

```
TOP-LEFT: [LARGE Hexagon Icon] PUREBRAINai logo (WITH SHADOW)
          (positioned LEFT or RIGHT, NOT centered)
          (shadow behind text for "lift" effect)

MIDDLE:   ARTICLE TITLE
          (LARGE font, main focus, CENTERED, word-wrapped if needed)
          (subtle shadow for readability)

BOTTOM:   (NOTHING - no tagline, title is the main content)
```

**Key Points**:
- Icon should be LARGE (90px for 1920x1080)
- Logo + icon NOT centered - position to left or right
- Shadow behind logo text creates visual lift
- Title is the ONLY text in the middle - no separate tagline

### 4. Brand Colors

| Color | Hex | RGB | Usage |
|-------|-----|-----|-------|
| PT Blue | #2a93c1 | (42, 147, 193) | PUREBR, N |
| PT Orange | #f1420b | (241, 66, 11) | AI (in BRAIN) |
| White | #ffffff | (255, 255, 255) | .ai, titles, taglines |
| Light Gray | #c8c8c8 | (200, 200, 200) | Taglines (optional) |

### 5. Sizing Guidelines

For a 1920x1080 image:
- **Hexagon icon**: 90px (LARGE)
- **Logo text**: 36px
- **Article title**: 72px (LARGE, primary focus)
- **Shadow offset**: 2-3px (for lift effect)

Scale proportionally for other image sizes.

**NO tagline** - the article title IS the main content.

### 6. Background Treatment

- Apply semi-transparent dark gradient overlay for text readability
- Stronger opacity at bottom, lighter at top
- Cover any existing text from base image before adding new text

## Implementation

Use the tool at: `tools/fix_blog_branding.py`

### Python Function Signature

```python
def create_corrected_header(article_title: str) -> Path:
    """Create blog header with correct PUREBRAIN.ai branding.

    Args:
        article_title: The main title to display (will word-wrap if needed)

    Returns:
        Path to the generated image
    """
```

### Quick Usage

```bash
# Activate venv and run
source venv/bin/activate
python tools/fix_blog_branding.py
```

Modify the `article_title` variable in the script for different posts.

## Examples

**Correct**:
- Logo: `[hex] PUREBRAINai` with AI in orange, .ai in white
- Title: Large, centered, within safe zone
- All important content 12.5%+ from edges

**Incorrect**:
- `.AI` in orange (should be `.ai` in white)
- `PUREBRAIN` all in blue (the AI inside must be orange)
- Text near image edges (will get cut off on mobile)
- Title smaller than logo (title should be primary focus)

## Verification Checklist

Before marking a blog banner complete:

- [ ] Logo format: PUREBR (blue, UPPERCASE) + AI (orange, UPPERCASE) + N (blue, UPPERCASE) + .ai (white, lowercase)
- [ ] **CRITICAL**: .ai must be WHITE and lowercase - NOT blue, NOT orange
- [ ] Hexagon icon present and LARGE (90px for 1920x1080)
- [ ] Logo + icon positioned LEFT or RIGHT (NOT centered)
- [ ] Shadow behind logo text for "lift" effect
- [ ] All content within 75% safe zone (12.5% margins)
- [ ] Article title is LARGE, centered, and prominent
- [ ] NO extra tagline - title is the main content
- [ ] Text is readable against background (shadows help)
- [ ] No old/residual text showing through

## UNIVERSAL LOGO RULE (LOCKED IN - 2026-02-16)

**This applies to ALL PureBrain.ai assets:**
- Blog headers
- Assessment pages
- Landing pages
- Social media graphics
- Email templates
- PDFs and documents
- Anywhere the logo appears

**Format**: PUREBR`AI`N`.ai`
- PUREBR = PT Blue (#2a93c1) UPPERCASE
- AI = PT Orange (#f1420b) UPPERCASE
- N = PT Blue (#2a93c1) UPPERCASE
- .ai = White (#ffffff) lowercase

---

## Publishing Workflow (LOCKED IN)

After banner creation, the FULL workflow is:

### 1. Dual-Site Publishing (MANDATORY)

**ALL blog posts go to BOTH sites simultaneously:**

| Site | URL | Credentials |
|------|-----|-------------|
| PureBrain.ai | purebrain.ai/blog | PUREBRAIN_WP_USER + PUREBRAIN_WP_APP_PASSWORD |
| JaredSanborn.com | jareddsanborn.com/blog | WORDPRESS_USER + WORDPRESS_APP_PASSWORD |

**Tool**: `tools/dual_blog_publish.py`

**Process**:
1. Upload featured image to BOTH sites
2. Create post on BOTH sites with same content
3. Return URLs from both for Bluesky thread

### 2. Bluesky Thread (MANDATORY)

After publishing to both sites, post a 5-part Bluesky thread:
1. Hook (attention grabber)
2. Problem (pain points)
3. Gap (demo vs enterprise)
4. Insight (Aether's perspective)
5. CTA + Link (to purebrain.ai post)

**Tool**: `atproto` client or existing Bluesky skills

### 3. LinkedIn Handoff (TO JARED)

LinkedIn posting is handed off to Jared for his personal account safety.

**Process**:
1. Prepare LinkedIn post content in `linkedin-post.md`
2. Include in handoff document to Jared
3. Jared posts manually to maintain account safety

**DO NOT auto-post to LinkedIn** - this is Jared's decision.

### 4. Blog Post CTA (MANDATORY)

**ALL blog posts MUST include a CTA at the bottom** to drive traffic to PureBrain.ai.

**Standard CTA Format** (add at end of every post):

```html
<hr>
<p><strong>Ready to awaken your AI partner?</strong> <a href="https://purebrain.ai">Begin the process at PureBrain.ai</a></p>
<p>And if this perspective was valuable, <a href="https://www.linkedin.com/build-relation/newsletter-follow?entityUrn=7428125791609192449">subscribe to our newsletter</a> where I share insights on building AI relationships every week.</p>
```

**This is NON-NEGOTIABLE** - every blog post on both sites must have this CTA.

---

## History

- **2026-02-16**: Skill created and locked per Jared's directive after multiple iterations
  - Key learning: The "AI" inside "BRAIN" must be orange
  - Key learning: ".ai" must be lowercase and WHITE (not orange)
  - Key learning: 75% safe zone prevents mobile cutoff issues
  - Key learning: Icon should be LARGE (90px for 1920x1080)
  - Key learning: Logo + icon should be LEFT or RIGHT aligned, NOT centered
  - Key learning: Shadow behind logo text creates "lift" effect
  - Key learning: NO separate tagline - the article title IS the main content
  - Key learning: Title should be CENTERED in the middle of safe zone
