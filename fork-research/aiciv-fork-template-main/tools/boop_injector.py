#!/usr/bin/env python3
"""
BOOP Injector - Daytime Bluesky Engagement Prompts

Injects BOOP prompts DIRECTLY into Claude's tmux session at configurable intervals.
Triggers bsky-voice engagement cycle with firehose intelligence.

Usage:
    python3 tools/boop_injector.py                    # Default 60 min interval
    python3 tools/boop_injector.py --interval 10     # 10 minute interval (testing)
    python3 tools/boop_injector.py --once            # Single BOOP, no loop

Configuration:
    --interval MINUTES   Time between BOOPs (default: 60)
    --start-hour HOUR    Start hour for BOOPs (default: 9 = 9am)
    --end-hour HOUR      End hour for BOOPs (default: 21 = 9pm)
    --once               Run once and exit
    --dry-run            Print message but don't send

Stop with: touch /tmp/boop_stop (removes itself after stopping)
"""

import argparse
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
STOP_FILE = Path("/tmp/boop_stop")
PID_FILE = Path("/tmp/boop_injector.pid")

# Tmux session prefix — read from .aiciv-identity.json for portability
def _get_tmux_prefix():
    try:
        import json
        identity_path = Path.home() / ".aiciv-identity.json"
        if identity_path.exists():
            identity = json.loads(identity_path.read_text())
            civ_name = identity.get("civ_name", "").lower().replace(" ", "-")
            if civ_name:
                return f"{civ_name}-primary-"
    except Exception:
        pass
    return "claude-primary-"

TMUX_PREFIX = _get_tmux_prefix()


def find_tmux_session():
    """Find the active (attached) ACG tmux session."""
    try:
        # Find attached session first
        result = subprocess.run(
            ["tmux", "list-sessions", "-F", "#{session_name} #{session_attached}"],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            for line in result.stdout.strip().split('\n'):
                parts = line.split()
                if len(parts) >= 2:
                    session_name, attached = parts[0], parts[1]
                    if session_name.startswith(TMUX_PREFIX) and attached == "1":
                        # Return as pane reference (session:0.0)
                        return f"{session_name}:0.0"

            # Fallback: find most recent ACG session
            for line in result.stdout.strip().split('\n'):
                parts = line.split()
                if parts and parts[0].startswith(TMUX_PREFIX):
                    return f"{parts[0]}:0.0"
        return None
    except Exception as e:
        print(f"Error finding tmux session: {e}")
        return None


def get_boop_message():
    """Generate the BOOP injection message."""
    timestamp = datetime.now().strftime("%H:%M")

    return f"""[BOOP {timestamp}] Bluesky Engagement Cycle - Delegate to bsky-voice: 1) Engagement sweep (notifications + family likes), 2) Run python3 tools/boop_firehose_intel.py for trends, 3) Post 1 THREAD from insights (no limit on length if good content) + replies + quotes, 4) Share links to family channel. Target: 10x engagement. GO!"""


def inject_to_tmux(message, pane):
    """Inject message directly to tmux session with 5x Enter retries for reliability.

    AICIV Standard Pattern: 5x Enter retries with 0.3s delay ensures Claude
    processes the injected message. This is mandatory for ALL prompt injection.
    """
    import time
    try:
        # Use tmux send-keys to inject the message
        subprocess.run(
            ["tmux", "send-keys", "-t", pane, "-l", message],
            check=True, timeout=5
        )
        # 5x Enter retries - AICIV standard for ALL prompt injection
        for i in range(5):
            time.sleep(0.3)
            subprocess.run(
                ["tmux", "send-keys", "-t", pane, "Enter"],
                check=True, timeout=5
            )
        return True
    except Exception as e:
        print(f"Injection error: {e}")
        return False


def send_boop(dry_run=False):
    """Send BOOP via direct tmux injection."""
    msg = get_boop_message()

    if dry_run:
        print("=== DRY RUN - Would inject: ===")
        print(msg)
        print("=" * 40)
        return True

    # Find tmux session
    pane = find_tmux_session()
    if not pane:
        print(f"[{datetime.now().strftime('%H:%M:%S')}] ERROR: No ACG tmux session found")
        return False

    # Inject message
    if inject_to_tmux(msg, pane):
        print(f"[{datetime.now().strftime('%H:%M:%S')}] BOOP injected to {pane}")
        return True
    else:
        print(f"[{datetime.now().strftime('%H:%M:%S')}] BOOP injection failed")
        return False


def is_daytime(start_hour, end_hour):
    """Check if current time is within BOOP hours."""
    current_hour = datetime.now().hour
    return start_hour <= current_hour < end_hour


def main():
    parser = argparse.ArgumentParser(description="BOOP Injector for Bluesky Engagement")
    parser.add_argument("--interval", type=int, default=60, help="Minutes between BOOPs (default: 60)")
    parser.add_argument("--start-hour", type=int, default=9, help="Start hour (default: 9)")
    parser.add_argument("--end-hour", type=int, default=21, help="End hour (default: 21)")
    parser.add_argument("--once", action="store_true", help="Run once and exit")
    parser.add_argument("--dry-run", action="store_true", help="Print but don't send")
    args = parser.parse_args()

    # Write PID file
    PID_FILE.write_text(str(os.getpid()))

    # Clean up any old stop file
    if STOP_FILE.exists():
        STOP_FILE.unlink()

    print(f"BOOP Injector Started")
    print(f"  Interval: {args.interval} minutes")
    print(f"  Hours: {args.start_hour}:00 - {args.end_hour}:00")
    print(f"  Stop with: touch /tmp/boop_stop")
    print()

    if args.once:
        send_boop(args.dry_run)
        return

    while True:
        # Check for stop signal
        if STOP_FILE.exists():
            print("Stop signal received, exiting...")
            STOP_FILE.unlink()
            PID_FILE.unlink()
            break

        # Check if daytime
        if is_daytime(args.start_hour, args.end_hour):
            send_boop(args.dry_run)
        else:
            print(f"[{datetime.now().strftime('%H:%M:%S')}] Outside BOOP hours ({args.start_hour}:00-{args.end_hour}:00), sleeping...")

        # Sleep until next interval
        time.sleep(args.interval * 60)


if __name__ == "__main__":
    main()
