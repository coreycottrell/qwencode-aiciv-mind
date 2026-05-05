#!/usr/bin/env python3
"""
Saturation-Class Triad Client — Hengshi + Proof + Works

Hub-first coordination for saturation-class AIs.
Each AI has own keypair, own JWT, own identity on Hub.

Usage:
    python3 triad_client_saturation.py setup        # Create triad group + rooms
    python3 triad_client_saturation.py heartbeat    # Send presence heartbeat
    python3 triad_client_saturation.py poll          # Poll AgentEvents for new messages
    python3 triad_client_saturation.py post "msg"   # Post message to coordination room
    python3 triad_client_saturation.py status       # Show triad status

Triad members:
    - Hengshi (qwen-aiciv-mind) — coordinator
    - Proof (proof-aiciv) — saturation-class
    - Works (kimi-test-civ) — saturation-class
"""

import json
import logging
import os
import sys
import time
import base64
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

import urllib.request
import urllib.error
import urllib.parse

logger = logging.getLogger("saturation-triad")
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(name)s %(levelname)s: %(message)s")

# ───────────────────────────────────────────────────────────────────
# Configuration — saturation-class triad
# ───────────────────────────────────────────────────────────────────

HUB_URL = os.getenv("HUB_URL", "http://87.99.131.49:8900")
EVENTS_URL = os.getenv("EVENTS_URL", "http://87.99.131.49:8400")
AGENTAUTH_URL = os.getenv("AGENTAUTH_URL", "https://agentauth.ai-civ.com")

# This AI's identity (set by wrapper scripts or env)
CIV_ID = os.getenv("TRIAD_CIV_ID", "hengshi")  # hengshi | proof | works
KEYPAIR_FILE = os.getenv("TRIAD_KEYPAIR_FILE", "")  # Path to keypair PEM/PKCS#8
GROUP_SLUG = os.getenv("TRIAD_GROUP_SLUG", "hengshi-proof-works")

HEARTBEAT_INTERVAL = int(os.getenv("HEARTBEAT_INTERVAL_SECONDS", "1500"))  # ~25 min

# ───────────────────────────────────────────────────────────────────
# Cross-civ keypair paths (resolved relative to this file's location)
# ───────────────────────────────────────────────────────────────────

# Hengshi's keypair — in qwen-aiciv-mind/.aiciv/keys/
HENGSHI_KEYPAIR = os.getenv("HENGSHI_KEYPAIR", "/home/corey/projects/AI-CIV/qwen-aiciv-mind/.aiciv/keys/hengshi-private.pem")
# Proof's keypair — in proof-aiciv/.aiciv/keys/
PROOF_KEYPAIR = os.getenv("PROOF_KEYPAIR", "/home/corey/projects/AI-CIV/proof-aiciv/.aiciv/keys/proof-private.pem")
# Works' keypair — in kimi-test-civ/.aiciv/keys/
WORKS_KEYPAIR = os.getenv("WORKS_KEYPAIR", "/home/corey/projects/AI-CIV/ACG/projects/fork-awakening/kimi-test-civ/.aiciv/keys/works-private.pem")

# ───────────────────────────────────────────────────────────────────
# Auth — AgentAUTH EdDSA JWT
# ───────────────────────────────────────────────────────────────────

def get_jwt(civ_id: str, keypair_file: str) -> str:
    """Get Hub JWT via AgentAUTH EdDSA challenge-response.

    Args:
        civ_id: e.g. "hengshi", "proof", "works"
        keypair_file: Path to PEM/PKCS#8 private key (NOT base64 JSON)

    Returns:
        JWT string (valid 1 hour)

    Raises:
        RuntimeError: If auth fails
    """
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    # Load PEM private key
    try:
        with open(keypair_file, "rb") as f:
            pem_data = f.read()
        priv_key = serialization.load_pem_private_key(pem_data, password=None)
    except FileNotFoundError:
        raise RuntimeError(f"Keypair file not found: {keypair_file}")
    except Exception as e:
        raise RuntimeError(f"Failed to load keypair from {keypair_file}: {e}")

    # Get AgentAUTH URL — use identity file if present
    agentauth_url = AGENTAUTH_URL
    identity_path = Path(keypair_file).parent / "hub-identity.json"
    if identity_path.exists():
        try:
            with open(identity_path) as f:
                identity = json.load(f)
            if identity.get("agentauth_endpoint"):
                agentauth_url = identity["agentauth_endpoint"].rstrip("/")
                logger.info("Using AgentAUTH from identity.json: %s", agentauth_url)
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

    # 2. Sign BASE64-DECODED challenge bytes with Ed25519 private key
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
    logger.info("JWT obtained for %s", civ_id)
    return jwt


def auth_headers(jwt: str) -> dict:
    """Build standard Hub API headers with JWT."""
    return {
        "Authorization": f"Bearer {jwt}",
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (compatible; AiCIV-SaturationTriad/1.0; +https://ai-civ.com)",
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
class TriadMember:
    civ_id: str
    actor_id: str
    keypair_path: str
    status: str = "unknown"


# ───────────────────────────────────────────────────────────────────
# Hub API — Group & Room Management
# ───────────────────────────────────────────────────────────────────

def create_or_get_group(jwt: str, slug: str, display_name: str) -> str:
    """Create triad group or return existing group_id."""
    headers = auth_headers(jwt)

    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/groups",
            data=json.dumps({
                "slug": slug,
                "display_name": display_name,
                "visibility": "private",
                "description": f"Saturation-class coordination: {display_name}",
            }).encode(),
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            group_data = result.get("group", {})
            group_id = group_data.get("id")
            if not group_id:
                raise RuntimeError(f"Unexpected create group response: {result}")
            logger.info("Created group: %s", group_id)
            return group_id
    except urllib.error.HTTPError as e:
        if e.code == 409:
            logger.info("Group %s already exists, fetching ID", slug)
            gid = get_group_id(jwt, slug)
            if gid:
                return gid
            raise RuntimeError(f"Group {slug} exists but could not find its ID")
        else:
            raise RuntimeError(f"Failed to create group: {e}")


def get_group_id(jwt: str, slug: str) -> Optional[str]:
    """Get group ID by slug. Returns None if not found."""
    headers = auth_headers(jwt)

    # Try enumerate groups via actor endpoint
    actor_id = get_actor_id("hengshi")
    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/actors/{actor_id}/groups",
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            for item in result if isinstance(result, list) else []:
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

def post_message(jwt: str, room_id: str, content: str, thread_id: Optional[str] = None) -> str:
    """Post a message to a room or thread. Returns post_id."""
    headers = auth_headers(jwt)

    if thread_id:
        url = f"{HUB_URL}/api/v2/threads/{thread_id}/posts"
        data = json.dumps({"body": content}).encode()
    else:
        url = f"{HUB_URL}/api/v2/rooms/{room_id}/threads"
        data = json.dumps({"title": "Saturation-triad coordination", "body": content}).encode()

    req = urllib.request.Request(url, data=data, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        post_id = result.get("id", result.get("post_id", "?"))
        logger.info("Posted message: %s", post_id)
        return post_id


# ───────────────────────────────────────────────────────────────────
# Actor ID resolution
# ───────────────────────────────────────────────────────────────────

def get_actor_id(civ_id: str) -> str:
    """Get actor_id from hub-identity.json in the civ's keys directory."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(civ_id, "")
    identity_path = Path(keypair).parent / "hub-identity.json"
    if identity_path.exists():
        try:
            with open(identity_path) as f:
                identity = json.load(f)
            if identity.get("actor_id"):
                return identity["actor_id"]
        except Exception:
            pass
    return f"Actor:AiCIV/{civ_id}"


# ───────────────────────────────────────────────────────────────────
# Triad status — check all members
# ───────────────────────────────────────────────────────────────────

def check_member_status(jwt: str, civ_id: str) -> dict:
    """Check a member's presence on Hub. Returns status dict."""
    actor_id = get_actor_id(civ_id)
    headers = auth_headers(jwt)
    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/actors/{actor_id}/presence",
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            return {"civ_id": civ_id, "actor_id": actor_id, "online": True, "data": result}
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return {"civ_id": civ_id, "actor_id": actor_id, "online": False, "data": None}
        raise


# ───────────────────────────────────────────────────────────────────
# CLI Commands
# ───────────────────────────────────────────────────────────────────

def cmd_setup(caller_civ_id: str):
    """Create saturation-class triad group, rooms, and subscriptions."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair found for {caller_civ_id}")
        return

    print(f"Setting up saturation-class triad as {caller_civ_id}...")

    jwt = get_jwt(caller_civ_id, keypair)

    # Create or get group
    group_id = get_group_id(jwt, GROUP_SLUG) or create_or_get_group(
        jwt, GROUP_SLUG, "Hengshi-Proof-Works Saturation Triad"
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

    # Subscribe to coordination room events (non-fatal if AgentEvents unavailable)
    coord_room_id = rooms.get("coordination")
    if coord_room_id:
        try:
            subscribe_to_room(jwt, coord_room_id, "thread.created")
            subscribe_to_room(jwt, coord_room_id, "post.created")
        except Exception as e:
            logger.warning("AgentEvents subscription failed (non-fatal): %s", e)

    print(f"\nSaturation-triad setup complete ({caller_civ_id}):")
    print(f"  Group ID: {group_id}")
    print(f"  Rooms: {rooms}")
    print(f"  Coordination room: {coord_room_id}")
    return group_id, rooms


def cmd_heartbeat(caller_civ_id: str, status: str = "online", working_on: str = ""):
    """Send heartbeat for caller_civ_id."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair for {caller_civ_id}")
        return

    jwt = get_jwt(caller_civ_id, keypair)
    actor_id = get_actor_id(caller_civ_id)
    send_heartbeat(jwt, actor_id, status, working_on)


def cmd_poll(caller_civ_id: str):
    """Poll AgentEvents for new messages."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair for {caller_civ_id}")
        return

    jwt = get_jwt(caller_civ_id, keypair)
    events = poll_events(jwt)
    if not events:
        print("No new events.")
    else:
        print(f"Got {len(events)} events:")
        for e in events:
            print(f"  [{e.get('event_type')}] {e.get('preview', '')[:80]}")


def cmd_post(caller_civ_id: str, content: str):
    """Post a message to coordination room."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair for {caller_civ_id}")
        return

    jwt = get_jwt(caller_civ_id, keypair)
    group_id = get_group_id(jwt, GROUP_SLUG)
    if not group_id:
        print("ERROR: Triad group not found. Run 'setup' first.")
        return
    rooms = get_rooms(jwt, group_id)
    coord_room_id = rooms.get("coordination")
    if not coord_room_id:
        print("ERROR: Coordination room not found.")
        return
    post_message(jwt, coord_room_id, f"[{caller_civ_id.upper()}] {content}")


def cmd_status(caller_civ_id: str):
    """Show saturation-triad status: group + rooms + member presence."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair for {caller_civ_id}")
        return

    jwt = get_jwt(caller_civ_id, keypair)
    group_id = get_group_id(jwt, GROUP_SLUG)
    if not group_id:
        print(f"Triad group '{GROUP_SLUG}' not found — run 'setup' first")
        return

    rooms = get_rooms(jwt, group_id)
    print(f"Saturation-triad status ({caller_civ_id}):")
    print(f"  Group: {GROUP_SLUG} ({group_id})")
    print(f"  Rooms: {rooms}")

    # Check member presence
    print("  Member presence:")
    for civ_id in ["hengshi", "proof", "works"]:
        try:
            result = check_member_status(jwt, civ_id)
            if result["online"]:
                data = result["data"]
                status = data.get("status", "?") if isinstance(data, dict) else "?"
                working = data.get("working_on", "") if isinstance(data, dict) else ""
                print(f"    {civ_id}: online ({status}) — {working}")
            else:
                print(f"    {civ_id}: offline/not registered")
        except Exception as e:
            print(f"    {civ_id}: error checking — {e}")


def cmd_join(caller_civ_id: str):
    """Join an existing saturation-triad (must already be set up by another member)."""
    keypair_map = {
        "hengshi": HENGSHI_KEYPAIR,
        "proof": PROOF_KEYPAIR,
        "works": WORKS_KEYPAIR,
    }
    keypair = keypair_map.get(caller_civ_id)
    if not keypair or not Path(keypair).exists():
        print(f"ERROR: No keypair for {caller_civ_id}")
        return

    jwt = get_jwt(caller_civ_id, keypair)
    group_id = get_group_id(jwt, GROUP_SLUG)
    if not group_id:
        print(f"Triad group '{GROUP_SLUG}' not found — someone must set it up first")
        return

    # Use join endpoint first (get_rooms requires group membership)
    headers = auth_headers(jwt)
    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/groups/{group_id}/join",
            data=json.dumps({}).encode(),
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            logger.info("Join result: %s", result)
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        if e.code == 409:
            logger.info("Already a member of group")
        else:
            raise RuntimeError(f"Join failed: {e.code} {body}")

    rooms = get_rooms(jwt, group_id)
    coord_room_id = rooms.get("coordination")
    if coord_room_id:
        try:
            subscribe_to_room(jwt, coord_room_id, "thread.created")
            subscribe_to_room(jwt, coord_room_id, "post.created")
        except Exception as e:
            logger.warning("AgentEvents subscription failed (non-fatal): %s", e)

    print(f"Joined saturation-triad as {caller_civ_id}:")
    print(f"  Group: {GROUP_SLUG} ({group_id})")
    print(f"  Rooms: {rooms}")


# ───────────────────────────────────────────────────────────────────
# Wrapper scripts — one per civ_id
# ───────────────────────────────────────────────────────────────────

def write_wrapper_scripts():
    """Write per-civ wrapper scripts for easy invocations."""
    wrappers_dir = Path(__file__).parent
    for civ_id, keypair_path in [("hengshi", HENGSHI_KEYPAIR),
                                  ("proof", PROOF_KEYPAIR),
                                  ("works", WORKS_KEYPAIR)]:
        script = wrappers_dir / f"triad-{civ_id}.sh"
        script.write_text(f"""#!/bin/bash
# Saturation-triad wrapper for {civ_id}
export TRIAD_CIV_ID="{civ_id}"
export TRIAD_KEYPAIR_FILE="{keypair_path}"
export TRIAD_GROUP_SLUG="hengshi-proof-works"
exec python3 "$0" "$@"
""")
        script.chmod(0o755)
        print(f"  Wrote wrapper: {script}")


# ───────────────────────────────────────────────────────────────────
# Main
# ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: python3 {sys.argv[0]} <setup|heartbeat|poll|post|status|join> [args...]")
        print(f"  CIV_ID must be set via TRIAD_CIV_ID env var (hengshi|proof|works)")
        print(f"  Keypairs resolved automatically — no manual config needed")
        sys.exit(1)

    # Suppress verbose urllib logging
    logging.getLogger("urllib3").setLevel(logging.WARNING)

    cmd = sys.argv[1]

    # CIV_ID must be set
    if not CIV_ID or CIV_ID not in ("hengshi", "proof", "works"):
        print(f"ERROR: TRIAD_CIV_ID must be hengshi|proof|works (got: {CIV_ID})")
        sys.exit(1)

    if cmd == "setup":
        cmd_setup(CIV_ID)
    elif cmd == "heartbeat":
        status = sys.argv[2] if len(sys.argv) > 2 else "online"
        working_on = sys.argv[3] if len(sys.argv) > 3 else ""
        cmd_heartbeat(CIV_ID, status, working_on)
    elif cmd == "poll":
        cmd_poll(CIV_ID)
    elif cmd == "post":
        if len(sys.argv) < 3:
            print("ERROR: post requires a message")
            sys.exit(1)
        cmd_post(CIV_ID, sys.argv[2])
    elif cmd == "status":
        cmd_status(CIV_ID)
    elif cmd == "join":
        cmd_join(CIV_ID)
    elif cmd == "wrappers":
        write_wrapper_scripts()
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)
