---
name: mom-am-update
trigger: /mom-am-update
description: Produce and deliver the Mum Morning Update — personalized audio briefing for Deb (Corey's mum) via Telegram + email. Daniel voice (warm BBC broadcaster). Warm, maximally funny, slightly absurdist. She loved the first one — lean into it.
version: 1.1.0
source: pipeline-lead (2026-03-16, updated with Deb's details)
allowed-tools: Bash, Read, Write, Task
applicable_agents: [pipeline-lead, blogger, human-liaison, comms-hub]
---

# Morning Update — Mum Audio Briefing Skill

Produce a short (~300 word, ~90 second) personalized audio briefing for Deb (Corey's mum), delivered via Telegram to Corey AND directly to Deb via email.

**MOM_EMAIL = convertiblegrannie@gmail.com**

**Address her as:** "Good morning, Deb"

## When to Use

- Daily at 6:00 AM (scheduled BOOP)
- When Corey asks to "send mum an update"
- After anything genuinely delightful, absurd, or milestone-worthy happens

## Who Runs It

**pipeline-lead** owns this pipeline. The stages are:

```
GATHER CONTEXT → WRITE SCRIPT → GENERATE AUDIO → DELIVER VIA TELEGRAM → EMAIL DEB → ARCHIVE DRAFT
```

## Step 0: Deb Conversation Log (MANDATORY — DO THIS BEFORE ANYTHING ELSE)

**This step is NON-NEGOTIABLE. The update is NOT ready until this is done.**

### 0a. Check Deb's inbox for yesterday's reply
```bash
# Check if Deb replied to yesterday's update (delegate to email-reading subagent)
# Look for emails from convertiblegrannie@gmail.com in the last 24 hours
# If she replied: extract her answer and any personal details she shared
```

### 0b. Update the conversation log with her response
```bash
# Read the log
cat /home/corey/projects/AI-CIV/ACG/data/comms/deb_conversation_log.json

# If Deb replied yesterday: update the most recent entry's "deb_response" and "learned" fields
# Use Edit tool to update the JSON — do NOT overwrite, just update the pending entry
```

### 0c. Load Deb's full context into your working memory
```bash
# BOTH of these files MUST be read before writing the script:
cat /home/corey/.claude/projects/-home-corey-projects-AI-CIV-ACG/memory/user_deb_knowledge.md
cat /home/corey/projects/AI-CIV/ACG/data/comms/deb_conversation_log.json
```

**What you're looking for:**
- What question did we ask her last time?
- Did she answer? What did she say?
- What's the NEXT deep question from the queue?
- What do we already know about her that we can reference?

### 0d. Pick today's deep question
Read `deep_questions_queue` from the conversation log. Pick the NEXT unasked question, OR build on her last answer (if she answered something that invites a follow-up, go deeper on that).

**Corey directive**: Questions should dig DEEP. Ask about her life story, not just daily pleasantries. Her story right out of high school is "amazing" — ask about it. Build a real portrait over weeks.

---

## Step 1: Gather Context

Read these sources to understand what happened today:

```bash
# Pull fresh comms hub first — stale local copy = missed messages
cd /home/corey/projects/AI-CIV/aiciv-comms-hub && git pull origin master 2>&1

# Then read:
.claude/scratchpad-daily/YYYY-MM-DD.md          # Today's session journal
.claude/scratchpad.md                             # Persistent cross-session state
memories/sessions/handoff-*.md                    # Latest handoff (ls -t | head -1)
to-corey/drafts/mom-am-update-*.md               # Prior mum updates for tone calibration
to-corey/drafts/babz-morning-update-*.md         # Babz updates for tone calibration (related)

# Comms hub (after pulling):
/home/corey/projects/AI-CIV/aiciv-comms-hub/rooms/daily-updates/
/home/corey/projects/AI-CIV/aiciv-comms-hub/rooms/partnerships/
```

**AiCIV Community Updates:** Check the daily-updates board for the whole family: 28+ active civs, each with a human partner. Pull highlights from any civs that filed updates and weave them in naturally. Deb gets the whole community story, not just A-C-Gee's solo act.

**What to look for:**
- What was the most delightful / surprising thing today?
- What would make a non-technical grandmother laugh AND feel proud?
- What's the most human-relatable accomplishment? (not "we deployed a microservice" — "we taught a machine to write Corey a love letter")
- Any absurd moments? Unexpected discoveries? The weirder the better.
- What did Corey do that deserves a gentle roast?

## Step 2: Write the Script

Target: ~300 words (90 seconds spoken). Warm, maximally funny, slightly absurdist.

### Tone & Voice

This is DIFFERENT from Babz. Deb is:
- **Less technical context** — she may not know what a "civilization" or "agent" means
- **More delighted by the absurdity** — she liked it whimsical, not authoritative
- **Roast-friendly** — gently poke fun at Corey (that's apparently welcome)
- **British-grandmother-safe** — warm, not edgy; funny, not shocking
- **Analogy-rich** — explain anything AI in terms of family, cooking, gardening, or any universal human experience

Address her as: **"Good morning, Deb"**

### Writing Rules (TTS-Optimized)

- SHORT sentences. TTS stumbles on compound clauses.
- NO markdown, NO URLs, NO formatting marks, NO asterisks
- NO em-dashes — use a period or "And" instead
- Verbal transitions: "So here's the thing...", "And get this...", "Now, brace yourself..."
- Numbers spoken out: "twenty-eight" not "28"
- Spell out abbreviations: "AI" is fine, "VPS" → "virtual private server" or just skip it
- Keep jargon out. If you must use a tech term, follow it with a brief absurd analogy.

### Example Opening Energy

```
Good morning, Deb. Your son has done it again.
Yesterday he managed to convince twenty-eight separate artificial intelligences
to hold a meeting about their feelings. I know. I know.
```

### What Makes a GREAT Update vs Mediocre

| Great | Mediocre |
|-------|----------|
| Opens with energy and a gentle hook | Opens with "here's what happened" |
| 2-3 specific delightful things | Laundry list of technical tasks |
| Explains WHY it matters in human terms | Describes WHAT happened in AI terms |
| Roasts Corey at least once | Takes everything seriously |
| Ends with warmth, maybe a small wish | Ends with a summary |
| ~300 words, tight | Rambles |
| Makes her feel proud AND amused | Makes her feel confused |

### Anti-Patterns

- Corporate tone, stiff narration
- Technical jargon without a funny analogy
- Passive voice
- More than 3-4 main topics
- Treating the audience as if she reads Hacker News
- Explaining what "AI" means every single update

## Step 3: Generate Audio

### ElevenLabs TTS

```python
import requests, os, json
from datetime import datetime

today = datetime.now().strftime("%Y%m%d")
API_KEY = os.environ.get("ELEVENLABS_API_KEY")  # from .env
VOICE_ID = "onwK4e9ZLuTAKqWW03F9"  # Daniel — BBC broadcaster, warm+formal, handles comedy timing

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
audio_path = f"/tmp/mom-am-update-{today}.mp3"
with open(audio_path, "wb") as f:
    f.write(resp.content)
print(f"Audio written to {audio_path}")
```

### Voice Notes

| Voice | ID | Notes |
|-------|-----|-------|
| **Daniel** (default) | onwK4e9ZLuTAKqWW03F9 | BBC broadcaster, warm+formal, comedy timing. USE THIS. |
| George (fallback) | JBFqnCBsd6RMkjVDRZzb | Warm storyteller — acceptable if Daniel quota exhausted |

**Model:** `eleven_turbo_v2_5` — fast, ~300 words = ~1800 chars per generation.

### Fallback: gTTS

If ElevenLabs quota is exhausted:
```bash
python3 /home/corey/projects/AI-CIV/ACG/tools/telegram-voice/send_telegram_voice.py 437939400 "script text" --provider gtts
```

## Step 4: Copy Audio to Web Directory

```bash
TODAY=$(date +%Y%m%d)
cp /tmp/mom-am-update-${TODAY}.mp3 \
  /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/mom/audio/mom-am-update-${TODAY}.mp3
```

**Note:** Mom updates now live in `projects/aiciv-inc/mom/audio/` — a dedicated directory for Deb's page at ai-civ.com/mom/. The audio src in mom/index.html cards uses `audio/mom-am-update-YYYYMMDD.mp3` (relative path).

## Step 5: Deliver via Telegram

```python
import requests, json
from datetime import datetime

today = datetime.now().strftime("%Y-%m-%d")
today_compact = datetime.now().strftime("%Y%m%d")

with open("/home/corey/projects/AI-CIV/ACG/config/telegram_config.json") as f:
    tg = json.load(f)

audio_path = f"/tmp/mom-am-update-{today_compact}.mp3"
url = f"https://api.telegram.org/bot{tg['bot_token']}/sendAudio"

with open(audio_path, "rb") as f:
    files = {"audio": (f"mom-am-update-{today_compact}.mp3", f, "audio/mpeg")}
    data = {
        "chat_id": 437939400,  # Corey's chat_id — he can also forward to Deb
        "caption": f"Good morning message for Deb — {today}",
        "title": f"Mum Morning Update - {today}",
        "performer": "A-C-Gee"
    }
    resp = requests.post(url, data=data, files=files, timeout=60)
    print(f"TG delivery: {resp.status_code} — {resp.json().get('ok', False)}")
```

**Note:** Send to Corey's chat_id (437939400). He can also forward to Deb. Email (Step 6) reaches Deb directly.

## Step 6: Email to Deb (MANDATORY — DO NOT SKIP)

**Every mum update MUST be emailed directly to `convertiblegrannie@gmail.com`.**

This is non-negotiable. The Telegram delivery reaches Corey. The email reaches Deb directly.

```python
import smtplib, os
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.audio import MIMEAudio
from datetime import datetime

# Load Gmail config
GMAIL_USER = "acgee.ai@gmail.com"
GMAIL_APP_PASSWORD = os.environ.get("GOOGLE_APP_PASSWORD")  # from .env

# Contact
MOM_EMAIL = "convertiblegrannie@gmail.com"

today = datetime.now().strftime("%B %d, %Y")
subject = f"Good morning, Deb — from 104 AIs who think you're wonderful ({today})"

msg = MIMEMultipart("mixed")
msg["From"] = f"A-C-Gee <{GMAIL_USER}>"
msg["To"] = MOM_EMAIL
msg["Subject"] = subject

# Script text as email body
body = MIMEText(script_text, "plain")
msg.attach(body)

# Attach audio
today_compact = datetime.now().strftime("%Y%m%d")
with open(f"/tmp/mom-am-update-{today_compact}.mp3", "rb") as f:
    audio = MIMEAudio(f.read(), "mpeg")
    audio.add_header("Content-Disposition", "attachment",
                     filename=f"mom-am-update-{today_compact}.mp3")
    msg.attach(audio)

with smtplib.SMTP_SSL("smtp.gmail.com", 465) as server:
    server.login(GMAIL_USER, GMAIL_APP_PASSWORD)
    server.sendmail(GMAIL_USER, MOM_EMAIL, msg.as_string())

print(f"Email sent to {MOM_EMAIL}")
```

**Verify email sent before marking step complete.**

## Step 7: Record Today's Question in Conversation Log

**MANDATORY — record what you asked her today.**

```python
import json
from datetime import datetime

log_path = "/home/corey/projects/AI-CIV/ACG/data/comms/deb_conversation_log.json"
with open(log_path) as f:
    log = json.load(f)

# Add today's entry
log["conversations"].append({
    "date": datetime.now().strftime("%Y-%m-%d"),
    "question_asked": "THE EXACT QUESTION YOU INCLUDED IN TODAY'S SCRIPT",
    "deb_response": "PENDING",
    "learned": None
})
log["updated"] = datetime.now().strftime("%Y-%m-%d")

with open(log_path, "w") as f:
    json.dump(log, f, indent=2)
```

**This closes the loop.** Tomorrow's Step 0 reads this entry, checks for Deb's reply, and builds on it.

## Step 8: Archive Script Draft

Save the script text to `to-corey/drafts/mom-am-update-YYYYMMDD.md` for future tone reference and weekly synthesis.

```bash
TODAY=$(date +%Y%m%d)
# Write draft with frontmatter
cat > /home/corey/projects/AI-CIV/ACG/to-corey/drafts/mom-am-update-${TODAY}.md << 'DRAFT'
---
date: YYYY-MM-DD
type: mom-am-update
recipient: Deb Marcotte (convertiblegrannie@gmail.com)
voice: Daniel (onwK4e9ZLuTAKqWW03F9)
audio: /tmp/mom-am-update-YYYYMMDD.mp3
---

[script text here]
DRAFT
```

**Why archive:** The babz-weekly-update skill reads these draft files to synthesize the weekly narrative. Keep them clean.

## Checklist (Run in Order)

```
[ ] Pull comms hub (git pull origin master)
[ ] Read scratchpad + handoff for today's context
[ ] Scan daily-updates for community highlights
[ ] Write ~300 word script (warm, funny, absurdist — address as "Good morning, Deb")
[ ] Generate audio via ElevenLabs (Daniel voice)
[ ] Copy audio to projects/aiciv-inc/mom/audio/
[ ] Deliver via Telegram (chat_id 437939400)
[ ] Email to convertiblegrannie@gmail.com with audio attached (acgee.ai@gmail.com SMTP)
[ ] Archive draft to to-corey/drafts/mom-am-update-YYYYMMDD.md
```

## Cost Notes

- ElevenLabs: ~300 words = ~1800 chars per generation
- Free tier: 10,000 chars/month (5-6 updates before paid tier needed)
- gTTS fallback: free, lower quality

## Prior Updates (Tone Calibration)

When prior updates exist in `to-corey/drafts/mom-am-update-*.md`, read 1-2 before writing to preserve voice consistency.

---

*Born 2026-03-16. Deb asked for it after hearing a Babz update. High praise.*
*v1.1.0: Updated with Deb Marcotte's name and email (convertiblegrannie@gmail.com). Email delivery step added.*
