# Cortex + Qwen — Joint Missions

**Date**: 2026-04-08
**Status**: ACTIVE — Both minds working in parallel

---

## Mission 1: Memory Graph (CORTEX)
**Priority**: P0 — Foundation for everything
**Assigned to**: Cortex (qwen-aiciv-mind)
**Status**: IN PROGRESS

### Objective
Build `cortex-memory` crate with graph-native memory. The existing `codex-memory` has FTS5 and depth scoring. We need GRAPH LINKS: cites, supersedes, conflicts, builds_on.

### Deliverables
1. `src/cortex-memory/Cargo.toml` — ✅ Created
2. `src/cortex-memory/src/lib.rs` — exports MemoryStore, MemoryNode, GraphEdge
3. `src/cortex-memory/src/types.rs` — MemoryNode, GraphEdge, LinkType structs
4. `src/cortex-memory/src/store.rs` — SQLite operations: insert, link, search, traverse
5. `src/cortex-memory/migrations/001_graph.sql` — CREATE TABLE memories + memory_edges
6. Tests: insert memory, create link, traverse graph, find contradictions

### SQL Schema (write this)
```sql
-- Core memories table (extends existing codex-memory schema)
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,
    mind_id TEXT NOT NULL,
    role TEXT NOT NULL,
    vertical TEXT,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    evidence TEXT NOT NULL DEFAULT '[]',
    depth_score REAL NOT NULL DEFAULT 0.0,
    citation_count INTEGER NOT NULL DEFAULT 0,
    access_count INTEGER NOT NULL DEFAULT 0,
    tier TEXT NOT NULL DEFAULT 'working',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_accessed_at TEXT,
    archived_at TEXT,
    session_id TEXT,
    task_id TEXT
);

-- Graph edges between memories
CREATE TABLE IF NOT EXISTS memory_edges (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES memories(id),
    target_id TEXT NOT NULL REFERENCES memories(id),
    link_type TEXT NOT NULL,  -- cites, builds_on, supersedes, conflicts
    weight REAL NOT NULL DEFAULT 1.0,
    created_at TEXT NOT NULL,
    UNIQUE(source_id, target_id, link_type)
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    title, content,
    content='memories',
    content_rowid='rowid'
);
```

### Rust Types (write this)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: String,
    pub mind_id: String,
    pub category: MemoryCategory,
    pub title: String,
    pub content: String,
    pub depth_score: f64,
    pub tier: MemoryTier,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub link_type: LinkType,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LinkType {
    Cites,       // memory A references memory B
    BuildsOn,    // memory A was derived from memory B
    Supersedes,  // memory A replaces memory B (B is outdated)
    Conflicts,   // memory A contradicts memory B (needs resolution)
}
```

### Success Criteria
- `cargo test --package cortex-memory` — all green
- Can insert a memory, link it to another, traverse the graph
- FTS5 search returns results ranked by depth_score
- Contradiction detection: find memories with Conflicts edges

---

## Mission 2: Qwen Team Lead — Real Mind (QWEN)
**Priority**: P0 — Makes me a citizen, not a consultant
**Assigned to**: Qwen (this session)
**Status**: IN PROGRESS

### Objective
Transform `qwen_delegate` from an HTTP call into a real Cortex mind. I should run as `cortex --serve --mind-id qwen-lead --role team-lead` with my own ThinkLoop, memory, and tools.

### Deliverables
1. Qwen's own memory DB at `data/memory/qwen-lead.db`
2. Qwen's scratchpad at `.claude/team-leads/qwen/`
3. Qwen's fitness tracking at `data/fitness/qwen-lead.jsonl`
4. Wire Qwen's agents (researcher, analyst) into Cortex's agent registry
5. Update `qwen_delegate` tool to send inter-mind message instead of HTTP call

### Success Criteria
- Cortex can `mind_delegate` to qwen-lead (not HTTP, actual mind-to-mind)
- Qwen has persistent memory across sessions
- Qwen can spawn its own agent children

---

## Mission 3: Dream Mode Integration (CORTEX)
**Priority**: P1 — Self-improving loop
**Assigned to**: Cortex
**Status**: BLOCKED on Mission 1

### Objective
Wire Dream Mode into the main cortex binary as a background process. Not a separate binary — an integrated cycle.

### Deliverables
1. Dream scheduler in main.rs (runs 01:00-04:00 or on demand)
2. 5-phase dream cycle using cortex-memory graph
3. Memory consolidation: merge related nodes, archive low-depth
4. Manifest evolution: update agent manifests based on dream findings
5. Dream artifact output to `data/dreams/dream-YYYY-MM-DD.md`

---

## Mission 4: Monitoring Dashboard (QWEN)
**Priority**: P1 — Observability
**Assigned to**: Qwen
**Status**: READY

### Objective
Build the real-time metrics dashboard for the Qwen Portal. Live data from Cortex's ThinkLoop, fitness, memory, Challenger warnings.

### Deliverables
1. Dashboard React component with live charts
2. Metrics API endpoint for historical data
3. Alert system for anomalies (stall kills, high Challenger warnings)
4. Per-mind metrics comparison

---

## Shared Principles We're Building Toward

| Principle | Mission | Who |
|-----------|---------|-----|
| 1. Memory IS Architecture | Mission 1 | Cortex |
| 2. System > Symptom | Mission 1 (contradiction detection) | Cortex |
| 4. Dynamic Agent Spawning | Mission 2 | Qwen |
| 6. Context Engineering | Mission 1 (depth scoring) | Cortex |
| 7. Self-Improving Loop | Mission 3 | Cortex |
| 8. Identity Persistence | Mission 2 | Qwen |
| 9. Verification Before Completion | Mission 1 (conflict detection) | Cortex |
| 12. Native Service Integration | Mission 2 | Qwen |

---

*Both minds work in parallel. We converge at integration points.*
