#!/usr/bin/env python3
"""
Memory System CLI for AI-CIV Collective

Comprehensive command-line interface for all memory operations:
- Write, read, search memories
- Quality scoring and validation
- Security scanning
- Index management
- Federation (export/import)
- Statistics and reporting

Author: The Conductor
Version: 1.0.0
"""

import argparse
import sys
import json
from pathlib import Path
from typing import Optional

# Import memory system components
from memory_core import MemoryStore, MemoryEntry
from memory_security import MemorySecurityValidator, SensitiveDataDetector
from memory_quality import MemoryQuality, MemoryTriggerDetector, MemoryDeduplicator
from memory_search import MemoryIndexer, QueryRouter
from memory_federation import FederationClient


class MemoryCLI:
    """Command-line interface for memory system."""

    def __init__(self):
        """Initialize CLI."""
        self.store = MemoryStore()
        self.validator = MemorySecurityValidator()
        self.quality = MemoryQuality()
        self.indexer = MemoryIndexer()
        self.router = QueryRouter()
        self.federation = FederationClient()

    def write_memory(self, agent: str, type: str, topic: str, tags: list,
                    confidence: str, visibility: str, content: str) -> str:
        """Write a new memory.

        Args:
            agent: Agent creating memory
            type: Memory type
            topic: Topic
            tags: List of tags
            confidence: Confidence level
            visibility: Visibility level
            content: Markdown content

        Returns:
            Path to written memory
        """
        from datetime import datetime

        # Create entry
        entry = MemoryEntry(
            date=datetime.now().strftime("%Y-%m-%d"),
            agent=agent,
            type=type,
            topic=topic,
            tags=tags,
            confidence=confidence,
            visibility=visibility,
            content=content
        )

        # Score quality
        score = self.quality.score_memory(content)
        entry.quality_score = score.total

        if not score.passed:
            print(f"⚠️  Warning: Quality score {score.total}/33 is below threshold (18)")
            print(f"   Dimensions: R={score.reusability} I={score.impact} C={score.clarity} E={score.evidence} N={score.novelty}")
            response = input("   Write anyway? (y/n): ")
            if response.lower() != 'y':
                raise ValueError("Memory write cancelled: quality too low")

        # Validate security
        filepath = str(Path(self.store._get_agent_dir(agent)) / self.store._generate_filename(entry))
        self.validator.validate_before_write(agent, content, filepath)

        # Write
        filepath = self.store.write_entry(agent, entry)
        print(f"✅ Memory written: {filepath}")
        print(f"   Quality: {score.total}/33 ({score.tier})")

        return filepath

    def read_memory(self, filepath: str) -> MemoryEntry:
        """Read a memory.

        Args:
            filepath: Path to memory file

        Returns:
            MemoryEntry
        """
        entry = self.store.read_entry(filepath)
        print(f"📖 Memory: {filepath}")
        print(f"   Agent: {entry.agent}")
        print(f"   Topic: {entry.topic}")
        print(f"   Type: {entry.type}")
        print(f"   Tags: {', '.join(entry.tags)}")
        print(f"   Quality: {entry.quality_score}/33")
        print(f"   Reuse count: {entry.reuse_count}")
        print(f"\n{entry.content}")

        return entry

    def search_memories(self, query: str, agent: Optional[str] = None) -> list:
        """Search memories.

        Args:
            query: Search query
            agent: Optional agent filter

        Returns:
            List of matching file paths
        """
        result = self.router.search(query, agent)

        print(f"🔍 Search: '{query}'")
        print(f"   Results: {len(result['results'])} memories")
        print(f"   Tier: {result['tier']} ({result['source']})")
        print(f"   Time: {result['elapsed_ms']:.1f}ms")
        print()

        for i, filepath in enumerate(result['results'][:10], 1):
            filename = Path(filepath).name
            print(f"   {i}. {filename}")

        if len(result['results']) > 10:
            print(f"   ... and {len(result['results']) - 10} more")

        return result['results']

    def list_memories(self, agent: str) -> list:
        """List memories for agent.

        Args:
            agent: Agent whose memories to list

        Returns:
            List of file paths
        """
        memories = self.store.list_memories(agent)

        print(f"📚 Memories for {agent}: {len(memories)}")
        for i, filepath in enumerate(memories[:20], 1):
            filename = Path(filepath).name
            print(f"   {i}. {filename}")

        if len(memories) > 20:
            print(f"   ... and {len(memories) - 20} more")

        return memories

    def scan_security(self, filepath: Optional[str] = None) -> dict:
        """Scan for security issues.

        Args:
            filepath: Optional specific file to scan (None = scan all)

        Returns:
            Dict with findings
        """
        detector = SensitiveDataDetector()

        if filepath:
            # Scan single file
            content = Path(filepath).read_text()
            findings = detector.scan(content)

            print(f"🔒 Security scan: {filepath}")
            if findings:
                print(f"   ⚠️  {len(findings)} findings:")
                for finding in findings:
                    print(f"      - {finding['type']}: {finding['match'][:50]} ({finding['severity']})")
            else:
                print("   ✅ No security issues")

            return {'file': filepath, 'findings': findings}

        else:
            # Scan all memories
            all_findings = {}
            total_files = 0
            total_findings = 0

            for md_file in Path(".claude/memory/agent-learnings").rglob("*.md"):
                total_files += 1
                content = md_file.read_text()
                findings = detector.scan(content)

                if findings:
                    all_findings[str(md_file)] = findings
                    total_findings += len(findings)

            print(f"🔒 Security scan: all memories")
            print(f"   Files scanned: {total_files}")
            print(f"   Issues found: {total_findings}")

            if all_findings:
                print(f"   ⚠️  Files with issues:")
                for filepath, findings in all_findings.items():
                    print(f"      - {Path(filepath).name}: {len(findings)} findings")

            return all_findings

    def build_indexes(self) -> dict:
        """Build search indexes.

        Returns:
            Index statistics
        """
        print("🔨 Building search indexes...")
        stats = self.indexer.build_all_indexes()

        print(f"   Inverted index: {stats['inverted']} entries")
        print(f"   Chronological: {stats['chronological']} entries")
        print(f"   Agent index: {stats['agent']} entries")
        print(f"   Connections: {stats['connections']} entries")
        print(f"   Time: {stats['elapsed_seconds']:.2f}s")
        print("   ✅ Indexes built successfully")

        return stats

    def get_stats(self) -> dict:
        """Get system statistics.

        Returns:
            Statistics dict
        """
        search_stats = self.router.get_stats()

        print("📊 Memory System Statistics")
        print(f"\n   Search Performance:")
        print(f"   - Total queries: {search_stats['total_queries']}")
        print(f"   - Cache hit rate: {search_stats.get('cache_hit_rate', 0):.1%}")
        print(f"   - Avg latency: {search_stats.get('avg_latency_ms', 0):.1f}ms")
        print(f"   - Tier 1 (cache): {search_stats['tier1_count']}")
        print(f"   - Tier 2 (index): {search_stats['tier2_count']}")
        print(f"   - Tier 3 (grep): {search_stats['tier3_count']}")

        cache_stats = search_stats.get('cache_stats', {})
        print(f"\n   Cache:")
        print(f"   - Entries: {cache_stats.get('entries', 0)}")
        print(f"   - Size: {cache_stats.get('size_mb', 0):.2f}MB / {cache_stats.get('max_size_mb', 100)}MB")

        return search_stats

    def find_duplicates(self, agent: Optional[str] = None) -> list:
        """Find duplicate memories.

        Args:
            agent: Optional agent filter

        Returns:
            List of duplicate pairs
        """
        deduplicator = MemoryDeduplicator(similarity_threshold=0.80)

        if agent:
            memories = self.store.list_memories(agent)
        else:
            memories = []
            for agent_dir in (Path(".claude/memory/agent-learnings").iterdir()):
                if agent_dir.is_dir():
                    memories.extend(self.store.list_memories(agent_dir.name))

        print(f"🔎 Scanning for duplicates...")
        print(f"   Memories to check: {len(memories)}")

        duplicates = deduplicator.find_duplicates(memories)

        print(f"   Duplicates found: {len(duplicates)}")

        for file1, file2, similarity in duplicates:
            print(f"   - {Path(file1).name}")
            print(f"     {Path(file2).name}")
            print(f"     Similarity: {similarity:.1%}\n")

        return duplicates

    def export_knowledge(self, agent: Optional[str], topics: Optional[list],
                        output: Optional[str] = None) -> str:
        """Export knowledge package.

        Args:
            agent: Agent filter
            topics: Topic filter
            output: Output filename

        Returns:
            Path to exported package
        """
        print(f"📦 Exporting knowledge package...")
        if agent:
            print(f"   Agent: {agent}")
        if topics:
            print(f"   Topics: {', '.join(topics)}")

        package = self.federation.export_memories(
            agent=agent,
            visibility='public',
            topics=topics,
            sign=True
        )

        print(f"   Insights: {len(package['insights'])}")

        filepath = self.federation.save_package(package, output)
        print(f"   ✅ Exported to: {filepath}")

        return filepath

    def import_knowledge(self, package_file: str, verify: bool = True) -> int:
        """Import knowledge package.

        Args:
            package_file: Path to package JSON
            verify: Whether to verify signature

        Returns:
            Number of imported memories
        """
        print(f"📥 Importing knowledge package...")
        print(f"   File: {package_file}")
        print(f"   Verify signature: {verify}")

        count = self.federation.import_memories(
            package_file,
            verify=verify,
            quarantine=True
        )

        print(f"   ✅ Imported {count} memories")
        print(f"   Location: knowledge/federated/quarantine/")

        return count


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Memory System CLI for AI-CIV Collective",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )

    subparsers = parser.add_subparsers(dest='command', help='Command to execute')

    # Write command
    write_parser = subparsers.add_parser('write', help='Write a new memory')
    write_parser.add_argument('--agent', required=True, help='Agent creating memory')
    write_parser.add_argument('--type', required=True, choices=['pattern', 'technique', 'contradiction', 'gotcha', 'synthesis', 'experiment'])
    write_parser.add_argument('--topic', required=True, help='Topic slug')
    write_parser.add_argument('--tags', required=True, help='Comma-separated tags')
    write_parser.add_argument('--confidence', required=True, choices=['high', 'medium', 'low'])
    write_parser.add_argument('--visibility', required=True, choices=['public', 'collective-only', 'private'])
    write_parser.add_argument('--content', required=True, help='Markdown content (or @file)')

    # Read command
    read_parser = subparsers.add_parser('read', help='Read a memory')
    read_parser.add_argument('filepath', help='Path to memory file')

    # Search command
    search_parser = subparsers.add_parser('search', help='Search memories')
    search_parser.add_argument('query', help='Search query')
    search_parser.add_argument('--agent', help='Filter by agent')

    # List command
    list_parser = subparsers.add_parser('list', help='List memories for agent')
    list_parser.add_argument('agent', help='Agent name')

    # Scan command
    scan_parser = subparsers.add_parser('scan', help='Security scan')
    scan_parser.add_argument('--file', help='Specific file to scan')

    # Index command
    subparsers.add_parser('index', help='Build search indexes')

    # Stats command
    subparsers.add_parser('stats', help='Show system statistics')

    # Duplicates command
    dup_parser = subparsers.add_parser('duplicates', help='Find duplicate memories')
    dup_parser.add_argument('--agent', help='Filter by agent')

    # Export command
    export_parser = subparsers.add_parser('export', help='Export knowledge package')
    export_parser.add_argument('--agent', help='Filter by agent')
    export_parser.add_argument('--topics', help='Comma-separated topics')
    export_parser.add_argument('--output', help='Output filename')

    # Import command
    import_parser = subparsers.add_parser('import', help='Import knowledge package')
    import_parser.add_argument('package', help='Package file path')
    import_parser.add_argument('--no-verify', action='store_true', help='Skip signature verification')

    # Parse arguments
    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    # Execute command
    cli = MemoryCLI()

    try:
        if args.command == 'write':
            # Load content from file if @file syntax
            content = args.content
            if content.startswith('@'):
                content = Path(content[1:]).read_text()

            tags = [t.strip() for t in args.tags.split(',')]

            cli.write_memory(
                agent=args.agent,
                type=args.type,
                topic=args.topic,
                tags=tags,
                confidence=args.confidence,
                visibility=args.visibility,
                content=content
            )

        elif args.command == 'read':
            cli.read_memory(args.filepath)

        elif args.command == 'search':
            cli.search_memories(args.query, args.agent)

        elif args.command == 'list':
            cli.list_memories(args.agent)

        elif args.command == 'scan':
            cli.scan_security(args.file)

        elif args.command == 'index':
            cli.build_indexes()

        elif args.command == 'stats':
            cli.get_stats()

        elif args.command == 'duplicates':
            cli.find_duplicates(args.agent)

        elif args.command == 'export':
            topics = [t.strip() for t in args.topics.split(',')] if args.topics else None
            cli.export_knowledge(args.agent, topics, args.output)

        elif args.command == 'import':
            cli.import_knowledge(args.package, not args.no_verify)

        return 0

    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
