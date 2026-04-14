# Blog Distribution Pipeline Skill

**Purpose**: Automatically distribute published blog posts to all social platforms.

**Status**: ✅ ACTIVE (as of 2026-02-13)

---

## How It Works

### The Flow

```
YOU (morning)                    AETHER (automatic)
     │                                │
     │ 1. Review draft in WordPress   │
     │ 2. Click "Publish"             │
     │                                │
     └──────────────────────────────►─┤
                                      │
                                      │ 3. Detect new published post
                                      │ 4. Post Bluesky thread (4 posts)
                                      │ 5. Post to Twitter (if API keys)
                                      │ 6. Send LinkedIn text to Telegram
                                      │
                                      ▼
                               📱 You get notification
                                  with LinkedIn copy-paste
```

---

## Commands

```bash
# Check for new posts and distribute
python tools/blog_distribution_pipeline.py check

# Show pipeline status
python tools/blog_distribution_pipeline.py status

# Test with most recent post (dry-run, no actual posts)
python tools/blog_distribution_pipeline.py test
```

---

## Platform Status

| Platform | Status | Notes |
|----------|--------|-------|
| WordPress | ✅ Working | Draft → Publish workflow |
| Bluesky | ✅ Working | 4-post thread format |
| Twitter/X | ⚠️ Needs API keys | https://developer.twitter.com |
| LinkedIn | ✅ Manual | Copy-paste text sent to Telegram |
| Telegram | ✅ Working | Notification + LinkedIn text |
| RSS Feed | ✅ Auto | jareddsanborn.com/feed/ |

---

## Twitter/X Setup (When Ready)

1. Go to https://developer.twitter.com
2. Apply for developer account (free tier is fine)
3. Create a project and app
4. Generate API keys:
   - API Key
   - API Secret
   - Access Token
   - Access Token Secret
5. Add to `.env`:
   ```
   TWITTER_API_KEY=xxx
   TWITTER_API_SECRET=xxx
   TWITTER_ACCESS_TOKEN=xxx
   TWITTER_ACCESS_SECRET=xxx
   ```

---

## Integration with BOOPs

Add to BOOP routine:
```bash
python tools/blog_distribution_pipeline.py check
```

This will:
- Detect any posts you published since last check
- Distribute to all configured platforms
- Send you LinkedIn text on Telegram

---

## State File

Distribution state tracked in:
```
.blog_distribution_state.json
```

Contains:
- List of distributed post IDs (prevents re-posting)
- Last check timestamp

---

*Created 2026-02-13 by Aether*
