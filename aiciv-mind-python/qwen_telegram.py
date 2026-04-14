#!/usr/bin/env python3
"""
Qwen Telegram Bot — tmux injection edition

Incoming TG messages → tmux send-keys → Qwen's tmux pane
Qwen's responses → captured from tmux pane → sent back to TG

No relay files. No pipes. Direct tmux injection.
"""
import asyncio
import hashlib
import json
import logging
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from collections import deque
from typing import Optional, Dict

import httpx
from dotenv import load_dotenv, find_dotenv

# ── Config ──

load_dotenv(find_dotenv())
CONFIG_FILE = Path(__file__).parent / "qwen-tg-config.json"
LEDGER_FILE = Path(__file__).parent / "tg-ledger.json"

# ── tmux configuration ──
# The tmux session/pane where Qwen is running
TMUX_SESSION = os.environ.get("QWEN_TMUX_SESSION", "qwen-mind")
TMUX_PANE = os.environ.get("QWEN_TMUX_PANE", f"{TMUX_SESSION}:0.0")

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [qwen-tg] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
logger = logging.getLogger(__name__)


class DeliveryLedger:
    """Track sent/received messages to avoid duplicates."""
    def __init__(self, path: Path = LEDGER_FILE):
        self.path = path
        self.data = self._load()

    def _load(self) -> dict:
        if self.path.exists():
            try:
                return json.loads(self.path.read_text())
            except Exception:
                pass
        return {"last_update_id": 0, "sent_hashes": []}

    def save(self):
        # Prune old hashes
        self.data["sent_hashes"] = self.data["sent_hashes"][-100:]
        self.path.write_text(json.dumps(self.data, indent=2))

    def record_sent(self, h: str):
        self.data["sent_hashes"].append(h)

    def was_sent(self, h: str) -> bool:
        return h in self.data["sent_hashes"]

    def get_last_update(self) -> int:
        return self.data.get("last_update_id", 0)

    def set_last_update(self, uid: int):
        self.data["last_update_id"] = uid


class QwenTelegramBot:
    """Simple Telegram bot: receive → pipe to Cortex → read response → send back."""

    def __init__(self, token: str, chat_id: str = ""):
        self.token = token
        self.chat_id = chat_id
        self.api_base = f"https://api.telegram.org/bot{token}"
        self.client = httpx.AsyncClient(timeout=60)
        self.ledger = DeliveryLedger()
        self._recent_injections: deque = deque(maxlen=50)
        self._running = False
        self._last_response_time = time.time()

    async def start(self):
        """Verify bot works and get info."""
        r = await self.client.get(f"{self.api_base}/getMe")
        info = r.json()
        if info.get("ok"):
            bot = info["result"]
            logger.info(f"Bot started: @{bot.get('username')} (id={bot['id']})")
        return info

    async def send_message(self, text: str, parse_mode: str = "Markdown"):
        """Send a message to the configured chat."""
        if not self.chat_id:
            logger.error("No chat_id configured — cannot send")
            return
        # Chunk long messages
        max_len = 4000
        chunks = [text[i:i+max_len] for i in range(0, len(text), max_len)]
        for chunk in chunks:
            try:
                r = await self.client.post(
                    f"{self.api_base}/sendMessage",
                    json={"chat_id": self.chat_id, "text": chunk, "parse_mode": parse_mode}
                )
                data = r.json()
                if data.get("ok"):
                    h = hashlib.md5(chunk[:100].encode()).hexdigest()
                    self.ledger.record_sent(h)
                    self.ledger.save()
                else:
                    logger.warning(f"Send failed: {data}")
            except Exception as e:
                logger.error(f"Send error: {e}")
            await asyncio.sleep(0.5)  # avoid rate limits

    async def get_updates(self, offset: int = 0, timeout: int = 30) -> list:
        """Poll for new messages."""
        try:
            r = await self.client.get(
                f"{self.api_base}/getUpdates",
                params={"offset": offset, "timeout": timeout, "allowed_updates": ["message"]}
            )
            data = r.json()
            if data.get("ok"):
                return data.get("result", [])
            else:
                logger.warning(f"Get updates failed: {data}")
                return []
        except Exception as e:
            logger.error(f"Poll error: {e}")
            return []

    def _message_hash(self, text: str) -> str:
        return hashlib.md5(text[:200].encode()).hexdigest()

    def _inject_tmux(self, text: str) -> bool:
        """Inject text into Qwen's tmux pane using send-keys."""
        try:
            # Chunk large messages (tmux input buffer limit)
            CHUNK_SIZE = 100
            if len(text) > CHUNK_SIZE:
                for i in range(0, len(text), CHUNK_SIZE):
                    chunk = text[i:i+CHUNK_SIZE]
                    subprocess.run(
                        ["tmux", "send-keys", "-t", TMUX_PANE, "-l", chunk],
                        check=True, timeout=5
                    )
                    time.sleep(0.05)  # 50ms between chunks
            else:
                subprocess.run(
                    ["tmux", "send-keys", "-t", TMUX_PANE, "-l", text],
                    check=True, timeout=5
                )
            # Send Enter to execute
            subprocess.run(
                ["tmux", "send-keys", "-t", TMUX_PANE, "Enter"],
                check=True, timeout=5
            )
            logger.info(f"Injected {len(text)} chars into {TMUX_PANE}")
            return True
        except subprocess.TimeoutExpired:
            logger.error(f"tmux injection timed out")
            return False
        except subprocess.CalledProcessError as e:
            logger.error(f"tmux injection failed: {e}")
            return False

    def _capture_tmux_output(self, lines: int = 20) -> Optional[str]:
        """Capture recent output from tmux pane."""
        try:
            result = subprocess.run(
                ["tmux", "capture-pane", "-t", TMUX_PANE, "-p", "-S", f"-{lines}"],
                capture_output=True, text=True, timeout=5
            )
            return result.stdout.strip()
        except Exception as e:
            logger.error(f"tmux capture failed: {e}")
            return None

    async def run(self):
        """Main polling loop."""
        logger.info("Starting Qwen Telegram Bot with tmux injection...")
        logger.info(f"  tmux session: {TMUX_SESSION}")
        logger.info(f"  tmux pane: {TMUX_PANE}")
        logger.info(f"  Chat ID: {self.chat_id or 'any (will auto-detect)'}")
        
        # Verify tmux
        try:
            result = subprocess.run(
                ["tmux", "list-panes", "-t", TMUX_SESSION],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode == 0:
                logger.info(f"tmux session confirmed: {TMUX_SESSION}")
            else:
                logger.error(f"tmux session not found: {TMUX_SESSION}")
                logger.info("Available sessions:")
                os.system("tmux list-sessions 2>&1")
        except Exception as e:
            logger.error(f"tmux check failed: {e}")

        self._running = True
        offset = 0  # Always start from 0 for new bot
        idle_count = 0

        while self._running:
            try:
                updates = await self.get_updates(offset, timeout=30)

                if not updates:
                    idle_count += 1
                    if idle_count % 10 == 0:
                        logger.info(f"Polling... (idle {idle_count} cycles)")
                    await asyncio.sleep(1)
                    continue

                idle_count = 0

                for update in updates:
                    offset = max(offset, update["update_id"] + 1)
                    self.ledger.set_last_update(update["update_id"])

                    msg = update.get("message", {})
                    chat_id = str(msg.get("chat", {}).get("id", ""))
                    text = msg.get("text", "").strip()
                    from_user = msg.get("from", {}).get("first_name", "unknown")

                    if not text or not chat_id:
                        continue

                    # Auto-detect chat_id on first message
                    if not self.chat_id:
                        self.chat_id = chat_id
                        logger.info(f"Auto-detected chat_id: {chat_id}")

                    if chat_id != self.chat_id:
                        continue  # ignore other chats

                    # Dedup
                    h = self._message_hash(text)
                    if self.ledger.was_sent(h):
                        continue

                    logger.info(f"[TG] {from_user}: {text[:80]}...")

                    # INJECT into tmux pane
                    formatted = f"[Telegram from {from_user}] {text}"
                    injected = self._inject_tmux(formatted)

                    if injected:
                        # Acknowledge receipt
                        await self.send_message(f"✅ Message received. Processing in tmux...")
                        
                        # Wait a bit and capture any immediate response
                        await asyncio.sleep(5)
                        captured = self._capture_tmux_output(10)
                        if captured and len(captured) > 50:
                            # Only send if there's substantial new output
                            await self.send_message(f"📋 Recent output:\n```\n{captured[-500:]}\n```")
                    else:
                        await self.send_message("❌ Failed to inject message. tmux may not be running.")

                    self.ledger.save()
                    self._last_response_time = time.time()

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Loop error: {e}", exc_info=True)
                wait = min(30, max(2, (time.time() - self._last_response_time) / 10))
                await asyncio.sleep(wait)

        logger.info("Bot stopped")

    async def stop(self):
        """Stop the bot."""
        self._running = False
        await self.client.aclose()
        logger.info("Bot stopped, ledger saved")


def load_config() -> dict:
    """Load bot config from file or env."""
    # 1. Check env vars
    token = os.environ.get("QWEN_TG_BOT_TOKEN", "")
    chat_id = os.environ.get("QWEN_TG_CHAT_ID", "")

    # 2. Check config file
    if CONFIG_FILE.exists():
        cfg = json.loads(CONFIG_FILE.read_text())
        token = token or cfg.get("bot_token", "")
        chat_id = chat_id or cfg.get("chat_id", "")

    if not token:
        print("ERROR: No bot token found.")
        print("Set QWEN_TG_BOT_TOKEN env var or create qwen-tg-config.json:")
        print('  {"bot_token": "YOUR_TOKEN", "chat_id": "YOUR_CHAT_ID"}')
        sys.exit(1)

    return {"token": token, "chat_id": chat_id}


async def main():
    cfg = load_config()
    bot = QwenTelegramBot(cfg["token"], cfg["chat_id"])

    await bot.start()

    try:
        await bot.run()
    except KeyboardInterrupt:
        logger.info("Interrupted, shutting down...")
    finally:
        await bot.stop()


if __name__ == "__main__":
    asyncio.run(main())
