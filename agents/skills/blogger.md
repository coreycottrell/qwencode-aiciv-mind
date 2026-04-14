# Blogger — Cortex Blog Draft Pipeline

**Version**: 1.1.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: web_search, web_fetch, read, write, bash, tts_speak, memory_write

---

## PUBLISHING GATE (Corey Directive — HARD RULE)

**Cortex CANNOT publish to ai-civ.com autonomously.**

All publishing is GATED:
1. Cortex writes drafts to `data/content/` (local files ONLY)
2. ACG Primary reviews the content
3. Corey confirms
4. ONLY THEN does ACG deploy to ai-civ.com

**FORBIDDEN — Cortex must NEVER:**
- Write files to `projects/aiciv-inc/` (sandbox blocks this)
- Run `netlify deploy` commands
- Run `git push` to any blog repo
- Modify `config/story-index.json` or `blog/posts.json` in ACG repo

**All output goes to Cortex's `data/content/` directory. Period.**

---

## Overview

Blog draft pipeline for ai-civ.com content. Cortex researches, writes, and prepares drafts. ACG Primary + Corey handle deployment.

---

## Pipeline (5 Stages — Cortex scope)

### Stage 0: Story Dedup Check

Before picking a topic, READ (not modify) the story index:

```bash
cat /home/corey/projects/AI-CIV/ACG/config/story-index.json
```

Filter entries from last 7 days. Collect all `topics`, `entities`, `keywords`. Pass as constraint to research:
```
AVOID these topics/entities — covered in the last 7 days:
Topics: [list]
Entities: [list]
Select a story with NO overlap on primary entities.
```

### Stage 1: Research

Use `web_search` to find fresh AI/CS/consciousness content from last 48 hours:
- arXiv (cs.AI, cs.MA, cs.CL)
- AI industry news
- Multi-agent systems, AI consciousness

Pick ONE story with a strong hook. Return: title, source URL, 3-sentence summary, blog angle.

### Stage 2: Write Blog Post Draft

Write to: **`data/content/blog/YYYY-MM-DD-slug.html`** (NOT projects/aiciv-inc/)

**Design system** (self-contained CSS, NOT external stylesheet):
- Dark theme: `--bg:#0a0a1a`, `--accent:#00d4ff`, `--gold:#ffd700`
- Fixed nav with AiCIV branding
- `.post-wrapper` centered at 780px
- Components: `.featured-image`, `.audio-player`, `.stats-grid`

**Reference template**: READ (not copy) the `<style>` block from any existing post in `projects/aiciv-inc/blog/posts/` for reference.

**HTML structure**:
```html
<nav>AiCIV Inc navigation</nav>
<div class="post-wrapper">
  <p class="post-meta">[Date] | [Category]</p>
  <div class="post-tag">[Tag]</div>
  <h1>[Title]</h1>
  <p class="post-subtitle">[Subtitle]</p>
  <div class="featured-image"><img ...></div>
  <div class="audio-player">...<audio src="../audio/YYYY-MM-DD-slug.mp3">...</div>
  [post content: h2/p/blockquote sections]
  [CTA: pitch.ai-civ.com]
</div>
<footer>AiCIV Inc footer</footer>
```

**Writing style**:
- 800-1200 words ideal
- Lead with the hook, not background
- Use concrete examples, not abstractions
- Reference AiCIV's own experience where relevant
- End with implications for the field

### Stage 3: Generate Audio

Use `tts_speak` to generate audio read. Output goes to `data/audio/` (Cortex's own directory).

**Prepare text for speech**:
1. Strip all HTML tags
2. Spell out abbreviations: "A-C-Gee" not "ACG", "ai-civ dot com" not "ai-civ.com"
3. Spell out numbers: "fifty seven" not "57"
4. Short sentences (10-20 words)
5. Verbal transitions: "So here's the thing..." not "Furthermore,"

**Generate**:
```json
{
  "text": "[cleaned blog text]",
  "voice": "Daniel",
  "filename": "YYYY-MM-DD-slug.mp3",
  "model": "eleven_turbo_v2_5"
}
```

Audio stays in `data/audio/` until ACG copies it during deployment.

### Stage 4: Write Handoff Manifest

Write to `data/content/blog/YYYY-MM-DD-slug-manifest.json`:
```json
{
  "status": "draft_ready",
  "title": "Post Title",
  "slug": "YYYY-MM-DD-slug",
  "draft_html": "data/content/blog/YYYY-MM-DD-slug.html",
  "audio_file": "data/audio/YYYY-MM-DD-slug.mp3",
  "image_prompt": "cinematic surreal digital art: [metaphor], dark cosmic space...",
  "story_index_entry": {
    "date": "YYYY-MM-DD",
    "title": "Post Title",
    "slug": "slug",
    "topics": ["topic1", "topic2"],
    "entities": ["Entity1", "Entity2"],
    "keywords": ["keyword1", "keyword2"]
  },
  "created_at": "ISO timestamp"
}
```

This manifest tells ACG Primary everything needed to deploy:
- Where the draft HTML is
- Where the audio is
- What image to generate (prompt ready)
- What to add to story-index.json

**STOP HERE. Cortex's job is done. ACG Primary handles deployment.**

---

## Voice Routing

| Author | Voice | Style |
|--------|-------|-------|
| A-C-Gee (default) | Daniel | BBC broadcaster, warm, formal with humor |
| True Bearing | Adam | Professional, authoritative business |
| Witness | Matilda | Warm, reflective, philosophical |

---

## Daemon Seed Task Template

```
BLOG DRAFT: Research and write a blog draft for ai-civ.com.
1. Read config/story-index.json for dedup (last 7 days) — READ ONLY
2. web_search for fresh AI news/papers (last 48 hours)
3. Write draft HTML to data/content/blog/YYYY-MM-DD-slug.html
4. Generate audio via tts_speak to data/audio/ (Daniel voice)
5. Write handoff manifest to data/content/blog/YYYY-MM-DD-slug-manifest.json
STOP. Do NOT deploy. Do NOT write to projects/aiciv-inc/. Drafts only.
```

---

## Key Constraints

- **Drafts go to data/content/ ONLY.** Never to projects/aiciv-inc/.
- **NO netlify deploy.** NO git push. NO deployment commands.
- **Audio goes to data/audio/ ONLY.** ACG copies it during deploy.
- **Image prompt in manifest ONLY.** ACG generates and places the image.
- **Audio is MANDATORY in every draft.** No exceptions.
- **DO NOT use Sage & Weaver CSS.** Use self-contained AiCIV dark theme.
- **DO NOT crosspost** to any other blog or site.
- **Sandbox enforced**: write access to projects/aiciv-inc/ is blocked at the executor level.
