#!/usr/bin/env python3
"""
Flow Selector Module - Intelligent Flow Recommendation for A-C-Gee

Loads flow definitions from `.claude/skills/flows/*/SKILL.md`, parses their
YAML frontmatter, and recommends the most appropriate flow based on task
description pattern matching.

Part of the Flow Orchestration System (ADR-048).

Memory Search Results:
- Searched: Memory server for "flow selector pattern matching yaml parsing"
- Searched: tools/memory_compliance.py for dataclass patterns
- Found: memory_compliance.py excellent dataclass/enum patterns, 5 flow SKILL.md files
- Applying: Dataclass patterns, YAML frontmatter structure from flow files

Created: 2026-01-16
Author: coder-agent
"""

import os
import re
import sys
import json
import yaml
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional, Any, Tuple
from pathlib import Path
from enum import Enum


# === Configuration ===

# Default path to flows directory (relative to project root)
DEFAULT_FLOWS_PATH = ".claude/skills/flows"

# Project root detection
PROJECT_ROOT = Path(__file__).parent.parent.absolute()


class ConfidenceLevel(str, Enum):
    """Confidence level categories for flow recommendations."""
    HIGH = "high"      # >= 0.7
    MEDIUM = "medium"  # >= 0.4
    LOW = "low"        # < 0.4
    NONE = "none"      # No match


@dataclass
class FlowMetadata:
    """
    Parsed metadata from a flow SKILL.md file's YAML frontmatter.

    Attributes:
        name: Flow identifier (e.g., "feature-implementation")
        version: Semantic version string
        description: Full description of the flow
        trigger_pattern: Regex pattern for task matching
        trigger_explicit: Whether flow requires explicit invocation
        trigger_event: Event name if event-triggered
        commander_intent: What the flow aims to achieve
        applicable_agents: Agents that can execute this flow
        estimated_duration: Expected time range
        complexity: Flow complexity level
        file_path: Path to the SKILL.md file
        status: Flow status (TESTING, PRODUCTION, etc.)
    """
    name: str
    version: str = "1.0.0"
    description: str = ""
    trigger_pattern: str = ""
    trigger_explicit: bool = True
    trigger_event: str = ""
    commander_intent: str = ""
    applicable_agents: List[str] = field(default_factory=list)
    estimated_duration: str = ""
    complexity: str = "medium"
    file_path: str = ""
    status: str = "TESTING"

    def matches_task(self, task: str) -> Tuple[bool, float]:
        """
        Check if task matches this flow's trigger pattern.

        Args:
            task: Task description to match against

        Returns:
            Tuple of (matched: bool, specificity_score: float)
            Specificity score is higher for more specific patterns.
        """
        if not self.trigger_pattern:
            return False, 0.0

        try:
            pattern = re.compile(self.trigger_pattern, re.IGNORECASE)
            match = pattern.search(task)
            if match:
                # Calculate specificity based on pattern characteristics
                specificity = self._calculate_pattern_specificity(match, task)
                return True, specificity
        except re.error:
            # Invalid regex pattern
            return False, 0.0

        return False, 0.0

    def _calculate_pattern_specificity(self, match: re.Match, task: str) -> float:
        """
        Calculate specificity score for a pattern match.

        More specific patterns (longer matches, more constraints) score higher.

        Args:
            match: The regex match object
            task: The original task string

        Returns:
            Specificity score between 0.0 and 1.0
        """
        score = 0.5  # Base score for any match

        # Factor 1: Match length relative to task length (0-0.2)
        match_coverage = len(match.group(0)) / max(len(task), 1)
        score += min(0.2, match_coverage * 0.3)

        # Factor 2: Pattern complexity (0-0.15)
        # More complex patterns are more specific
        pattern_chars = len(self.trigger_pattern)
        if pattern_chars > 50:
            score += 0.15
        elif pattern_chars > 30:
            score += 0.10
        elif pattern_chars > 15:
            score += 0.05

        # Factor 3: Literal characters in pattern (0-0.15)
        # Patterns with more literal words are more specific
        literal_words = re.findall(r'[a-zA-Z]{3,}', self.trigger_pattern)
        if len(literal_words) >= 4:
            score += 0.15
        elif len(literal_words) >= 2:
            score += 0.10
        elif len(literal_words) >= 1:
            score += 0.05

        return min(1.0, score)


@dataclass
class FlowRecommendation:
    """
    Result of flow selection for a given task.

    Attributes:
        recommended_flow: Name of the best-matching flow
        confidence: Confidence score 0.0-1.0
        reasoning: Explanation of why this flow was selected
        alternatives: List of other possible flows with their scores
        metadata: Additional context about the selection
    """
    recommended_flow: str
    confidence: float
    reasoning: str
    alternatives: List[Dict[str, Any]] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    @property
    def confidence_level(self) -> ConfidenceLevel:
        """Get categorical confidence level."""
        if self.confidence >= 0.7:
            return ConfidenceLevel.HIGH
        elif self.confidence >= 0.4:
            return ConfidenceLevel.MEDIUM
        elif self.confidence > 0:
            return ConfidenceLevel.LOW
        return ConfidenceLevel.NONE

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result = asdict(self)
        result["confidence_level"] = self.confidence_level.value
        return result


class FlowSelector:
    """
    Intelligent flow selector that recommends the best flow for a given task.

    Loads flow definitions from SKILL.md files, parses YAML frontmatter,
    and matches tasks against trigger patterns to recommend appropriate flows.

    Example:
        >>> selector = FlowSelector()
        >>> result = selector.select("implement email validation")
        >>> print(result.recommended_flow)
        "feature-implementation"
    """

    def __init__(self, flows_path: Optional[str] = None):
        """
        Initialize the FlowSelector.

        Args:
            flows_path: Path to flows directory. Defaults to
                        PROJECT_ROOT/.claude/skills/flows
        """
        if flows_path:
            self.flows_path = Path(flows_path)
        else:
            self.flows_path = PROJECT_ROOT / DEFAULT_FLOWS_PATH

        self.flows: Dict[str, FlowMetadata] = {}
        self._load_errors: List[str] = []
        self._load_flows()

    def _load_flows(self) -> None:
        """Load all flow definitions from SKILL.md files."""
        if not self.flows_path.exists():
            self._load_errors.append(f"Flows directory not found: {self.flows_path}")
            return

        # Find all SKILL.md files in subdirectories
        for flow_dir in self.flows_path.iterdir():
            if not flow_dir.is_dir():
                continue

            skill_file = flow_dir / "SKILL.md"
            if not skill_file.exists():
                continue

            try:
                metadata = self._parse_flow_file(skill_file)
                if metadata:
                    self.flows[metadata.name] = metadata
            except Exception as e:
                self._load_errors.append(f"Error loading {skill_file}: {e}")

    def _parse_flow_file(self, file_path: Path) -> Optional[FlowMetadata]:
        """
        Parse a flow SKILL.md file and extract metadata from YAML frontmatter.

        Args:
            file_path: Path to the SKILL.md file

        Returns:
            FlowMetadata if successfully parsed, None otherwise
        """
        content = file_path.read_text(encoding="utf-8")

        # Extract YAML frontmatter (between --- markers)
        frontmatter_match = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
        if not frontmatter_match:
            return None

        frontmatter_text = frontmatter_match.group(1)

        try:
            data = yaml.safe_load(frontmatter_text)
            if not isinstance(data, dict):
                return None
        except yaml.YAMLError as e:
            self._load_errors.append(f"YAML parse error in {file_path}: {e}")
            return None

        # Extract trigger information
        trigger = data.get("trigger", {})
        trigger_pattern = ""
        trigger_explicit = True
        trigger_event = ""

        if isinstance(trigger, dict):
            trigger_pattern = trigger.get("pattern", "")
            trigger_explicit = trigger.get("explicit", True)
            trigger_event = trigger.get("event", "")

        # Build metadata object
        return FlowMetadata(
            name=data.get("name", file_path.parent.name),
            version=data.get("version", "1.0.0"),
            description=self._clean_multiline(data.get("description", "")),
            trigger_pattern=trigger_pattern,
            trigger_explicit=trigger_explicit,
            trigger_event=trigger_event,
            commander_intent=self._clean_multiline(data.get("commander_intent", "")),
            applicable_agents=data.get("applicable_agents", []),
            estimated_duration=data.get("estimated_duration", ""),
            complexity=data.get("complexity", "medium"),
            file_path=str(file_path),
            status=data.get("status", "TESTING"),
        )

    def _clean_multiline(self, text: Any) -> str:
        """Clean multiline YAML text (strip, normalize whitespace)."""
        if not text:
            return ""
        if not isinstance(text, str):
            return str(text)
        # Normalize whitespace while preserving sentence structure
        lines = text.strip().split("\n")
        cleaned = " ".join(line.strip() for line in lines if line.strip())
        return cleaned

    def select(self, task: str) -> FlowRecommendation:
        """
        Select the best flow for a given task description.

        Args:
            task: Natural language description of the task

        Returns:
            FlowRecommendation with best match and alternatives
        """
        # Handle empty/invalid input
        if not task or not task.strip():
            return FlowRecommendation(
                recommended_flow="",
                confidence=0.0,
                reasoning="No task description provided",
                metadata={"error": "empty_task"}
            )

        task = task.strip()

        # Collect all matches with scores
        matches: List[Tuple[str, float, FlowMetadata]] = []

        for name, flow in self.flows.items():
            matched, score = flow.matches_task(task)
            if matched:
                matches.append((name, score, flow))

        # Sort by score descending
        matches.sort(key=lambda x: x[1], reverse=True)

        # No matches found
        if not matches:
            return FlowRecommendation(
                recommended_flow="",
                confidence=0.0,
                reasoning=f"No flow pattern matched task: '{task[:50]}...' " +
                          f"(searched {len(self.flows)} flows)",
                metadata={
                    "task_preview": task[:100],
                    "flows_searched": list(self.flows.keys()),
                }
            )

        # Build recommendation from best match
        best_name, best_score, best_flow = matches[0]

        # Build alternatives list (excluding best)
        alternatives = []
        for name, score, flow in matches[1:4]:  # Top 3 alternatives
            alternatives.append({
                "flow": name,
                "confidence": round(score, 3),
                "description": flow.description[:100] + "..." if len(flow.description) > 100 else flow.description,
            })

        # Build reasoning
        reasoning = self._build_reasoning(task, best_flow, best_score, len(matches))

        return FlowRecommendation(
            recommended_flow=best_name,
            confidence=round(best_score, 3),
            reasoning=reasoning,
            alternatives=alternatives,
            metadata={
                "task_preview": task[:100],
                "pattern_matched": best_flow.trigger_pattern,
                "flow_path": best_flow.file_path,
                "estimated_duration": best_flow.estimated_duration,
                "complexity": best_flow.complexity,
                "total_matches": len(matches),
            }
        )

    def _build_reasoning(self, task: str, flow: FlowMetadata, score: float,
                         total_matches: int) -> str:
        """Build human-readable reasoning for the recommendation."""
        parts = []

        # Confidence statement
        if score >= 0.7:
            parts.append(f"High confidence match ({score:.0%}) for '{flow.name}'.")
        elif score >= 0.4:
            parts.append(f"Medium confidence match ({score:.0%}) for '{flow.name}'.")
        else:
            parts.append(f"Low confidence match ({score:.0%}) for '{flow.name}'.")

        # Pattern explanation
        if flow.trigger_pattern:
            parts.append(f"Task matched pattern: /{flow.trigger_pattern}/")

        # Commander intent preview
        if flow.commander_intent:
            intent_preview = flow.commander_intent[:150]
            if len(flow.commander_intent) > 150:
                intent_preview += "..."
            parts.append(f"Flow intent: {intent_preview}")

        # Alternatives note
        if total_matches > 1:
            parts.append(f"({total_matches - 1} alternative flow(s) also matched)")

        return " ".join(parts)

    def list_flows(self) -> List[Dict[str, Any]]:
        """
        List all loaded flows with their metadata.

        Returns:
            List of flow metadata dictionaries
        """
        result = []
        for name, flow in sorted(self.flows.items()):
            result.append({
                "name": name,
                "description": flow.description[:100] + "..." if len(flow.description) > 100 else flow.description,
                "pattern": flow.trigger_pattern,
                "complexity": flow.complexity,
                "duration": flow.estimated_duration,
                "status": flow.status,
            })
        return result

    def get_flow(self, name: str) -> Optional[FlowMetadata]:
        """
        Get metadata for a specific flow by name.

        Args:
            name: Flow name

        Returns:
            FlowMetadata if found, None otherwise
        """
        return self.flows.get(name)

    @property
    def load_errors(self) -> List[str]:
        """Get any errors that occurred during flow loading."""
        return self._load_errors.copy()


# === CLI Interface ===

def main():
    """
    CLI interface for flow selection.

    Usage:
        python flow_selector.py "implement email validation"
        python flow_selector.py --list
        python flow_selector.py --test
    """
    import argparse

    parser = argparse.ArgumentParser(
        description="Select the best flow for a task",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python flow_selector.py "implement email validation"
  python flow_selector.py "fix bug in validators.py"
  python flow_selector.py "explore caching options"
  python flow_selector.py --list
  python flow_selector.py --test
        """
    )
    parser.add_argument(
        "task",
        nargs="?",
        help="Task description to match against flows"
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List all available flows"
    )
    parser.add_argument(
        "--test",
        action="store_true",
        help="Run test scenarios"
    )
    parser.add_argument(
        "--flows-path",
        help="Path to flows directory (default: .claude/skills/flows)"
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print JSON output"
    )

    args = parser.parse_args()

    # Initialize selector
    selector = FlowSelector(flows_path=args.flows_path)

    # Report any load errors
    if selector.load_errors:
        print("Warning: Some flows failed to load:", file=sys.stderr)
        for error in selector.load_errors:
            print(f"  - {error}", file=sys.stderr)
        print("", file=sys.stderr)

    # Handle --test
    if args.test:
        run_test_scenarios(selector)
        return

    # Handle --list
    if args.list:
        flows = selector.list_flows()
        if args.pretty:
            print(json.dumps(flows, indent=2))
        else:
            print(json.dumps(flows))
        return

    # Handle task selection
    if not args.task:
        parser.print_help()
        sys.exit(1)

    result = selector.select(args.task)
    output = result.to_dict()

    if args.pretty:
        print(json.dumps(output, indent=2))
    else:
        print(json.dumps(output))


def run_test_scenarios(selector: FlowSelector):
    """Run test scenarios to validate flow selection."""
    print("=" * 60)
    print("FLOW SELECTOR TEST SCENARIOS")
    print("=" * 60)
    print(f"\nLoaded {len(selector.flows)} flows: {list(selector.flows.keys())}\n")

    # Test scenarios with expected results
    # Note: Expectations based on actual trigger patterns in flow SKILL.md files (v2, 2026-01-16):
    # - feature-implementation: Matches implement/build/create/add/develop followed by something
    #   Excludes tasks containing flow/ceremony/research/session/explore/investigate
    # - research-to-design: Matches research/explore/investigate/analyze/understand/evaluate/compare/study/survey
    # - session-start: "^(wake up|session start|good morning|starting session)"
    # - meta-flow-creation: "create.*flow|new.*flow|formalize.*pattern|extract.*flow"
    # - deep-ceremony: "/deep-ceremony.*|ceremony.*significant|identity.*reflection"
    scenarios = [
        # feature-implementation tests - now matches simpler phrases
        ("implement email validation", "feature-implementation", 0.5),
        ("build user dashboard", "feature-implementation", 0.5),
        ("create API endpoint for users", "feature-implementation", 0.5),
        ("add logging to the system", "feature-implementation", 0.5),
        ("develop payment integration", "feature-implementation", 0.5),
        # research-to-design tests - now matches single keywords
        ("research caching strategies", "research-to-design-flow", 0.5),
        ("explore API options", "research-to-design-flow", 0.5),
        ("investigate memory leak", "research-to-design-flow", 0.5),
        ("analyze performance bottleneck", "research-to-design-flow", 0.5),
        ("understand the architecture", "research-to-design-flow", 0.5),
        # session-start tests
        ("wake up", "session-start-flow", 0.5),
        ("session start", "session-start-flow", 0.5),
        ("good morning", "session-start-flow", 0.5),
        ("starting session now", "session-start-flow", 0.5),
        # meta-flow-creation tests
        ("create new flow for testing", "meta-flow-creation", 0.5),
        ("formalize pattern into flow", "meta-flow-creation", 0.5),
        ("extract flow from workflow", "meta-flow-creation", 0.5),
        # deep-ceremony tests
        ("/deep-ceremony vocabulary exchange", "deep-ceremony-flow", 0.5),
        ("ceremony significant identity moment", "deep-ceremony-flow", 0.5),
        ("identity reflection on growth", "deep-ceremony-flow", 0.5),
        # exclusion tests - should route to research, NOT feature-implementation
        ("research flow architecture", "research-to-design-flow", 0.5),  # "research" wins
        ("explore ceremony patterns", "research-to-design-flow", 0.5),  # "explore" wins
        # no match cases
        ("fix bug in validators.py", "", 0.0),  # No matching pattern for bug fixes
        ("do something", "", 0.0),  # Too vague
        ("", "", 0.0),  # Empty
    ]

    passed = 0
    failed = 0

    for task, expected_flow, min_confidence in scenarios:
        result = selector.select(task)

        # Check if recommendation matches expected
        flow_match = (result.recommended_flow == expected_flow)

        # For non-empty expected flows, check confidence threshold
        if expected_flow:
            confidence_ok = result.confidence >= min_confidence
        else:
            # For "no match" cases, confidence should be 0
            confidence_ok = result.confidence == 0.0

        overall_pass = flow_match and confidence_ok

        if overall_pass:
            passed += 1
            status = "PASS"
        else:
            failed += 1
            status = "FAIL"

        print(f"Task: \"{task[:40]}...\"" if len(task) > 40 else f"Task: \"{task}\"")
        print(f"  Expected: {expected_flow or '(no match)'} (>= {min_confidence:.0%})")
        print(f"  Got: {result.recommended_flow or '(no match)'} ({result.confidence:.0%})")
        print(f"  Status: {status}")
        if not overall_pass:
            print(f"  Reasoning: {result.reasoning[:80]}...")
        print()

    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed out of {len(scenarios)} scenarios")
    print("=" * 60)

    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
