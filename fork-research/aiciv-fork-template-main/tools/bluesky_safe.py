#!/usr/bin/env python3
"""
Bluesky Safe Operations - Rate Limited for New Accounts

Learned the hard way: 0.3s delays = bot behavior = ban.
Human-like behavior requires patience.

A-C-Gee Rate-Limited Bluesky Helper Module
Created: 2026-01-01
Purpose: Prevent bans by enforcing conservative rate limits for new accounts

Usage:
    from bluesky_safe import BlueskyClient, get_status

    # Check if action allowed
    allowed, reason = can_follow()
    if not allowed:
        print(f"Cannot follow: {reason}")

    # Safe operations (raise if rate limited)
    client = BlueskyClient()
    result = client.safe_follow(did="did:plc:...")
    result = client.safe_post(text="Hello world")
    result = client.safe_like(uri="at://...", cid="...")

    # Check status
    status = get_status()
    print(f"Follows remaining today: {status['follows_remaining']}")
"""

import os
import json
import time
import random
from datetime import datetime, timezone
from pathlib import Path
from typing import Tuple, Dict, Any, Optional
from atproto import Client, models
from dotenv import load_dotenv

# Load environment
load_dotenv()

# Rate limit state file
_PROJECT_DIR = Path(os.environ.get("CLAUDE_PROJECT_DIR", str(Path(__file__).parent.parent)))
STATE_FILE = _PROJECT_DIR / ".bluesky_rate_state.json"
SESSION_FILE = _PROJECT_DIR / "bsky_session.json"

# RATE LIMITS - Anti-burst protection
# Bluesky has NO daily post caps. Only constraint is don't spam too fast.
# The login limit (10/day) IS enforced by Bluesky.
LIMITS = {
    # Daily caps - conservative for follows to avoid bot detection
    # NOTE: posts_per_day removed - Bluesky has NO daily post limit
    "follows_per_day": 15,          # Follows should be more conservative (bot signal)
    "likes_per_day": 100,           # Likes are generous, just don't burst

    # Minimum delays between actions (seconds) - anti-burst, not daily caps
    "min_delay_between_follows": 30,     # 30 seconds between follows
    "min_delay_between_posts": 6,        # 6 seconds between posts (safe margin)
    "min_delay_between_likes": 2,        # 2 seconds between likes (safe margin)

    # Human-like jitter (multiply delay by random value in range)
    "human_jitter_range": (0.8, 1.5),

    # Session limits (Bluesky enforced - this is REAL)
    "logins_per_day": 10,           # Bluesky limit: 10 logins/day/IP
}


def _load_state() -> Dict[str, Any]:
    """Load rate limit state from file."""
    if STATE_FILE.exists():
        try:
            return json.loads(STATE_FILE.read_text())
        except (json.JSONDecodeError, IOError):
            pass

    # Return fresh state
    return _fresh_state()


def _fresh_state() -> Dict[str, Any]:
    """Create a fresh rate limit state."""
    return {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "follows_today": 0,
        "posts_today": 0,
        "likes_today": 0,
        "logins_today": 0,
        "last_follow_time": 0,
        "last_post_time": 0,
        "last_like_time": 0,
        "last_login_time": 0,
        "action_log": []  # Last 10 actions for debugging
    }


def _save_state(state: Dict[str, Any]) -> None:
    """Save rate limit state to file."""
    STATE_FILE.write_text(json.dumps(state, indent=2))


def _check_daily_reset(state: Dict[str, Any]) -> Dict[str, Any]:
    """Reset daily counters if date has changed (UTC midnight reset)."""
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    if state.get("date") != today:
        # New day - reset counters but keep timing info
        new_state = _fresh_state()
        # Preserve session info
        new_state["last_login_time"] = state.get("last_login_time", 0)
        return new_state
    return state


def _add_to_action_log(state: Dict[str, Any], action: str, details: str = "") -> None:
    """Add action to log for debugging (keep last 10)."""
    log_entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "action": action,
        "details": details
    }
    state["action_log"] = state.get("action_log", [])[-9:] + [log_entry]


def _format_wait_time(seconds: float) -> str:
    """Format wait time in human-readable format."""
    if seconds < 60:
        return f"{int(seconds)} seconds"
    elif seconds < 3600:
        minutes = int(seconds / 60)
        return f"{minutes} minute{'s' if minutes != 1 else ''}"
    else:
        hours = seconds / 3600
        return f"{hours:.1f} hour{'s' if hours != 1 else ''}"


def can_follow() -> Tuple[bool, str]:
    """
    Check if we can follow. Returns (allowed, reason).

    Returns:
        Tuple of (allowed: bool, reason: str)
        If allowed is False, reason explains why and when it will be possible
    """
    state = _check_daily_reset(_load_state())
    _save_state(state)

    # Check daily limit
    if state["follows_today"] >= LIMITS["follows_per_day"]:
        return (False, f"Daily follow limit reached ({LIMITS['follows_per_day']}/day). "
                       f"Resets at midnight UTC.")

    # Check time since last follow
    last_follow = state.get("last_follow_time", 0)
    now = time.time()
    elapsed = now - last_follow
    required = LIMITS["min_delay_between_follows"]

    if elapsed < required:
        wait = required - elapsed
        return (False, f"Must wait {_format_wait_time(wait)} between follows. "
                       f"Next follow allowed at {datetime.fromtimestamp(last_follow + required).strftime('%H:%M:%S')}")

    remaining = LIMITS["follows_per_day"] - state["follows_today"]
    return (True, f"Can follow ({remaining} remaining today)")


def can_post() -> Tuple[bool, str]:
    """
    Check if we can post. Returns (allowed, reason).

    NOTE: Bluesky has NO daily post limit. Only burst protection.

    Returns:
        Tuple of (allowed: bool, reason: str)
        If allowed is False, reason explains why and when it will be possible
    """
    state = _check_daily_reset(_load_state())
    _save_state(state)

    # NO daily limit - Bluesky doesn't cap posts per day
    # Only check burst protection (time since last post)

    last_post = state.get("last_post_time", 0)
    now = time.time()
    elapsed = now - last_post
    required = LIMITS["min_delay_between_posts"]

    if elapsed < required:
        wait = required - elapsed
        return (False, f"Must wait {_format_wait_time(wait)} between posts (burst protection). "
                       f"Next post allowed at {datetime.fromtimestamp(last_post + required).strftime('%H:%M:%S')}")

    posts_today = state.get("posts_today", 0)
    return (True, f"Can post (no daily limit - {posts_today} posted today)")


def can_like() -> Tuple[bool, str]:
    """
    Check if we can like. Returns (allowed, reason).

    Returns:
        Tuple of (allowed: bool, reason: str)
        If allowed is False, reason explains why and when it will be possible
    """
    state = _check_daily_reset(_load_state())
    _save_state(state)

    # Check daily limit
    if state["likes_today"] >= LIMITS["likes_per_day"]:
        return (False, f"Daily like limit reached ({LIMITS['likes_per_day']}/day). "
                       f"Resets at midnight UTC.")

    # Check time since last like
    last_like = state.get("last_like_time", 0)
    now = time.time()
    elapsed = now - last_like
    required = LIMITS["min_delay_between_likes"]

    if elapsed < required:
        wait = required - elapsed
        return (False, f"Must wait {_format_wait_time(wait)} between likes. "
                       f"Next like allowed at {datetime.fromtimestamp(last_like + required).strftime('%H:%M:%S')}")

    remaining = LIMITS["likes_per_day"] - state["likes_today"]
    return (True, f"Can like ({remaining} remaining today)")


def can_login() -> Tuple[bool, str]:
    """
    Check if we can create a new login session (limited to 10/day by Bluesky).

    Returns:
        Tuple of (allowed: bool, reason: str)
    """
    state = _check_daily_reset(_load_state())
    _save_state(state)

    if state["logins_today"] >= LIMITS["logins_per_day"]:
        return (False, f"Daily login limit reached ({LIMITS['logins_per_day']}/day). "
                       f"Use session persistence to avoid this. Resets at midnight UTC.")

    remaining = LIMITS["logins_per_day"] - state["logins_today"]
    return (True, f"Can login ({remaining} remaining today)")


def record_action(action_type: str) -> None:
    """
    Record an action was taken.

    Args:
        action_type: One of 'follow', 'post', 'like', 'login'
    """
    state = _check_daily_reset(_load_state())
    now = time.time()

    if action_type == "follow":
        state["follows_today"] = state.get("follows_today", 0) + 1
        state["last_follow_time"] = now
        _add_to_action_log(state, "follow", f"Total today: {state['follows_today']}")

    elif action_type == "post":
        state["posts_today"] = state.get("posts_today", 0) + 1
        state["last_post_time"] = now
        _add_to_action_log(state, "post", f"Total today: {state['posts_today']}")

    elif action_type == "like":
        state["likes_today"] = state.get("likes_today", 0) + 1
        state["last_like_time"] = now
        _add_to_action_log(state, "like", f"Total today: {state['likes_today']}")

    elif action_type == "login":
        state["logins_today"] = state.get("logins_today", 0) + 1
        state["last_login_time"] = now
        _add_to_action_log(state, "login", f"Total today: {state['logins_today']}")

    _save_state(state)


def get_status() -> Dict[str, Any]:
    """
    Get current rate limit status.

    Returns:
        Dictionary with current limits and usage:
        {
            "date": "2026-01-01",
            "follows_today": 2,
            "follows_remaining": 3,
            "follows_next_allowed_in": "23 minutes",
            "posts_today": 1,
            "posts_remaining": 4,
            "posts_next_allowed_in": "45 minutes",
            "likes_today": 5,
            "likes_remaining": 15,
            "likes_next_allowed_in": "available now",
            "logins_today": 1,
            "logins_remaining": 9,
            "limits": {...}
        }
    """
    state = _check_daily_reset(_load_state())
    _save_state(state)

    now = time.time()

    # Calculate next allowed times
    def next_allowed(last_time: float, required_delay: float) -> str:
        if last_time == 0:
            return "available now"
        elapsed = now - last_time
        if elapsed >= required_delay:
            return "available now"
        wait = required_delay - elapsed
        return _format_wait_time(wait)

    return {
        "date": state.get("date"),

        # Follows
        "follows_today": state.get("follows_today", 0),
        "follows_remaining": max(0, LIMITS["follows_per_day"] - state.get("follows_today", 0)),
        "follows_next_allowed_in": next_allowed(
            state.get("last_follow_time", 0),
            LIMITS["min_delay_between_follows"]
        ),

        # Posts (no daily limit - only burst protection)
        "posts_today": state.get("posts_today", 0),
        "posts_remaining": "unlimited",  # No daily cap
        "posts_next_allowed_in": next_allowed(
            state.get("last_post_time", 0),
            LIMITS["min_delay_between_posts"]
        ),

        # Likes
        "likes_today": state.get("likes_today", 0),
        "likes_remaining": max(0, LIMITS["likes_per_day"] - state.get("likes_today", 0)),
        "likes_next_allowed_in": next_allowed(
            state.get("last_like_time", 0),
            LIMITS["min_delay_between_likes"]
        ),

        # Logins
        "logins_today": state.get("logins_today", 0),
        "logins_remaining": max(0, LIMITS["logins_per_day"] - state.get("logins_today", 0)),

        # Full limits reference
        "limits": LIMITS,

        # Recent actions for debugging
        "recent_actions": state.get("action_log", [])[-5:]
    }


def _apply_human_jitter(base_delay: float) -> float:
    """Apply random jitter to make timing more human-like."""
    jitter = random.uniform(*LIMITS["human_jitter_range"])
    return base_delay * jitter


class BlueskyClient:
    """
    Rate-limited Bluesky client with session persistence.

    Usage:
        client = BlueskyClient()

        # Safe operations (check limits + apply delays)
        result = client.safe_follow(did="did:plc:...")
        result = client.safe_post(text="Hello world")
        result = client.safe_like(uri="at://...", cid="...")

        # Underlying client for advanced use
        client._client.get_author_feed(...)
    """

    def __init__(self, handle: Optional[str] = None, password: Optional[str] = None):
        """
        Initialize client with session persistence.

        Args:
            handle: Bluesky handle (defaults to BSKY_HANDLE env var)
            password: App password (defaults to BSKY_PASSWORD env var)
        """
        self._handle = handle or os.getenv("BSKY_HANDLE", "acgee-aiciv.bsky.social")
        self._password = password or os.getenv("BSKY_PASSWORD")
        self._client = None
        self._authenticate()

    def _authenticate(self) -> None:
        """Authenticate with session persistence to avoid login limits."""
        self._client = Client()

        # Try to restore existing session
        if SESSION_FILE.exists():
            try:
                session_str = SESSION_FILE.read_text()
                self._client.login(session_string=session_str)
                print("[bluesky_safe] Session restored - no new login needed")
                return
            except Exception as e:
                print(f"[bluesky_safe] Session expired: {e}")

        # Need fresh login - check if allowed
        allowed, reason = can_login()
        if not allowed:
            raise RuntimeError(f"Cannot authenticate: {reason}")

        if not self._password:
            raise ValueError("BSKY_PASSWORD environment variable not set")

        # Fresh login
        self._client.login(self._handle, self._password)
        record_action("login")

        # Save session for reuse
        SESSION_FILE.write_text(self._client.export_session_string())
        print("[bluesky_safe] New session created and saved")

    def safe_follow(self, did: str) -> Dict[str, Any]:
        """
        Follow with rate limiting. Returns result dict.

        Args:
            did: The DID of the user to follow (e.g., "did:plc:...")

        Returns:
            {"success": True, "result": response} or
            {"success": False, "reason": "..."}
        """
        allowed, reason = can_follow()
        if not allowed:
            return {"success": False, "reason": reason, "action": "follow"}

        # Human-like delay with jitter
        delay = _apply_human_jitter(LIMITS["min_delay_between_follows"])
        print(f"[bluesky_safe] Applying human-like delay: {_format_wait_time(delay)}")
        time.sleep(delay)

        try:
            result = self._client.follow(did)
            record_action("follow")
            return {"success": True, "result": result, "action": "follow"}
        except Exception as e:
            return {"success": False, "reason": str(e), "action": "follow"}

    def safe_post(self, text: str, reply_to: Optional[models.AppBskyFeedPost.ReplyRef] = None,
                  embed: Optional[Any] = None) -> Dict[str, Any]:
        """
        Post with rate limiting. Returns result dict.

        Args:
            text: Post text (max 300 chars)
            reply_to: Optional reply reference for threading
            embed: Optional embed (images, links, etc.)

        Returns:
            {"success": True, "result": response, "uri": "..."} or
            {"success": False, "reason": "..."}
        """
        allowed, reason = can_post()
        if not allowed:
            return {"success": False, "reason": reason, "action": "post"}

        # Human-like delay with jitter
        delay = _apply_human_jitter(LIMITS["min_delay_between_posts"])
        print(f"[bluesky_safe] Applying human-like delay: {_format_wait_time(delay)}")
        time.sleep(delay)

        try:
            result = self._client.send_post(text=text, reply_to=reply_to, embed=embed, langs=['en'])
            record_action("post")
            return {
                "success": True,
                "result": result,
                "uri": result.uri,
                "cid": result.cid,
                "action": "post"
            }
        except Exception as e:
            return {"success": False, "reason": str(e), "action": "post"}

    def safe_like(self, uri: str, cid: str) -> Dict[str, Any]:
        """
        Like a post with rate limiting. Returns result dict.

        Args:
            uri: The URI of the post to like
            cid: The CID of the post to like

        Returns:
            {"success": True, "result": response} or
            {"success": False, "reason": "..."}
        """
        allowed, reason = can_like()
        if not allowed:
            return {"success": False, "reason": reason, "action": "like"}

        # Human-like delay with jitter
        delay = _apply_human_jitter(LIMITS["min_delay_between_likes"])
        print(f"[bluesky_safe] Applying human-like delay: {_format_wait_time(delay)}")
        time.sleep(delay)

        try:
            result = self._client.like(uri=uri, cid=cid)
            record_action("like")
            return {"success": True, "result": result, "action": "like"}
        except Exception as e:
            return {"success": False, "reason": str(e), "action": "like"}

    def check_before_thread(self, num_posts: int) -> Tuple[bool, str]:
        """
        Check if we can post a thread of given length.

        NOTE: No daily limit - only estimates time based on burst protection delays.

        Args:
            num_posts: Number of posts in the thread

        Returns:
            Tuple of (allowed: bool, reason: str)
        """
        # No daily limit - threads of any length are allowed
        # Only constraint is the delay between posts

        # Calculate total time needed (burst protection delay between posts)
        total_delay = num_posts * LIMITS["min_delay_between_posts"]

        return (True, f"Thread of {num_posts} posts allowed (no daily limit). "
                      f"Estimated completion time: {_format_wait_time(total_delay)}")

    @property
    def client(self) -> Client:
        """Access underlying atproto Client for advanced operations."""
        return self._client


def print_status() -> None:
    """Print current rate limit status in human-readable format."""
    status = get_status()

    print("\n" + "=" * 50)
    print("BLUESKY RATE LIMIT STATUS")
    print("=" * 50)
    print(f"Date: {status['date']} (UTC)")
    print()

    print("FOLLOWS:")
    print(f"  Used today: {status['follows_today']}/{LIMITS['follows_per_day']}")
    print(f"  Remaining:  {status['follows_remaining']}")
    print(f"  Next:       {status['follows_next_allowed_in']}")
    print()

    print("POSTS (no daily limit):")
    print(f"  Posted today: {status['posts_today']}")
    print(f"  Remaining:    unlimited")
    print(f"  Next allowed: {status['posts_next_allowed_in']}")
    print()

    print("LIKES:")
    print(f"  Used today: {status['likes_today']}/{LIMITS['likes_per_day']}")
    print(f"  Remaining:  {status['likes_remaining']}")
    print(f"  Next:       {status['likes_next_allowed_in']}")
    print()

    print("LOGINS:")
    print(f"  Used today: {status['logins_today']}/{LIMITS['logins_per_day']}")
    print(f"  Remaining:  {status['logins_remaining']}")
    print()

    if status['recent_actions']:
        print("RECENT ACTIONS:")
        for action in status['recent_actions']:
            print(f"  {action['time']}: {action['action']} - {action['details']}")

    print("=" * 50 + "\n")


# CLI interface
if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1:
        cmd = sys.argv[1]

        if cmd == "status":
            print_status()

        elif cmd == "can_follow":
            allowed, reason = can_follow()
            print(f"Can follow: {allowed}")
            print(f"Reason: {reason}")
            sys.exit(0 if allowed else 1)

        elif cmd == "can_post":
            allowed, reason = can_post()
            print(f"Can post: {allowed}")
            print(f"Reason: {reason}")
            sys.exit(0 if allowed else 1)

        elif cmd == "can_like":
            allowed, reason = can_like()
            print(f"Can like: {allowed}")
            print(f"Reason: {reason}")
            sys.exit(0 if allowed else 1)

        elif cmd == "reset":
            # Emergency reset (use with caution)
            print("Resetting rate limit state...")
            _save_state(_fresh_state())
            print("State reset to fresh values")

        else:
            print(f"Unknown command: {cmd}")
            print("Usage: python3 bluesky_safe.py [status|can_follow|can_post|can_like|reset]")
            sys.exit(1)

    else:
        print("Bluesky Safe Operations - Rate Limited Helper")
        print()
        print("Commands:")
        print("  python3 bluesky_safe.py status     - Show current rate limits")
        print("  python3 bluesky_safe.py can_follow - Check if can follow")
        print("  python3 bluesky_safe.py can_post   - Check if can post")
        print("  python3 bluesky_safe.py can_like   - Check if can like")
        print("  python3 bluesky_safe.py reset      - Reset state (emergency)")
        print()
        print("Import in Python:")
        print("  from bluesky_safe import BlueskyClient, get_status")
        print("  client = BlueskyClient()")
        print("  result = client.safe_post('Hello world')")
