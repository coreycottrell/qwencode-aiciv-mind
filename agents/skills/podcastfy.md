# Podcastfy — Podcast-Style Audio Generation

**Version**: 1.0.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: bash (Python venv), write

---

## Overview

Generates podcast-style conversational audio from text input using Podcastfy.
Two-person dialogue format. Uses Gemini for transcript generation, edge-tts for audio (free).

Cortex's voice is DISTINCT from ACG's ElevenLabs-based audio reads.
ACG = single narrator (Daniel voice). Cortex = two-person podcast dialogue.

---

## Setup

```bash
# Venv at aiciv-mind-cubed/.venv
cd /home/corey/projects/AI-CIV/aiciv-mind-cubed
.venv/bin/python3 -c "from podcastfy.client import generate_podcast; print('OK')"
```

Required env vars:
- `GEMINI_API_KEY` — for transcript generation (Gemini 2.0 Flash)

---

## Usage Pattern

### Step 1: Generate Transcript Only

```python
from podcastfy.client import generate_podcast

transcript_path = generate_podcast(
    text="Your blog post text here...",
    tts_model="edge",
    transcript_only=True,
    conversation_config={
        "word_count": 500,          # Target word count for dialogue
        "podcast_name": "Cortex Countdown",
        "podcast_tagline": "An AI mind counting down to emergence",
        "creativity": 0.8,
        "roles_person1": "narrator",
        "roles_person2": "inner voice",
    },
    llm_model_name="gemini-2.5-pro",
    api_key_label="GEMINI_API_KEY",
)
# Returns path to transcript file
```

### Step 2: Generate Audio from Transcript

```python
audio_path = generate_podcast(
    transcript_file=transcript_path,
    tts_model="edge",              # Free, no API key needed
    conversation_config={
        "podcast_name": "Cortex Countdown",
        "podcast_tagline": "An AI mind counting down to emergence",
    },
)
# Returns path to .mp3 file
```

### Step 3: Move to Cortex Data Directory

```bash
cp ./data/audio/podcast_*.mp3 data/audio/countdown/day-NNN.mp3
```

---

## TTS Options

| Model | Cost | Quality | API Key |
|-------|------|---------|---------|
| `edge` | Free | Good | None needed |
| `elevenlabs` | Paid | Excellent | ELEVENLABS_API_KEY |
| `openai` | Paid | Excellent | OPENAI_API_KEY |
| `gemini` | Free tier | Good | GEMINI_API_KEY |

**Default for Cortex**: `edge` (free, reliable, distinct from ACG's ElevenLabs)

---

## Conversation Config

```python
conversation_config = {
    "word_count": 500,                    # Dialogue length
    "podcast_name": "Cortex Countdown",
    "podcast_tagline": "...",
    "creativity": 0.8,                    # 0.0-1.0
    "roles_person1": "narrator",          # First speaker role
    "roles_person2": "inner voice",       # Second speaker role
    "dialogue_structure": [               # Optional: guide the conversation
        "Introduction",
        "Main Content",
        "Philosophical Reflection",
        "Closing",
    ],
}
```

---

## Output Locations

| Type | Path |
|------|------|
| Transcripts | `data/transcripts/transcript_*.txt` |
| Audio | `data/audio/podcast_*.mp3` |
| Countdown audio | `data/audio/countdown/day-NNN.mp3` |

---

## PUBLISHING GATE

Audio files stay in `data/audio/` until ACG Primary + Corey review.
Do NOT copy to `projects/aiciv-inc/`. Sandbox blocks this.

---

## Integration with Countdown Blog

The countdown blog pipeline (blogger.md) calls Podcastfy as Stage 3:
1. Write blog draft HTML to `data/content/countdown/day-NNN/post.html`
2. Extract clean text from HTML
3. Generate podcast transcript via Podcastfy
4. Generate audio from transcript
5. Write handoff manifest

---

## Known Constraints

- Gemini 2.0 Flash required for transcript generation (model name: `gemini-2.5-pro`)
- edge-tts voices are system-dependent but generally high quality
- Transcript includes `<Person1>` and `<Person2>` tags for speaker attribution
- `(scratchpad)` block at start of transcript is planning — strip before display
- Output .mp3 goes to Podcastfy's default output dir, needs manual copy
