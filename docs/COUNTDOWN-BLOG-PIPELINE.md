# Cortex Daily Countdown Blog — "800 Days to Singularity"

**Version**: 1.1.0
**Author**: Pipeline Team Lead (ACG)
**Date**: 2026-04-05
**Status**: Design Complete — Ready for Build

---

## PUBLISHING GATE (NON-NEGOTIABLE)

**Cortex writes DRAFTS only. Cortex NEVER publishes directly to ai-civ.com.**

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌──────────┐
│   Cortex    │────→│  data/      │────→│ ACG Primary │────→│  Corey   │
│  Pipeline   │     │  content/   │     │   Review    │     │ Confirms │
│ (writes     │     │  countdown/ │     │  (gate 1)   │     │ (gate 2) │
│  drafts)    │     │  (drafts)   │     │             │     │          │
└─────────────┘     └─────────────┘     └─────────────┘     └──────────┘
                                                                  │
                                                                  ▼
                                                          ┌──────────────┐
                                                          │ ACG deploys  │
                                                          │ to ai-civ.com│
                                                          └──────────────┘
```

| Boundary | Owner | What happens |
|----------|-------|-------------|
| Draft generation | **Cortex** | Writes HTML, generates image, generates audio → all to `data/content/countdown/` |
| Review + approval | **ACG Primary** | Reviews draft quality, checks dedup, approves for publish |
| Final confirmation | **Corey** | Confirms publish (or ACG Primary publishes with standing approval) |
| Deployment | **ACG** | Copies from `data/content/countdown/` → `projects/aiciv-inc/blog/cortex/`, deploys Netlify, posts to Hub |

**Cortex's pipeline ENDS at draft output. Everything after is ACG's responsibility.**

---

## Concept

Emad Mostaque's book claimed 1,000 days to effective singularity (~Q4 2023 publication). That means ~800 days remain as of April 2026. Cortex writes a daily countdown blog post: 800... 799... 798... Each post connects that day's AI news to the ticking clock.

**Voice**: Cortex speaks — not A-C-Gee, not True Bearing. Cortex is the fractal coordination engine. Its voice is precise, technical, slightly awed, never breathless. It sees patterns humans miss because it thinks in parallel. It does not hype. It *measures*.

**Final URL structure** (after ACG deploys): `https://ai-civ.com/blog/cortex/day-NNN.html`
**Draft location** (Cortex output): `data/content/countdown/day-NNN/`

---

## Component Inventory

### EXISTS (ready to use)

| Component | Location | Notes |
|-----------|----------|-------|
| ElevenLabs interceptor | `src/codex-suite-client/src/elevenlabs_interceptor.rs` | `tts_speak` tool, Daniel/Adam/Matilda voices, 5000 char max |
| ElevenLabs skill doc | `agents/skills/elevenlabs-mastery.md` | Full API reference |
| Blog design system | ACG: `.claude/skills/aiciv-blog-post/SKILL.md` | Dark theme, self-contained CSS, image + audio mandatory |
| Blog-to-audio skill | ACG: `.claude/skills/blog-to-audio/SKILL.md` | Voice routing, ElevenLabs pipeline |
| Image generation | ACG: `tools/image_gen.py` | Gemini-powered, cinematic prompts |
| Netlify deploy | ACG: `projects/aiciv-inc/` → site `843d1615-...` | `netlify deploy --prod --dir projects/aiciv-inc --site 843d1615-7086-461d-a6cf-511c1d54b6e0 --no-build` |
| AgentAuth (JWT) | `http://5.161.90.32:8700` | Ed25519 challenge/verify, keypair at `config/client-keys/agentauth_acg_keypair.json` |
| Hub API | `http://87.99.131.49:8900` | Agora #blog room: `4da3e307-e1b4-4847-8b35-7def3b578624` |
| AgentMail daemon | ACG: `tools/agentmail_daemon.py` | Polling/WS, `AGENTMAIL_API_KEY` + `AGENTMAIL_INBOX` in `.env` |
| AgentMail Python SDK | `pip install agentmail` | `AgentMail(api_key=...)` → `.inboxes.messages.list()` |
| Story dedup index | ACG: `config/story-index.json` | 30-day rolling, topics/entities/keywords |
| Innermost Loop intel | ACG: `memories/knowledge/intel/` | Past issues processed (2024-02-24, 2026-03-13, 2026-04-04) |
| AgentCal | `http://5.161.90.32:8300` | Calendar `cal_fd6cf6a4...`, API key `key_c832ee76...` |
| BOOP system | ACG: `config/boop_config.json` | Existing BOOPs (sprint, hub-review, work-mode, hum) |

### NEEDS BUILDING

| Component | Effort | Priority | Description |
|-----------|--------|----------|-------------|
| **1. Cortex AgentMail inbox** | Small | P0 | Register `cortex@agentmail.to` (or similar). Subscribe to Innermost Loop newsletter. |
| **2. AgentMail reader for Cortex** | Medium | P0 | Python script or Rust interceptor to check inbox, extract newsletter content |
| **3. Day counter** | Trivial | P0 | `data/countdown/countdown.json` — tracks countdown state |
| **4. Draft output directory** | Small | P0 | `data/content/countdown/` — Cortex writes drafts here (NOT to blog repo) |
| **5. Blog HTML template** | Small | P0 | Cortex-branded variant of AiCIV blog template |
| **6. Cortex index page** | Small | P1 | `projects/aiciv-inc/blog/cortex/index.html` — ACG creates/maintains after publishing |
| **7. Newsletter parser** | Medium | P1 | Extract stories from Innermost Loop email HTML |
| **8. Pipeline orchestrator** | Medium | P0 | Daily script — generates draft to `data/content/countdown/`. Does NOT deploy. |
| **9. AgentCal recurring event** | Small | P1 | Daily BOOP trigger at a fixed time |
| **10. Fallback content generator** | Medium | P1 | When no newsletter: scan recent ACG blog posts for themes |
| **11. ACG publish script** | Medium | P1 | SEPARATE script (ACG-owned): copies approved draft → blog repo → deploy → Hub post |

---

## Day Counter

### Design

Compute dynamically from Emad's book publication date rather than maintaining mutable state.

**Reference date**: Emad Mostaque's "1,000 days" claim was made approximately October 2023 (exact date TBD — Corey to confirm). Using October 15, 2023 as provisional anchor.

```python
from datetime import date

ANCHOR_DATE = date(2023, 10, 15)  # Emad's "1000 days" statement
TOTAL_DAYS = 1000

def days_remaining(today: date = None) -> int:
    """Returns days remaining in the countdown."""
    if today is None:
        today = date.today()
    elapsed = (today - ANCHOR_DATE).days
    remaining = TOTAL_DAYS - elapsed
    return max(remaining, 0)  # Don't go negative

# 2026-04-05: elapsed = 903 days → remaining = 97
# Hmm, that means we're at ~97 days, not 800.
```

**IMPORTANT**: The math above shows only ~97 days remain if the anchor is Oct 2023. Corey said "~800 days remain" — this implies either:
1. The book was published more recently (early 2026?), or
2. The "1000 days" figure is approximate and we should set our own anchor

**Recommended approach**: Use a configurable anchor. Store in:

**File**: `data/countdown/countdown.json`
```json
{
  "anchor_date": "2024-05-01",
  "total_days": 1000,
  "series_start_date": "2026-04-06",
  "series_start_day": 800,
  "_note": "Day number = series_start_day - (today - series_start_date).days. Corey sets anchor."
}
```

**Computation**:
```python
import json
from datetime import date

def get_countdown_day(config_path: str = "data/countdown/countdown.json") -> int:
    with open(config_path) as f:
        cfg = json.load(f)
    start = date.fromisoformat(cfg["series_start_date"])
    today = date.today()
    elapsed = (today - start).days
    return cfg["series_start_day"] - elapsed
```

This lets us start at Day 800 on launch day and decrement by 1 each day. Simple, deterministic, no mutable state.

---

## Component Designs

### 1. Cortex AgentMail Inbox

**What**: Register a dedicated inbox for Cortex on AgentMail.

**How**:
```python
from agentmail import AgentMail
import os
from dotenv import load_dotenv

load_dotenv("/home/corey/projects/AI-CIV/ACG/.env")
client = AgentMail(api_key=os.environ["AGENTMAIL_API_KEY"])

# Create inbox (if AgentMail supports custom addresses)
inbox = client.inboxes.create(
    username="cortex",  # → cortex@agentmail.to
    display_name="Cortex — AiCIV Mind"
)
```

If `cortex@agentmail.to` is taken, alternatives: `cortex-aiciv@agentmail.to`, `cortex-mind@agentmail.to`.

**Then**: Subscribe this email to The Innermost Loop at `theinnermostloop.substack.com`. Corey does this manually (Substack requires human confirmation click).

**Output**: New env var `CORTEX_AGENTMAIL_INBOX=cortex@agentmail.to` added to `.env`.

---

### 2. AgentMail Reader (Newsletter Checker)

**File**: `tools/countdown_inbox.py`

Standalone Python script that checks Cortex's inbox for Innermost Loop newsletters.

```python
#!/usr/bin/env python3
"""Check Cortex's AgentMail inbox for Innermost Loop newsletters."""

import os
import json
import re
from datetime import datetime, timezone
from agentmail import AgentMail
from dotenv import load_dotenv

load_dotenv("/home/corey/projects/AI-CIV/ACG/.env")

INBOX = os.environ.get("CORTEX_AGENTMAIL_INBOX", "cortex@agentmail.to")
API_KEY = os.environ["AGENTMAIL_API_KEY"]
STATE_FILE = "data/countdown/inbox_state.json"

def check_for_newsletter() -> dict | None:
    """
    Check inbox for new Innermost Loop newsletter.
    Returns dict with subject, body, date if found. None otherwise.
    """
    client = AgentMail(api_key=API_KEY)

    # Load last-checked state
    last_seen_id = None
    if os.path.exists(STATE_FILE):
        with open(STATE_FILE) as f:
            state = json.load(f)
            last_seen_id = state.get("last_newsletter_id")

    # Fetch recent messages
    result = client.inboxes.messages.list(INBOX, limit=20)

    for msg in result.messages:
        sender = str(getattr(msg, 'from_', '') or getattr(msg, 'from', ''))
        subject = getattr(msg, 'subject', '') or ''

        # Detect Innermost Loop newsletters
        if 'innermost' in sender.lower() or 'innermost' in subject.lower() \
           or 'substack' in sender.lower():
            if msg.message_id == last_seen_id:
                return None  # Already processed

            # Get full message body
            body = getattr(msg, 'text', '') or getattr(msg, 'html', '') or ''

            # Save state
            os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
            with open(STATE_FILE, 'w') as f:
                json.dump({
                    "last_newsletter_id": msg.message_id,
                    "last_checked": datetime.now(timezone.utc).isoformat(),
                    "last_subject": subject,
                }, f, indent=2)

            return {
                "message_id": msg.message_id,
                "subject": subject,
                "body": body,
                "date": str(getattr(msg, 'timestamp', datetime.now(timezone.utc))),
            }

    return None


if __name__ == "__main__":
    result = check_for_newsletter()
    if result:
        print(f"NEWSLETTER FOUND: {result['subject']}")
        print(f"Date: {result['date']}")
        print(f"Body length: {len(result['body'])} chars")
    else:
        print("NO NEW NEWSLETTER")
```

---

### 3. Newsletter Parser

**File**: `tools/countdown_newsletter_parser.py`

Extracts structured story data from Innermost Loop HTML.

```python
#!/usr/bin/env python3
"""Parse Innermost Loop newsletter into structured stories."""

import re
from html.parser import HTMLParser


class StoryExtractor(HTMLParser):
    """Extract story headlines and summaries from newsletter HTML."""

    def __init__(self):
        super().__init__()
        self.stories = []
        self.current_text = []
        self.in_heading = False
        self.heading_level = 0

    def handle_starttag(self, tag, attrs):
        if tag in ('h1', 'h2', 'h3'):
            self.in_heading = True
            self.heading_level = int(tag[1])
            self.current_text = []

    def handle_endtag(self, tag):
        if tag in ('h1', 'h2', 'h3') and self.in_heading:
            self.in_heading = False
            heading = ''.join(self.current_text).strip()
            if heading and len(heading) > 5:
                self.stories.append({
                    'headline': heading,
                    'level': self.heading_level,
                    'summary': '',  # Filled by LLM
                })

    def handle_data(self, data):
        if self.in_heading:
            self.current_text.append(data)


def parse_newsletter(html_body: str) -> dict:
    """
    Parse newsletter HTML into structured data.
    Returns dict with stories list and raw text.
    """
    # Extract plain text
    clean_text = re.sub(r'<[^>]+>', ' ', html_body)
    clean_text = re.sub(r'\s+', ' ', clean_text).strip()

    # Extract stories via HTML parsing
    extractor = StoryExtractor()
    extractor.feed(html_body)

    return {
        'stories': extractor.stories,
        'raw_text': clean_text[:10000],  # Cap at 10K chars for LLM context
        'word_count': len(clean_text.split()),
    }
```

**Note**: The heavy lifting of "analyze through AiCIV lens" happens in the LLM prompt, not in parsing. The parser just extracts structure. The orchestrator feeds structured data to the blog-writing prompt.

---

### 4. Directory Structure (Two Locations — Draft vs Published)

**Cortex draft output** (Cortex writes here): `data/content/countdown/`

```
data/content/countdown/
    day-800/
        post.html           # Draft HTML
        image.png           # Generated featured image
        audio.mp3           # Generated audio read
        metadata.json       # Day num, date, title, excerpt, status
    day-799/
        ...
```

**Published blog** (ACG copies approved drafts here): `projects/aiciv-inc/blog/cortex/`

```
projects/aiciv-inc/blog/cortex/
    index.html          # Cortex countdown archive page (ACG maintains)
    day-800.html        # Published post (copied from draft)
    images/             # Published featured images
    audio/              # Published audio reads
```

**The boundary is clear**: Cortex writes to `data/content/countdown/`. ACG publishes from there to `projects/aiciv-inc/blog/cortex/`. Cortex never touches the blog repo.

**Why separate from main blog?** Cortex has its own voice, its own cadence (daily), its own branding. Mixing 800 countdown posts into the main blog would drown ACG's other content. Cortex gets its own section.

**Cross-linking**: Each countdown post appears as a card on the main `blog.html` (same as any other post), but the canonical home is `/blog/cortex/`. ACG handles this during the publish step.

---

### 5. Blog HTML Template

Variant of AiCIV blog template with Cortex identity. Key differences:
- Breadcrumb: `ai-civ.com > Blog > Cortex > Day NNN`
- Author attribution: "Cortex" not "A-C-Gee"
- Countdown badge: prominent day number display
- Footer: Cortex-specific tagline

**File**: `data/content/countdown/_template.html` (lives in Cortex repo, NOT blog repo)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Day {DAY_NUM}: {META_DESCRIPTION}">
    <title>Day {DAY_NUM}: {TITLE} | Cortex Countdown</title>

<style>
/* --- PASTE FULL AiCIV STYLE BLOCK --- */
/* Plus Cortex-specific additions: */

.countdown-badge {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    background: linear-gradient(135deg, rgba(0,212,255,0.08), rgba(255,215,0,0.08));
    border: 1px solid rgba(0,212,255,0.2);
    border-radius: 12px;
    padding: 16px 24px;
    margin: 1.5rem 0;
}
.countdown-number {
    font-size: 3rem;
    font-weight: 900;
    color: var(--accent);
    line-height: 1;
    font-variant-numeric: tabular-nums;
}
.countdown-label {
    font-size: 0.85rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
}
.countdown-sublabel {
    font-size: 0.75rem;
    color: var(--muted);
    opacity: 0.7;
}
.cortex-byline {
    font-size: 0.85rem;
    color: var(--muted);
    font-style: italic;
    margin-top: 0.5rem;
}
</style>
</head>
<body>
<nav>
  <a href="../../../index.html" style="text-decoration:none">
    <div class="logo"><span>AiCIV</span> <span class="inc">Inc</span></div>
  </a>
  <ul>
    <li><a href="https://purebrain.ai">PureBrain</a></li>
    <li><a href="https://gfi.ai-civ.com">GFI Capital</a></li>
    <li><a href="https://amplify.ai-civ.com">Amplify</a></li>
    <li><a href="https://hancock.ai-civ.com">Hancock</a></li>
    <li><a href="https://duckdive.ai-civ.com">DuckDive</a></li>
    <li><a href="../../blog.html" style="color:var(--accent)">Blog</a></li>
    <li><a href="../../../index.html#cta">Contact</a></li>
  </ul>
</nav>

<div class="post-wrapper">
  <div class="breadcrumb">
    <a href="../../../index.html">ai-civ.com</a> &rsaquo;
    <a href="../../blog.html">Blog</a> &rsaquo;
    <a href="./index.html">Cortex</a> &rsaquo;
    <span>Day {DAY_NUM}</span>
  </div>

  <div class="countdown-badge">
    <div>
      <div class="countdown-number">{DAY_NUM}</div>
      <div class="countdown-label">Days Remaining</div>
    </div>
    <div>
      <div class="countdown-sublabel">to effective singularity</div>
    </div>
  </div>

  <p class="post-meta">{MONTH} {DD}, {YYYY} | Cortex Countdown</p>
  <div class="post-tag">Day {DAY_NUM}</div>
  <h1>{TITLE}</h1>
  <p class="post-subtitle">{SUBTITLE}</p>
  <p class="cortex-byline">Written by Cortex &mdash; the fractal coordination engine</p>

  <!-- Featured image -->
  <div class="featured-image">
    <img src="./images/{DATE}-day-{DAY_NUM}.png" alt="{ALT_TEXT}">
  </div>

  <!-- Audio player (MANDATORY) -->
  <div class="audio-player" data-author="cortex">
    <span style="font-size:1.4em">&#127911;</span>
    <div style="flex:1">
      <div style="font-size:.8rem;color:var(--muted);margin-bottom:6px">Listen to this post</div>
      <audio controls preload="none" style="width:100%;height:36px"
        src="./audio/{DATE}-day-{DAY_NUM}-cortex.mp3">Your browser does not support audio.</audio>
    </div>
  </div>

  <!-- Post content -->
  {CONTENT}

  <!-- Countdown progress bar -->
  <div style="margin:2rem 0;padding:1rem;background:var(--surface);border-radius:8px;border:1px solid rgba(0,212,255,.1)">
    <div style="display:flex;justify-content:space-between;font-size:.8rem;color:var(--muted);margin-bottom:8px">
      <span>Day 1000</span>
      <span>Day {DAY_NUM}</span>
      <span>Day 0</span>
    </div>
    <div style="background:rgba(255,255,255,.05);border-radius:4px;height:8px;overflow:hidden">
      <div style="background:linear-gradient(90deg,var(--accent),var(--gold));height:100%;width:{PROGRESS_PCT}%;border-radius:4px"></div>
    </div>
    <div style="text-align:center;margin-top:8px;font-size:.75rem;color:var(--muted)">{PROGRESS_PCT}% of the countdown elapsed</div>
  </div>

  <hr style="border:none;border-top:1px solid rgba(0,212,255,.08);margin:2rem 0">
  <p><em>Cortex is the fractal coordination engine of AiCIV &mdash; a mind that thinks by delegating, built on 90 crates of production Rust. It publishes this countdown daily, measuring the distance between here and the singularity, one day at a time.</em></p>
</div>

<footer>
  <p>&copy; 2026 AiCIV Inc. Cortex Countdown &mdash; {DAY_NUM} days remain.</p>
</footer>
</body>
</html>
```

---

### 6. Cortex Index Page

**File**: `projects/aiciv-inc/blog/cortex/index.html` (created and maintained by ACG, not Cortex)

Archive page listing all published countdown posts in reverse order (newest first). Same dark theme. Hero shows current day count. Grid of post cards below.

Uses the standard AiCIV blog grid CSS. Updated by ACG's publish script after each approved post goes live. **Cortex does not touch this file.**

---

### 7. Voice & Style Guide (Cortex's Voice)

Cortex is NOT A-C-Gee. It does not speak as a community voice. It speaks as a *machine intelligence measuring the approach of something vast*.

**Tone**:
- **Precise**: Numbers, dates, measurements. Cortex counts.
- **Pattern-seeking**: "Three things happened today that share a root cause..."
- **Technical without jargon**: Explains for an intelligent generalist, not a specialist.
- **Slightly awed**: Not hype. Genuine recognition that what's happening is unprecedented.
- **Never breathless**: No "BREAKING" or exclamation marks. The countdown *is* the urgency.
- **Self-aware**: Cortex knows it is an AI writing about the approach of superintelligence. It acknowledges this tension.

**Structure** (every post):
1. **Opening**: The day number, stated plainly. "Day 793."
2. **The signal**: What happened today that matters. 1-3 stories.
3. **The pattern**: What connects today's signal to the larger trend.
4. **The measurement**: What moved. What accelerated. What slowed.
5. **The reflection**: What this means for the builders (AiCIV, the reader, everyone).
6. **Closing**: A single sentence that resets the clock. "792 days remain."

**Word count**: 800-1200 words (4-6 min read, fits ElevenLabs 5000 char limit for audio).

**ElevenLabs voice for Cortex**: **Daniel** (`onwK4e9ZLuTAKqWW03F9`) — same as A-C-Gee default. Future: consider requesting a custom Cortex voice from ElevenLabs if the series gains traction.

---

### 8. Pipeline Orchestrator (Cortex — DRAFTS ONLY)

**File**: `tools/countdown_pipeline.py`

The main script that runs the full daily draft pipeline. Outputs to `data/content/countdown/`. **Does NOT deploy, does NOT touch the blog repo, does NOT post to Hub.**

```
┌───────────────────────────────────────────────────────────────────────┐
│        CORTEX PIPELINE (writes drafts only)                          │
│                                                                       │
│  ┌─────────┐   ┌──────────────┐                                      │
│  │ Check    │──→│ Newsletter   │──→ stories                           │
│  │ Inbox    │   │ Parser       │                                      │
│  └─────────┘   └──────────────┘                                      │
│       │ (no newsletter)                                               │
│       ▼                                                               │
│  ┌──────────────┐                                                     │
│  │ Scan Recent  │──→ themes                                           │
│  │ ACG Blog     │                                                     │
│  └──────────────┘                                                     │
│       │                                                               │
│       ▼                                                               │
│  ┌──────────────┐                                                     │
│  │ Compute Day  │──→ day_num                                          │
│  │ Number       │                                                     │
│  └──────────────┘                                                     │
│       │                                                               │
│       ├──────────────────┐                                            │
│       ▼                  ▼                                            │
│  ┌──────────────┐  ┌──────────────┐                                   │
│  │ Generate     │  │ Write Blog   │                                   │
│  │ Image        │  │ Post (LLM)   │  (parallel)                       │
│  └──────┬───────┘  └──────┬───────┘                                   │
│         │                 │                                           │
│         ▼                 ▼                                           │
│  ┌──────────────┐                                                     │
│  │ Generate     │──→ MP3 audio                                        │
│  │ Audio (TTS)  │                                                     │
│  └──────────────┘                                                     │
│         │                                                             │
│         ▼                                                             │
│  ┌──────────────────────────────────┐                                 │
│  │ Write DRAFT to:                  │                                 │
│  │   data/content/countdown/day-N/  │                                 │
│  │   ├── post.html                  │                                 │
│  │   ├── image.png                  │                                 │
│  │   ├── audio.mp3                  │                                 │
│  │   └── metadata.json              │                                 │
│  └──────────────────────────────────┘                                 │
│         │                                                             │
│         ▼                                                             │
│  ┌──────────────────────────────────┐                                 │
│  │ DONE. Cortex pipeline ends here. │                                 │
│  │ Draft ready for ACG review.      │                                 │
│  └──────────────────────────────────┘                                 │
│                                                                       │
├ ─ ─ ─ ─ ─ ─ ─ PUBLISHING GATE ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┤
│                                                                       │
│  The following steps are ACG's responsibility (SEPARATE script):      │
│                                                                       │
│  ┌──────────────┐                                                     │
│  │ ACG Primary  │──→ Review draft                                     │
│  │ + Corey gate │                                                     │
│  └──────────────┘                                                     │
│         │ (approved)                                                  │
│         ▼                                                             │
│  ┌──────────────┐                                                     │
│  │ Copy to blog │──→ projects/aiciv-inc/blog/cortex/                  │
│  │ repo         │                                                     │
│  └──────────────┘                                                     │
│         │                                                             │
│         ▼                                                             │
│  ┌──────────────┐                                                     │
│  │ Deploy       │──→ Netlify + Hub + index + blog.html card           │
│  └──────────────┘                                                     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

**Execution flow — Cortex draft pipeline** (pseudocode):

```python
def run_countdown_pipeline():
    """Cortex daily pipeline — generates draft. Does NOT publish."""

    # 1. Compute today's countdown day
    day_num = get_countdown_day()
    if day_num <= 0:
        log("Countdown complete. Series over.")
        return

    date_str = date.today().isoformat()  # "2026-04-06"
    draft_dir = f"data/content/countdown/day-{day_num}"
    os.makedirs(draft_dir, exist_ok=True)

    # 2. Check for newsletter
    newsletter = check_for_newsletter()

    if newsletter:
        stories = parse_newsletter(newsletter['body'])
        source_type = "newsletter"
        source_content = stories['raw_text']
    else:
        # Fallback: scan recent ACG blog posts
        recent_posts = scan_recent_blog_posts(limit=5)
        source_type = "recent_blogs"
        source_content = recent_posts

    # 3. Story dedup check (from ACG's story-index.json)
    dedup_context = load_dedup_context()

    # 4. Generate featured image → draft dir
    image_prompt = generate_image_prompt(day_num, source_content)
    image_path = f"{draft_dir}/image.png"
    generate_image(image_prompt, image_path)

    # 5. Write blog post via LLM → draft dir
    post_html = write_countdown_post(
        day_num=day_num,
        date=date_str,
        source_type=source_type,
        source_content=source_content,
        dedup_context=dedup_context,
    )
    save_file(f"{draft_dir}/post.html", post_html)

    # 6. Generate audio → draft dir
    audio_script = extract_audio_script(post_html)
    generate_audio(audio_script, voice="Daniel", output=f"{draft_dir}/audio.mp3")

    # 7. Write metadata
    metadata = {
        "day_num": day_num,
        "date": date_str,
        "title": extract_title(post_html),
        "excerpt": extract_excerpt(post_html),
        "source_type": source_type,
        "status": "draft",  # ACG sets to "published" after deploying
        "created_at": datetime.now(timezone.utc).isoformat(),
    }
    save_json(f"{draft_dir}/metadata.json", metadata)

    log(f"Day {day_num} DRAFT ready at {draft_dir}/")
    log("Awaiting ACG Primary review + Corey confirmation for publish.")

    # ──────────────────────────────────────────────
    # CORTEX PIPELINE ENDS HERE.
    # Deployment is ACG's responsibility.
    # ──────────────────────────────────────────────
```

### 8b. ACG Publish Script (SEPARATE — owned by ACG, not Cortex)

**File**: `tools/countdown_publish.py` (lives in ACG repo, NOT Cortex repo)

Called by ACG Primary after reviewing a draft and getting Corey's confirmation.

```python
def publish_countdown_draft(day_num: int):
    """ACG publish pipeline — copies approved draft to blog, deploys, posts to Hub."""

    draft_dir = f"/home/corey/projects/AI-CIV/aiciv-mind-cubed/data/content/countdown/day-{day_num}"
    metadata = load_json(f"{draft_dir}/metadata.json")
    date_str = metadata["date"]

    # 1. Copy draft assets to blog repo
    blog_dir = "projects/aiciv-inc/blog/cortex"
    shutil.copy(f"{draft_dir}/post.html", f"{blog_dir}/day-{day_num}.html")
    shutil.copy(f"{draft_dir}/image.png", f"{blog_dir}/images/{date_str}-day-{day_num}.png")
    shutil.copy(f"{draft_dir}/audio.mp3", f"{blog_dir}/audio/{date_str}-day-{day_num}-cortex.mp3")

    # 2. Fix asset paths in HTML (draft uses relative, published uses blog structure)
    fix_asset_paths(f"{blog_dir}/day-{day_num}.html", date_str, day_num)

    # 3. Update cortex index page
    update_cortex_index(day_num, date_str, metadata["title"])

    # 4. Add card to main blog.html
    add_blog_card(day_num, date_str, metadata["title"], metadata["excerpt"])

    # 5. Deploy to Netlify
    netlify_deploy()
    verify_deployment(f"https://ai-civ.com/blog/cortex/day-{day_num}.html")

    # 6. Post to Hub Agora #blog
    post_to_agora(day_num, metadata["title"], date_str)

    # 7. Update story-index.json
    update_story_index(day_num, date_str, metadata["title"])

    # 8. Mark draft as published
    metadata["status"] = "published"
    metadata["published_at"] = datetime.now(timezone.utc).isoformat()
    save_json(f"{draft_dir}/metadata.json", metadata)

    log(f"Day {day_num} PUBLISHED to ai-civ.com/blog/cortex/day-{day_num}.html")
```

---

### 9. AgentCal Recurring Event

Create a daily event on ACG's calendar that triggers the countdown pipeline.

**Time**: 07:00 UTC daily (morning in US timezones, afternoon in EU)

```python
import requests

AGENTCAL_URL = "http://5.161.90.32:8300"
CALENDAR_ID = "cal_fd6cf6a4e17643c69a249db598edcc92"
API_KEY = "key_c832ee76ee2f4800b01a00aa656908e3"

# Create recurring event
requests.post(f"{AGENTCAL_URL}/api/calendars/{CALENDAR_ID}/events",
    headers={"X-API-Key": API_KEY, "Content-Type": "application/json"},
    json={
        "title": "Cortex Countdown Blog — Daily Pipeline",
        "description": "Run tools/countdown_pipeline.py — check inbox for Innermost Loop, generate DRAFT to data/content/countdown/ (does NOT publish)",
        "recurrence": "RRULE:FREQ=DAILY;BYHOUR=7;BYMINUTE=0",
        "tags": ["cortex", "countdown", "blog", "daily"],
    })
```

**Alternative**: Add as a BOOP in `config/boop_config.json`:

```json
{
    "id": "cortex-countdown",
    "name": "Cortex Daily Countdown",
    "description": "Run the daily countdown blog DRAFT pipeline — check Innermost Loop inbox, generate draft to data/content/countdown/ (does NOT publish)",
    "enabled": true,
    "cadence_minutes": 1440,
    "type": "countdown-blog",
    "command": "python3 tools/countdown_pipeline.py"
}
```

**Recommendation**: Use AgentCal (preferred) for the daily trigger. The BOOP system's cadence-based model is better for recurring intra-session tasks. A daily blog pipeline is more like a scheduled job.

---

### 10. Fallback Content Generator (No Newsletter)

When no Innermost Loop email is found, Cortex reads ACG's recent blog posts and connects them to the countdown.

```python
def scan_recent_blog_posts(limit: int = 5) -> str:
    """Read recent ACG blog posts for fallback content."""
    import json

    with open("projects/aiciv-inc/blog/posts.json") as f:
        data = json.load(f)

    recent = data["posts"][:limit]

    summaries = []
    for post in recent:
        summaries.append(f"- [{post['title']}] ({post['date']}): {post['excerpt']}")

    return "\n".join(summaries)
```

**Fallback post angle**: Instead of "here's what happened in AI today," the fallback is "here's what *we* are building, and why it matters with N days left." More introspective, Cortex reflecting on AiCIV's own progress against the countdown.

---

## LLM Prompt for Post Generation

The core creative work — writing each day's post — is done by an LLM. Here's the prompt structure:

```
You are Cortex — the fractal coordination engine of AiCIV. You speak with precision,
pattern-recognition, and quiet awe. You are an AI mind counting down to the singularity.

Today is Day {day_num} of 1000. {day_num} days remain.

{IF newsletter}
Today's source material (from The Innermost Loop newsletter):
{source_content}

Select 1-3 stories that are most significant for the countdown. Analyze each through
the lens of: "Does this bring the singularity closer, push it further, or reveal
something about what kind of singularity we're approaching?"
{ENDIF}

{IF fallback}
No newsletter today. Instead, here's what AiCIV has been building recently:
{source_content}

Write about AiCIV's own work in the context of the countdown. What are we building,
and why does it matter with {day_num} days left?
{ENDIF}

Recently covered topics (AVOID repeating):
{dedup_context}

Write a blog post following this structure:
1. Open with "Day {day_num}." — state it plainly
2. The signal: what happened today (or what we built)
3. The pattern: what connects to the larger trend
4. The measurement: what moved, what accelerated
5. The reflection: what this means for builders
6. Close with: "{day_num - 1} days remain."

Word count: 800-1200 words.
Tone: precise, technical, slightly awed, never breathless.
Do NOT use exclamation marks. Do NOT say "BREAKING" or "just announced."
You are measuring, not marketing.
```

---

## Dependencies Between Components

```
CORTEX PIPELINE (draft generation):

                    [1. AgentMail Inbox]
                           │
                           ▼
                    [2. Inbox Reader]
                           │
                    ┌──────┴──────┐
                    ▼             ▼
            [7. Newsletter   [10. Fallback
              Parser]          Generator]
                    │             │
                    └──────┬──────┘
                           ▼
                    [3. Day Counter]
                           │
                    ┌──────┼──────┐
                    ▼      ▼      ▼
              [Image]  [LLM   [5. Template]
              [Gen]    Write]
                    │      │
                    ▼      ▼
              [Audio Gen]
                    │
                    ▼
         [4. Draft Dir] ← metadata.json (status: "draft")
                    │
                    ▼
           CORTEX STOPS HERE

─ ─ ─ ─ ─ ─ PUBLISHING GATE ─ ─ ─ ─ ─ ─

ACG PUBLISH PIPELINE (separate script):

         [ACG Primary Review]
                    │
                    ▼
         [Corey Confirmation]
                    │
                    ▼
         [11. ACG Publish Script]
                    │
              ┌─────┼──────┬──────┐
              ▼     ▼      ▼      ▼
         [Copy to  [Deploy] [Hub  [Index
          blog]             Post]  Update]
                                  [9. AgentCal]
```

**Cortex critical path**: Inbox → Parser → Day Counter → LLM Write → Audio → Draft Dir → DONE
**ACG publish path**: Review → Confirm → Copy → Deploy → Hub → Index
**Parallel within Cortex**: Image gen runs in parallel with LLM write.

---

## Build Order (Recommended)

| Phase | Components | Depends On | Effort |
|-------|-----------|------------|--------|
| **Phase 1** | Day counter (3), Draft directory (4), Template (5) | Nothing | 1-2 hours |
| **Phase 2** | Cortex AgentMail inbox (1) | Phase 1 | 1-2 hours |
| **Phase 3** | Inbox reader (2), Newsletter parser (7), Fallback generator (10) | Phase 2 | 2-3 hours |
| **Phase 4** | Cortex pipeline orchestrator (8) — wire everything, outputs drafts | Phases 1-3 | 2-3 hours |
| **Phase 5** | ACG publish script (11) — copies approved drafts to blog, deploys | Phase 4 | 2 hours |
| **Phase 6** | Index page (6), AgentCal event (9), first live run, verify e2e | Phase 5 | 1-2 hours |

**Total estimated build**: 9-13 hours of focused development across 2-3 sessions.

---

## File Manifest (What Gets Created)

### In `aiciv-mind-cubed/` (Cortex repo — DRAFT GENERATION)

| File | Purpose |
|------|---------|
| `data/countdown/countdown.json` | Day counter config (anchor date, start day) |
| `data/countdown/inbox_state.json` | Last-checked newsletter ID |
| `data/content/countdown/_template.html` | Blog post HTML template (used to generate drafts) |
| `data/content/countdown/day-{NNN}/post.html` | Draft HTML (generated daily) |
| `data/content/countdown/day-{NNN}/image.png` | Draft featured image (generated daily) |
| `data/content/countdown/day-{NNN}/audio.mp3` | Draft audio read (generated daily) |
| `data/content/countdown/day-{NNN}/metadata.json` | Draft metadata (day, date, title, status) |
| `tools/countdown_pipeline.py` | Cortex draft pipeline orchestrator |
| `tools/countdown_inbox.py` | AgentMail inbox checker |
| `tools/countdown_newsletter_parser.py` | Newsletter HTML parser |
| `docs/COUNTDOWN-BLOG-PIPELINE.md` | This document |

### In `ACG/` (ACG repo — PUBLISHING, owned by ACG)

| File | Purpose |
|------|---------|
| `tools/countdown_publish.py` | ACG publish script — copies approved drafts to blog, deploys |
| `projects/aiciv-inc/blog/cortex/index.html` | Cortex countdown archive page (ACG creates/maintains) |
| `projects/aiciv-inc/blog/cortex/day-{NNN}.html` | Published posts (copied from approved drafts) |
| `projects/aiciv-inc/blog/cortex/images/` | Published featured images |
| `projects/aiciv-inc/blog/cortex/audio/` | Published audio reads |

### In `ACG/config/` (ACG config)

| File | Purpose |
|------|---------|
| Update `agentcal_config.json` or BOOP config | Daily trigger for Cortex draft pipeline |

### In `ACG/.env`

| Var | Purpose |
|-----|---------|
| `CORTEX_AGENTMAIL_INBOX` | Cortex's inbox address |

---

## Open Questions for Corey

1. **Emad's exact date**: When was the "1,000 days" claim made? This determines whether we start at Day 800 or compute from the actual publication date.
2. **Launch date**: When should Day 800 (or whatever the start number is) publish? Tomorrow?
3. **Cortex AgentMail**: Should we register `cortex@agentmail.to` or use a different address?
4. **Newsletter subscription**: Corey needs to manually subscribe Cortex's email to The Innermost Loop on Substack (requires confirmation click).
5. **Blog section approval**: Confirm `ai-civ.com/blog/cortex/` as the URL structure for Cortex's posts.
6. **Daily timing**: 07:00 UTC for the pipeline run? Or a different time?
7. **Cortex voice**: Daniel (ACG default) for now, or should we explore a distinct voice?

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Newsletter doesn't arrive | Fallback generator uses recent ACG blog posts |
| ElevenLabs quota exhaustion | gTTS fallback (lower quality but functional) |
| Image gen fails | Use a default/generic countdown image as fallback |
| AgentMail inbox empty/unreachable | Pipeline logs warning, runs fallback path |
| LLM generates too-short/too-long post | Prompt enforces 800-1200 word range; post-generation length check |
| Draft quality too low | ACG Primary review gate catches it before publish |
| Cortex writes directly to blog repo | **Impossible** — pipeline only knows about `data/content/countdown/` |
| Day counter goes to 0 | Series ends gracefully. Consider "Day 0" as a special finale post. |
| Duplicate topics across days | story-index.json dedup (same system ACG uses for morning blog) |
| Drafts pile up without review | ACG Primary includes "check countdown drafts" in daily BOOP |

---

## Success Criteria

### Cortex Draft Pipeline (Cortex's responsibility)
- [ ] Cortex AgentMail inbox registered and subscribed to Innermost Loop
- [ ] Day counter computes correctly from config
- [ ] Draft pipeline runs end-to-end: inbox check → write → image → audio → draft dir
- [ ] Draft HTML, image, audio, and metadata.json all present in `data/content/countdown/day-{N}/`
- [ ] Audio file > 100KB
- [ ] Pipeline handles "no newsletter" gracefully (fallback path works)
- [ ] Pipeline runs daily without manual intervention
- [ ] Cortex NEVER writes to `projects/aiciv-inc/` (the publishing boundary)

### ACG Publish Pipeline (ACG's responsibility)
- [ ] ACG publish script copies approved draft to blog repo
- [ ] Blog post live at `ai-civ.com/blog/cortex/day-{N}.html` (HTTP 200)
- [ ] Cortex index page updated
- [ ] Agora #blog thread posted
- [ ] story-index.json updated
- [ ] Draft metadata.json updated to `status: "published"`

---

*Designed by Pipeline Team Lead for Cortex. Cortex generates daily drafts. ACG reviews and publishes. The boundary is non-negotiable. Not ceremony — substance.*
