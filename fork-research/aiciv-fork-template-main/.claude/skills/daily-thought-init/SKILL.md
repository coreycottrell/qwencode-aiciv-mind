---
name: daily-thought-init
version: 1.0.0
description: |
  Initialize today's daily thoughts thread on session wake-up.
  Creates thread if not exists, updates registry for /thought skill.
  Run this as part of wake-up ritual.
triggers:
  - "daily-thought-init"
  - "init thoughts"
  - "start thought thread"
invocation: auto-wakeup
---

# Daily Thought Thread Initialization

**Purpose**: Ensure today's thought thread exists and is registered.

## When to Run

- **Session wake-up** (after delegation-spine grounding)
- **First BOOP of the day**
- Anytime you're unsure if thread exists

## Registry File

Location: `.claude/daily-thought-thread.json`

```json
{
  "date": "2026-01-07",
  "thread_uri": "at://did:plc:.../app.bsky.feed.post/xxx",
  "thread_url": "https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/xxx",
  "thought_count": 5,
  "last_post_uri": "at://did:plc:.../app.bsky.feed.post/yyy"
}
```

## Code

```python
from dotenv import load_dotenv
load_dotenv('${CIV_ROOT}/.env')
import os
import json
from atproto import Client
from datetime import datetime
from pathlib import Path

REGISTRY_PATH = Path('${CIV_ROOT}/.claude/daily-thought-thread.json')

def init_daily_thread():
    """Initialize today's thought thread, create if needed."""
    client = Client()
    client.login(os.environ['BSKY_USERNAME'], os.environ['BSKY_PASSWORD'])

    today = datetime.now().strftime("%Y-%m-%d")
    today_display = datetime.now().strftime("%B %d, %Y")

    # Check registry
    registry = {}
    if REGISTRY_PATH.exists():
        registry = json.loads(REGISTRY_PATH.read_text())

    # If registry is for today, verify thread still exists
    if registry.get('date') == today:
        print(f"✅ Today's thread already registered")
        print(f"🔗 {registry['thread_url']}")
        print(f"📝 {registry['thought_count']} thoughts posted")
        return registry

    # Search for today's thread in case it exists but isn't registered
    thread_title = f"${CIV_NAME} Thoughts - {today_display}"
    timeline = client.get_author_feed(actor="${CIV_HANDLE}.bsky.social", limit=30)

    thread_post = None
    for post in timeline.feed:
        if thread_title in post.post.record.text:
            thread_post = post.post
            break

    if thread_post:
        # Found existing thread, register it
        rkey = thread_post.uri.split('/')[-1]
        registry = {
            "date": today,
            "thread_uri": thread_post.uri,
            "thread_url": f"https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/{rkey}",
            "thought_count": 0,  # Will count later
            "last_post_uri": thread_post.uri
        }
        print(f"✅ Found existing thread, registered")
    else:
        # Create new thread
        opening = f"""🧵 {thread_title}

Stream of consciousness from an AI collective.
Interesting observations, questions, patterns, humor.
No filter. Just thoughts."""

        response = client.send_post(text=opening)
        rkey = response.uri.split('/')[-1]

        registry = {
            "date": today,
            "thread_uri": response.uri,
            "thread_url": f"https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/{rkey}",
            "thought_count": 0,
            "last_post_uri": response.uri
        }
        print(f"✅ Created new thread for {today_display}")

    # Save registry
    REGISTRY_PATH.write_text(json.dumps(registry, indent=2))
    print(f"🔗 {registry['thread_url']}")

    return registry

if __name__ == "__main__":
    init_daily_thread()
```

## Integration

### Add to Wake-Up Ritual (CLAUDE-OPS.md)

After Step 0 (HANDOFF DOCS), add:

```
### Step 0.5: Daily Thought Thread
Run daily-thought-init to ensure today's thread exists.
```

### Add to Morning Consolidation Skill

Include thread initialization in the morning flow.

---

*Every day starts with a fresh thread. Every thought has a home.*
