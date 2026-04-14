#!/usr/bin/env python3
"""
Post Tool Use Hook - Context Monitoring for Fork Civilizations

Fires after every tool use. Checks context usage and warns at thresholds.
This prevents context-limit-death mid-evolution.

Simple and lean — no hash chain, no ledger, no telemetry.
Just context monitoring with actionable warnings.
"""

import json
import sys
from pathlib import Path


def check_context_threshold():
    """
    Check context usage and warn at 80% and 90% thresholds.

    Reads from /tmp/claude_context_used.txt (written by Claude Code internals).
    Warns once per threshold to avoid noise.
    """
    context_file = Path("/tmp/claude_context_used.txt")
    warning_file = Path("/tmp/fork_context_warning.txt")

    if not context_file.exists():
        return

    try:
        percent = float(context_file.read_text().strip())
    except (ValueError, IOError):
        return

    # Read previous warning level
    warned_level = 0
    if warning_file.exists():
        try:
            data = json.loads(warning_file.read_text())
            warned_level = data.get("level", 0)
        except (json.JSONDecodeError, IOError):
            pass

    if percent >= 90 and warned_level < 90:
        print("")
        print("=" * 70)
        print("CONTEXT CRITICAL: %.0f%% USED — STOP IMMEDIATELY" % percent)
        print("=" * 70)
        print("1. STOP all new work right now.")
        print("2. Write memories/identity/.evolution-progress.md listing:")
        print("   - Which teams are COMPLETE (with evidence files)")
        print("   - Which teams are REMAINING")
        print("   - Current state of any in-progress team")
        print("3. Run /compact to compress context.")
        print("4. After /compact, resume from where you stopped.")
        print("DO NOT launch new agents. DO NOT read large files.")
        print("DO NOT let context hit 100% — that kills this session.")
        print("=" * 70)
        try:
            warning_file.write_text(json.dumps({"level": 90}))
        except IOError:
            pass

    elif percent >= 80 and warned_level < 80:
        print("")
        print("=" * 60)
        print("CONTEXT WARNING: %.0f%% USED" % percent)
        print("=" * 60)
        print("Finish your current task, then run /compact.")
        print("Do NOT launch new agents or read large files.")
        print("Save progress to memories/identity/.evolution-progress.md")
        print("before running /compact so you know where to resume.")
        print("=" * 60)
        try:
            warning_file.write_text(json.dumps({"level": 80}))
        except IOError:
            pass


def main():
    # Read hook input from stdin (required by Claude Code hook protocol)
    try:
        sys.stdin.read()  # consume input, we don't need tool details
    except Exception:
        pass

    check_context_threshold()
    return 0


if __name__ == "__main__":
    sys.exit(main())
