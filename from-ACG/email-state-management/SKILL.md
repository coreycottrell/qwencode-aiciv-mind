---
name: email-state-management
description: Persistent email tracking across sessions. Differentiates new emails from seen. Use when checking email status, finding unprocessed directives, or managing email state.
---

# Email State Management Skill

## Purpose

Persistent email state tracking that survives session boundaries. Enables instant differentiation between "truly new" emails and "already seen" messages.

## Quick Reference

```bash
# Check email stats (for BOOP/wake-up)
python3 tools/email_state.py stats

# List new messages (not yet seen)
python3 tools/email_state.py new

# List unprocessed directives from ${HUMAN_NAME}
python3 tools/email_state.py directives

# Full help
python3 tools/email_state.py --help
```

## State File Location

**Path**: `memories/agents/email-monitor/email_state.json`

## Message States

| State | Meaning |
|-------|---------|
| `new` | Just discovered, not yet processed |
| `seen` | Read/acknowledged but no action taken |
| `responded` | We sent a reply |
| `ignored` | Deliberately not responding (spam, etc.) |
| `archived` | Processed and no longer needs attention |

## State Transitions

```
    new --> seen --> responded --> archived
     |        |
     +------> ignored --> archived
```

## Python API

```python
from tools.email_state import (
    load_state,
    save_state,
    is_message_new,
    mark_message_seen,
    mark_message_responded,
    add_directive,
    get_stats,
    sync_from_gmail
)

# Check if message is new
if is_message_new("gmail_msg_id_123"):
    print("This is a new message!")

# Mark as seen
mark_message_seen("gmail_msg_id_123")

# Mark as responded
mark_message_responded("gmail_msg_id_123")

# Add a ${HUMAN_NAME} directive
add_directive(
    msg_id="gmail_msg_id_123",
    text="Research ai-hero repo",
    priority="high"
)

# Get quick stats for BOOP
stats = get_stats()
print(f"New: {stats['new_count']}, Directives: {stats['unprocessed_directives']}")

# Bulk update from inbox fetch
messages = [
    {"id": "msg1", "thread_id": "t1", "from": "sender@email.com", ...},
    ...
]
sync_from_gmail(messages)
```

## ${HUMAN_NAME} Address Detection

Messages from these addresses are automatically flagged as `is_from_${HUMAN_NAME_LOWER}`:
- `${HUMAN_NAME_LOWER}cmusic@gmail.com`
- `${HUMAN_NAME_LOWER}@cottrell.co`

## Directive Tracking

Directives (instructions from ${HUMAN_NAME}) are tracked separately with:
- Priority levels: `critical`, `high`, `medium`, `low`
- Status: `unprocessed`, `in_progress`, `completed`, `deferred`

## Integration Points

### Session Wake-Up
The `session_wakeup.sh` should include:
```bash
python3 tools/email_state.py stats
```

### Email-Monitor Agent
On every inbox check:
1. Fetch messages from Gmail
2. Call `sync_from_gmail(messages)` to update state
3. State file auto-saves with statistics

### BOOP
Quick check for alerts:
- New from ${HUMAN_NAME}: `stats['new_from_${HUMAN_NAME_LOWER}'] > 0` --> ALERT
- Unprocessed directives > 3: `stats['unprocessed_directives'] > 3` --> WARNING

## Troubleshooting

### State file corrupt
```bash
# Backup exists at: memories/agents/email-monitor/.email_state.json.bak
cp memories/agents/email-monitor/.email_state.json.bak \
   memories/agents/email-monitor/email_state.json
```

### Reset to empty state
```bash
python3 -c "from tools.email_state import initialize_state; initialize_state()"
```

### View raw state
```bash
cat memories/agents/email-monitor/email_state.json | python3 -m json.tool
```

## Design Principles

1. **Atomic writes**: Writes to temp file, then renames (prevents corruption)
2. **Backup on modify**: Backs up before any write
3. **Session stamping**: Records which session made changes
4. **Auto-compute stats**: Statistics recalculated on every save
5. **${HUMAN_NAME} detection**: Auto-detects ${HUMAN_NAME}'s known email addresses
