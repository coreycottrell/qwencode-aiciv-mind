#!/usr/bin/env python3
"""
Session Summarizer — Hermes Pattern

Captures session state for continuity. Write a session summary at end of work
that the next session can read to quickly orient.

Usage:
    python3 session_summarizer.py snapshot [--output <path>]
    python3 session_summarizer.py analyze [--log <path>]
"""

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path


@dataclass
class SessionSnapshot:
    session_id: str
    civ: str
    ts: str
    active_files: list[str]
    uncommitted_changes: bool
    git_status: str
    open_tasks: list[str]
    key_decisions: list[str]
    next_steps: list[str]
    tools_used: list[str]

    def to_dict(self) -> dict:
        return asdict(self)


def get_git_status() -> str:
    try:
        r = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True, timeout=5)
        if r.returncode == 0:
            lines = [l for l in r.stdout.strip().splitlines() if l]
            return "\n".join(lines) if lines else "clean"
        return "unknown"
    except Exception:
        return "unavailable"


def get_active_files() -> list[str]:
    """Get list of recently modified files."""
    try:
        r = subprocess.run(
            ["git", "status", "--porcelain", "-uall"],
            capture_output=True, text=True, timeout=5, cwd=Path.cwd()
        )
        if r.returncode == 0:
            files = []
            for line in r.stdout.strip().splitlines():
                if line and line[1] == " ":
                    f = line[2:].strip()
                    if f and not f.startswith("."):
                        files.append(f)
            return sorted(files)[:20]
        return []
    except Exception:
        return []


def cmd_snapshot(args) -> SessionSnapshot:
    output = Path(args.output) if args.output else Path("memories/sessions/summaries/snapshot.jsonl")
    output.parent.mkdir(parents=True, exist_ok=True)

    git_status = get_git_status()
    active_files = get_active_files()

    snapshot = SessionSnapshot(
        session_id=args.session_id or f"session-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}",
        civ=args.civ or "hengshi",
        ts=datetime.now(timezone.utc).isoformat(),
        active_files=active_files,
        uncommitted_changes=git_status not in ("", "clean", "unknown"),
        git_status=git_status,
        open_tasks=args.tasks or [],
        key_decisions=args.decisions or [],
        next_steps=args.next_steps or [],
        tools_used=args.tools or [],
    )

    with open(output, "a") as f:
        f.write(json.dumps(snapshot.to_dict()) + "\n")

    print(f"Session snapshot saved: {snapshot.session_id}")
    print(f"  Active files: {len(active_files)}")
    print(f"  Uncommitted: {snapshot.uncommitted_changes}")
    print(f"  Output: {output}")

    return snapshot


def cmd_analyze(args):
    log_path = Path(args.log) if args.log else Path("memories/sessions/summaries/snapshot.jsonl")
    if not log_path.exists():
        print(f"No log found: {log_path}")
        sys.exit(1)

    snapshots = []
    with open(log_path) as f:
        for line in f:
            try:
                snapshots.append(json.loads(line))
            except json.JSONDecodeError:
                continue

    if not snapshots:
        print(f"No snapshots in {log_path}")
        sys.exit(0)

    print("")
    print("=" * 60)
    print("SESSION SUMMARIZER — Recent Sessions")
    print("=" * 60)
    for s in sorted(snapshots, key=lambda x: x["ts"], reverse=True)[:10]:
        uncommitted = "✗" if s.get("uncommitted_changes") else "✓"
        files = len(s.get("active_files", []))
        print(f"  {s['session_id']} [{s['civ']}] {s['ts'][:10]} — {files} files, uncommitted={uncommitted}")

    print(f"\nTotal snapshots: {len(snapshots)}")


def main():
    parser = argparse.ArgumentParser(description="Session Summarizer — Hermes")
    sub = parser.add_subparsers()

    p_snap = sub.add_parser("snapshot", help="Capture session snapshot")
    p_snap.add_argument("--session-id", default=None, help="Session ID")
    p_snap.add_argument("--civ", default="hengshi", help="Civ name")
    p_snap.add_argument("--output", default=None, help="Output path")
    p_snap.add_argument("--tasks", nargs="*", default=[], help="Open tasks")
    p_snap.add_argument("--decisions", nargs="*", default=[], help="Key decisions")
    p_snap.add_argument("--next-steps", nargs="*", default=[], help="Next steps")
    p_snap.add_argument("--tools", nargs="*", default=[], help="Tools used this session")
    p_snap.set_defaults(func=cmd_snapshot)

    p_analyze = sub.add_parser("analyze", help="Analyze past sessions")
    p_analyze.add_argument("--log", default=None, help="Log path")
    p_analyze.set_defaults(func=cmd_analyze)

    args = parser.parse_args()
    if hasattr(args, "func"):
        args.func(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
