#!/usr/bin/env python3
"""
Memory Quality Control for AI-CIV Collective

Implements quality scoring, write triggers, and deduplication.
Ensures only high-value memories are written and maintained.

Author: The Conductor & Doc Synthesizer
Version: 1.0.0
"""

import re
import difflib
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass


@dataclass
class QualityScore:
    """Quality scoring result."""
    reusability: int  # 1-3
    impact: int  # 1-3
    clarity: int  # 1-3
    evidence: int  # 1-3
    novelty: int  # 1-3
    total: int  # 0-33 (weighted sum)
    passed: bool  # >= 18/33 threshold
    tier: str  # excellent/good/acceptable/poor

    def __str__(self):
        return f"{self.total}/33 ({self.tier})"


class MemoryQuality:
    """Quality scoring and validation for memories."""

    # Quality threshold (55% of max)
    MIN_QUALITY_SCORE = 18
    MAX_QUALITY_SCORE = 33

    # Weighted scoring formula
    WEIGHTS = {
        'reusability': 3,
        'impact': 3,
        'clarity': 2,
        'evidence': 2,
        'novelty': 1,
    }

    def score_memory(self, content: str, metadata: Optional[Dict] = None) -> QualityScore:
        """Score memory quality on 33-point scale.

        Args:
            content: Memory content (markdown)
            metadata: Optional metadata dict

        Returns:
            QualityScore with dimensions and total
        """
        # Score each dimension (1-3)
        reusability = self._score_reusability(content, metadata)
        impact = self._score_impact(content, metadata)
        clarity = self._score_clarity(content)
        evidence = self._score_evidence(content)
        novelty = self._score_novelty(content, metadata)

        # Calculate weighted total
        total = (
            reusability * self.WEIGHTS['reusability'] +
            impact * self.WEIGHTS['impact'] +
            clarity * self.WEIGHTS['clarity'] +
            evidence * self.WEIGHTS['evidence'] +
            novelty * self.WEIGHTS['novelty']
        )

        # Determine tier
        if total >= 27:
            tier = "excellent"
        elif total >= 21:
            tier = "good"
        elif total >= 18:
            tier = "acceptable"
        else:
            tier = "poor"

        return QualityScore(
            reusability=reusability,
            impact=impact,
            clarity=clarity,
            evidence=evidence,
            novelty=novelty,
            total=total,
            passed=(total >= self.MIN_QUALITY_SCORE),
            tier=tier
        )

    def _score_reusability(self, content: str, metadata: Optional[Dict]) -> int:
        """Score reusability (1-3).

        3 = Applies to many future tasks
        2 = Applies to several similar tasks
        1 = One-time insight
        """
        content_lower = content.lower()

        # High reusability indicators
        if any(word in content_lower for word in ['pattern', 'always', 'whenever', 'general', 'universal']):
            return 3

        # Medium reusability indicators
        if any(word in content_lower for word in ['often', 'usually', 'common', 'typical']):
            return 2

        # Default to low
        return 1

    def _score_impact(self, content: str, metadata: Optional[Dict]) -> int:
        """Score impact (1-3).

        3 = Saves significant time or prevents critical issues
        2 = Moderate time savings or issue prevention
        1 = Minor improvement
        """
        content_lower = content.lower()

        # High impact indicators
        high_impact_words = [
            'critical', 'severe', 'vulnerability', 'security',
            'hours', 'days', 'expensive', '10x', 'massive'
        ]
        if any(word in content_lower for word in high_impact_words):
            return 3

        # Medium impact indicators
        medium_impact_words = [
            'important', 'significant', 'minutes', 'improves',
            'faster', 'better', 'optimization'
        ]
        if any(word in content_lower for word in medium_impact_words):
            return 2

        # Default to low
        return 1

    def _score_clarity(self, content: str) -> int:
        """Score clarity (1-3).

        3 = Clear structure, examples, actionable
        2 = Understandable but could be clearer
        1 = Confusing or vague
        """
        # Check for structural elements
        has_sections = bool(re.search(r'^##\s+', content, re.MULTILINE))
        has_code = '```' in content or '`' in content
        has_list = bool(re.search(r'^\s*[-*\d]+\.?\s+', content, re.MULTILINE))

        # Check for actionable elements
        has_checklist = '[ ]' in content or '[x]' in content
        has_example = 'example' in content.lower()

        clarity_score = 1

        # Add points for structure
        if has_sections:
            clarity_score += 1

        # Add points for examples and actionability
        if (has_code or has_example) and (has_list or has_checklist):
            clarity_score = 3

        return min(clarity_score, 3)

    def _score_evidence(self, content: str) -> int:
        """Score evidence (1-3).

        3 = Concrete examples, code, benchmarks
        2 = Some examples or references
        1 = Claims without evidence
        """
        # Count evidence types
        evidence_count = 0

        if '```' in content:  # Code blocks
            evidence_count += 1

        if re.search(r'\d+%|\d+x|\d+ms|\d+s', content):  # Metrics
            evidence_count += 1

        if re.search(r'example|instance|case', content, re.IGNORECASE):
            evidence_count += 1

        if evidence_count >= 2:
            return 3
        elif evidence_count == 1:
            return 2
        else:
            return 1

    def _score_novelty(self, content: str, metadata: Optional[Dict]) -> int:
        """Score novelty (1-3).

        3 = New insight or technique
        2 = Interesting combination
        1 = Well-known or trivial
        """
        content_lower = content.lower()

        # High novelty indicators
        if any(word in content_lower for word in ['discovered', 'invented', 'novel', 'new approach', 'breakthrough']):
            return 3

        # Medium novelty indicators
        if any(word in content_lower for word in ['interesting', 'unexpected', 'surprising', 'insight']):
            return 2

        # Check confidence level from metadata
        if metadata and metadata.get('confidence') == 'high':
            return 2

        # Default to low
        return 1


class MemoryTriggerDetector:
    """Detects when memories should be written."""

    def should_write_pattern(self, occurrences: int, threshold: int = 3) -> Tuple[bool, str]:
        """Check if pattern detection warrants memory write.

        Args:
            occurrences: Number of times pattern was seen
            threshold: Minimum occurrences to write

        Returns:
            Tuple of (should_write: bool, reason: str)
        """
        if occurrences >= threshold:
            return True, f"Pattern found {occurrences} times (threshold: {threshold})"
        else:
            return False, f"Only {occurrences} occurrences (need {threshold})"

    def should_write_technique(self, complexity: int, time_saved: int) -> Tuple[bool, str]:
        """Check if novel technique warrants memory write.

        Args:
            complexity: Complexity score (1-10)
            time_saved: Estimated time saved in seconds

        Returns:
            Tuple of (should_write: bool, reason: str)
        """
        # High complexity or significant time savings
        if complexity >= 7 or time_saved >= 300:  # 5+ minutes
            return True, f"Complex technique (complexity: {complexity}, saves {time_saved}s)"
        else:
            return False, f"Low complexity/impact (complexity: {complexity}, saves {time_saved}s)"

    def should_write_dead_end(self, time_wasted: int, threshold: int = 1800) -> Tuple[bool, str]:
        """Check if dead end warrants memory write.

        Args:
            time_wasted: Time spent in seconds
            threshold: Minimum time to warrant write (default: 30 min)

        Returns:
            Tuple of (should_write: bool, reason: str)
        """
        if time_wasted >= threshold:
            return True, f"High-impact dead end (wasted {time_wasted}s, threshold: {threshold}s)"
        else:
            return False, f"Low-impact dead end (wasted {time_wasted}s, need {threshold}s)"

    def should_write_synthesis(self, num_sources: int, threshold: int = 3) -> Tuple[bool, str]:
        """Check if synthesis warrants memory write.

        Args:
            num_sources: Number of sources synthesized
            threshold: Minimum sources to warrant write

        Returns:
            Tuple of (should_write: bool, reason: str)
        """
        if num_sources >= threshold:
            return True, f"Complex synthesis ({num_sources} sources, threshold: {threshold})"
        else:
            return False, f"Simple synthesis ({num_sources} sources, need {threshold})"


class MemoryDeduplicator:
    """Detects and handles duplicate memories."""

    def __init__(self, similarity_threshold: float = 0.80):
        """Initialize deduplicator.

        Args:
            similarity_threshold: Similarity ratio to consider duplicate (0-1)
        """
        self.similarity_threshold = similarity_threshold

    def find_duplicates(self, memories: List[str]) -> List[Tuple[str, str, float]]:
        """Find duplicate memories based on content similarity.

        Args:
            memories: List of memory file paths

        Returns:
            List of (file1, file2, similarity) tuples
        """
        duplicates = []

        # Compare all pairs
        for i in range(len(memories)):
            for j in range(i + 1, len(memories)):
                try:
                    content1 = Path(memories[i]).read_text()
                    content2 = Path(memories[j]).read_text()

                    # Extract markdown content (skip frontmatter)
                    body1 = self._extract_body(content1)
                    body2 = self._extract_body(content2)

                    # Calculate similarity
                    similarity = self._calculate_similarity(body1, body2)

                    if similarity >= self.similarity_threshold:
                        duplicates.append((memories[i], memories[j], similarity))

                except Exception:
                    continue

        return duplicates

    def _extract_body(self, markdown: str) -> str:
        """Extract body content from markdown (skip frontmatter)."""
        parts = markdown.split('---', 2)
        if len(parts) >= 3:
            return parts[2].strip()
        return markdown

    def _calculate_similarity(self, text1: str, text2: str) -> float:
        """Calculate similarity ratio between two texts.

        Args:
            text1: First text
            text2: Second text

        Returns:
            Similarity ratio (0-1)
        """
        return difflib.SequenceMatcher(None, text1, text2).ratio()

    def merge_duplicates(self, file1: str, file2: str) -> str:
        """Merge two duplicate memories.

        Args:
            file1: First memory file path
            file2: Second memory file path

        Returns:
            Merged content
        """
        # Read both files
        content1 = Path(file1).read_text()
        content2 = Path(file2).read_text()

        # Extract bodies
        body1 = self._extract_body(content1)
        body2 = self._extract_body(content2)

        # Simple merge: combine unique paragraphs
        paragraphs = set()
        for text in [body1, body2]:
            for para in text.split('\n\n'):
                if para.strip():
                    paragraphs.add(para.strip())

        merged_body = '\n\n'.join(sorted(paragraphs))

        # TODO: Merge metadata intelligently
        # For now, just return merged body
        return merged_body


# Test functions
def test_quality_scoring():
    """Test quality scoring system."""
    print("Testing quality scoring...")

    quality = MemoryQuality()

    # Test excellent quality memory
    excellent_content = """# JWT Authentication Pattern

**Context**: Discovered during security audit of 5 production systems.

## The Pattern

All JWT endpoints MUST validate three properties:
1. Signature - Verify with secret key
2. Expiration - Check exp claim
3. Issuer - Validate iss claim

## Evidence

**Example 1** - Validates all 3:
```python
payload = jwt.decode(token, SECRET_KEY,
                     algorithms=['RS256'],
                     options={'verify_exp': True, 'verify_iss': True})
```

Found in 40% of endpoints (2 out of 5).

## Impact

- Severity: HIGH
- Time saved: 2 hours per audit
- Critical security issue prevention

## Application Checklist

- [ ] Verify signature
- [ ] Check expiration
- [ ] Validate issuer
"""

    score = quality.score_memory(excellent_content)
    print(f"Excellent content: {score}")
    assert score.total >= 24, f"Expected high score, got {score.total}"
    assert score.passed, "Should pass quality threshold"
    print("✅ Excellent content scored highly")

    # Test poor quality memory
    poor_content = """Fixed a bug."""

    score = quality.score_memory(poor_content)
    print(f"Poor content: {score}")
    assert score.total < 18, f"Expected low score, got {score.total}"
    assert not score.passed, "Should fail quality threshold"
    print("✅ Poor content scored low")


def test_trigger_detection():
    """Test write trigger detection."""
    print("\nTesting write triggers...")

    detector = MemoryTriggerDetector()

    # Test pattern detection
    should_write, reason = detector.should_write_pattern(occurrences=5)
    assert should_write, "Should trigger on 5 occurrences"
    print(f"✅ Pattern trigger: {reason}")

    should_write, reason = detector.should_write_pattern(occurrences=2)
    assert not should_write, "Should not trigger on 2 occurrences"
    print(f"✅ Pattern no-trigger: {reason}")

    # Test technique trigger
    should_write, reason = detector.should_write_technique(complexity=8, time_saved=600)
    assert should_write, "Should trigger on complex technique"
    print(f"✅ Technique trigger: {reason}")

    # Test dead end trigger
    should_write, reason = detector.should_write_dead_end(time_wasted=2400)  # 40 min
    assert should_write, "Should trigger on 40-minute dead end"
    print(f"✅ Dead end trigger: {reason}")


def test_deduplication():
    """Test deduplication detection."""
    print("\nTesting deduplication...")

    import tempfile
    import shutil

    temp_dir = tempfile.mkdtemp()

    try:
        # Create two similar files
        content1 = """---
date: 2025-10-03
agent: test-agent
type: pattern
topic: jwt
tags: [auth]
confidence: high
visibility: public
---

# JWT Pattern

Always validate signature, expiration, and issuer.
"""

        content2 = """---
date: 2025-10-03
agent: test-agent
type: pattern
topic: jwt-auth
tags: [auth]
confidence: high
visibility: public
---

# JWT Pattern

Always validate signature, expiration, and issuer.
"""

        file1 = Path(temp_dir) / "memory1.md"
        file2 = Path(temp_dir) / "memory2.md"

        file1.write_text(content1)
        file2.write_text(content2)

        # Detect duplicates
        deduplicator = MemoryDeduplicator(similarity_threshold=0.80)
        duplicates = deduplicator.find_duplicates([str(file1), str(file2)])

        assert len(duplicates) == 1, f"Expected 1 duplicate pair, found {len(duplicates)}"
        print(f"✅ Found duplicate (similarity: {duplicates[0][2]:.2f})")

    finally:
        shutil.rmtree(temp_dir)


if __name__ == "__main__":
    test_quality_scoring()
    test_trigger_detection()
    test_deduplication()
    print("\n🎉 All quality control tests passed!")
