#!/usr/bin/env python3
"""
Skill Injection Tracker - Tracks when skills are loaded into agent contexts.

Corey Requirement (2025-12-26):
"I want to be able to track the number of times, and the types, SKILLs that
get injected into your or our agents context."

This module provides:
1. SkillTracker class for logging skill injections
2. Query utilities for analyzing skill usage patterns
3. Integration with existing hook infrastructure

Schema (JSONL format):
{
    "timestamp": "2025-12-26T10:30:00Z",
    "skill": "memory-first-protocol",
    "skill_path": ".claude/skills/custom/memory-first-protocol.md",
    "agent": "coder",
    "session": "session-20251226-103000",
    "context": "manual_read",  # or "hook_injection", "manifest_reference", "compact_recovery"
    "success": true
}

Usage:
    # Track a skill injection
    from tools.skill_tracker import SkillTracker
    tracker = SkillTracker()
    tracker.log_injection("memory-first-protocol", "coder", "manual_read")

    # Query skill usage
    tracker.get_stats()
    tracker.get_agent_skills("coder")
    tracker.get_skill_agents("memory-first-protocol")

CLI:
    python3 tools/skill_tracker.py log <skill> <agent> [--context manual_read]
    python3 tools/skill_tracker.py stats
    python3 tools/skill_tracker.py agent <agent_id>
    python3 tools/skill_tracker.py skill <skill_name>
    python3 tools/skill_tracker.py recent [--limit 20]
"""

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Any
from collections import Counter, defaultdict

# Project paths
PROJECT_DIR = os.environ.get("CLAUDE_PROJECT_DIR", "/home/corey/projects/AI-CIV/ACG")
TRACKING_FILE = Path(PROJECT_DIR) / "memories" / "system" / "skill-injections.jsonl"
SKILLS_DIR = Path(PROJECT_DIR) / ".claude" / "skills"

# Skill contexts - how the skill was injected
INJECTION_CONTEXTS = {
    "manual_read": "Agent manually read skill file",
    "hook_injection": "Skill loaded via hook (compact recovery, session start)",
    "manifest_reference": "Skill referenced in agent manifest",
    "compact_recovery": "Skill reloaded after context compaction",
    "delegation_context": "Skill included in delegation to agent",
    "primary_guidance": "Primary AI provided skill as context",
    "unknown": "Context not specified"
}


class SkillTracker:
    """
    Tracks skill injections into agent contexts.

    Thread-safe append-only logging with query utilities.
    """

    def __init__(self, tracking_file: Optional[Path] = None):
        self.tracking_file = tracking_file or TRACKING_FILE
        self._ensure_file_exists()

    def _ensure_file_exists(self):
        """Ensure tracking file and parent directory exist."""
        self.tracking_file.parent.mkdir(parents=True, exist_ok=True)
        if not self.tracking_file.exists():
            self.tracking_file.touch()

    def _get_session_id(self) -> str:
        """Get current session ID from ledger if available."""
        ledger_path = Path(PROJECT_DIR) / "memories" / "sessions" / "current-session.jsonl"
        if ledger_path.exists():
            try:
                with open(ledger_path, 'r') as f:
                    first_line = f.readline().strip()
                    if first_line:
                        entry = json.loads(first_line)
                        return entry.get("session_id", "unknown")
            except (json.JSONDecodeError, IOError):
                pass
        return f"session-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}"

    def _normalize_skill_name(self, skill: str) -> str:
        """
        Normalize skill name to consistent format.

        Examples:
            ".claude/skills/gmail-mastery.md" -> "gmail-mastery"
            "gmail-mastery.md" -> "gmail-mastery"
            "memory-first-protocol" -> "memory-first-protocol"
        """
        # Remove path components
        if "/" in skill:
            skill = Path(skill).stem
        # Remove .md extension
        if skill.endswith(".md"):
            skill = skill[:-3]
        return skill

    def _find_skill_path(self, skill: str) -> Optional[str]:
        """Find the full path to a skill file given its name."""
        skill_name = self._normalize_skill_name(skill)

        # Search in skills directory
        for skill_file in SKILLS_DIR.rglob("*.md"):
            if skill_file.stem == skill_name:
                return str(skill_file.relative_to(PROJECT_DIR))

        # Check SKILL.md files (subdirectory skills)
        for skill_file in SKILLS_DIR.rglob("SKILL.md"):
            parent_name = skill_file.parent.name
            if parent_name == skill_name or skill_name in parent_name:
                return str(skill_file.relative_to(PROJECT_DIR))

        return None

    def log_injection(
        self,
        skill: str,
        agent: str,
        context: str = "unknown",
        success: bool = True,
        metadata: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Log a skill injection event.

        Args:
            skill: Skill name or path (will be normalized)
            agent: Agent ID that received the skill
            context: How the skill was injected (see INJECTION_CONTEXTS)
            success: Whether injection was successful
            metadata: Additional context (optional)

        Returns:
            The logged entry
        """
        skill_name = self._normalize_skill_name(skill)
        skill_path = self._find_skill_path(skill)

        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "skill": skill_name,
            "skill_path": skill_path or f".claude/skills/{skill_name}.md",
            "agent": agent,
            "session": self._get_session_id(),
            "context": context,
            "success": success
        }

        if metadata:
            entry["metadata"] = metadata

        # Append to tracking file
        with open(self.tracking_file, 'a') as f:
            f.write(json.dumps(entry) + "\n")

        return entry

    def _read_all_entries(self) -> List[Dict[str, Any]]:
        """Read all entries from tracking file."""
        entries = []
        if self.tracking_file.exists():
            with open(self.tracking_file, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line:
                        try:
                            entries.append(json.loads(line))
                        except json.JSONDecodeError:
                            continue
        return entries

    def get_stats(self) -> Dict[str, Any]:
        """
        Get comprehensive skill injection statistics.

        Returns:
            Dictionary with usage counts, top skills, top agents, etc.
        """
        entries = self._read_all_entries()

        if not entries:
            return {
                "total_injections": 0,
                "unique_skills": 0,
                "unique_agents": 0,
                "skills_by_frequency": {},
                "agents_by_frequency": {},
                "contexts_by_frequency": {},
                "success_rate": 0.0
            }

        skill_counts = Counter(e["skill"] for e in entries)
        agent_counts = Counter(e["agent"] for e in entries)
        context_counts = Counter(e.get("context", "unknown") for e in entries)
        success_count = sum(1 for e in entries if e.get("success", True))

        return {
            "total_injections": len(entries),
            "unique_skills": len(skill_counts),
            "unique_agents": len(agent_counts),
            "skills_by_frequency": dict(skill_counts.most_common()),
            "agents_by_frequency": dict(agent_counts.most_common()),
            "contexts_by_frequency": dict(context_counts.most_common()),
            "success_rate": round(success_count / len(entries) * 100, 1) if entries else 0.0,
            "first_entry": entries[0]["timestamp"] if entries else None,
            "last_entry": entries[-1]["timestamp"] if entries else None
        }

    def get_agent_skills(self, agent: str) -> Dict[str, Any]:
        """
        Get all skills used by a specific agent.

        Args:
            agent: Agent ID

        Returns:
            Dictionary with agent's skill usage patterns
        """
        entries = [e for e in self._read_all_entries() if e["agent"] == agent]

        if not entries:
            return {
                "agent": agent,
                "total_injections": 0,
                "unique_skills": 0,
                "skills": []
            }

        skill_counts = Counter(e["skill"] for e in entries)
        context_counts = Counter(e.get("context", "unknown") for e in entries)

        # Get most recent use for each skill
        skill_last_used = {}
        for e in entries:
            skill = e["skill"]
            ts = e["timestamp"]
            if skill not in skill_last_used or ts > skill_last_used[skill]:
                skill_last_used[skill] = ts

        skills_detail = [
            {
                "skill": skill,
                "count": count,
                "last_used": skill_last_used.get(skill)
            }
            for skill, count in skill_counts.most_common()
        ]

        return {
            "agent": agent,
            "total_injections": len(entries),
            "unique_skills": len(skill_counts),
            "skills": skills_detail,
            "contexts": dict(context_counts),
            "first_injection": entries[0]["timestamp"],
            "last_injection": entries[-1]["timestamp"]
        }

    def get_skill_agents(self, skill: str) -> Dict[str, Any]:
        """
        Get all agents that have used a specific skill.

        Args:
            skill: Skill name

        Returns:
            Dictionary with skill's usage across agents
        """
        skill_name = self._normalize_skill_name(skill)
        entries = [e for e in self._read_all_entries() if e["skill"] == skill_name]

        if not entries:
            return {
                "skill": skill_name,
                "total_injections": 0,
                "unique_agents": 0,
                "agents": []
            }

        agent_counts = Counter(e["agent"] for e in entries)
        context_counts = Counter(e.get("context", "unknown") for e in entries)
        session_counts = Counter(e.get("session", "unknown") for e in entries)

        return {
            "skill": skill_name,
            "skill_path": entries[0].get("skill_path"),
            "total_injections": len(entries),
            "unique_agents": len(agent_counts),
            "unique_sessions": len(session_counts),
            "agents": dict(agent_counts.most_common()),
            "contexts": dict(context_counts),
            "first_injection": entries[0]["timestamp"],
            "last_injection": entries[-1]["timestamp"]
        }

    def get_recent(self, limit: int = 20) -> List[Dict[str, Any]]:
        """
        Get most recent skill injections.

        Args:
            limit: Number of entries to return

        Returns:
            List of recent injection entries (newest first)
        """
        entries = self._read_all_entries()
        # Sort by timestamp descending
        entries.sort(key=lambda x: x.get("timestamp", ""), reverse=True)
        return entries[:limit]

    def get_session_skills(self, session_id: Optional[str] = None) -> Dict[str, Any]:
        """
        Get skills used in a specific session.

        Args:
            session_id: Session ID (defaults to current session)

        Returns:
            Dictionary with session's skill usage
        """
        if session_id is None:
            session_id = self._get_session_id()

        entries = [e for e in self._read_all_entries() if e.get("session") == session_id]

        if not entries:
            return {
                "session": session_id,
                "total_injections": 0,
                "skills": [],
                "agents": []
            }

        skill_counts = Counter(e["skill"] for e in entries)
        agent_counts = Counter(e["agent"] for e in entries)

        return {
            "session": session_id,
            "total_injections": len(entries),
            "skills": dict(skill_counts.most_common()),
            "agents": dict(agent_counts.most_common()),
            "first_injection": entries[0]["timestamp"],
            "last_injection": entries[-1]["timestamp"]
        }

    def get_skill_health(self, stale_days: int = 7) -> Dict[str, Any]:
        """
        Get skill system health metrics for auditor integration.

        Provides a comprehensive health assessment suitable for inclusion
        in system audit reports.

        Args:
            stale_days: Number of days without use before a skill is considered stale

        Returns:
            Dictionary with health metrics:
            - total_injections: Total number of skill injections recorded
            - success_rate: Percentage of successful injections (0-100)
            - skill_diversity: Number of unique skills in use
            - agent_coverage: Number of unique agents receiving skills
            - anomalies: List of detected issues (failures, stale skills, unknown agents)
            - health_score: Overall health score (0-100)
            - recommendations: List of suggested improvements
            - period: Time range of data analyzed
        """
        entries = self._read_all_entries()
        now = datetime.now(timezone.utc)

        # Handle empty case
        if not entries:
            return {
                "total_injections": 0,
                "success_rate": 0.0,
                "skill_diversity": 0,
                "agent_coverage": 0,
                "anomalies": [{
                    "type": "no_data",
                    "severity": "warning",
                    "message": "No skill injection data recorded yet"
                }],
                "health_score": 0,
                "recommendations": ["Start tracking skill injections to build usage data"],
                "period": {"start": None, "end": None, "days": 0}
            }

        # Basic metrics
        total = len(entries)
        success_count = sum(1 for e in entries if e.get("success", True))
        success_rate = round(success_count / total * 100, 1)

        skill_counts = Counter(e["skill"] for e in entries)
        agent_counts = Counter(e["agent"] for e in entries)
        skill_diversity = len(skill_counts)
        agent_coverage = len(agent_counts)

        # Parse timestamps for time-based analysis
        def parse_ts(ts_str: str) -> datetime:
            """Parse ISO timestamp, handling various formats."""
            try:
                # Handle both +00:00 and Z formats
                ts_str = ts_str.replace('Z', '+00:00')
                if '.' in ts_str:
                    # Truncate microseconds if too long
                    base, frac = ts_str.split('.')
                    tz_part = ''
                    if '+' in frac:
                        frac, tz_part = frac.split('+')
                        tz_part = '+' + tz_part
                    elif '-' in frac and len(frac) > 6:
                        frac, tz_part = frac.rsplit('-', 1)
                        tz_part = '-' + tz_part
                    frac = frac[:6]
                    ts_str = f"{base}.{frac}{tz_part}"
                return datetime.fromisoformat(ts_str)
            except (ValueError, AttributeError):
                return now

        timestamps = [parse_ts(e["timestamp"]) for e in entries]
        first_ts = min(timestamps)
        last_ts = max(timestamps)
        period_days = (last_ts - first_ts).days + 1

        # Detect anomalies
        anomalies = []

        # 1. Failed injections
        failures = [e for e in entries if not e.get("success", True)]
        if failures:
            failure_skills = Counter(e["skill"] for e in failures)
            anomalies.append({
                "type": "injection_failures",
                "severity": "error" if len(failures) > 5 else "warning",
                "count": len(failures),
                "message": f"{len(failures)} failed injection(s) detected",
                "details": dict(failure_skills.most_common(5))
            })

        # 2. Unknown agent injections (from hooks without agent context)
        unknown_agent_count = agent_counts.get("unknown_from_hook", 0)
        if unknown_agent_count > 0:
            pct = round(unknown_agent_count / total * 100, 1)
            anomalies.append({
                "type": "unknown_agents",
                "severity": "info" if pct < 50 else "warning",
                "count": unknown_agent_count,
                "percentage": pct,
                "message": f"{unknown_agent_count} injection(s) ({pct}%) lack agent context"
            })

        # 3. Stale skills (not used recently)
        stale_threshold = now - __import__('datetime').timedelta(days=stale_days)
        skill_last_used = {}
        for e in entries:
            skill = e["skill"]
            ts = parse_ts(e["timestamp"])
            if skill not in skill_last_used or ts > skill_last_used[skill]:
                skill_last_used[skill] = ts

        stale_skills = [
            skill for skill, last_used in skill_last_used.items()
            if last_used < stale_threshold
        ]
        if stale_skills and period_days > stale_days:
            anomalies.append({
                "type": "stale_skills",
                "severity": "info",
                "count": len(stale_skills),
                "message": f"{len(stale_skills)} skill(s) not used in {stale_days}+ days",
                "skills": stale_skills[:10]  # Limit to top 10
            })

        # 4. Low diversity warning
        available_skills = list_available_skills()
        available_count = len(available_skills)
        if available_count > 0 and skill_diversity < available_count * 0.3:
            anomalies.append({
                "type": "low_diversity",
                "severity": "info",
                "message": f"Only {skill_diversity}/{available_count} available skills in use ({round(skill_diversity/available_count*100, 1)}%)"
            })

        # 5. Concentration warning (one skill dominates)
        if skill_diversity > 0:
            top_skill, top_count = skill_counts.most_common(1)[0]
            concentration = top_count / total * 100
            if concentration > 50 and skill_diversity > 3:
                anomalies.append({
                    "type": "high_concentration",
                    "severity": "info",
                    "message": f"'{top_skill}' accounts for {round(concentration, 1)}% of all injections"
                })

        # Calculate health score (0-100)
        # Components: success rate (40%), diversity (30%), coverage (20%), recency (10%)
        health_score = 0

        # Success rate component (40 points max)
        health_score += success_rate * 0.4

        # Diversity component (30 points max) - how many skills are being used
        if available_count > 0:
            diversity_ratio = min(skill_diversity / max(available_count * 0.5, 1), 1.0)
            health_score += diversity_ratio * 30

        # Coverage component (20 points max) - how many agents receiving skills
        # Assume 35 agents in civilization (from CLAUDE.md)
        coverage_ratio = min(agent_coverage / 10, 1.0)  # Target: 10+ agents
        health_score += coverage_ratio * 20

        # Recency component (10 points max) - recent activity
        days_since_last = (now - last_ts).days
        if days_since_last == 0:
            health_score += 10
        elif days_since_last < 3:
            health_score += 7
        elif days_since_last < 7:
            health_score += 4
        else:
            health_score += 0

        health_score = round(health_score, 1)

        # Generate recommendations
        recommendations = []
        if success_rate < 95:
            recommendations.append("Investigate failed skill injections")
        if unknown_agent_count > total * 0.3:
            recommendations.append("Improve agent context tracking in hook injections")
        if stale_skills:
            recommendations.append(f"Review underutilized skills: {', '.join(stale_skills[:3])}")
        if available_count > 0 and skill_diversity < available_count * 0.3:
            recommendations.append("Consider broader skill adoption across agents")
        if not recommendations:
            recommendations.append("Skill system operating normally")

        return {
            "total_injections": total,
            "success_rate": success_rate,
            "skill_diversity": skill_diversity,
            "agent_coverage": agent_coverage,
            "anomalies": anomalies,
            "health_score": health_score,
            "recommendations": recommendations,
            "period": {
                "start": first_ts.isoformat(),
                "end": last_ts.isoformat(),
                "days": period_days
            },
            "top_skills": dict(skill_counts.most_common(5)),
            "top_agents": dict(agent_counts.most_common(5))
        }


def list_available_skills() -> List[Dict[str, str]]:
    """List all available skills in the skills directory."""
    skills = []

    for skill_file in sorted(SKILLS_DIR.rglob("*.md")):
        rel_path = skill_file.relative_to(PROJECT_DIR)

        # Get title from first line
        try:
            with open(skill_file, 'r') as f:
                first_line = f.readline().strip()
                # Handle YAML frontmatter
                if first_line == "---":
                    # Skip frontmatter
                    for line in f:
                        if line.strip() == "---":
                            first_line = f.readline().strip()
                            break
                title = first_line.lstrip("#").strip()
        except IOError:
            title = skill_file.stem

        skills.append({
            "name": skill_file.stem,
            "path": str(rel_path),
            "title": title[:80] if title else skill_file.stem
        })

    return skills


def main():
    """CLI interface for skill tracking."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Track and analyze skill injections into agent contexts"
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # log command
    log_parser = subparsers.add_parser("log", help="Log a skill injection")
    log_parser.add_argument("skill", help="Skill name or path")
    log_parser.add_argument("agent", help="Agent ID receiving the skill")
    log_parser.add_argument(
        "--context", "-c",
        default="manual_read",
        choices=list(INJECTION_CONTEXTS.keys()),
        help="Injection context (default: manual_read)"
    )
    log_parser.add_argument(
        "--failed", "-f",
        action="store_true",
        help="Mark injection as failed"
    )

    # stats command
    subparsers.add_parser("stats", help="Show overall skill usage statistics")

    # agent command
    agent_parser = subparsers.add_parser("agent", help="Show skills used by an agent")
    agent_parser.add_argument("agent_id", help="Agent ID to query")

    # skill command
    skill_parser = subparsers.add_parser("skill", help="Show agents using a skill")
    skill_parser.add_argument("skill_name", help="Skill name to query")

    # recent command
    recent_parser = subparsers.add_parser("recent", help="Show recent injections")
    recent_parser.add_argument(
        "--limit", "-n",
        type=int,
        default=20,
        help="Number of entries to show (default: 20)"
    )

    # session command
    session_parser = subparsers.add_parser("session", help="Show skills used in session")
    session_parser.add_argument(
        "session_id",
        nargs="?",
        help="Session ID (defaults to current)"
    )

    # list command
    subparsers.add_parser("list", help="List all available skills")

    # health command
    health_parser = subparsers.add_parser("health", help="Show skill system health metrics")
    health_parser.add_argument(
        "--stale-days", "-s",
        type=int,
        default=7,
        help="Days without use before skill is considered stale (default: 7)"
    )

    args = parser.parse_args()

    tracker = SkillTracker()

    if args.command == "log":
        entry = tracker.log_injection(
            skill=args.skill,
            agent=args.agent,
            context=args.context,
            success=not args.failed
        )
        print(f"Logged skill injection:")
        print(json.dumps(entry, indent=2))

    elif args.command == "stats":
        stats = tracker.get_stats()
        print("=== Skill Injection Statistics ===\n")
        print(f"Total Injections: {stats['total_injections']}")
        print(f"Unique Skills: {stats['unique_skills']}")
        print(f"Unique Agents: {stats['unique_agents']}")
        print(f"Success Rate: {stats['success_rate']}%")

        if stats['first_entry']:
            print(f"\nFirst Entry: {stats['first_entry']}")
            print(f"Last Entry: {stats['last_entry']}")

        if stats['skills_by_frequency']:
            print("\n--- Top Skills ---")
            for skill, count in list(stats['skills_by_frequency'].items())[:10]:
                print(f"  {skill}: {count}")

        if stats['agents_by_frequency']:
            print("\n--- Top Agents ---")
            for agent, count in list(stats['agents_by_frequency'].items())[:10]:
                print(f"  {agent}: {count}")

        if stats['contexts_by_frequency']:
            print("\n--- Injection Contexts ---")
            for ctx, count in stats['contexts_by_frequency'].items():
                print(f"  {ctx}: {count}")

    elif args.command == "agent":
        data = tracker.get_agent_skills(args.agent_id)
        print(f"=== Skills for Agent: {args.agent_id} ===\n")
        print(f"Total Injections: {data['total_injections']}")
        print(f"Unique Skills: {data['unique_skills']}")

        if data['skills']:
            print("\n--- Skills Used ---")
            for s in data['skills']:
                print(f"  {s['skill']}: {s['count']} times (last: {s['last_used'][:10]})")

    elif args.command == "skill":
        data = tracker.get_skill_agents(args.skill_name)
        print(f"=== Usage of Skill: {data['skill']} ===\n")

        if data.get('skill_path'):
            print(f"Path: {data['skill_path']}")
        print(f"Total Injections: {data['total_injections']}")
        print(f"Unique Agents: {data['unique_agents']}")
        print(f"Unique Sessions: {data.get('unique_sessions', 0)}")

        if data.get('agents'):
            print("\n--- Agents Using This Skill ---")
            for agent, count in data['agents'].items():
                print(f"  {agent}: {count}")

    elif args.command == "recent":
        entries = tracker.get_recent(args.limit)
        print(f"=== Recent Skill Injections (last {len(entries)}) ===\n")
        for e in entries:
            ts = e['timestamp'][:19].replace('T', ' ')
            status = "OK" if e.get('success', True) else "FAIL"
            print(f"[{ts}] {e['agent']} <- {e['skill']} ({e.get('context', 'unknown')}) [{status}]")

    elif args.command == "session":
        data = tracker.get_session_skills(args.session_id)
        print(f"=== Skills in Session: {data['session']} ===\n")
        print(f"Total Injections: {data['total_injections']}")

        if data.get('skills'):
            print("\n--- Skills ---")
            for skill, count in data['skills'].items():
                print(f"  {skill}: {count}")

        if data.get('agents'):
            print("\n--- Agents ---")
            for agent, count in data['agents'].items():
                print(f"  {agent}: {count}")

    elif args.command == "list":
        skills = list_available_skills()
        print(f"=== Available Skills ({len(skills)}) ===\n")
        for s in skills:
            print(f"  {s['name']}")
            print(f"    Path: {s['path']}")
            print(f"    Title: {s['title']}")
            print()

    elif args.command == "health":
        health = tracker.get_skill_health(stale_days=args.stale_days)
        print("=== Skill System Health Report ===\n")

        # Health score with visual indicator
        score = health['health_score']
        if score >= 80:
            indicator = "HEALTHY"
        elif score >= 60:
            indicator = "FAIR"
        elif score >= 40:
            indicator = "NEEDS ATTENTION"
        else:
            indicator = "CRITICAL"
        print(f"Health Score: {score}/100 ({indicator})")
        print()

        # Core metrics
        print("--- Core Metrics ---")
        print(f"  Total Injections: {health['total_injections']}")
        print(f"  Success Rate: {health['success_rate']}%")
        print(f"  Skill Diversity: {health['skill_diversity']} unique skills")
        print(f"  Agent Coverage: {health['agent_coverage']} agents")

        if health['period']['start']:
            print(f"\n--- Period ---")
            print(f"  From: {health['period']['start'][:19]}")
            print(f"  To: {health['period']['end'][:19]}")
            print(f"  Duration: {health['period']['days']} days")

        if health.get('top_skills'):
            print("\n--- Top Skills ---")
            for skill, count in health['top_skills'].items():
                print(f"  {skill}: {count}")

        if health.get('top_agents'):
            print("\n--- Top Agents ---")
            for agent, count in health['top_agents'].items():
                print(f"  {agent}: {count}")

        # Anomalies
        if health['anomalies']:
            print("\n--- Anomalies Detected ---")
            for a in health['anomalies']:
                severity = a['severity'].upper()
                print(f"  [{severity}] {a['message']}")
                if 'details' in a:
                    for k, v in a['details'].items():
                        print(f"    - {k}: {v}")
                if 'skills' in a:
                    print(f"    Skills: {', '.join(a['skills'][:5])}")

        # Recommendations
        if health['recommendations']:
            print("\n--- Recommendations ---")
            for r in health['recommendations']:
                print(f"  * {r}")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
