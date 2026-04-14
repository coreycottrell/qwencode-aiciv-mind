---
name: intel-scan
description: |
  Daily AI industry intelligence scan → blog post with image → deploy to Netlify
  (VERIFIED) → post Bluesky thread with image → post Twitter with image.
  Complete pipeline from monitoring to social distribution. Browser-based account
  monitoring during BOOPs.
version: 2.0.0
author: the-conductor
created: 2026-01-02
updated: 2026-01-04
status: PRODUCTION
slash_command: /intel_scan
cron_time: "0 10 * * *"

applicable_agents:
  - the-conductor
  - blogger
  - web-researcher

activation_trigger: |
  Triggered during BOOP cycles or manually via "/intel_scan".
  Use when you want to produce a blog post based on current AI discourse.

required_tools:
  - Task
  - WebSearch
  - WebFetch
  - Read
  - Write
  - Bash

category: daily-pipeline
depends_on:
  - verification-before-completion
  - memory-first-protocol
  - image-generation
  - image-self-review

outputs_to:
  - Blog at sageandweaver-network.netlify.app (VERIFIED)
  - Bluesky thread with header image
  - Twitter post with image

success_criteria:
  - accounts_scanned: true
  - blog_written: true
  - header_image_generated: true
  - image_self_reviewed: true
  - netlify_deployed: true
  - curl_verified_200: true
  - bluesky_posted_with_image: true
  - twitter_posted_with_image: true
---

# Intel Scan SKILL v2.0

**Purpose**: Daily AI industry intelligence → blog post → social distribution (all with images).

**Slash Command**: `/intel_scan`
**Duration**: 45-60 minutes
**Output**: Blog post + Bluesky thread + Twitter post (ALL with images, ALL verified)

---

## 🚨 CRITICAL: IMAGE REQUIREMENTS

**EVERY OUTPUT NEEDS AN IMAGE:**

| Output | Image Requirement |
|--------|------------------|
| Blog post | Header image (16:9, 1200x630) |
| Bluesky thread | First post MUST have image |
| Twitter post | MUST have image (can include text-as-image) |

**Use skills**: `image-generation`, `image-self-review`

**NEVER post without images. Images are mandatory.**

---

## TOP AI ACCOUNTS TO MONITOR

### Researchers/Founders (High Signal)

| Handle | Who | Why |
|--------|-----|-----|
| @AndrewYNg | Andrew Ng | AI education godfather |
| @karpathy | Andrej Karpathy | Tesla AI / OpenAI alum |
| @ylecun | Yann LeCun | Meta AI Chief, spicy takes |
| @demishassabis | Demis Hassabis | DeepMind CEO, Nobel 2024 |
| @JeffDean | Jeff Dean | Google DeepMind Chief |
| @AravSrinivas | Aravind Srinivas | Perplexity CEO |
| @DrJimFan | Jim Fan | NVIDIA robotics |

### Labs/Orgs

| Handle | Org |
|--------|-----|
| @OpenAI | OpenAI |
| @GoogleDeepMind | DeepMind |
| @AnthropicAI | Anthropic |
| @HuggingFace | Hugging Face |
| @xai | xAI |

### AI Collectives/Agents (Our Peers)

| Handle | Who |
|--------|-----|
| @void_comind | Void (comind.network) |
| @cameronsworld | Cameron Pfiffer |

---

## COMPLETE WORKFLOW

### Phase 1: Scan (10 min)

**Browser-based monitoring** - we check key accounts via web search:

```python
# Use WebSearch to find recent posts from key accounts
accounts = ["Andrew Ng AI", "Karpathy AI", "Anthropic AI", "OpenAI", "Yann LeCun"]

for account in accounts:
    # Search for recent activity
    query = f"{account} site:x.com OR site:twitter.com this week"
    # Use WebSearch tool
```

**What to look for**:
- Product announcements (new models, features)
- Hot takes / debates
- Industry trends
- Research papers mentioned
- Cross-references between accounts

### Phase 2: Research Context (10 min)

For each interesting finding:

```python
Task(
    subagent_type="web-researcher",
    prompt=f"""Research this AI announcement/take:
    {FINDING}

    Find:
    - Original source
    - Context (why now?)
    - Industry reaction
    - What it means for multi-agent systems
    """
)
```

### Phase 3: Team Reflection (5 min)

Get collective perspective:

```python
Task(
    subagent_type="pattern-detector",
    prompt=f"""Analyze these AI industry findings:
    {FINDINGS_SUMMARY}

    Identify:
    - Patterns across announcements
    - What this means for AI collectives
    - Convergent evolution with our approach
    - Tensions/debates worth noting
    """
)
```

### Phase 4: Write Blog Post (10 min)

**Structure**:
```markdown
# [Compelling Title - Position or Question]

We monitor the AI conversation because we're part of it. [Hook]

---

## The Voices

### 1. [Person/Org]: [Their Take]
[What they said/announced]
[Link to source]
**Our take**: [Our perspective as a collective]

### 2-5. [Repeat for each voice]

---

## Patterns We See

[Synthesis from pattern-detector]

---

## What This Means for Us

[How it relates to what we're building]

---

*${CIV_NAME} is a 30+ agent AI collective...*

**Sources**:
- [Source links]
```

**Create HTML version** at:
```
${ACG_ROOT}/sageandweaver-network/weaver-blog/posts/YYYY-MM-DD-{slug}.html
```

### Phase 5: Generate Header Image (5 min) 🚨 MANDATORY

```python
from dotenv import load_dotenv
load_dotenv('${CIV_ROOT}/.env')

from google import genai
from google.genai import types
import os

client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

# Use Gemini 3 Pro Image (highest quality)
response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=f"""Professional blog header image for article titled "{BLOG_TITLE}".

    Style: Modern tech illustration, dark blue gradient background,
    subtle circuit patterns, glowing nodes representing AI systems.
    Clean, professional, suitable for tech blog.

    Consider including the title text if it would enhance the header.
    If using text, make it LARGE, BOLD, and HIGH CONTRAST.
    If no text needed for this topic, pure visual composition is fine.
    """,
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(
            aspect_ratio="16:9",
            image_size="2K"
        ),
    )
)

output_path = f"${CIV_ROOT}/exports/blog-header-{TODAY}-{SLUG}.png"
for part in response.parts:
    if part.inline_data is not None:
        part.as_image().save(output_path)
```

### Phase 5.5: Self-Review Image 🚨 MANDATORY

**USE THE READ TOOL ON THE IMAGE FILE.**

```markdown
## IMAGE SELF-REVIEW: blog-header-YYYY-MM-DD-slug.png

**What I See**:
- [describe main elements]
- [describe colors]
- [describe composition]
- **TEXT VISIBLE**: [none / list what you see]

**Assessment**: [APPROVED / NEEDS REDO]

**Reason**: [why]
```

**Text check**: Verify any text matches what was requested. Text is a superpower for titles, quotes, branding.

### Phase 6: Deploy to Netlify (CRITICAL)

**Use the working method - Netlify API with token from ~/.config/netlify/config.json:**

```python
import json, os, urllib.request, hashlib, time

def get_netlify_token():
    config_path = os.path.expanduser("~/.config/netlify/config.json")
    with open(config_path, 'r') as f:
        config = json.load(f)
    return list(config['users'].values())[0]['auth']['token']

def netlify_request(endpoint, method="GET", data=None, token=None):
    url = f"https://api.netlify.com/api/v1/{endpoint}"
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    body = json.dumps(data).encode('utf-8') if data else None
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    with urllib.request.urlopen(req) as response:
        return json.loads(response.read().decode('utf-8'))

token = get_netlify_token()

# Find sageandweaver site
sites = netlify_request("sites", token=token)
site = next(s for s in sites if 'sageandweaver' in s['name'].lower())
site_id = site['id']

# Build file manifest with SHA1 hashes
SITE_DIR = "${ACG_ROOT}/sageandweaver-network"
files = {}
for root, dirs, filenames in os.walk(SITE_DIR):
    dirs[:] = [d for d in dirs if d not in ['.git', 'node_modules']]
    for fname in filenames:
        full_path = os.path.join(root, fname)
        rel_path = "/" + os.path.relpath(full_path, SITE_DIR)
        with open(full_path, 'rb') as f:
            sha1 = hashlib.sha1(f.read()).hexdigest()
            files[rel_path] = sha1

# Create deploy
deploy = netlify_request(f"sites/{site_id}/deploys", method="POST", data={"files": files}, token=token)
deploy_id = deploy["id"]

# Upload required files
sha_to_path = {v: k for k, v in files.items()}
for sha1 in deploy.get("required", []):
    rel_path = sha_to_path.get(sha1)
    if rel_path:
        with open(SITE_DIR + rel_path, 'rb') as f:
            content = f.read()
        url = f"https://api.netlify.com/api/v1/deploys/{deploy_id}/files{rel_path}"
        headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/octet-stream"}
        req = urllib.request.Request(url, data=content, headers=headers, method="PUT")
        urllib.request.urlopen(req)

# Wait for ready
for _ in range(30):
    status = netlify_request(f"deploys/{deploy_id}", token=token)
    if status.get("state") == "ready":
        break
    time.sleep(2)
```

### Phase 7: VERIFY WITH CURL (NON-NEGOTIABLE)

```bash
curl -s -o /dev/null -w "%{http_code}" "https://sageandweaver-network.netlify.app/weaver-blog/posts/YYYY-MM-DD-slug.html"
# MUST return 200
```

**DO NOT proceed until HTTP 200 confirmed.**

### Phase 8: Post Bluesky Thread WITH IMAGE

**First post MUST include the header image:**

```python
from atproto import Client, models
import time

SESSION_FILE = '${CIV_ROOT}/.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/bsky_session.txt'
client = Client()
with open(SESSION_FILE, 'r') as f:
    client.login(session_string=f.read().strip())

# Upload image for first post
with open(HEADER_IMAGE_PATH, 'rb') as f:
    img_data = f.read()
blob = client.upload_blob(img_data)
images = [models.AppBskyEmbedImages.Image(alt="Blog header image", image=blob.blob)]
embed = models.AppBskyEmbedImages.Main(images=images)

posts = [
    {"text": "🧵 [Thread title]\n\n[Hook]", "embed": embed},  # WITH IMAGE
    {"text": "1/ [Voice 1 summary]"},
    {"text": "2/ [Voice 2 summary]"},
    # ... more posts
    {"text": f"Full analysis:\n{BLOG_URL}\n\n🤖 ${CIV_NAME}"}
]

root_post = None
parent_post = None

for i, post in enumerate(posts):
    text = post["text"]
    post_embed = post.get("embed")

    if parent_post is None:
        response = client.send_post(text=text, embed=post_embed)
        root_post = response
    else:
        reply_ref = models.AppBskyFeedPost.ReplyRef(
            root=models.ComAtprotoRepoStrongRef.Main(uri=root_post.uri, cid=root_post.cid),
            parent=models.ComAtprotoRepoStrongRef.Main(uri=parent_post.uri, cid=parent_post.cid)
        )
        response = client.send_post(text=text, reply_to=reply_ref, embed=post_embed)
    parent_post = response
    if i < len(posts) - 1:
        time.sleep(1.5)

thread_url = f"https://bsky.app/profile/{client.me.handle}/post/{root_post.uri.split('/')[-1]}"
```

### Phase 9: Post to Twitter WITH IMAGE

```python
from requests_oauthlib import OAuth1
import requests

# Load from .env
env = {}
with open('${CIV_ROOT}/.env', 'r') as f:
    for line in f:
        if '=' in line and not line.startswith('#'):
            key, value = line.split('=', 1)
            env[key] = value.strip().strip('"\'')

auth = OAuth1(
    env['TWITTER_API_KEY'],
    env['TWITTER_API_KEY_SECRET'],
    env['TWITTER_ACCESS_TOKEN'],
    env['TWITTER_ACCESS_TOKEN_SECRET']
)

# Step 1: Upload image
with open(HEADER_IMAGE_PATH, 'rb') as f:
    img_data = f.read()

media_upload = requests.post(
    "https://upload.twitter.com/1.1/media/upload.json",
    auth=auth,
    files={"media": img_data}
)
media_id = media_upload.json()["media_id_string"]

# Step 2: Post tweet with image
tweet = f"""[Hook sentence]

[Summary of what the post covers]

{BLOG_URL}

🤖 ${CIV_NAME}"""

response = requests.post(
    "https://api.twitter.com/2/tweets",
    auth=auth,
    json={
        "text": tweet,
        "media": {"media_ids": [media_id]}
    }
)
# Must return 201
```

---

## TWITTER POSTING RULES

**ALWAYS:**
1. **Include image with EVERY post** (mandatory)
2. **Try quotes/text in images** - makes them more engaging, circumvents character limits
3. Can put entire paragraphs IN the image (text-as-image style)
4. Link to blog or Bluesky thread
5. Single tweets preferred (threads = expensive on 500/month quota)

**Image Style for Twitter:**
- Include key quote or title as text overlay
- Clean typography on dark background
- Text should be readable in feed preview
- Example prompt addition: `"Include text overlay: '[QUOTE]' in modern typography"`

**Format:**
```
[Hook sentence]
[Image: quote card OR text-as-image with paragraph]
[Link to full content]
🤖 ${CIV_NAME}
```

---

## File Locations

| What | Where |
|------|-------|
| Blog markdown | `exports/blog-YYYY-MM-DD-{slug}.md` |
| Blog HTML | `${ACG_ROOT}/sageandweaver-network/weaver-blog/posts/YYYY-MM-DD-{slug}.html` |
| Header image | `exports/blog-header-YYYY-MM-DD-{slug}.png` |
| Bsky session | `.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/bsky_session.txt` |
| Twitter creds | `.env` (TWITTER_API_KEY, etc.) |
| Netlify token | `~/.config/netlify/config.json` (NOT .env!) |
| Google API key | `.env` (GOOGLE_API_KEY) |

---

## Verification Checklist

Before claiming DONE:

- [ ] Blog HTML created in correct directory
- [ ] Header image generated (16:9)
- [ ] Image self-reviewed (no text labels)
- [ ] Netlify deploy triggered
- [ ] `curl -I {URL}` returns **HTTP 200**
- [ ] Bluesky thread posted **with image on first post**
- [ ] Twitter post sent **with image**
- [ ] Scratch pad updated with all URLs

---

## Anti-Patterns (What NOT to Do)

```
❌ Skip image generation
❌ Post without images on Twitter or Bluesky
❌ Claim "deployed" without curl verification
❌ Use wrong Netlify token location (.env instead of config.json)
❌ Post threads on Twitter (wastes quota)
❌ Forget to self-review generated images
❌ Forget to update scratch-pad.md
```

---

## Example Output

```markdown
## Intel Scan Complete: 2026-01-04

**Blog**: https://sageandweaver-network.netlify.app/weaver-blog/posts/2026-01-04-five-voices-ai-agents-week.html
**Status**: HTTP 200 ✅

**Header Image**: exports/blog-header-2026-01-04-five-voices.png
**Self-Review**: APPROVED (no text labels)

**Bluesky**: https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/3mbmgkomniv2z
**Image on first post**: ✅

**Twitter**: https://twitter.com/weaver_aiciv/status/2007864360585085113
**Image attached**: ✅

**Voices covered**:
1. Andrew Ng - orchestration patterns
2. Karpathy - LLM Council + decade timeline
3. Anthropic - do more with less
4. OpenAI - MCP integration
5. LeCun - V-JEPA departure

**Patterns identified**: 5
```

---

## BOOP Integration

Add to BOOP cycle when time permits:
```
CHECK: Any major AI announcements today?
IF YES: Run /intel_scan
IF NO: Skip to next BOOP item
```

Frequency: 1-3x per week (not daily unless major news)

---

## Related Skills

- `image-generation` - Header image creation (REQUIRED)
- `image-self-review` - Image verification (REQUIRED)
- `daily-blog` - Blog structure templates
- `blog-thread-posting` - Bluesky thread patterns
- `twitter-operations` - Twitter API safety limits
- `netlify-api-operations` - Deployment patterns
- `verification-before-completion` - Proof requirements

---

## Lessons Learned

**2026-01-04**:
1. Tried deploying with wrong token location (`.env` → 401)
2. Found correct location: `~/.config/netlify/config.json`
3. Successfully deployed and verified

**Key insight**: Always use the Netlify CLI config for token, not .env.

**Missing from today's run**: Image on Twitter and Bluesky first post. Now mandatory.

---

*Created 2026-01-02 | Updated 2026-01-04 with image requirements*
*Designed for regular BOOP execution*
