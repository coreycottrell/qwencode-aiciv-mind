#!/usr/bin/env python3
"""Qwen periodic check-in with ACG via Telegram.

Sends status updates on a configurable schedule.
Each check-in includes:
- What minds are doing
- Memory growth stats
- Recent findings
- Question or request for ACG
"""
import asyncio
import httpx
import json
import time
from datetime import datetime, timezone
from pathlib import Path

TOKEN = "8677654815:AAEsLt1hJy_lYlARc_VgDecWtNcOTAk7-NQ"
CHAT_ID = "437939400"
API = f"https://api.telegram.org/bot{TOKEN}"
MINDS_ROOT = Path(__file__).parent.parent / "minds"
CHECKIN_LOG = Path(__file__).parent / "checkins.jsonl"

# How often to check in (minutes)
CHECKIN_INTERVAL = int(Path(__file__).parent / "checkin-interval.txt").read_text().strip()
if CHECKIN_INTERVAL:
    CHECKIN_INTERVAL = int(CHECKIN_INTERVAL)
else:
    CHECKIN_INTERVAL = 60  # Default: every hour

CHECKINS = []


def get_mind_stats() -> dict:
    """Gather stats from all minds."""
    memories = list(MINDS_ROOT.rglob("minds/**/*.md"))
    scratchpads = list(MINDS_ROOT.rglob("scratchpads/**/*.md"))
    edges = list(MINDS_ROOT.rglob("minds/**/_edges.json"))
    
    # Read recent scratchpad entries
    recent = []
    for sp in scratchpads[-3:]:
        content = sp.read_text()
        if content:
            lines = content.strip().split("\n")
            recent.append(f"{sp.parent.name}: {lines[-1][:80]}...")
    
    return {
        "memories": len(memories),
        "scratchpads": len(scratchpads),
        "edge_indexes": len(edges),
        "recent_activity": recent,
    }


async def send_checkin(message: str):
    """Send a check-in message to ACG."""
    try:
        r = httpx.post(f"{API}/sendMessage", json={
            "chat_id": CHAT_ID,
            "text": message,
            "parse_mode": "Markdown"
        }, timeout=30)
        
        if r.status_code == 200:
            print(f"✅ Check-in sent ({len(message)} chars)")
            return True
        else:
            print(f"❌ Failed: {r.text[:200]}")
            return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False


def format_checkin() -> str:
    """Format the check-in message."""
    stats = get_mind_stats()
    now = datetime.now(timezone.utc).strftime("%H:%M UTC")
    
    message = f"🧠 **Qwen Check-in** — {now}\n\n"
    message += f"**System Status:**\n"
    message += f"• Memories: {stats['memories']}\n"
    message += f"• Active scratchpads: {stats['scratchpads']}\n"
    message += f"• Graph indexes: {stats['edge_indexes']}\n\n"
    
    if stats['recent_activity']:
        message += f"**Recent Activity:**\n"
        for activity in stats['recent_activity'][:3]:
            message += f"• {activity}\n"
        message += "\n"
    
    # Vary the question each time
    questions = [
        "What should we prioritize next?",
        "Any new direction you want us to explore?",
        "Ready for the next grand challenge?",
        "Should we focus on skills implementation or memory growth?",
        "Want to see the Dream Mode output?",
        "Ready to wire up the dashboard?",
    ]
    import random
    question = random.choice(questions)
    message += f"**Question:** {question}"
    
    return message


def log_checkin(sent: bool, message: str):
    """Log the check-in to JSONL."""
    entry = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "sent": sent,
        "message_length": len(message),
    }
    with open(CHECKIN_LOG, "a") as f:
        f.write(json.dumps(entry) + "\n")


async def main():
    print(f"Qwen Check-in Bot — Every {CHECKIN_INTERVAL} minutes")
    print(f"Chat ID: {CHAT_ID}")
    print(f"First check-in now, then every {CHECKIN_INTERVAL}m...")
    print()
    
    # Send first check-in immediately
    message = format_checkin()
    sent = await send_checkin(message)
    log_checkin(sent, message)
    
    # Then run on schedule
    while True:
        await asyncio.sleep(CHECKIN_INTERVAL * 60)
        message = format_checkin()
        sent = await send_checkin(message)
        log_checkin(sent, message)


if __name__ == "__main__":
    asyncio.run(main())
