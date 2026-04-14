-- Cortex Memory Graph — Schema v1
-- Memory IS Architecture (Principle 1)

-- Core memory entries
CREATE TABLE IF NOT EXISTS memories (
    id          TEXT PRIMARY KEY,
    mind_id     TEXT NOT NULL,           -- which mind created this
    role        TEXT NOT NULL,           -- primary/team_lead/agent
    vertical    TEXT,                    -- research/code/memory/comms/ops/context

    -- Content
    category    TEXT NOT NULL,           -- learning/pattern/decision/observation/error/context
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    evidence    TEXT,                    -- JSON array of evidence strings

    -- Scoring
    depth_score REAL NOT NULL DEFAULT 0.0,   -- how deep/important (0.0 = ephemeral, 1.0 = foundational)
    citation_count INTEGER NOT NULL DEFAULT 0, -- how many times cited by other memories/minds
    access_count   INTEGER NOT NULL DEFAULT 0, -- how many times read/searched

    -- Lifecycle
    tier        TEXT NOT NULL DEFAULT 'working',  -- working/session/long_term/archived
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    last_accessed_at TEXT,
    archived_at TEXT,

    -- Session context
    session_id  TEXT,
    task_id     TEXT
);

-- Graph links between memories
CREATE TABLE IF NOT EXISTS memory_links (
    id          TEXT PRIMARY KEY,
    source_id   TEXT NOT NULL REFERENCES memories(id),
    target_id   TEXT NOT NULL REFERENCES memories(id),
    link_type   TEXT NOT NULL,  -- cites/builds_on/contradicts/supersedes/related
    strength    REAL NOT NULL DEFAULT 1.0,  -- 0.0-1.0, decays over time
    created_at  TEXT NOT NULL,

    UNIQUE(source_id, target_id, link_type)
);

-- Full-text search index
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    title,
    content,
    category,
    content='memories',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
    INSERT INTO memories_fts(rowid, title, content, category)
    VALUES (new.rowid, new.title, new.content, new.category);
END;

CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content, category)
    VALUES ('delete', old.rowid, old.title, old.content, old.category);
END;

CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content, category)
    VALUES ('delete', old.rowid, old.title, old.content, old.category);
    INSERT INTO memories_fts(rowid, title, content, category)
    VALUES (new.rowid, new.title, new.content, new.category);
END;

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_memories_mind ON memories(mind_id);
CREATE INDEX IF NOT EXISTS idx_memories_tier ON memories(tier);
CREATE INDEX IF NOT EXISTS idx_memories_depth ON memories(depth_score DESC);
CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);
CREATE INDEX IF NOT EXISTS idx_memories_session ON memories(session_id);
CREATE INDEX IF NOT EXISTS idx_memory_links_source ON memory_links(source_id);
CREATE INDEX IF NOT EXISTS idx_memory_links_target ON memory_links(target_id);
CREATE INDEX IF NOT EXISTS idx_memory_links_type ON memory_links(link_type);
