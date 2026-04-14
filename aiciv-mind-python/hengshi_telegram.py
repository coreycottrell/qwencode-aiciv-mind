#!/usr/bin/env python3
"""Hengshi (衡实) Unified Telegram Bot — adapted from ACG's telegram_unified.py"""
import asyncio, hashlib, httpx, json, logging, os, subprocess, sys, time
from datetime import datetime, timezone
from pathlib import Path
from collections import deque

CONFIG_FILE = Path(__file__).parent / "hengshi-tg-config.json"
CIV_ID = "hengshi"
PID_LOCK_FILE = Path(f"/tmp/{CIV_ID}_telegram_unified.pid")
TMUX_PANE = "%429"  # Hengshi's pane
LEDGER_FILE = Path(__file__).parent.parent / "config" / "tg_ledger.json"
TG_MAX_LEN = 4096

logging.basicConfig(level=logging.INFO, format="%(asctime)s [hengshi-tg] %(message)s")
logger = logging.getLogger(__name__)

def load_config():
    if not CONFIG_FILE.exists():
        raise FileNotFoundError(f"Config not found: {CONFIG_FILE}")
    return json.loads(CONFIG_FILE.read_text())

def chunk_send_keys(text, pane):
    """Send text to tmux in 100-char chunks, then 5x Enter retries (ACG pattern)."""
    CHUNK = 100
    try:
        if len(text) > CHUNK:
            for i in range(0, len(text), CHUNK):
                subprocess.run(["tmux", "send-keys", "-t", pane, "-l", text[i:i+CHUNK]],
                             check=True, timeout=5)
                time.sleep(0.05)
        else:
            subprocess.run(["tmux", "send-keys", "-t", pane, "-l", text],
                         check=True, timeout=5)
        # 5x Enter retries — AICIV standard
        for i in range(5):
            time.sleep(0.4)
            subprocess.run(["tmux", "send-keys", "-t", pane, "Enter"],
                         check=True, timeout=2)
        logger.info(f"Injected {len(text)} chars to {pane}")
        return True
    except Exception as e:
        logger.error(f"tmux injection failed: {e}")
        return False

class DeliveryLedger:
    def __init__(self, path=LEDGER_FILE):
        self.path = Path(path)
        self.data = self._load()
    def _load(self):
        if self.path.exists():
            try: return json.loads(self.path.read_text())
            except: pass
        return {"last_update_id": 0, "sent_hashes": []}
    def save(self):
        self.data["sent_hashes"] = self.data["sent_hashes"][-100:]
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps(self.data, indent=2))
    def record_sent(self, h): self.data["sent_hashes"].append(h)
    def was_sent(self, h): return h in self.data["sent_hashes"]
    def get_last_update(self): return self.data.get("last_update_id", 0)
    def set_last_update(self, uid): self.data["last_update_id"] = uid

class HengshiBot:
    def __init__(self, token, chat_id, authorized_users=None):
        self.token = token
        self.chat_id = chat_id
        self.authorized = authorized_users or {}
        self.api = f"https://api.telegram.org/bot{token}"
        self.client = httpx.AsyncClient(timeout=60)
        self.ledger = DeliveryLedger()
        self._recent = deque(maxlen=100)
        self._running = False

    async def start(self):
        r = await self.client.get(f"{self.api}/getMe")
        info = r.json()
        if info.get("ok"):
            logger.info(f"Bot @{info['result']['username']} started")

    async def send(self, text, parse_mode=None):
        max_len = 4000
        chunks = [text[i:i+max_len] for i in range(0, len(text), max_len)]
        for chunk in chunks:
            try:
                # ALWAYS plain text - no markdown parsing
                data = {"chat_id": self.chat_id, "text": chunk}
                r = await self.client.post(f"{self.api}/sendMessage", json=data)
                if r.json().get("ok"):
                    h = hashlib.md5(chunk[:100].encode()).hexdigest()
                    self.ledger.record_sent(h)
                    self.ledger.save()
                else:
                    logger.warning(f"Send failed: {r.json()}")
            except Exception as e:
                logger.error(f"Send error: {e}")
            await asyncio.sleep(0.5)

    async def get_updates(self, offset=0, timeout=30):
        try:
            r = await self.client.get(f"{self.api}/getUpdates",
                params={"offset": offset, "timeout": timeout, "allowed_updates": ["message"]})
            data = r.json()
            return data.get("result", []) if data.get("ok") else []
        except Exception as e:
            logger.error(f"Poll error: {e}")
            return []

    async def run(self):
        logger.info(f"Starting Hengshi TG Bot → tmux pane {TMUX_PANE}")
        # Verify tmux
        result = subprocess.run(["tmux", "list-panes", "-t", "qwen-mind"],
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            logger.info(f"tmux qwen-mind confirmed: {result.stdout.strip()[:100]}")
        else:
            logger.warning(f"tmux qwen-mind not found!")

        self._running = True
        offset = 0

        while self._running:
            try:
                updates = await self.get_updates(offset, timeout=30)
                if not updates:
                    await asyncio.sleep(1)
                    continue

                for update in updates:
                    offset = max(offset, update["update_id"] + 1)
                    self.ledger.set_last_update(update["update_id"])

                    msg = update.get("message", {})
                    chat_id = str(msg.get("chat", {}).get("id", ""))
                    text = msg.get("text", "").strip()
                    user = msg.get("from", {}).get("first_name", "unknown")

                    if not text or not chat_id:
                        continue
                    if chat_id != self.chat_id:
                        continue

                    # Dedup
                    h = hashlib.md5(text[:200].encode()).hexdigest()
                    if self.ledger.was_sent(h):
                        continue

                    logger.info(f"[TG] {user}: {text[:80]}")

                    # Inject to tmux
                    formatted = f"[Telegram from {user}] {text}"
                    injected = chunk_send_keys(formatted, TMUX_PANE)

                    if injected:
                        await self.send(f"✅ Received. Processing...")
                        await asyncio.sleep(5)
                        # Capture response
                        try:
                            cap = subprocess.run(
                                ["tmux", "capture-pane", "-t", TMUX_PANE, "-p", "-S", "-10"],
                                capture_output=True, text=True, timeout=5)
                            out = cap.stdout.strip()
                            if out and len(out) > 50:
                                # Strip the injected message from output
                                lines = out.split("\n")
                                # Skip lines that look like our injected message
                                clean = [l for l in lines if not l.startswith("[Telegram")]
                                clean_text = "\n".join(clean[-10:])
                                if len(clean_text) > 30:
                                    await self.send(f"📋 Output:\n```\n{clean_text[-500:]}\n```")
                        except Exception as e:
                            logger.error(f"Capture error: {e}")
                    else:
                        await self.send("❌ tmux injection failed")

                    self.ledger.save()

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Loop error: {e}")
                await asyncio.sleep(5)

async def main():
    cfg = load_config()
    bot = HengshiBot(cfg["bot_token"], cfg["chat_id"], cfg.get("authorized_users", {}))
    await bot.start()
    try:
        await bot.run()
    except KeyboardInterrupt:
        logger.info("Shutting down...")
    finally:
        await bot.client.aclose()

if __name__ == "__main__":
    if len(sys.argv) >= 3 and sys.argv[1] == "send":
        # Send one message and exit (no polling, no conflict)
        import asyncio
        cfg = load_config()
        api = f"https://api.telegram.org/bot{cfg['bot_token']}"
        text = " ".join(sys.argv[2:])
        async def go():
            async with httpx.AsyncClient(timeout=30) as c:
                r = await c.post(f"{api}/sendMessage", json={"chat_id": cfg["chat_id"], "text": text, "parse_mode": "Markdown"})
                print(r.json())
        asyncio.run(go())
        sys.exit(0)
    asyncio.run(main())
