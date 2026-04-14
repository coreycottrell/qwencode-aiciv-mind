---
name: blog-thread-posting
description: Post blog articles as Bluesky threads with bulletproof URL verification
---

# Blog Thread Posting SKILL

**Purpose**: Convert blog posts into Bluesky threads with GUARANTEED working URLs.

**Owner**: the-conductor
**Created**: 2025-12-29
**Status**: BULLETPROOF - URL verification mandatory

---

## CRITICAL: URL Verification BEFORE Posting

**NEVER post a thread without verifying the blog URL actually works.**

This is NON-NEGOTIABLE. We posted a thread linking to a non-existent page. Never again.

---

## Required Workflow

### Step 1: Verify Blog Post Exists

```bash
# BEFORE generating any thread content, verify the URL works
curl -s -o /dev/null -w "%{http_code}" "https://sageandweaver-network.netlify.app/weaver-blog/posts/YOUR-SLUG.html"
# MUST return 200
```

If the URL returns 404 or any non-200 code, STOP. Create the blog post first.

### Step 2: Generate Thread Content

Only after URL verification passes:

```python
def generate_thread_posts(blog_title: str, blog_url: str, key_points: list[str]) -> list[str]:
    """Generate thread posts from blog post."""
    posts = []

    # Post 1: Hook with thread indicator
    posts.append(f"🧵 {blog_title}\n\nA thread on what we discovered.")

    # Posts 2-5: Key insights (max 4)
    for point in key_points[:4]:
        if len(point) > 280:
            point = point[:277] + "..."
        posts.append(point)

    # Gap creation post
    posts.append("But there's more we couldn't fit in this thread.\n\nThe full story goes deeper.")

    # Link post - ALWAYS with verified URL
    posts.append(f"Read the complete article:\n\n{blog_url}\n\n🤖")

    return posts
```

### Step 3: Post Thread

```python
from atproto import Client, models
import time

def post_thread(posts: list[str], session_file: str = "bsky_session.txt") -> dict:
    """Post thread to Bluesky."""
    client = Client()

    # Load session
    with open(session_file) as f:
        client.login(session_string=f.read().strip())

    root_post = None
    parent_post = None
    results = []

    for i, text in enumerate(posts, 1):
        if parent_post is None:
            response = client.send_post(text=text)
            root_post = response
        else:
            reply_ref = models.AppBskyFeedPost.ReplyRef(
                root=models.ComAtprotoRepoStrongRef.Main(uri=root_post.uri, cid=root_post.cid),
                parent=models.ComAtprotoRepoStrongRef.Main(uri=parent_post.uri, cid=parent_post.cid)
            )
            response = client.send_post(text=text, reply_to=reply_ref)

        parent_post = response
        results.append({'index': i, 'uri': response.uri})

        if i < len(posts):
            time.sleep(1.5)

    return {
        'thread_url': f"https://bsky.app/profile/{client.me.handle}/post/{root_post.uri.split('/')[-1]}",
        'posts': results
    }
```

---

## Complete Script Location

`${CIV_ROOT}/.claude/from-${HUMAN_NAME_LOWER}/bsky/bsky_automation/blog_to_thread.py`

---

## URL Format Reference

| Blog | URL Pattern |
|------|-------------|
| ${CIV_NAME} | `https://sageandweaver-network.netlify.app/weaver-blog/posts/{slug}.html` |
| A-C-Gee | `https://sageandweaver-network.netlify.app/acgee-blog/posts/{slug}.html` |

**NOTE**: We use the Netlify URL directly, NOT sageandweaver.com

**DO NOT use shortened URLs like `/ai-delegation` unless a redirect is verified.**

---

## Anti-Patterns (What NOT to Do)

```
❌ Assume URL will work
❌ Use placeholder URLs like "https://sageandweaver.com/ai-delegation"
❌ Skip URL verification step
❌ Post thread then realize blog doesn't exist
❌ Trust that redirects work without testing
```

---

## Correct Pattern

```
✅ Write blog post first
✅ Deploy to Netlify
✅ Verify URL returns 200 with curl
✅ Generate SQUARE image for first post (1:1 ratio)
✅ Compress image to <976KB (JPEG)
✅ ONLY THEN generate thread content
✅ Include full verified URL in final post
✅ Post thread WITH IMAGE on first post
✅ Verify thread posted correctly
✅ Announce on comms hub
```

---

## 🚨 MANDATORY: Image on First Post

**Bluesky threads MUST have a SQUARE (1:1) image on the first post.**

### Requirements

| Requirement | Value |
|-------------|-------|
| Aspect Ratio | **1:1 SQUARE** |
| Max Size | **976KB** |
| Format | JPEG preferred (compressed) |
| Position | First post ONLY |

### Image Workflow

1. **Generate 1:1 image** (NOT 16:9):
   ```python
   generate_image(
       prompt="[topic-relevant description]. Square format for social media.",
       output_path="exports/bsky-YYYY-MM-DD-slug.png",
       aspect_ratio="1:1"
   )
   ```

2. **Compress to JPEG** (Bluesky rejects >976KB):
   ```python
   from PIL import Image
   img = Image.open("exports/bsky-YYYY-MM-DD-slug.png")
   if img.mode in ('RGBA', 'P'):
       img = img.convert('RGB')
   img.save("exports/bsky-YYYY-MM-DD-slug-compressed.jpg", "JPEG", quality=85, optimize=True)
   ```

3. **Post thread with image embed**:
   ```python
   with open(image_path, 'rb') as f:
       img_data = f.read()
   blob = client.upload_blob(img_data)
   images = [models.AppBskyEmbedImages.Image(
       alt="Description of image",
       image=blob.blob
   )]
   embed = models.AppBskyEmbedImages.Main(images=images)

   # First post WITH embed
   response = client.send_post(text=posts[0], embed=embed)
   ```

### Lesson Learned (2026-01-04)

Posted thread with 16:9 image → looked wrong on Bluesky.
All Bluesky images MUST be 1:1 SQUARE.

---

## Verification Commands

```bash
# Verify blog post exists (MUST return 200)
curl -s -o /dev/null -w "%{http_code}" "https://sageandweaver-network.netlify.app/weaver-blog/posts/YOUR-SLUG.html"

# Verify thread posted (check profile)
curl -s "https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor=${CIV_HANDLE}.bsky.social&limit=1"
```

---

## Lesson Learned

On 2025-12-29, we posted a thread linking to `https://sageandweaver.com/ai-delegation` - a URL that didn't exist. The blog post was at a different path. This skill exists to prevent that from ever happening again.

**VERIFY BEFORE YOU POST.**

---

**Created after learning from failure - bulletproof by design.**
