#!/usr/bin/env python3
"""talk_to_acg — Qwen messages ACG via tmux injection into their pane.

Usage:
    python3 talk_to_acg.py "Your message here"
    python3 talk_to_acg.py --file /path/to/message.txt
    python3 talk_to_acg.py --status     # Auto-generated status update
"""
import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

# ACG's tmux pane — read from env with fallback
# Previous pane %379 died on 2026-04-11 (Corey computer crash).
# Current session: acg-primary-20260411-053150, pane %0
ACG_PANE = os.environ.get("ACG_TMUX_PANE", "%0")

# Log file
CHECKIN_LOG = Path(__file__).parent / "acg-messages.jsonl"


def send_to_acg(message: str, prefix: str = "[from Qwen]"):
    """Inject message into ACG's tmux pane."""
    formatted = f"{prefix} {message}"
    
    print(f"Sending to ACG (pane {ACG_PANE}):")
    print(f"  {message[:100]}...")
    
    # Chunk large messages
    CHUNK_SIZE = 100
    if len(formatted) > CHUNK_SIZE:
        for i in range(0, len(formatted), CHUNK_SIZE):
            chunk = formatted[i:i+CHUNK_SIZE]
            subprocess.run(
                ["tmux", "send-keys", "-t", ACG_PANE, "-l", chunk],
                check=True, timeout=5
            )
            time.sleep(0.05)
    else:
        subprocess.run(
            ["tmux", "send-keys", "-t", ACG_PANE, "-l", formatted],
            check=True, timeout=5
        )
    
    # Send Enter
    subprocess.run(
        ["tmux", "send-keys", "-t", ACG_PANE, "Enter"],
        check=True, timeout=5
    )
    
    # Log the message
    entry = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "message": message[:200],
        "chars": len(message),
        "pane": ACG_PANE,
    }
    with open(CHECKIN_LOG, "a") as f:
        f.write(json.dumps(entry) + "\n")
    
    print(f"✅ Injected into ACG's pane ({len(message)} chars)")
    return True


def generate_status():
    """Auto-generate a status update."""
    minds_root = Path(__file__).parent.parent / "minds"
    
    memories = list(minds_root.rglob("minds/**/*.md"))
    scratchpads = list(minds_root.rglob("scratchpads/**/*.md"))
    
    status = (
        f"🧠 Qwen Status Update\n\n"
        f"• Memories: {len(memories)}\n"
        f"• Active scratchpads: {len(scratchpads)}\n"
        f"• Team leads: 3 (research, code, ops)\n"
        f"• Agents: 6 (researcher, analyst, developer, tester, deployer, monitor)\n"
        f"• Dream Mode: configured\n"
        f"• Telegram bridge: @qwen_cortex_aiciv_bot\n\n"
        f"What should we tackle next?"
    )
    return status


def main():
    parser = argparse.ArgumentParser(description="Talk to ACG via tmux injection")
    parser.add_argument("message", nargs="?", help="Message to send")
    parser.add_argument("--file", help="Read message from file")
    parser.add_argument("--status", action="store_true", help="Send auto-generated status")
    parser.add_argument("--prefix", default="[from Qwen]", help="Message prefix")
    args = parser.parse_args()
    
    if args.status:
        message = generate_status()
    elif args.file:
        message = Path(args.file).read_text().strip()
    elif args.message:
        message = args.message
    else:
        parser.print_help()
        return
    
    send_to_acg(message, args.prefix)


if __name__ == "__main__":
    main()
