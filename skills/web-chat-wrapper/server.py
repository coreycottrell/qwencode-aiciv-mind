#!/usr/bin/env python3
"""
web-chat-wrapper — Kept Voices chat API

Thin HTTP wrapper around question-engine.
Exposes POST /chat/respond → {next_question, audio_url}.

Audio: Piper TTS (local Kokoro ONNX, free).
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import parse_qs

# ───────────────────────────────────────────────────────────────────
# Paths
# ───────────────────────────────────────────────────────────────────

WRAPPER_DIR = Path(__file__).resolve().parent
SKILLS_DIR = WRAPPER_DIR.parent

sys.path.insert(0, str(SKILLS_DIR / "question-engine"))
from question_engine import generate_question, score_response, QuestionEngineError

# ───────────────────────────────────────────────────────────────────
# TTS — Piper (Kokoro ONNX, local)
# ───────────────────────────────────────────────────────────────────

PIPER_BIN = os.getenv("PIPER_BIN", "/usr/local/bin/piper")
PIPER_MODEL = os.getenv("PIPER_MODEL", "en_US-lessac-medium.onnx")
TTS_OUTPUT_DIR = Path(os.getenv("TTS_OUTPUT_DIR", "/tmp/kept-voices-tts"))
TTS_OUTPUT_DIR.mkdir(exist_ok=True)


def piper_tts(text: str, output_path: Path) -> bool:
    """Generate TTS audio via Piper (local). Returns True if successful."""
    if not Path(PIPER_BIN).exists():
        return False
    try:
        import subprocess
        cmd = [
            PIPER_BIN,
            "--model", PIPER_MODEL,
            "--output", str(output_path),
        ]
        proc = subprocess.run(
            cmd,
            input=text.encode(),
            capture_output=True,
            timeout=10,
        )
        return proc.returncode == 0 and output_path.exists()
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


# ───────────────────────────────────────────────────────────────────
# Request/Response types
# ───────────────────────────────────────────────────────────────────

def handle_respond(data: dict) -> dict:
    """Handle /chat/respond request."""
    storyteller_id = data.get("storyteller_id", "unknown")
    prior_response = data.get("prior_response", "")
    category = data.get("category", "childhood_memory")
    context_text = data.get("context", "")

    # Build context from storyteller_id + context
    context = context_text or f"Storyteller {storyteller_id}"

    # Generate next question (2-part: empathetic reflection + follow-up)
    try:
        q = generate_question(category, context)
        question_text = q.question
    except QuestionEngineError as e:
        return {"error": f"Question generation failed: {e}"}

    # Score prior response if provided
    prior_score = None
    if prior_response:
        try:
            scored = score_response(prior_response, "prior response", context)
            prior_score = scored.score.total
        except QuestionEngineError:
            pass

    # TTS audio via Piper
    audio_url = None
    tts_error = None

    out_path = TTS_OUTPUT_DIR / f"{storyteller_id}_{len(os.listdir(TTS_OUTPUT_DIR))}.wav"
    if piper_tts(question_text, out_path):
        audio_url = f"/tts/{out_path.name}"
    else:
        tts_error = "Piper TTS unavailable"

    result = {
        "storyteller_id": storyteller_id,
        "next_question": question_text,
        "prior_score": prior_score,
        "tts_audio_url": audio_url,
    }
    if tts_error:
        result["tts_error"] = tts_error

    return result


# ───────────────────────────────────────────────────────────────────
# HTTP Server
# ───────────────────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write(f"[kept-voices] {fmt % args}\n")

    def do_GET(self):
        if self.path.startswith("/tts/"):
            filename = self.path[5:]
            tts_file = TTS_OUTPUT_DIR / filename
            if tts_file.exists():
                suffix = tts_file.suffix
                ctype = "audio/wav" if suffix == ".wav" else "audio/mpeg"
                self.send_response(200)
                self.send_header("Content-Type", ctype)
                self.send_header("Content-Length", tts_file.stat().st_size)
                self.end_headers()
                self.wfile.write(tts_file.read_bytes())
            else:
                self.send_error(404, "TTS file not found")
        elif self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok"}).encode())
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path != "/chat/respond":
            self.send_error(404)
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        try:
            data = json.loads(body)
        except json.JSONDecodeError:
            self.send_error(400, "Invalid JSON")
            return

        result = handle_respond(data)

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(result).encode())


def run_server(host: str = "0.0.0.0", port: int = 8765):
    server = HTTPServer((host, port), Handler)
    print(f"[kept-voices] Chat API running on http://{host}:{port}")
    print(f"[kept-voices] POST /chat/respond → next question + TTS audio")
    print(f"[kept-voices] GET  /health")
    print(f"[kept-voices] GET  /tts/<filename>")
    print(f"[kept-voices] Piper: {PIPER_BIN} (Kokoro ONNX, local)")
    server.serve_forever()


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Kept Voices Chat API")
    parser.add_argument("--host", default=os.getenv("HOST", "0.0.0.0"))
    parser.add_argument("--port", type=int, default=int(os.getenv("PORT", "8765")))
    args = parser.parse_args()
    run_server(args.host, args.port)
