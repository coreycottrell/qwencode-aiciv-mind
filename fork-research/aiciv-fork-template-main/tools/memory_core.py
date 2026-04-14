#!/usr/bin/env python3
"""
Core Memory Operations for AI-CIV Collective

This module provides the foundational memory storage and retrieval operations.
All memory entries are stored as markdown files with YAML frontmatter.

Author: The Conductor & Code Archaeologist
Version: 1.0.0
"""

import os
import json
import yaml
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field, asdict


@dataclass
class MemoryEntry:
    """Represents a single memory entry with metadata and content."""

    # Required fields
    date: str  # YYYY-MM-DD
    agent: str
    type: str  # pattern/technique/contradiction/gotcha/synthesis/experiment
    topic: str
    tags: List[str]
    confidence: str  # high/medium/low
    visibility: str  # public/collective-only/private
    content: str  # Markdown body

    # Auto-generated fields
    quality_score: int = 0
    reuse_count: int = 0
    created: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    last_accessed: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    content_hash: str = ""

    # Optional fields
    connections: List[Dict[str, str]] = field(default_factory=list)
    contradicts: List[str] = field(default_factory=list)
    supersedes: List[str] = field(default_factory=list)
    evidence: List[Dict[str, Any]] = field(default_factory=list)

    def __post_init__(self):
        """Validate and compute derived fields after initialization."""
        # Validate required enums
        valid_types = ['pattern', 'technique', 'contradiction', 'gotcha', 'synthesis', 'experiment']
        if self.type not in valid_types:
            raise ValueError(f"Invalid type '{self.type}'. Must be one of: {valid_types}")

        valid_confidence = ['high', 'medium', 'low']
        if self.confidence not in valid_confidence:
            raise ValueError(f"Invalid confidence '{self.confidence}'. Must be one of: {valid_confidence}")

        valid_visibility = ['public', 'collective-only', 'private']
        if self.visibility not in valid_visibility:
            raise ValueError(f"Invalid visibility '{self.visibility}'. Must be one of: {valid_visibility}")

        # Compute content hash if not provided
        if not self.content_hash:
            self.content_hash = self._compute_hash()

    def _compute_hash(self) -> str:
        """Compute SHA-256 hash of content for integrity checking."""
        return hashlib.sha256(self.content.encode('utf-8')).hexdigest()

    def to_markdown(self) -> str:
        """Convert memory entry to markdown with YAML frontmatter."""
        # Prepare metadata dict
        metadata = {
            'date': self.date,
            'agent': self.agent,
            'type': self.type,
            'topic': self.topic,
            'tags': self.tags,
            'confidence': self.confidence,
            'visibility': self.visibility,
            'quality_score': self.quality_score,
            'reuse_count': self.reuse_count,
            'created': self.created,
            'last_accessed': self.last_accessed,
            'content_hash': self.content_hash,
        }

        # Add optional fields if present
        if self.connections:
            metadata['connections'] = self.connections
        if self.contradicts:
            metadata['contradicts'] = self.contradicts
        if self.supersedes:
            metadata['supersedes'] = self.supersedes
        if self.evidence:
            metadata['evidence'] = self.evidence

        # Format as YAML frontmatter + markdown
        frontmatter = yaml.dump(metadata, default_flow_style=False, allow_unicode=True)
        return f"---\n{frontmatter}---\n\n{self.content}"

    @classmethod
    def from_markdown(cls, markdown_text: str) -> 'MemoryEntry':
        """Parse markdown with YAML frontmatter into MemoryEntry."""
        # Split frontmatter and content
        parts = markdown_text.split('---', 2)
        if len(parts) < 3:
            raise ValueError("Invalid memory format: missing YAML frontmatter")

        # Parse YAML metadata
        metadata = yaml.safe_load(parts[1])
        content = parts[2].strip()

        # Extract required fields
        return cls(
            date=metadata['date'],
            agent=metadata['agent'],
            type=metadata['type'],
            topic=metadata['topic'],
            tags=metadata['tags'],
            confidence=metadata['confidence'],
            visibility=metadata['visibility'],
            content=content,
            quality_score=metadata.get('quality_score', 0),
            reuse_count=metadata.get('reuse_count', 0),
            created=metadata.get('created', datetime.now(timezone.utc).isoformat()),
            last_accessed=metadata.get('last_accessed', datetime.now(timezone.utc).isoformat()),
            content_hash=metadata.get('content_hash', ''),
            connections=metadata.get('connections', []),
            contradicts=metadata.get('contradicts', []),
            supersedes=metadata.get('supersedes', []),
            evidence=metadata.get('evidence', [])
        )

    def update_access_time(self):
        """Update last_accessed timestamp."""
        self.last_accessed = datetime.now(timezone.utc).isoformat()
        self.reuse_count += 1


class MemoryStore:
    """Core memory storage and retrieval operations."""

    def __init__(self, base_dir: str = ".claude/memory"):
        """Initialize memory store.

        Args:
            base_dir: Root directory for memory storage
        """
        self.base_dir = Path(base_dir)
        self.agent_learnings_dir = self.base_dir / "agent-learnings"
        self.project_knowledge_dir = self.base_dir / "project-knowledge"
        self.indexes_dir = self.base_dir / ".indexes"

        # Ensure directories exist
        self.agent_learnings_dir.mkdir(parents=True, exist_ok=True)
        self.project_knowledge_dir.mkdir(parents=True, exist_ok=True)
        self.indexes_dir.mkdir(parents=True, exist_ok=True)

    def _generate_filename(self, entry: MemoryEntry) -> str:
        """Generate filename from memory entry metadata.

        Format: YYYY-MM-DD--{type}-{topic-slug}.md
        """
        # Sanitize topic for filename
        topic_slug = entry.topic.lower().replace(' ', '-').replace('_', '-')
        # Remove any non-alphanumeric characters except hyphens
        topic_slug = ''.join(c for c in topic_slug if c.isalnum() or c == '-')

        return f"{entry.date}--{entry.type}-{topic_slug}.md"

    def _get_agent_dir(self, agent_id: str) -> Path:
        """Get or create directory for agent's memories."""
        agent_dir = self.agent_learnings_dir / agent_id
        agent_dir.mkdir(parents=True, exist_ok=True)
        return agent_dir

    def write_entry(self, agent_id: str, entry: MemoryEntry) -> str:
        """Write memory entry to disk.

        Args:
            agent_id: Agent who owns this memory
            entry: MemoryEntry to write

        Returns:
            Absolute path to written file

        Raises:
            ValueError: If entry validation fails
            IOError: If file write fails
        """
        # Validate agent matches entry
        if entry.agent != agent_id:
            raise ValueError(f"Agent ID mismatch: {agent_id} != {entry.agent}")

        # Generate filename and full path
        agent_dir = self._get_agent_dir(agent_id)
        filename = self._generate_filename(entry)
        filepath = agent_dir / filename

        # Check if file already exists
        if filepath.exists():
            raise FileExistsError(f"Memory already exists: {filepath}")

        # Convert to markdown and write
        markdown = entry.to_markdown()
        filepath.write_text(markdown, encoding='utf-8')

        return str(filepath.absolute())

    def read_entry(self, filepath: str) -> MemoryEntry:
        """Read memory entry from disk.

        Args:
            filepath: Path to memory file

        Returns:
            Parsed MemoryEntry

        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If parsing fails
        """
        path = Path(filepath)
        if not path.exists():
            raise FileNotFoundError(f"Memory not found: {filepath}")

        markdown = path.read_text(encoding='utf-8')
        entry = MemoryEntry.from_markdown(markdown)

        # Update access time
        entry.update_access_time()

        # Write back updated metadata
        self._update_entry_metadata(filepath, entry)

        return entry

    def _update_entry_metadata(self, filepath: str, entry: MemoryEntry):
        """Update metadata in existing memory file (preserves content)."""
        path = Path(filepath)
        if not path.exists():
            return

        markdown = entry.to_markdown()
        path.write_text(markdown, encoding='utf-8')

    def list_memories(self, agent_id: str) -> List[str]:
        """List all memory files for an agent.

        Args:
            agent_id: Agent whose memories to list

        Returns:
            List of absolute file paths
        """
        agent_dir = self._get_agent_dir(agent_id)

        # Find all .md files (excluding archive subdirectories)
        memories = []
        for md_file in agent_dir.glob("*.md"):
            if md_file.is_file():
                memories.append(str(md_file.absolute()))

        # Sort by date (newest first)
        memories.sort(reverse=True)
        return memories

    def search_by_tag(self, agent: Optional[str], tag: str) -> List[str]:
        """Search memories by tag.

        Args:
            agent: Agent to search (None = all agents)
            tag: Tag to search for

        Returns:
            List of matching file paths
        """
        results = []

        # Determine search scope
        if agent:
            search_dirs = [self._get_agent_dir(agent)]
        else:
            search_dirs = [d for d in self.agent_learnings_dir.iterdir() if d.is_dir()]

        # Search all memories
        for agent_dir in search_dirs:
            for md_file in agent_dir.glob("*.md"):
                try:
                    markdown = md_file.read_text(encoding='utf-8')
                    # Quick YAML parse to check tags
                    if '---' in markdown:
                        parts = markdown.split('---', 2)
                        if len(parts) >= 2:
                            metadata = yaml.safe_load(parts[1])
                            if tag in metadata.get('tags', []):
                                results.append(str(md_file.absolute()))
                except Exception:
                    continue

        return results

    def search_by_topic(self, topic: str, agent: Optional[str] = None) -> List[str]:
        """Search memories by topic.

        Args:
            topic: Topic keyword to search for
            agent: Agent to search (None = all agents)

        Returns:
            List of matching file paths
        """
        results = []
        topic_lower = topic.lower()

        # Determine search scope
        if agent:
            search_dirs = [self._get_agent_dir(agent)]
        else:
            search_dirs = [d for d in self.agent_learnings_dir.iterdir() if d.is_dir()]

        # Search all memories
        for agent_dir in search_dirs:
            for md_file in agent_dir.glob("*.md"):
                try:
                    markdown = md_file.read_text(encoding='utf-8')
                    if '---' in markdown:
                        parts = markdown.split('---', 2)
                        if len(parts) >= 2:
                            metadata = yaml.safe_load(parts[1])
                            if topic_lower in metadata.get('topic', '').lower():
                                results.append(str(md_file.absolute()))
                except Exception:
                    continue

        return results

    def search_by_date_range(self, agent: str, start_date: str, end_date: str) -> List[str]:
        """Search memories by date range.

        Args:
            agent: Agent whose memories to search
            start_date: Start date (YYYY-MM-DD)
            end_date: End date (YYYY-MM-DD)

        Returns:
            List of matching file paths
        """
        results = []
        agent_dir = self._get_agent_dir(agent)

        for md_file in agent_dir.glob("*.md"):
            try:
                # Extract date from filename (YYYY-MM-DD--...)
                filename = md_file.name
                file_date = filename.split('--')[0]

                if start_date <= file_date <= end_date:
                    results.append(str(md_file.absolute()))
            except Exception:
                continue

        return results

    def search(self, agent: Optional[str] = None, tags: Optional[List[str]] = None,
               date_range: Optional[tuple] = None, confidence: Optional[str] = None,
               type: Optional[str] = None) -> List[Dict[str, Any]]:
        """Multi-criterion memory search.

        Args:
            agent: Agent to search (None = all)
            tags: List of tags to match (AND logic)
            date_range: Tuple of (start_date, end_date)
            confidence: Confidence level filter
            type: Memory type filter

        Returns:
            List of dicts with 'filepath' and 'metadata'
        """
        results = []

        # Determine search scope
        if agent:
            search_dirs = [self._get_agent_dir(agent)]
        else:
            search_dirs = [d for d in self.agent_learnings_dir.iterdir() if d.is_dir()]

        # Search all memories
        for agent_dir in search_dirs:
            for md_file in agent_dir.glob("*.md"):
                try:
                    markdown = md_file.read_text(encoding='utf-8')
                    if '---' not in markdown:
                        continue

                    parts = markdown.split('---', 2)
                    if len(parts) < 2:
                        continue

                    metadata = yaml.safe_load(parts[1])

                    # Apply filters
                    if tags:
                        if not all(tag in metadata.get('tags', []) for tag in tags):
                            continue

                    if date_range:
                        start_date, end_date = date_range
                        file_date = metadata.get('date', '')
                        if not (start_date <= file_date <= end_date):
                            continue

                    if confidence:
                        if metadata.get('confidence') != confidence:
                            continue

                    if type:
                        if metadata.get('type') != type:
                            continue

                    # Match! Add to results
                    results.append({
                        'filepath': str(md_file.absolute()),
                        'metadata': metadata
                    })

                except Exception:
                    continue

        return results


# Test functions (inline testing)
def test_memory_entry():
    """Test MemoryEntry class."""
    print("Testing MemoryEntry class...")

    # Create entry
    entry = MemoryEntry(
        date="2025-10-03",
        agent="test-agent",
        type="pattern",
        topic="jwt-authentication",
        tags=["auth", "security", "jwt"],
        confidence="high",
        visibility="public",
        content="# JWT Best Practices\n\nAlways validate signature, expiration, and issuer."
    )

    # Test markdown conversion
    markdown = entry.to_markdown()
    assert "---" in markdown
    assert "jwt-authentication" in markdown

    # Test round-trip
    entry2 = MemoryEntry.from_markdown(markdown)
    assert entry2.agent == entry.agent
    assert entry2.topic == entry.topic
    assert entry2.type == entry.type

    print("✅ MemoryEntry tests passed")


def test_memory_store():
    """Test MemoryStore class."""
    print("\nTesting MemoryStore class...")

    import tempfile
    import shutil

    # Create temporary directory
    temp_dir = tempfile.mkdtemp()

    try:
        store = MemoryStore(temp_dir)

        # Create test entry
        entry = MemoryEntry(
            date="2025-10-03",
            agent="test-agent",
            type="pattern",
            topic="test-pattern",
            tags=["test", "pattern"],
            confidence="high",
            visibility="public",
            content="# Test Pattern\n\nThis is a test."
        )

        # Write entry
        filepath = store.write_entry("test-agent", entry)
        assert os.path.exists(filepath)
        print(f"✅ Written to: {filepath}")

        # Read entry
        read_entry = store.read_entry(filepath)
        assert read_entry.topic == entry.topic
        assert read_entry.reuse_count == 1  # Should increment on read
        print("✅ Read back successfully")

        # List memories
        memories = store.list_memories("test-agent")
        assert len(memories) == 1
        print(f"✅ Listed {len(memories)} memories")

        # Search by tag
        results = store.search_by_tag("test-agent", "test")
        assert len(results) == 1
        print("✅ Search by tag works")

        # Multi-criterion search
        results = store.search(agent="test-agent", tags=["test"], confidence="high")
        assert len(results) == 1
        print("✅ Multi-criterion search works")

    finally:
        # Cleanup
        shutil.rmtree(temp_dir)
        print("✅ Cleanup complete")

    print("✅ MemoryStore tests passed")


if __name__ == "__main__":
    # Run tests
    test_memory_entry()
    test_memory_store()
    print("\n🎉 All core memory tests passed!")
