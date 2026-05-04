# web-chat-wrapper — Firing Contract

## WHEN

web-lead (Kept Voices chat UI) calls `POST /chat/respond` to get the next interview question for a storyteller.

## WHAT

`POST /chat/respond` → wraps question-engine.generate_question() + Kokoro TTS.

Request body:
```json
{
  "storyteller_id": "string",
  "prior_response": "string (optional, for scoring)",
  "category": "string (question category)",
  "context": "string (storyteller background)"
}
```

Response:
```json
{
  "storyteller_id": "string",
  "next_question": "string (2-part: empathetic reflection + follow-up question)",
  "prior_score": 14,
  "tts_audio_url": "/tts/file.wav",
  "tts_error": null
}
```

## PRECONDITIONS

1. Local Ollama must be running at `OLLAMA_URL`
2. `OLLAMA_MODEL` must be available in Ollama
3. Piper binary at `PIPER_BIN` (Kokoro ONNX local TTS)

## POSTCONDITIONS

### /chat/respond success:
- `next_question` is non-empty string (2-part spoken narration)
- `storyteller_id` matches input
- `tts_audio_url` present if TTS succeeded, null if failed
- `tts_error` null if TTS succeeded, string if failed

### /health success:
- Returns `{"status": "ok"}` with HTTP 200

## FAILURE MODES

| Failure | Behavior |
|---------|----------|
| Ollama down | HTTP 200 with `{"error": "Question generation failed: ..."}` |
| Piper unavailable | TTS fails silently — `tts_audio_url: null`, `tts_error` set, question still returned |
| Invalid JSON body | HTTP 400 |
| Unknown endpoint | HTTP 404 |

## OBSERVABILITY

- All requests logged to stderr with `[kept-voices]` prefix
- TTS errors surfaced in `tts_error` field (not blocking)
- No persistent logs (stateless per request)

## TTS ARCHITECTURE

Kokoro-only (per Corey directive 2026-05-04). No ElevenLabs, no cloud TTS.
- Piper binary: `PIPER_BIN` (default /usr/local/bin/piper)
- Piper model: `PIPER_MODEL` (default en_US-lessac-medium.onnx)
- Output: WAV format at `/tts/{filename}.wav`
