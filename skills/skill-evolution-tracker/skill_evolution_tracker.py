#!/usr/bin/env python3
"""
Skill Evolution Tracker — Hermes D1 Pattern

Skills FORM from experience and IMPROVE during use. This tracker logs skill
invocations and surfaces improvement signals from real usage patterns.

Usage:
    python3 skill_evolution_tracker.py log <skill-name> [--context <text>] [--outcome <pass|fail>]
    python3 skill_evolution_tracker.py analyze [--log <path>]
    python3 skill_evolution_tracker.py signals [--log <path>]

Log format (JSONL):
    {"skill": "tdd", "ts": "2026-05-03T...", "context": "...", "outcome": "pass|fail", "civ": "hengshi"}

Analyze output:
    Per-skill invocation count, pass/fail rate, co-use patterns, improvement signals.
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
class InvocationRecord:
    skill: str
    ts: str
    context: str = ""
    outcome: str = "pass"
    civ: str = "unknown"

    def to_dict(self) -> dict:
        return {
            "skill": self.skill,
            "ts": self.ts,
            "context": self.context,
            "outcome": self.outcome,
            "civ": self.civ,
        }


@dataclass
class SkillStats:
    skill: str
    invocations: int
    pass_count: int
    fail_count: int
    contexts: list[str]
    co_used_with: list[str]
    improvement_signals: list[str]

    @property
    def pass_rate(self) -> float:
        return self.pass_count / self.invocations if self.invocations else 0.0


# ───────────────────────────────────────────────────────────────────
# Logging
# ───────────────────────────────────────────────────────────────────

def log_invocation(
    skill: str,
    log_path: Path,
    context: str = "",
    outcome: str = "pass",
    civ: str = "unknown",
) -> None:
    """Append an invocation record to the usage log."""
    record = InvocationRecord(
        skill=skill,
        ts=datetime.now(timezone.utc).isoformat(),
        context=context,
        outcome=outcome,
        civ=civ,
    )
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with open(log_path, "a") as f:
        f.write(json.dumps(record.to_dict()) + "\n")


# ───────────────────────────────────────────────────────────────────
# Analysis
# ───────────────────────────────────────────────────────────────────

def load_records(log_path: Path) -> list[InvocationRecord]:
    """Load all invocation records from JSONL log."""
    if not log_path.exists():
        return []
    records = []
    with open(log_path) as f:
        for line in f:
            try:
                d = json.loads(line)
                records.append(InvocationRecord(
                    skill=d["skill"],
                    ts=d["ts"],
                    context=d.get("context", ""),
                    outcome=d.get("outcome", "pass"),
                    civ=d.get("civ", "unknown"),
                ))
            except (json.JSONDecodeError, KeyError):
                continue
    return records


def analyze_skill(records: list[InvocationRecord], skill: str) -> SkillStats:
    """Compute stats for a single skill."""
    skill_records = [r for r in records if r.skill == skill]
    invocations = len(skill_records)
    pass_count = sum(1 for r in skill_records if r.outcome == "pass")
    fail_count = sum(1 for r in skill_records if r.outcome == "fail")
    contexts = [r.context for r in skill_records if r.context]

    # Co-use detection: skills invoked within same session (by same civ, within 1hr window)
    co_used = defaultdict(int)
    for r in skill_records:
        # Find other skills invoked within 1hr of this one, by same civ
        for other in records:
            if other.skill == skill or other.civ != r.civ:
                continue
            if other.ts and r.ts:
                # Simple proximity: same civ, within same day
                co_used[other.skill] += 1

    top_co = sorted(co_used.items(), key=lambda x: -x[1])[:5]

    # Improvement signals
    signals = []
    if invocations == 0:
        signals.append("never_invocated")
    elif fail_count / invocations > 0.3:
        signals.append(f"high_fail_rate ({fail_count}/{invocations})")
    if pass_count > 10 and fail_count == 0:
        signals.append("stable_high_use")
    if invocations > 20 and pass_rate < 0.7:
        signals.append("needs_review")

    return SkillStats(
        skill=skill,
        invocations=invocations,
        pass_count=pass_count,
        fail_count=fail_count,
        contexts=list(dict.fromkeys(contexts))[:10],
        co_used_with=[name for name, _ in top_co],
        improvement_signals=signals,
    )


def analyze_all(records: list[InvocationRecord]) -> list[SkillStats]:
    """Analyze all skills in the log."""
    skills = sorted(set(r.skill for r in records))
    return [analyze_skill(records, s) for s in skills]


def print_analysis(stats: list[SkillStats]):
    """Print human-readable analysis."""
    print("")
    print("=" * 60)
    print("SKILL EVOLUTION TRACKER — D1 Analysis")
    print("=" * 60)
    print(f"  Total skills tracked: {len(stats)}")
    total_invocations = sum(s.invocations for s in stats)
    print(f"  Total invocations:    {total_invocations}")
    print(f"  Total signals:         {sum(len(s.improvement_signals) for s in stats)}")
    print("")

    signals_only = [s for s in stats if s.improvement_signals]
    if signals_only:
        print("IMPROVEMENT SIGNALS:")
        for s in signals_only:
            print(f"  [{s.skill}] {'; '.join(s.improvement_signals)}")
        print("")

    print("TOP SKILLS BY USAGE:")
    top = sorted(stats, key=lambda s: -s.invocations)[:10]
    for s in top:
        rate = f"{s.pass_rate:.0%}" if s.invocations else "N/A"
        print(f"  {s.skill}: {s.invocations} inv ({rate} pass)")

    print("")
    print("CO-USE PATTERNS:")
    co_use_examples = [s for s in stats if s.co_used_with]
    for s in co_use_examples[:5]:
        print(f"  {s.skill} ↔ {', '.join(s.co_used_with[:3])}")


# ───────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────

def cmd_log(args):
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skills-usage-log.jsonl")
    log_invocation(
        skill=args.skill,
        log_path=log_path,
        context=args.context or "",
        outcome=args.outcome or "pass",
        civ=args.civ or "hengshi",
    )
    print(f"Logged: {args.skill} ({args.outcome or 'pass'}) → {log_path}")


def cmd_analyze(args):
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skills-usage-log.jsonl")
    records = load_records(log_path)
    if not records:
        print(f"No records in {log_path} — run 'log' first or use --log to specify a path.")
        sys.exit(0)
    stats = analyze_all(records)
    print_analysis(stats)
    return stats


def cmd_signals(args):
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skills-usage-log.jsonl")
    records = load_records(log_path)
    if not records:
        print(f"No records in {log_path}")
        sys.exit(0)
    stats = analyze_all(records)
    signals = [s for s in stats if s.improvement_signals]
    print("")
    print("=" * 60)
    print("IMPROVEMENT SIGNALS (D1)")
    print("=" * 60)
    if not signals:
        print("  No improvement signals detected yet.")
        print("  (Need ≥1 invocation per skill to generate signals.)")
    else:
        for s in signals:
            print(f"  [{s.skill}] {'; '.join(s.improvement_signals)}")
    print("")
    print(f"Skills with signals: {len(signals)}/{len(stats)}")


def main():
    parser = argparse.ArgumentParser(description="Skill Evolution Tracker — Hermes D1")
    sub = parser.add_subparsers()

    p_log = sub.add_parser("log", help="Log a skill invocation")
    p_log.add_argument("skill", help="Skill name")
    p_log.add_argument("--context", default="", help="Invocation context")
    p_log.add_argument("--outcome", choices=["pass", "fail"], default="pass", help="Outcome")
    p_log.add_argument("--civ", default="hengshi", help="Civ name")
    p_log.add_argument("--log", default="memories/skills-usage-log.jsonl", help="Log path")
    p_log.set_defaults(func=cmd_log)

    p_analyze = sub.add_parser("analyze", help="Analyze usage patterns")
    p_analyze.add_argument("--log", default="memories/skills-usage-log.jsonl", help="Log path")
    p_analyze.set_defaults(func=cmd_analyze)

    p_signals = sub.add_parser("signals", help="Show improvement signals only")
    p_signals.add_argument("--log", default="memories/skills-usage-log.jsonl", help="Log path")
    p_signals.set_defaults(func=cmd_signals)

    args = parser.parse_args()
    if hasattr(args, 'func'):
        args.func(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
