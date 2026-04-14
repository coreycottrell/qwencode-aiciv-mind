#!/usr/bin/env python3
"""
Send file/document to Telegram

Usage:
    python3 tools/send_telegram_file.py /path/to/file "optional caption"
    python3 tools/send_telegram_file.py /path/to/file --chat_id 437939400

Examples:
    # Send file to default chat (from config)
    python3 tools/send_telegram_file.py /tmp/debug.log "Debug log from session"

    # Send to specific chat
    python3 tools/send_telegram_file.py /home/corey/file.pdf --chat_id 437939400

    # Python import
    from tools.send_telegram_file import send_telegram_document
    result = send_telegram_document("/path/to/file.pdf", caption="Report")

Returns JSON with message_id on success, error details on failure.
"""

import argparse
import json
import mimetypes
import sys
from pathlib import Path
from typing import Optional, Dict, Any

import requests

# Constants
PROJECT_ROOT = Path(__file__).parent.parent
CONFIG_FILE = PROJECT_ROOT / "config" / "telegram_config.json"


def load_config() -> Dict[str, Any]:
    """Load Telegram configuration."""
    if not CONFIG_FILE.exists():
        raise FileNotFoundError(f"Config file not found: {CONFIG_FILE}")

    with open(CONFIG_FILE, 'r') as f:
        config = json.load(f)

    if not config.get('bot_token'):
        raise ValueError("No bot_token in config")

    return config


def send_telegram_document(
    file_path: str,
    caption: Optional[str] = None,
    chat_id: Optional[str] = None,
    parse_mode: Optional[str] = None
) -> Dict[str, Any]:
    """
    Send a document/file to Telegram.

    Args:
        file_path: Path to the file to send
        caption: Optional caption for the file (max 1024 chars)
        chat_id: Target chat ID (defaults to config chat_id)
        parse_mode: Caption parse mode ("HTML", "Markdown", or None)

    Returns:
        Dict with:
            - success: bool
            - message_id: int (if success)
            - error: str (if failed)
            - response: dict (full API response)
    """
    # Validate file exists
    path = Path(file_path)
    if not path.exists():
        return {
            "success": False,
            "error": f"File not found: {file_path}"
        }

    if not path.is_file():
        return {
            "success": False,
            "error": f"Path is not a file: {file_path}"
        }

    # Check file size (Telegram limit: 50MB for bots)
    file_size = path.stat().st_size
    max_size = 50 * 1024 * 1024  # 50 MB
    if file_size > max_size:
        return {
            "success": False,
            "error": f"File too large: {file_size / (1024*1024):.1f}MB (max 50MB)"
        }

    # Load config
    try:
        config = load_config()
    except Exception as e:
        return {
            "success": False,
            "error": f"Config error: {e}"
        }

    bot_token = config['bot_token']
    target_chat = chat_id or config.get('chat_id')

    if not target_chat:
        return {
            "success": False,
            "error": "No chat_id provided and none in config"
        }

    # Build API request
    url = f"https://api.telegram.org/bot{bot_token}/sendDocument"

    # Prepare form data
    data = {
        'chat_id': target_chat
    }

    if caption:
        # Truncate caption if too long (Telegram limit: 1024 chars)
        if len(caption) > 1024:
            caption = caption[:1021] + "..."
        data['caption'] = caption

    if parse_mode:
        data['parse_mode'] = parse_mode

    # Detect MIME type
    mime_type, _ = mimetypes.guess_type(str(path))
    if mime_type is None:
        mime_type = 'application/octet-stream'

    try:
        with open(path, 'rb') as f:
            files = {
                'document': (path.name, f, mime_type)
            }

            response = requests.post(url, data=data, files=files, timeout=60)

        result = response.json()

        if response.status_code == 200 and result.get('ok'):
            message = result.get('result', {})
            return {
                "success": True,
                "message_id": message.get('message_id'),
                "file_name": path.name,
                "file_size": file_size,
                "response": result
            }
        else:
            return {
                "success": False,
                "error": result.get('description', f"HTTP {response.status_code}"),
                "response": result
            }

    except requests.Timeout:
        return {
            "success": False,
            "error": "Request timed out (file upload may be slow)"
        }
    except requests.RequestException as e:
        return {
            "success": False,
            "error": f"Request failed: {e}"
        }
    except Exception as e:
        return {
            "success": False,
            "error": f"Unexpected error: {e}"
        }


def main():
    parser = argparse.ArgumentParser(
        description="Send file/document to Telegram",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 send_telegram_file.py /tmp/debug.log "Debug log"
  python3 send_telegram_file.py /path/to/file.pdf --chat_id 437939400
  python3 send_telegram_file.py screenshot.png "Screenshot" --parse_mode HTML
        """
    )

    parser.add_argument("file_path", help="Path to file to send")
    parser.add_argument("caption", nargs="?", default=None, help="Optional caption")
    parser.add_argument("--chat_id", help="Target chat ID (defaults to config)")
    parser.add_argument(
        "--parse_mode",
        choices=["HTML", "Markdown", "MarkdownV2"],
        help="Caption parse mode"
    )
    parser.add_argument("--json", action="store_true", help="Output as JSON")

    args = parser.parse_args()

    result = send_telegram_document(
        file_path=args.file_path,
        caption=args.caption,
        chat_id=args.chat_id,
        parse_mode=args.parse_mode
    )

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if result['success']:
            print(f"Sent: {result.get('file_name')} ({result.get('file_size', 0) / 1024:.1f} KB)")
            print(f"Message ID: {result.get('message_id')}")
        else:
            print(f"Error: {result.get('error')}", file=sys.stderr)

    sys.exit(0 if result['success'] else 1)


if __name__ == "__main__":
    main()
