---
name: telegram-integration
description: Complete reference for Telegram bot operation, file transfers, voice messages, and troubleshooting
---

# Telegram Integration SKILL

**Purpose**: Complete reference for A-C-Gee's Telegram integration - bot operation, attachment handling, voice messages, and troubleshooting.

**Owner**: tg-archi
**Last Updated**: 2025-12-17

---

## Quick Reference

### Bot Scripts

| Script | Purpose | Command |
|--------|---------|---------|
| `tools/telegram_unified.py` | Main unified bot (text + attachments) | `./tools/start_telegram_bot.sh restart` |
| `tools/telegram-voice/telegram_voice_bridge.py` | Voice-specific bridge (STT/TTS) | `python3 tools/telegram-voice/telegram_voice_bridge.py` |

### Key Locations

| Item | Path |
|------|------|
| Config | `config/telegram_config.json` |
| Attachments | `downloads/telegram_attachments/` |
| Voice Temp | `.tg_voice_temp/` |
| Bot Log | `/tmp/telegram_unified.log` |
| Session Marker | `.current_session` |

---

## Telegram Bot API - File Transfers

### Sending Files TO Telegram

**CRITICAL PATTERN** (Added 2025-12-17):

```python
import httpx
from pathlib import Path

bot_token = "YOUR_BOT_TOKEN"
chat_id = "TARGET_CHAT_ID"
file_path = "/path/to/file.md"

url = f"https://api.telegram.org/bot{bot_token}/sendDocument"

with open(file_path, 'rb') as f:
    files = {'document': ('filename.md', f)}
    data = {'chat_id': chat_id, 'caption': 'Optional caption here'}

    response = httpx.post(url, data=data, files=files, timeout=30)
    print(f"Status: {response.status_code}")
```

**Supported send methods:**
| Method | Use Case | Max Size |
|--------|----------|----------|
| `sendDocument` | Any file type | 50MB |
| `sendPhoto` | Images (compressed) | 10MB |
| `sendVideo` | Videos | 50MB |
| `sendAudio` | Audio files | 50MB |
| `sendVoice` | Voice messages (.ogg) | 50MB |

**Quick send script location:** Can be run inline with Python heredoc (no dependencies beyond httpx)

---

### Downloading Files FROM Telegram

**CRITICAL PATTERN** (Don't lose this again!):

```python
# Step 1: Get file_id from message
# For photos: message["photo"][-1]["file_id"]  # Last = largest
# For documents: message["document"]["file_id"]
# For voice: message["voice"]["file_id"]
# For video: message["video"]["file_id"]

# Step 2: Call getFile API
response = await client.get(f"https://api.telegram.org/bot{TOKEN}/getFile",
                            params={"file_id": file_id})
file_path = response.json()["result"]["file_path"]

# Step 3: Download from file URL
file_url = f"https://api.telegram.org/file/bot{TOKEN}/{file_path}"
content = await client.get(file_url)

# Step 4: Save locally
Path("downloads/filename.ext").write_bytes(content.content)
```

### Attachment Types

| Type | Message Field | file_id Location |
|------|---------------|------------------|
| Photo | `message.photo` | `photos[-1]["file_id"]` (array, last is largest) |
| Document | `message.document` | `document["file_id"]` + `document["file_name"]` |
| Voice | `message.voice` | `voice["file_id"]` |
| Video | `message.video` | `video["file_id"]` + `video.get("file_name")` |
| Audio | `message.audio` | `audio["file_id"]` |
| Sticker | `message.sticker` | `sticker["file_id"]` |

### File Size Limits

- **Download limit**: 20MB per file (Telegram Bot API limitation)
- Larger files need Telegram Premium or alternative approach

---

## Voice Message Flow

Located in `telegram_voice_bridge.py`:

1. **Receive voice** → Download .ogg file
2. **Convert** → ogg → wav (using ffmpeg)
3. **Transcribe** → Google Speech Recognition (free)
4. **Inject** → Send text to Claude via tmux
5. **Response** → Convert text to speech (gTTS or ElevenLabs)
6. **Reply** → Send voice message back to Telegram

### Dependencies

```bash
pip install SpeechRecognition gTTS pydub python-telegram-bot python-dotenv
# Optional: pip install elevenlabs (premium TTS)
```

### ffmpeg Location

```
tools/bin/ffmpeg-7.0.2-amd64-static/ffmpeg
```

---

## Bot Management

### Start/Restart Bot

```bash
./tools/start_telegram_bot.sh restart
```

### Health Check

```bash
./tools/telegram_health_check.sh
```

### Check Logs

```bash
tail -f /tmp/telegram_unified.log
```

### Verify Session Detection

The bot auto-detects ACG tmux sessions matching pattern: `acg-primary-*`

Check current session:
```bash
cat .current_session
```

---

## Configuration

### `config/telegram_config.json`

```json
{
  "bot_token": "YOUR_BOT_TOKEN",
  "chat_id": "YOUR_CHAT_ID",
  "authorized_users": {
    "USER_ID": {
      "name": "Name",
      "role": "creator",
      "admin": true
    }
  }
}
```

### Voice Config (Optional)

`tools/telegram-voice/voice_config.json`:

```json
{
  "tts_provider": "gtts",
  "fallback_to_gtts": true,
  "gtts": {"language": "en"},
  "elevenlabs": {
    "api_key_env": "ELEVENLABS_API_KEY",
    "voice_id": "RHY5GMXg2XfJq73yKR1a"
  }
}
```

---

## Troubleshooting

### Bot not receiving messages

1. Check bot is running: `pgrep -f telegram_unified`
2. Check log for errors: `tail /tmp/telegram_unified.log`
3. Verify tmux session exists: `tmux list-sessions`
4. Restart: `./tools/start_telegram_bot.sh restart`

### Attachments not downloading

1. Check `downloads/telegram_attachments/` exists
2. Check file size (must be <20MB)
3. Check log for "getFile" errors
4. Verify bot token has file access

### Voice transcription failing

1. Check ffmpeg exists: `ls tools/bin/ffmpeg-7.0.2-amd64-static/ffmpeg`
2. Check internet (Google Speech API)
3. Speak clearly, reduce background noise

### Session not detected

1. Verify session marker: `cat .current_session`
2. List tmux sessions: `tmux list-sessions`
3. Bot expects pattern: `acg-primary-YYYYMMDD-HHMMSS`

---

## Implementation History

| Date | Change | Location |
|------|--------|----------|
| 2025-12-17 | Added file SENDING to Telegram | SKILL.md (inline script) |
| 2025-12-17 | Added attachment download support | `telegram_unified.py` |
| 2025-10-XX | Voice bridge created | `telegram_voice_bridge.py` |
| 2025-10-XX | Unified bot created | `telegram_unified.py` |

---

## Related Files

- `.claude/agents/tg-archi.md` - Telegram specialist agent
- `memories/agents/tg-archi/` - Agent memory directory
- `tools/start_telegram_bot.sh` - Bot launcher
- `tools/telegram_health_check.sh` - Health check script

---

**Remember**: This SKILL doc exists so we don't lose knowledge again. When you implement new Telegram features, UPDATE THIS DOC!
