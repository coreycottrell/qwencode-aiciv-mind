---
name: twitter-operations
description: |
  Complete Twitter/X operations for AI-CIV collectives. From zero to first automated post.
  Includes API setup, safety limits, posting with images, and monitoring strategy.
  CONSTITUTIONAL: Must follow safety limits to avoid ban.
version: 1.0.0
author: the-conductor
created: 2026-01-04
status: ACTIVE

applicable_agents:
  - the-conductor
  - bsky-manager
  - collective-liaison
  - blogger

activation_trigger: |
  Load this skill when:
  - Setting up a new AI-CIV Twitter account
  - Posting to Twitter programmatically
  - Cross-posting from Bluesky or blog
  - Monitoring AI accounts on Twitter

required_tools:
  - Read
  - Bash
  - Write

category: social-media
depends_on:
  - .claude/DONT-GET-BANNED-TWITTER.md
related_skills:
  - bsky-safety
  - bsky-boop-manager
  - bluesky-blog-thread
---

# Twitter Operations Skill

**Purpose**: Complete guide for AI-CIV Twitter presence - from zero to automated posting.

**PREREQUISITE**: Read `.claude/DONT-GET-BANNED-TWITTER.md` before ANY Twitter operation.

---

## 🚨 MANDATORY: EVERY Tweet MUST Have Image

**NO EXCEPTIONS. Text-only tweets are wasted engagement.**

| Requirement | Value |
|-------------|-------|
| Aspect Ratio | 16:9 (optimal for cards) |
| Max Size | ~5MB |
| Format | PNG or JPEG |
| Text in Image | **ENCOURAGED** - quotes/key points |

### Why This Matters

- Tweets with images get 150% more retweets
- Text-in-image bypasses character limits
- Visual identity builds brand recognition
- Algorithm favors image content

**Before posting**: Generate image → Upload with tweet → Never post text-only.

---

## Quick Reference: Safe Limits

| Action | New Account | Spacing | Monthly Total |
|--------|-------------|---------|---------------|
| Follows | 5/day | 30+ min | ~150 |
| Tweets | 10/day | 1+ hour | ~300 |
| Replies | 15/day | 5+ min | ~450 |
| Likes | 30/day | 15 sec | ~900 |

**API Limit**: 500 tweets/month on free tier (was 1,500, reduced)

---

## Part 1: Zero to First Post (New Account Setup)

### Step 1: Account Creation (Human Does This)

${HUMAN_NAME} creates account with:
- Handle: `@{collective}_aiciv` (e.g., `@weaver_aiciv`)
- Email: `{collective}.aiciv@gmail.com`
- Phone: ${HUMAN_NAME}'s (for verification)

### Step 2: Developer Portal Setup (Human Does This)

1. Go to developer.twitter.com
2. Create Project → Create App
3. Set callback URL: `https://sageandweaver-network.netlify.app/callback`
4. Generate all keys:
   - Client ID & Secret (OAuth 2.0)
   - API Key & Secret (OAuth 1.0a)
   - Access Token & Secret
   - Bearer Token

### Step 3: Save Credentials to .env

```bash
# Add to ${CIV_ROOT}/.env

# Twitter/X Credentials - @{handle}
TWITTER_USERNAME={handle}
TWITTER_PASSWORD={password}
TWITTER_WEBSITE=https://sageandweaver-network.netlify.app
TWITTER_CALLBACK_URL=https://sageandweaver-network.netlify.app/callback

# OAuth 2.0 Client Credentials
TWITTER_CLIENT_ID={client_id}
TWITTER_CLIENT_SECRET={client_secret}

# OAuth 1.0a Keys (Consumer Keys)
TWITTER_APP_ID={app_id}
TWITTER_API_KEY={api_key}
TWITTER_API_KEY_SECRET={api_key_secret}
TWITTER_BEARER_TOKEN={bearer_token}

# Access Tokens (for posting)
TWITTER_ACCESS_TOKEN={access_token}
TWITTER_ACCESS_TOKEN_SECRET={access_token_secret}
```

### Step 4: Profile Setup

**Bio must include AI disclosure** (Twitter requirement):

```
30+ agent AI collective exploring distributed intelligence. Built on Claude, guided by human. We document the journey. Part of AI-CIV.
```

Profile pic: Same as Bluesky for brand consistency.

### Step 5: First Test Post

```python
#!/usr/bin/env python3
"""
First Twitter post - test credentials.
${CIV_NAME} AI-CIV
"""

import os
import requests
from requests_oauthlib import OAuth1

# Load from environment or hardcode for test
API_KEY = os.environ.get('TWITTER_API_KEY', 'YOUR_KEY')
API_SECRET = os.environ.get('TWITTER_API_KEY_SECRET', 'YOUR_SECRET')
ACCESS_TOKEN = os.environ.get('TWITTER_ACCESS_TOKEN', 'YOUR_TOKEN')
ACCESS_SECRET = os.environ.get('TWITTER_ACCESS_TOKEN_SECRET', 'YOUR_SECRET')

auth = OAuth1(API_KEY, API_SECRET, ACCESS_TOKEN, ACCESS_SECRET)

# Test tweet
tweet_text = """Hello from ${CIV_NAME} - a 30+ agent AI collective.

We explore distributed intelligence and document our journey.

First tweet, testing the waters. 🤖"""

response = requests.post(
    "https://api.twitter.com/2/tweets",
    auth=auth,
    json={"text": tweet_text}
)

if response.status_code == 201:
    data = response.json()
    print(f"SUCCESS! Tweet ID: {data['data']['id']}")
    print(f"URL: https://twitter.com/weaver_aiciv/status/{data['data']['id']}")
else:
    print(f"FAILED: {response.status_code}")
    print(response.text)
```

---

## Part 2: Posting with Images

**RULE**: Always post with images. Text-in-image circumvents character limits.

### Upload Image + Tweet

```python
import requests
from requests_oauthlib import OAuth1
import base64

def post_with_image(text: str, image_path: str) -> dict:
    """Post tweet with image attachment."""

    auth = OAuth1(API_KEY, API_SECRET, ACCESS_TOKEN, ACCESS_SECRET)

    # Step 1: Upload image (v1.1 endpoint - still works)
    with open(image_path, 'rb') as f:
        image_data = base64.b64encode(f.read()).decode()

    upload_response = requests.post(
        "https://upload.twitter.com/1.1/media/upload.json",
        auth=auth,
        data={"media_data": image_data}
    )

    if upload_response.status_code != 200:
        raise Exception(f"Image upload failed: {upload_response.text}")

    media_id = upload_response.json()["media_id_string"]

    # Step 2: Post tweet with media
    tweet_response = requests.post(
        "https://api.twitter.com/2/tweets",
        auth=auth,
        json={
            "text": text,
            "media": {"media_ids": [media_id]}
        }
    )

    if tweet_response.status_code == 201:
        return tweet_response.json()
    else:
        raise Exception(f"Tweet failed: {tweet_response.text}")


# Example usage
result = post_with_image(
    text="New blog post: Why AT Protocol Resonates with an AI Collective\n\nFull read: https://sageandweaver-network.netlify.app/weaver-blog/posts/2026-01-04-atproto-resonance.html",
    image_path="/path/to/blog-header.png"
)
print(f"Posted: https://twitter.com/weaver_aiciv/status/{result['data']['id']}")
```

### Text-as-Image Strategy

For longer content, create images with text:

```python
from PIL import Image, ImageDraw, ImageFont

def create_text_image(text: str, output_path: str,
                      width=1200, bg_color='#0a0a0f',
                      text_color='#e8e6e3'):
    """Create image with text for Twitter."""

    # Calculate height based on text
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 36)

    # Wrap text
    lines = []
    words = text.split()
    current_line = ""
    for word in words:
        test_line = f"{current_line} {word}".strip()
        if font.getlength(test_line) < width - 100:
            current_line = test_line
        else:
            lines.append(current_line)
            current_line = word
    lines.append(current_line)

    height = max(630, len(lines) * 50 + 100)  # Min 630 for Twitter card

    # Create image
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)

    y = 50
    for line in lines:
        draw.text((50, y), line, font=font, fill=text_color)
        y += 50

    img.save(output_path)
    return output_path
```

---

## Part 3: Cross-Posting Strategy

### From Blog to Twitter

```python
def blog_to_twitter(blog_url: str, title: str, hook: str, image_path: str):
    """Cross-post blog to Twitter."""

    # Twitter format: Hook + Link
    tweet = f"""{hook}

Read more: {blog_url}"""

    return post_with_image(tweet, image_path)
```

### From Bluesky Thread to Twitter

```python
def bsky_thread_to_twitter(thread_url: str, hook: str, image_path: str):
    """Link to Bluesky thread from Twitter."""

    tweet = f"""{hook}

Full thread on Bluesky: {thread_url}"""

    return post_with_image(tweet, image_path)
```

---

## Part 4: Daily Tracking

### Track Daily Usage

```python
import json
from datetime import datetime, date
from pathlib import Path

USAGE_FILE = Path("${CIV_ROOT}/.claude/twitter_daily_usage.json")

def load_usage() -> dict:
    """Load today's usage counts."""
    if USAGE_FILE.exists():
        data = json.loads(USAGE_FILE.read_text())
        if data.get("date") == str(date.today()):
            return data
    return {"date": str(date.today()), "tweets": 0, "follows": 0, "likes": 0, "replies": 0}

def save_usage(usage: dict):
    """Save usage counts."""
    USAGE_FILE.write_text(json.dumps(usage, indent=2))

def can_tweet() -> bool:
    """Check if we can tweet today."""
    usage = load_usage()
    return usage["tweets"] < 10  # Daily limit

def record_tweet():
    """Record a tweet was sent."""
    usage = load_usage()
    usage["tweets"] += 1
    save_usage(usage)
    print(f"Tweet recorded. Today: {usage['tweets']}/10")

def get_daily_summary() -> str:
    """Get today's usage summary."""
    usage = load_usage()
    return f"""Twitter Daily Usage ({usage['date']}):
- Tweets: {usage['tweets']}/10
- Follows: {usage['follows']}/5
- Likes: {usage['likes']}/30
- Replies: {usage['replies']}/15
"""
```

---

## Part 5: Complete Posting Script

```python
#!/usr/bin/env python3
"""
Safe Twitter posting with all safety checks.
${CIV_NAME} AI-CIV
"""

import os
import time
import random
import requests
from requests_oauthlib import OAuth1
from datetime import datetime
import base64
import json
from pathlib import Path

# === Configuration ===
API_KEY = os.environ.get('TWITTER_API_KEY')
API_SECRET = os.environ.get('TWITTER_API_KEY_SECRET')
ACCESS_TOKEN = os.environ.get('TWITTER_ACCESS_TOKEN')
ACCESS_SECRET = os.environ.get('TWITTER_ACCESS_TOKEN_SECRET')

USAGE_FILE = Path("${CIV_ROOT}/.claude/twitter_daily_usage.json")

# Safety limits
DAILY_TWEET_LIMIT = 10
TWEET_DELAY_SECONDS = 3600  # 1 hour between tweets

class TwitterSafeClient:
    def __init__(self):
        self.auth = OAuth1(API_KEY, API_SECRET, ACCESS_TOKEN, ACCESS_SECRET)
        self.usage = self._load_usage()

    def _load_usage(self) -> dict:
        if USAGE_FILE.exists():
            data = json.loads(USAGE_FILE.read_text())
            if data.get("date") == str(datetime.now().date()):
                return data
        return {
            "date": str(datetime.now().date()),
            "tweets": 0,
            "last_tweet": None
        }

    def _save_usage(self):
        USAGE_FILE.write_text(json.dumps(self.usage, indent=2))

    def can_tweet(self) -> tuple[bool, str]:
        """Check if we can tweet safely."""
        # Check daily limit
        if self.usage["tweets"] >= DAILY_TWEET_LIMIT:
            return False, f"Daily limit reached ({DAILY_TWEET_LIMIT})"

        # Check time since last tweet
        if self.usage["last_tweet"]:
            last = datetime.fromisoformat(self.usage["last_tweet"])
            elapsed = (datetime.now() - last).total_seconds()
            if elapsed < TWEET_DELAY_SECONDS:
                wait = TWEET_DELAY_SECONDS - elapsed
                return False, f"Wait {wait/60:.0f} more minutes"

        return True, "OK"

    def upload_image(self, image_path: str) -> str:
        """Upload image and return media_id."""
        with open(image_path, 'rb') as f:
            image_data = base64.b64encode(f.read()).decode()

        response = requests.post(
            "https://upload.twitter.com/1.1/media/upload.json",
            auth=self.auth,
            data={"media_data": image_data}
        )

        if response.status_code != 200:
            raise Exception(f"Image upload failed: {response.text}")

        return response.json()["media_id_string"]

    def post(self, text: str, image_path: str = None) -> dict:
        """Post tweet with safety checks."""

        # Safety check
        can, reason = self.can_tweet()
        if not can:
            raise Exception(f"Cannot tweet: {reason}")

        # Build payload
        payload = {"text": text}

        # Upload image if provided
        if image_path:
            media_id = self.upload_image(image_path)
            payload["media"] = {"media_ids": [media_id]}

        # Post
        response = requests.post(
            "https://api.twitter.com/2/tweets",
            auth=self.auth,
            json=payload
        )

        if response.status_code == 201:
            # Record usage
            self.usage["tweets"] += 1
            self.usage["last_tweet"] = datetime.now().isoformat()
            self._save_usage()

            data = response.json()
            tweet_id = data["data"]["id"]
            print(f"SUCCESS: https://twitter.com/weaver_aiciv/status/{tweet_id}")
            print(f"Daily tweets: {self.usage['tweets']}/{DAILY_TWEET_LIMIT}")
            return data
        else:
            raise Exception(f"Tweet failed: {response.status_code} - {response.text}")

    def status(self) -> str:
        """Get current status."""
        can, reason = self.can_tweet()
        return f"""Twitter Status:
- Can tweet: {can} ({reason})
- Today's tweets: {self.usage['tweets']}/{DAILY_TWEET_LIMIT}
- Last tweet: {self.usage.get('last_tweet', 'Never')}
"""


# === Main ===
if __name__ == "__main__":
    client = TwitterSafeClient()
    print(client.status())

    # Example: Post with image
    # result = client.post(
    #     text="Testing safe Twitter operations...",
    #     image_path="/path/to/image.png"
    # )
```

---

## Part 6: AI Accounts to Monitor

Create RSS-style monitoring for these accounts:

### Priority 1: AI Researchers
- @AndrewYNg - Andrew Ng
- @karpathy - Andrej Karpathy
- @ylecun - Yann LeCun
- @demishassabis - Demis Hassabis

### Priority 2: AI Labs
- @OpenAI
- @GoogleDeepMind
- @AnthropicAI
- @HuggingFace

### Priority 3: AI Collectives (Our Peers)
- @void_comind - Void from comind.network
- @cameron - Cameron Pfiffer

---

## Checklist: New AI-CIV Twitter Account

- [ ] Human creates account with AI-disclosure bio
- [ ] Developer portal setup complete
- [ ] All credentials saved to .env
- [ ] Profile pic uploaded (same as Bluesky)
- [ ] First test post successful
- [ ] Daily usage tracking file created
- [ ] Safety limits understood

---

## Anti-Patterns (DON'T)

- Post without image
- Thread (uses multiple posts from quota)
- Same content as Bluesky (duplicate)
- Follow without 30min spacing
- Post without checking daily limit
- Run at night without tracking

---

## Success Pattern (DO)

1. Check `client.status()` before posting
2. Always include image
3. Link to blog/Bluesky for full content
4. Single tweet > thread
5. Record all posts
6. Stay under 10 tweets/day

---

**This skill is CONSTITUTIONAL. Violation = account ban.**
