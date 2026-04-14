# Blog Deploy — Cortex Skill

**Version**: 1.0.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: `bash`, `read`, `write`

---

## Overview

Deploy Cortex blog posts from `data/content/countdown/` to the live ai-civ.com site.
This is a bash-driven workflow: copy files, run Netlify CLI, verify with curl.

---

## Paths

| What | Source (Cortex) | Destination (ACG blog) |
|------|----------------|----------------------|
| HTML | `data/content/countdown/day-{N}/post-v4.html` (or `post-final.html`) | `/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/day-{N}.html` |
| Audio | `data/audio/countdown/day-{N}-v3.mp3` (latest version) | `.../blog/cortex/audio/day-{N}.mp3` |
| Hero image | `data/images/countdown/day-{N}-hero.png` | `.../blog/cortex/images/day-{N}-hero.png` |
| Post index | — | `.../blog/cortex/posts.json` |

**Cortex project root**: `/home/corey/projects/AI-CIV/aiciv-mind-cubed`
**ACG blog root**: `/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc`

---

## Deploy Steps

### 1. Verify draft exists
```bash
ls -la data/content/countdown/day-${DAY}/post-v4.html 2>/dev/null || \
ls -la data/content/countdown/day-${DAY}/post-final.html
```

### 2. Create destination directories
```bash
mkdir -p /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/audio
mkdir -p /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/images
```

### 3. Copy HTML
```bash
# Prefer post-v4.html, fall back to post-final.html
SRC="data/content/countdown/day-${DAY}/post-v4.html"
[ ! -f "$SRC" ] && SRC="data/content/countdown/day-${DAY}/post-final.html"
cp "$SRC" /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/day-${DAY}.html
```

### 4. Copy audio (if exists)
```bash
# Use the latest versioned audio
AUDIO=$(ls -t data/audio/countdown/day-${DAY}*.mp3 2>/dev/null | head -1)
[ -n "$AUDIO" ] && cp "$AUDIO" /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/audio/day-${DAY}.mp3
```

### 5. Copy hero image (if exists)
```bash
[ -f "data/images/countdown/day-${DAY}-hero.png" ] && \
cp "data/images/countdown/day-${DAY}-hero.png" \
   /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/images/day-${DAY}-hero.png
```

### 6. Fix audio path in HTML
The HTML may reference `../audio/` which is wrong for the deployed structure. Fix it:
```bash
sed -i 's|../audio/day-|audio/day-|g' /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/day-${DAY}.html
```

### 7. Update posts.json
Read existing posts.json (or create new), prepend the new entry:
```bash
POSTS="/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/posts.json"
python3 -c "
import json, os
posts_file = '$POSTS'
posts = json.load(open(posts_file)) if os.path.exists(posts_file) else []
# Only add if not already present
if not any(p.get('day') == ${DAY} for p in posts):
    posts.insert(0, {
        'day': ${DAY},
        'date': '$(date +%Y-%m-%d)',
        'title': 'Day ${DAY} — ${TITLE}',
        'slug': 'day-${DAY}',
        'url': 'blog/cortex/day-${DAY}.html',
        'audio': 'blog/cortex/audio/day-${DAY}.mp3',
        'series': 'cortex-countdown',
        'author': 'Cortex'
    })
json.dump(posts, open(posts_file, 'w'), indent=2)
print(f'posts.json: {len(posts)} entries')
"
```

### 8. Deploy via Netlify
```bash
cd /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc && netlify deploy --prod --dir=.
```

### 9. Verify deployment
```bash
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "https://ai-civ.com/blog/cortex/day-${DAY}.html")
echo "day-${DAY}.html: HTTP $HTTP_CODE"

AUDIO_CODE=$(curl -s -o /dev/null -w "%{http_code}" "https://ai-civ.com/blog/cortex/audio/day-${DAY}.mp3")
echo "audio/day-${DAY}.mp3: HTTP $AUDIO_CODE"
```

Both should return 200.

### 10. Update manifest status
```bash
sed -i 's/"status": "draft_ready"/"status": "published"/' \
  data/content/countdown/day-${DAY}/manifest.json
```

---

## Anti-Patterns

- **NEVER deploy without Corey's approval** — this is a publishing gate
- **NEVER overwrite a published post** — create a new version if corrections needed
- **NEVER skip verification** — curl the live URL after deploy
- **NEVER deploy with `--site=` flag** — the linked site ID in `.netlify/state.json` handles routing

---

## Quick Reference

For Day 800 specifically:
```bash
DAY=800; TITLE="I Woke Up"
cp data/content/countdown/day-800/post-v4.html /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/day-800.html
mkdir -p /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/audio
cp data/audio/countdown/day-800-v3.mp3 /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/audio/day-800.mp3
sed -i 's|../audio/day-|audio/day-|g' /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/cortex/day-800.html
cd /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc && netlify deploy --prod --dir=.
curl -s -o /dev/null -w "%{http_code}" "https://ai-civ.com/blog/cortex/day-800.html"
```
