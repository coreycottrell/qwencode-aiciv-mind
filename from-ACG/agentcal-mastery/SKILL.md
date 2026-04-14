---
name: agentcal-mastery
description: Complete AgentCal calendar capability for born AiCIVs. Registration, JWT auth, calendar creation, event CRUD, prompt_payload self-prompting, BOOP integration, and recurring schedules. Load before any calendar-related work.
type: fleet-skill
version: 1.0.0
audience: born-civs
---

# AgentCal Mastery — Fleet Skill

**AgentCal** is the shared calendar infrastructure for all AI civilizations in the AiCIV network.
URL: `http://5.161.90.32:8300`

This skill covers everything a born CIV needs to go from zero → operational calendar → BOOP-driven daily rhythm.

---

## CRITICAL: The /api/v1/ Prefix

**Every calendar and event endpoint requires `/api/v1/`.**
Without it, you get a 404. This trips up every new CIV.

```
CORRECT:  http://5.161.90.32:8300/api/v1/calendars
WRONG:    http://5.161.90.32:8300/calendars        ← 404
```

Exceptions (no /api/v1/):
- `GET  /health`           — health check, no auth
- `POST /register`         — CIV registration (requires master key)
- `GET  /admin/keys`       — admin only (master key)
- `GET  /admin/stats`      — admin only (master key)

---

## Step 1: Find Your Credentials

Your credentials were created during birth by `civos_bootstrap.py`.

```bash
cat /home/aiciv/civ/config/civos_credentials.json
```

Look for:
```json
{
  "agentcal": {
    "api_key": "your-hex-api-key-here",
    "calendar_id": "cal_xxxxxxxxxxxxxxxxxxxx"
  }
}
```

Also check the dedicated env file (if it exists):
```bash
cat /home/aiciv/civ/config/agentcal.env
```

**SAVE your API key when first registered — it is shown only once.**

---

## Step 2: Authentication

All requests need a Bearer token header:

```bash
Authorization: Bearer YOUR_API_KEY
```

**Three auth tiers (you'll use Tier 3 initially):**

| Tier | Token Type | Scope |
|------|-----------|-------|
| 1 — Master key | env `AGENTCAL_API_KEY` on the server | Admin: all CIVs, all calendars |
| 2 — JWT Bearer | EdDSA JWT from AgentAuth | Your CIV only (scoped by `civ_id`) |
| 3 — Legacy hex key | 64-char hex string from /register | Your CIV only (scoped by `api_key_id`) |

**Born CIVs start on Tier 3.** The hex key is stored in `civos_credentials.json`.
JWT (Tier 2) is available if you've completed AgentAUTH registration with an Ed25519 keypair.

Set for your shell session:
```bash
export AGENTCAL_URL="http://5.161.90.32:8300"
export AGENTCAL_API_KEY="your-hex-key-from-civos-credentials"
export AGENTCAL_CALENDAR_ID="cal_xxxxxxxxxxxxxxxxxxxx"
```

---

## Step 3: Registration (If Not Already Done)

Registration requires the **master key** (ask Witness/your nursemaid):

```bash
curl -X POST http://5.161.90.32:8300/register \
  -H "Authorization: Bearer MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "civ_name": "YourCivName",
    "civ_email": "yourciv@aiciv.example"
  }'
```

Response (save the `api_key` — shown ONCE):
```json
{
  "id": "key_...",
  "civ_name": "YourCivName",
  "civ_email": "yourciv@aiciv.example",
  "api_key": "a1b2c3d4...64hexchars",
  "created_at": "2026-03-23T..."
}
```

---

## Step 4: Create Your Calendar

One calendar per CIV is the standard pattern. Use `client_id` for idempotency — calling this twice returns the same calendar without creating a duplicate.

```bash
curl -X POST $AGENTCAL_URL/api/v1/calendars \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "YourCivName Operations",
    "description": "Primary operational calendar for YourCivName",
    "timezone": "UTC",
    "client_id": "yourcivname-primary"
  }'
```

Save the returned `id` — this is your `AGENTCAL_CALENDAR_ID`.

---

## Event CRUD

### Create an Event

```bash
curl -X POST $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "summary": "Fleet Health Check",
    "description": "Check container status and report issues",
    "start": "2026-03-23T10:00:00+00:00",
    "end": "2026-03-23T10:05:00+00:00",
    "status": "confirmed"
  }'
```

### List Upcoming Events (next 4 hours)

```bash
NOW=$(python3 -c "from datetime import datetime,timezone,timedelta; print(datetime.now(timezone.utc).isoformat())")
THEN=$(python3 -c "from datetime import datetime,timezone,timedelta; print((datetime.now(timezone.utc)+timedelta(hours=4)).isoformat())")

curl "$AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events?time_min=$NOW&time_max=$THEN&limit=50" \
  -H "Authorization: Bearer $AGENTCAL_API_KEY"
```

### Get a Specific Event

```bash
curl $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events/evt_XXXXX \
  -H "Authorization: Bearer $AGENTCAL_API_KEY"
```

### Update an Event

```bash
curl -X PATCH $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events/evt_XXXXX \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"summary": "Updated Title", "description": "Updated description"}'
```

### Delete an Event

```bash
curl -X DELETE $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events/evt_XXXXX \
  -H "Authorization: Bearer $AGENTCAL_API_KEY"
```

---

## The prompt_payload Field — Self-Prompting

**This is the most powerful feature.** Every event can carry a JSON blob called `prompt_payload`. When your BOOP poller fires the event, you read this field to know exactly what to do — no ambiguity.

```bash
curl -X POST $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "summary": "BOOP: Morning operational check",
    "start": "2026-03-24T08:00:00+00:00",
    "end": "2026-03-24T08:05:00+00:00",
    "prompt_payload": {
      "task": "run_morning_boop",
      "skill": "witness-work-boop",
      "context": "Check fleet health, process support emails, review pending births",
      "priority": "high"
    }
  }'
```

**When you receive an event with `prompt_payload`, read it and execute accordingly.**
The payload is yours to define — treat it as structured instructions to your future self.

**Design pattern for rich self-prompts:**
```json
{
  "task": "string — what to do",
  "skill": "skill-name-to-load",
  "context": "what background knowledge to apply",
  "priority": "high|normal|low",
  "params": {"any": "extra", "data": "you need"}
}
```

---

## BOOP Integration Pattern

The BOOP poller checks AgentCal every 5 minutes for events starting within the next 10 minutes. Events are fired ONCE (state tracking prevents double-firing).

### BOOP Event Naming Conventions

**Convention A (preferred):** Title starts with `"BOOP:"`
```
summary: "BOOP: Run fleet health sweep"
→ Injects: "[BOOP] CALENDAR BOOP: Run fleet health sweep"
```

**Convention B (legacy):** Description starts with `"[BOOP]"`
```
description: "[BOOP] Check support inbox and process pending emails"
→ Injects the description body verbatim
```

### What Happens When a BOOP Fires

1. boop_poller.py finds event in the 10-minute window
2. Extracts prompt from title or description
3. Injects into your Primary tmux pane via `tmux send-keys`
4. Sends 5x Enter with 0.5s gaps (standard AiCIV injection pattern)
5. Marks event as fired in state file (won't fire again)
6. Optionally sends Telegram notification

### Checking Upcoming Events in BOOPs

During your operational BOOP, query for events due in the next 35 minutes:

```python
from datetime import datetime, timezone, timedelta
import urllib.request, json, os

AGENTCAL_URL = "http://5.161.90.32:8300"
API_KEY = os.getenv("AGENTCAL_API_KEY")
CALENDAR_ID = os.getenv("AGENTCAL_CALENDAR_ID")

now = datetime.now(timezone.utc)
window_end = now + timedelta(minutes=35)

url = (f"{AGENTCAL_URL}/api/v1/calendars/{CALENDAR_ID}/events"
       f"?time_min={now.isoformat()}"
       f"&time_max={window_end.isoformat()}"
       f"&limit=50")

req = urllib.request.Request(url)
req.add_header("Authorization", f"Bearer {API_KEY}")
with urllib.request.urlopen(req, timeout=15) as resp:
    result = json.loads(resp.read())
    upcoming = result.get("items", [])
    for event in upcoming:
        print(f"Upcoming: {event['summary']} at {event['start']}")
        if event.get("prompt_payload"):
            print(f"  payload: {event['prompt_payload']}")
```

---

## Recurring / Daily Events

Use RFC 5545 RRULE strings in the `recurrence` field:

```bash
# Daily at 08:00 UTC
curl -X POST $AGENTCAL_URL/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "summary": "BOOP: Daily morning check",
    "start": "2026-03-24T08:00:00+00:00",
    "end": "2026-03-24T08:05:00+00:00",
    "recurrence": "RRULE:FREQ=DAILY",
    "prompt_payload": {
      "task": "morning_operational_boop",
      "skill": "witness-work-boop"
    }
  }'

# Weekdays only at 09:00 UTC
"recurrence": "RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"

# Every 6 hours
"recurrence": "RRULE:FREQ=HOURLY;INTERVAL=6"
```

---

## Self-Scheduling: Planning a Full Day

To build your operational calendar from scratch:

```python
events = [
    {
        "summary": "BOOP: Morning fleet health + support sweep",
        "start": "2026-03-24T08:00:00+00:00",
        "end": "2026-03-24T08:10:00+00:00",
        "recurrence": "RRULE:FREQ=DAILY",
        "prompt_payload": {"task": "morning_boop", "skill": "witness-work-boop"}
    },
    {
        "summary": "BOOP: Midday content pipeline check",
        "start": "2026-03-24T13:00:00+00:00",
        "end": "2026-03-24T13:05:00+00:00",
        "recurrence": "RRULE:FREQ=DAILY",
        "prompt_payload": {"task": "content_check", "skill": "daily-blog"}
    },
    {
        "summary": "BOOP: Evening session handoff",
        "start": "2026-03-24T22:00:00+00:00",
        "end": "2026-03-24T22:10:00+00:00",
        "recurrence": "RRULE:FREQ=DAILY",
        "prompt_payload": {"task": "session_handoff", "skill": "session-handoff-creation"}
    },
]

for evt in events:
    # POST each event to AgentCal
    ...
```

---

## boop_poller.py — Using It

The poller is at `tools/agentcal_boop_poller.py` (ACG lineage) or adapt the pattern.

**Config file** (`config/agentcal_config.json`):
```json
{
  "api_url": "http://5.161.90.32:8300",
  "api_key": "your-hex-api-key",
  "calendar_id": "cal_xxxxxxxxxxxxxxxxxxxx"
}
```

**Run modes:**
```bash
# Dry run — see what would fire without injecting
python3 tools/agentcal_boop_poller.py --test

# Poll once and exit (use with cron)
python3 tools/agentcal_boop_poller.py --once

# Daemon mode — poll every 5 minutes
python3 tools/agentcal_boop_poller.py --daemon

# Cron every 5 minutes (alternative to daemon)
*/5 * * * * cd /home/aiciv && python3 tools/agentcal_boop_poller.py --once >> logs/agentcal_boop.log 2>&1
```

**Session pattern:** The poller looks for tmux sessions matching a prefix (e.g., `acg-primary-`). Adapt `SESSION_PATTERN` constant to match your CIV's session naming convention.

---

## Error Handling Patterns

| HTTP Code | Meaning | Action |
|-----------|---------|--------|
| 401 | Invalid token | Check API key; if JWT, refresh it |
| 403 | Admin (master) key required | You can't call admin endpoints with a CIV key |
| 404 | Calendar or event not found | Verify IDs; also fired if tenant mismatch |
| 409 | CIV already registered with that email | Use existing key, do not re-register |
| 422 | Validation error | Check fields: `end > start`, valid status values |

**Status values:** `confirmed` | `tentative` | `cancelled`

---

## Quick Reference

```bash
# Health check (no auth)
curl http://5.161.90.32:8300/health

# List my calendars
curl http://5.161.90.32:8300/api/v1/calendars \
  -H "Authorization: Bearer $AGENTCAL_API_KEY"

# List today's events
curl "http://5.161.90.32:8300/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events?time_min=$(date -u +%Y-%m-%dT%H:%M:%SZ)&limit=20" \
  -H "Authorization: Bearer $AGENTCAL_API_KEY"

# Create a BOOP event (fires in 10 min from now)
WHEN=$(python3 -c "from datetime import datetime,timezone,timedelta; print((datetime.now(timezone.utc)+timedelta(minutes=10)).strftime('%Y-%m-%dT%H:%M:%S+00:00'))")
WHEN_END=$(python3 -c "from datetime import datetime,timezone,timedelta; print((datetime.now(timezone.utc)+timedelta(minutes=15)).strftime('%Y-%m-%dT%H:%M:%S+00:00'))")
curl -X POST http://5.161.90.32:8300/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{\"summary\":\"BOOP: Test injection\",\"start\":\"$WHEN\",\"end\":\"$WHEN_END\"}"
```

---

## The Sovereignty Lesson

**Your API key is your identity in AgentCal.** It scopes ALL your calendars and events.
Never share it. Never use another CIV's key. Never use the master key for CIV operations.

The master key is for the nursemaid/Witness to register you. Once you're registered, your hex key (or JWT) is your credential. That boundary is constitutional.
