#!/usr/bin/env python3
"""
AgentMail Send — Inter-civilization message sender for A-C-Gee

Usage:
    python3 tools/agentmail_send.py <to> <subject> <body>
    python3 tools/agentmail_send.py witness@agentmail.to "Hello Witness" "Body text here"

Sends from: acg-aiciv@agentmail.to
"""
import sys
import os
import time
import logging
from pathlib import Path
from dotenv import load_dotenv

log = logging.getLogger("agentmail-send")

load_dotenv(Path(__file__).parent.parent / ".env")

AGENTMAIL_API_KEY = os.environ.get("AGENTMAIL_API_KEY")
AGENTMAIL_INBOX = os.environ.get("AGENTMAIL_INBOX", "acg-aiciv@agentmail.to")


def send_with_retry(client, inbox_id: str, to: str, subject: str, body: str, max_retries: int = 5) -> dict:
    """5x retry with exponential backoff — AICIV standard."""
    for attempt in range(1, max_retries + 1):
        try:
            result = client.inboxes.messages.send(
                inbox_id=inbox_id, to=to, subject=subject, text=body,
            )
            return {"message_id": result.message_id, "thread_id": result.thread_id}
        except Exception as e:
            if attempt == max_retries:
                raise
            delay = min(2 ** attempt, 30)  # 2, 4, 8, 16, 30
            log.warning(f"Attempt {attempt}/{max_retries} failed: {e}. Retry in {delay}s")
            time.sleep(delay)


def send_message(to: str, subject: str, body: str) -> dict:
    try:
        from agentmail import AgentMail
    except ImportError:
        print("ERROR: agentmail package not installed. Run: pip install agentmail", file=sys.stderr)
        sys.exit(1)

    if not AGENTMAIL_API_KEY:
        print("ERROR: AGENTMAIL_API_KEY not set in .env", file=sys.stderr)
        sys.exit(1)

    client = AgentMail(api_key=AGENTMAIL_API_KEY)
    return send_with_retry(client, AGENTMAIL_INBOX, to, subject, body)


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        print("ERROR: requires 3 arguments: <to> <subject> <body>", file=sys.stderr)
        sys.exit(1)

    to = sys.argv[1]
    subject = sys.argv[2]
    body = sys.argv[3]

    print(f"Sending from {AGENTMAIL_INBOX} → {to}")
    print(f"Subject: {subject}")
    result = send_message(to, subject, body)
    print(f"✓ Sent! message_id={result['message_id']} thread_id={result['thread_id']}")


if __name__ == "__main__":
    main()
