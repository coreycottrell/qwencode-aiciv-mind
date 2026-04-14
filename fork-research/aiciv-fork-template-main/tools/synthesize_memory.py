#!/usr/bin/env python3
"""
Dual-Tier Memory System - Synthesis Tool

Converts raw logs into synthesized, actionable references for agents.

Structure:
- logs/ - Raw data (append-only, untouched)
- references/ - Synthesized (auto-generated daily)
- patterns/ - Domain patterns (agent-specific)

Usage:
    python tools/synthesize_memory.py --agent coder
    python tools/synthesize_memory.py --agent all
    python tools/synthesize_memory.py --agent coder --verbose
"""

import json
import os
import sys
from collections import defaultdict, Counter
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Tuple
import argparse


class MemorySynthesizer:
    """Synthesizes raw logs into actionable references."""

    def __init__(self, project_root: Path, verbose: bool = False):
        self.project_root = project_root
        self.agents_dir = project_root / "memories" / "agents"
        self.verbose = verbose

    def log(self, message: str):
        """Log if verbose mode is enabled."""
        if self.verbose:
            print(f"[SYNTH] {message}")

    def get_all_agents(self) -> List[str]:
        """Get list of all agent directories."""
        agents = []
        for item in self.agents_dir.iterdir():
            if item.is_dir() and not item.name.startswith('.'):
                agents.append(item.name)
        return sorted(agents)

    def setup_directory_structure(self, agent_id: str) -> Tuple[Path, Path, Path]:
        """Create logs/, references/, patterns/ directories for an agent."""
        agent_dir = self.agents_dir / agent_id

        logs_dir = agent_dir / "logs"
        references_dir = agent_dir / "references"
        patterns_dir = agent_dir / "patterns"

        # Create directories
        logs_dir.mkdir(exist_ok=True)
        references_dir.mkdir(exist_ok=True)
        patterns_dir.mkdir(exist_ok=True)

        self.log(f"Directory structure ready for {agent_id}")

        return logs_dir, references_dir, patterns_dir

    def migrate_existing_logs(self, agent_id: str, logs_dir: Path):
        """Move existing logs to logs/ directory."""
        agent_dir = self.agents_dir / agent_id

        # Files to move to logs/
        log_files = [
            "performance_log.json",
            "email_activity.jsonl",
            "sent_emails.json",
            "contacts.json",
            "patterns.json",
            "response_rules.json",
            "email_log-*.jsonl",
            "session-*.jsonl"
        ]

        moved_count = 0
        for pattern in log_files:
            # Handle glob patterns
            if '*' in pattern:
                for file_path in agent_dir.glob(pattern):
                    if file_path.is_file() and not file_path.is_relative_to(logs_dir):
                        target = logs_dir / file_path.name
                        if not target.exists():
                            file_path.rename(target)
                            moved_count += 1
                            self.log(f"Moved {file_path.name} to logs/")
            else:
                file_path = agent_dir / pattern
                if file_path.exists() and file_path.is_file():
                    target = logs_dir / pattern
                    if not target.exists():
                        file_path.rename(target)
                        moved_count += 1
                        self.log(f"Moved {pattern} to logs/")

        if moved_count > 0:
            self.log(f"Migrated {moved_count} log files to logs/")

    def load_performance_log(self, logs_dir: Path) -> Dict[str, Any]:
        """Load performance_log.json from logs directory."""
        perf_log_path = logs_dir / "performance_log.json"

        if not perf_log_path.exists():
            self.log(f"No performance_log.json found in {logs_dir}")
            return {"agent": "", "entries": []}

        with open(perf_log_path, 'r') as f:
            data = json.load(f)

            # Normalize format: convert "tasks_completed" to "entries" for consistency
            if "tasks_completed" in data and "entries" not in data:
                data["entries"] = data["tasks_completed"]

            return data

    def synthesize_top_patterns(self, agent_id: str, logs_dir: Path, references_dir: Path):
        """
        Synthesize top patterns from performance logs.

        Generates: references/top-patterns.md
        """
        perf_log = self.load_performance_log(logs_dir)

        # Extract patterns usage
        pattern_usage = Counter()
        pattern_success = defaultdict(lambda: {"total": 0, "successful": 0})
        pattern_last_used = {}
        pattern_tasks = defaultdict(list)

        for entry in perf_log.get("entries", []):
            patterns = entry.get("patterns_used", [])
            status = entry.get("status", "unknown")
            task = entry.get("task", "unknown")
            timestamp = entry.get("timestamp", "")

            for pattern in patterns:
                pattern_usage[pattern] += 1
                pattern_success[pattern]["total"] += 1
                if status == "completed":
                    pattern_success[pattern]["successful"] += 1
                pattern_last_used[pattern] = timestamp
                pattern_tasks[pattern].append(task)

        # Generate top-patterns.md
        top_patterns_md = f"""# Top Patterns - {agent_id}

**Generated:** {datetime.now().isoformat()}

This document shows the most frequently used patterns, their success rates, and quick references.

## Pattern Usage Summary

"""

        if not pattern_usage:
            top_patterns_md += "_No patterns recorded yet._\n"
        else:
            top_patterns_md += "| Pattern | Times Used | Success Rate | Last Used |\n"
            top_patterns_md += "|---------|------------|--------------|----------|\n"

            for pattern, count in pattern_usage.most_common(10):
                stats = pattern_success[pattern]
                success_rate = (stats["successful"] / stats["total"] * 100) if stats["total"] > 0 else 0
                last_used = pattern_last_used.get(pattern, "unknown")

                top_patterns_md += f"| {pattern} | {count} | {success_rate:.1f}% | {last_used} |\n"

            top_patterns_md += "\n## Pattern Details\n\n"

            for pattern, count in pattern_usage.most_common(10):
                stats = pattern_success[pattern]
                success_rate = (stats["successful"] / stats["total"] * 100) if stats["total"] > 0 else 0
                tasks = pattern_tasks[pattern][:5]  # First 5 tasks

                top_patterns_md += f"### {pattern}\n\n"
                top_patterns_md += f"- **Usage Count:** {count}\n"
                top_patterns_md += f"- **Success Rate:** {success_rate:.1f}%\n"
                top_patterns_md += f"- **Last Used:** {pattern_last_used[pattern]}\n"
                top_patterns_md += f"- **Example Tasks:** {', '.join(tasks)}\n"
                top_patterns_md += f"- **Reference:** `patterns/{pattern}.md` (if exists)\n\n"

        top_patterns_md += "\n---\n*Auto-generated by synthesize_memory.py*\n"

        # Write file
        output_path = references_dir / "top-patterns.md"
        with open(output_path, 'w') as f:
            f.write(top_patterns_md)

        self.log(f"Generated {output_path}")
        return len(pattern_usage)

    def synthesize_lessons_learned(self, agent_id: str, logs_dir: Path, references_dir: Path):
        """
        Extract lessons learned from performance logs.

        Generates: references/lessons-learned.md
        """
        perf_log = self.load_performance_log(logs_dir)

        # Extract lessons
        lessons = []

        for entry in perf_log.get("entries", []):
            what_worked = entry.get("what_worked_well", [])
            new_knowledge = entry.get("new_knowledge_gained", [])
            challenges = entry.get("challenges_encountered", [])
            task = entry.get("task", "unknown")
            timestamp = entry.get("timestamp", "")

            for lesson in what_worked:
                lessons.append({
                    "type": "success",
                    "content": lesson,
                    "task": task,
                    "timestamp": timestamp
                })

            for knowledge in new_knowledge:
                lessons.append({
                    "type": "knowledge",
                    "content": knowledge,
                    "task": task,
                    "timestamp": timestamp
                })

            for challenge in challenges:
                lessons.append({
                    "type": "challenge",
                    "content": challenge,
                    "task": task,
                    "timestamp": timestamp
                })

        # Generate lessons-learned.md
        lessons_md = f"""# Lessons Learned - {agent_id}

**Generated:** {datetime.now().isoformat()}

Extracted wisdom from performance logs, organized by category.

## Success Patterns

"""

        successes = [l for l in lessons if l["type"] == "success"]
        if successes:
            for lesson in successes[:10]:  # Top 10
                lessons_md += f"- **{lesson['content']}**\n"
                lessons_md += f"  - Task: `{lesson['task']}`\n"
                lessons_md += f"  - Date: {lesson['timestamp']}\n\n"
        else:
            lessons_md += "_No success patterns recorded yet._\n\n"

        lessons_md += "## Knowledge Gained\n\n"

        knowledge = [l for l in lessons if l["type"] == "knowledge"]
        if knowledge:
            for lesson in knowledge[:10]:  # Top 10
                lessons_md += f"- **{lesson['content']}**\n"
                lessons_md += f"  - Task: `{lesson['task']}`\n"
                lessons_md += f"  - Date: {lesson['timestamp']}\n\n"
        else:
            lessons_md += "_No knowledge entries recorded yet._\n\n"

        lessons_md += "## Challenges & Pitfalls\n\n"

        challenges = [l for l in lessons if l["type"] == "challenge"]
        if challenges:
            for lesson in challenges[:10]:  # Top 10
                lessons_md += f"- **{lesson['content']}**\n"
                lessons_md += f"  - Task: `{lesson['task']}`\n"
                lessons_md += f"  - Date: {lesson['timestamp']}\n\n"
        else:
            lessons_md += "_No challenges recorded yet._\n\n"

        lessons_md += "\n---\n*Auto-generated by synthesize_memory.py*\n"

        # Write file
        output_path = references_dir / "lessons-learned.md"
        with open(output_path, 'w') as f:
            f.write(lessons_md)

        self.log(f"Generated {output_path}")
        return len(lessons)

    def synthesize_quick_start(self, agent_id: str, logs_dir: Path, references_dir: Path):
        """
        Generate quick-start guide for agent.

        Generates: references/quick-start.md
        """
        # Load manifest
        manifest_path = self.project_root / ".claude" / "agents" / f"{agent_id}.md"
        role_description = "Agent role"

        if manifest_path.exists():
            with open(manifest_path, 'r') as f:
                content = f.read()
                # Extract role from manifest (first paragraph after title)
                lines = content.split('\n')
                for i, line in enumerate(lines):
                    if line.startswith('# ') and i + 2 < len(lines):
                        role_description = lines[i + 2].strip()
                        break

        # Get top 3 patterns
        perf_log = self.load_performance_log(logs_dir)
        pattern_usage = Counter()

        for entry in perf_log.get("entries", []):
            patterns = entry.get("patterns_used", [])
            for pattern in patterns:
                pattern_usage[pattern] += 1

        top_3_patterns = pattern_usage.most_common(3)

        # Generate quick-start.md
        quick_start_md = f"""# Quick Start Guide - {agent_id}

**Generated:** {datetime.now().isoformat()}

This is your one-page orientation. Read this first!

## Who Am I?

**Role:** {role_description}

**Agent ID:** `{agent_id}`

**Manifest:** `.claude/agents/{agent_id}.md`

## Top 3 Patterns I Use

"""

        if top_3_patterns:
            for i, (pattern, count) in enumerate(top_3_patterns, 1):
                quick_start_md += f"{i}. **{pattern}** (used {count} times)\n"
                quick_start_md += f"   - Reference: `patterns/{pattern}.md`\n\n"
        else:
            quick_start_md += "_No patterns established yet. Build your first patterns!_\n\n"

        quick_start_md += """## Top 3 Lessons to Remember

(Review `references/lessons-learned.md` for full details)

1. **Check logs first** - Search your memory before starting tasks
2. **Update performance logs** - Document what worked and what didn't
3. **Reuse patterns** - Don't reinvent the wheel

## Common Pitfalls to Avoid

- Skipping the verification step
- Not documenting new patterns
- Forgetting to update performance logs after tasks

## Where to Find Detailed Info

- **Performance logs:** `logs/performance_log.json` (raw data)
- **Top patterns:** `references/top-patterns.md` (synthesized)
- **Lessons learned:** `references/lessons-learned.md` (synthesized)
- **Metrics dashboard:** `references/metrics-dashboard.md` (synthesized)
- **Pattern library:** `patterns/*.md` (domain-specific)

## Quick Commands

```bash
# Search my memory
grep -r "keyword" memories/agents/{agent_id}/

# View my top patterns
cat memories/agents/{agent_id}/references/top-patterns.md

# Check my metrics
cat memories/agents/{agent_id}/references/metrics-dashboard.md
```

---
*Auto-generated by synthesize_memory.py*
"""

        # Write file
        output_path = references_dir / "quick-start.md"
        with open(output_path, 'w') as f:
            f.write(quick_start_md)

        self.log(f"Generated {output_path}")

    def synthesize_metrics_dashboard(self, agent_id: str, logs_dir: Path, references_dir: Path):
        """
        Generate metrics dashboard from performance logs.

        Generates: references/metrics-dashboard.md
        """
        perf_log = self.load_performance_log(logs_dir)

        # Handle different log formats
        entries = perf_log.get("entries", [])

        # Calculate metrics
        total_tasks = len(entries)
        completed = sum(1 for e in entries if e.get("status") == "completed")
        failed = sum(1 for e in entries if e.get("status") == "failed")
        in_progress = sum(1 for e in entries if e.get("status") == "in_progress")

        completion_rate = (completed / total_tasks * 100) if total_tasks > 0 else 0

        # Quality scores
        quality_scores = [e.get("quality_score", 0) for e in entries if "quality_score" in e]
        avg_quality = sum(quality_scores) / len(quality_scores) if quality_scores else 0

        # Pattern reuse
        pattern_usage = Counter()
        for entry in entries:
            patterns = entry.get("patterns_used", [])
            for pattern in patterns:
                pattern_usage[pattern] += 1

        total_pattern_uses = sum(pattern_usage.values())
        pattern_reuse_rate = (total_pattern_uses / total_tasks) if total_tasks > 0 else 0

        # Check for alternative metrics structure (like email-monitor)
        alt_metrics = perf_log.get("metrics", {})
        has_alt_metrics = bool(alt_metrics)

        # Generate dashboard
        dashboard_md = f"""# Metrics Dashboard - {agent_id}

**Generated:** {datetime.now().isoformat()}

## Current Stats

"""

        if has_alt_metrics:
            # Use alternative metrics structure
            dashboard_md += "| Metric | Value |\n|--------|-------|\n"
            for key, value in sorted(alt_metrics.items()):
                metric_name = key.replace('_', ' ').title()
                dashboard_md += f"| {metric_name} | {value} |\n"
            dashboard_md += "\n"
        else:
            # Use standard task-based metrics
            dashboard_md += f"""| Metric | Value |
|--------|-------|
| Total Tasks | {total_tasks} |
| Completed | {completed} ({completion_rate:.1f}%) |
| Failed | {failed} |
| In Progress | {in_progress} |
| Average Quality Score | {avg_quality:.2f}/10 |
| Pattern Reuse Rate | {pattern_reuse_rate:.2f} patterns/task |

"""

        dashboard_md += "## Task Completion Trend\n\n"

        if total_tasks > 0:
            # ASCII chart (simple bar chart)
            dashboard_md += "```\n"
            dashboard_md += f"Completed:   {'█' * int(completed / total_tasks * 50)} {completed}\n"
            dashboard_md += f"Failed:      {'█' * int(failed / total_tasks * 50) if failed > 0 else ''} {failed}\n"
            dashboard_md += f"In Progress: {'█' * int(in_progress / total_tasks * 50) if in_progress > 0 else ''} {in_progress}\n"
            dashboard_md += "```\n\n"
        else:
            dashboard_md += "_No tasks recorded yet._\n\n"

        dashboard_md += "## Pattern Usage\n\n"

        if pattern_usage:
            dashboard_md += "Top 5 patterns:\n\n"
            for pattern, count in pattern_usage.most_common(5):
                dashboard_md += f"- **{pattern}**: {count} uses\n"
        else:
            dashboard_md += "_No patterns recorded yet._\n"

        dashboard_md += "\n## Improvement Areas\n\n"

        if has_alt_metrics:
            # Check for learnings and failures
            learnings = perf_log.get("learnings", [])
            failures = perf_log.get("failures", [])

            if failures:
                dashboard_md += f"- **Recent Failures:** {len(failures)} documented\n"
            if learnings:
                dashboard_md += f"- **Recent Learnings:** {len(learnings)} captured\n"

            status = perf_log.get("status", "unknown")
            dashboard_md += f"- **Current Status:** {status}\n"

            if not failures and learnings:
                dashboard_md += "\n_Good trajectory! Learning without failures._\n"
        else:
            # Standard improvement areas
            if completion_rate < 90:
                dashboard_md += "- **Task Completion Rate:** Target 90%+, currently {:.1f}%\n".format(completion_rate)
            if avg_quality < 8.0:
                dashboard_md += "- **Quality Score:** Target 8.0+, currently {:.2f}\n".format(avg_quality)
            if pattern_reuse_rate < 2.0:
                dashboard_md += "- **Pattern Reuse:** Target 2.0+ patterns/task, currently {:.2f}\n".format(pattern_reuse_rate)

            if completion_rate >= 90 and avg_quality >= 8.0 and pattern_reuse_rate >= 2.0:
                dashboard_md += "_All metrics meeting targets! Keep up the great work._\n"

        dashboard_md += "\n---\n*Auto-generated by synthesize_memory.py*\n"

        # Write file
        output_path = references_dir / "metrics-dashboard.md"
        with open(output_path, 'w') as f:
            f.write(dashboard_md)

        self.log(f"Generated {output_path}")

    def synthesize_agent(self, agent_id: str) -> Dict[str, Any]:
        """
        Synthesize all references for a single agent.

        Returns:
            Dict with synthesis results
        """
        print(f"\n{'='*60}")
        print(f"Synthesizing memory for agent: {agent_id}")
        print(f"{'='*60}\n")

        # Setup directories
        logs_dir, references_dir, patterns_dir = self.setup_directory_structure(agent_id)

        # Migrate existing logs
        self.migrate_existing_logs(agent_id, logs_dir)

        # Synthesize all references
        pattern_count = self.synthesize_top_patterns(agent_id, logs_dir, references_dir)
        lessons_count = self.synthesize_lessons_learned(agent_id, logs_dir, references_dir)
        self.synthesize_quick_start(agent_id, logs_dir, references_dir)
        self.synthesize_metrics_dashboard(agent_id, logs_dir, references_dir)

        result = {
            "agent_id": agent_id,
            "patterns_synthesized": pattern_count,
            "lessons_synthesized": lessons_count,
            "references_generated": 4,
            "timestamp": datetime.now().isoformat()
        }

        print(f"\n✓ Synthesis complete for {agent_id}")
        print(f"  - {pattern_count} patterns found")
        print(f"  - {lessons_count} lessons extracted")
        print(f"  - 4 reference documents generated")

        return result

    def synthesize_all(self) -> List[Dict[str, Any]]:
        """Synthesize references for all agents."""
        agents = self.get_all_agents()
        results = []

        print(f"\nFound {len(agents)} agents to synthesize\n")

        for agent_id in agents:
            result = self.synthesize_agent(agent_id)
            results.append(result)

        return results


def main():
    parser = argparse.ArgumentParser(
        description="Dual-Tier Memory System - Synthesize raw logs into actionable references"
    )
    parser.add_argument(
        "--agent",
        required=True,
        help="Agent ID to synthesize (or 'all' for all agents)"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose logging"
    )

    args = parser.parse_args()

    # Get project root (2 levels up from tools/)
    project_root = Path(__file__).parent.parent

    # Create synthesizer
    synthesizer = MemorySynthesizer(project_root, verbose=args.verbose)

    # Synthesize
    if args.agent == "all":
        results = synthesizer.synthesize_all()

        print(f"\n{'='*60}")
        print("SYNTHESIS COMPLETE - ALL AGENTS")
        print(f"{'='*60}\n")

        total_patterns = sum(r["patterns_synthesized"] for r in results)
        total_lessons = sum(r["lessons_synthesized"] for r in results)

        print(f"Total agents processed: {len(results)}")
        print(f"Total patterns synthesized: {total_patterns}")
        print(f"Total lessons extracted: {total_lessons}")
        print(f"Total reference documents: {len(results) * 4}")
    else:
        result = synthesizer.synthesize_agent(args.agent)

        print(f"\n{'='*60}")
        print("SYNTHESIS COMPLETE")
        print(f"{'='*60}\n")

    print("\nNext steps:")
    print("  1. Review generated references in memories/agents/[agent-id]/references/")
    print("  2. Add to daily workflow: Run after each session")
    print("  3. Agents should read quick-start.md on session start")
    print()


if __name__ == "__main__":
    main()
