//! MemoryStore — SQLite-backed memory graph with graph edges.
//!
//! Provides:
//! - Memory node CRUD with depth scoring and lifecycle tiers
//! - Graph edges: cites, builds_on, supersedes, conflicts
//! - FTS5 full-text search
//! - Graph traversal (multi-hop)
//! - Contradiction detection (find Conflicts edges)
//! - Archive candidates by depth threshold

use chrono::{DateTime, Utc};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::types::*;

/// Errors from the memory store.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("memory not found: {0}")]
    NotFound(String),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

/// SQLite-backed memory graph.
pub struct MemoryStore {
    pool: SqlitePool,
}

impl MemoryStore {
    /// Open (or create) a memory database at the given path.
    pub async fn new(db_path: &str) -> Result<Self, MemoryError> {
        let url = if db_path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{db_path}?mode=rwc")
        };

        let pool = SqlitePool::connect(&url).await?;

        // Run schema migrations (raw_sql handles multi-statement SQL including triggers)
        sqlx::raw_sql(include_str!("../migrations/001_graph.sql"))
            .execute(&pool)
            .await?;

        info!("cortex-memory initialized at {db_path}");
        Ok(Self { pool })
    }

    // ── CRUD ────────────────────────────────────────────────────────────────

    /// Store a new memory. Returns the generated ID.
    pub async fn store(&self, mem: NewMemory) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let evidence = serde_json::to_string(&mem.evidence)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
        let category = mem.category.to_string();
        let tier = mem.tier.to_string();

        sqlx::query(
            "INSERT INTO memories (id, mind_id, role, vertical, category, title, content, \
             evidence, depth_score, citation_count, access_count, tier, created_at, updated_at, \
             session_id, task_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0.0, 0, 0, ?9, ?10, ?10, ?11, ?12)",
        )
        .bind(&id)
        .bind(&mem.mind_id)
        .bind(&mem.role)
        .bind(&mem.vertical)
        .bind(&category)
        .bind(&mem.title)
        .bind(&mem.content)
        .bind(&evidence)
        .bind(&tier)
        .bind(&now)
        .bind(&mem.session_id)
        .bind(&mem.task_id)
        .execute(&self.pool)
        .await?;

        debug!("Stored memory {id}: {}", mem.title);
        Ok(id)
    }

    /// Retrieve a memory by ID. Increments access_count.
    pub async fn get(&self, id: &str) -> Result<MemoryNode, MemoryError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE memories SET access_count = access_count + 1, last_accessed_at = ?1 \
             WHERE id = ?2",
        )
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM memories WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))?;

        row_to_node(&row)
    }

    /// Update an existing memory's content.
    pub async fn update(&self, id: &str, title: &str, content: &str) -> Result<(), MemoryError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE memories SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
        )
        .bind(title)
        .bind(content)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Search ──────────────────────────────────────────────────────────────

    /// Search memories using FTS5 and/or column filters.
    pub async fn search(&self, query: &MemoryQuery) -> Result<Vec<SearchResult>, MemoryError> {
        let limit = query.limit.unwrap_or(20);

        if let Some(text) = &query.text {
            // FTS5 search — escape each term to avoid operator interpretation
            let escaped = text
                .split_whitespace()
                .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(" ");

            let mut sql = String::from(
                "SELECT m.*, fts.rank FROM memories m \
                 JOIN memories_fts fts ON m.rowid = fts.rowid \
                 WHERE memories_fts MATCH ?1",
            );
            let mut binds: Vec<String> = vec![escaped];
            let mut idx = 2;

            if let Some(mind_id) = &query.mind_id {
                sql.push_str(&format!(" AND m.mind_id = ?{idx}"));
                binds.push(mind_id.clone());
                idx += 1;
            }
            if let Some(cat) = &query.category {
                sql.push_str(&format!(" AND m.category = ?{idx}"));
                binds.push(cat.to_string());
                idx += 1;
            }
            if let Some(tier) = &query.tier {
                sql.push_str(&format!(" AND m.tier = ?{idx}"));
                binds.push(tier.to_string());
                idx += 1;
            }
            if let Some(min_depth) = query.min_depth {
                sql.push_str(&format!(" AND m.depth_score >= ?{idx}"));
                binds.push(min_depth.to_string());
                idx += 1;
            }

            sql.push_str(&format!(" ORDER BY fts.rank LIMIT ?{idx}"));
            binds.push(limit.to_string());

            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = q.bind(b);
            }

            let rows = q.fetch_all(&self.pool).await?;
            rows.iter()
                .map(|row| {
                    let memory = row_to_node(row)?;
                    let rank: f64 = row.try_get("rank").unwrap_or(0.0);
                    let relevance = 1.0 / (1.0 + rank.abs());
                    Ok(SearchResult { memory, relevance })
                })
                .collect()
        } else {
            // Filter-only search (no FTS)
            let mut sql = String::from("SELECT * FROM memories WHERE 1=1");
            let mut binds: Vec<String> = vec![];
            let mut idx = 1;

            if let Some(mind_id) = &query.mind_id {
                sql.push_str(&format!(" AND mind_id = ?{idx}"));
                binds.push(mind_id.clone());
                idx += 1;
            }
            if let Some(cat) = &query.category {
                sql.push_str(&format!(" AND category = ?{idx}"));
                binds.push(cat.to_string());
                idx += 1;
            }
            if let Some(tier) = &query.tier {
                sql.push_str(&format!(" AND tier = ?{idx}"));
                binds.push(tier.to_string());
                idx += 1;
            }
            if let Some(min_depth) = query.min_depth {
                sql.push_str(&format!(" AND depth_score >= ?{idx}"));
                binds.push(min_depth.to_string());
                idx += 1;
            }

            sql.push_str(&format!(" ORDER BY depth_score DESC LIMIT ?{idx}"));
            binds.push(limit.to_string());

            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = q.bind(b);
            }

            let rows = q.fetch_all(&self.pool).await?;
            rows.iter()
                .map(|row| {
                    let memory = row_to_node(row)?;
                    Ok(SearchResult {
                        memory: memory.clone(),
                        relevance: memory.depth_score,
                    })
                })
                .collect()
        }
    }

    // ── Graph Edges ─────────────────────────────────────────────────────────

    /// Create a graph edge between two memories.
    pub async fn edge(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: LinkType,
        weight: f64,
    ) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let lt = link_type.to_string();

        sqlx::query(
            "INSERT OR REPLACE INTO memory_edges (id, source_id, target_id, link_type, weight, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&id)
        .bind(source_id)
        .bind(target_id)
        .bind(&lt)
        .bind(weight)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        debug!("Edge: {source_id} --{lt}[{weight}]--> {target_id}");
        Ok(id)
    }

    /// Cite a memory: creates a Cites edge, increments citation_count, boosts depth_score.
    ///
    /// Authorization: `mind_id` must match the `mind_id` of the citer memory.
    /// A mind can only cite FROM its own memories. The cited memory can belong to any mind.
    pub async fn cite(&self, citer_id: &str, cited_id: &str, mind_id: &str) -> Result<(), MemoryError> {
        // Verify the citer memory exists and belongs to the calling mind
        let citer_row = sqlx::query("SELECT mind_id FROM memories WHERE id = ?1")
            .bind(citer_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| MemoryError::NotFound(format!("citer memory not found: {citer_id}")))?;

        let citer_mind: String = citer_row
            .try_get("mind_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?;

        if citer_mind != mind_id {
            warn!(
                mind_id = %mind_id,
                citer_id = %citer_id,
                citer_owner = %citer_mind,
                "Unauthorized cite attempt: mind does not own citer memory"
            );
            return Err(MemoryError::Unauthorized(format!(
                "mind '{mind_id}' cannot cite from memory '{citer_id}' owned by '{citer_mind}'"
            )));
        }

        // Verify the cited memory exists
        let _cited_exists = sqlx::query("SELECT id FROM memories WHERE id = ?1")
            .bind(cited_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| MemoryError::NotFound(format!("cited memory not found: {cited_id}")))?;

        self.edge(citer_id, cited_id, LinkType::Cites, 1.0).await?;

        sqlx::query(
            "UPDATE memories SET \
             citation_count = citation_count + 1, \
             depth_score = MIN(1.0, depth_score + 0.1), \
             updated_at = ?1 \
             WHERE id = ?2",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(cited_id)
        .execute(&self.pool)
        .await?;

        debug!("Cited {cited_id} from {citer_id} (mind: {mind_id})");
        Ok(())
    }

    /// Build on a memory: creates BuildsOn edge.
    pub async fn build_on(&self, source_id: &str, target_id: &str) -> Result<(), MemoryError> {
        self.edge(source_id, target_id, LinkType::BuildsOn, 1.0)
            .await?;
        Ok(())
    }

    /// Supersede a memory: creates Supersedes edge.
    pub async fn supersede(&self, new_id: &str, old_id: &str) -> Result<(), MemoryError> {
        self.edge(new_id, old_id, LinkType::Supersedes, 1.0)
            .await?;
        Ok(())
    }

    /// Flag a conflict: creates a Conflicts edge between two memories.
    pub async fn flag_conflict(&self, a_id: &str, b_id: &str) -> Result<(), MemoryError> {
        // Bidirectional: both edges
        self.edge(a_id, b_id, LinkType::Conflicts, 1.0).await?;
        self.edge(b_id, a_id, LinkType::Conflicts, 1.0).await?;
        Ok(())
    }

    // ── Graph Traversal ─────────────────────────────────────────────────────

    /// Get all edges involving a memory.
    pub async fn get_edges(&self, memory_id: &str) -> Result<Vec<GraphEdge>, MemoryError> {
        let rows = sqlx::query(
            "SELECT * FROM memory_edges WHERE source_id = ?1 OR target_id = ?1",
        )
        .bind(memory_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_edge).collect()
    }

    /// Get edges filtered by link type.
    pub async fn get_edges_by_type(
        &self,
        memory_id: &str,
        link_type: LinkType,
    ) -> Result<Vec<GraphEdge>, MemoryError> {
        let lt = link_type.to_string();
        let rows = sqlx::query(
            "SELECT * FROM memory_edges \
             WHERE (source_id = ?1 OR target_id = ?1) AND link_type = ?2",
        )
        .bind(memory_id)
        .bind(&lt)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_edge).collect()
    }

    /// Traverse the graph from a starting memory, following edges.
    pub async fn traverse(
        &self,
        query: &GraphQuery,
    ) -> Result<TraversalResult, MemoryError> {
        let mut visited = std::collections::HashSet::new();
        let mut path: Vec<MemoryNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // Start node
        let start = self.get(&query.memory_id).await?;
        path.push(start.clone());
        visited.insert(query.memory_id.clone());

        let mut frontier: Vec<(String, u32)> = vec![(query.memory_id.clone(), 0)];

        while let Some((current_id, depth)) = frontier.pop() {
            if depth >= query.max_depth || path.len() >= query.limit as usize {
                continue;
            }

            // Get outgoing or incoming edges
            let all_edges = self.get_edges(&current_id).await?;
            let filtered: Vec<&GraphEdge> = all_edges.iter().filter(|e| {
                if let Some(ref types) = query.link_types {
                    types.contains(&e.link_type)
                } else {
                    true
                }
            }).collect();

            for edge in filtered {
                let next_id = match query.direction {
                    TraversalDirection::Outgoing => {
                        if edge.source_id == current_id {
                            &edge.target_id
                        } else {
                            continue;
                        }
                    }
                    TraversalDirection::Incoming => {
                        if edge.target_id == current_id {
                            &edge.source_id
                        } else {
                            continue;
                        }
                    }
                    TraversalDirection::Both => {
                        if edge.source_id == current_id {
                            &edge.target_id
                        } else if edge.target_id == current_id {
                            &edge.source_id
                        } else {
                            continue;
                        }
                    }
                };

                if !visited.contains(next_id) {
                    visited.insert(next_id.clone());
                    edges.push(edge.clone());
                    if let Ok(node) = self.get(next_id).await {
                        path.push(node);
                        frontier.push((next_id.clone(), depth + 1));
                    }
                }
            }
        }

        let total_depth = path.len() as u32;
        Ok(TraversalResult {
            path,
            edges,
            total_depth,
        })
    }

    /// Find all contradictions: pairs of memories linked by Conflicts edges.
    pub async fn find_contradictions(
        &self,
        limit: i64,
    ) -> Result<Vec<(MemoryNode, MemoryNode, GraphEdge)>, MemoryError> {
        let rows = sqlx::query(
            "SELECT e.*, s.title as source_title, t.title as target_title \
             FROM memory_edges e \
             JOIN memories s ON e.source_id = s.id \
             JOIN memories t ON e.target_id = t.id \
             WHERE e.link_type = 'conflicts' \
             LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in &rows {
            let source_id: String = row.try_get("source_id")?;
            let target_id: String = row.try_get("target_id")?;
            let source = self.get(&source_id).await?;
            let target = self.get(&target_id).await?;
            let edge = row_to_edge(row)?;
            results.push((source, target, edge));
        }

        Ok(results)
    }

    // ── Lifecycle ───────────────────────────────────────────────────────────

    /// Promote a memory to the next lifecycle tier.
    /// Working → Validated → (cannot promote further; use archive explicitly)
    pub async fn promote(&self, id: &str) -> Result<MemoryTier, MemoryError> {
        let row = sqlx::query("SELECT tier FROM memories WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))?;

        let current: String = row
            .try_get("tier")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
        let current_tier: MemoryTier = current.parse().map_err(MemoryError::InvalidData)?;

        let next = match current_tier {
            MemoryTier::Working => MemoryTier::Validated,
            MemoryTier::Validated | MemoryTier::Archived => {
                return Err(MemoryError::InvalidData(format!(
                    "Cannot promote from {current_tier}"
                )));
            }
        };

        sqlx::query("UPDATE memories SET tier = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(next.to_string())
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await?;

        debug!("Promoted {id}: {current_tier} -> {next}");
        Ok(next)
    }

    /// Find memories below a depth threshold — candidates for archival.
    pub async fn archive_candidates(
        &self,
        max_depth: f64,
        limit: i64,
    ) -> Result<Vec<MemoryNode>, MemoryError> {
        let rows = sqlx::query(
            "SELECT * FROM memories WHERE tier != 'archived' AND depth_score < ?1 \
             ORDER BY depth_score ASC, access_count ASC LIMIT ?2",
        )
        .bind(max_depth)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_node).collect()
    }

    /// Archive a memory.
    pub async fn archive(&self, id: &str) -> Result<(), MemoryError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE memories SET tier = 'archived', archived_at = ?1, updated_at = ?1 \
             WHERE id = ?2",
        )
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        debug!("Archived memory {id}");
        Ok(())
    }

    // ── Stats ───────────────────────────────────────────────────────────────

    /// Count total memories.
    pub async fn count(&self) -> Result<i64, MemoryError> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM memories")
            .fetch_one(&self.pool)
            .await?;
        let cnt: i64 = row
            .try_get("cnt")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
        Ok(cnt)
    }

    /// Count edges by type.
    pub async fn edge_counts(&self) -> Result<std::collections::HashMap<String, i64>, MemoryError>
    {
        let rows = sqlx::query(
            "SELECT link_type, COUNT(*) as cnt FROM memory_edges GROUP BY link_type",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map = std::collections::HashMap::new();
        for row in &rows {
            let lt: String = row.try_get("link_type")?;
            let cnt: i64 = row.try_get("cnt")?;
            map.insert(lt, cnt);
        }
        Ok(map)
    }

    // ── Session Persistence ─────────────────────────────────────────────────

    /// Start a new session. Returns the session ID.
    pub async fn start_session(&self, notes: Option<&str>) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Create sessions table if not exists (not in 001_graph.sql)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'active',
                boot_count INTEGER NOT NULL DEFAULT 1,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                memory_count INTEGER DEFAULT 0,
                coordination_state TEXT,
                notes TEXT
            )",
        )
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT COALESCE(MAX(boot_count), 0) as bc FROM sessions")
            .fetch_optional(&self.pool)
            .await?;
        let prev_boots: i64 = row.and_then(|r| r.try_get("bc").ok()).unwrap_or(0);

        sqlx::query(
            "INSERT INTO sessions (id, status, boot_count, started_at, notes) \
             VALUES (?1, 'active', ?2, ?3, ?4)",
        )
        .bind(&id)
        .bind(prev_boots + 1)
        .bind(&now)
        .bind(notes)
        .execute(&self.pool)
        .await?;

        info!(session_id = %id, boot = prev_boots + 1, "Session started");
        Ok(id)
    }

    /// End the current session.
    pub async fn end_session(
        &self,
        session_id: &str,
        coordination_state_json: &str,
    ) -> Result<(), MemoryError> {
        let now = Utc::now().to_rfc3339();
        let mem_count = self.count().await?;

        sqlx::query(
            "UPDATE sessions SET status = 'completed', coordination_state = ?1, \
             memory_count = ?2, ended_at = ?3 WHERE id = ?4",
        )
        .bind(coordination_state_json)
        .bind(mem_count)
        .bind(&now)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        info!(session_id = %session_id, memories = mem_count, "Session ended");
        Ok(())
    }

    /// Load the latest completed session.
    pub async fn load_latest_session(
        &self,
    ) -> Result<Option<SavedSession>, MemoryError> {
        let row = sqlx::query(
            "SELECT id, boot_count, coordination_state, started_at, ended_at \
             FROM sessions WHERE status = 'completed' AND coordination_state IS NOT NULL \
             ORDER BY ended_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let id: String = r.try_get("id")?;
                let boot_count: i64 = r.try_get("boot_count")?;
                let state: String = r.try_get("coordination_state")?;
                let started: String = r.try_get("started_at")?;
                let ended: Option<String> = r.try_get("ended_at").ok().flatten();

                Ok(Some(SavedSession {
                    id,
                    boot_count,
                    coordination_state_json: state,
                    started_at: parse_dt(&started)?,
                    ended_at: ended.and_then(|s| parse_dt(&s).ok()),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get total boot count.
    /// Returns 0 if the sessions table does not yet exist (lazy creation in start_session).
    pub async fn boot_count(&self) -> Result<i64, MemoryError> {
        let row = sqlx::query("SELECT COALESCE(MAX(boot_count), 0) as bc FROM sessions")
            .fetch_optional(&self.pool)
            .await;
        match row {
            Ok(Some(r)) => {
                let bc: i64 = r.try_get("bc").unwrap_or(0);
                Ok(bc)
            }
            Ok(None) => Ok(0),
            Err(_) => {
                // Table may not exist yet (created lazily in start_session)
                Ok(0)
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn parse_dt(s: &str) -> Result<DateTime<Utc>, MemoryError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MemoryError::InvalidData(e.to_string()))
}

fn row_to_node(row: &SqliteRow) -> Result<MemoryNode, MemoryError> {
    let evidence_json: String = row
        .try_get("evidence")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let evidence: Vec<String> =
        serde_json::from_str(&evidence_json).unwrap_or_default();

    let category_str: String = row.try_get("category")?;
    let tier_str: String = row.try_get("tier")?;
    let created_str: String = row.try_get("created_at")?;
    let updated_str: String = row.try_get("updated_at")?;
    let last_accessed_str: Option<String> = row.try_get("last_accessed_at").ok().flatten();
    let archived_str: Option<String> = row.try_get("archived_at").ok().flatten();

    Ok(MemoryNode {
        id: row.try_get("id")?,
        mind_id: row.try_get("mind_id")?,
        role: row.try_get("role")?,
        vertical: row.try_get("vertical").ok().flatten(),
        category: category_str.parse().map_err(MemoryError::InvalidData)?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        evidence,
        depth_score: row.try_get("depth_score")?,
        citation_count: row.try_get("citation_count")?,
        access_count: row.try_get("access_count")?,
        tier: tier_str.parse().map_err(MemoryError::InvalidData)?,
        created_at: parse_dt(&created_str)?,
        updated_at: parse_dt(&updated_str)?,
        last_accessed_at: last_accessed_str.and_then(|s| parse_dt(&s).ok()),
        archived_at: archived_str.and_then(|s| parse_dt(&s).ok()),
        session_id: row.try_get("session_id").ok().flatten(),
        task_id: row.try_get("task_id").ok().flatten(),
    })
}

fn row_to_edge(row: &SqliteRow) -> Result<GraphEdge, MemoryError> {
    let lt_str: String = row.try_get("link_type")?;
    let created_str: String = row.try_get("created_at")?;

    Ok(GraphEdge {
        id: row.try_get("id")?,
        source_id: row.try_get("source_id")?,
        target_id: row.try_get("target_id")?,
        link_type: lt_str.parse().map_err(MemoryError::InvalidData)?,
        weight: row.try_get("weight")?,
        created_at: parse_dt(&created_str)?,
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store() -> MemoryStore {
        MemoryStore::new(":memory:").await.unwrap()
    }

    fn test_memory() -> NewMemory {
        NewMemory {
            mind_id: "cortex-primary".into(),
            role: "primary".into(),
            vertical: Some("memory".into()),
            category: MemoryCategory::Learning,
            title: "Depth scoring compounds".into(),
            content: "Cited memories grow deeper. Uncited memories fade.".into(),
            evidence: vec!["obs-001".into()],
            tier: MemoryTier::Working,
            session_id: Some("sess-001".into()),
            task_id: None,
        }
    }

    #[tokio::test]
    async fn store_and_get() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();
        let mem = store.get(&id).await.unwrap();

        assert_eq!(mem.title, "Depth scoring compounds");
        assert_eq!(mem.mind_id, "cortex-primary");
        assert_eq!(mem.access_count, 1);
    }

    #[tokio::test]
    async fn search_fts() {
        let store = test_store().await;
        store.store(test_memory()).await.unwrap();
        store
            .store(NewMemory {
                title: "Unrelated".into(),
                content: "Something else entirely.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        let results = store
            .search(&MemoryQuery {
                text: Some("depth scoring".into()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memory.title, "Depth scoring compounds");
        assert!(results[0].relevance > 0.0);
    }

    #[tokio::test]
    async fn search_filter_only() {
        let store = test_store().await;
        store.store(test_memory()).await.unwrap();
        store
            .store(NewMemory {
                category: MemoryCategory::Decision,
                title: "Chose SQLite".into(),
                content: "SQLite for embedded memory.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        let results = store
            .search(&MemoryQuery {
                category: Some(MemoryCategory::Decision),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memory.title, "Chose SQLite");
    }

    #[tokio::test]
    async fn cite_boosts_depth() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap();
        let id2 = store
            .store(NewMemory {
                title: "Citer".into(),
                content: "I cite depth scoring.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store.cite(&id2, &id1, "cortex-primary").await.unwrap();
        let mem = store.get(&id1).await.unwrap();

        assert_eq!(mem.citation_count, 1);
        assert!((mem.depth_score - 0.1).abs() < f64::EPSILON);

        // Cite again — compounds
        let id3 = store
            .store(NewMemory {
                title: "Second citer".into(),
                content: "Also cites.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        store.cite(&id3, &id1, "cortex-primary").await.unwrap();
        let mem = store.get(&id1).await.unwrap();

        assert_eq!(mem.citation_count, 2);
        assert!((mem.depth_score - 0.2).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn create_and_traverse_graph() {
        let store = test_store().await;
        let a = store
            .store(NewMemory {
                title: "A: Foundation".into(),
                content: "Base knowledge.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        let b = store
            .store(NewMemory {
                title: "B: Extension".into(),
                content: "Builds on A.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        let c = store
            .store(NewMemory {
                title: "C: Further".into(),
                content: "Builds on B.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store.build_on(&b, &a).await.unwrap();
        store.build_on(&c, &b).await.unwrap();

        // Traverse from A — edges go B→A and C→B (B builds on A, C builds on B)
        // Use Incoming direction to follow from A back through its builders
        let traversal = store
            .traverse(&GraphQuery {
                memory_id: a.clone(),
                link_types: Some(vec![LinkType::BuildsOn]),
                direction: TraversalDirection::Incoming,
                max_depth: 5,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(traversal.path.len(), 3); // A → B → C
        assert_eq!(traversal.edges.len(), 2);
    }

    #[tokio::test]
    async fn find_contradictions() {
        let store = test_store().await;
        let a = store
            .store(NewMemory {
                title: "Claim A".into(),
                content: "X is true.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        let b = store
            .store(NewMemory {
                title: "Claim B".into(),
                content: "X is false.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store.flag_conflict(&a, &b).await.unwrap();

        let contradictions = store.find_contradictions(10).await.unwrap();
        assert!(!contradictions.is_empty());
        assert_eq!(contradictions[0].0.title, "Claim A");
        assert_eq!(contradictions[0].1.title, "Claim B");
    }

    #[tokio::test]
    async fn promote_tiers() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();

        let t1 = store.promote(&id).await.unwrap();
        assert_eq!(t1, MemoryTier::Validated);

        // Cannot promote past Validated
        assert!(store.promote(&id).await.is_err());
    }

    #[tokio::test]
    async fn archive_low_depth() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();

        let candidates = store.archive_candidates(0.5, 10).await.unwrap();
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].id, id);

        store.archive(&id).await.unwrap();
        let mem = store.get(&id).await.unwrap();
        assert_eq!(mem.tier, MemoryTier::Archived);
    }

    #[tokio::test]
    async fn edge_counts() {
        let store = test_store().await;
        let a = store.store(test_memory()).await.unwrap();
        let b = store
            .store(NewMemory {
                title: "B".into(),
                content: "Something.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store.cite(&b, &a, "cortex-primary").await.unwrap();
        store.build_on(&b, &a).await.unwrap();

        let counts = store.edge_counts().await.unwrap();
        assert_eq!(*counts.get("cites").unwrap_or(&0), 1);
        assert_eq!(*counts.get("builds_on").unwrap_or(&0), 1);
    }

    #[tokio::test]
    async fn update_memory() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();
        store
            .update(&id, "Updated Title", "New content here.")
            .await
            .unwrap();
        let mem = store.get(&id).await.unwrap();
        assert_eq!(mem.title, "Updated Title");
        assert_eq!(mem.content, "New content here.");
    }

    #[tokio::test]
    async fn cite_authorized_same_mind() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap();
        let id2 = store
            .store(NewMemory {
                title: "Citer from same mind".into(),
                content: "Same mind citing.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        // Should succeed: both memories belong to "cortex-primary"
        let result = store.cite(&id2, &id1, "cortex-primary").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn cite_unauthorized_wrong_mind() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap(); // owned by "cortex-primary"
        let id2 = store
            .store(NewMemory {
                title: "Another memory".into(),
                content: "Also owned by cortex-primary.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        // Should fail: "evil-mind" does not own the citer memory
        let result = store.cite(&id2, &id1, "evil-mind").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, MemoryError::Unauthorized(_)),
            "Expected Unauthorized error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn cite_cross_mind_allowed_when_owned() {
        let store = test_store().await;
        // Memory owned by "mind-alpha"
        let alpha_mem = store
            .store(NewMemory {
                mind_id: "mind-alpha".into(),
                title: "Alpha's insight".into(),
                content: "Something worth citing.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        // Memory owned by "mind-beta"
        let beta_mem = store
            .store(NewMemory {
                mind_id: "mind-beta".into(),
                title: "Beta's work".into(),
                content: "Beta cites alpha.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        // Should succeed: mind-beta owns the citer memory, citing alpha's memory
        let result = store.cite(&beta_mem, &alpha_mem, "mind-beta").await;
        assert!(result.is_ok());

        // Verify the citation boosted alpha's memory
        let cited = store.get(&alpha_mem).await.unwrap();
        assert_eq!(cited.citation_count, 1);
    }

    #[tokio::test]
    async fn cite_nonexistent_citer_fails() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap();

        let result = store.cite("nonexistent-id", &id1, "cortex-primary").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MemoryError::NotFound(_)));
    }

    #[tokio::test]
    async fn cite_nonexistent_cited_fails() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap();

        let result = store.cite(&id1, "nonexistent-id", "cortex-primary").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MemoryError::NotFound(_)));
    }

    #[tokio::test]
    async fn boot_count_works() {
        let store = test_store().await;

        // Before any sessions, boot count should be 0
        assert_eq!(store.boot_count().await.unwrap(), 0);

        // Start and end a session
        let s1 = store.start_session(Some("first boot")).await.unwrap();
        store.end_session(&s1, "{}").await.unwrap();

        assert_eq!(store.boot_count().await.unwrap(), 1);

        // Start and end another session
        let s2 = store.start_session(None).await.unwrap();
        store.end_session(&s2, "{}").await.unwrap();

        assert_eq!(store.boot_count().await.unwrap(), 2);
    }
}
