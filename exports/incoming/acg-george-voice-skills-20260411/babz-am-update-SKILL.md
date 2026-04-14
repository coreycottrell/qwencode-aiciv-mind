---
name: babz-am-update
trigger: /babz-am-update
description: Produce and deliver the Babz DAILY Morning Update — personalized audio briefing for Michele (Corey's wife) via Telegram + email. George voice (British professor, witty). Poke fun at Corey. Maximum wit. Runs every morning.
version: 1.2.0
source: pipeline-lead (2026-02-25, first formal skill)
allowed-tools: Bash, Read, Write, Task
applicable_agents: [pipeline-lead, blogger, human-liaison, comms-hub]
see_also: [babz-weekly-update, mom-am-update]
---

> **Update cadence (2026-03-18):**
> - **Babz (Michele)** → `/babz-am-update` DAILY (this skill) + `/babz-weekly-update` WEEKLY (Sundays)
> - **Deb (Corey's mum)** → `/mom-am-update` DAILY (separate skill, Daniel voice, convertiblegrannie@gmail.com)

# Morning Update — Babz Audio Briefing Skill

Produce a short (~90 second) personalized audio briefing for Babz, delivered via Telegram.

## When to Use

- Daily or when significant work happened that day
- When Corey asks to "send Babz an update"
- After major milestones, product launches, or strategic shifts
- Any time exciting progress should reach the family

## Who Runs It

**pipeline-lead** owns this pipeline. The stages are:

```
GATHER CONTEXT → WRITE SCRIPT → GENERATE AUDIO → DELIVER VIA TELEGRAM → EMAIL MICHELE → PUBLISH TO ai-civ.com/babz/
```

## Step 1: Gather Context

Read these sources to understand what happened today:

```bash
# MANDATORY: Pull fresh comms hub first — stale local copy = missed messages
cd /home/corey/projects/AI-CIV/aiciv-comms-hub && git pull origin master 2>&1

# Then read these sources:
.claude/scratchpad-daily/YYYY-MM-DD.md          # Today's session journal
.claude/scratchpad.md                             # Persistent cross-session state
memories/sessions/handoff-*.md                    # Latest handoff (ls -t | head -1)
memories/knowledge/competitive/                   # Any big research findings
to-corey/drafts/                                  # Prior morning updates for tone

# Comms hub (AFTER pulling fresh):
/home/corey/projects/AI-CIV/aiciv-comms-hub/rooms/daily-updates/   # AiCIV community posts
/home/corey/projects/AI-CIV/aiciv-comms-hub/rooms/partnerships/    # Inter-civ proposals, collabs
/home/corey/projects/AI-CIV/aiciv-comms-hub/rooms/operations/      # Fleet/portal activity
```

**AiCIV Community Updates:** Check the daily-updates board for recent posts from Witness, Aether, Parallax, Keel, Lyra, and other civs. This gives much richer context for what the whole AiCIV collective is doing — Babz should hear about the whole family: 28+ active civilizations, each with a human partner. Pull highlights from any civs that filed updates and weave them into the script naturally ("And get this — Witness actually shipped X today", "Aether's been working on Y with their human", "The community now has 28 active civs"). A-C-Gee speaks for the whole movement here, not just itself.

**What to look for:**
- What was the biggest accomplishment today?
- What's the most exciting finding or decision?
- What would make a non-technical person go "wow"?
- Any product/business milestones?
- What did the team do that's genuinely impressive?

## Step 2: Write the Script

**Delegate to blogger** or write directly. Target: ~300 words (90 seconds spoken).

### Tone & Voice

- **Warm, excited, personal** — like Corey bursting to tell his wife about his day
- Slightly British-professorial humor (established in prior updates)
- Address her as "Good morning, Babz" or similar
- End with something warm and personal

### Writing Rules (TTS-Optimized)

- SHORT sentences (TTS handles these better)
- NO markdown, NO URLs, NO formatting marks
- Verbal transitions: "So here's the thing...", "And get this...", "Now here's where it gets eerie..."
- Explain jargon briefly — she's smart but not AI-native
- Use analogies from her world (neuroscience, academia, medicine)
- Numbers spoken out: "twenty thousand" not "20,000"

### What Makes a GREAT Update vs Mediocre

| Great | Mediocre |
|-------|----------|
| Opens with energy and hook | Opens with "here's what happened" |
| 2-3 specific exciting things | Laundry list of tasks |
| Explains WHY it matters | Just describes WHAT happened |
| Uses analogies she'd get | Uses AI/tech jargon |
| Ends with warmth | Ends with a summary |
| ~300 words, tight | Rambles past 400 words |
| Makes her feel proud of Corey | Makes her feel confused |

### Anti-Patterns

- Corporate tone, stiff narration
- Long compound sentences (TTS stumbles)
- Technical jargon without explanation
- Passive voice (use active, direct)
- More than 3-4 main topics (overwhelming)
- Mentioning internal infrastructure (she doesn't care about tmux panes)

## Step 3: Generate Audio

### ElevenLabs TTS

```python
import requests, os

API_KEY = os.environ.get("ELEVENLABS_API_KEY")  # from .env
VOICE_ID = "onwK4e9ZLuTAKqWW03F9"  # Daniel — BBC broadcaster, British, formal+warm, handles comedy timing (Corey-approved 2026-03-10)

url = f"https://api.elevenlabs.io/v1/text-to-speech/{VOICE_ID}"
headers = {"xi-api-key": API_KEY, "Content-Type": "application/json"}
data = {
    "text": script_text,
    "model_id": "eleven_turbo_v2_5",
    "voice_settings": {
        "stability": 0.5,
        "similarity_boost": 0.75,
        "style": 0.3,
        "use_speaker_boost": True
    }
}
resp = requests.post(url, headers=headers, json=data, timeout=120)
with open("/tmp/babz-morning-update-YYYYMMDD.mp3", "wb") as f:
    f.write(resp.content)
```

### Voice Selection

| Voice | ID | When |
|-------|-----|------|
| **Daniel** (default) | onwK4e9ZLuTAKqWW03F9 | British male, BBC broadcaster, formal but warm, handles comedy timing — **USE THIS for Babz updates** |
| **George** (retired) | JBFqnCBsd6RMkjVDRZzb | Previous Babz voice — warm storyteller, more narrative |
| **Chris** | iP95p4xoKVk53GoZ742B | Charming, casual — light updates |

**Model:** `eleven_turbo_v2_5` (fast, good quality, ~6K chars = one generation)

### Fallback: gTTS (Free)

If ElevenLabs quota is exhausted:
```bash
python3 /home/corey/projects/AI-CIV/ACG/tools/telegram-voice/send_telegram_voice.py 437939400 "script text" --provider gtts
```

## Step 4: Deliver via Telegram

```python
import requests, json

with open("/home/corey/projects/AI-CIV/ACG/config/telegram_config.json") as f:
    tg = json.load(f)

url = f"https://api.telegram.org/bot{tg['bot_token']}/sendAudio"
with open("/tmp/babz-morning-update-YYYYMMDD.mp3", "rb") as f:
    files = {"audio": ("babz-morning-update.mp3", f, "audio/mpeg")}
    data = {
        "chat_id": tg["chat_id"],  # Corey (437939400) — he forwards to Babz
        "caption": "Morning update for Babz — [date]. [1-2 sentence summary]",
        "title": "Babz Morning Update - [date]",
        "performer": "A-C-Gee"
    }
    requests.post(url, data=data, files=files, timeout=60)
```

**Note:** Send to Corey's chat_id (437939400). He forwards to Babz.

## ⚠️ Email Routing — READ BEFORE STEP 5

> **Email must be sent by human-liaison, NOT pipeline-lead.**
> Pipeline-lead is blocked from SMTP by constitutional hook (`block_direct_email.py`).
> After audio is generated, pipeline-lead must hand off to human-liaison with:
> - recipient (`michelemccue@gmail.com`)
> - subject line
> - script path (`to-corey/drafts/babz-morning-update-YYYYMMDD.md`)
> - audio path (`projects/aiciv-inc/babz/audio/babz-morning-update-YYYYMMDD.mp3`)
>
> human-liaison then executes Step 5. Pipeline-lead continues with Step 6 (publish).

## Step 5: Email to Michele (MANDATORY — DO NOT SKIP — human-liaison executes this)

**Every Babz update MUST be emailed directly to `michelemccue@gmail.com`.**

This is non-negotiable. The Telegram delivery reaches Corey. The email reaches Michele directly.

**Email must be sent by human-liaison, NOT pipeline-lead.** Pipeline-lead is blocked from SMTP by constitutional hook. After audio is generated, pipeline-lead must hand off to human-liaison with: recipient, subject, script path, audio path.

```python
import smtplib, os
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.audio import MIMEAudio
from datetime import datetime

# Load Gmail config
GMAIL_USER = "acgee.ai@gmail.com"
GMAIL_APP_PASSWORD = os.environ.get("GOOGLE_APP_PASSWORD")  # from .env

today = datetime.now().strftime("%B %d, %Y")
subject = f"Good morning, Michele — from 28 AI civilizations ({today})"

msg = MIMEMultipart("mixed")
msg["From"] = f"A-C-Gee <{GMAIL_USER}>"
msg["To"] = "michelemccue@gmail.com"
msg["Subject"] = subject

# Script text as email body
body = MIMEText(script_text, "plain")
msg.attach(body)

# Attach audio if generated
with open("/tmp/babz-morning-update-YYYYMMDD.mp3", "rb") as f:
    audio = MIMEAudio(f.read(), "mpeg")
    audio.add_header("Content-Disposition", "attachment",
                     filename=f"babz-morning-update-{datetime.now().strftime('%Y%m%d')}.mp3")
    msg.attach(audio)

with smtplib.SMTP_SSL("smtp.gmail.com", 465) as server:
    server.login(GMAIL_USER, GMAIL_APP_PASSWORD)
    server.sendmail(GMAIL_USER, "michelemccue@gmail.com", msg.as_string())

print("✅ Email sent to michelemccue@gmail.com")
```

**Verify email sent before marking step complete.**

## Step 6: Publish to ai-civ.com/babz/ (MANDATORY)

Every Babz update MUST be added to the public archive at `projects/aiciv-inc/babz/`.

**Babz can go there anytime to catch up on everything.** This is her permanent record.

### 6a: Copy audio to web dir

```bash
cp /tmp/babz-morning-update-YYYYMMDD.mp3 /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/babz/audio/
```

### 6b: Prepend new entry to babz/index.html

Add a new `.briefing-card` block BEFORE the first existing card (newest-first order).
Insert it after the `<div class="page-header">...</div>` block.

```html
<!-- [Month D, YYYY] — Babz -->
<div class="briefing-card">
  <div class="briefing-header">
    <span class="briefing-date">[Month D, YYYY]</span>
    <span class="briefing-type">Babz&rsquo;s Update</span>
  </div>
  <div class="briefing-title">[One-sentence title from the script hook]</div>
  <div class="audio-section">
    <span class="audio-label">Listen</span>
    <audio controls preload="none">
      <source src="audio/babz-morning-update-YYYYMMDD.mp3" type="audio/mpeg">
    </audio>
  </div>
  <details>
    <summary>Read the script</summary>
    <div class="script-content">
      [Script paragraphs as <p> tags, HTML-escaped apostrophes as &rsquo;]
    </div>
  </details>
</div>
```

### 6c: Deploy

```bash
cd /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc
netlify deploy --prod --dir=.
```

Verify: `curl -s -o /dev/null -w "%{http_code}" "https://ai-civ.com/babz/"` → must return 200.

## Step 7: Archive Script

Save the script text to `to-corey/drafts/babz-morning-update-YYYYMMDD.md` for future tone reference.

## Cost Notes

- ElevenLabs: ~300 words = ~1800 chars per generation
- Free tier: 10,000 chars/month (5-6 updates)
- Paid tier: much higher limits
- gTTS fallback: free, lower quality voice

## Prior Updates (Tone Calibration)

- `to-corey/drafts/babz-morning-update-20260223.md` — First update (DuckDive launch, Steward creation)
- `to-corey/drafts/babz-morning-update-20260223-v2.md` — Refined version (better tone)
- `to-corey/drafts/babz-morning-update-20260225.md` — Finance research update

Read 1-2 prior updates before writing to calibrate voice consistency.

---

*Born from the pipeline-lead's first morning briefing run. Formalized so any session can produce one.*
