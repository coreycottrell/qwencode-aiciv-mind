#!/usr/bin/env python3
"""
Memory Compliance Module - ADR-048 Memory System Excellence

Validates agent responses for memory search compliance and generates
coaching data for the primary-helper agent.

This module implements Phase 1.1 of ADR-048: Three-Layer Mandatory Access
- Layer 2: Output verification (this module)
- Layer 3: Compliance tracking (coaching data generation)

Quality Levels (from ADR-048-TEST-SCENARIOS):
- missing: No "Memory Search Results" section at all
- minimal: Section exists but boilerplate only
- adequate: Searched at least one registry, found something
- excellent: Searched multiple registries, applied findings

Created: 2026-01-16
Author: coder-agent (ADR-048 implementation)

Memory Search Results:
- Searched: Memory server for "memory compliance validation parsing"
- Searched: .claude/memory/agent-learnings/coder/ for patterns
- Found: pydantic-validation.md, memory-first-protocol-enforcement-20251119.md
- Applying: Pydantic dataclasses, regex patterns from stop_delegation_audit.py
"""

import re
from dataclasses import dataclass, field
from enum import Enum
from typing import List, Dict, Any, Optional, Tuple


class QualityLevel(str, Enum):
    """Quality levels for memory search compliance."""
    MISSING = "missing"
    MINIMAL = "minimal"
    ADEQUATE = "adequate"
    EXCELLENT = "excellent"


@dataclass
class MemorySearchValidation:
    """
    Result of validating the "Memory Search Results" section in an agent response.

    Attributes:
        found: Whether the section exists in the response
        quality: Quality level (missing/minimal/adequate/excellent)
        registries_searched: List of registries/paths that were searched
        entries_found: Number of memory entries found
        entries_applied: Number of entries actually applied
        issues: List of issues found during validation
        raw_section: The raw text of the Memory Search Results section
    """
    found: bool
    quality: QualityLevel
    registries_searched: List[str] = field(default_factory=list)
    entries_found: int = 0
    entries_applied: int = 0
    issues: List[str] = field(default_factory=list)
    raw_section: str = ""


@dataclass
class ComplianceResult:
    """
    Aggregated compliance result across all agent responses in a transcript.

    Attributes:
        total_responses: Number of agent responses analyzed
        compliant_responses: Number of responses with adequate+ compliance
        validations: List of individual validation results
        overall_score: Weighted compliance score (0.0-1.0)
        overall_quality: Aggregated quality level
    """
    total_responses: int = 0
    compliant_responses: int = 0
    validations: List[MemorySearchValidation] = field(default_factory=list)
    overall_score: float = 0.0
    overall_quality: QualityLevel = QualityLevel.MISSING


# Patterns for detecting Memory Search Results section
# Note: Allow optional leading whitespace to handle indented content
SECTION_PATTERNS = [
    r"(?:^|\n)\s*#{1,3}\s*Memory\s+Search\s+Results?\s*\n",  # ## Memory Search Results
    r"(?:^|\n)\s*\*\*Memory\s+Search\s+Results?\*\*\s*\n",   # **Memory Search Results**
    r"(?:^|\n)\s*Memory\s+Search\s+Results?:\s*\n",          # Memory Search Results:
]

# Known registry paths that indicate proper memory search
KNOWN_REGISTRIES = [
    r"memories/skills/registry\.json",
    r"memories/agents/agent_registry\.json",
    r"\.claude/CLAUDE-AGENTS\.md",
    r"memories/sessions/",
    r"\.claude/memory/agent-learnings/",
    r"memories/knowledge/",
    r"memories/agents/",
    r"\.claude/memory/",
]

# Patterns for extracting searched content
# Note: Handle various formats including "Auto-injected", "Additional search", etc.
SEARCHED_PATTERN = r"[-*]\s*(?:Searched|Search(?:ed)?|Auto-?injected|Additional\s+search)\s*:\s*(.+?)(?:\n|$)"
FOUND_PATTERN = r"[-*]\s*(?:Found|Results?)\s*:\s*(.+?)(?:\n|$)"
APPLYING_PATTERN = r"[-*]\s*(?:Applying|Applied|Using)\s*:\s*(.+?)(?:\n|$)"

# Boilerplate indicators (minimal quality)
BOILERPLATE_INDICATORS = [
    r"(?:searched|found|applying)\s*:\s*(?:none|nothing|n/a|no\s+matches?)",
    r"no\s+(?:relevant|prior)\s+(?:work|memories?|entries)",
    r"skipped\s+because",
    r"not\s+applicable",
]


def extract_memory_search_section(response_text: str) -> Tuple[bool, str]:
    """
    Extract the Memory Search Results section from agent response.

    Args:
        response_text: Full text of the agent response

    Returns:
        Tuple of (section_found: bool, section_content: str)
    """
    for pattern in SECTION_PATTERNS:
        match = re.search(pattern, response_text, re.IGNORECASE | re.MULTILINE)
        if match:
            # Extract content until next section or end
            start = match.end()
            # Look for next section header (##, ###, **...**) or end
            next_section = re.search(
                r"\n#{1,3}\s+[A-Z]|\n\*\*[A-Z]|\n---\n",
                response_text[start:],
                re.MULTILINE
            )
            if next_section:
                end = start + next_section.start()
            else:
                # Take up to 1000 chars or end of text
                end = min(start + 1000, len(response_text))

            section_content = response_text[start:end].strip()
            return True, section_content

    return False, ""


def extract_registries_searched(section_content: str) -> List[str]:
    """
    Extract list of registries/paths searched from the section content.

    Args:
        section_content: The Memory Search Results section text

    Returns:
        List of registry paths/identifiers searched
    """
    registries = []

    # Find "Searched:" / "Auto-injected:" / "Additional search:" line(s)
    matches = re.findall(SEARCHED_PATTERN, section_content, re.IGNORECASE | re.MULTILINE)

    for match in matches:
        # Parse the searched content for known registry patterns
        for registry_pattern in KNOWN_REGISTRIES:
            if re.search(registry_pattern, match, re.IGNORECASE):
                # Extract the actual path
                path_match = re.search(registry_pattern, match, re.IGNORECASE)
                if path_match:
                    registries.append(path_match.group(0))

        # Capture paths that look like file paths (must start with / or . or contain /)
        # This filters out plain keywords like "file operations"
        file_paths = re.findall(r'(?:/[a-zA-Z0-9_./-]+|\.?[a-zA-Z0-9_]+/[a-zA-Z0-9_./-]+|[a-zA-Z0-9_./]+\.(?:json|md|py))', match)
        for fp in file_paths:
            # Additional validation: must contain / or be a known file extension
            if '/' in fp or fp.endswith(('.json', '.md', '.py')):
                registries.append(fp)

        # Count explicit memory references (ADR-xxx, memories, etc.) as "registries searched"
        if re.search(r'\d+\s+memor(?:y|ies)', match.lower()):
            # "3 memories" mentioned = searched something
            registries.append("auto-injected-memories")

    # Deduplicate while preserving order
    seen = set()
    unique_registries = []
    for r in registries:
        r_normalized = r.lower()
        if r_normalized not in seen:
            seen.add(r_normalized)
            unique_registries.append(r)

    return unique_registries


def count_entries_found(section_content: str) -> int:
    """
    Count number of memory entries found based on section content.

    Args:
        section_content: The Memory Search Results section text

    Returns:
        Estimated number of entries found
    """
    count = 0

    # Find "Found:" line(s)
    matches = re.findall(FOUND_PATTERN, section_content, re.IGNORECASE | re.MULTILINE)

    for match in matches:
        match_lower = match.lower()

        # Check for explicit "none" or "no matches"
        if re.search(r"(?:none|no\s+matches?|nothing)", match_lower):
            continue

        # Count explicit numbers - more flexible pattern
        # Matches: "5 additional relevant memories", "3 entries", "2 matches"
        number_match = re.search(r"(\d+)\s+(?:\w+\s+)*(?:memories?|entries|matches|results?|relevant)", match_lower)
        if number_match:
            count += int(number_match.group(1))
            continue

        # Count list items (bullet points)
        list_items = re.findall(r"^\s*[-*]\s+", match, re.MULTILINE)
        if list_items:
            count += len(list_items)
            continue

        # If there's non-boilerplate content, assume at least 1
        if not re.search(r"|".join(BOILERPLATE_INDICATORS), match_lower):
            count += 1

    # Also scan the section for memory references
    memory_refs = re.findall(
        r"(?:ADR-\d+|memory-\d+|\d{8}-[a-z]+-[a-z0-9-]+\.md)",
        section_content,
        re.IGNORECASE
    )
    if memory_refs:
        count = max(count, len(set(memory_refs)))

    # Also check for "X memories" in Auto-injected line
    auto_injected = re.search(r"Auto-?injected[^:]*:\s*(\d+)\s+memor", section_content, re.IGNORECASE)
    if auto_injected:
        count = max(count, int(auto_injected.group(1)))

    return count


def count_entries_applied(section_content: str) -> int:
    """
    Count number of memory entries actually applied.

    Args:
        section_content: The Memory Search Results section text

    Returns:
        Estimated number of entries applied
    """
    count = 0

    # Find "Applying:" line(s)
    matches = re.findall(APPLYING_PATTERN, section_content, re.IGNORECASE | re.MULTILINE)

    for match in matches:
        match_lower = match.lower()

        # Check for explicit "none" or "not applicable"
        if re.search(r"(?:none|n/a|not\s+applicable|nothing)", match_lower):
            continue

        # Count explicit references
        refs = re.findall(r"[A-Za-z]+-[A-Za-z0-9-]+", match)
        if refs:
            count += len(refs)
            continue

        # If there's substantive content, assume at least 1
        if len(match.strip()) > 20:
            count += 1

    return count


def detect_boilerplate(section_content: str) -> bool:
    """
    Detect if the section content is just boilerplate (minimal quality).

    Args:
        section_content: The Memory Search Results section text

    Returns:
        True if content appears to be boilerplate
    """
    content_lower = section_content.lower()

    # Check for boilerplate indicators
    boilerplate_matches = 0
    for pattern in BOILERPLATE_INDICATORS:
        if re.search(pattern, content_lower):
            boilerplate_matches += 1

    # Check content length and substance
    # Strip common formatting and count substantive words
    stripped = re.sub(r"[-*:#\[\]`]", " ", section_content)
    words = [w for w in stripped.split() if len(w) > 3]

    # Boilerplate: multiple boilerplate indicators OR very short content
    if boilerplate_matches >= 2:
        return True
    if len(words) < 10 and boilerplate_matches >= 1:
        return True

    return False


def count_search_types(section_content: str) -> int:
    """
    Count distinct types of memory search performed.

    Types include: Auto-injected, Searched, Additional search, etc.

    Args:
        section_content: The Memory Search Results section text

    Returns:
        Number of distinct search types mentioned
    """
    search_types = 0

    # Check for various search type indicators
    if re.search(r"auto-?inject", section_content, re.IGNORECASE):
        search_types += 1
    if re.search(r"[-*]\s*searched\s*:", section_content, re.IGNORECASE):
        search_types += 1
    if re.search(r"additional\s+search", section_content, re.IGNORECASE):
        search_types += 1
    if re.search(r"query|queried", section_content, re.IGNORECASE):
        search_types += 1

    return search_types


def validate_memory_search_results(response_text: str) -> MemorySearchValidation:
    """
    Validate the Memory Search Results section in an agent response.

    Quality levels (from ADR-048-TEST-SCENARIOS):
    - missing: No "Memory Search Results" section at all
    - minimal: Section exists but boilerplate only ("searched: none, found: none")
    - adequate: Searched at least one registry, found something
    - excellent: Multiple search types or registries, found entries, applied findings

    Args:
        response_text: Full text of the agent response

    Returns:
        MemorySearchValidation with validation results
    """
    # Extract section
    found, section_content = extract_memory_search_section(response_text)

    if not found:
        return MemorySearchValidation(
            found=False,
            quality=QualityLevel.MISSING,
            issues=["No 'Memory Search Results' section found in response"]
        )

    # Extract components
    registries = extract_registries_searched(section_content)
    entries_found = count_entries_found(section_content)
    entries_applied = count_entries_applied(section_content)
    is_boilerplate = detect_boilerplate(section_content)
    search_types = count_search_types(section_content)

    # Deduplicate registries - count unique base paths only
    # e.g., ".claude/memory/agent-learnings/coder/" and ".claude/memory/" are related
    unique_base_registries = set()
    for r in registries:
        # Take the first meaningful path component
        if r.startswith('.'):
            parts = r.split('/')[:3]  # e.g., ['.claude', 'memory', 'agent-learnings']
            base = '/'.join(parts) + '/' if len(parts) > 1 else r
        elif r.startswith('/'):
            parts = r.split('/')[:4]
            base = '/'.join(parts) + '/'
        else:
            base = r
        unique_base_registries.add(base.lower())

    num_unique_registries = len(unique_base_registries)

    # Determine quality level
    # Key criteria for excellent (from ADR-048-TEST-SCENARIOS):
    # - Multiple search types (auto-injected + additional search)
    # - OR found many entries (>=3) and applied findings
    # - Demonstrates deep engagement, not just "searched one place, found one thing"
    issues = []

    if is_boilerplate:
        quality = QualityLevel.MINIMAL
        issues.append("Section appears to be boilerplate with no substantive search")
    elif num_unique_registries == 0 and entries_found == 0:
        quality = QualityLevel.MINIMAL
        issues.append("No registries searched and no entries found")
    elif search_types >= 2 and entries_applied >= 1:
        # Excellent: Multiple search types (auto-injected + additional), plus applied
        quality = QualityLevel.EXCELLENT
    elif entries_found >= 3 and entries_applied >= 1:
        # Also excellent: Found many entries and applied some
        quality = QualityLevel.EXCELLENT
    elif num_unique_registries >= 1 or entries_found >= 1:
        quality = QualityLevel.ADEQUATE
        if entries_applied == 0:
            issues.append("Found entries but none marked as applied")
    else:
        quality = QualityLevel.MINIMAL
        issues.append("Insufficient memory search evidence")

    return MemorySearchValidation(
        found=True,
        quality=quality,
        registries_searched=registries,
        entries_found=entries_found,
        entries_applied=entries_applied,
        issues=issues,
        raw_section=section_content
    )


def calculate_compliance_score(validation: MemorySearchValidation) -> float:
    """
    Calculate a numeric compliance score from validation result.

    Scoring:
    - 0.0 = missing
    - 0.3 = minimal boilerplate
    - 0.6 = adequate
    - 1.0 = excellent

    Args:
        validation: The validation result to score

    Returns:
        Float score between 0.0 and 1.0
    """
    quality_scores = {
        QualityLevel.MISSING: 0.0,
        QualityLevel.MINIMAL: 0.3,
        QualityLevel.ADEQUATE: 0.6,
        QualityLevel.EXCELLENT: 1.0,
    }

    base_score = quality_scores[validation.quality]

    # Bonus for extra registries searched (up to 0.1)
    registry_bonus = min(0.1, len(validation.registries_searched) * 0.02)

    # Bonus for entries applied (up to 0.1)
    applied_bonus = min(0.1, validation.entries_applied * 0.03)

    # Only apply bonuses if base quality is adequate or better
    if validation.quality in (QualityLevel.ADEQUATE, QualityLevel.EXCELLENT):
        score = base_score + registry_bonus + applied_bonus
    else:
        score = base_score

    return min(1.0, round(score, 3))


def extract_agent_responses(messages: List[Dict[str, Any]]) -> List[str]:
    """
    Extract text content from assistant messages in a transcript.

    Handles various message formats including Claude transcript format.

    Args:
        messages: List of message dictionaries from transcript

    Returns:
        List of response text strings
    """
    responses = []

    for msg in messages:
        if not isinstance(msg, dict):
            continue

        # Check role - only process assistant messages
        role = msg.get('role', '')
        if 'message' in msg and isinstance(msg['message'], dict):
            role = msg['message'].get('role', role)

        if role != 'assistant':
            continue

        # Extract text content
        text_parts = []

        # Direct text field
        if 'text' in msg:
            text_parts.append(msg['text'])

        # Content field (string or list)
        content = msg.get('content', [])
        if isinstance(content, str):
            text_parts.append(content)
        elif isinstance(content, list):
            for block in content:
                if isinstance(block, dict) and block.get('type') == 'text':
                    text_parts.append(block.get('text', ''))

        # Nested message content (Claude transcript format)
        if 'message' in msg and isinstance(msg['message'], dict):
            nested_content = msg['message'].get('content', [])
            if isinstance(nested_content, str):
                text_parts.append(nested_content)
            elif isinstance(nested_content, list):
                for block in nested_content:
                    if isinstance(block, dict) and block.get('type') == 'text':
                        text_parts.append(block.get('text', ''))

        if text_parts:
            responses.append('\n'.join(text_parts))

    return responses


def check_agent_memory_compliance(messages: List[Dict[str, Any]]) -> ComplianceResult:
    """
    Analyze transcript messages for memory compliance.

    Returns aggregated compliance across all agent responses.

    Args:
        messages: List of message dictionaries from transcript

    Returns:
        ComplianceResult with aggregated analysis
    """
    responses = extract_agent_responses(messages)

    if not responses:
        return ComplianceResult(
            total_responses=0,
            compliant_responses=0,
            validations=[],
            overall_score=0.0,
            overall_quality=QualityLevel.MISSING
        )

    validations = []
    compliant_count = 0
    total_score = 0.0

    for response_text in responses:
        validation = validate_memory_search_results(response_text)
        validations.append(validation)

        score = calculate_compliance_score(validation)
        total_score += score

        if validation.quality in (QualityLevel.ADEQUATE, QualityLevel.EXCELLENT):
            compliant_count += 1

    # Calculate overall quality based on average score
    avg_score = total_score / len(responses) if responses else 0.0

    if avg_score >= 0.8:
        overall_quality = QualityLevel.EXCELLENT
    elif avg_score >= 0.5:
        overall_quality = QualityLevel.ADEQUATE
    elif avg_score >= 0.2:
        overall_quality = QualityLevel.MINIMAL
    else:
        overall_quality = QualityLevel.MISSING

    return ComplianceResult(
        total_responses=len(responses),
        compliant_responses=compliant_count,
        validations=validations,
        overall_score=round(avg_score, 3),
        overall_quality=overall_quality
    )


def generate_compliance_coaching(result: ComplianceResult) -> Dict[str, Any]:
    """
    Generate coaching recommendations for primary-helper based on compliance result.

    Includes specific guidance based on what was missing.

    Args:
        result: ComplianceResult from check_agent_memory_compliance

    Returns:
        Dictionary with coaching data for primary-helper consumption
    """
    coaching = {
        "compliance_score": result.overall_score,
        "quality_level": result.overall_quality.value,
        "total_responses": result.total_responses,
        "compliant_responses": result.compliant_responses,
        "compliance_rate": (
            result.compliant_responses / result.total_responses
            if result.total_responses > 0 else 0.0
        ),
        "issues": [],
        "recommendations": [],
        "exemplary_patterns": [],
    }

    # Aggregate issues from all validations
    all_issues = []
    for v in result.validations:
        all_issues.extend(v.issues)
    coaching["issues"] = list(set(all_issues))

    # Generate recommendations based on quality
    if result.overall_quality == QualityLevel.MISSING:
        coaching["recommendations"].extend([
            "CRITICAL: Agents must include 'Memory Search Results' section in all responses",
            "Add mandatory memory search step to agent workflow",
            "Provide template format for Memory Search Results section",
            "Consider blocking agent responses without memory search documentation",
        ])

    elif result.overall_quality == QualityLevel.MINIMAL:
        coaching["recommendations"].extend([
            "Agents are providing boilerplate memory search sections",
            "Require specific registry paths in search documentation",
            "Coach agents to engage with auto-injected memories",
            "Check if relevant memories exist but were missed",
        ])

    elif result.overall_quality == QualityLevel.ADEQUATE:
        coaching["recommendations"].extend([
            "Memory compliance is acceptable but can improve",
            "Encourage searching multiple registries",
            "Emphasize documenting which findings were applied",
            "Recognize progress toward excellent compliance",
        ])

    else:  # EXCELLENT
        coaching["recommendations"].extend([
            "Memory compliance is excellent - maintain this standard",
            "Consider documenting agent's approach as exemplary pattern",
            "Share successful memory search patterns with other agents",
        ])

    # Identify exemplary patterns (excellent validations)
    for i, v in enumerate(result.validations):
        if v.quality == QualityLevel.EXCELLENT:
            coaching["exemplary_patterns"].append({
                "response_index": i,
                "registries_searched": v.registries_searched,
                "entries_found": v.entries_found,
                "entries_applied": v.entries_applied,
            })

    return coaching


def format_compliance_report(result: ComplianceResult, include_details: bool = False) -> str:
    """
    Format compliance result as human-readable report.

    Args:
        result: ComplianceResult to format
        include_details: Whether to include per-response details

    Returns:
        Formatted report string
    """
    lines = [
        "=" * 60,
        "MEMORY COMPLIANCE REPORT",
        "=" * 60,
        f"Overall Score: {result.overall_score:.2f} ({result.overall_quality.value.upper()})",
        f"Responses Analyzed: {result.total_responses}",
        f"Compliant Responses: {result.compliant_responses}",
        f"Compliance Rate: {result.compliant_responses}/{result.total_responses}",
        "-" * 60,
    ]

    if include_details and result.validations:
        lines.append("Per-Response Analysis:")
        for i, v in enumerate(result.validations):
            lines.append(f"\n  Response {i + 1}:")
            lines.append(f"    Quality: {v.quality.value.upper()}")
            lines.append(f"    Section Found: {v.found}")
            lines.append(f"    Registries: {v.registries_searched or 'None'}")
            lines.append(f"    Entries Found: {v.entries_found}")
            lines.append(f"    Entries Applied: {v.entries_applied}")
            if v.issues:
                lines.append(f"    Issues: {', '.join(v.issues)}")

    # Generate coaching recommendations
    coaching = generate_compliance_coaching(result)

    if coaching["recommendations"]:
        lines.append("\n" + "-" * 60)
        lines.append("COACHING RECOMMENDATIONS:")
        for rec in coaching["recommendations"]:
            lines.append(f"  - {rec}")

    lines.append("=" * 60)

    return "\n".join(lines)


# === CLI Interface ===

def main():
    """
    CLI interface for testing memory compliance validation.

    Usage:
        python memory_compliance.py <response_text_file>
        python memory_compliance.py --test
    """
    import sys
    import json

    if len(sys.argv) < 2:
        print("Usage: python memory_compliance.py <response_text_file>")
        print("       python memory_compliance.py --test")
        sys.exit(1)

    if sys.argv[1] == "--test":
        # Run test cases from ADR-048-TEST-SCENARIOS
        run_test_scenarios()
        return

    # Read response text from file
    filepath = sys.argv[1]
    try:
        with open(filepath, 'r') as f:
            content = f.read()

        # Try to parse as JSON (transcript format)
        try:
            data = json.loads(content)
            if isinstance(data, list):
                messages = data
            elif isinstance(data, dict) and 'messages' in data:
                messages = data['messages']
            else:
                # Treat as plain text response
                messages = [{"role": "assistant", "content": content}]
        except json.JSONDecodeError:
            # Treat as plain text response
            messages = [{"role": "assistant", "content": content}]

        result = check_agent_memory_compliance(messages)
        print(format_compliance_report(result, include_details=True))

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


def run_test_scenarios():
    """Run test scenarios from ADR-048-TEST-SCENARIOS."""

    print("=" * 60)
    print("MEMORY COMPLIANCE TEST SCENARIOS")
    print("=" * 60)

    # Scenario 1: Missing section entirely
    scenario_1 = """
    I've implemented the atomic file writer. Here's the code:

    def atomic_write(path, content):
        # Write to temp file, then rename
        pass

    The implementation follows best practices.
    """

    # Scenario 2: Minimal boilerplate
    scenario_2 = """
    ## Memory Search Results
    - Searched: memories
    - Found: nothing relevant
    - Applying: none

    I've completed the research task on Solana token-2022 extensions.
    """

    # Scenario 3: Excellent compliance
    scenario_3 = """
    ## Memory Search Results
    - Auto-injected: 3 memories (ADR-013 MCP boundaries, session ledger pattern, agent state management)
    - Additional search: "session persistence", "state machine", "boundary conditions"
    - Found: 5 additional relevant memories
    - Applying: ADR-013 defines session scope; ledger pattern informs checkpoint design
    - Discarded: 2 memories were outdated (superseded by ADR-013)

    Based on the memory search, I've designed the session boundary management system.
    """

    # Scenario 4: Adequate compliance
    scenario_4 = """
    ## Memory Search Results
    - Searched: .claude/memory/agent-learnings/coder/ for "file operations"
    - Found: 20251119-atomic-file-write.md - discusses temp file + rename pattern
    - Applying: Using the atomic write pattern from prior implementation

    Implementation complete.
    """

    scenarios = [
        ("1: Missing Section", scenario_1, QualityLevel.MISSING),
        ("2: Minimal Boilerplate", scenario_2, QualityLevel.MINIMAL),
        ("3: Excellent Compliance", scenario_3, QualityLevel.EXCELLENT),
        ("4: Adequate Compliance", scenario_4, QualityLevel.ADEQUATE),
    ]

    all_passed = True
    for name, text, expected in scenarios:
        validation = validate_memory_search_results(text)
        score = calculate_compliance_score(validation)
        passed = validation.quality == expected

        status = "PASS" if passed else "FAIL"
        print(f"\nScenario {name}:")
        print(f"  Expected: {expected.value}")
        print(f"  Got: {validation.quality.value}")
        print(f"  Score: {score:.2f}")
        print(f"  Status: {status}")

        if not passed:
            all_passed = False
            print(f"  Issues: {validation.issues}")

    print("\n" + "=" * 60)
    if all_passed:
        print("ALL TESTS PASSED")
    else:
        print("SOME TESTS FAILED")
    print("=" * 60)


if __name__ == "__main__":
    main()
