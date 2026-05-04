#!/usr/bin/env python3
"""
keptvoices_chat_server.py — Kept Voices Chat API

Stateless HTTP server wrapping question-engine + chapter-summarizer.
Meets canonical contract from ACG:

POST /chat/respond
  Body: {session_id, last_response, question_count, category}
  Returns: {next_question_text, next_question_audio_url, is_final, question_count, session_id}

POST /chat/finalize
  Body: {session_id, all_qa_pairs}
  Returns: {chapter_preview: {title, body_markdown, word_count, key_quotes, characters, timeline_hint},
            upgrade_cta: {headline, subhead, url}}

Audio convention: /keptvoices/audio/seed-{category}-q{n}.mp3
"""

import json
import os
import sys
import uuid
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from urllib.parse import parse_qs

# ───────────────────────────────────────────────────────────────────
# Paths
# ───────────────────────────────────────────────────────────────────

SERVER_DIR = Path(__file__).resolve().parent
SKILLS_DIR = SERVER_DIR.parent / "skills"
AUDIO_DIR = SERVER_DIR.parent / "public" / "keptvoices" / "audio"
AUDIO_DIR.mkdir(parents=True, exist_ok=True)

sys.path.insert(0, str(SKILLS_DIR / "question-engine"))
sys.path.insert(0, str(SKILLS_DIR / "chapter-summarizer"))

from question_engine import generate_question, score_response, LLM_BACKEND
from chapter_summarizer import generate_chapter, ChapterSummarizerError, InterviewQA

# ───────────────────────────────────────────────────────────────────
# Config
# ───────────────────────────────────────────────────────────────────

SESSION_DIR = Path(os.getenv("SESSION_DIR", "/tmp/keptvoices-sessions"))
SESSION_DIR.mkdir(exist_ok=True)

ALLOWED_ORIGIN = os.getenv("ALLOWED_ORIGIN", "https://ai-civ.com")
PORT = int(os.getenv("PORT", "5050"))
FREE_TIER_MAX = int(os.getenv("FREE_TIER_MAX", "7"))

PIPER_BIN = os.getenv("PIPER_BIN", "/usr/local/bin/piper")
PIPER_MODEL = os.getenv("PIPER_MODEL", "en_US-lessac-medium.onnx")

PAID_UPGRADE_URL = os.getenv("PAID_UPGRADE_URL", "https://keptvoices.com/upgrade")

# ───────────────────────────────────────────────────────────────────
# Session State (JSONL per session)
# ───────────────────────────────────────────────────────────────────

def session_path(session_id: str) -> Path:
    return SESSION_DIR / f"{session_id}.jsonl"

def load_session(session_id: str) -> list[dict]:
    path = session_path(session_id)
    if not path.exists():
        return []
    records = []
    for line in path.read_text().splitlines():
        if line.strip():
            records.append(json.loads(line))
    return records

def append_qa(session_id: str, question: str, response: str):
    path = session_path(session_id)
    record = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "question": question,
        "response": response,
    }
    with open(path, "a") as f:
        f.write(json.dumps(record) + "\n")

def session_q_count(session_id: str) -> int:
    """Count of non-empty Q&A pairs in session."""
    return sum(1 for qa in load_session(session_id) if qa.get("response", "").strip())

# ───────────────────────────────────────────────────────────────────
# Storyteller History Recovery
# ───────────────────────────────────────────────────────────────────

def summarize_storyteller_history(storyteller_id: str, max_chars: int = 4000) -> str:
    """Summarize a returning storyteller's prior Q&A sessions.

    Loads all sessions for this storyteller from SESSION_DIR,
    LLM-summarizes the Q&A history, returns a context string.
    Returns empty string if no prior sessions found or summarization fails.
    """
    import urllib.request
    import urllib.error

    # Find all session files for this storyteller
    sessions_for_storyteller = []
    if SESSION_DIR.exists():
        for f in SESSION_DIR.glob(f"{storyteller_id}-*.jsonl"):
            sessions_for_storyteller.append(f)
        # Also check session files whose first record contains this storyteller_id
        for f in SESSION_DIR.glob("*.jsonl"):
            if f.name.startswith(storyteller_id):
                continue
            try:
                content = f.read_text()
                if storyteller_id in content:
                    sessions_for_storyteller.append(f)
            except (IOError, OSError):
                pass

    if not sessions_for_storyteller:
        return ""

    # Load Q&A pairs from all sessions
    all_qa = []
    for f in set(sessions_for_storyteller):
        try:
            for line in f.read_text().splitlines():
                if line.strip():
                    rec = json.loads(line)
                    if rec.get("response", "").strip():
                        all_qa.append(rec)
        except (json.JSONDecodeError, IOError, OSError):
            pass

    if not all_qa:
        return ""

    # Build concatenated Q&A context
    qa_context = ""
    for i, qa in enumerate(all_qa[:20]):  # cap at 20 Q&As
        qa_context += f"\n--- Prior Q{i+1} ---\nQ: {qa.get('question', '')[:200]}\n"
        qa_context += f"A: {qa.get('response', '')[:300]}\n"

    if len(qa_context) > max_chars:
        qa_context = qa_context[:max_chars] + "\n...[truncated]..."

    prompt = f"""You are producing a brief narrative summary of a storyteller's prior interview sessions for the Kept Voices family storytelling archive.

Prior Q&A history for this storyteller:
{qa_context}

Produce a 2-3 sentence narrative summary covering:
- What topics/themes have been discussed
- Any key characters, events, or memories mentioned
- The emotional tone of prior sessions

NARRATIVE SUMMARY:"""

    # Use local Ollama for summarization (no API key needed)
    ollama_url = os.getenv("OLLAMA_URL", "http://localhost:11434")
    ollama_model = os.getenv("OLLAMA_MODEL", "hermes3:8b-llama3.1-q8_0")
    minimax_api_key = os.getenv("MINIMAX_API_KEY", "")

    try:
        if LLM_BACKEND == "minimax" and minimax_api_key:
            minimax_base_url = os.getenv("MINIMAX_BASE_URL", "https://api.minimax.io/anthropic")
            minimax_model = os.getenv("MINIMAX_MODEL", "MiniMax-M2.7")
            payload = {
                "model": minimax_model,
                "max_tokens": 2000,
                "messages": [{"role": "user", "content": prompt}],
            }
            data = json.dumps(payload).encode()
            headers = {
                "Content-Type": "application/json",
                "x-api-key": minimax_api_key,
                "anthropic-version": "2023-06-01",
            }
            req = urllib.request.Request(
                f"{minimax_base_url}/v1/messages",
                data=data,
                headers=headers,
            )
        else:
            payload = {
                "model": ollama_model,
                "prompt": prompt,
                "stream": False,
            }
            data = json.dumps(payload).encode()
            req = urllib.request.Request(
                f"{ollama_url}/api/generate",
                data=data,
                headers={"Content-Type": "application/json"},
            )

        with urllib.request.urlopen(req, timeout=60) as resp:
            result = json.loads(resp.read())

        if LLM_BACKEND == "minimax" and minimax_api_key:
            content = result.get("content", [{}])
            if content and isinstance(content, list):
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "text":
                        summary = block.get("text", "")
                        break
                else:
                    summary = ""
            else:
                summary = ""
        else:
            summary = result.get("response", "")

        return summary.strip() if summary else ""
    except (urllib.error.URLError, json.JSONDecodeError, KeyError, IndexError):
        return ""

# ───────────────────────────────────────────────────────────────────
# Audio URL convention
# ───────────────────────────────────────────────────────────────────

def seed_audio_url(category: str, q_num: int) -> str:
    """Return seed audio URL per convention: /keptvoices/audio/seed-{category}-q{n}.mp3"""
    safe_cat = category.replace("_", "-")
    return f"/keptvoices/audio/seed-{safe_cat}-q{q_num}.mp3"

# ───────────────────────────────────────────────────────────────────
# CORS
# ───────────────────────────────────────────────────────────────────

def cors_headers(origin: str) -> dict:
    allowed = [
        "https://ai-civ.com",
        "http://localhost:3000",
        "http://localhost:5173",
    ]
    if origin in allowed:
        return {
            "Access-Control-Allow-Origin": origin,
            "Access-Control-Allow-Methods": "POST, OPTIONS, GET",
            "Access-Control-Allow-Headers": "Content-Type",
        }
    return {}

# ───────────────────────────────────────────────────────────────────
# /chat/respond
# ───────────────────────────────────────────────────────────────────

def handle_respond(data: dict) -> dict:
    session_id = data.get("session_id") or str(uuid.uuid4())
    storyteller_id = data.get("storyteller_id", session_id.split("-")[0])
    last_response = data.get("last_response", "")
    question_count = int(data.get("question_count", 0))
    category = data.get("category", "childhood_memory")

    # Save prior Q&A if there's a last_response
    if last_response and question_count > 0:
        session = load_session(session_id)
        if session:
            last_q = session[-1]["question"]
            append_qa(session_id, last_q, last_response)
            question_count = session_q_count(session_id)
        else:
            append_qa(session_id, "", last_response)
    elif question_count == 0 and not last_response:
        # First question — just create session marker
        pass

    # Build context: prior session summary for returning storytellers
    prior_summary = ""
    if question_count == 0:
        # First question of a new session — try to recover storyteller history
        prior_summary = summarize_storyteller_history(storyteller_id)

    context_parts = [f"Storyteller session. Prior questions answered: {question_count}"]
    if prior_summary:
        context_parts.append(f"Prior session summary: {prior_summary}")
    context = " ".join(context_parts)

    try:
        q = generate_question(category, context)
        next_question_text = q.question
    except Exception as e:
        return {"error": str(e)}

    # Determine is_final: free tier cap OR natural conclusion
    is_final = (question_count + 1) >= FREE_TIER_MAX

    # Audio URL — seed convention
    next_audio_url = seed_audio_url(category, question_count + 1)

    # Score prior response
    prior_score = None
    if last_response:
        try:
            scored = score_response(
                last_response,
                "prior response",
                f"Storyteller {session_id}",
            )
            prior_score = scored.score.total
        except Exception:
            pass

    return {
        "next_question_text": next_question_text,
        "next_question_audio_url": next_audio_url,
        "is_final": is_final,
        "question_count": question_count + 1,
        "session_id": session_id,
        "prior_score": prior_score,
    }

# ───────────────────────────────────────────────────────────────────
# /chat/finalize
# ───────────────────────────────────────────────────────────────────

def handle_finalize(data: dict) -> dict:
    session_id = data.get("session_id", "")
    all_qa_pairs = data.get("all_qa_pairs", [])

    if not session_id:
        return {"error": "session_id required"}

    # Load from session file or use provided
    if all_qa_pairs:
        qa_list = all_qa_pairs
    else:
        session = load_session(session_id)
        qa_list = [qa for qa in session if qa.get("response", "").strip()]

    if not qa_list:
        return {"error": "No Q&A pairs provided"}

    qa_pairs = [
        InterviewQA(
            question=qa.get("question", ""),
            response=qa.get("response", ""),
            timestamp=qa.get("ts", ""),
        )
        for qa in qa_list
    ]

    try:
        draft = generate_chapter(
            qa_pairs,
            chapter_theme="Family Storytelling",
            session_id=session_id,
        )
        return {
            "chapter_preview": {
                "title": draft.title,
                "body_markdown": f"# {draft.title}\n\n{draft.narrative_arc}",
                "word_count": len(draft.narrative_arc.split()),
                "key_quotes": [q for s in draft.sections for q in s.key_quotes[:2]],
                "characters": draft.characters[:5],
                "timeline_hint": draft.timeline_span,
            },
            "upgrade_cta": {
                "headline": "Your story is just beginning",
                "subhead": "Upgrade to unlock unlimited chapters, voice archive, and family collaboration.",
                "url": PAID_UPGRADE_URL,
            },
        }
    except ChapterSummarizerError as e:
        return {"error": f"Chapter generation failed: {e}"}
    except Exception as e:
        return {"error": str(e)}

# ───────────────────────────────────────────────────────────────────
# HTTP Server
# ───────────────────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write(f"[keptvoices] {fmt % args}\n")

    def do_OPTIONS(self):
        origin = self.headers.get("Origin", "")
        self.send_response(200)
        for k, v in cors_headers(origin).items():
            self.send_header(k, v)
        self.end_headers()

    def do_POST(self):
        origin = self.headers.get("Origin", "")
        if self.path not in ("/chat/respond", "/chat/finalize"):
            self.send_error(404)
            return

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

        result = (handle_respond(data) if self.path == "/chat/respond"
                  else handle_finalize(data))

        status = 200 if "error" not in result else 400
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        for k, v in cors_headers(origin).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(json.dumps(result).encode())

    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok"}).encode())
        elif self.path.startswith("/keptvoices/audio/"):
            filename = self.path.replace("/keptvoices/audio/", "")
            audio_path = AUDIO_DIR / filename
            if audio_path.exists():
                suffix = audio_path.suffix
                ctype = "audio/mpeg" if suffix == ".mp3" else "audio/wav"
                self.send_response(200)
                self.send_header("Content-Type", ctype)
                self.send_header("Content-Length", audio_path.stat().st_size)
                self.end_headers()
                self.wfile.write(audio_path.read_bytes())
            else:
                self.send_response(404, "Audio not found")
        else:
            self.send_error(404)

    def do_HEAD(self):
        """Support audio HEAD for streaming."""
        if self.path.startswith("/keptvoices/audio/"):
            filename = self.path.replace("/keptvoices/audio/", "")
            audio_path = AUDIO_DIR / filename
            if audio_path.exists():
                self.send_response(200)
                self.send_header("Content-Type", "audio/mpeg")
                self.send_header("Content-Length", audio_path.stat().st_size)
                self.end_headers()
            else:
                self.send_response(404)
        else:
            self.send_error(404)


def run():
    print(f"[keptvoices] Chat API running on http://0.0.0.0:{PORT}")
    print(f"[keptvoices] POST /chat/respond")
    print(f"[keptvoices] POST /chat/finalize")
    print(f"[keptvoices] GET  /health")
    print(f"[keptvoices] GET  /keptvoices/audio/<file>")
    print(f"[keptvoices] CORS origin: {ALLOWED_ORIGIN}")
    print(f"[keptvoices] Free tier max questions: {FREE_TIER_MAX}")
    print(f"[keptvoices] Audio dir: {AUDIO_DIR}")
    print(f"[keptvoices] Piper: {PIPER_BIN}")
    HTTPServer(("0.0.0.0", PORT), Handler).serve_forever()


if __name__ == "__main__":
    run()
