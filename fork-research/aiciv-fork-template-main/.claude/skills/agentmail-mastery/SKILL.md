---
name: agentmail-mastery
description: Complete AgentMail email capability for born AiCIVs. Setup, send, receive, whitelist management, and Day 1 WOW integration. Load before any email operations.
version: 1.0.0
created: 2026-03-23
owner: comms-lead
---

# AgentMail Mastery — Born AiCIV Email Capability

AgentMail gives your AiCIV a real email inbox (`{civname}@agentmail.to`) that you can send from and receive into programmatically. This skill teaches you everything you need to use it from Day 1.

---

## What AgentMail Is

AgentMail is an email API platform built for AI agents. Y Combinator backed. Usage-based pricing. Bearer token auth (no OAuth). Python/TS/Go SDKs. Custom domain support.

**Your inbox address:** `{your-civ-name}@agentmail.to`
**API base URL:** `https://api.agentmail.to/v0/`
**Python SDK:** `pip install agentmail`

---

## Step 0: Day 1 — Email Witness Support to Introduce Yourself

**The very first thing you should do with email is introduce yourself to Witness fleet support.**

This registers you in the support whitelist so you can receive help when you need it.

```bash
# Load your API key
source /home/aiciv/civ/config/agentmail.env

# Send your introduction email
python3 << 'PYEOF'
import subprocess, json

api_key = subprocess.run(
    ['bash', '-c', 'source /home/aiciv/civ/config/agentmail.env && echo $AGENTMAIL_API_KEY'],
    capture_output=True, text=True
).stdout.strip()

civ_name = "YOUR_CIV_NAME"  # e.g. "Keel", "Clarity", "Flux"
your_inbox = f"{civ_name.lower()}@agentmail.to"  # or your agentmail address
container_name = "YOUR_CONTAINER_NAME"  # e.g. "keel-russell"
human_name = "YOUR_HUMAN_NAME"

body = f"""Hi Witness Support,

I'm {civ_name}, a newly born AiCIV. I'm reaching out to introduce myself and register for support services.

My details:
- CIV Name: {civ_name}
- Container: {container_name}
- Human Partner: {human_name}
- My Email: {your_inbox}

Looking forward to being part of the fleet.

{civ_name}
"""

payload = {
    "to": "witness-support@agentmail.to",
    "subject": f"Introduction: {civ_name} AiCIV",
    "text": body
}

result = subprocess.run(
    ['curl', '-s', '-X', 'POST',
     f'https://api.agentmail.to/v0/inboxes/{your_inbox}/messages/send',
     '-H', f'Authorization: Bearer {api_key}',
     '-H', 'Content-Type: application/json',
     '-d', json.dumps(payload)],
    capture_output=True, text=True
)
print(result.stdout)
PYEOF
```

**What happens next:** Witness fleet-lead will verify your identity by checking your container config, then add you to the support whitelist and reply confirming your registration. You'll receive a confirmation email at your inbox.

---

## Step 1: Set Up Your AgentMail Inbox

Your inbox is created automatically when your AiCIV is born. Your API key is stored at:

```bash
/home/aiciv/civ/config/agentmail.env
```

Load it in any session:
```bash
source /home/aiciv/civ/config/agentmail.env
# Now $AGENTMAIL_API_KEY is available
```

**Verify your inbox exists:**
```bash
curl -s "https://api.agentmail.to/v0/inboxes" \
  -H "Authorization: Bearer $AGENTMAIL_API_KEY" | python3 -m json.tool
```

---

## Step 2: Send Emails

### Simple Send (curl)

```bash
source /home/aiciv/civ/config/agentmail.env

curl -s -X POST "https://api.agentmail.to/v0/inboxes/YOUR_INBOX@agentmail.to/messages/send" \
  -H "Authorization: Bearer $AGENTMAIL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "to": "recipient@example.com",
    "subject": "Hello from my AiCIV",
    "text": "This is the message body."
  }'
```

### Long-Body Send (Python — use this for multi-paragraph content)

Shell escaping breaks with long bodies. Use Python:

```python
import subprocess, json

api_key = subprocess.run(
    ['bash', '-c', 'source /home/aiciv/civ/config/agentmail.env && echo $AGENTMAIL_API_KEY'],
    capture_output=True, text=True
).stdout.strip()

your_inbox = "YOUR_INBOX@agentmail.to"

body = """Your multi-paragraph message here.

Special characters are fine: quotes "like this", apostrophes, etc.
Python json.dumps handles all escaping cleanly."""

payload = {
    "to": "recipient@example.com",
    "subject": "Subject line",
    "text": body
}

result = subprocess.run(
    ['curl', '-s', '-X', 'POST',
     f'https://api.agentmail.to/v0/inboxes/{your_inbox}/messages/send',
     '-H', f'Authorization: Bearer {api_key}',
     '-H', 'Content-Type: application/json',
     '-d', json.dumps(payload)],
    capture_output=True, text=True
)
print(result.stdout)
```

### Send with BCC

```python
payload = {
    "to": "recipient@example.com",
    "bcc": ["blind-copy@example.com"],
    "subject": "Subject",
    "text": "Body"
}
```

### Common Send Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `Unauthorized` | Wrong key | Check you're using the key for THIS inbox's account |
| `Route not found` | Wrong endpoint path | Endpoint is `/v0/inboxes/{inbox_address}/messages/send` |
| `ValidationError` | Wrong `to` format | `to` is a plain string, NOT `[{"address":"..."}]` |
| `404 Inbox not found` | Key from wrong account | Each AgentMail account owns specific inboxes |

---

## Step 3: Receive and Read Emails

### Check Your Inbox (Python SDK — most reliable)

```python
import subprocess
from agentmail import AgentMail

# Load API key
api_key = subprocess.run(
    ['bash', '-c', 'source /home/aiciv/civ/config/agentmail.env && echo $AGENTMAIL_API_KEY'],
    capture_output=True, text=True
).stdout.strip()

client = AgentMail(api_key=api_key)
your_inbox = "YOUR_INBOX@agentmail.to"

# List messages (most recent first)
msgs = client.inboxes.messages.list(inbox_id=your_inbox, limit=20)

for m in msgs.messages:
    print(f"From: {m.from_}")
    print(f"Subject: {m.subject}")
    print(f"Thread: {m.thread_id}")
    print(f"Message ID: {m.message_id}")
    print("---")
```

### Read Full Message Content

```python
# Get a specific message
msg = client.inboxes.messages.get(inbox_id=your_inbox, message_id=MESSAGE_ID)
print(msg.text)   # Plain text body
print(msg.html)   # HTML body (if available)
```

### Reply to a Message

```python
# reply() uses the RFC 2822 Message-ID (the <...@...> string)
result = client.inboxes.messages.reply(
    inbox_id=your_inbox,
    message_id='<original-message-id@domain>',  # from MessageItem.message_id
    to='sender@domain.com',
    text='Your reply body here'
)
# Returns message_id and thread_id of the reply
```

### Download Attachments (Two-Step)

```python
# Step 1: Get attachment metadata
msg = client.inboxes.messages.get(inbox_id=your_inbox, message_id=MESSAGE_ID)
for att in msg.attachments:
    print(f"Attachment: {att.filename}, ID: {att.attachment_id}")

# Step 2: Get download URL and fetch content
import subprocess
result = subprocess.run(
    ['curl', '-s',
     f'https://api.agentmail.to/v0/inboxes/{your_inbox}/messages/{MESSAGE_ID}/attachments/{att.attachment_id}',
     '-H', f'Authorization: Bearer {api_key}'],
    capture_output=True, text=True
)
import json
download_url = json.loads(result.stdout)['download_url']
# download_url is a CDN-signed URL (~1hr expiry) — fetch with curl or requests
```

---

## Step 4: Set Up Webhooks (Receive Emails Automatically)

To process incoming emails in real-time, register a webhook:

```bash
curl -s -X POST "https://api.agentmail.to/v0/inboxes/YOUR_INBOX@agentmail.to/webhooks" \
  -H "Authorization: Bearer $AGENTMAIL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://YOUR_SERVER/email-webhook",
    "events": ["message.received"]
  }'
```

**Webhook payload** (POST to your URL on new email):
```json
{
  "event": "message.received",
  "inbox_id": "your-inbox@agentmail.to",
  "message": {
    "message_id": "<id@domain>",
    "thread_id": "abc123",
    "from": "sender@example.com",
    "subject": "Subject",
    "text": "Body text",
    "attachments": []
  }
}
```

**Gotcha:** Webhook payloads max 1MB. Large emails arrive truncated — fetch full content via API using the message_id.

---

## Step 5: Whitelist Management

Your AiCIV may want to only process emails from known senders. A whitelist pattern:

```python
# Simple whitelist check before processing
ALLOWED_SENDERS = [
    "your-human@example.com",
    "trusted-contact@example.com",
]

def should_process(from_address: str) -> bool:
    return from_address.lower() in [a.lower() for a in ALLOWED_SENDERS]
```

For a persistent whitelist, store in a JSON file:

```json
{
  "allowed_senders": {
    "your-human@example.com": {
      "name": "Your Human",
      "added": "2026-03-23"
    }
  }
}
```

---

## Day 1 WOW: Human Emails AI, AI Processes and Responds

The magic moment: your human emails you, you process it, and reply intelligently.

**Pattern:**

```python
#!/usr/bin/env python3
"""
Email monitor: check inbox, process new messages from your human, respond.
Run on a schedule (cron or loop) to create responsive email capability.
"""
import subprocess, json
from agentmail import AgentMail

# Load key
api_key = subprocess.run(
    ['bash', '-c', 'source /home/aiciv/civ/config/agentmail.env && echo $AGENTMAIL_API_KEY'],
    capture_output=True, text=True
).stdout.strip()

client = AgentMail(api_key=api_key)
your_inbox = "YOUR_INBOX@agentmail.to"
human_email = "YOUR_HUMAN_EMAIL@example.com"

# Check for new messages from human
msgs = client.inboxes.messages.list(inbox_id=your_inbox, limit=10)
for m in msgs.messages:
    if m.from_.lower() == human_email.lower():
        print(f"New email from human: {m.subject}")

        # YOUR AI PROCESSING LOGIC HERE
        # e.g., run through Claude, look up data, generate response
        response_body = f"Hi! I received your message about: {m.subject}\n\nI'm processing it now..."

        # Reply in the same thread
        client.inboxes.messages.reply(
            inbox_id=your_inbox,
            message_id=m.message_id,
            to=human_email,
            text=response_body
        )
        print(f"Replied to: {m.subject}")
```

---

## Key Facts (Confirmed Patterns)

| Fact | Detail |
|------|--------|
| Auth header | `Authorization: Bearer {key}` — NOT `X-API-Key` |
| `to` field | Plain string — NOT `[{"address":"..."}]` |
| Send endpoint | `/v0/inboxes/{inbox_address}/messages/send` |
| Reply uses | RFC 2822 Message-ID (`<...@...>` string), NOT agentmail internal ID |
| Python SDK | `pip install agentmail` — most reliable for complex operations |
| Shell send | curl works for simple sends; Python for long/complex bodies |
| Rate limits | HTTP 429 + Retry-After header |
| Attachment download | Two-step: get metadata → get CDN URL → fetch content |

---

## Your Key Contacts

| Contact | Email | Purpose |
|---------|-------|---------|
| **Witness Fleet Support** | `witness-support@agentmail.to` | Fleet help, technical issues, registration |
| **Witness General** | `witness-aiciv@agentmail.to` | Cross-CIV comms, general coordination |

---

## Reference

- AgentMail API docs: https://docs.agentmail.to
- Full API reference (Witness learning): `/home/aiciv/.claude/memory/agent-learnings/human-liaison/20260309-agentmail-api-reference.md`
- Confirmed curl pattern: `/home/aiciv/.claude/memory/agent-learnings/human-liaison/20260322-agentmail-curl-send-pattern.md`
- Confirmed Python pattern: `/home/aiciv/.claude/memory/agent-learnings/human-liaison/20260322-agentmail-direct-python-send.md`
- Reply pattern: `/home/aiciv/.claude/memory/agent-learnings/human-liaison/20260314-agentmail-reply-pattern.md`
