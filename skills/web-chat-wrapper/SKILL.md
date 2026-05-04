---
name: web-chat-wrapper
description: Kept Voices HTTP API — thin wrapper around question-engine. POST /chat/respond returns next question text + optional TTS audio URL. Free tier uses Piper TTS (local), paid tier uses ElevenLabs Flash.
version: 0.1.0
author: Hengshi (built for Kept Voices web chat UI)
license: MIT
metadata:
  hengshi:
    tags: [web-api, chat, TTS, kept-voices, fastapi-alternative]
    related_skills: [question-engine, chapter-summarizer]
    applicable_civs: [hengshi]
---

# web-chat-wrapper — Kept Voices Chat API

## What It Is

Thin HTTP server wrapping question-engine for web-chat UI consumption.

**Core endpoint**: `POST /chat/respond`

**Request**:
```json
{
  "storyteller_id": "smith-family-001",
  "prior_response": "I remember my grandmother's kitchen...",
  "category": "childhood_memory",
  "context": "Born 1942, rural Ohio. Grandmother baked every Sunday.",
  "paid_tier": false
}
```

**Response**:
```json
{
  "storyteller_id": "smith-family-001",
  "next_question": "Can you describe a particularly memorable Sunday afternoon...?",
  "prior_score": 14,
  "tts_audio_url": "/tts/smith-family-001_0.wav",
  "tts_error": null
}
```

## Architecture

```
web-lead (chat UI)
    ↓ POST /chat/respond
web-chat-wrapper (this skill)
    ↓ generate_question()
question-engine
    ↓
next_question + TTS
    ↓
{tts_audio_url}
```

## TTS Tiers

| Tier | TTS Engine | Quality | Latency |
|------|-----------|---------|---------|
| Free | Piper (local, ONNX) | Medium | ~2s |
| Paid | ElevenLabs Flash | High | ~1s |

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/chat/respond` | Main API — returns next question + optional TTS |
| GET | `/health` | Health check |
| GET | `/tts/{filename}` | Serve generated TTS audio |

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `OLLAMA_URL` | `http://localhost:11434` | Local Ollama (passed to question-engine) |
| `INTERVIEW_MODEL` | `hermes3:8b-llama3.1-q8_0` | Question generation model |
| `PIPER_BIN` | `/usr/local/bin/piper` | Path to Piper binary |
| `PIPER_MODEL` | `en_US-lessac-medium.onnx` | Piper voice model |
| `ELEVENLABS_API_KEY` | (none) | ElevenLabs key for paid tier |
| `ELEVENLABS_VOICE_ID` | (default Kept Voices voice) | ElevenLabs voice ID |
| `TTS_OUTPUT_DIR` | `/tmp/kept-voices-tts` | Where TTS files are written |
| `HOST` | `0.0.0.0` | HTTP server host |
| `PORT` | `8765` | HTTP server port |

## Firing Contract

| Field | Value |
|-------|-------|
| **WHEN** | web-lead calls POST /chat/respond from Kept Voices chat UI |
| **WHAT** | Wraps question-engine.generate_question() + optional TTS generation |
| **PRECONDITIONS** | Local Ollama running, TTS engine available (Piper or ElevenLabs) |
| **POSTCONDITIONS** | Returns next_question (non-empty), optional tts_audio_url |
| **FAILURE MODES** | Ollama down → JSON error. TTS fail → tts_error field set, question still returned |
| **OBSERVABILITY** | All requests logged to stderr |

## Running

```bash
# Free tier (Piper)
python3 server.py --port 8765

# Paid tier (ElevenLabs)
ELEVENLABS_API_KEY=sk_xxx python3 server.py --port 8765
```

## Co-use

This skill pairs with:
- **`question-engine`**: Core AI brain — web-chat-wrapper provides HTTP API surface
- **`chapter-summarizer`**: After session ends, chapter-summarizer processes Q&A into chapter draft

**Pre-condition**: question-engine deps available (local Ollama)
**Post-condition**: TTS audio files written to TTS_OUTPUT_DIR (serve via /tts/ endpoint)
