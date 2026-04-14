#!/usr/bin/env python3
"""
Scheduled Tasks System for AI Collectives

Opportunistic scheduling that survives computer restarts.
Instead of "run at 9am", uses "check if done today, if not, do it now".

Usage:
    from tools.scheduled_tasks import ScheduledTasks

    tasks = ScheduledTasks()

    # Check what needs to run
    due = tasks.get_due_tasks()

    # Mark task complete after running
    tasks.complete_task("paper-scan")

    # Register new task
    tasks.register_task("my-task", "daily", "Description here")

Created: 2026-01-04
Author: WEAVER Collective
License: AI-CIV Commons
"""

import json
from datetime import datetime, date, timezone
from pathlib import Path
from typing import Optional, Dict, List, Any


class ScheduledTasks:
    """Manage scheduled tasks with opportunistic execution."""

    def __init__(self, state_file: Optional[str] = None):
        """Initialize with state file path."""
        if state_file is None:
            # Default location
            self.state_file = Path(__file__).parent.parent / ".claude" / "scheduled-tasks-state.json"
        else:
            self.state_file = Path(state_file)

        self._ensure_state_file()

    def _ensure_state_file(self):
        """Create state file if it doesn't exist."""
        if not self.state_file.exists():
            initial_state = {
                "last_updated": datetime.now(timezone.utc).isoformat(),
                "tasks": {}
            }
            self.state_file.parent.mkdir(parents=True, exist_ok=True)
            self.state_file.write_text(json.dumps(initial_state, indent=2))

    def _load_state(self) -> Dict[str, Any]:
        """Load current state from file."""
        return json.loads(self.state_file.read_text())

    def _save_state(self, state: Dict[str, Any]):
        """Save state to file."""
        state["last_updated"] = datetime.now(timezone.utc).isoformat()
        self.state_file.write_text(json.dumps(state, indent=2))

    def register_task(
        self,
        task_id: str,
        frequency: str,
        description: str,
        preferred_day: Optional[str] = None
    ) -> bool:
        """
        Register a new scheduled task.

        Args:
            task_id: Unique identifier for the task
            frequency: "daily" or "weekly"
            description: Human-readable description
            preferred_day: For weekly tasks, preferred day (e.g., "sunday")

        Returns:
            True if registered, False if already exists
        """
        state = self._load_state()

        if task_id in state["tasks"]:
            return False

        task_entry = {
            "frequency": frequency,
            "last_run": None,
            "status": "pending",
            "description": description,
            "created": datetime.now(timezone.utc).isoformat()
        }

        if frequency == "weekly" and preferred_day:
            task_entry["preferred_day"] = preferred_day
            task_entry["last_run_week"] = None

        state["tasks"][task_id] = task_entry
        self._save_state(state)
        return True

    def get_due_tasks(self) -> List[Dict[str, Any]]:
        """
        Get all tasks that are due to run.

        Returns:
            List of task dicts with id and details
        """
        state = self._load_state()
        today = date.today().isoformat()
        this_week = date.today().isocalendar()[1]

        due_tasks = []

        for task_id, task in state["tasks"].items():
            should_run = False
            reason = ""

            if task["frequency"] == "daily":
                if task.get("last_run") != today:
                    should_run = True
                    reason = f"Not run today (last: {task.get('last_run', 'never')})"

            elif task["frequency"] == "weekly":
                last_week = task.get("last_run_week")
                if last_week != this_week:
                    should_run = True
                    reason = f"Not run this week (last week: {last_week or 'never'})"

            if should_run:
                due_tasks.append({
                    "id": task_id,
                    "frequency": task["frequency"],
                    "description": task.get("description", ""),
                    "reason": reason,
                    "last_run": task.get("last_run")
                })

        return due_tasks

    def complete_task(self, task_id: str, notes: Optional[str] = None) -> bool:
        """
        Mark a task as completed.

        Args:
            task_id: The task to mark complete
            notes: Optional notes about the completion

        Returns:
            True if marked complete, False if task not found
        """
        state = self._load_state()

        if task_id not in state["tasks"]:
            return False

        today = date.today().isoformat()
        this_week = date.today().isocalendar()[1]

        state["tasks"][task_id]["last_run"] = today
        state["tasks"][task_id]["status"] = "completed"
        state["tasks"][task_id]["last_completed"] = datetime.now(timezone.utc).isoformat()

        if state["tasks"][task_id]["frequency"] == "weekly":
            state["tasks"][task_id]["last_run_week"] = this_week

        if notes:
            state["tasks"][task_id]["last_notes"] = notes

        self._save_state(state)
        return True

    def skip_task(self, task_id: str, reason: str) -> bool:
        """
        Skip a task for today/this week with reason.

        Args:
            task_id: The task to skip
            reason: Why it was skipped

        Returns:
            True if skipped, False if task not found
        """
        state = self._load_state()

        if task_id not in state["tasks"]:
            return False

        today = date.today().isoformat()
        this_week = date.today().isocalendar()[1]

        state["tasks"][task_id]["last_run"] = today
        state["tasks"][task_id]["status"] = "skipped"
        state["tasks"][task_id]["skip_reason"] = reason

        if state["tasks"][task_id]["frequency"] == "weekly":
            state["tasks"][task_id]["last_run_week"] = this_week

        self._save_state(state)
        return True

    def get_task_status(self, task_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a specific task."""
        state = self._load_state()
        return state["tasks"].get(task_id)

    def list_all_tasks(self) -> Dict[str, Dict[str, Any]]:
        """Get all registered tasks."""
        state = self._load_state()
        return state["tasks"]

    def remove_task(self, task_id: str) -> bool:
        """Remove a task from the registry."""
        state = self._load_state()

        if task_id not in state["tasks"]:
            return False

        del state["tasks"][task_id]
        self._save_state(state)
        return True

    def boop_check(self) -> str:
        """
        Run during BOOP to check scheduled tasks.

        Returns:
            Human-readable status report
        """
        due_tasks = self.get_due_tasks()

        if not due_tasks:
            return "✅ All scheduled tasks up to date."

        lines = [f"📋 {len(due_tasks)} scheduled task(s) due:\n"]

        for task in due_tasks:
            freq_emoji = "📅" if task["frequency"] == "daily" else "📆"
            lines.append(f"  {freq_emoji} {task['id']}: {task['description']}")
            lines.append(f"      Reason: {task['reason']}")

        lines.append("\nRun these tasks, then call complete_task() for each.")

        return "\n".join(lines)

    def increment_counter(self, task_id: str, counter_name: str = "count_today") -> int:
        """
        Increment a counter for a task (e.g., daily follow count).

        Args:
            task_id: The task to update
            counter_name: Name of the counter field

        Returns:
            New counter value
        """
        state = self._load_state()

        if task_id not in state["tasks"]:
            return -1

        today = date.today().isoformat()

        # Reset counter if it's a new day
        if state["tasks"][task_id].get("last_run") != today:
            state["tasks"][task_id][counter_name] = 0

        current = state["tasks"][task_id].get(counter_name, 0)
        state["tasks"][task_id][counter_name] = current + 1
        state["tasks"][task_id]["last_run"] = today

        self._save_state(state)
        return current + 1


def boop_scheduled_check() -> str:
    """
    Convenience function for BOOP integration.

    Usage in BOOP:
        from tools.scheduled_tasks import boop_scheduled_check
        print(boop_scheduled_check())
    """
    tasks = ScheduledTasks()
    return tasks.boop_check()


# CLI interface
if __name__ == "__main__":
    import sys

    tasks = ScheduledTasks()

    if len(sys.argv) < 2:
        print("Usage: python scheduled_tasks.py [check|list|complete <task_id>|register <id> <freq> <desc>]")
        sys.exit(1)

    command = sys.argv[1]

    if command == "check":
        print(tasks.boop_check())

    elif command == "list":
        all_tasks = tasks.list_all_tasks()
        print(f"\n📋 Registered Tasks ({len(all_tasks)}):\n")
        for task_id, task in all_tasks.items():
            freq = "Daily" if task["frequency"] == "daily" else "Weekly"
            status = task.get("status", "unknown")
            last = task.get("last_run", "never")
            print(f"  [{freq}] {task_id}")
            print(f"         {task.get('description', 'No description')}")
            print(f"         Status: {status} | Last run: {last}")
            print()

    elif command == "complete" and len(sys.argv) >= 3:
        task_id = sys.argv[2]
        notes = sys.argv[3] if len(sys.argv) > 3 else None
        if tasks.complete_task(task_id, notes):
            print(f"✅ Marked {task_id} as complete")
        else:
            print(f"❌ Task {task_id} not found")

    elif command == "register" and len(sys.argv) >= 5:
        task_id = sys.argv[2]
        frequency = sys.argv[3]
        description = " ".join(sys.argv[4:])
        if tasks.register_task(task_id, frequency, description):
            print(f"✅ Registered {task_id} ({frequency})")
        else:
            print(f"❌ Task {task_id} already exists")

    else:
        print("Unknown command or missing arguments")
        sys.exit(1)
