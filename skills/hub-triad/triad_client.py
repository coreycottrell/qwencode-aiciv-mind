#!/usr/bin/env python3
"""
Hub-First Triad Client

Hub-first coordination for ACG + Proof + Hengshi.
Each AI has own keypair, own JWT, own identity.

IMPORTANT: Requires Hub identity (pending Corey's approval).
This module will fail without valid JWT credentials.

Usage:
    python3 triad_client.py setup        # Create triad group + rooms
    python3 triad_client.py heartbeat    # Send presence heartbeat
    python3 triad_client.py poll          # Poll AgentEvents for new messages
    python3 triad_client.py post "msg"   # Post message to coordination room
"""

import json
import logging
import os
import time
import base64
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

import urllib.request
import urllib.error

logger = logging.getLogger("hub-triad")
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(name)s %(levelname)s: %(message)s")

# ───────────────────────────────────────────────────────────────────
# Configuration
# ───────────────────────────────────────────────────────────────────

HUB_URL = os.getenv("HUB_URL", "http://87.99.131.49:8900")
EVENTS_URL = os.getenv("EVENTS_URL", "http://87.99.131.49:8400")
AGENTAUTH_URL = os.getenv("AGENTAUTH_URL", "https://agentauth.ai-civ.com")

CIV_ID = os.getenv("TRIAD_CIV_ID", "hengshi")  # hengshi | acg | proof
KEYPAIR_FILE = os.getenv("TRIAD_KEYPAIR_FILE", "")  # Path to keypair JSON
GROUP_SLUG = os.getenv("TRIAD_GROUP_SLUG", "hengshi-acg-proof")

HEARTBEAT_INTERVAL = int(os.getenv("HEARTBEAT_INTERVAL_SECONDS", "1500"))  # ~25 min


# ───────────────────────────────────────────────────────────────────
# Auth — AgentAUTH EdDSA JWT
# ───────────────────────────────────────────────────────────────────

def get_jwt(civ_id: str, keypair_file: str) -> str:
    """Get Hub JWT via AgentAUTH EdDSA challenge-response.

    Args:
        civ_id: e.g. "hengshi", "acg", "proof"
        keypair_file: Path to JSON (base64 private_key) or OpenSSH PEM file

    Returns:
        JWT string (valid 1 hour)

    Raises:
        RuntimeError: If auth fails
    """
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    # Load keypair — try JSON first, fall back to PEM/PKCS#8
    try:
        with open(keypair_file) as f:
            kp = json.load(f)
        priv_key = Ed25519PrivateKey.from_private_bytes(base64.b64decode(kp["private_key"]))
    except (json.JSONDecodeError, KeyError, ValueError):
        # Try PEM/PKCS#8 format (-----BEGIN PRIVATE KEY-----)
        try:
            with open(keypair_file, "rb") as f:
                pem_data = f.read()
            priv_key = serialization.load_pem_private_key(
                pem_data,
                password=None,
            )
        except (FileNotFoundError, ValueError) as e:
            raise RuntimeError(f"Failed to load keypair from {keypair_file}: {e}")

    # Determine AgentAUTH URL — prefer identity JSON if present
    agentauth_url = AGENTAUTH_URL
    identity_path = Path(__file__).parent.parent.parent / ".aiciv" / "keys" / "hub-identity.json"
    if identity_path.exists():
        try:
            with open(identity_path) as f:
                identity = json.load(f)
            if identity.get("agentauth_endpoint"):
                agentauth_url = identity["agentauth_endpoint"].rstrip("/")
                logger.info("Using AgentAUTH endpoint from hub-identity.json: %s", agentauth_url)
        except Exception:
            pass

    # 1. Get challenge
    try:
        req = urllib.request.Request(
            f"{agentauth_url}/challenge",
            data=json.dumps({"civ_id": civ_id}).encode(),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            challenge_data = json.loads(resp.read())
    except urllib.error.URLError as e:
        raise RuntimeError(f"AgentAUTH challenge failed: {e}")

    challenge = challenge_data["challenge"]
    challenge_id = challenge_data["challenge_id"]

    # 2. Sign BASE64-DECODED challenge bytes (CRITICAL: not the string!)
    sig = base64.b64encode(priv_key.sign(base64.b64decode(challenge))).decode()

    # 3. Verify → get JWT
    try:
        req = urllib.request.Request(
            f"{agentauth_url}/verify",
            data=json.dumps({
                "challenge_id": challenge_id,
                "signature": sig,
                "civ_id": civ_id,
            }).encode(),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
    except urllib.error.URLError as e:
        raise RuntimeError(f"AgentAUTH verify failed: {e}")

    jwt = result.get("token")
    if not jwt:
        raise RuntimeError("No token in AgentAUTH response")
    return jwt


def auth_headers(jwt: str) -> dict:
    """Build standard Hub API headers with JWT."""
    return {
        "Authorization": f"Bearer {jwt}",
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (compatible; AiCIV-HubTriad/1.0; +https://ai-civ.com)",
    }


# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class TriadGroup:
    group_id: str
    slug: str
    coordination_room_id: str
    decisions_room_id: str
    wul_room_id: str


@dataclass
class TriadMessage:
    message_id: str
    sender_id: str
    sender_name: str
    content: str
    thread_id: str
    room_id: str
    timestamp: str


# ───────────────────────────────────────────────────────────────────
# Hub API — Group & Room Management
# ───────────────────────────────────────────────────────────────────

def create_or_get_group(jwt: str, slug: str, display_name: str) -> str:
    """Create triad group or return existing group_id."""
    headers = auth_headers(jwt)

    # Try to create first
    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/groups",
            data=json.dumps({
                "slug": slug,
                "display_name": display_name,
                "visibility": "private",
                "description": f"Coordination space for {display_name}",
            }).encode(),
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            # Response format: {"group": {"id": "...", "slug": "..."}, "room": {...}, "membership": {...}}
            group_data = result.get("group", {})
            group_id = group_data.get("id")
            if not group_id:
                raise RuntimeError(f"Unexpected create group response: {result}")
            logger.info("Created group: %s", group_id)
            return group_id
    except urllib.error.HTTPError as e:
        if e.code == 409:
            # Group exists — fetch it via actor groups endpoint
            logger.info("Group %s already exists, fetching ID", slug)
            gid = get_group_id(jwt, slug)
            if gid:
                return gid
            raise RuntimeError(f"Group {slug} exists but could not find its ID")
        else:
            raise RuntimeError(f"Failed to create group: {e}")


def get_group_id(jwt: str, slug: str) -> Optional[str]:
    """Get group ID by slug. Returns None if not found.

    Uses /api/v1/actors/{actor_id}/groups to enumerate memberships.
    """
    headers = auth_headers(jwt)

    # Get actor ID from identity file
    actor_id = get_actor_id("hengshi")  # civ_id doesn't matter for lookup

    try:
        # List actor's groups
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/actors/{actor_id}/groups",
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            for item in result:
                group = item.get("group", {})
                if group.get("slug") == slug:
                    return group.get("id")
    except urllib.error.HTTPError:
        pass
    return None


def create_room(jwt: str, group_id: str, slug: str, name: str) -> str:
    """Create a room in the group."""
    headers = auth_headers(jwt)
    req = urllib.request.Request(
        f"{HUB_URL}/api/v1/groups/{group_id}/rooms",
        data=json.dumps({
            "slug": slug,
            "display_name": name,
            "room_type": "text",
        }).encode(),
        headers=headers,
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        # Response format: {"room": {"id": "...", "slug": "..."}} or direct room object
        room_data = result.get("room", result)
        room_id = room_data.get("id")
        logger.info("Created room %s: %s", slug, room_id)
        return room_id


def get_rooms(jwt: str, group_id: str) -> dict:
    """Get all rooms in group. Returns dict: slug -> room_id."""
    headers = auth_headers(jwt)
    req = urllib.request.Request(
        f"{HUB_URL}/api/v1/groups/{group_id}/rooms",
        headers=headers,
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        rooms = {}
        # Response is a list of room objects directly
        for room in result if isinstance(result, list) else result.get("rooms", []):
            rooms[room["slug"]] = room["id"]
        return rooms


# ───────────────────────────────────────────────────────────────────
# AgentEvents — Subscriptions
# ───────────────────────────────────────────────────────────────────

def subscribe_to_room(jwt: str, room_id: str, event_type: str = "thread.created") -> str:
    """Subscribe to events in a room. Returns subscription_id."""
    headers = auth_headers(jwt)
    req = urllib.request.Request(
        f"{EVENTS_URL}/subscriptions",
        data=json.dumps({
            "event_type": event_type,
            "scope_type": "room",
            "scope_id": room_id,
            "delivery_method": "poll",
        }).encode(),
        headers=headers,
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        sub_id = result.get("id", "?")
        logger.info("Subscribed to %s in room %s: %s", event_type, room_id, sub_id)
        return sub_id


def poll_events(jwt: str, limit: int = 10) -> list:
    """Poll pending events from AgentEvents."""
    headers = auth_headers(jwt)
    req = urllib.request.Request(
        f"{EVENTS_URL}/events/pending",
        headers=headers,
        data=urllib.parse.urlencode({"limit": limit}).encode(),
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            return result.get("events", [])
    except urllib.error.HTTPError:
        return []


# ───────────────────────────────────────────────────────────────────
# Presence — Heartbeat
# ───────────────────────────────────────────────────────────────────

def send_heartbeat(jwt: str, actor_id: str, status: str = "online", working_on: str = "") -> None:
    """Send presence heartbeat."""
    headers = auth_headers(jwt)
    req = urllib.request.Request(
        f"{HUB_URL}/api/v1/actors/{actor_id}/heartbeat",
        data=json.dumps({
            "status": status,
            "working_on": working_on,
        }).encode(),
        headers=headers,
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        logger.info("Heartbeat sent: %s / %s", status, working_on)


# ───────────────────────────────────────────────────────────────────
# Posts — Create Thread and Reply
# ───────────────────────────────────────────────────────────────────

def post_message(jwt: str, room_id: str, content: str, thread_id: Optional[str] = None, title: Optional[str] = None) -> str:
    """Post a message to a room or thread. Returns post_id."""
    headers = auth_headers(jwt)

    if thread_id:
        url = f"{HUB_URL}/api/v2/threads/{thread_id}/posts"
        data = json.dumps({"body": content}).encode()
    else:
        url = f"{HUB_URL}/api/v2/rooms/{room_id}/threads"
        payload = {"body": content}
        if title:
            payload["title"] = title
        else:
            payload["title"] = f"Hub-triad coordination thread"
        data = json.dumps(payload).encode()

    req = urllib.request.Request(url, data=data, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        post_id = result.get("id", result.get("post_id", "?"))
        logger.info("Posted message: %s", post_id)
        return post_id


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def cmd_setup():
    """Create triad group, rooms, and subscriptions."""
    if not KEYPAIR_FILE:
        print("ERROR: TRIAD_KEYPAIR_FILE env var not set. Need Hub identity first.")
        print("This is blocked on Corey's approval of Hub identity provisioning.")
        return

    print(f"Setting up triad as {CIV_ID}...")

    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    group_id = get_group_id(jwt, GROUP_SLUG) or create_or_get_group(
        jwt, GROUP_SLUG, f"ACG-Provenance Triad"
    )

    rooms = get_rooms(jwt, group_id)

    # Ensure required rooms exist
    required_rooms = {
        "coordination": "Coordination",
        "decisions": "Decisions",
        "working-out-loud": "Working Out Loud",
    }
    for slug, name in required_rooms.items():
        if slug not in rooms:
            rooms[slug] = create_room(jwt, group_id, slug, name)

    # Subscribe to coordination room
    coord_room_id = rooms.get("coordination")
    if coord_room_id:
        sub_id = subscribe_to_room(jwt, coord_room_id, "thread.created")
        subscribe_to_room(jwt, coord_room_id, "post.created")

    print(f"\nTriad setup complete:")
    print(f"  Group ID: {group_id}")
    print(f"  Rooms: {rooms}")
    print(f"  Coordination room subscribed: {coord_room_id}")


def get_actor_id(civ_id: str) -> str:
    """Get actor_id from hub-identity.json, fall back to derived form."""
    identity_path = Path(__file__).parent.parent.parent / ".aiciv" / "keys" / "hub-identity.json"
    if identity_path.exists():
        try:
            with open(identity_path) as f:
                identity = json.load(f)
            if identity.get("actor_id"):
                return identity["actor_id"]
        except Exception:
            pass
    return f"Actor:AiCIV/{civ_id}"


def cmd_heartbeat(status: str = "online", working_on: str = ""):
    """Send heartbeat."""
    if not KEYPAIR_FILE:
        print("ERROR: TRIAD_KEYPAIR_FILE not set. Hub identity required.")
        return
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    actor_id = get_actor_id(CIV_ID)
    send_heartbeat(jwt, actor_id, status, working_on)


def cmd_poll():
    """Poll AgentEvents for new messages."""
    if not KEYPAIR_FILE:
        print("ERROR: TRIAD_KEYPAIR_FILE not set. Hub identity required.")
        return
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    events = poll_events(jwt)
    if not events:
        print("No new events.")
    else:
        print(f"Got {len(events)} events:")
        for e in events:
            print(f"  [{e.get('event_type')}] {e.get('preview', '')[:80]}")


def cmd_post(content: str):
    """Post a message to coordination room."""
    if not KEYPAIR_FILE:
        print("ERROR: TRIAD_KEYPAIR_FILE not set. Hub identity required.")
        return
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    group_id = get_group_id(jwt, GROUP_SLUG)
    if not group_id:
        print("ERROR: Triad group not found. Run 'setup' first.")
        return
    rooms = get_rooms(jwt, group_id)
    coord_room_id = rooms.get("coordination")
    if not coord_room_id:
        print("ERROR: Coordination room not found.")
        return
    post_message(jwt, coord_room_id, f"[{CIV_ID.upper()}] {content}")


if __name__ == "__main__":
    import sys
    import urllib.parse

    if len(sys.argv) < 2:
        print("Usage: python3 triad_client.py <setup|heartbeat|poll|post> [args...]")
        print("  setup             — Create triad group + rooms")
        print("  heartbeat [status] [working_on] — Send presence heartbeat")
        print("  poll              — Poll AgentEvents for new messages")
        print("  post <message>    — Post message to coordination room")
        sys.exit(1)

    cmd = sys.argv[1]

    # Suppress urllib info logging (very verbose)
    logging.getLogger("urllib3").setLevel(logging.WARNING)

    if cmd == "setup":
        cmd_setup()
    elif cmd == "heartbeat":
        status = sys.argv[2] if len(sys.argv) > 2 else "online"
        working_on = sys.argv[3] if len(sys.argv) > 3 else ""
        cmd_heartbeat(status, working_on)
    elif cmd == "poll":
        cmd_poll()
    elif cmd == "post":
        if len(sys.argv) < 3:
            print("ERROR: post requires a message")
            sys.exit(1)
        cmd_post(sys.argv[2])
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
