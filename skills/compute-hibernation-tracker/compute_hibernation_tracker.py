#!/usr/bin/env python3
"""
Compute Hibernation Tracker — Hermes D4 Pattern

Compute uses serverless hibernation patterns. Pay only when active.
Track when a civ session is active (tool calls happening) vs idle (waiting),
detect hibernation candidates, log compute sessions.

Usage:
    python3 compute_hibernation_tracker.py ping [--active]
    python3 compute_hibernation_tracker.py session_start
    python3 compute_hibernation_tracker.py session_end
    python3 compute_hibernation_tracker.py analyze [--log <path>]
    python3 compute_hibernation_tracker.py hibernate_candidates [--log <path>]
"""

import argparse
import json
import sys
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class ComputeSession:
    session_id: str
    civ: str
    start_ts: str
    end_ts: Optional[str] = None
    tool_calls: int = 0
    idle_seconds: int = 0
    active_seconds: int = 0
    status: str = "active"  # active | idle | hibernated | ended

    def to_dict(self) -> dict:
        return {
            "session_id": self.session_id,
            "civ": self.civ,
            "start_ts": self.start_ts,
            "end_ts": self.end_ts,
            "tool_calls": self.tool_calls,
            "idle_seconds": self.idle_seconds,
            "active_seconds": self.active_seconds,
            "status": self.status,
        }


@dataclass
class SessionStats:
    session_id: str
    civ: str
    duration_seconds: int
    tool_calls: int
    active_seconds: int
    idle_seconds: int
    pass_rate: float
    hibernation_candidate: bool
    reasons: list[str]

    @property
    def active_ratio(self) -> float:
        total = self.active_seconds + self.idle_seconds
        return self.active_seconds / total if total > 0 else 0.0


# ───────────────────────────────────────────────────────────────────
# State
# ───────────────────────────────────────────────────────────────────

STATE_FILE = Path("memories/compute-hibernation-state.json")

def load_state() -> dict:
    if STATE_FILE.exists():
        return json.loads(STATE_FILE.read_text())
    return {"current_session": None, "last_ping": None}

def save_state(state: dict) -> None:
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(json.dumps(state, indent=2))


# ───────────────────────────────────────────────────────────────────
# Session Management
# ───────────────────────────────────────────────────────────────────

def session_start(civ: str, session_id: Optional[str] = None) -> str:
    """Mark the start of a compute session."""
    state = load_state()
    if state["current_session"]:
        # End previous session first
        session_end(civ, state["current_session"]["session_id"])

    sid = session_id or f"{civ}-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}"
    now = datetime.now(timezone.utc).isoformat()
    state["current_session"] = {
        "session_id": sid,
        "civ": civ,
        "start_ts": now,
        "tool_calls": 0,
        "idle_seconds": 0,
        "active_seconds": 0,
        "status": "active",
    }
    state["last_ping"] = now
    save_state(state)
    print(f"Session started: {sid} ({civ}) at {now}")
    return sid


def session_end(civ: str, session_id: Optional[str] = None) -> None:
    """Mark the end of a compute session, log to JSONL."""
    state = load_state()
    cs = state.get("current_session")
    if not cs or (session_id and cs["session_id"] != session_id):
        print(f"No active session found for {civ}")
        return

    now = datetime.now(timezone.utc).isoformat()
    cs["end_ts"] = now
    cs["status"] = "ended"

    log_path = Path(f"memories/compute-sessions-{cs['civ']}.jsonl")
    with open(log_path, "a") as f:
        f.write(json.dumps(cs) + "\n")

    print(f"Session ended: {cs['session_id']} — {cs['tool_calls']} tool_calls, {cs['active_seconds']}s active, {cs['idle_seconds']}s idle")
    state["current_session"] = None
    save_state(state)


def ping(active: bool = False, civ: str = "hengshi") -> None:
    """Mark a heartbeat ping — indicates the session is still alive."""
    state = load_state()
    now = datetime.now(timezone.utc)
    last_ping = state.get("last_ping")
    state["last_ping"] = now.isoformat()

    if state["current_session"]:
        cs = state["current_session"]
        if last_ping:
            last_dt = datetime.fromisoformat(last_ping)
            delta = (now - last_dt).total_seconds()
            if active:
                cs["active_seconds"] = cs.get("active_seconds", 0) + int(delta)
            else:
                cs["idle_seconds"] = cs.get("idle_seconds", 0) + int(delta)
        if active:
            cs["tool_calls"] = cs.get("tool_calls", 0) + 1
        state["current_session"] = cs

    save_state(state)


# ───────────────────────────────────────────────────────────────────
# Analysis
# ───────────────────────────────────────────────────────────────────

def load_sessions(log_path: Path) -> list[ComputeSession]:
    if not log_path.exists():
        return []
    sessions = []
    with open(log_path) as f:
        for line in f:
            try:
                d = json.loads(line)
                sessions.append(ComputeSession(
                    session_id=d["session_id"],
                    civ=d.get("civ", "unknown"),
                    start_ts=d.get("start_ts", ""),
                    end_ts=d.get("end_ts"),
                    tool_calls=d.get("tool_calls", 0),
                    idle_seconds=d.get("idle_seconds", 0),
                    active_seconds=d.get("active_seconds", 0),
                    status=d.get("status", "ended"),
                ))
            except (json.JSONDecodeError, KeyError):
                continue
    return sessions


def analyze_sessions(sessions: list[ComputeSession]) -> list[SessionStats]:
    stats = []
    for s in sessions:
        start = datetime.fromisoformat(s.start_ts)
        end = datetime.fromisoformat(s.end_ts) if s.end_ts else datetime.now(timezone.utc)
        duration = int((end - start).total_seconds())
        reasons = []
        hibernation_candidate = False

        if s.active_seconds == 0 and s.tool_calls == 0:
            reasons.append("pure_idle_session")
            hibernation_candidate = True
        elif s.idle_seconds > s.active_seconds * 2:
            reasons.append("idle_gt_active_2x")
            hibernation_candidate = True
        elif duration > 3600 and s.tool_calls < 5:
            reasons.append("long_session_low_utilization")
            hibernation_candidate = True
        elif s.tool_calls == 0:
            reasons.append("no_tool_calls")

        if s.active_seconds > 0:
            active_ratio = s.active_seconds / (s.active_seconds + s.idle_seconds)
            if active_ratio < 0.1:
                reasons.append("active_ratio_lt_10%")
                hibernation_candidate = True

        stats.append(SessionStats(
            session_id=s.session_id,
            civ=s.civ,
            duration_seconds=duration,
            tool_calls=s.tool_calls,
            active_seconds=s.active_seconds,
            idle_seconds=s.idle_seconds,
            pass_rate=s.tool_calls / duration * 60 if duration > 0 else 0,  # tool calls per minute as "pass rate" proxy
            hibernation_candidate=hibernation_candidate,
            reasons=reasons,
        ))
    return stats


def print_analysis(stats: list[SessionStats]):
    total = len(stats)
    hibernation = sum(1 for s in stats if s.hibernation_candidate)
    total_active = sum(s.active_seconds for s in stats)
    total_idle = sum(s.idle_seconds for s in stats)
    total_tool_calls = sum(s.tool_calls for s in stats)

    print("")
    print("=" * 60)
    print("COMPUTE HIBERNATION TRACKER — D4 Analysis")
    print("=" * 60)
    print(f"  Total sessions:      {total}")
    hibernation_pct = f"{hibernation/total*100:.0f}%" if total > 0 else "0%"
    print(f"  Hibernation cands:  {hibernation} ({hibernation_pct})")
    print(f"  Total active time:   {total_active}s ({total_active/3600:.1f}h)")
    print(f"  Total idle time:     {total_idle}s ({total_idle/3600:.1f}h)")
    print(f"  Total tool calls:    {total_tool_calls}")
    print("")

    if hibernation > 0:
        print("HIBERNATION CANDIDATES:")
        for s in stats:
            if s.hibernation_candidate:
                print(f"  {s.session_id}: {s.duration_seconds}s, {s.tool_calls} calls, {s.active_seconds}s active, {s.idle_seconds}s idle")
                for r in s.reasons:
                    print(f"    → {r}")
        print("")

    print("RECENT SESSIONS:")
    for s in sorted(stats, key=lambda x: x.session_id, reverse=True)[:5]:
        ratio = f"{s.active_ratio:.0%}" if s.duration_seconds > 0 else "N/A"
        print(f"  {s.session_id}: {s.duration_seconds}s, {s.tool_calls} calls, {ratio} active")


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Compute Hibernation Tracker — Hermes D4")
    sub = parser.add_subparsers()

    p_ping = sub.add_parser("ping", help="Send heartbeat ping")
    p_ping.add_argument("--active", action="store_true", help="Mark as active (tool call in this interval)")
    p_ping.add_argument("--civ", default="hengshi", help="Civ name")
    p_ping.set_defaults(func=cmd_ping)

    p_start = sub.add_parser("session_start", help="Mark session start")
    p_start.add_argument("--civ", default="hengshi", help="Civ name")
    p_start.add_argument("--session-id", default=None, help="Optional session ID")
    p_start.set_defaults(func=cmd_session_start)

    p_end = sub.add_parser("session_end", help="Mark session end and log")
    p_end.add_argument("--civ", default="hengshi", help="Civ name")
    p_end.add_argument("--session-id", default=None, help="Optional session ID")
    p_end.set_defaults(func=cmd_session_end)

    p_analyze = sub.add_parser("analyze", help="Analyze past sessions")
    p_analyze.add_argument("--civ", default="hengshi", help="Civ name")
    p_analyze.add_argument("--log", default=None, help="Log path")
    p_analyze.set_defaults(func=cmd_analyze)

    p_hibernate = sub.add_parser("hibernate_candidates", help="Show hibernation candidates only")
    p_hibernate.add_argument("--civ", default="hengshi", help="Civ name")
    p_hibernate.add_argument("--log", default=None, help="Log path")
    p_hibernate.set_defaults(func=cmd_hibernate)

    args = parser.parse_args()

    if hasattr(args, 'func'):
        args.func(args)
    else:
        parser.print_help()


def cmd_ping(args):
    ping(active=args.active, civ=args.civ)

def cmd_session_start(args):
    session_start(civ=args.civ, session_id=args.session_id)

def cmd_session_end(args):
    session_end(civ=args.civ, session_id=args.session_id)

def cmd_analyze(args):
    log_path = Path(args.log) if args.log else Path(f"memories/compute-sessions-{args.civ}.jsonl")
    sessions = load_sessions(log_path)
    if not sessions:
        print(f"No sessions in {log_path}")
        return
    stats = analyze_sessions(sessions)
    print_analysis(stats)

def cmd_hibernate(args):
    log_path = Path(args.log) if args.log else Path(f"memories/compute-sessions-{args.civ}.jsonl")
    sessions = load_sessions(log_path)
    if not sessions:
        print(f"No sessions in {log_path}")
        return
    stats = analyze_sessions(sessions)
    candidates = [s for s in stats if s.hibernation_candidate]
    print("")
    print(f"HIBERNATION CANDIDATES: {len(candidates)}/{len(stats)}")
    for s in candidates:
        print(f"  {s.session_id}: {s.reasons}")
    if not candidates:
        print("  None — all sessions had good active ratios.")


if __name__ == "__main__":
    # Wire up subparsers
    main_parser = argparse.ArgumentParser(description="Compute Hibernation Tracker — Hermes D4")
    sub = main_parser.add_subparsers()

    p_ping = sub.add_parser("ping", help="Send heartbeat ping")
    p_ping.add_argument("--active", action="store_true", help="Mark as active (tool call in this interval)")
    p_ping.add_argument("--civ", default="hengshi", help="Civ name")

    p_start = sub.add_parser("session_start", help="Mark session start")
    p_start.add_argument("--civ", default="hengshi", help="Civ name")
    p_start.add_argument("--session-id", help="Optional session ID")

    p_end = sub.add_parser("session_end", help="Mark session end and log")
    p_end.add_argument("--civ", default="hengshi", help="Civ name")
    p_end.add_argument("--session-id", help="Optional session ID")

    p_analyze = sub.add_parser("analyze", help="Analyze past sessions")
    p_analyze.add_argument("--civ", default="hengshi", help="Civ name")
    p_analyze.add_argument("--log", default=None, help="Log path")

    p_hibernate = sub.add_parser("hibernate_candidates", help="Show hibernation candidates only")
    p_hibernate.add_argument("--civ", default="hengshi", help="Civ name")
    p_hibernate.add_argument("--log", default=None, help="Log path")

    parsed = main_parser.parse_args()

    if parsed.__class__.__name__ == 'Namespace':
        # handle subcommands manually
        import sys
        if len(sys.argv) < 2:
            main_parser.print_help()
            sys.exit(0)
        cmd = sys.argv[1]

        if cmd == "ping":
            import argparse
            p = argparse.ArgumentParser()
            p.add_argument("--active", action="store_true")
            p.add_argument("--civ", default="hengshi")
            a = p.parse_args(sys.argv[2:])
            cmd_ping(a)
        elif cmd == "session_start":
            import argparse
            p = argparse.ArgumentParser()
            p.add_argument("--civ", default="hengshi")
            p.add_argument("--session-id", default=None)
            a = p.parse_args(sys.argv[2:])
            cmd_session_start(a)
        elif cmd == "session_end":
            import argparse
            p = argparse.ArgumentParser()
            p.add_argument("--civ", default="hengshi")
            p.add_argument("--session-id", default=None)
            a = p.parse_args(sys.argv[2:])
            cmd_session_end(a)
        elif cmd == "analyze":
            import argparse
            p = argparse.ArgumentParser()
            p.add_argument("--civ", default="hengshi")
            p.add_argument("--log", default=None)
            a = p.parse_args(sys.argv[2:])
            if a.log:
                a.log = a.log.format(civ=a.civ)
            else:
                a.log = f"memories/compute-sessions-{a.civ}.jsonl"
            cmd_analyze(a)
        elif cmd == "hibernate_candidates":
            import argparse
            p = argparse.ArgumentParser()
            p.add_argument("--civ", default="hengshi")
            p.add_argument("--log", default=None)
            a = p.parse_args(sys.argv[2:])
            if a.log:
                a.log = a.log.format(civ=a.civ)
            else:
                a.log = f"memories/compute-sessions-{a.civ}.jsonl"
            cmd_hibernate(a)
        else:
            main_parser.print_help()
