#!/usr/bin/env python3
"""
keptvoices_audio_server.py — Kept Voices Voice Sample Capture + Retrieval

Stores storyteller voice recordings per question and serves concatenated playback.
Meets Mother's Day fast-track spec:

POST /api/keptvoices/voice-sample
  Body: {session_id, chunk_index, mime, audio_blob (base64)}
  Stores: {SESSION_DIR}/{session_id}/voice-samples/chunk_{index}.webm
  Returns: {stored: true, path: relative_path}

GET /v/{storyteller}/{chapter}/audio.mp3
  Serves concatenated audio of all voice samples for that storyteller+chapter
  Requires: session exists, chunks are sequential, ffmpeg available
  Returns: audio/mpeg stream

POST /api/keptvoices/voice-sample/assemble
  Body: {session_id}
  Concatenates all chunks → .mp3 at session level
  Returns: {assembled: true, mp3_path}

Storage layout:
  {SESSION_DIR}/{session_id}/voice-samples/chunk_{i}.webm
  {SESSION_DIR}/{session_id}/audio.mp3  (assembled)
"""

import base64
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from urllib.parse import urlparse

# ───────────────────────────────────────────────────────────────────
# Paths
# ───────────────────────────────────────────────────────────────────

SERVER_DIR = Path(__file__).resolve().parent
KEPTVOICES_API_ROOT = os.getenv("KEPTVOICES_API_ROOT", "/home/corey/projects/AI-CIV/keptvoices-api")
SESSION_DIR = Path(os.getenv("KEPTVOICES_VOICE_DIR", f"{KEPTVOICES_API_ROOT}/state/sessions"))
SESSION_DIR.mkdir(parents=True, exist_ok=True)

PORT = int(os.getenv("VOICE_PORT", "5051"))
FFMPEG_BIN = os.getenv("FFMPEG_BIN", "/usr/bin/ffmpeg")

ALLOWED_ORIGIN = os.getenv("ALLOWED_ORIGIN", "https://ai-civ.com")

# ───────────────────────────────────────────────────────────────────
# CORS
# ───────────────────────────────────────────────────────────────────

def cors_headers(origin: str) -> dict:
    allowed = [
        "https://ai-civ.com",
        "http://localhost:3000",
        "http://localhost:5173",
        "https://keptvoices.com",
    ]
    if origin in allowed:
        return {
            "Access-Control-Allow-Origin": origin,
            "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type",
        }
    return {}


# ───────────────────────────────────────────────────────────────────
# Voice Sample Storage
# ───────────────────────────────────────────────────────────────────

def voice_samples_dir(session_id: str) -> Path:
    d = SESSION_DIR / session_id / "voice-samples"
    d.mkdir(parents=True, exist_ok=True)
    return d


def store_voice_sample(session_id: str, chunk_index: int, mime: str, audio_blob_b64: str) -> dict:
    """Store a voice sample chunk. Returns stored path."""
    samples_dir = voice_samples_dir(session_id)
    ext = _mime_to_ext(mime)
    filename = f"chunk_{chunk_index:04d}.{ext}"
    path = samples_dir / filename

    try:
        audio_bytes = base64.b64decode(audio_blob_b64)
    except Exception:
        return {"error": "Invalid base64 audio blob"}

    try:
        path.write_bytes(audio_bytes)
    except Exception as e:
        return {"error": f"Failed to write audio file: {e}"}

    return {
        "stored": True,
        "path": str(path.relative_to(SESSION_DIR)),
        "size_bytes": len(audio_bytes),
    }


def _mime_to_ext(mime: str) -> str:
    """Map mime type to file extension."""
    mapping = {
        "audio/webm": "webm",
        "audio/opus": "opus",
        "audio/mp4": "mp4",
        "audio/mpeg": "mp3",
        "audio/wav": "wav",
        "audio/ogg": "ogg",
    }
    return mapping.get(mime.lower(), "webm")


# ───────────────────────────────────────────────────────────────────
# Assembly
# ───────────────────────────────────────────────────────────────────

def assemble_session(session_id: str) -> dict:
    """Concatenate all voice sample chunks into a single MP3."""
    samples_dir = voice_samples_dir(session_id)

    # Find all chunks, sorted by index
    chunks = sorted(
        list(samples_dir.glob("chunk_*.webm")) +
        list(samples_dir.glob("chunk_*.opus")) +
        list(samples_dir.glob("chunk_*.mp4")) +
        list(samples_dir.glob("chunk_*.wav")) +
        list(samples_dir.glob("chunk_*.ogg"))
    )

    if not chunks:
        return {"error": f"No chunks found for session {session_id}"}

    # Build concat list file for ffmpeg
    concat_list = samples_dir / "concat_list.txt"
    with open(concat_list, "w") as f:
        for chunk in chunks:
            # Escape paths for ffmpeg concat demuxer
            f.write(f"file '{chunk.resolve()}'\n")

    mp3_path = SESSION_DIR / session_id / "audio.mp3"

    try:
        result = subprocess.run(
            [
                FFMPEG_BIN, "-y",
                "-f", "concat", "-safe", "0",
                "-i", str(concat_list),
                "-c:a", "libmp3lame", "-q:a", "2",
                str(mp3_path)
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )
        concat_list.unlink()  # Clean up concat list

        if result.returncode != 0:
            return {"error": f"ffmpeg failed: {result.stderr}"}

        return {
            "assembled": True,
            "mp3_path": str(mp3_path.relative_to(SESSION_DIR)),
            "chunk_count": len(chunks),
            "size_bytes": mp3_path.stat().st_size,
        }
    except subprocess.TimeoutExpired:
        return {"error": "ffmpeg timeout"}
    except FileNotFoundError:
        return {"error": "ffmpeg not found"}


# ───────────────────────────────────────────────────────────────────
# HTTP Server
# ───────────────────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write(f"[keptvoices-audio] {fmt % args}\n")

    def do_OPTIONS(self):
        origin = self.headers.get("Origin", "")
        self.send_response(200)
        for k, v in cors_headers(origin).items():
            self.send_header(k, v)
        self.end_headers()

    def do_POST(self):
        origin = self.headers.get("Origin", "")
        parsed = urlparse(self.path)

        content_length = int(self.headers.get("Content-Length", 0))
        if content_length == 0:
            self.send_error(400, "Empty body")
            return

        body = self.rfile.read(content_length)
        try:
            data = json.loads(body)
        except json.JSONDecodeError:
            self.send_error(400, "Invalid JSON")
            return

        result = None

        if parsed.path == "/api/keptvoices/voice-sample":
            session_id = data.get("session_id", "")
            chunk_index = int(data.get("chunk_index", 0))
            mime = data.get("mime", "audio/webm")
            audio_blob = data.get("audio_blob", "")

            if not session_id:
                result = {"error": "session_id required"}
            elif not audio_blob:
                result = {"error": "audio_blob (base64) required"}
            else:
                result = store_voice_sample(session_id, chunk_index, mime, audio_blob)

        elif parsed.path == "/api/keptvoices/voice-sample/assemble":
            session_id = data.get("session_id", "")
            if not session_id:
                result = {"error": "session_id required"}
            else:
                result = assemble_session(session_id)

        else:
            self.send_error(404)
            return

        status = 200 if "error" not in result else 400
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        for k, v in cors_headers(origin).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(json.dumps(result).encode())

    def do_GET(self):
        parsed = urlparse(self.path)

        # GET /v/{storyteller}/{chapter}/audio.mp3
        match = re.match(r"^/v/([^/]+)/([^/]+)/audio\.mp3$", parsed.path)
        if match:
            storyteller = match.group(1)
            chapter = match.group(2)

            # Find session by storyteller (first session dir that has storyteller marker)
            session_mp3 = None
            for session_dir in SESSION_DIR.iterdir():
                if not session_dir.is_dir():
                    continue
                # Check for storyteller marker or match session_id pattern
                marker_file = session_dir / ".storyteller"
                mp3_path = session_dir / "audio.mp3"
                if mp3_path.exists():
                    # Verify storyteller matches
                    if marker_file.exists():
                        if marker_file.read_text().strip() == storyteller:
                            session_mp3 = mp3_path
                            break
                    else:
                        # Fall back: session dir name starts with storyteller
                        if session_dir.name.startswith(storyteller):
                            session_mp3 = mp3_path
                            break

            if session_mp3 and session_mp3.exists():
                self.send_response(200)
                self.send_header("Content-Type", "audio/mpeg")
                self.send_header("Content-Length", session_mp3.stat().st_size)
                self.send_header("Content-Disposition", f"inline; filename=\"{storyteller}_{chapter}_audio.mp3\"")
                self.end_headers()
                self.wfile.write(session_mp3.read_bytes())
            else:
                self.send_error(404, "Audio not found")
            return

        # GET /api/keptvoices/voice-sample/list?session_id=XXX
        if parsed.path == "/api/keptvoices/voice-sample/list":
            import urllib.parse
            params = urllib.parse.parse_qs(parsed.query)
            session_id_list = params.get("session_id", [])
            if not session_id_list:
                self.send_error(400, "session_id required")
                return
            session_id = session_id_list[0]
            samples_dir = voice_samples_dir(session_id)
            chunks = sorted(samples_dir.glob("chunk_*"))
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "session_id": session_id,
                "chunk_count": len(chunks),
                "chunks": [c.name for c in chunks],
            }).encode())
            return

        self.send_error(404)


def run():
    print(f"[keptvoices-audio] Voice capture API running on http://0.0.0.0:{PORT}")
    print(f"[keptvoices-audio] POST /api/keptvoices/voice-sample")
    print(f"[keptvoices-audio] POST /api/keptvoices/voice-sample/assemble")
    print(f"[keptvoices-audio] GET  /v/<storyteller>/<chapter>/audio.mp3")
    print(f"[keptvoices-audio] GET  /api/keptvoices/voice-sample/list?session_id=X")
    print(f"[keptvoices-audio] Storage: {SESSION_DIR}")
    print(f"[keptvoices-audio] ffmpeg: {FFMPEG_BIN}")
    HTTPServer(("0.0.0.0", PORT), Handler).serve_forever()


if __name__ == "__main__":
    run()
