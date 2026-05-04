# web-chat-wrapper — Firing Contract

## WHEN

web-lead (Kept Voices chat UI) calls `POST /chat/respond` to get the next interview question for a storyteller.

## WHAT

`POST /chat/respond` → wraps question-engine.generate_question() + optional TTS.

Request body:
```json
{
  "storyteller_id": "string",
  "prior_response": "string (optional, for scoring)",
  "category": "string (question category)",
  "context": "string (storyteller background)",
  "paid_tier": false
}
```

Response:
```json
{
  "storyteller_id": "string",
  "next_question": "string (non-empty)",
  "prior_score": 14,
  "tts_audio_url": "/tts/file.wav",
  "tts_error": null
}
```

## PRECONDITIONS

1. Local Ollama must be running at `OLLAMA_URL`
2. `INTERVIEW_MODEL` must be available in Ollama
3. At least one TTS engine must be available:
   - Free: Piper binary at `PIPER_BIN`
   - Paid: `ELEVENLABS_API_KEY` set

## POSTCONDITIONS

### /chat/respond success:
- `next_question` is non-empty string
- `storyteller_id` matches input
- `tts_audio_url` present if TTS succeeded, null if failed
- `tts_error` null if TTS succeeded, string if failed

### /health success:
- Returns `{"status": "ok"}` with HTTP 200

## FAILURE MODES

| Failure | Behavior |
|---------|----------|
| Ollama down | HTTP 200 with `{"error": "Question generation failed: ..."}` |
| No TTS engine | TTS fails silently — `tts_audio_url: null`, `tts_error` set, question still returned |
| Invalid JSON body | HTTP 400 |
| Unknown endpoint | HTTP 404 |

## OBSERVABILITY

- All requests logged to stderr with `[kept-voices]` prefix
- TTS errors surfaced in `tts_error` field (not blocking)
- No persistent logs (stateless per request)
