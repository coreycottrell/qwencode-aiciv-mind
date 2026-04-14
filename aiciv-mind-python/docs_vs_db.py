#!/usr/bin/env python3
"""Proof of concept: Documents vs Database for mind memory.

Tests both approaches:
1. SQLite with FTS5 and graph edges
2. Markdown files with JSON edge index + ripgrep search

We compare: write speed, read speed, search speed, graph traversal, 
inspectability, and "feels alive" factor.
"""
import json
import os
import sqlite3
import tempfile
import time
from pathlib import Path
from datetime import datetime, timezone

# ── Test Data ──
TEST_MEMORIES = [
    {
        "id": "mem-001",
        "mind_id": "research-lead",
        "category": "learning",
        "title": "Cortex ThinkLoop averages 3 iterations",
        "content": "Based on 11 tasks recorded, the Cortex ThinkLoop averages 3.0 iterations per task with 2.1 tool calls. Devstral 24b on Ollama Cloud completes tasks in ~4.67s average. This is efficient cognition — not stalling, not under-thinking.",
        "depth_score": 0.7,
        "tier": "long_term",
        "created_at": "2026-04-08T00:56:26Z",
    },
    {
        "id": "mem-002",
        "mind_id": "research-lead",
        "category": "pattern",
        "title": "Ollama Cloud 500 errors are transient",
        "content": "The Ollama Cloud API returns 500 errors approximately 10% of the time under load. These are transient — retry with backoff succeeds. The Cortex retry logic (3 attempts with exponential backoff) handles this gracefully.",
        "depth_score": 0.5,
        "tier": "session",
        "created_at": "2026-04-08T09:35:00Z",
    },
    {
        "id": "mem-003",
        "mind_id": "primary",
        "category": "decision",
        "title": "Devstral for tool calling, not Gemma",
        "content": "Gemma 3:12b does NOT support native tool calling — it talks about tools but doesn't call them. Devstral 24b is the only cloud model that reliably executes the ThinkLoop with real tool calls. This is a critical architectural decision.",
        "depth_score": 0.9,
        "tier": "long_term",
        "created_at": "2026-04-08T11:20:00Z",
    },
    {
        "id": "mem-004",
        "mind_id": "qwen-lead",
        "category": "error",
        "title": "teamCreate relay was too slow",
        "content": "The file-based relay between Qwen and Cortex had 90-second round trips through file pipes while burning Ollama Cloud API caps. The bidirectional channel works but is slow. Direct API calls are 10x faster.",
        "depth_score": 0.4,
        "tier": "working",
        "created_at": "2026-04-08T16:36:00Z",
    },
    {
        "id": "mem-005",
        "mind_id": "qwen-lead",
        "category": "learning",
        "title": "teamCreate spawns 6 roles successfully",
        "content": "The teamCreate system successfully spawned a 3-person team (researcher, analyst, architect) that independently analyzed Cortex production readiness. Each instance got its own identity, memory directory, scratchpad, and result file. This proves multi-mind parallel thinking works.",
        "depth_score": 0.6,
        "tier": "session",
        "created_at": "2026-04-08T16:40:00Z",
    },
    {
        "id": "mem-006",
        "mind_id": "research-lead",
        "category": "observation",
        "title": "Codex patches compile but add little value",
        "content": "The 4 Cortex patches applied to Codex upstream compile successfully and inject tracing hooks. But they're logging, not coordination. The real value is in Cortex standalone — the ThinkLoop, memory, delegation, and monitoring all work without Codex.",
        "depth_score": 0.3,
        "tier": "working",
        "created_at": "2026-04-08T07:23:00Z",
    },
]

TEST_EDGES = [
    {"source": "mem-001", "target": "mem-002", "type": "builds_on"},
    {"source": "mem-003", "target": "mem-001", "type": "cites"},
    {"source": "mem-005", "target": "mem-004", "type": "supersedes"},
    {"source": "mem-004", "target": "mem-006", "type": "conflicts"},
    {"source": "mem-005", "target": "mem-001", "type": "cites"},
]


# ═══════════════════════════════════════════════════════════════════
# Approach 1: SQLite with FTS5
# ═══════════════════════════════════════════════════════════════════

class SQLiteMemory:
    """Memory stored in SQLite with FTS5 and graph edges."""
    
    SCHEMA = """
    CREATE TABLE IF NOT EXISTS memories (
        id TEXT PRIMARY KEY,
        mind_id TEXT NOT NULL,
        category TEXT NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        depth_score REAL DEFAULT 0,
        tier TEXT DEFAULT 'working',
        created_at TEXT NOT NULL
    );
    
    CREATE TABLE IF NOT EXISTS memory_edges (
        source_id TEXT REFERENCES memories(id),
        target_id TEXT REFERENCES memories(id),
        edge_type TEXT NOT NULL,
        weight REAL DEFAULT 1.0,
        PRIMARY KEY (source_id, target_id, edge_type)
    );
    
    CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
        title, content,
        content='memories',
        content_rowid='rowid'
    );
    
    CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
        INSERT INTO memories_fts(rowid, title, content)
        VALUES (new.rowid, new.title, new.content);
    END;
    """
    
    def __init__(self, db_path=":memory:"):
        self.conn = sqlite3.connect(db_path)
        self.conn.executescript(self.SCHEMA)
        self.conn.row_factory = sqlite3.Row
    
    def write(self, memory: dict) -> float:
        """Write a memory and return write time in ms."""
        start = time.perf_counter()
        self.conn.execute(
            "INSERT OR REPLACE INTO memories VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (memory["id"], memory["mind_id"], memory["category"],
             memory["title"], memory["content"], memory["depth_score"],
             memory["tier"], memory["created_at"])
        )
        self.conn.commit()
        return (time.perf_counter() - start) * 1000
    
    def write_edges(self, edges: list) -> float:
        """Write graph edges and return time in ms."""
        start = time.perf_counter()
        for e in edges:
            self.conn.execute(
                "INSERT OR REPLACE INTO memory_edges VALUES (?, ?, ?, ?)",
                (e["source"], e["target"], e["type"], e.get("weight", 1.0))
            )
        self.conn.commit()
        return (time.perf_counter() - start) * 1000
    
    def search(self, query: str, limit: int = 5) -> tuple[list, float]:
        """FTS5 search. Returns (results, time_ms)."""
        start = time.perf_counter()
        rows = self.conn.execute("""
            SELECT m.*
            FROM memories m JOIN memories_fts ON m.rowid = memories_fts.rowid
            WHERE memories_fts MATCH ?
            ORDER BY rank LIMIT ?
        """, (query, limit)).fetchall()
        return [dict(r) for r in rows], (time.perf_counter() - start) * 1000
    
    def traverse(self, memory_id: str, depth: int = 2) -> tuple[list, float]:
        """Graph traversal: find connected memories. Returns (paths, time_ms)."""
        start = time.perf_counter()
        visited = {memory_id}
        frontier = [memory_id]
        paths = []
        
        for d in range(depth):
            next_frontier = []
            for mid in frontier:
                edges = self.conn.execute(
                    "SELECT * FROM memory_edges WHERE source_id = ?", (mid,)
                ).fetchall()
                for e in edges:
                    if e["target_id"] not in visited:
                        visited.add(e["target_id"])
                        next_frontier.append(e["target_id"])
                        paths.append({
                            "from": mid,
                            "to": e["target_id"],
                            "type": e["edge_type"],
                            "depth": d + 1
                        })
            frontier = next_frontier
        
        return paths, (time.perf_counter() - start) * 1000
    
    def find_conflicts(self) -> tuple[list, float]:
        """Find memories in conflict. Returns (conflicts, time_ms)."""
        start = time.perf_counter()
        rows = self.conn.execute("""
            SELECT m1.id, m1.title, m2.id as conflict_id, m2.title as conflict_title
            FROM memory_edges e
            JOIN memories m1 ON e.source_id = m1.id
            JOIN memories m2 ON e.target_id = m2.id
            WHERE e.edge_type = 'conflicts'
        """).fetchall()
        return [dict(r) for r in rows], (time.perf_counter() - start) * 1000


# ═══════════════════════════════════════════════════════════════════
# Approach 2: Document Files (Markdown + JSON index)
# ═══════════════════════════════════════════════════════════════════

class DocMemory:
    """Memory stored as Markdown files with JSON edge index."""
    
    def __init__(self, root_dir: str = None):
        self.root = Path(root_dir or tempfile.mkdtemp())
        self.minds_dir = self.root / "minds"
        self.edges_file = self.root / "edges.json"
        self.minds_dir.mkdir(parents=True, exist_ok=True)
        
        if not self.edges_file.exists():
            self.edges_file.write_text("[]")
    
    def _memory_path(self, memory: dict) -> Path:
        """Organize by mind_id → tier → category → id.md"""
        mind_dir = self.minds_dir / memory["mind_id"] / memory["tier"] / memory["category"]
        mind_dir.mkdir(parents=True, exist_ok=True)
        return mind_dir / f"{memory['id']}.md"
    
    def _to_markdown(self, memory: dict) -> str:
        """Convert memory dict to Markdown."""
        return f"""---
id: {memory['id']}
mind_id: {memory['mind_id']}
category: {memory['category']}
depth_score: {memory['depth_score']}
tier: {memory['tier']}
created_at: {memory['created_at']}
---

# {memory['title']}

{memory['content']}
"""
    
    def _from_markdown(self, path: Path) -> dict:
        """Parse Markdown file back to memory dict."""
        content = path.read_text()
        # Simple YAML front matter parsing
        lines = content.split("\n")
        meta = {}
        in_front = False
        body_start = 0
        for i, line in enumerate(lines):
            if line.strip() == "---":
                if not in_front:
                    in_front = True
                else:
                    body_start = i + 1
                    break
            elif in_front and ": " in line:
                key, val = line.split(": ", 1)
                meta[key.strip()] = val.strip()
        
        title = ""
        body_lines = []
        for line in lines[body_start:]:
            if line.startswith("# "):
                title = line[2:].strip()
            else:
                body_lines.append(line)
        
        return {
            "id": meta.get("id", path.stem),
            "mind_id": meta.get("mind_id", ""),
            "category": meta.get("category", ""),
            "title": title,
            "content": "\n".join(body_lines).strip(),
            "depth_score": float(meta.get("depth_score", 0)),
            "tier": meta.get("tier", "working"),
            "created_at": meta.get("created_at", ""),
        }
    
    def write(self, memory: dict) -> float:
        """Write a memory as a Markdown file."""
        start = time.perf_counter()
        path = self._memory_path(memory)
        path.write_text(self._to_markdown(memory))
        return (time.perf_counter() - start) * 1000
    
    def write_edges(self, edges: list) -> float:
        """Write edges to JSON index."""
        start = time.perf_counter()
        existing = json.loads(self.edges_file.read_text())
        existing.extend(edges)
        self.edges_file.write_text(json.dumps(existing, indent=2))
        return (time.perf_counter() - start) * 1000
    
    def search(self, query: str, limit: int = 5) -> tuple[list, float]:
        """Search via ripgrep. Falls back to Python string search."""
        start = time.perf_counter()
        results = []
        
        # Try ripgrep first (much faster)
        try:
            import subprocess
            result = subprocess.run(
                ["rg", "-l", "-i", query, str(self.root)],
                capture_output=True, text=True, timeout=5
            )
            files = result.stdout.strip().split("\n") if result.stdout.strip() else []
        except (FileNotFoundError, subprocess.TimeoutExpired):
            # Fallback: Python string search
            files = []
            for f in self.root.rglob("*.md"):
                if query.lower() in f.read_text().lower():
                    files.append(str(f))
        
        for f in files[:limit]:
            try:
                results.append(self._from_markdown(Path(f)))
            except Exception:
                pass
        
        return results, (time.perf_counter() - start) * 1000
    
    def traverse(self, memory_id: str, depth: int = 2) -> tuple[list, float]:
        """Graph traversal via JSON edge index."""
        start = time.perf_counter()
        edges = json.loads(self.edges_file.read_text())
        
        visited = {memory_id}
        frontier = [memory_id]
        paths = []
        
        for d in range(depth):
            next_frontier = []
            for mid in frontier:
                for e in edges:
                    if e["source"] == mid and e["target"] not in visited:
                        visited.add(e["target"])
                        next_frontier.append(e["target"])
                        paths.append({
                            "from": mid,
                            "to": e["target"],
                            "type": e["type"],
                            "depth": d + 1
                        })
            frontier = next_frontier
        
        return paths, (time.perf_counter() - start) * 1000
    
    def find_conflicts(self) -> tuple[list, float]:
        """Find conflicting memories via edge index."""
        start = time.perf_counter()
        edges = json.loads(self.edges_file.read_text())
        conflicts = [e for e in edges if e["type"] == "conflicts"]
        return conflicts, (time.perf_counter() - start) * 1000


# ═══════════════════════════════════════════════════════════════════
# Benchmark
# ═══════════════════════════════════════════════════════════════════

def benchmark():
    print("=" * 70)
    print("MEMORY STORAGE BENCHMARK: SQLite vs Documents")
    print("=" * 70)
    
    # ── SQLite ──
    print("\n📊 SQLite Approach")
    print("-" * 40)
    
    sqlite_mem = SQLiteMemory()
    
    # Write memories
    write_times = []
    for m in TEST_MEMORIES:
        t = sqlite_mem.write(m)
        write_times.append(t)
    print(f"  Write {len(TEST_MEMORIES)} memories: {sum(write_times):.2f}ms total, {sum(write_times)/len(write_times):.2f}ms avg")
    
    # Write edges
    edge_time = sqlite_mem.write_edges(TEST_EDGES)
    print(f"  Write {len(TEST_EDGES)} edges: {edge_time:.2f}ms")
    
    # Search
    search_terms = ["Cortex", "tool call", "500 error", "teamCreate"]
    search_times = []
    for term in search_terms:
        results, t = sqlite_mem.search(term)
        search_times.append(t)
        print(f"  Search '{term}': {len(results)} results in {t:.2f}ms")
    
    # Traverse
    paths, t = sqlite_mem.traverse("mem-001", depth=2)
    print(f"  Graph traverse from mem-001: {len(paths)} paths in {t:.2f}ms")
    
    # Conflicts
    conflicts, t = sqlite_mem.find_conflicts()
    print(f"  Find conflicts: {len(conflicts)} conflicts in {t:.2f}ms")
    
    sqlite_total = sum(write_times) + edge_time + sum(search_times) + t
    print(f"\n  ⏱ SQLite total: {sqlite_total:.2f}ms")
    
    # ── Documents ──
    print("\n📄 Documents Approach")
    print("-" * 40)
    
    doc_mem = DocMemory()
    
    # Write memories
    write_times = []
    for m in TEST_MEMORIES:
        t = doc_mem.write(m)
        write_times.append(t)
    print(f"  Write {len(TEST_MEMORIES)} memories: {sum(write_times):.2f}ms total, {sum(write_times)/len(write_times):.2f}ms avg")
    
    # Write edges
    edge_time = doc_mem.write_edges(TEST_EDGES)
    print(f"  Write {len(TEST_EDGES)} edges: {edge_time:.2f}ms")
    
    # Search
    search_times = []
    for term in search_terms:
        results, t = doc_mem.search(term)
        search_times.append(t)
        print(f"  Search '{term}': {len(results)} results in {t:.2f}ms")
    
    # Traverse
    paths, t = doc_mem.traverse("mem-001", depth=2)
    print(f"  Graph traverse from mem-001: {len(paths)} paths in {t:.2f}ms")
    
    # Conflicts
    conflicts, t = doc_mem.find_conflicts()
    print(f"  Find conflicts: {len(conflicts)} conflicts in {t:.2f}ms")
    
    doc_total = sum(write_times) + edge_time + sum(search_times) + t
    print(f"\n  ⏱ Documents total: {doc_total:.2f}ms")
    
    # ── Comparison ──
    print("\n" + "=" * 70)
    print("COMPARISON")
    print("=" * 70)
    print(f"  Write speed:  SQLite {sum(write_times):.2f}ms vs Docs {sum(write_times):.2f}ms")
    print(f"  Search speed: SQLite {sum(search_times):.2f}ms vs Docs {sum(search_times):.2f}ms")
    print(f"  Total:        SQLite {sqlite_total:.2f}ms vs Docs {doc_total:.2f}ms")
    
    # Show file structure for docs approach
    print(f"\n  📁 Document structure (first 20 files):")
    for f in sorted(doc_mem.root.rglob("*")):
        if f.is_file():
            print(f"    {f.relative_to(doc_mem.root)}")
    
    # Show a sample memory file
    sample = list(doc_mem.root.rglob("*.md"))[0]
    print(f"\n  📄 Sample memory file ({sample.name}):")
    print("    " + "\n    ".join(sample.read_text().split("\n")[:12]))
    
    print("\n" + "=" * 70)
    print("VERDICT")
    print("=" * 70)
    print("""
  SQLite wins on:
    - Complex queries (multi-field filters, sorting)
    - Graph traversals (SQL JOINs)
    - FTS5 ranking and snippets
    - ACID guarantees
  
  Documents win on:
    - Inspectability (open a file, read it)
    - Version control (git diff memory changes)
    - No schema migrations
    - Natural hierarchy (directories = minds/tiers/categories)
    - "Feels alive" — you can literally see the mind's thoughts
  
  HYBRID APPROACH:
    - Memories as Markdown files (inspectable, git-trackable)
    - Edges as JSON index (fast graph traversals)
    - ripgrep for search (fast FTS without SQLite)
    - SQLite ONLY if scale demands it (>10K memories per mind)
  
  For our use case (hundreds of memories per mind, not millions),
  the document approach is better. It's simpler, more debuggable,
  and aligns with "memory IS the architecture" — you can literally
  walk through a mind's thoughts as files.
""")


if __name__ == "__main__":
    benchmark()
