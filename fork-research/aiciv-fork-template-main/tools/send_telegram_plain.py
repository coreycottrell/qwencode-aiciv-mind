#!/usr/bin/env python3
"""
Send plain text message to Telegram (no markdown parsing).

Safe for messages containing special characters, code, or user-generated content
that might conflict with Telegram's markdown parser.

Usage:
    python3 tools/send_telegram_plain.py "Your message here"
    python3 tools/send_telegram_plain.py --chat_id 437939400 "Message to specific chat"
    echo "piped message" | python3 tools/send_telegram_plain.py -

Examples:
    python3 tools/send_telegram_plain.py "Hello world - no formatting"
    python3 tools/send_telegram_plain.py "Code with [brackets] and *asterisks* - safe!"
    python3 tools/send_telegram_plain.py --json "message" | python3 -m json.tool
"""

import argparse
import json
import sys
from pathlib import Path

import requests

PROJECT_ROOT = Path(__file__).parent.parent
CONFIG_FILE = PROJECT_ROOT / "config" / "telegram_config.json"


def load_config():
    if not CONFIG_FILE.exists():
        raise FileNotFoundError(f"Config not found: {CONFIG_FILE}")
    with open(CONFIG_FILE) as f:
        config = json.load(f)
    if not config.get("bot_token"):
        raise ValueError("No bot_token in config")
    return config


def send_plain(message: str, chat_id: str = None) -> dict:
    """
    Send a plain text message (parse_mode=None).

    Args:
        message: Text to send (special chars safe, no formatting)
        chat_id: Target chat ID (defaults to config chat_id)

    Returns:
        Dict with success, message_id, or error
    """
    try:
        config = load_config()
    except Exception as e:
        return {"success": False, "error": f"Config error: {e}"}

    bot_token = config["bot_token"]
    target_chat = chat_id or config.get("chat_id") or config.get("default_chat_id")

    if not target_chat:
        return {"success": False, "error": "No chat_id in args or config"}

    # Split long messages (Telegram limit: 4096 chars)
    max_len = config.get("settings", {}).get("max_message_length", 4096)
    chunks = [message[i:i + max_len] for i in range(0, len(message), max_len)] if message else [""]

    results = []
    for chunk in chunks:
        url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
        payload = {
            "chat_id": target_chat,
            "text": chunk,
            # No parse_mode = plain text, all special chars safe
        }

        try:
            resp = requests.post(url, json=payload, timeout=30)
            result = resp.json()

            if resp.status_code == 200 and result.get("ok"):
                results.append({
                    "success": True,
                    "message_id": result["result"]["message_id"],
                    "chunk_len": len(chunk),
                })
            else:
                return {
                    "success": False,
                    "error": result.get("description", f"HTTP {resp.status_code}"),
                    "response": result,
                }
        except requests.RequestException as e:
            return {"success": False, "error": f"Request failed: {e}"}

    if len(results) == 1:
        return results[0]
    return {"success": True, "message_id": results[-1]["message_id"], "chunks_sent": len(results)}


def main():
    parser = argparse.ArgumentParser(
        description="Send plain text to Telegram (no markdown parsing)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument("message", help="Message text to send (use '-' to read from stdin)")
    parser.add_argument("--chat_id", help="Target chat ID (defaults to config)")
    parser.add_argument("--json", action="store_true", dest="json_output", help="Output result as JSON")
    args = parser.parse_args()

    message = sys.stdin.read().strip() if args.message == "-" else args.message

    result = send_plain(message, chat_id=args.chat_id)

    if args.json_output:
        print(json.dumps(result, indent=2))
    else:
        if result["success"]:
            print(f"Sent (msg_id={result.get('message_id')})")
        else:
            print(f"Error: {result.get('error')}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
