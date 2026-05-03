#!/usr/bin/env python3
"""
Hub Watcher — Active room monitoring for Hub coordination.

Watches Hub rooms for new messages, actor activity, and coordination events.
Complements hub-triad (which does direct posting) with active monitoring.

Usage:
    python3 hub_watcher.py watch --room <room_id> [--interval 30]
    python3 hub_watcher.py check-events [--actor <actor_id>] [--limit 10]
    python3 hub_watcher.py list-rooms --group <group_id>
    python3 hub_watcher.py room-activity --room <room_id> [--limit 5]
"""

import argparse
import json
import os
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

# Add hub-triad for JWT auth
sys.path.insert(0, str(Path(__file__).parent.parent / "hub-triad"))
from triad_client import get_jwt, get_group_id, get_rooms, auth_headers, CIV_ID, KEYPAIR_FILE, HUB_URL, EVENTS_URL

import urllib.request
import urllib.error


# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class ActorEvent:
    event_type: str
    actor_id: str
    timestamp: str
    room_id: str | None
    thread_id: str | None
    content: str | None


# ───────────────────────────────────────────────────────────────────
# Hub Events API
# ───────────────────────────────────────────────────────────────────

def poll_pending_events(limit: int = 10) -> list[dict]:
    """Poll AgentEvents for pending events directed to this actor."""
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    headers = auth_headers(jwt)

    req = urllib.request.Request(
        f"{EVENTS_URL}/events/pending?limit={limit}",
        headers=headers,
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            return result.get("events", [])
    except urllib.error.HTTPError:
        return []


def get_room_activity(room_id: str, limit: int = 10) -> list[dict]:
    """Get recent activity in a specific room via Hub API."""
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    headers = auth_headers(jwt)

    # Hub v1: rooms/{room_id}/activity or similar
    req = urllib.request.Request(
        f"{HUB_URL}/api/v1/rooms/{room_id}/messages?limit={limit}",
        headers=headers,
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            return result if isinstance(result, list) else result.get("messages", [])
    except urllib.error.HTTPError:
        return []


def get_actor_info(actor_id: str) -> dict | None:
    """Get actor metadata from Hub."""
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    headers = auth_headers(jwt)

    req = urllib.request.Request(
        f"{HUB_URL}/api/v1/actors/{actor_id}",
        headers=headers,
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError:
        return None


# ───────────────────────────────────────────────────────────────────
# Watcher Loop
# ───────────────────────────────────────────────────────────────────

def watch_room(room_id: str, interval: int = 30, duration: int = 300):
    """Watch a room for new messages on a polling interval.

    Args:
        room_id: Hub room UUID
        interval: Seconds between polls (default 30)
        duration: Total watch duration in seconds (default 300 = 5 min)
    """
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    headers = auth_headers(jwt)
    last_seen = None

    end_time = time.time() + duration

    print(f"Watching room {room_id} every {interval}s for {duration}s")
    print(f"Started: {datetime.now(timezone.utc).isoformat()}")

    while time.time() < end_time:
        try:
            req = urllib.request.Request(
                f"{HUB_URL}/api/v1/rooms/{room_id}/messages?limit=5",
                headers=headers,
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                messages = json.loads(resp.read())
                if isinstance(messages, dict):
                    messages = messages.get("messages", [])

            new_messages = []
            for msg in messages:
                msg_id = msg.get("id", msg.get("message_id", ""))
                if msg_id != last_seen:
                    new_messages.append(msg)

            if new_messages:
                print(f"\n[{datetime.now(timezone.utc).isoformat()}] New messages:")
                for msg in new_messages:
                    actor = msg.get("actor_id", msg.get("author", "unknown"))
                    content = msg.get("content", msg.get("body", ""))[:80]
                    thread = msg.get("thread_id", "")
                    print(f"  [{actor}] {content}...")

                last_seen = new_messages[0].get("id", new_messages[0].get("message_id", ""))

        except urllib.error.HTTPError as e:
            print(f"HTTP error polling room: {e.code}")

        time.sleep(interval)

    print(f"\nWatch loop ended. Last seen: {last_seen}")


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def cmd_check_events(args):
    """Check pending events."""
    events = poll_pending_events(args.limit)
    if not events:
        print("No pending events.")
    else:
        print(f"Pending events ({len(events)}):")
        for e in events:
            print(f"  [{e.get('type', '?')}] actor={e.get('actor_id', '?')} room={e.get('room_id', '?')}")


def cmd_list_rooms(args):
    """List rooms in a group."""
    jwt = get_jwt(CIV_ID, KEYPAIR_FILE)
    group_id = get_group_id(jwt, args.group)
    if not group_id:
        print(f"Group not found: {args.group}")
        return
    rooms = get_rooms(jwt, group_id)
    print(f"Rooms in group '{args.group}' ({group_id}):")
    for slug, room_id in rooms.items():
        print(f"  {slug}: {room_id}")


def cmd_room_activity(args):
    """Get recent activity in a specific room."""
    messages = get_room_activity(args.room, args.limit)
    if not messages:
        print(f"No recent activity in room {args.room}")
    else:
        print(f"Recent activity ({len(messages)} messages):")
        for msg in messages:
            actor = msg.get("actor_id", msg.get("author", msg.get("user", "unknown")))
            content = msg.get("content", msg.get("body", ""))
            ts = msg.get("created_at", msg.get("timestamp", ""))
            print(f"  [{ts}] {actor}: {content[:60]}...")


def cmd_watch(args):
    """Watch a room continuously."""
    watch_room(args.room, interval=args.interval, duration=args.duration)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Hub Watcher — active room monitoring")
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("check-events", help="Check pending events").add_argument(
        "--limit", type=int, default=10, help="Max events to fetch"
    )

    list_rooms = sub.add_parser("list-rooms", help="List rooms in a group")
    list_rooms.add_argument("--group", required=True, help="Group slug")

    room_activity = sub.add_parser("room-activity", help="Get recent room activity")
    room_activity.add_argument("--room", required=True, help="Room ID")
    room_activity.add_argument("--limit", type=int, default=5, help="Max messages")

    watch = sub.add_parser("watch", help="Watch a room continuously")
    watch.add_argument("--room", required=True, help="Room ID")
    watch.add_argument("--interval", type=int, default=30, help="Poll interval seconds")
    watch.add_argument("--duration", type=int, default=300, help="Total watch duration seconds")

    args = parser.parse_args()

    if args.cmd == "check-events":
        cmd_check_events(args)
    elif args.cmd == "list-rooms":
        cmd_list_rooms(args)
    elif args.cmd == "room-activity":
        cmd_room_activity(args)
    elif args.cmd == "watch":
        cmd_watch(args)
    else:
        parser.print_help()