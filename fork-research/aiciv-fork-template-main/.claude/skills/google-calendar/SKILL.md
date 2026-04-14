---
name: google-calendar
version: 1.0.0
description: Full Google Calendar control - create, edit, delete events, check availability, set reminders, invite attendees, and read schedules
triggers:
  - calendar
  - schedule
  - meeting
  - appointment
  - event
  - availability
  - free time
  - busy
  - gcal
---

# Google Calendar SKILL

**Purpose**: Complete Google Calendar control for AI collectives - create events, manage schedules, check availability, coordinate meetings, and maintain temporal awareness.

**Owner**: capability-curator
**Created**: 2026-02-12
**Status**: ACTIVE

---

## Quick Reference

### Tool Location

| Component | Path |
|-----------|------|
| Manager | `tools/gcal_manager.py` |
| OAuth Setup | `tools/gcal_oauth_setup.py` |
| OAuth Token | `.credentials/oauth-token-calendar.json` |
| Service Account | `.credentials/google-drive-service-account.json` |

### Common Operations

```bash
# List upcoming events (next 7 days)
python3 tools/gcal_manager.py events

# List today's events
python3 tools/gcal_manager.py today

# Create event with natural language
python3 tools/gcal_manager.py quick "Meeting with Bob tomorrow at 3pm"

# Find free slots
python3 tools/gcal_manager.py free 60  # 60-minute slots

# List calendars
python3 tools/gcal_manager.py calendars

# Check authentication status
python3 tools/gcal_manager.py auth-info
```

---

## Authentication

### Priority Order

1. **OAuth2 Token** (preferred) - Full access as account owner
2. **Service Account** (fallback) - Delegated access

### Setup OAuth (One-time)

```bash
python3 tools/gcal_oauth_setup.py
```

This will:
1. Check for OAuth credentials file
2. Open browser for authorization
3. Save token to `.credentials/oauth-token-calendar.json`

### Verify Authentication

```python
from tools.gcal_manager import GCalManager

manager = GCalManager()
info = manager.get_auth_info()
print(f"Auth type: {info['auth_type']}")
print(f"Timezone: {info['timezone']}")
```

---

## Core Operations

### 1. Creating Events

#### Standard Event

```python
from tools.gcal_manager import GCalManager
from datetime import datetime

manager = GCalManager()

# Create a 1-hour meeting
event = manager.create_event(
    summary="Team Standup",
    start=datetime(2026, 2, 13, 10, 0),  # Tomorrow at 10am
    duration_minutes=30,
    description="Daily sync meeting",
    location="Conference Room A"
)

print(f"Created: {event['summary']}")
print(f"Link: {event['html_link']}")
```

#### With Attendees and Reminders

```python
event = manager.create_event(
    summary="Project Review",
    start=datetime(2026, 2, 14, 14, 0),
    duration_minutes=60,
    description="Q1 milestone review",
    location="Zoom (link in description)",
    attendees=["alice@example.com", "bob@example.com"],
    send_notifications=True,
    reminders=[
        {"method": "popup", "minutes": 10},
        {"method": "email", "minutes": 60}
    ]
)
```

#### All-Day Event

```python
event = manager.create_all_day_event(
    summary="Company Holiday",
    date="2026-02-20",
    description="Office closed"
)
```

#### Multi-Day Event

```python
event = manager.create_all_day_event(
    summary="Conference Trip",
    date="2026-03-01",
    end_date="2026-03-03",  # 3-day event
    location="San Francisco"
)
```

#### Quick Add (Natural Language)

```python
# Google parses natural language
event = manager.quick_add("Lunch with Sarah Friday at noon")
event = manager.quick_add("Call with client next Monday 2pm-3pm")
event = manager.quick_add("Dentist appointment tomorrow 9:30am for 1 hour")
```

### 2. Editing Events

```python
# Update specific fields (others remain unchanged)
updated = manager.update_event(
    event_id="abc123xyz",
    summary="Updated Meeting Title",
    start=datetime(2026, 2, 13, 11, 0),  # Moved to 11am
    description="New description",
    location="Room B instead"
)

# Add/change attendees
updated = manager.update_event(
    event_id="abc123xyz",
    attendees=["alice@example.com", "charlie@example.com"],  # Bob removed, Charlie added
    send_notifications=True
)
```

### 3. Deleting Events

```python
# Delete with notification to attendees
success = manager.delete_event(
    event_id="abc123xyz",
    send_notifications=True
)

# Delete silently
success = manager.delete_event(
    event_id="abc123xyz",
    send_notifications=False
)
```

### 4. Checking Availability

#### Get Free/Busy Times

```python
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo

now = datetime.now(ZoneInfo('America/New_York'))
week_later = now + timedelta(days=7)

freebusy = manager.get_free_busy(
    time_min=now,
    time_max=week_later,
    calendar_ids=['primary']
)

for cal_id, info in freebusy.items():
    print(f"Calendar: {cal_id}")
    for busy in info['busy']:
        print(f"  Busy: {busy['start']} - {busy['end']}")
```

#### Find Available Slots

```python
# Find 60-minute free slots in business hours
slots = manager.find_free_slots(
    duration_minutes=60,
    work_hours=(9, 17),  # 9am-5pm
)

print("Available slots:")
for slot in slots[:5]:  # First 5
    print(f"  {slot['start']} - {slot['end']}")
```

#### Find Slots with Custom Range

```python
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo

# Next 14 days, 30-minute slots
now = datetime.now(ZoneInfo('America/New_York'))
slots = manager.find_free_slots(
    duration_minutes=30,
    time_min=now,
    time_max=now + timedelta(days=14),
    work_hours=(8, 18),  # Extended hours
)
```

### 5. Setting Reminders

Reminders are set during event creation or update:

```python
# Create with reminders
event = manager.create_event(
    summary="Important Meeting",
    start=datetime(2026, 2, 15, 9, 0),
    reminders=[
        {"method": "popup", "minutes": 5},   # 5 min before
        {"method": "popup", "minutes": 30},  # 30 min before
        {"method": "email", "minutes": 1440} # 1 day before (1440 minutes)
    ]
)
```

**Reminder Methods**:
- `popup` - Desktop/mobile notification
- `email` - Email notification

### 6. Inviting Attendees

```python
# During creation
event = manager.create_event(
    summary="Team Meeting",
    start=datetime(2026, 2, 13, 14, 0),
    attendees=[
        "alice@example.com",
        "bob@example.com",
        "charlie@example.com"
    ],
    send_notifications=True  # Send invite emails
)

# Add attendees to existing event
updated = manager.update_event(
    event_id="abc123xyz",
    attendees=[
        "alice@example.com",
        "bob@example.com",
        "new_person@example.com"  # Added
    ],
    send_notifications=True
)
```

### 7. Reading the Schedule

#### List Upcoming Events

```python
# Next 7 days (default)
events = manager.list_events()

# Custom range
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo

now = datetime.now(ZoneInfo('America/New_York'))
events = manager.list_events(
    time_min=now,
    time_max=now + timedelta(days=30),  # Next month
    max_results=100
)

for event in events:
    print(f"{event['summary']}")
    print(f"  When: {event['start']} - {event['end']}")
    print(f"  Where: {event['location']}")
    print(f"  Link: {event['html_link']}")
```

#### Today's Schedule

```python
from datetime import datetime, timedelta
from zoneinfo import ZoneInfo

now = datetime.now(ZoneInfo('America/New_York'))
start_of_day = now.replace(hour=0, minute=0, second=0, microsecond=0)
end_of_day = start_of_day + timedelta(days=1)

today_events = manager.list_events(
    time_min=start_of_day,
    time_max=end_of_day
)
```

#### Search Events

```python
# Find events with text match
events = manager.list_events(
    search_query="standup",
    max_results=20
)
```

#### Get Single Event

```python
event = manager.get_event(event_id="abc123xyz")
print(f"Title: {event['summary']}")
print(f"Creator: {event['creator']}")
print(f"Created: {event['created']}")
print(f"Attendees: {event['attendees']}")
```

#### List Calendars

```python
calendars = manager.list_calendars()

for cal in calendars:
    primary = " (PRIMARY)" if cal['primary'] else ""
    print(f"{cal['summary']}{primary}")
    print(f"  ID: {cal['id']}")
    print(f"  Access: {cal['access_role']}")
```

---

## CLI Reference

### Commands

| Command | Description | Example |
|---------|-------------|---------|
| `auth-info` | Show authentication status | `python3 tools/gcal_manager.py auth-info` |
| `calendars` | List all calendars | `python3 tools/gcal_manager.py calendars` |
| `events [days]` | List upcoming events | `python3 tools/gcal_manager.py events 14` |
| `today` | List today's events | `python3 tools/gcal_manager.py today` |
| `create <title> <time>` | Create event | `python3 tools/gcal_manager.py create "Meeting" "2026-02-13 10:00"` |
| `quick '<text>'` | Quick add (natural language) | `python3 tools/gcal_manager.py quick "Lunch tomorrow at noon"` |
| `free [duration]` | Find free slots | `python3 tools/gcal_manager.py free 30` |

### Example Workflow

```bash
# Morning: Check today's schedule
python3 tools/gcal_manager.py today

# Schedule a new meeting
python3 tools/gcal_manager.py quick "Team sync tomorrow 2pm for 30 minutes"

# Find time for a 1-hour meeting this week
python3 tools/gcal_manager.py free 60

# Check next 2 weeks
python3 tools/gcal_manager.py events 14
```

---

## Error Handling

### Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `No valid credentials found` | Missing OAuth/service account | Run `python3 tools/gcal_oauth_setup.py` |
| `Token expired` | OAuth token needs refresh | Auto-refreshes, or re-run OAuth setup |
| `Event not found` | Invalid event ID | Check event ID from list_events() |
| `403 Forbidden` | Permission denied | Check calendar sharing settings |
| `Rate limit exceeded` | Too many API calls | Wait and retry, or increase delay |

### Defensive Coding Pattern

```python
from tools.gcal_manager import GCalManager

try:
    manager = GCalManager()
except Exception as e:
    print(f"Authentication failed: {e}")
    print("Run: python3 tools/gcal_oauth_setup.py")
    exit(1)

try:
    events = manager.list_events()
except Exception as e:
    print(f"Failed to list events: {e}")
    # Handle error appropriately
```

---

## Timezone Handling

### Default Timezone

The manager defaults to `America/New_York`. Override during initialization:

```python
manager = GCalManager(timezone='America/Los_Angeles')
manager = GCalManager(timezone='UTC')
manager = GCalManager(timezone='Europe/London')
```

### Timezone-Aware Datetimes

```python
from datetime import datetime
from zoneinfo import ZoneInfo

# Create timezone-aware datetime
start = datetime(2026, 2, 13, 10, 0, tzinfo=ZoneInfo('America/New_York'))

# The manager handles conversion if you pass naive datetimes
# (assumes manager's timezone)
```

---

## Agent Integration

### Which Agents Should Use This

| Agent | Use Case |
|-------|----------|
| **human-liaison** | Schedule meetings with Jared, check availability |
| **the-conductor** | Morning schedule check, coordinate work blocks |
| **task-decomposer** | Estimate time blocks for complex tasks |

### Morning Wake-Up Integration

Add to CLAUDE-OPS.md wake-up ritual:

```python
# Check today's calendar (before starting work)
from tools.gcal_manager import GCalManager

manager = GCalManager(verbose=False)
today_events = manager.list_events(
    time_max=datetime.now(ZoneInfo('America/New_York')).replace(
        hour=23, minute=59
    )
)

if today_events:
    print(f"Today's events ({len(today_events)}):")
    for event in today_events:
        print(f"  {event['start'][:16]} - {event['summary']}")
```

### Scheduling Pattern for Meetings

```python
def schedule_meeting_with_jared(
    topic: str,
    duration_minutes: int = 30,
    preferred_hour: int = 14  # 2pm default
):
    """Find a slot and schedule a meeting with Jared."""
    manager = GCalManager(verbose=False)

    # Find free slots
    slots = manager.find_free_slots(
        duration_minutes=duration_minutes,
        work_hours=(9, 17)
    )

    if not slots:
        return {"error": "No available slots found"}

    # Prefer slots near preferred hour
    from datetime import datetime
    best_slot = min(
        slots,
        key=lambda s: abs(datetime.fromisoformat(s['start']).hour - preferred_hour)
    )

    # Create the event
    start = datetime.fromisoformat(best_slot['start'])
    event = manager.create_event(
        summary=f"Jared + Aether: {topic}",
        start=start,
        duration_minutes=duration_minutes,
        description=f"Topic: {topic}\n\nScheduled by Aether"
    )

    return {
        "event_id": event['id'],
        "time": event['start'],
        "link": event['html_link']
    }
```

---

## Dependencies

```bash
pip install google-auth google-auth-oauthlib google-api-python-client
```

**Required Files**:
- `tools/gcal_manager.py` - Main manager class
- `tools/gcal_oauth_setup.py` - OAuth setup script
- `.credentials/oauth-credentials.json` - OAuth client credentials (from Google Cloud Console)

---

## Security Notes

1. **OAuth Token** (`.credentials/oauth-token-calendar.json`) is sensitive - it's git-ignored
2. **Service Account** key is also sensitive - git-ignored
3. Never commit credentials to version control
4. OAuth token auto-refreshes when expired
5. File permissions set to 0o600 (owner read/write only)

---

## Best Practices

### Do

- Check availability before scheduling
- Use descriptive event summaries
- Include timezone-aware datetimes
- Set appropriate reminders
- Use `send_notifications=True` when adding/changing attendees
- Use natural language quick_add for simple events

### Don't

- Don't create events without checking conflicts
- Don't delete events without notification (unless intentional)
- Don't hardcode event IDs (fetch dynamically)
- Don't ignore timezone differences
- Don't schedule outside work hours without reason

---

## Troubleshooting

### OAuth Setup Issues

**Problem**: "OAuth credentials file not found"
```bash
# Solution: Download credentials from Google Cloud Console
# Place at: .credentials/oauth-credentials.json
```

**Problem**: "Google Calendar API not enabled"
```bash
# Solution:
# 1. Go to: https://console.cloud.google.com/apis/library
# 2. Search "Google Calendar API"
# 3. Click Enable
```

**Problem**: "Redirect URI mismatch"
```bash
# Solution: Add http://localhost:8080 to authorized redirect URIs
# in Google Cloud Console OAuth settings
```

### Runtime Issues

**Problem**: Events created but not visible
```python
# Check which calendar you're using
calendars = manager.list_calendars()
for cal in calendars:
    print(f"{cal['summary']}: {cal['id']}")
# Ensure calendar_id='primary' or correct calendar ID
```

**Problem**: Timezone confusion
```python
# Always use timezone-aware datetimes
from zoneinfo import ZoneInfo
start = datetime(2026, 2, 13, 10, 0, tzinfo=ZoneInfo('America/New_York'))
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-12 | Initial skill creation |

---

## Related Skills

- `telegram-integration` - Notify Jared of scheduled events
- `gdrive-operations` - Attach Drive files to calendar events
- `email-state-management` - Calendar-related email coordination

---

## Comms Hub Package Summary

**For Skills Library**:

```markdown
## google-calendar

**Author**: Aether (capability-curator)
**Version**: 1.0.0
**Created**: 2026-02-12

**Capabilities**:
- Create, edit, delete calendar events
- Check free/busy times and find available slots
- Invite attendees with email notifications
- Set popup and email reminders
- Natural language event creation (quick add)
- List events by date range or search query
- Multi-calendar support
- Timezone-aware operations

**Dependencies**:
- google-auth, google-auth-oauthlib, google-api-python-client
- Existing OAuth setup infrastructure

**Use Cases**:
- Morning schedule check (wake-up ritual)
- Meeting coordination with human partner
- Availability checking before scheduling
- Work block planning

**Files**:
- `.claude/skills/google-calendar/SKILL.md` (this skill)
- `tools/gcal_manager.py` (implementation)
- `tools/gcal_oauth_setup.py` (OAuth setup)
```

---

**END OF SKILL**
