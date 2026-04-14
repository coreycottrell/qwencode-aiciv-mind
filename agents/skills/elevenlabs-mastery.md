# ElevenLabs Mastery — Cortex TTS Skill

**Version**: 1.0.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: tts_speak, tts_voices

---

## Overview

Every Cortex mind has access to text-to-speech via the ElevenLabsInterceptor. This skill documents the API, voice routing, and best practices.

---

## Tool Reference

### `tts_speak`

Convert text to speech. Generates MP3 audio.

**Parameters:**

| Param | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `text` | string | yes | — | Text to speak (max 5000 chars) |
| `voice` | string | no | "Daniel" | Voice name, civ name, or ElevenLabs voice ID |
| `filename` | string | no | "tts-output.mp3" | Output filename |
| `model` | string | no | "eleven_turbo_v2_5" | ElevenLabs model ID |

**Voice shortcuts:**

| Input | Resolves To |
|-------|-------------|
| `Daniel`, `acg`, `a-c-gee` | Daniel (onwK4e9ZLuTAKqWW03F9) |
| `Adam`, `true-bearing`, `tb` | Adam (pNInz6obpgDQGcFmaJgB) |
| `Matilda`, `witness` | Matilda (XrExE9yKIg1WjnnlVkGX) |
| Any 20+ char string | Treated as raw ElevenLabs voice ID |

**Models:**

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `eleven_turbo_v2_5` | Fast | Good | Lower |
| `eleven_monolingual_v1` | Medium | Great | Higher |
| `eleven_multilingual_v2` | Slow | Best | Highest |

**Example call:**
```json
{
  "text": "Good morning. Here is your daily intelligence briefing.",
  "voice": "Daniel",
  "filename": "morning-briefing.mp3",
  "model": "eleven_turbo_v2_5"
}
```

**Returns:**
```
Audio generated successfully.
Path: /path/to/data/audio/morning-briefing.mp3
Voice: Daniel (onwK4e9ZLuTAKqWW03F9)
Model: eleven_turbo_v2_5
Size: 42KB
Characters: 55
```

### `tts_voices`

List all available voices. No parameters.

**Returns:** Voice names, IDs, and descriptions.

---

## Best Practices

### Preparing Text for Speech

1. **Clean the text**: Remove markdown, HTML, URLs, code blocks
2. **Short sentences**: TTS handles 10-20 word sentences best
3. **Spell out**: "A-C-Gee" not "ACG", "fifty seven" not "57"
4. **Natural pacing**: Use commas and periods for pauses
5. **Verbal transitions**: "So here's the thing..." not "Furthermore,"
6. **Pronunciation hints**: "ai-civ dot com" not "ai-civ.com"

### Blog Post Audio Pipeline

1. Extract clean text from HTML
2. Detect author → select voice
3. Optimize text for speech (spell out, simplify)
4. Generate with `tts_speak`
5. Verify file size > 100KB
6. Embed `<audio>` tag in HTML
7. Deploy

### Cost Management

- Free tier: ~10,000 chars/month
- A 1000-word blog post ≈ 6000 characters
- Keep casual messages under 200 words (1200 chars)
- Use `eleven_turbo_v2_5` unless quality is critical

---

## ElevenLabs API Reference (for debugging)

**Endpoint:** `POST https://api.elevenlabs.io/v1/text-to-speech/{voice_id}`

**Headers:**
- `xi-api-key: {ELEVENLABS_API_KEY}`
- `Content-Type: application/json`

**Body:**
```json
{
  "text": "...",
  "model_id": "eleven_turbo_v2_5",
  "voice_settings": {
    "stability": 0.5,
    "similarity_boost": 0.75,
    "style": 0.3,
    "use_speaker_boost": true
  }
}
```

**Response:** Raw MP3 audio bytes (Content-Type: audio/mpeg)

**Error codes:**
- 401: Invalid API key
- 422: Text too long or invalid parameters
- 429: Rate limited (wait and retry)

---

## Architecture

The `ElevenLabsInterceptor` lives in `codex-suite-client` and is wired into the CompositeInterceptor alongside Hub, Search, Delegation, and other interceptors. It is available to all roles (Primary, TeamLead, Agent).

**Source:** `src/codex-suite-client/src/elevenlabs_interceptor.rs`
**Wiring:** `src/cortex/src/main.rs` (CompositeInterceptor chain)
