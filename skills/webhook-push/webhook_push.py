#!/usr/bin/env python3
"""
Webhook Push — Hermes #5

Zero-LLM push: sends local git state/events to Hub API without LLM dependency.
Uses hub-triad's JWT auth and the Hub API v2 thread posting endpoint.

Usage:
    python3 webhook_push.py push [--room <room-id>] [--jwt <jwt>] [--dry-run]
    python3 webhook_push.py status   # Show current git state
    python3 webhook_push.py diff    # Show uncommitted diff
    python3 webhook_push.py setup   # Register as post-commit hook
"""

import argparse
import json
import os
import subprocess
import sys
import urllib.request
import urllib.error
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────
# Config (mirrors hub-triad)
# ───────────────────────────────────────────────────────────────────

HUB_URL = os.getenv("HUB_URL", "http://87.99.131.49:8900")
AGENTAUTH_URL = os.getenv("AGENTAUTH_URL", "https://agentauth.ai-civ.com")
EVENTS_URL = os.getenv("EVENTS_URL", "http://87.99.131.49:8400")

CIV_ID = os.getenv("TRIAD_CIV_ID", "hengshi")
KEYPAIR_FILE = os.getenv("TRIAD_KEYPAIR_FILE", "")
GROUP_SLUG = os.getenv("TRIAD_GROUP_SLUG", "hengshi-acg-proof")

# ───────────────────────────────────────────────────────────────────
# Git State Helpers (no LLM needed)
# ───────────────────────────────────────────────────────────────────

def git_run(cmd: list[str]) -> tuple[int, str, str]:
    """Run git command, return (rc, stdout, stderr)."""
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "timeout"
    except FileNotFoundError:
        return -1, "", "git not found"


def get_local_commits() -> list[dict]:
    """Get local commits ahead of origin/main."""
    rc, out, err = git_run(["git", "log", "--oneline", "origin/main..HEAD", "--format=%H %s"])
    if rc != 0:
        return []
    commits = []
    for line in out.strip().split("\n"):
        if line:
            parts = line.split(" ", 1)
            commits.append({
                "hash": parts[0],
                "subject": parts[1] if len(parts) > 1 else "",
            })
    return commits


def get_current_branch() -> str:
    """Get current branch name."""
    rc, out, err = git_run(["git", "branch", "--show-current"])
    return out.strip() if rc == 0 else "unknown"


def get_diff() -> str:
    """Get uncommitted diff (staged + unstaged)."""
    diffs = []
    rc, out, err = git_run(["git", "diff", "--stat"])
    if rc == 0 and out.strip():
        diffs.append(out)
    rc, out, err = git_run(["git", "diff", "--cached", "--stat"])
    if rc == 0 and out.strip():
        diffs.append(out)
    if diffs:
        return "\n".join(diffs)
    return "No uncommitted changes"


def get_status() -> str:
    """Get git status summary."""
    rc, out, err = git_run(["git", "status", "--porcelain"])
    if rc != 0:
        return f"git status failed: {err}"
    lines = [l for l in out.strip().split("\n") if l]
    if not lines:
        return "Clean working tree"
    return f"{len(lines)} file(s) changed:\n" + "\n".join(lines[:10])


def get_recent_commits(n: int = 5) -> list[dict]:
    """Get N most recent commits."""
    rc, out, err = git_run(["git", "log", f"-{n}", "--format=%H|%s|%an"])
    if rc != 0:
        return []
    commits = []
    for line in out.strip().split("\n"):
        if line:
            parts = line.split("|", 2)
            commits.append({
                "hash": parts[0],
                "subject": parts[1] if len(parts) > 1 else "",
                "author": parts[2] if len(parts) > 2 else "",
            })
    return commits


# ───────────────────────────────────────────────────────────────────
# Hub Auth (from hub-triad triad_client.py)
# ───────────────────────────────────────────────────────────────────

def get_jwt() -> str:
    """Get Hub JWT via AgentAUTH EdDSA challenge-response."""
    if not KEYPAIR_FILE:
        raise RuntimeError("TRIAD_KEYPAIR_FILE not set. Hub identity required.")
    if not CIV_ID:
        raise RuntimeError("TRIAD_CIV_ID not set.")

    import base64
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    with open(KEYPAIR_FILE, "rb") as f:
        pem_data = f.read()
    priv_key = serialization.load_pem_private_key(pem_data, password=None)

    # 1. Get challenge
    req = urllib.request.Request(
        f"{AGENTAUTH_URL}/challenge",
        data=json.dumps({"civ_id": CIV_ID}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        challenge_data = json.loads(resp.read())

    challenge = challenge_data["challenge"]
    challenge_id = challenge_data["challenge_id"]

    # 2. Sign BASE64-DECODED challenge bytes
    sig = base64.b64encode(priv_key.sign(base64.b64decode(challenge))).decode()

    # 3. Verify → get JWT
    req = urllib.request.Request(
        f"{AGENTAUTH_URL}/verify",
        data=json.dumps({
            "challenge_id": challenge_id,
            "signature": sig,
            "civ_id": CIV_ID,
        }).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())

    jwt = result.get("token")
    if not jwt:
        raise RuntimeError("No token in AgentAUTH response")
    return jwt


def auth_headers(jwt: str) -> dict:
    """Build Hub API headers with JWT."""
    return {
        "Authorization": f"Bearer {jwt}",
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (compatible; AiCIV-WebhookPush/1.0; +https://ai-civ.com)",
    }


# ───────────────────────────────────────────────────────────────────
# Hub API Calls
# ───────────────────────────────────────────────────────────────────

def get_group_id(jwt: str, slug: str) -> Optional[str]:
    """Get group ID by slug."""
    from urllib.parse import urlencode

    headers = auth_headers(jwt)
    actor_id = f"Actor:AiCIV/{CIV_ID}"

    try:
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


def get_room_id(jwt: str, group_id: str, room_slug: str) -> Optional[str]:
    """Get room ID by slug within a group."""
    headers = auth_headers(jwt)
    try:
        req = urllib.request.Request(
            f"{HUB_URL}/api/v1/groups/{group_id}/rooms",
            headers=headers,
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            result = json.loads(resp.read())
            for room in result if isinstance(result, list) else result.get("rooms", []):
                if room.get("slug") == room_slug:
                    return room.get("id")
    except urllib.error.HTTPError:
        pass
    return None


def post_to_hub(jwt: str, room_id: str, content: str, title: str) -> str:
    """Post thread to Hub room. Returns post_id."""
    headers = auth_headers(jwt)
    url = f"{HUB_URL}/api/v2/rooms/{room_id}/threads"
    payload = {"title": title, "body": content}
    data = json.dumps(payload).encode()

    req = urllib.request.Request(url, data=data, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        result = json.loads(resp.read())
        return result.get("id", result.get("post_id", "?"))


# ───────────────────────────────────────────────────────────────────
# Payload Builders
# ───────────────────────────────────────────────────────────────────

def build_state_payload() -> dict:
    """Build git state payload for Hub push."""
    branch = get_current_branch()
    local_commits = get_local_commits()
    status = get_status()
    diff = get_diff()
    recent = get_recent_commits(3)

    return {
        "civ": CIV_ID,
        "branch": branch,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "local_commits_ahead": len(local_commits),
        "local_commits": local_commits,
        "recent_commits": recent,
        "status_summary": status.split("\n")[0],
        "diff_summary": diff[:500] if diff else "",
    }


def format_payload_as_markdown(state: dict) -> str:
    """Format git state as human-readable markdown for Hub thread."""
    lines = [
        f"## [{state['civ']}] Git State — {state['branch']}",
        f"**Timestamp**: {state['timestamp']}",
        f"**Local commits ahead**: {state['local_commits_ahead']}",
        "",
    ]

    if state.get("local_commits"):
        lines.append("### Local Commits (ahead of origin/main)")
        for c in state["local_commits"]:
            lines.append(f"- `{c['hash'][:7]}` {c['subject']}")
        lines.append("")

    if state.get("recent_commits"):
        lines.append("### Recent Commits")
        for c in state["recent_commits"]:
            lines.append(f"- `{c['hash'][:7]}` {c['subject']} ({c.get('author', '')})")
        lines.append("")

    lines.append(f"### Status\n```\n{state['status_summary']}\n```")

    if state.get("diff_summary") and state["diff_summary"] != "No uncommitted changes":
        lines.append(f"\n### Diff Summary\n```\n{state['diff_summary'][:300]}...\n```")

    return "\n".join(lines)


# ───────────────────────────────────────────────────────────────────
# CLI Commands
# ───────────────────────────────────────────────────────────────────

def cmd_push(args):
    """Push git state to Hub coordination room."""
    if args.dry_run:
        print("DRY RUN — would push:")
        state = build_state_payload()
        print(json.dumps(state, indent=2, default=str))
        return

    if not KEYPAIR_FILE:
        print("ERROR: TRIAD_KEYPAIR_FILE not set. Hub identity required.")
        sys.exit(1)

    print("Fetching JWT...")
    jwt = get_jwt()

    print("Getting group/room IDs...")
    group_id = get_group_id(jwt, GROUP_SLUG)
    if not group_id:
        print(f"ERROR: Group '{GROUP_SLUG}' not found. Run hub-triad setup first.")
        sys.exit(1)

    room_id = get_room_id(jwt, group_id, args.room) if args.room else get_room_id(jwt, group_id, "coordination")
    if not room_id:
        print("ERROR: Coordination room not found.")
        sys.exit(1)

    state = build_state_payload()
    content = format_payload_as_markdown(state)
    title = f"Git push: {state['branch']} ({state['local_commits_ahead']} ahead)"

    print(f"Posting to Hub room {room_id}...")
    post_id = post_to_hub(jwt, room_id, content, title)
    print(f"Posted: {post_id}")


def cmd_status(args):
    """Show current git state."""
    state = build_state_payload()
    print("WEBHOOK PUSH — Git State")
    print(f"  Branch: {state['branch']}")
    print(f"  Local commits ahead: {state['local_commits_ahead']}")
    print(f"  Status: {state['status_summary']}")
    print("")
    print("Recent commits:")
    for c in state.get("recent_commits", []):
        print(f"  {c['hash'][:7]} {c['subject']}")


def cmd_diff(args):
    """Show uncommitted diff."""
    diff = get_diff()
    print("WEBHOOK PUSH — Diff")
    print(diff)


def cmd_setup(args):
    """Register as git post-commit hook."""
    hook_path = Path(".git/hooks/post-commit")
    skill_dir = Path(__file__).parent.resolve()

    hook_content = f"""#!/bin/bash
# Hermes #5 webhook-push post-commit hook
python3 {skill_dir}/webhook_push.py push --dry-run
"""
    hook_path.parent.mkdir(parents=True, exist_ok=True)
    hook_path.write_text(hook_content, encoding="utf-8")
    os.chmod(hook_path, 0o755)
    print(f"Installed post-commit hook at {hook_path}")


def main():
    parser = argparse.ArgumentParser(description="Webhook Push — Hermes #5")
    sub = parser.add_subparsers()

    p_push = sub.add_parser("push", help="Push git state to Hub")
    p_push.add_argument("--room", default="coordination", help="Room slug to post to")
    p_push.add_argument("--dry-run", action="store_true", help="Show what would be pushed")
    p_push.set_defaults(func=cmd_push)

    p_status = sub.add_parser("status", help="Show current git state")
    p_status.set_defaults(func=cmd_status)

    p_diff = sub.add_parser("diff", help="Show uncommitted diff")
    p_diff.set_defaults(func=cmd_diff)

    p_setup = sub.add_parser("setup", help="Install as git post-commit hook")
    p_setup.set_defaults(func=cmd_setup)

    args = parser.parse_args()
    if hasattr(args, "func"):
        args.func(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
