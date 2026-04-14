#!/usr/bin/env python3
"""
Unified Telegram Bot for A-C-Gee Civilization
Bidirectional communication: TG <-> Claude via tmux injection + log streaming

Features:
- Incoming TG messages inject to Claude's tmux session
- Claude's responses stream to TG feed (like Nexus dashboard)
- Intelligent message chunking for large posts
- Auto-detects ACG tmux sessions
- Voice Mode: Toggle with /voice_mode for audio summaries alongside text

Voice Mode:
- When enabled, sends succinct voice summaries after text responses
- Uses ElevenLabs or gTTS for text-to-speech
- Voice summaries are optimized for listening (short, conversational)
- Toggle per-user with /voice_mode command
"""

import asyncio
import fcntl
import hashlib
import json
import logging
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional, Dict, List, Set, Any
from collections import deque

import httpx


# ========== Persistent Delivery Ledger ==========
# Survives restarts, session switches, and prevents duplicate sends

LEDGER_FILE = Path(__file__).parent.parent / "config" / "tg_delivery_ledger.json"
LEDGER_MAX_ENTRIES = 1000  # Prune beyond this


class DeliveryLedger:
    """
    Persistent ledger tracking sent/received Telegram messages.

    Survives:
    - Process restarts
    - Session switches (no more clearing _sent_ids)
    - Multiple bot instances (file is source of truth)

    Structure:
    {
        "outbound": {
            "sent_messages": {
                "uuid-123": {"tg_msg_id": 13838, "ts": "...", "preview": "first 50 chars"}
            }
        },
        "inbound": {
            "received_messages": {
                "12345": {"ts": "...", "from": "user", "preview": "..."}  # key is update_id
            }
        },
        "cursor": {
            "session_id": "current-session-id",
            "last_update_id": 12345
        }
    }
    """

    def __init__(self, ledger_path: Path = LEDGER_FILE):
        self.ledger_path = ledger_path
        self._data: Dict[str, Any] = self._load()
        self._logger = logging.getLogger(__name__)

    def _load(self) -> Dict[str, Any]:
        """Load ledger from disk or create empty structure."""
        if self.ledger_path.exists():
            try:
                with self.ledger_path.open('r') as f:
                    data = json.load(f)
                    # Ensure required structure exists
                    if "outbound" not in data:
                        data["outbound"] = {"sent_messages": {}}
                    if "inbound" not in data:
                        data["inbound"] = {"received_messages": {}}
                    if "cursor" not in data:
                        data["cursor"] = {"session_id": None, "last_update_id": 0}
                    return data
            except (json.JSONDecodeError, IOError) as e:
                logging.getLogger(__name__).warning(f"Ledger load failed, starting fresh: {e}")

        return {
            "outbound": {"sent_messages": {}},
            "inbound": {"received_messages": {}},
            "cursor": {"session_id": None, "last_update_id": 0}
        }

    def _save(self):
        """Persist ledger to disk."""
        try:
            self.ledger_path.parent.mkdir(parents=True, exist_ok=True)
            # Atomic write: write to temp file then rename
            tmp_path = self.ledger_path.with_suffix('.tmp')
            with tmp_path.open('w') as f:
                json.dump(self._data, f, indent=2)
            tmp_path.rename(self.ledger_path)
        except IOError as e:
            self._logger.error(f"Ledger save failed: {e}")

    def _prune_if_needed(self):
        """Keep ledger under max size."""
        outbound = self._data["outbound"]["sent_messages"]
        if len(outbound) > LEDGER_MAX_ENTRIES:
            # Sort by timestamp, keep most recent
            sorted_keys = sorted(
                outbound.keys(),
                key=lambda k: outbound[k].get("ts", ""),
                reverse=True
            )[:LEDGER_MAX_ENTRIES]
            self._data["outbound"]["sent_messages"] = {
                k: outbound[k] for k in sorted_keys
            }
            self._logger.info(f"Pruned outbound ledger to {LEDGER_MAX_ENTRIES} entries")

        inbound = self._data["inbound"]["received_messages"]
        if len(inbound) > LEDGER_MAX_ENTRIES:
            sorted_keys = sorted(
                inbound.keys(),
                key=lambda k: inbound[k].get("ts", ""),
                reverse=True
            )[:LEDGER_MAX_ENTRIES]
            self._data["inbound"]["received_messages"] = {
                k: inbound[k] for k in sorted_keys
            }
            self._logger.info(f"Pruned inbound ledger to {LEDGER_MAX_ENTRIES} entries")

    def is_outbound_sent(self, message_id: str) -> bool:
        """Check if a message UUID has already been sent."""
        return message_id in self._data["outbound"]["sent_messages"]

    def record_outbound(self, message_id: str, tg_msg_id: Optional[int], text: str):
        """Record that a message was sent to Telegram."""
        self._data["outbound"]["sent_messages"][message_id] = {
            "tg_msg_id": tg_msg_id,
            "ts": datetime.utcnow().isoformat() + "Z",
            "preview": text[:50] if text else ""
        }
        self._prune_if_needed()
        self._save()

    def is_inbound_processed(self, update_id: int) -> bool:
        """Check if an inbound update has already been processed."""
        return str(update_id) in self._data["inbound"]["received_messages"]

    def record_inbound(self, update_id: int, from_user: str, text: str):
        """Record that an inbound message was processed."""
        self._data["inbound"]["received_messages"][str(update_id)] = {
            "ts": datetime.utcnow().isoformat() + "Z",
            "from": from_user,
            "preview": text[:50] if text else ""
        }
        self._prune_if_needed()
        self._save()

    def get_last_update_id(self) -> int:
        """Get the last processed update ID."""
        return self._data["cursor"].get("last_update_id", 0)

    def set_last_update_id(self, update_id: int):
        """Set the last processed update ID."""
        self._data["cursor"]["last_update_id"] = update_id
        self._save()

    def get_session_id(self) -> Optional[str]:
        """Get the current session ID from cursor."""
        return self._data["cursor"].get("session_id")

    def set_session_id(self, session_id: str):
        """Set the current session ID in cursor (NO clearing sent_ids)."""
        self._data["cursor"]["session_id"] = session_id
        self._save()

    def get_stats(self) -> Dict[str, int]:
        """Get ledger statistics."""
        return {
            "outbound_count": len(self._data["outbound"]["sent_messages"]),
            "inbound_count": len(self._data["inbound"]["received_messages"]),
            "last_update_id": self._data["cursor"].get("last_update_id", 0)
        }


class RateLimiter:
    """
    Exponential backoff and rate limiting for Telegram API calls.

    Philosophy: We're guests using Telegram's free API. Be polite.
    Back off when asked. Don't be the reason they add stricter limits.

    Behavior:
    - Start at 1 second delay on first failure
    - Double on each consecutive failure (1s, 2s, 4s, 8s, 16s...)
    - Cap at 5 minutes max backoff
    - Reset to 1s after successful call
    - After 5 consecutive failures: slow polling to 30s intervals
    - After 10 consecutive failures: stop polling, wait for recovery
    """

    def __init__(self):
        self.consecutive_failures = 0
        self.backoff_seconds = 1.0
        self.max_backoff = 300.0  # 5 minutes
        self.last_success = time.time()
        self.last_request_time = 0.0
        self.min_request_interval = 1.0  # Don't poll faster than 1 req/sec
        self.min_send_interval = 0.5  # 0.5s delay between message sends

        # Degradation thresholds
        self.slow_poll_threshold = 5   # failures before slowing to 30s
        self.stop_poll_threshold = 10  # failures before stopping
        self.slow_poll_interval = 30.0  # interval when degraded

        # Recovery tracking
        self.downtime_start: Optional[float] = None
        self.is_degraded = False
        self.is_stopped = False

    def record_failure(self) -> float:
        """
        Record an API failure and return the backoff delay to wait.
        """
        self.consecutive_failures += 1
        self.backoff_seconds = min(self.backoff_seconds * 2, self.max_backoff)

        # Track when downtime started
        if self.downtime_start is None:
            self.downtime_start = time.time()

        # Check degradation thresholds
        if self.consecutive_failures >= self.stop_poll_threshold:
            if not self.is_stopped:
                self.is_stopped = True
                logging.getLogger(__name__).warning(
                    f"API unreachable after {self.consecutive_failures} consecutive failures. "
                    "Stopping active polling. Waiting for recovery..."
                )
        elif self.consecutive_failures >= self.slow_poll_threshold:
            if not self.is_degraded:
                self.is_degraded = True
                logging.getLogger(__name__).warning(
                    f"Degraded mode: {self.consecutive_failures} consecutive failures. "
                    f"Slowing polling to {self.slow_poll_interval}s intervals."
                )

        logging.getLogger(__name__).info(
            f"Backing off: {self.backoff_seconds:.1f}s after {self.consecutive_failures} failures"
        )

        return self.backoff_seconds

    def record_success(self):
        """
        Record a successful API call and reset backoff state.
        """
        if self.consecutive_failures > 0:
            downtime = time.time() - (self.downtime_start or self.last_success)
            logging.getLogger(__name__).info(
                f"API recovered after {downtime:.0f}s downtime "
                f"({self.consecutive_failures} failures)"
            )

        self.consecutive_failures = 0
        self.backoff_seconds = 1.0
        self.last_success = time.time()
        self.downtime_start = None
        self.is_degraded = False
        self.is_stopped = False

    async def wait_for_rate_limit(self, is_send: bool = False):
        """
        Wait if needed to respect rate limits.

        Args:
            is_send: True if this is a message send (uses shorter interval)
        """
        now = time.time()
        interval = self.min_send_interval if is_send else self.min_request_interval
        elapsed = now - self.last_request_time

        if elapsed < interval:
            wait_time = interval - elapsed
            logging.getLogger(__name__).debug(f"Rate limit: waiting {wait_time:.2f}s")
            await asyncio.sleep(wait_time)

        self.last_request_time = time.time()

    def get_poll_interval(self) -> float:
        """
        Get the appropriate poll interval based on current state.

        Returns:
            Poll interval in seconds, or -1 if polling should stop
        """
        if self.is_stopped:
            return -1  # Signal to stop polling
        elif self.is_degraded:
            return self.slow_poll_interval
        else:
            return self.min_request_interval

    def should_attempt_request(self) -> bool:
        """
        Check if we should attempt a request based on current state.

        Returns:
            True if request should be attempted, False if still in backoff
        """
        if self.is_stopped:
            # In stopped state, only attempt every 60s for recovery check
            return time.time() - self.last_request_time >= 60.0
        return True

# CIV identity - derived at runtime for any fork
_vf = Path(__file__).parent.parent / "variables.template.json"
try:
    import json as _json
    _d = _json.load(open(_vf))
    _civ = _d.get("CIV_NAME", "").lower().replace(" ", "")
    if not _civ or _civ in ("your_name", "yourname"):
        raise ValueError()
except Exception:
    _civ = Path(__file__).parent.parent.name.lower()
CIV_ID = _civ

# Singleton lock file (CIV-specific for Multi-CIV isolation)
PID_LOCK_FILE = Path(f"/tmp/{CIV_ID}_telegram_unified.pid")

# Voice mode imports
VOICE_MODE_STATE_FILE = Path(__file__).parent.parent / ".tg_sessions" / "voice_mode_state.json"
VOICE_SCRIPT = Path(__file__).parent / "telegram-voice" / "send_telegram_voice.py"

# Attachment download directory
PROJECT_ROOT = Path(__file__).parent.parent
ATTACHMENT_DIR = PROJECT_ROOT / "downloads" / "telegram_attachments"
ATTACHMENT_DIR.mkdir(parents=True, exist_ok=True)

# Configure logging
logging.basicConfig(
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    level=logging.INFO
)
logger = logging.getLogger(__name__)

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
CONFIG_FILE = PROJECT_ROOT / "config" / "telegram_config.json"

# Claude log paths
HISTORY_FILE = Path.home() / ".claude" / "history.jsonl"
PROJECT_MATCH = str(PROJECT_ROOT)
PROJECT_SLUG = str(PROJECT_ROOT).replace("/", "-").lstrip("-")
LOG_ROOT = Path.home() / ".claude" / "projects" / PROJECT_SLUG

# Telegram limits
TG_MAX_MESSAGE_LEN = 4096
TG_SAFE_LEN = 3500  # Leave room for headers


class TelegramBot:
    """Unified Telegram bot with bidirectional communication."""

    def __init__(self, config: Dict):
        self.config = config
        self.bot_token = config["bot_token"]
        self.chat_id = config["chat_id"]
        self.authorized_users = config.get("authorized_users", {})

        # Telegram API
        self.api_base = f"https://api.telegram.org/bot{self.bot_token}"
        self.client: Optional[httpx.AsyncClient] = None

        # tmux target
        self.tmux_session: Optional[str] = None
        self.tmux_pane: Optional[str] = None

        # Log streaming state
        self.current_session: Optional[str] = None
        self.current_log_path: Optional[Path] = None
        self.last_position: int = 0

        # Persistent delivery ledger (replaces in-memory _sent_ids)
        self._ledger = DeliveryLedger()
        self._last_update_id: int = self._ledger.get_last_update_id()  # Restore from ledger

        # Running state
        self._running = False

        # Voice mode state
        self._voice_mode_state: Dict[str, Dict] = self._load_voice_mode_state()

        # Injection deduplication (defense in depth against multiple instances)
        # Tracks recent message hashes to prevent duplicate injection
        self._recent_injections: deque = deque(maxlen=100)

        # Rate limiting and exponential backoff
        self.rate_limiter = RateLimiter()

        # ========== Gateway Integration (PROJECT-177) ==========
        # Read gateway config from telegram_config.json
        self.use_gateway: bool = config.get("use_gateway", False)
        self.gateway_url: str = config.get("gateway_url", "http://localhost:8098").rstrip("/")
        self.gateway_token: str = config.get("gateway_token", "")
        self._gateway_registered: bool = False
        # Polling interval for gateway response (seconds)
        self._gateway_poll_interval: float = 2.0
        # Max total wait for gateway response before timeout (seconds)
        self._gateway_response_timeout: float = 120.0

    # ========== Voice Mode State Management ==========

    def _load_voice_mode_state(self) -> Dict[str, Dict]:
        """Load voice mode state from file."""
        try:
            if VOICE_MODE_STATE_FILE.exists():
                with open(VOICE_MODE_STATE_FILE, 'r') as f:
                    return json.load(f)
        except Exception as e:
            logger.warning(f"Failed to load voice mode state: {e}")
        return {}

    def _save_voice_mode_state(self):
        """Save voice mode state to file."""
        try:
            VOICE_MODE_STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
            with open(VOICE_MODE_STATE_FILE, 'w') as f:
                json.dump(self._voice_mode_state, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save voice mode state: {e}")

    def is_voice_mode_enabled(self, user_id: str) -> bool:
        """Check if voice mode is enabled for a user."""
        user_state = self._voice_mode_state.get(user_id, {})
        return user_state.get("enabled", False)

    def toggle_voice_mode(self, user_id: str) -> bool:
        """Toggle voice mode for a user. Returns new state."""
        if user_id not in self._voice_mode_state:
            self._voice_mode_state[user_id] = {}

        current = self._voice_mode_state[user_id].get("enabled", False)
        new_state = not current

        self._voice_mode_state[user_id] = {
            "enabled": new_state,
            "last_toggled": datetime.utcnow().isoformat() + "Z"
        }
        self._save_voice_mode_state()

        logger.info(f"Voice mode for user {user_id}: {new_state}")
        return new_state

    # ========== Gateway Integration (PROJECT-177) ==========

    def _gateway_headers(self) -> Dict[str, str]:
        """Build Authorization headers for gateway requests."""
        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if self.gateway_token:
            headers["Authorization"] = f"Bearer {self.gateway_token}"
        return headers

    async def _gateway_register(self):
        """
        Register this Telegram bot as a gateway client on startup.

        Calls:
          POST {gateway_url}/api/clients/register
          POST {gateway_url}/api/start  (fires portal notification + capabilities context)

        Sets self._gateway_registered = True on success.
        Falls back gracefully on any error (gateway mode is disabled for the session).
        """
        if not self.client:
            return

        register_payload = {
            "client_id": f"telegram-{CIV_ID}-bot",
            "client_type": "telegram",
            "capabilities": {
                "markdown": False,
                "images": False,
                "html_preview": False,
                "voice_output": False,
                "canvas": False,
            },
        }

        try:
            resp = await self.client.post(
                f"{self.gateway_url}/api/clients/register",
                json=register_payload,
                headers=self._gateway_headers(),
                timeout=10,
            )
            if resp.status_code == 200:
                logger.info("Gateway: client registered successfully")
            else:
                logger.warning(
                    f"Gateway: client registration returned {resp.status_code}: {resp.text[:200]}"
                )
                # Non-fatal — continue without gateway
                return
        except Exception as e:
            logger.warning(f"Gateway: client registration failed ({type(e).__name__}: {e}). Falling back to tmux.")
            return

        # Fire /api/start to trigger portal notification + capabilities context injection
        start_payload = {
            "username": f"telegram-{CIV_ID}-bot",
            "client_type": "telegram",
        }
        try:
            resp = await self.client.post(
                f"{self.gateway_url}/api/start",
                json=start_payload,
                headers=self._gateway_headers(),
                timeout=10,
            )
            if resp.status_code == 200:
                logger.info("Gateway: /api/start fired successfully")
            else:
                logger.warning(f"Gateway: /api/start returned {resp.status_code}: {resp.text[:200]}")
                # Non-fatal — registration succeeded, we can still inject
        except Exception as e:
            logger.warning(f"Gateway: /api/start failed ({type(e).__name__}: {e}). Continuing anyway.")

        self._gateway_registered = True
        logger.info(f"Gateway mode ACTIVE — routing via {self.gateway_url}")

    async def _gateway_inject(self, content: str) -> bool:
        """
        Inject a message through the gateway (POST /api/inject).

        Returns True on success, False on any failure (caller falls back to tmux).
        """
        if not self.client or not self._gateway_registered:
            return False

        payload = {
            "client_id": f"telegram-{CIV_ID}-bot",
            "client_type": "telegram",
            "content": content,
        }
        try:
            resp = await self.client.post(
                f"{self.gateway_url}/api/inject",
                json=payload,
                headers=self._gateway_headers(),
                timeout=15,
            )
            if resp.status_code == 200:
                logger.info(f"Gateway: injected message ({len(content)} chars)")
                return True
            else:
                logger.warning(
                    f"Gateway: inject returned {resp.status_code}: {resp.text[:200]}. Falling back."
                )
                return False
        except Exception as e:
            logger.warning(
                f"Gateway: inject failed ({type(e).__name__}: {e}). Falling back to tmux."
            )
            return False

    async def _gateway_poll_response(self) -> Optional[str]:
        """
        Poll GET /api/response every 2s until status=ready, then return the content.

        Returns:
            The response content string if status=ready.
            None if timed out, stalled, or errored.
        """
        if not self.client:
            return None

        deadline = time.time() + self._gateway_response_timeout
        while time.time() < deadline:
            try:
                resp = await self.client.get(
                    f"{self.gateway_url}/api/response",
                    headers=self._gateway_headers(),
                    timeout=10,
                )
                if resp.status_code == 200:
                    data = resp.json()
                    status = data.get("status", "")
                    if status == "ready":
                        content = data.get("content", "")
                        logger.info(f"Gateway: response ready ({len(content)} chars)")
                        return content
                    elif status in ("stalled", "error"):
                        logger.warning(f"Gateway: response status={status}. Giving up.")
                        return None
                    # status=thinking or idle: keep polling
                    logger.debug(f"Gateway: response status={status}, polling again...")
                else:
                    logger.warning(f"Gateway: /api/response returned {resp.status_code}")
            except Exception as e:
                logger.warning(f"Gateway: /api/response poll error ({type(e).__name__}: {e})")

            await asyncio.sleep(self._gateway_poll_interval)

        logger.warning(f"Gateway: response polling timed out after {self._gateway_response_timeout}s")
        return None

    async def start(self):
        """Start the bot."""
        self.client = httpx.AsyncClient(timeout=30)

        # Detect tmux session
        self.tmux_session = self._detect_acg_session()
        if self.tmux_session:
            self.tmux_pane = f"{self.tmux_session}:0.0"
            logger.info(f"Detected ACG tmux session: {self.tmux_session}")
        else:
            logger.warning("No ACG tmux session detected - injection will fail")

        # Find Claude log session
        session_id = self._find_claude_session()
        if session_id:
            self._switch_session(session_id)
            logger.info(f"Monitoring Claude session: {session_id}")

        # Gateway registration (if enabled)
        if self.use_gateway:
            await self._gateway_register()

        self._running = True
        logger.info("Telegram bot started")

        # Send startup message
        mode_note = " (gateway mode)" if self.use_gateway and self._gateway_registered else ""
        await self._send_message(f"A-C-Gee Telegram bot online. Claude log streaming active.{mode_note}")

    async def shutdown(self):
        """Shutdown the bot."""
        self._running = False
        if self.client:
            await self.client.aclose()
        logger.info("Telegram bot stopped")

    async def run(self):
        """Main run loop - polls for TG updates and Claude logs."""
        await self.start()

        # Session refresh counter (check every ~60 seconds)
        loop_count = 0
        SESSION_REFRESH_INTERVAL = 120  # loops (at 0.5s = 60 seconds)
        last_session_refresh = time.time()

        try:
            while self._running:
                # Get current poll interval based on rate limiter state
                poll_interval = self.rate_limiter.get_poll_interval()

                if poll_interval < 0:
                    # API is stopped, only do recovery checks every 60s
                    if self.rate_limiter.should_attempt_request():
                        await self._poll_telegram_updates()
                    await asyncio.sleep(60)
                    continue

                # Check for new Telegram messages (with rate limiting)
                if self.rate_limiter.should_attempt_request():
                    await self._poll_telegram_updates()

                # Check for new Claude log entries (local, no rate limiting needed)
                await self._poll_claude_logs()

                # Periodic session refresh - detect new sessions
                # Use time-based check instead of loop counter for accuracy
                now = time.time()
                if now - last_session_refresh >= 60:
                    last_session_refresh = now
                    await self._refresh_session_if_needed()

                # Sleep based on rate limiter state
                await asyncio.sleep(poll_interval)

        except KeyboardInterrupt:
            logger.info("Interrupted by user")
        except Exception as e:
            logger.exception(f"Bot error: {e}")
        finally:
            await self.shutdown()

    async def _refresh_session_if_needed(self):
        """Check if a newer ACG session exists and switch to it."""
        try:
            new_session = self._detect_acg_session()
            if new_session and new_session != self.tmux_session:
                logger.info(f"Session change detected: {self.tmux_session} -> {new_session}")
                self.tmux_session = new_session
                self.tmux_pane = f"{new_session}:0.0"
                await self._send_message(f"Switched to new session: {new_session}")
            elif not new_session and self.tmux_session:
                # Current session may have died, try to find any available
                logger.warning(f"Current session {self.tmux_session} may be gone, checking...")
                if not self._check_tmux_session():
                    logger.error("Session lost! Will retry detection on next cycle.")
                    self.tmux_session = None
                    self.tmux_pane = None
        except Exception as e:
            logger.error(f"Session refresh error: {e}")

    # ========== Telegram Polling ==========

    async def _poll_telegram_updates(self):
        """Poll for new Telegram messages with rate limiting and exponential backoff."""
        try:
            # Apply rate limiting before making request
            await self.rate_limiter.wait_for_rate_limit(is_send=False)

            url = f"{self.api_base}/getUpdates"
            params = {
                "offset": self._last_update_id + 1,
                "timeout": 1,  # Short poll timeout
                "allowed_updates": ["message"],
            }

            resp = await self.client.get(url, params=params, timeout=5)

            if resp.status_code == 429:
                # Rate limited by Telegram - back off
                retry_after = int(resp.headers.get("Retry-After", 30))
                logger.warning(f"Rate limited by Telegram. Retry-After: {retry_after}s")
                self.rate_limiter.record_failure()
                await asyncio.sleep(retry_after)
                return

            if resp.status_code != 200:
                # API error - apply exponential backoff
                backoff = self.rate_limiter.record_failure()
                logger.warning(f"Telegram API error {resp.status_code}. Backing off {backoff:.1f}s")
                await asyncio.sleep(backoff)
                return

            data = resp.json()
            if not data.get("ok"):
                # API returned error - apply backoff
                backoff = self.rate_limiter.record_failure()
                logger.warning(f"Telegram API not OK: {data}. Backing off {backoff:.1f}s")
                await asyncio.sleep(backoff)
                return

            # Success! Reset backoff state
            self.rate_limiter.record_success()

            for update in data.get("result", []):
                update_id = update["update_id"]
                self._last_update_id = update_id

                # Check ledger for duplicate inbound (defense in depth)
                if self._ledger.is_inbound_processed(update_id):
                    logger.debug(f"Skipping already-processed update {update_id}")
                    continue

                await self._handle_telegram_update(update)

                # Record in ledger after successful processing
                message = update.get("message", {})
                user = message.get("from", {})
                username = user.get("username") or user.get("first_name") or "user"
                text = message.get("text", "") or message.get("caption", "") or "[attachment]"
                self._ledger.record_inbound(update_id, username, text)
                self._ledger.set_last_update_id(update_id)

        except httpx.TimeoutException:
            pass  # Expected on long poll timeout - not a failure
        except httpx.ConnectError as e:
            # Connection failed - back off
            backoff = self.rate_limiter.record_failure()
            logger.error(f"Telegram connection error: {e}. Backing off {backoff:.1f}s")
            await asyncio.sleep(backoff)
        except Exception as e:
            # Unexpected error - back off
            backoff = self.rate_limiter.record_failure()
            logger.error(f"Telegram poll error: {type(e).__name__}: {e}. Backing off {backoff:.1f}s")
            await asyncio.sleep(backoff)

    async def _download_telegram_file(self, file_id: str, filename: str) -> Optional[Path]:
        """
        Download a file from Telegram using file_id.

        Pattern discovered in telegram_voice_bridge.py - preserved here for future reference.
        Uses Telegram Bot API: getFile -> download from file_path
        """
        try:
            # Step 1: Get file path from Telegram (with rate limiting)
            await self.rate_limiter.wait_for_rate_limit(is_send=False)

            url = f"{self.api_base}/getFile"
            resp = await self.client.get(url, params={"file_id": file_id})

            if resp.status_code == 429:
                retry_after = int(resp.headers.get("Retry-After", 30))
                logger.warning(f"getFile rate limited. Retry-After: {retry_after}s")
                self.rate_limiter.record_failure()
                await asyncio.sleep(retry_after)
                return None

            if resp.status_code != 200:
                self.rate_limiter.record_failure()
                logger.error(f"getFile failed: {resp.status_code}")
                return None

            data = resp.json()
            if not data.get("ok"):
                self.rate_limiter.record_failure()
                logger.error(f"getFile error: {data}")
                return None

            self.rate_limiter.record_success()
            file_path = data["result"]["file_path"]

            # Step 2: Download the file (with rate limiting)
            await self.rate_limiter.wait_for_rate_limit(is_send=False)

            file_url = f"https://api.telegram.org/file/bot{self.bot_token}/{file_path}"
            file_resp = await self.client.get(file_url)

            if file_resp.status_code == 429:
                retry_after = int(file_resp.headers.get("Retry-After", 30))
                logger.warning(f"File download rate limited. Retry-After: {retry_after}s")
                self.rate_limiter.record_failure()
                await asyncio.sleep(retry_after)
                return None

            if file_resp.status_code != 200:
                self.rate_limiter.record_failure()
                logger.error(f"File download failed: {file_resp.status_code}")
                return None

            self.rate_limiter.record_success()

            # Step 3: Save to attachment directory with timestamp
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            safe_filename = re.sub(r'[^\w\-_\.]', '_', filename)
            save_path = ATTACHMENT_DIR / f"{timestamp}_{safe_filename}"

            save_path.write_bytes(file_resp.content)
            logger.info(f"Downloaded attachment to {save_path}")

            return save_path

        except httpx.ConnectError as e:
            self.rate_limiter.record_failure()
            logger.error(f"File download connection error: {e}")
            return None
        except Exception as e:
            self.rate_limiter.record_failure()
            logger.error(f"File download error: {e}")
            return None

    async def _handle_telegram_update(self, update: Dict):
        """Handle a single Telegram update."""
        message = update.get("message")
        if not message:
            return

        user = message.get("from", {})
        user_id = str(user.get("id", ""))
        username = user.get("username") or user.get("first_name") or "user"

        # Extract chat context for multi-chat support
        chat = message.get("chat", {})
        chat_id = chat.get("id", "")
        chat_type = chat.get("type", "private")  # "private", "group", "supergroup", "channel"
        chat_title = chat.get("title", "")  # Only present for groups/supergroups/channels

        # Normalize chat type for display
        if chat_type in ("supergroup", "channel"):
            chat_type = "group"  # Treat supergroups/channels as groups for simplicity

        # Authorization check
        if user_id not in self.authorized_users:
            logger.warning(f"Unauthorized user {user_id} attempted access")
            return

        # Check for attachments first
        attachment_path = None
        attachment_type = None

        # Handle photo attachments (array of sizes, get largest)
        if message.get("photo"):
            photos = message["photo"]
            largest = photos[-1]  # Last is largest
            file_id = largest["file_id"]
            attachment_path = await self._download_telegram_file(file_id, "photo.jpg")
            attachment_type = "PHOTO"

        # Handle document attachments
        elif message.get("document"):
            doc = message["document"]
            file_id = doc["file_id"]
            filename = doc.get("file_name", "document")
            attachment_path = await self._download_telegram_file(file_id, filename)
            attachment_type = "DOCUMENT"

        # Handle voice attachments
        elif message.get("voice"):
            voice = message["voice"]
            file_id = voice["file_id"]
            attachment_path = await self._download_telegram_file(file_id, "voice.ogg")
            attachment_type = "VOICE"

        # Handle video attachments
        elif message.get("video"):
            video = message["video"]
            file_id = video["file_id"]
            filename = video.get("file_name", "video.mp4")
            attachment_path = await self._download_telegram_file(file_id, filename)
            attachment_type = "VIDEO"

        # If we got an attachment, inject notification
        if attachment_path:
            # Format with multi-chat context per telegram-multi-chat skill
            if chat_type == "group" and chat_title:
                attachment_msg = f"[TELEGRAM group:{chat_id} \"{chat_title}\" from @{username}] {attachment_type} saved to: {attachment_path}"
            else:
                attachment_msg = f"[TELEGRAM {chat_type}:{chat_id} from @{username}] {attachment_type} saved to: {attachment_path}"
            logger.info(attachment_msg)
            await self._send_message(f"Downloaded {attachment_type.lower()} to: {attachment_path}")

            # Inject to tmux so Claude knows about the file
            if self.tmux_pane:
                try:
                    subprocess.run(
                        ["tmux", "send-keys", "-t", self.tmux_pane, "-l", attachment_msg],
                        check=True, timeout=5
                    )
                    # 5x Enter retries - AICIV standard for ALL prompt injection
                    for _ in range(5):
                        await asyncio.sleep(0.4)
                        subprocess.run(
                            ["tmux", "send-keys", "-t", self.tmux_pane, "Enter"],
                            check=True, timeout=2
                        )
                except Exception as e:
                    logger.error(f"tmux injection failed for attachment: {e}")

            # Also handle any caption text
            caption = message.get("caption", "")
            if caption:
                await self._inject_and_respond(f"[Caption: {caption}]", username, chat_id, chat_type, chat_title)
            return

        # Handle text messages
        text = message.get("text", "")
        if not text:
            return

        logger.info(f"Message from @{username} in {chat_type}:{chat_id}: {text[:50]}...")

        # Handle commands
        if text.startswith("/"):
            await self._handle_command(text, user_id, username)
            return

        # Inject to tmux with chat context
        await self._inject_and_respond(text, username, chat_id, chat_type, chat_title)

    async def _handle_command(self, text: str, user_id: str, username: str):
        """Handle bot commands."""
        cmd = text.split()[0].lower()

        if cmd == "/start" or cmd == "/help":
            voice_status = "ON" if self.is_voice_mode_enabled(user_id) else "OFF"
            help_text = f"""A-C-Gee Telegram Bridge

Commands:
/start, /help - This message
/ping - Health check
/status - Session status
/voice_mode - Toggle voice summaries ({voice_status})

Just send a message to talk to Claude.
Responses stream automatically.

Voice Mode: When ON, you'll receive succinct voice summaries alongside text responses."""
            await self._send_message(help_text)

        elif cmd == "/ping":
            tmux_ok = self._check_tmux_session()
            claude_ok = self.current_session is not None
            await self._send_message(
                f"Pong!\n"
                f"tmux: {'OK' if tmux_ok else 'NOT FOUND'} ({self.tmux_session})\n"
                f"Claude log: {'OK' if claude_ok else 'NOT FOUND'}"
            )

        elif cmd == "/status":
            voice_status = "ON" if self.is_voice_mode_enabled(user_id) else "OFF"

            # Rate limiter status
            rl = self.rate_limiter
            if rl.is_stopped:
                api_status = "STOPPED (waiting for recovery)"
            elif rl.is_degraded:
                api_status = f"DEGRADED (polling every {rl.slow_poll_interval:.0f}s)"
            else:
                api_status = "HEALTHY"

            # Calculate uptime since last success
            uptime = time.time() - rl.last_success if rl.consecutive_failures == 0 else 0
            downtime = time.time() - (rl.downtime_start or time.time()) if rl.consecutive_failures > 0 else 0

            # Ledger stats
            ledger_stats = self._ledger.get_stats()

            # Gateway status
            if self.use_gateway:
                gw_status = f"ACTIVE ({self.gateway_url})" if self._gateway_registered else f"ENABLED but not registered ({self.gateway_url})"
            else:
                gw_status = "DISABLED (tmux mode)"

            status = f"""Session Status:
tmux: {self.tmux_pane or 'Not detected'}
Claude session: {self.current_session or 'Not found'}
Log path: {self.current_log_path or 'None'}
Voice mode: {voice_status}
Gateway: {gw_status}

Delivery Ledger (persistent):
- Outbound tracked: {ledger_stats['outbound_count']}
- Inbound tracked: {ledger_stats['inbound_count']}
- Last update ID: {ledger_stats['last_update_id']}

API Status: {api_status}
Consecutive failures: {rl.consecutive_failures}
Current backoff: {rl.backoff_seconds:.1f}s
Last success: {uptime:.0f}s ago""" + (f"\nDowntime: {downtime:.0f}s" if downtime > 0 else "")
            await self._send_message(status)

        elif cmd == "/voice_mode":
            new_state = self.toggle_voice_mode(user_id)
            state_text = "ON" if new_state else "OFF"
            emoji = "🔊" if new_state else "🔇"
            await self._send_message(
                f"{emoji} Voice mode: {state_text}\n\n"
                f"{'You will now receive voice summaries with text responses.' if new_state else 'Voice summaries disabled. Text-only mode.'}"
            )

    async def _inject_and_respond(self, text: str, username: str, chat_id: int = 0, chat_type: str = "private", chat_title: str = ""):
        """Inject message to Claude and await response.

        Gateway mode (use_gateway=True):
          - Sends via POST /api/inject
          - Polls GET /api/response every 2s until ready
          - Delivers response directly to Telegram
          - Falls back to tmux on any gateway error

        tmux mode (use_gateway=False or fallback):
          - Injects via tmux send-keys (original behavior)
          - Response comes via log streaming (_poll_claude_logs)

        Format per telegram-multi-chat skill:
        - Private: [TELEGRAM private:CHAT_ID from @username] message
        - Group: [TELEGRAM group:CHAT_ID "Group Title" from @username] message
        """
        # Deduplication check (defense in depth against multiple bot instances)
        msg_hash = hashlib.md5(f"{text}:{username}:{chat_id}".encode()).hexdigest()
        if msg_hash in self._recent_injections:
            logger.warning(f"Duplicate injection prevented: {text[:30]}...")
            return
        self._recent_injections.append(msg_hash)

        # Format with multi-chat context per telegram-multi-chat skill
        if chat_type == "group" and chat_title:
            formatted = f"[TELEGRAM group:{chat_id} \"{chat_title}\" from @{username}] {text}"
        else:
            formatted = f"[TELEGRAM {chat_type}:{chat_id} from @{username}] {text}"

        # ---- Gateway path ----
        if self.use_gateway and self._gateway_registered:
            injected = await self._gateway_inject(formatted)
            if injected:
                await self._send_message("Message received. Waiting for response...")
                response = await self._gateway_poll_response()
                if response:
                    chunks = self._chunk_message(response)
                    for i, chunk in enumerate(chunks):
                        if len(chunks) > 1:
                            await self._send_message(f"[{i+1}/{len(chunks)}]\n\n{chunk}")
                        else:
                            await self._send_message(chunk)
                        if i < len(chunks) - 1:
                            await asyncio.sleep(0.3)
                else:
                    await self._send_message(
                        "No response received from gateway (timeout or stalled). "
                        "Check Claude session status."
                    )
                return
            else:
                # Gateway inject failed — fall through to tmux
                logger.warning("Gateway inject failed, falling back to tmux injection")
                await self._send_message("Gateway unavailable, falling back to direct injection...")

        # ---- tmux path (original behavior) ----
        if not self.tmux_pane:
            await self._send_message("Error: No tmux session detected. Cannot inject.")
            return

        try:
            # Send text to tmux using chunked approach for reliability
            # Issue: Large messages (>100 chars) can silently fail with tmux send-keys
            # Root cause: tmux input buffer overflow when sending large strings at once
            # Solution: Send in chunks with small delays between them
            CHUNK_SIZE = 100  # Characters per chunk

            if len(formatted) > CHUNK_SIZE:
                # Chunked sending for large messages
                for i in range(0, len(formatted), CHUNK_SIZE):
                    chunk = formatted[i:i+CHUNK_SIZE]
                    subprocess.run(
                        ["tmux", "send-keys", "-t", self.tmux_pane, "-l", chunk],
                        check=True, timeout=5
                    )
                    await asyncio.sleep(0.05)  # 50ms between chunks
                logger.debug(f"Sent {len(formatted)} chars in {(len(formatted) + CHUNK_SIZE - 1) // CHUNK_SIZE} chunks")
            else:
                # Single send for short messages
                subprocess.run(
                    ["tmux", "send-keys", "-t", self.tmux_pane, "-l", formatted],
                    check=True, timeout=5
                )

            # 5x Enter retries - AICIV standard for ALL prompt injection
            # Ensures Claude processes the injected message reliably
            try:
                for i in range(5):
                    await asyncio.sleep(0.4)
                    subprocess.run(
                        ["tmux", "send-keys", "-t", self.tmux_pane, "Enter"],
                        check=True, timeout=2
                    )
                logger.debug("5x Enter injection complete")
            except subprocess.CalledProcessError as e:
                logger.error(f"Enter key injection failed: {e}")
                await self._send_message("Warning: Enter key may not have registered. Send '.' if needed.")
                return

            logger.info(f"Injected to tmux: {formatted[:50]}...")

            # Acknowledge injection (response will come via log stream)
            await self._send_message(f"Injected to Claude. Awaiting response...")

        except subprocess.CalledProcessError as e:
            logger.error(f"tmux injection failed: {e}")
            await self._send_message(f"Error: tmux injection failed")
        except subprocess.TimeoutExpired:
            logger.error("tmux injection timed out")
            await self._send_message("Error: tmux injection timed out")

    # ========== Claude Log Streaming ==========

    async def _poll_claude_logs(self):
        """Poll Claude logs for new entries."""
        try:
            # Check if session changed
            session_id = self._find_claude_session()
            if session_id and session_id != self.current_session:
                self._switch_session(session_id)
                logger.info(f"Switched to Claude session: {session_id}")

            if not self.current_log_path or not self.current_log_path.exists():
                return

            # Read new entries
            entries = self._read_new_entries()

            for entry in entries:
                await self._send_claude_entry(entry)

        except Exception as e:
            logger.error(f"Claude log poll error: {e}")

    def _find_claude_session(self) -> Optional[str]:
        """Find current Claude session ID from history."""
        try:
            if not HISTORY_FILE.exists():
                return None

            with HISTORY_FILE.open("r") as f:
                f.seek(0, 2)
                length = f.tell()
                window = min(16384, length)
                f.seek(max(0, length - window))
                lines = f.read().splitlines()

            for line in reversed(lines):
                if not line.strip():
                    continue
                try:
                    entry = json.loads(line)
                    if PROJECT_MATCH in entry.get("project", ""):
                        return entry.get("sessionId")
                except json.JSONDecodeError:
                    continue

            return None
        except Exception as e:
            logger.error(f"Error finding Claude session: {e}")
            return None

    def _switch_session(self, session_id: str):
        """Switch to a new Claude session.

        IMPORTANT: We do NOT clear sent_ids anymore. The persistent ledger
        tracks all sent messages across sessions, preventing duplicates
        even after session switches or restarts.
        """
        self.current_session = session_id
        self.current_log_path = LOG_ROOT / f"{session_id}.jsonl"
        # NO MORE: self._sent_ids.clear()  # Ledger persists across sessions
        self._ledger.set_session_id(session_id)

        # Seek to end of file (don't replay history)
        if self.current_log_path.exists():
            with self.current_log_path.open("r") as f:
                f.seek(0, 2)
                self.last_position = f.tell()
        else:
            self.last_position = 0

    def _read_new_entries(self) -> List[Dict]:
        """Read new entries from Claude log.

        Uses persistent ledger for deduplication instead of in-memory set.
        This survives restarts and session switches.
        """
        entries = []

        try:
            with self.current_log_path.open("r") as f:
                f.seek(self.last_position)
                while True:
                    line = f.readline()
                    if not line:
                        break
                    self.last_position = f.tell()

                    line = line.strip()
                    if not line:
                        continue

                    try:
                        entry = json.loads(line)
                        payload = self._build_payload(entry)
                        if payload and not self._ledger.is_outbound_sent(payload["id"]):
                            entries.append(payload)
                    except json.JSONDecodeError:
                        continue

        except FileNotFoundError:
            pass
        except Exception as e:
            logger.error(f"Error reading Claude log: {e}")

        return entries

    def _build_payload(self, entry: Dict) -> Optional[Dict]:
        """Build payload from Claude log entry."""
        message = entry.get("message", {})
        content_blocks = message.get("content", []) or []
        text_parts = []

        for block in content_blocks:
            if isinstance(block, str):
                if block.strip():
                    text_parts.append(block.strip())
            elif isinstance(block, dict):
                if block.get("type") == "text":
                    text = (block.get("text") or "").strip()
                    if text:
                        text_parts.append(text)

        if not text_parts:
            return None

        combined_text = "\n\n".join(text_parts)
        role = message.get("role", entry.get("type", "assistant"))

        # Only forward assistant messages to avoid echo
        if role != "assistant":
            return None

        # Process timestamp
        timestamp = entry.get("timestamp")
        if isinstance(timestamp, (int, float)):
            timestamp = datetime.fromtimestamp(timestamp / 1000, tz=timezone.utc).isoformat()
        elif not timestamp:
            timestamp = datetime.utcnow().replace(tzinfo=timezone.utc).isoformat()

        return {
            "id": entry.get("uuid") or f"log-{timestamp}",
            "role": role,
            "text": combined_text,
            "timestamp": timestamp,
        }

    async def _send_claude_entry(self, entry: Dict):
        """Send a Claude log entry to Telegram.

        Records successful sends in the persistent ledger.
        """
        text = entry.get("text", "")
        message_id = entry.get("id", "")
        if not text:
            return

        # Send text directly without header (cleaner output)
        full_message = text

        # Chunk if needed
        chunks = self._chunk_message(full_message)

        for i, chunk in enumerate(chunks):
            if len(chunks) > 1:
                # Add continuation marker
                marker = f"[{i+1}/{len(chunks)}]\n\n"
                await self._send_message(marker + chunk)
            else:
                await self._send_message(chunk)

            # Small delay between chunks
            if i < len(chunks) - 1:
                await asyncio.sleep(0.3)

        # Record in ledger AFTER successful send
        self._ledger.record_outbound(message_id, None, text)

        # Check if voice mode is enabled for any authorized user
        # Send voice summary after text is complete
        for user_id in self.authorized_users.keys():
            if self.is_voice_mode_enabled(user_id):
                await self._send_voice_summary(text, user_id)
                break  # Only send once (to primary chat)

    async def _send_voice_summary(self, text: str, user_id: str):
        """Generate and send a voice summary of the text."""
        try:
            # Generate voice-optimized summary
            summary = self._generate_voice_summary(text)

            if not summary or len(summary) < 10:
                logger.debug("Skipping voice summary - text too short or no summary generated")
                return

            logger.info(f"Sending voice summary: {summary[:50]}...")

            # Use send_telegram_voice.py to send voice message
            if not VOICE_SCRIPT.exists():
                logger.error(f"Voice script not found: {VOICE_SCRIPT}")
                return

            # Run voice script - using subprocess for simplicity
            # Note: user_id is from authorized_users (trusted), summary is generated text
            result = subprocess.run(
                ["python3", str(VOICE_SCRIPT), user_id, summary],
                capture_output=True,
                text=True,
                timeout=60
            )

            if result.returncode == 0:
                logger.info(f"Voice summary sent successfully")
            else:
                logger.warning(f"Voice summary failed: {result.stderr}")

        except subprocess.TimeoutExpired:
            logger.error("Voice summary generation timed out")
        except Exception as e:
            logger.error(f"Failed to send voice summary: {e}")

    def _generate_voice_summary(self, text: str) -> str:
        """
        Generate a concise, voice-friendly summary of Claude's response.

        Voice summaries should be:
        - Short (2-3 sentences max)
        - Conversational tone
        - Focus on key action/answer
        - Skip technical details, code blocks, lists
        """
        # Skip very short messages
        if len(text) < 100:
            # For short messages, just use the text directly if it's voice-friendly
            # Skip if it looks like code or technical output
            if '```' in text or text.count('\n') > 5 or '{' in text:
                return ""
            return text.strip()

        # Skip pure code responses
        if text.count('```') >= 2:
            # Extract any text before/after code blocks
            parts = text.split('```')
            non_code = ' '.join(parts[::2])  # Get even indices (non-code parts)
            if len(non_code.strip()) < 50:
                return "I've generated some code for you. Check the text message for details."

        # For longer messages, extract key points
        lines = text.split('\n')

        # Look for summary indicators
        summary_lines = []
        for line in lines:
            line = line.strip()
            if not line:
                continue
            # Skip code blocks, bullet points, technical patterns
            if line.startswith('```') or line.startswith('- ') or line.startswith('* '):
                continue
            if line.startswith('#'):  # Headers
                continue
            if '{' in line or '}' in line or line.startswith('|'):  # Code/tables
                continue
            if len(line) < 10:
                continue

            summary_lines.append(line)

            # Take first 2-3 meaningful lines
            if len(summary_lines) >= 3:
                break

        if not summary_lines:
            return "I've provided a detailed response. Check the text for specifics."

        # Combine and truncate
        summary = ' '.join(summary_lines)

        # Truncate to reasonable voice length (approx 15-20 seconds of speech)
        max_chars = 300
        if len(summary) > max_chars:
            # Find last sentence boundary
            for punct in ['. ', '! ', '? ']:
                last_punct = summary[:max_chars].rfind(punct)
                if last_punct > 100:
                    summary = summary[:last_punct + 1]
                    break
            else:
                summary = summary[:max_chars] + "..."

        return summary.strip()

    # ========== Message Chunking ==========

    def _chunk_message(self, text: str, max_len: int = TG_SAFE_LEN) -> List[str]:
        """
        Chunk long messages intelligently.

        Priority:
        1. Paragraph breaks (double newline)
        2. Line breaks (single newline)
        3. Sentence boundaries (. ! ?)
        4. Word boundaries (space)
        """
        if len(text) <= max_len:
            return [text]

        chunks = []
        current = ""

        # Split by paragraphs first
        paragraphs = text.split('\n\n')

        for para in paragraphs:
            test = current + ('\n\n' if current else '') + para

            if len(test) <= max_len:
                current = test
            else:
                # Save current chunk
                if current:
                    chunks.append(current)
                    current = ""

                # Check if paragraph itself fits
                if len(para) <= max_len:
                    current = para
                else:
                    # Split paragraph by lines
                    para_chunks = self._chunk_by_lines(para, max_len)
                    for pc in para_chunks[:-1]:
                        chunks.append(pc)
                    current = para_chunks[-1] if para_chunks else ""

        if current:
            chunks.append(current)

        return chunks if chunks else [text]

    def _chunk_by_lines(self, text: str, max_len: int) -> List[str]:
        """Chunk text by line breaks."""
        chunks = []
        current = ""

        for line in text.split('\n'):
            test = current + ('\n' if current else '') + line

            if len(test) <= max_len:
                current = test
            else:
                if current:
                    chunks.append(current)
                    current = ""

                if len(line) <= max_len:
                    current = line
                else:
                    # Split line by words
                    word_chunks = self._chunk_by_words(line, max_len)
                    for wc in word_chunks[:-1]:
                        chunks.append(wc)
                    current = word_chunks[-1] if word_chunks else ""

        if current:
            chunks.append(current)

        return chunks

    def _chunk_by_words(self, text: str, max_len: int) -> List[str]:
        """Chunk text by word boundaries (last resort)."""
        chunks = []
        current = ""

        for word in text.split(' '):
            test = current + (' ' if current else '') + word

            if len(test) <= max_len:
                current = test
            else:
                if current:
                    chunks.append(current)
                current = word[:max_len]  # Truncate very long words

        if current:
            chunks.append(current)

        return chunks

    # ========== Markdown Conversion ==========

    def _markdown_to_telegram_html(self, text: str) -> str:
        """
        Convert standard markdown to Telegram-compatible HTML.

        Telegram HTML supports: <b>, <i>, <u>, <s>, <code>, <pre>, <a href="">
        Standard markdown we need to handle:
        - ### Headers -> <b>Header</b>
        - **bold** -> <b>bold</b>
        - *italic* or _italic_ -> <i>italic</i>
        - `code` -> <code>code</code>
        - ```code blocks``` -> <pre>code</pre>
        - [link](url) -> <a href="url">link</a>
        - ~~strikethrough~~ -> <s>strikethrough</s>
        """
        result = text

        # First, escape HTML special chars that aren't part of our formatting
        # We'll do this carefully to not break intentional HTML
        result = result.replace('&', '&amp;')
        result = result.replace('<', '&lt;')
        result = result.replace('>', '&gt;')

        # Code blocks (``` ... ```) - must be done before inline code
        # Handle with language specifier: ```python ... ```
        result = re.sub(
            r'```(?:\w+)?\n?(.*?)```',
            r'<pre>\1</pre>',
            result,
            flags=re.DOTALL
        )

        # Inline code (`code`)
        result = re.sub(r'`([^`]+)`', r'<code>\1</code>', result)

        # Headers (### Header) - convert to bold
        # Handle h1-h6
        result = re.sub(r'^#{1,6}\s+(.+?)$', r'<b>\1</b>', result, flags=re.MULTILINE)

        # Bold (**text** or __text__)
        result = re.sub(r'\*\*(.+?)\*\*', r'<b>\1</b>', result)
        result = re.sub(r'__(.+?)__', r'<b>\1</b>', result)

        # Italic (*text* or _text_) - be careful not to match underscores in words
        # Only match *text* when not preceded/followed by *
        result = re.sub(r'(?<!\*)\*([^\*\n]+?)\*(?!\*)', r'<i>\1</i>', result)
        # For underscores, only at word boundaries
        result = re.sub(r'(?<!\w)_([^_\n]+?)_(?!\w)', r'<i>\1</i>', result)

        # Strikethrough (~~text~~)
        result = re.sub(r'~~(.+?)~~', r'<s>\1</s>', result)

        # Links [text](url)
        result = re.sub(r'\[([^\]]+)\]\(([^\)]+)\)', r'<a href="\2">\1</a>', result)

        # Bullet points: convert "- item" or "* item" to proper bullets
        # Telegram doesn't have list support, so we just clean up the markers
        result = re.sub(r'^[\-\*]\s+', '- ', result, flags=re.MULTILINE)

        return result

    # ========== Telegram API ==========

    async def _send_message(self, text: str, use_html: bool = True, _retry_count: int = 0):
        """Send message to Telegram with proper formatting and rate limiting."""
        if not self.client:
            return

        # Max retries for send operations
        MAX_SEND_RETRIES = 3

        try:
            # Apply rate limiting before sending
            await self.rate_limiter.wait_for_rate_limit(is_send=True)

            url = f"{self.api_base}/sendMessage"

            # Convert markdown to Telegram HTML
            if use_html:
                formatted_text = self._markdown_to_telegram_html(text)
            else:
                formatted_text = text

            data = {
                "chat_id": self.chat_id,
                "text": formatted_text[:TG_MAX_MESSAGE_LEN],  # Safety truncation
                "parse_mode": "HTML" if use_html else None,
            }
            # Remove None values
            data = {k: v for k, v in data.items() if v is not None}

            resp = await self.client.post(url, json=data)

            if resp.status_code == 429:
                # Rate limited by Telegram
                retry_after = int(resp.headers.get("Retry-After", 30))
                logger.warning(f"Send rate limited. Retry-After: {retry_after}s")
                self.rate_limiter.record_failure()
                if _retry_count < MAX_SEND_RETRIES:
                    await asyncio.sleep(retry_after)
                    await self._send_message(text, use_html, _retry_count + 1)
                return

            if resp.status_code != 200:
                # If HTML parsing failed, try plain text as fallback
                if use_html and resp.status_code == 400:
                    logger.warning(f"HTML parse failed, falling back to plain text")
                    await self._send_message(text, use_html=False, _retry_count=_retry_count)
                else:
                    backoff = self.rate_limiter.record_failure()
                    logger.warning(f"Telegram send failed: {resp.status_code} {resp.text}")
                    if _retry_count < MAX_SEND_RETRIES:
                        await asyncio.sleep(backoff)
                        await self._send_message(text, use_html, _retry_count + 1)
                return

            # Success - reset backoff
            self.rate_limiter.record_success()

        except httpx.ConnectError as e:
            backoff = self.rate_limiter.record_failure()
            logger.error(f"Telegram send connection error: {e}")
            if _retry_count < MAX_SEND_RETRIES:
                await asyncio.sleep(backoff)
                await self._send_message(text, use_html, _retry_count + 1)
        except Exception as e:
            backoff = self.rate_limiter.record_failure()
            logger.error(f"Telegram send error: {type(e).__name__}: {e}")
            if _retry_count < MAX_SEND_RETRIES:
                await asyncio.sleep(backoff)
                await self._send_message(text, use_html, _retry_count + 1)
            else:
                # Log full traceback for debugging on final failure
                import traceback
                logger.debug(f"Full traceback: {traceback.format_exc()}")

    # ========== tmux Utilities ==========

    def _detect_acg_session(self) -> Optional[str]:
        """Auto-detect ACG tmux session."""
        try:
            # Method 1: Check marker file
            marker_file = PROJECT_ROOT / ".current_session"
            if marker_file.exists():
                session_name = marker_file.read_text().strip()
                result = subprocess.run(
                    ["tmux", "has-session", "-t", session_name],
                    capture_output=True, timeout=5
                )
                if result.returncode == 0:
                    return session_name

            # Method 2: List sessions
            result = subprocess.run(
                ["tmux", "list-sessions", "-F", "#{session_name}"],
                capture_output=True, text=True, timeout=5
            )

            if result.returncode != 0:
                return None

            sessions = result.stdout.strip().split('\n')
            acg_sessions = [s for s in sessions if s.startswith(f'{CIV_ID}-primary-')]

            if acg_sessions:
                return sorted(acg_sessions)[-1]  # Latest

            return None

        except Exception as e:
            logger.error(f"Session detection error: {e}")
            return None

    def _check_tmux_session(self) -> bool:
        """Check if tmux session exists."""
        if not self.tmux_session:
            return False
        try:
            result = subprocess.run(
                ["tmux", "has-session", "-t", self.tmux_session],
                capture_output=True, timeout=5
            )
            return result.returncode == 0
        except:
            return False


def load_config() -> Dict:
    """Load configuration from file."""
    if not CONFIG_FILE.exists():
        raise FileNotFoundError(f"Config file not found: {CONFIG_FILE}")

    with CONFIG_FILE.open() as f:
        config = json.load(f)

    if not config.get("bot_token"):
        raise ValueError("No bot_token in config")
    if not config.get("chat_id"):
        raise ValueError("No chat_id in config")

    return config


async def send_one_message(message: str) -> int:
    """Send a single message and exit immediately (no polling)."""
    try:
        config = load_config()
    except Exception as e:
        logger.error(f"Config error: {e}")
        return 1

    async with httpx.AsyncClient(timeout=30) as client:
        url = f"https://api.telegram.org/bot{config['bot_token']}/sendMessage"
        data = {
            "chat_id": config["chat_id"],
            "text": message[:TG_MAX_MESSAGE_LEN],
        }
        resp = await client.post(url, json=data)
        if resp.status_code == 200:
            logger.info(f"Message sent: {message[:50]}...")
            return 0
        else:
            logger.error(f"Send failed: {resp.status_code} {resp.text}")
            return 1


def ensure_single_instance():
    """
    Ensure only one bot instance runs at a time.
    Uses file locking (flock) which is automatically released on process exit.
    Returns the lock file handle (must be kept open).
    """
    try:
        # Open file for writing (create if doesn't exist)
        lock_file = open(PID_LOCK_FILE, 'w')
        # Try to get exclusive lock (non-blocking)
        fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        # Write our PID
        lock_file.write(str(os.getpid()))
        lock_file.flush()
        logger.info(f"Acquired singleton lock (PID: {os.getpid()})")
        return lock_file  # Must keep file open to maintain lock
    except IOError:
        # Lock acquisition failed - another instance is running
        try:
            with open(PID_LOCK_FILE, 'r') as f:
                existing_pid = f.read().strip()
            logger.error(f"Another bot instance is running (PID: {existing_pid}). Exiting.")
        except:
            logger.error("Another bot instance is running. Exiting.")
        sys.exit(1)


async def main():
    """Main entry point."""
    # Handle send mode: python3 telegram_unified.py send "message"
    if len(sys.argv) >= 3 and sys.argv[1] == "send":
        message = " ".join(sys.argv[2:])
        return await send_one_message(message)

    # Bot mode (no args or unrecognized args): run polling loop
    # CRITICAL: Ensure only one instance runs to prevent message duplication
    lock_file = ensure_single_instance()

    logger.info("Starting A-C-Gee Unified Telegram Bot")

    try:
        config = load_config()
        logger.info("Configuration loaded")
    except Exception as e:
        logger.error(f"Config error: {e}")
        return 1

    bot = TelegramBot(config)
    await bot.run()
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(asyncio.run(main()))
