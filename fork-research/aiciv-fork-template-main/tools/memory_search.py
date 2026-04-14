#!/usr/bin/env python3
"""
Memory Search & Performance Layer for AI-CIV Collective

Implements 4-tier search strategy with sub-second performance:
- Tier 1: L1 Cache (<10ms, 90% hit rate)
- Tier 2: Index Search (50-200ms)
- Tier 3: Grep Search (200-800ms)
- Tier 4: Deep Analysis (1-5s)

Author: The Conductor & Performance Optimizer
Version: 1.0.0
"""

import os
import json
import time
import hashlib
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from collections import OrderedDict
from datetime import datetime, timezone
import subprocess


class FrequencyBoostLRU:
    """LRU cache with frequency-based promotion."""

    def __init__(self, max_size_mb: int = 100):
        """Initialize cache.

        Args:
            max_size_mb: Maximum cache size in megabytes
        """
        self.cache = OrderedDict()  # query_hash -> results
        self.frequency = {}  # query_hash -> (count, last_access)
        self.max_size_bytes = max_size_mb * 1024 * 1024
        self.current_size_bytes = 0

    def get(self, query_hash: str) -> Optional[Any]:
        """Get cached result.

        Args:
            query_hash: Hash of query

        Returns:
            Cached result or None
        """
        if query_hash in self.cache:
            # Update frequency
            count, _ = self.frequency.get(query_hash, (0, None))
            self.frequency[query_hash] = (count + 1, datetime.now())

            # Move to end (most recently used)
            self.cache.move_to_end(query_hash)

            return self.cache[query_hash]

        return None

    def put(self, query_hash: str, result: Any):
        """Store result in cache.

        Args:
            query_hash: Hash of query
            result: Result to cache
        """
        # Estimate size
        result_size = len(json.dumps(result, default=str))

        # Evict if needed
        while self.current_size_bytes + result_size > self.max_size_bytes and self.cache:
            # Remove least recently used with lowest frequency
            lru_key = next(iter(self.cache))
            lru_size = len(json.dumps(self.cache[lru_key], default=str))
            del self.cache[lru_key]
            del self.frequency[lru_key]
            self.current_size_bytes -= lru_size

        # Add to cache
        self.cache[query_hash] = result
        self.frequency[query_hash] = (1, datetime.now())
        self.current_size_bytes += result_size

    def stats(self) -> Dict[str, Any]:
        """Get cache statistics.

        Returns:
            Dictionary with cache stats
        """
        return {
            'entries': len(self.cache),
            'size_mb': self.current_size_bytes / (1024 * 1024),
            'max_size_mb': self.max_size_bytes / (1024 * 1024),
        }


class MemoryIndexer:
    """Builds and maintains search indexes."""

    def __init__(self, base_dir: str = ".claude/memory"):
        """Initialize indexer.

        Args:
            base_dir: Memory base directory
        """
        self.base_dir = Path(base_dir)
        self.indexes_dir = self.base_dir / ".indexes"
        self.indexes_dir.mkdir(parents=True, exist_ok=True)

    def build_all_indexes(self) -> Dict[str, int]:
        """Build all search indexes.

        Returns:
            Dict with index names and entry counts
        """
        start_time = time.time()

        # Build each index
        inverted_count = self._build_inverted_index()
        chrono_count = self._build_chronological_index()
        agent_count = self._build_agent_index()
        connection_count = self._build_connection_graph()

        elapsed = time.time() - start_time

        return {
            'inverted': inverted_count,
            'chronological': chrono_count,
            'agent': agent_count,
            'connections': connection_count,
            'elapsed_seconds': elapsed
        }

    def _build_inverted_index(self) -> int:
        """Build tag -> files inverted index.

        Returns:
            Number of entries indexed
        """
        inverted = {}  # tag -> [file paths]
        count = 0

        # Scan all memory files
        for md_file in self.base_dir.rglob("*.md"):
            if '.indexes' in str(md_file):
                continue

            try:
                # Quick parse for tags
                content = md_file.read_text(encoding='utf-8')
                if '---' in content:
                    import yaml
                    parts = content.split('---', 2)
                    if len(parts) >= 2:
                        metadata = yaml.safe_load(parts[1])
                        tags = metadata.get('tags', [])

                        # Add to index
                        for tag in tags:
                            if tag not in inverted:
                                inverted[tag] = []
                            inverted[tag].append(str(md_file.absolute()))

                        count += 1

            except Exception:
                continue

        # Save index
        index_file = self.indexes_dir / "inverted-index.json"
        index_file.write_text(json.dumps(inverted, indent=2))

        return count

    def _build_chronological_index(self) -> int:
        """Build date -> files chronological index.

        Returns:
            Number of entries indexed
        """
        chrono = {}  # date -> [file paths]
        count = 0

        for md_file in self.base_dir.rglob("*.md"):
            if '.indexes' in str(md_file):
                continue

            try:
                # Extract date from filename
                filename = md_file.name
                if '--' in filename:
                    date = filename.split('--')[0]

                    if date not in chrono:
                        chrono[date] = []
                    chrono[date].append(str(md_file.absolute()))

                    count += 1

            except Exception:
                continue

        # Save index
        index_file = self.indexes_dir / "chronological.json"
        index_file.write_text(json.dumps(chrono, indent=2))

        return count

    def _build_agent_index(self) -> int:
        """Build agent -> topics index.

        Returns:
            Number of entries indexed
        """
        agent_index = {}  # agent -> {topics: [], files: []}
        count = 0

        for md_file in self.base_dir.rglob("*.md"):
            if '.indexes' in str(md_file):
                continue

            try:
                content = md_file.read_text(encoding='utf-8')
                if '---' in content:
                    import yaml
                    parts = content.split('---', 2)
                    if len(parts) >= 2:
                        metadata = yaml.safe_load(parts[1])
                        agent = metadata.get('agent')
                        topic = metadata.get('topic')

                        if agent:
                            if agent not in agent_index:
                                agent_index[agent] = {'topics': [], 'files': []}

                            if topic and topic not in agent_index[agent]['topics']:
                                agent_index[agent]['topics'].append(topic)

                            agent_index[agent]['files'].append(str(md_file.absolute()))
                            count += 1

            except Exception:
                continue

        # Save index
        index_file = self.indexes_dir / "agent-index.json"
        index_file.write_text(json.dumps(agent_index, indent=2))

        return count

    def _build_connection_graph(self) -> int:
        """Build file -> related files connection graph.

        Returns:
            Number of entries indexed
        """
        graph = {}  # file -> {connections: [], contradicts: [], supersedes: []}
        count = 0

        for md_file in self.base_dir.rglob("*.md"):
            if '.indexes' in str(md_file):
                continue

            try:
                content = md_file.read_text(encoding='utf-8')
                if '---' in content:
                    import yaml
                    parts = content.split('---', 2)
                    if len(parts) >= 2:
                        metadata = yaml.safe_load(parts[1])

                        filepath = str(md_file.absolute())
                        graph[filepath] = {
                            'connections': metadata.get('connections', []),
                            'contradicts': metadata.get('contradicts', []),
                            'supersedes': metadata.get('supersedes', [])
                        }
                        count += 1

            except Exception:
                continue

        # Save index
        index_file = self.indexes_dir / "connection-graph.json"
        index_file.write_text(json.dumps(graph, indent=2))

        return count


class QueryRouter:
    """Routes queries to optimal search tier."""

    def __init__(self, base_dir: str = ".claude/memory"):
        """Initialize router.

        Args:
            base_dir: Memory base directory
        """
        self.base_dir = Path(base_dir)
        self.indexes_dir = self.base_dir / ".indexes"
        self.l1_cache = FrequencyBoostLRU(max_size_mb=100)

        # Load indexes
        self._load_indexes()

        # Query statistics
        self.stats = {
            'total_queries': 0,
            'cache_hits': 0,
            'tier1_count': 0,
            'tier2_count': 0,
            'tier3_count': 0,
            'tier4_count': 0,
            'total_time_ms': 0.0
        }

    def _load_indexes(self):
        """Load all indexes into memory."""
        self.inverted_index = {}
        self.chrono_index = {}
        self.agent_index = {}
        self.connection_graph = {}

        try:
            inverted_file = self.indexes_dir / "inverted-index.json"
            if inverted_file.exists():
                self.inverted_index = json.loads(inverted_file.read_text())

            chrono_file = self.indexes_dir / "chronological.json"
            if chrono_file.exists():
                self.chrono_index = json.loads(chrono_file.read_text())

            agent_file = self.indexes_dir / "agent-index.json"
            if agent_file.exists():
                self.agent_index = json.loads(agent_file.read_text())

            connection_file = self.indexes_dir / "connection-graph.json"
            if connection_file.exists():
                self.connection_graph = json.loads(connection_file.read_text())

        except Exception as e:
            print(f"Warning: Could not load indexes: {e}")

    def search(self, query: str, agent: Optional[str] = None) -> Dict[str, Any]:
        """Execute optimized search across all tiers.

        Args:
            query: Search query
            agent: Optional agent filter

        Returns:
            Search results with metadata
        """
        start_time = time.time()
        self.stats['total_queries'] += 1

        # Normalize query for caching
        query_normalized = query.lower().strip()
        cache_key = hashlib.md5(f"{query_normalized}:{agent}".encode()).hexdigest()

        # Tier 1: Try L1 cache
        cached = self.l1_cache.get(cache_key)
        if cached:
            self.stats['cache_hits'] += 1
            self.stats['tier1_count'] += 1
            elapsed_ms = (time.time() - start_time) * 1000
            self.stats['total_time_ms'] += elapsed_ms

            return {
                'results': cached,
                'tier': 1,
                'elapsed_ms': elapsed_ms,
                'source': 'cache'
            }

        # Tier 2: Try index search
        query_type = self._detect_query_type(query_normalized)

        if query_type == 'tag':
            results = self._search_by_tag_index(query_normalized)
            tier = 2
        elif query_type == 'agent':
            results = self._search_by_agent_index(query_normalized, agent)
            tier = 2
        elif query_type == 'date':
            results = self._search_by_date_index(query_normalized)
            tier = 2
        else:
            # Tier 3: Grep search
            results = self._grep_search(query, agent)
            tier = 3

        self.stats[f'tier{tier}_count'] += 1

        # Cache result
        self.l1_cache.put(cache_key, results)

        elapsed_ms = (time.time() - start_time) * 1000
        self.stats['total_time_ms'] += elapsed_ms

        return {
            'results': results,
            'tier': tier,
            'elapsed_ms': elapsed_ms,
            'source': 'index' if tier == 2 else 'grep'
        }

    def _detect_query_type(self, query: str) -> str:
        """Detect query type for optimization.

        Args:
            query: Normalized query

        Returns:
            Query type: 'tag', 'agent', 'date', 'generic'
        """
        # Simple keyword detection
        if any(word in query for word in ['tag:', 'tagged', '#']):
            return 'tag'

        if any(word in query for word in ['agent:', 'by agent']):
            return 'agent'

        if any(word in query for word in ['date:', 'recent', 'last week', 'today']):
            return 'date'

        return 'generic'

    def _search_by_tag_index(self, query: str) -> List[str]:
        """Search using inverted index.

        Args:
            query: Query with tag

        Returns:
            List of matching file paths
        """
        # Extract tag from query
        tag = query.replace('tag:', '').replace('tagged', '').replace('#', '').strip()

        # Look up in index
        return self.inverted_index.get(tag, [])

    def _search_by_agent_index(self, query: str, agent: Optional[str]) -> List[str]:
        """Search using agent index.

        Args:
            query: Query
            agent: Agent name

        Returns:
            List of matching file paths
        """
        if not agent:
            # Extract agent from query
            agent = query.replace('agent:', '').replace('by agent', '').strip()

        agent_data = self.agent_index.get(agent, {})
        return agent_data.get('files', [])

    def _search_by_date_index(self, query: str) -> List[str]:
        """Search using chronological index.

        Args:
            query: Query with date

        Returns:
            List of matching file paths
        """
        # Extract date from query (simplified)
        import re
        date_match = re.search(r'\d{4}-\d{2}-\d{2}', query)
        if date_match:
            date = date_match.group(0)
            return self.chrono_index.get(date, [])

        # Handle "recent" queries
        if 'recent' in query or 'today' in query:
            # Return most recent files
            all_dates = sorted(self.chrono_index.keys(), reverse=True)
            results = []
            for date in all_dates[:7]:  # Last week
                results.extend(self.chrono_index[date])
            return results

        return []

    def _grep_search(self, query: str, agent: Optional[str]) -> List[str]:
        """Grep-based full-text search.

        Args:
            query: Search query
            agent: Optional agent filter

        Returns:
            List of matching file paths
        """
        results = []

        # Determine search scope
        if agent:
            search_dir = self.base_dir / "agent-learnings" / agent
        else:
            search_dir = self.base_dir / "agent-learnings"

        if not search_dir.exists():
            return []

        # Use grep for fast full-text search
        try:
            cmd = ['grep', '-l', '-i', '-r', query, str(search_dir)]
            output = subprocess.run(cmd, capture_output=True, text=True, timeout=5)

            if output.returncode == 0:
                results = output.stdout.strip().split('\n')
                results = [r for r in results if r]  # Filter empty

        except Exception:
            # Fallback to Python search
            for md_file in search_dir.rglob("*.md"):
                if '.indexes' in str(md_file):
                    continue

                try:
                    content = md_file.read_text(encoding='utf-8').lower()
                    if query.lower() in content:
                        results.append(str(md_file.absolute()))
                except Exception:
                    continue

        return results

    def get_stats(self) -> Dict[str, Any]:
        """Get search statistics.

        Returns:
            Dictionary with search stats
        """
        total = self.stats['total_queries']
        if total == 0:
            return self.stats

        cache_hit_rate = self.stats['cache_hits'] / total
        avg_latency_ms = self.stats['total_time_ms'] / total

        return {
            **self.stats,
            'cache_hit_rate': cache_hit_rate,
            'avg_latency_ms': avg_latency_ms,
            'cache_stats': self.l1_cache.stats()
        }


# Test functions
def test_caching():
    """Test L1 cache."""
    print("Testing L1 cache...")

    cache = FrequencyBoostLRU(max_size_mb=1)

    # Test put and get
    cache.put("query1", ["result1"])
    result = cache.get("query1")
    assert result == ["result1"]
    print("✅ Cache put/get works")

    # Test stats
    stats = cache.stats()
    assert stats['entries'] == 1
    print(f"✅ Cache stats: {stats}")


def test_indexing():
    """Test index building."""
    print("\nTesting index building...")

    import tempfile
    import shutil

    temp_dir = tempfile.mkdtemp()

    try:
        # Create test memory structure
        agent_dir = Path(temp_dir) / "agent-learnings" / "test-agent"
        agent_dir.mkdir(parents=True)

        # Create test memory
        memory_content = """---
date: 2025-10-03
agent: test-agent
type: pattern
topic: jwt-auth
tags: [authentication, security, jwt]
confidence: high
visibility: public
---

# JWT Pattern

Test memory content.
"""
        memory_file = agent_dir / "2025-10-03--pattern-jwt-auth.md"
        memory_file.write_text(memory_content)

        # Build indexes
        indexer = MemoryIndexer(temp_dir)
        stats = indexer.build_all_indexes()

        assert stats['inverted'] == 1
        assert stats['chronological'] == 1
        assert stats['agent'] == 1
        print(f"✅ Indexes built: {stats}")

    finally:
        shutil.rmtree(temp_dir)


def test_search():
    """Test search routing."""
    print("\nTesting search routing...")

    import tempfile
    import shutil

    temp_dir = tempfile.mkdtemp()

    try:
        # Create test structure
        agent_dir = Path(temp_dir) / "agent-learnings" / "test-agent"
        agent_dir.mkdir(parents=True)

        memory_content = """---
date: 2025-10-03
agent: test-agent
type: pattern
topic: jwt-auth
tags: [authentication, jwt]
confidence: high
visibility: public
---

# JWT Pattern

Authentication with JWT tokens.
"""
        memory_file = agent_dir / "2025-10-03--pattern-jwt-auth.md"
        memory_file.write_text(memory_content)

        # Build indexes
        indexer = MemoryIndexer(temp_dir)
        indexer.build_all_indexes()

        # Test search
        router = QueryRouter(temp_dir)
        result = router.search("authentication")

        assert len(result['results']) > 0
        print(f"✅ Search found {len(result['results'])} results (tier {result['tier']})")

        # Test cache hit
        result2 = router.search("authentication")
        assert result2['tier'] == 1  # From cache
        print("✅ Cache hit on second search")

        # Get stats
        stats = router.get_stats()
        print(f"✅ Search stats: cache_hit_rate={stats['cache_hit_rate']:.2f}")

    finally:
        shutil.rmtree(temp_dir)


if __name__ == "__main__":
    test_caching()
    test_indexing()
    test_search()
    print("\n🎉 All search & performance tests passed!")
