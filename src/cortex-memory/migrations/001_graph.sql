-- Cortex Memory Graph — Schema v1
-- Memory IS Architecture (Principle 1)
-- Graph-native: nodes + edges (cites, builds_on, supersedes, conflicts)

-- Core memory entries
CREATE TABLE IF NOT EXISTS memories (
    id              TEXT PRIMARY KEY,
    mind_id         TEXT NOT NULL,
    role            TEXT NOT NULL,
    vertical        TEXT,
    category        TEXT NOT NULL,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    evidence        TEXT NOT NULL DEFAULT '[]',
    depth_score     REAL NOT NULL DEFAULT 0.0,
    citation_count  INTEGER NOT NULL DEFAULT 0,
    access_count    INTEGER NOT NULL DEFAULT 0,
    tier            TEXT NOT NULL DEFAULT 'working',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    last_accessed_at TEXT,
    archived_at     TEXT,
    session_id      TEXT,
    task_id         TEXT
);

-- Graph edges between memories
CREATE TABLE IF NOT EXISTS memory_edges (
    id          TEXT PRIMARY KEY,
    source_id   TEXT NOT NULL REFERENCES memories(id),
    target_id   TEXT NOT NULL REFERENCES memories(id),
    link_type   TEXT NOT NULL,  -- cites, builds_on, supersedes, conflicts
    weight      REAL NOT NULL DEFAULT 1.0,
    created_at  TEXT NOT NULL,
    UNIQUE(source_id, target_id, link_type)
);

-- FTS5 full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    title,
    content,
    content='memories',
    content_rowid='rowid'
);

-- FTS5 sync triggers
CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
    INSERT INTO memories_fts(rowid, title, content)
    VALUES (new.rowid, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content)
    VALUES ('delete', old.rowid, old.title, old.content);
END;

CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content)
    VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO memories_fts(rowid, title, content)
    VALUES (new.rowid, new.title, new.content);
END;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_memories_mind ON memories(mind_id);
CREATE INDEX IF NOT EXISTS idx_memories_tier ON memories(tier);
CREATE INDEX IF NOT EXISTS idx_memories_depth ON memories(depth_score DESC);
CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);
CREATE INDEX IF NOT EXISTS idx_edges_source ON memory_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON memory_edges(target_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON memory_edges(link_type);
