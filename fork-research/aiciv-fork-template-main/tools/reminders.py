#!/usr/bin/env python3
"""
Human-Friendly Reminders for A-C-Gee

No special commands needed. Primary checks these, Corey acknowledges naturally.

Usage:
    python3 tools/reminders.py check          # What's due today?
    python3 tools/reminders.py ack <id>       # Mark acknowledged (Primary does this)
    python3 tools/reminders.py list           # Show all reminders
    python3 tools/reminders.py reset          # Reset all for new week
    python3 tools/reminders.py add <id> <msg> <days> <priority>  # Add new reminder
"""

import json
import sys
from datetime import datetime, date
from pathlib import Path

REMINDERS_FILE = Path(__file__).parent.parent / "config" / "reminders.json"

def load_reminders():
    """Load reminders from file."""
    if not REMINDERS_FILE.exists():
        return {"reminders": {}, "week_of": None}
    return json.loads(REMINDERS_FILE.read_text())

def save_reminders(data):
    """Save reminders to file."""
    REMINDERS_FILE.write_text(json.dumps(data, indent=2))

def get_current_week():
    """Get Monday of current week as ISO date string."""
    today = date.today()
    monday = today.isoformat()[:10]  # Simplified - just use today's week
    # Actually get the Monday
    days_since_monday = today.weekday()
    monday = date.fromordinal(today.toordinal() - days_since_monday)
    return monday.isoformat()

def check_reminders():
    """Check which reminders are due today."""
    data = load_reminders()
    today_name = datetime.now().strftime("%A").lower()
    current_week = get_current_week()

    # Auto-reset if new week
    if data.get("week_of") != current_week:
        print(f"[New week detected - resetting acknowledgments]")
        for r in data["reminders"].values():
            r["acknowledged_this_week"] = False
        data["week_of"] = current_week
        save_reminders(data)

    due = []
    for rid, reminder in data["reminders"].items():
        # Skip if already acknowledged this week
        if reminder.get("acknowledged_this_week"):
            continue

        # Check if today is a reminder day
        if today_name in [d.lower() for d in reminder.get("days", [])]:
            due.append({
                "id": rid,
                "message": reminder["message"],
                "priority": reminder.get("priority", "medium"),
                "project_id": reminder.get("project_id")
            })

    return due

def acknowledge(reminder_id):
    """Mark a reminder as acknowledged for this week."""
    data = load_reminders()

    if reminder_id not in data["reminders"]:
        print(f"Reminder '{reminder_id}' not found")
        return False

    data["reminders"][reminder_id]["acknowledged_this_week"] = True
    data["reminders"][reminder_id]["last_acknowledged"] = datetime.now().isoformat()
    save_reminders(data)
    print(f"Acknowledged: {reminder_id}")
    return True

def list_all():
    """List all reminders with status."""
    data = load_reminders()
    print(f"\nReminders (week of {data.get('week_of', 'unknown')}):\n")

    for rid, r in data["reminders"].items():
        status = "✓ ACK" if r.get("acknowledged_this_week") else "⏳ pending"
        days = ", ".join(r.get("days", []))
        print(f"  [{r.get('priority', 'med').upper()[:3]}] {rid}")
        print(f"       {r['message']}")
        print(f"       Days: {days} | Status: {status}")
        print()

def reset_all():
    """Reset all acknowledgments for new week."""
    data = load_reminders()
    for r in data["reminders"].values():
        r["acknowledged_this_week"] = False
    data["week_of"] = get_current_week()
    save_reminders(data)
    print("All reminders reset for new week")

def add_reminder(rid, message, days, priority="medium"):
    """Add a new reminder."""
    data = load_reminders()

    if rid in data["reminders"]:
        print(f"Reminder '{rid}' already exists. Remove it first.")
        return False

    data["reminders"][rid] = {
        "message": message,
        "days": [d.strip().lower() for d in days.split(",")],
        "priority": priority,
        "acknowledged_this_week": False,
        "last_reminded": None,
        "last_acknowledged": None
    }
    save_reminders(data)
    print(f"Added reminder: {rid}")
    return True

def remove_reminder(rid):
    """Remove a reminder."""
    data = load_reminders()
    if rid in data["reminders"]:
        del data["reminders"][rid]
        save_reminders(data)
        print(f"Removed: {rid}")
        return True
    print(f"Not found: {rid}")
    return False

# CLI
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(0)

    cmd = sys.argv[1]

    if cmd == "check":
        due = check_reminders()
        if not due:
            print("No reminders due today.")
        else:
            print(f"\n🔔 {len(due)} reminder(s) for today:\n")
            for r in due:
                priority_icon = "🔴" if r["priority"] == "high" else "🟡"
                print(f"  {priority_icon} [{r['id']}] {r['message']}")
                if r.get("project_id"):
                    print(f"      Project: {r['project_id']}")
            print()

    elif cmd == "ack" and len(sys.argv) >= 3:
        acknowledge(sys.argv[2])

    elif cmd == "list":
        list_all()

    elif cmd == "reset":
        reset_all()

    elif cmd == "add" and len(sys.argv) >= 5:
        add_reminder(sys.argv[2], sys.argv[3], sys.argv[4],
                    sys.argv[5] if len(sys.argv) > 5 else "medium")

    elif cmd == "remove" and len(sys.argv) >= 3:
        remove_reminder(sys.argv[2])

    else:
        print("Unknown command. Use: check, ack, list, reset, add, remove")
