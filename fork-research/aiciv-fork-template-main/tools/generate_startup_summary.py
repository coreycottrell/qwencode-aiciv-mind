#!/usr/bin/env python3
"""
Automated Startup Summary Generator for AI Agents

Purpose: Auto-generate narrative session startup summaries for agents
by reading their performance logs and identifying relevant patterns.

Usage:
    python3 generate_startup_summary.py --agent <agent-id> --task "<task description>"
    python3 generate_startup_summary.py --agent coder --task "implement REST API endpoints"
    python3 generate_startup_summary.py --agent researcher --task "research GraphQL vs REST" --output summary.md
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set


# Common stop words to filter out during keyword extraction
STOP_WORDS = {
    'a', 'an', 'and', 'are', 'as', 'at', 'be', 'by', 'for', 'from', 'has', 'he',
    'in', 'is', 'it', 'its', 'of', 'on', 'that', 'the', 'to', 'was', 'will', 'with',
    'this', 'but', 'they', 'have', 'had', 'what', 'when', 'where', 'who', 'which',
    'why', 'how', 'all', 'each', 'every', 'both', 'few', 'more', 'most', 'other',
    'some', 'such', 'no', 'nor', 'not', 'only', 'own', 'same', 'so', 'than', 'too',
    'very', 'can', 'just', 'should', 'now'
}


def extract_keywords(text: str, min_length: int = 3) -> Set[str]:
    """Extract meaningful keywords from text by removing stop words."""
    words = text.lower().replace('-', ' ').replace('_', ' ').split()
    keywords = {
        word.strip('.,!?;:()"\'[]{}')
        for word in words
        if len(word) >= min_length and word.lower() not in STOP_WORDS
    }
    return keywords


def load_performance_log(agent_id: str, base_path: Path) -> Dict:
    """Load agent's performance log from JSON file."""
    # Try both possible locations
    log_paths = [
        base_path / "memories" / "agents" / agent_id / "logs" / "performance_log.json",
        base_path / "memories" / "agents" / agent_id / "performance_log.json"
    ]

    for log_path in log_paths:
        if log_path.exists():
            try:
                with open(log_path, 'r') as f:
                    data = json.load(f)
                    # Handle both old format (task_history) and new format (entries)
                    if "task_history" not in data and "entries" not in data:
                        return {"task_history": [], "entries": []}
                    return data
            except json.JSONDecodeError:
                print(f"Warning: Could not parse performance log for {agent_id}", file=sys.stderr)
                return {"task_history": [], "entries": []}

    # No log found in either location
    return {"task_history": [], "entries": []}


def get_recent_entries(performance_log: Dict, limit: int = 3) -> List[Dict]:
    """Get the most recent N entries from performance log."""
    # Try multiple formats: entries, task_history, tasks_completed
    entries = performance_log.get("entries", [])
    if not entries:
        entries = performance_log.get("task_history", [])
    if not entries:
        entries = performance_log.get("tasks_completed", [])

    # Sort by timestamp or date (most recent first)
    sorted_entries = sorted(
        entries,
        key=lambda x: x.get("timestamp") or x.get("date") or "",
        reverse=True
    )

    return sorted_entries[:limit]


def search_patterns_directory(agent_id: str, base_path: Path, keywords: Set[str]) -> List[Dict]:
    """Search agent's patterns directory for files matching keywords."""
    patterns_dir = base_path / "memories" / "agents" / agent_id / "patterns"

    if not patterns_dir.exists():
        return []

    matching_patterns = []

    for pattern_file in patterns_dir.glob("*.md"):
        try:
            content = pattern_file.read_text()
            file_keywords = extract_keywords(content)

            # Calculate relevance score based on keyword overlap
            overlap = keywords & file_keywords
            if overlap:
                score = len(overlap)
                matching_patterns.append({
                    "file": pattern_file.name,
                    "path": str(pattern_file),
                    "score": score,
                    "matched_keywords": list(overlap)
                })
        except Exception as e:
            print(f"Warning: Could not read pattern file {pattern_file}: {e}", file=sys.stderr)

    # Sort by relevance score
    matching_patterns.sort(key=lambda x: x["score"], reverse=True)
    return matching_patterns


def search_knowledge_base(base_path: Path, keywords: Set[str], limit: int = 5) -> List[Dict]:
    """Search knowledge base for relevant documents."""
    knowledge_dir = base_path / "memories" / "knowledge"

    if not knowledge_dir.exists():
        return []

    matching_docs = []

    for doc_file in knowledge_dir.rglob("*.md"):
        try:
            content = doc_file.read_text()
            file_keywords = extract_keywords(content)

            overlap = keywords & file_keywords
            if overlap:
                score = len(overlap)
                matching_docs.append({
                    "file": doc_file.name,
                    "path": str(doc_file.relative_to(base_path)),
                    "score": score,
                    "matched_keywords": list(overlap)
                })
        except Exception as e:
            print(f"Warning: Could not read knowledge file {doc_file}: {e}", file=sys.stderr)

    matching_docs.sort(key=lambda x: x["score"], reverse=True)
    return matching_docs[:limit]


def generate_narrative_summary(
    agent_id: str,
    task_description: str,
    recent_entries: List[Dict],
    matching_patterns: List[Dict],
    matching_knowledge: List[Dict],
    base_path: Path
) -> str:
    """Generate narrative startup summary for the agent."""

    # Load agent manifest for identity info
    manifest_path = base_path / ".claude" / "agents" / f"{agent_id}.md"
    agent_role = "Unknown"

    if manifest_path.exists():
        try:
            manifest_content = manifest_path.read_text()
            # Extract role from manifest (simple parsing)
            for line in manifest_content.split('\n'):
                if line.startswith('**Role:**'):
                    agent_role = line.replace('**Role:**', '').strip()
                    break
        except Exception:
            pass

    # Build the narrative
    lines = []
    lines.append("# Your Context (Session Startup Summary)")
    lines.append("")
    lines.append(f"**Agent:** {agent_id}")
    lines.append(f"**Role:** {agent_role}")
    lines.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    lines.append("")

    # Recent work summary
    lines.append("## Recent Work History")
    if recent_entries:
        last_entry = recent_entries[0]
        task_desc = (
            last_entry.get("task_description") or
            last_entry.get("description") or
            last_entry.get("task_id", "Unknown task")
        )
        status = last_entry.get("status", "unknown")
        timestamp = last_entry.get("timestamp") or last_entry.get("date", "unknown time")

        lines.append(f"**Last worked on:** {task_desc}")
        lines.append(f"**Status:** {status}")
        lines.append(f"**When:** {timestamp}")
        lines.append("")

        # Extract key outcomes/learnings from last 3 tasks
        learnings = []
        for entry in recent_entries[:3]:
            # Check various fields for learnings
            if "key_findings" in entry:
                learnings.extend(entry["key_findings"][:2])  # Top 2 findings per task
            if "notes" in entry and isinstance(entry["notes"], list):
                learnings.extend(entry["notes"][:2])
            if "success_factors" in entry and isinstance(entry["success_factors"], list):
                learnings.extend(entry["success_factors"][:2])
            if "features_implemented" in entry and isinstance(entry["features_implemented"], list):
                learnings.append(f"Implemented: {', '.join(entry['features_implemented'][:3])}")

        if learnings:
            lines.append("**Recent Learnings:**")
            for i, learning in enumerate(learnings[:3], 1):
                lines.append(f"{i}. {learning}")
            lines.append("")
    else:
        lines.append("*No previous work history found. This may be your first task!*")
        lines.append("")

    # Tools and patterns used
    if recent_entries:
        all_tools = set()
        all_technologies = set()
        for entry in recent_entries:
            tools = entry.get("tools_used", [])
            all_tools.update(tools)
            techs = entry.get("technologies", [])
            all_technologies.update(techs)

        if all_tools:
            lines.append(f"**Tools you've used recently:** {', '.join(sorted(all_tools))}")
            lines.append("")
        elif all_technologies:
            lines.append(f"**Technologies you've used recently:** {', '.join(sorted(list(all_technologies)[:5]))}")
            lines.append("")

    # Current task analysis
    lines.append("## Today's Focus")
    lines.append(f"**Task:** {task_description}")
    lines.append("")

    task_keywords = extract_keywords(task_description)
    if task_keywords:
        lines.append("**Key concepts identified:** " + ", ".join(sorted(list(task_keywords)[:10])))
        lines.append("")

    # Recommended patterns
    lines.append("## Recommended Patterns & Resources")

    if matching_patterns:
        lines.append("**Relevant patterns from your experience:**")
        for i, pattern in enumerate(matching_patterns[:5], 1):
            keywords_str = ", ".join(pattern["matched_keywords"][:3])
            lines.append(f"{i}. `{pattern['file']}` (matches: {keywords_str})")
            lines.append(f"   Path: `{pattern['path']}`")
        lines.append("")
    else:
        lines.append("*No existing patterns found. You'll be creating new ones!*")
        lines.append("")

    if matching_knowledge:
        lines.append("**Relevant knowledge base articles:**")
        for i, doc in enumerate(matching_knowledge[:5], 1):
            keywords_str = ", ".join(doc["matched_keywords"][:3])
            lines.append(f"{i}. `{doc['file']}` (matches: {keywords_str})")
            lines.append(f"   Path: `{doc['path']}`")
        lines.append("")

    # Success criteria reminder
    lines.append("## Success Checklist")
    lines.append("- [ ] Task matches specification requirements")
    lines.append("- [ ] Code/output passes quality checks")
    lines.append("- [ ] Tests pass (if applicable)")
    lines.append("- [ ] Performance log updated")
    lines.append("- [ ] New patterns documented (if discovered)")
    lines.append("")

    lines.append("---")
    lines.append("*Summary auto-generated by `tools/generate_startup_summary.py`*")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Generate narrative startup summary for AI agents",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --agent coder --task "implement REST API endpoints"
  %(prog)s --agent researcher --task "research GraphQL vs REST"
  %(prog)s --agent tester --task "write integration tests" --output summary.md
        """
    )

    parser.add_argument(
        "--agent",
        required=True,
        help="Agent ID (e.g., coder, researcher, tester)"
    )

    parser.add_argument(
        "--task",
        required=True,
        help="Current task description"
    )

    parser.add_argument(
        "--output",
        help="Output file path (default: stdout)"
    )

    parser.add_argument(
        "--base-path",
        type=Path,
        default=Path.cwd(),
        help="Base repository path (default: current directory)"
    )

    args = parser.parse_args()

    # Validate agent exists
    agent_dir = args.base_path / "memories" / "agents" / args.agent
    if not agent_dir.exists():
        print(f"Error: Agent '{args.agent}' not found in memories/agents/", file=sys.stderr)
        print(f"Checked path: {agent_dir}", file=sys.stderr)
        sys.exit(1)

    # Load performance log
    performance_log = load_performance_log(args.agent, args.base_path)
    recent_entries = get_recent_entries(performance_log, limit=3)

    # Extract keywords from task
    task_keywords = extract_keywords(args.task)

    # Search for relevant patterns and knowledge
    matching_patterns = search_patterns_directory(args.agent, args.base_path, task_keywords)
    matching_knowledge = search_knowledge_base(args.base_path, task_keywords, limit=5)

    # Generate narrative summary
    summary = generate_narrative_summary(
        args.agent,
        args.task,
        recent_entries,
        matching_patterns,
        matching_knowledge,
        args.base_path
    )

    # Output
    if args.output:
        output_path = Path(args.output)
        output_path.write_text(summary)
        print(f"Summary written to: {output_path}")
    else:
        print(summary)


if __name__ == "__main__":
    main()
