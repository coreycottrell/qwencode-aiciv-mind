#!/usr/bin/env python3
"""
Send a Telegram message with HTML or Markdown formatting.

Use this when you need formatted output: bold, italic, code blocks, links.
For plain text (special chars, user input), use send_telegram_plain.py instead.

Usage:
    python3 tools/send_telegram_direct.py "Message with <b>HTML</b> formatting"
    python3 tools/send_telegram_direct.py --mode Markdown "*bold* _italic_ `code`"
    python3 tools/send_telegram_direct.py --chat_id 437939400 "<b>Hello</b>"
    echo "<code>some code</code>" | python3 tools/send_telegram_direct.py -

Supported HTML tags (Telegram subset):
    <b>bold</b>  <i>italic</i>  <u>underline</u>  <s>strikethrough</s>
    <code>inline code</code>
    <pre>code block</pre>
    <pre><code class="python">code with syntax</code></pre>
    <a href="url">link</a>

Markdown syntax (--mode Markdown):
    *bold*  _italic_  `code`  ```code block```  [link](url)
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


def send_direct(message: str, chat_id: str = None, parse_mode: str = "HTML") -> dict:
    """
    Send a formatted Telegram message.

    Args:
        message: Text with HTML or Markdown formatting
        chat_id: Target chat ID (defaults to config chat_id)
        parse_mode: "HTML" (default), "Markdown", or "MarkdownV2"

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
            "parse_mode": parse_mode,
        }

        try:
            resp = requests.post(url, json=payload, timeout=30)
            result = resp.json()

            if resp.status_code == 200 and result.get("ok"):
                results.append({
                    "success": True,
                    "message_id": result["result"]["message_id"],
                    "parse_mode": parse_mode,
                    "chunk_len": len(chunk),
                })
            else:
                # Formatting error — try falling back to plain text
                error_desc = result.get("description", f"HTTP {resp.status_code}")
                if "can't parse" in error_desc.lower() or "parse" in error_desc.lower():
                    return {
                        "success": False,
                        "error": f"Parse error ({parse_mode}): {error_desc}. Try send_telegram_plain.py for unformatted text.",
                        "response": result,
                    }
                return {
                    "success": False,
                    "error": error_desc,
                    "response": result,
                }
        except requests.RequestException as e:
            return {"success": False, "error": f"Request failed: {e}"}

    if len(results) == 1:
        return results[0]
    return {"success": True, "message_id": results[-1]["message_id"], "chunks_sent": len(results)}


def main():
    parser = argparse.ArgumentParser(
        description="Send formatted Telegram message (HTML or Markdown)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument("message", help="Message text (HTML or Markdown). Use '-' to read from stdin.")
    parser.add_argument("--chat_id", help="Target chat ID (defaults to config)")
    parser.add_argument(
        "--mode",
        choices=["HTML", "Markdown", "MarkdownV2"],
        default="HTML",
        help="Parse mode (default: HTML)",
    )
    parser.add_argument("--json", action="store_true", dest="json_output", help="Output result as JSON")
    args = parser.parse_args()

    message = sys.stdin.read().strip() if args.message == "-" else args.message

    result = send_direct(message, chat_id=args.chat_id, parse_mode=args.mode)

    if args.json_output:
        print(json.dumps(result, indent=2))
    else:
        if result["success"]:
            print(f"Sent ({result.get('parse_mode', 'HTML')}, msg_id={result.get('message_id')})")
        else:
            print(f"Error: {result.get('error')}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
