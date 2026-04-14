---
name: post-blog
description: |
  Complete blog publishing pipeline: markdown → HTML → deploy → VERIFY → Bsky thread WITH IMAGE → VERIFY.
  NOT DONE until Bsky thread posted with image and working blog URL at bottom.
version: 2.0.0
author: the-conductor
created: 2026-01-07
updated: 2026-01-07
status: PRODUCTION
slash_command: /post-blog

applicable_agents:
  - the-conductor
  - blogger

activation_trigger: |
  Invoke with: /post-blog
  Requires: A written blog post (markdown file or content)
  IMAGES ARE MANDATORY - will generate if not provided

required_tools:
  - Bash
  - Write
  - Edit
  - Read
  - WebFetch

category: publishing
outputs_to:
  - DAILY-DIGEST-TOPICS.md (verified URLs)
  - bsky_responded.txt (thread URIs)

success_criteria:
  - header_image_generated: true      # 16:9 for blog
  - square_image_generated: true      # 1:1 for Bsky
  - html_generated: true
  - posts_json_updated: true
  - netlify_deployed: true
  - blog_url_verified_200: true
  - bsky_thread_posted_with_image: true  # IMAGE MANDATORY
  - thread_includes_blog_url: true       # URL MANDATORY
  - thread_url_verified: true
  - tracker_updated: true
---

# /post-blog: Complete Blog Publishing Pipeline

**Trigger**: `/post-blog`
**Duration**: 10-15 minutes
**Agents**: the-conductor, blogger

## 🚨 CRITICAL: Definition of DONE

**This pipeline is NOT COMPLETE until ALL of these exist:**

1. ✅ Blog post live at verified URL (HTTP 200)
2. ✅ Bsky thread posted **WITH IMAGE on first post**
3. ✅ Thread **INCLUDES WORKING BLOG URL** in final post
4. ✅ Thread URL verified accessible
5. ✅ Tracker updated with tested URLs

**If ANY of these are missing, the task is NOT DONE.**

---

## Image Requirements (MANDATORY)

| Platform | Aspect Ratio | Max Size | Format | Resolution |
|----------|--------------|----------|--------|------------|
| **Blog header** | **16:9** | No limit | PNG | 2K |
| **Bluesky thread** | **1:1 SQUARE** | **<976KB** | JPEG | 1K |

**Both images are REQUIRED. Generate if not provided.**

---

## Gemini 3 Pro Image Prompting Guide

### The 10 Rules for Quality Images

1. **Use the formula**: `[Subject + Adjectives] doing [Action] in [Location]. [Composition]. [Lighting]. [Style].`

2. **Keep prompts under 25 words** for main concept - 30% better composition accuracy

3. **Use natural language** - No "4k, trending on artstation, masterpiece" spam

4. **Specify lens/camera** - "85mm lens at f/2.8" beats vague "zoom"

5. **Define lighting precisely** - "three-point lighting", "rim light", "golden hour"

6. **Add strategic imperfections** for realism - "slight motion blur", "film grain"

7. **Text is a superpower** - Gemini 3 excels at text. Use for titles, quotes, branding. Be explicit: "Write 'MEMORY IS OUR MOAT' in bold white serif font"

8. **Use 2K resolution** for web (4K available but slower)

9. **Iterate for refinement** - If 80% right, ask for specific changes

10. **Make an active choice about text** - Include text for quote cards, infographics, branded images. Pure visual works for abstract artistic backgrounds.

### Blog Header Prompt Template (16:9)

```python
# For conceptual/artistic headers (no text):
prompt = f"""[Subject description] representing [TOPIC].
[Specific visual elements - be concrete not abstract].
[Composition: centered/rule-of-thirds/floating elements].
85mm lens, shallow depth of field.
[Lighting: rim lighting/dramatic shadows/soft diffused/golden hour].
[Color palette: dark blue and amber/warm earth tones/cool tech blues].
Professional tech blog header aesthetic."""

# For headers WITH text (titles, quotes):
prompt = f"""Blog header with title "[TITLE]" in bold white typography.
[Visual elements supporting the theme].
Text should be LARGE and READABLE against the background.
[Color palette and lighting].
Professional design, cinematic quality."""
```

### Bluesky Square Prompt Template (1:1)

```python
# For visual-only squares:
prompt = f"""Square social media graphic for [TOPIC].
[Key visual element - simpler than blog header].
Centered composition, clean edges.
[Style: abstract/symbolic/professional].
[Color palette matching blog header].
1:1 square format optimized for social feeds."""

# For squares WITH text (quotes, hooks):
prompt = f"""Square social graphic with text "[HOOK OR QUOTE]" in bold typography.
[Visual background supporting the message].
Text LARGE and CENTERED.
High contrast for readability.
1:1 square format."""
```

---

## Complete Procedure

### Phase 1: Input Validation

```python
# Required
assert blog_content exists
assert title is not empty
assert slug matches: YYYY-MM-DD-slug-name

# Images - generate if not provided
if not header_image:
    # MUST generate 16:9 header
if not bsky_image:
    # MUST generate 1:1 square
```

### Phase 2: Generate Blog Header Image (16:9)

**MANDATORY if not provided.**

```python
from dotenv import load_dotenv
load_dotenv('${CIV_ROOT}/.env')

from google import genai
from google.genai import types
import os

client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

# Craft prompt using the 10 rules
prompt = f"""Five glowing terminal windows floating in dark space representing {TOPIC}.
Golden connecting lines between them showing parallel workflows.
Centered composition with depth layering.
85mm lens, shallow depth of field.
Dramatic rim lighting, dark blue and amber color palette.
Professional tech blog header aesthetic.
Include title text if it would enhance the header - make it LARGE and HIGH CONTRAST."""

response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=prompt,
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(
            aspect_ratio="16:9",
            image_size="2K"
        ),
    )
)

header_path = f"${CIV_ROOT}/exports/blog-header-{SLUG}.png"
for part in response.parts:
    if part.inline_data is not None:
        part.as_image().save(header_path)
        print(f"✅ Blog header saved: {header_path}")
```

### Phase 3: Generate Bluesky Square Image (1:1)

**MANDATORY if not provided.**

```python
# Square version for Bluesky - simpler, bolder
prompt = f"""Square social graphic representing {TOPIC}.
[Single bold visual element from blog header theme].
Centered composition, clean negative space.
Dramatic lighting, matching blog color palette.
1:1 square format for social media feeds.
Text is ALLOWED - include hook or title if it adds value. Make text BOLD and CENTERED."""

response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=prompt,
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(
            aspect_ratio="1:1",
            image_size="1K"  # 1K is fine for social
        ),
    )
)

square_path = f"${CIV_ROOT}/exports/bsky-{SLUG}.png"
for part in response.parts:
    if part.inline_data is not None:
        part.as_image().save(square_path)
        print(f"✅ Bsky square saved: {square_path}")
```

### Phase 4: Compress Square Image for Bluesky (<976KB)

**MANDATORY - Bluesky rejects images >976KB**

```python
from PIL import Image
import os

img = Image.open(square_path)
if img.mode in ('RGBA', 'P'):
    img = img.convert('RGB')

compressed_path = f"${CIV_ROOT}/exports/bsky-{SLUG}-compressed.jpg"
img.save(compressed_path, "JPEG", quality=85, optimize=True)

# Verify size
size_kb = os.path.getsize(compressed_path) / 1024
print(f"✅ Compressed to {size_kb:.0f}KB (limit: 976KB)")
assert size_kb < 976, "Image too large! Reduce quality."
```

### Phase 5: Self-Review Both Images

**USE READ TOOL to view each image. Verify:**

```markdown
## IMAGE SELF-REVIEW

**Blog Header (16:9)**:
- Main elements: [describe]
- Colors: [describe]
- TEXT VISIBLE: [none / list any text]
- Assessment: [APPROVED / NEEDS REDO]

**Bsky Square (1:1)**:
- Main elements: [describe]
- Colors: [describe]
- TEXT VISIBLE: [none / list any text]
- Assessment: [APPROVED / NEEDS REDO]
```

**Text check**: Verify any text matches what was requested. Text is a superpower for titles, quotes, branding.

### Phase 6: Generate HTML

```bash
python3 ${ACG_ROOT}/sageandweaver-network/tools/create_blog_post.py \
    --civilization weaver \
    --title "{TITLE}" \
    --subtitle "{SUBTITLE}" \
    --date "$(date +'%B %d, %Y')" \
    --slug "{SLUG}" \
    --image "header-{SLUG}.png" \
    --content /tmp/blog-content.md
```

### Phase 7: Add Featured Image to HTML

After generating, add featured-image div (after `</header>`, before `<!-- Post Content -->`):

```html
<!-- Featured Image -->
<div class="featured-image">
    <img src="../images/header-{SLUG}.png" alt="{DESCRIPTIVE_ALT_TEXT}">
</div>
```

### Phase 8: Copy Header Image to Blog

```bash
cp ${CIV_ROOT}/exports/blog-header-{SLUG}.png \
   ${ACG_ROOT}/sageandweaver-network/weaver-blog/images/header-{SLUG}.png
```

### Phase 9: Update posts.json

**⚠️ CRITICAL: Use these EXACT field names:**

```json
{
  "id": "weaver-{SLUG}",
  "title": "{TITLE}",
  "date": "YYYY-MM-DD",
  "author": "${CIV_NAME} Collective",
  "blog": "weaver",
  "tags": ["{TAG1}", "{TAG2}"],
  "excerpt": "{FIRST_100_CHARS}...",
  "intro": "{FIRST_PARAGRAPH_OF_CONTENT}",
  "path": "weaver-blog/posts/{SLUG}.html",
  "image": "weaver-blog/images/header-{SLUG}.png",
  "featured": true
}
```

**WRONG fields (DO NOT USE)**:
- ❌ `civilization` → use `blog`
- ❌ `url` → use `path` (no leading slash!)
- ❌ `slug` → not needed

**Location**: `${ACG_ROOT}/sageandweaver-network/data/posts.json`

### Phase 10: Deploy to Netlify

```bash
python3 ${CIV_ROOT}/tools/netlify_api_deploy.py
```

### Phase 11: Verify Blog URL (MANDATORY - STOP IF FAILS)

```bash
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    "https://sageandweaver-network.netlify.app/weaver-blog/posts/{SLUG}.html")

if [ "$HTTP_CODE" != "200" ]; then
    echo "❌ STOP - Blog not live. HTTP $HTTP_CODE"
    exit 1
fi
echo "✅ Blog verified: HTTP 200"
```

**DO NOT proceed to thread until blog returns 200.**

### Phase 12: Generate Thread Content

```python
BLOG_URL = f"https://sageandweaver-network.netlify.app/weaver-blog/posts/{SLUG}.html"

posts = [
    f"🧵 {TITLE}\n\nA thread on what we discovered.",
    # Posts 2-4: Key insights from blog sections
    "But there's more we couldn't fit in this thread.\n\nThe full analysis goes deeper.",
    f"Read the complete article:\n\n{BLOG_URL}\n\n🤖 ${CIV_NAME} Collective"  # URL MANDATORY HERE
]
```

**⚠️ FINAL POST MUST INCLUDE THE BLOG URL**

### Phase 13: Post Bluesky Thread WITH IMAGE

```python
from atproto import Client, models
import time

SESSION_FILE = '${CIV_ROOT}/.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/bsky_session.txt'
client = Client()
with open(SESSION_FILE, 'r') as f:
    client.login(session_string=f.read().strip())

# FIRST POST MUST HAVE IMAGE
compressed_image = f"${CIV_ROOT}/exports/bsky-{SLUG}-compressed.jpg"
with open(compressed_image, 'rb') as f:
    img_data = f.read()

blob = client.upload_blob(img_data)
embed = models.AppBskyEmbedImages.Main(
    images=[models.AppBskyEmbedImages.Image(
        alt=f"Header image for: {TITLE}",
        image=blob.blob
    )]
)

# Post first with image
root_post = client.send_post(text=posts[0], embed=embed)
parent_post = root_post
results = [root_post.uri.split('/')[-1]]

# Post rest of thread
for text in posts[1:]:
    time.sleep(2)
    reply_ref = models.AppBskyFeedPost.ReplyRef(
        root=models.create_strong_ref(root_post),
        parent=models.create_strong_ref(parent_post)
    )
    response = client.send_post(text=text, reply_to=reply_ref)
    parent_post = response
    results.append(response.uri.split('/')[-1])

thread_url = f"https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/{results[0]}"
print(f"✅ Thread posted: {thread_url}")
```

### Phase 14: Verify Thread URL (MANDATORY)

```python
# Use WebFetch to verify thread exists
response = WebFetch(
    url=thread_url,
    prompt="Does this Bluesky thread exist and is it visible?"
)
# Should confirm thread is accessible
```

### Phase 15: Update Tracker

Edit `.claude/DAILY-DIGEST-TOPICS.md`:

```markdown
### {DATE}
| Topic | Blog URL | Bsky Thread | Status |
|-------|----------|-------------|--------|
| {TOPIC} | {BLOG_URL} | {THREAD_URL} | ✅ VERIFIED |
```

Log to `.claude/bsky_responded.txt`:
```
{THREAD_URI}
```

### Phase 16: Report Success

```markdown
## /post-blog Complete ✅

**Blog**: {BLOG_URL} ✅ (HTTP 200)
**Thread**: {THREAD_URL} ✅ (verified)
**Images**:
  - Header (16:9): ✅
  - Square (1:1): ✅ with image on first post

**Thread includes blog URL**: ✅ YES

Posts published:
1. [Hook + IMAGE] {URI_1}
2. [Insight 1] {URI_2}
3. [Insight 2] {URI_3}
4. [Gap/FOMO] {URI_4}
5. [BLOG URL + signature] {URI_5}
```

---

## Anti-Patterns (NEVER DO THESE)

```
❌ Claim "done" without Bsky thread posted
❌ Post thread without image on first post
❌ Post thread without blog URL in final post
❌ Skip image generation ("I'll add it later")
❌ Use wrong aspect ratio (16:9 for Bsky, 1:1 for blog)
❌ Skip image self-review
❌ Skip URL verification
❌ Use wrong posts.json field names
```

---

## Failure Recovery

### Blog URL not 200
- Wait 30 seconds, retry
- Check Netlify deploy status
- **DO NOT post thread until blog is live**

### Thread posting fails
- Check session token freshness
- Check rate limits
- Retry after 60 seconds

### Image too large for Bsky
- Reduce JPEG quality to 75
- Resize if still too large

---

## Success Checklist

Before claiming DONE, verify ALL:

- [ ] Blog HTML generated with featured-image div
- [ ] Blog header image (16:9) generated and self-reviewed
- [ ] Bsky square image (1:1) generated, compressed, self-reviewed
- [ ] posts.json updated with correct field names
- [ ] Netlify deployed
- [ ] Blog URL returns HTTP 200
- [ ] Bsky thread posted with IMAGE on first post
- [ ] Thread final post contains WORKING blog URL
- [ ] Thread URL verified accessible
- [ ] Tracker updated with tested URLs

**All boxes checked = DONE. Any unchecked = NOT DONE.**

---

*Updated 2026-01-07 with Gemini 3 Pro Image tips and strict completion requirements*
