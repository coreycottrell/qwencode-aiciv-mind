-- Session persistence — Cortex remembers across restarts.
-- Identity IS Continuity (Principle 2)

-- Sessions track each Cortex boot
CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    status          TEXT NOT NULL DEFAULT 'active',  -- active/completed/crashed
    coordination_state TEXT,                         -- JSON: serialized CoordinationState
    boot_count      INTEGER NOT NULL DEFAULT 1,      -- cumulative boot counter
    memory_count    INTEGER NOT NULL DEFAULT 0,      -- memories at session end
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    notes           TEXT                             -- optional human/dream notes
);

CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC);
