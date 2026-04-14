//! MemoryStore — SQLite-backed memory graph with depth scoring and graph links.

use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqlitePoolOptions, SqliteRow};
use sqlx::{Row, SqlitePool};
use tracing::{debug, info};
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
}

/// SQLite-backed memory graph with depth scoring and graph links.
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

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        // Run schema migrations (raw_sql handles multi-statement SQL including triggers)
        sqlx::raw_sql(include_str!("../migrations/001_init.sql"))
            .execute(&pool)
            .await?;
        sqlx::raw_sql(include_str!("../migrations/002_sessions.sql"))
            .execute(&pool)
            .await?;

        info!("Memory store initialized at {db_path}");
        Ok(Self { pool })
    }

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

    /// Retrieve a memory by ID. Also increments access_count and updates last_accessed_at.
    pub async fn get(&self, id: &str) -> Result<Memory, MemoryError> {
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

        row_to_memory(&row)
    }

    /// Search memories using FTS5 full-text search and/or column filters.
    pub async fn search(&self, query: &MemoryQuery) -> Result<Vec<SearchResult>, MemoryError> {
        let limit = query.limit.unwrap_or(20);

        if let Some(text) = &query.text {
            // Escape FTS5 query: wrap each term in double quotes to prevent
            // operator interpretation (e.g. "multi-agent" → "multi" NOT column:agent)
            let escaped = text
                .split_whitespace()
                .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(" ");

            // FTS5 search path
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
                    let memory = row_to_memory(row)?;
                    let rank: f64 = row.try_get("rank").unwrap_or(0.0);
                    // FTS5 rank is negative (more negative = better); normalize to 0..1
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
                    let memory = row_to_memory(row)?;
                    let relevance = memory.depth_score;
                    Ok(SearchResult { memory, relevance })
                })
                .collect()
        }
    }

    /// Create a link between two memories in the graph.
    pub async fn link(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: LinkType,
        strength: f64,
    ) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let lt = link_type.to_string();

        sqlx::query(
            "INSERT OR REPLACE INTO memory_links (id, source_id, target_id, link_type, strength, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&id)
        .bind(source_id)
        .bind(target_id)
        .bind(&lt)
        .bind(strength)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        debug!("Linked {source_id} --{lt}--> {target_id}");
        Ok(id)
    }

    /// Cite a memory: creates a Cites link, increments citation_count, boosts depth_score.
    ///
    /// Depth boost is +0.1 per citation, capped at 1.0. This is how Cortex learns
    /// what matters — cited memories grow deeper, uncited memories fade.
    pub async fn cite(&self, citer_id: &str, cited_id: &str) -> Result<(), MemoryError> {
        self.link(citer_id, cited_id, LinkType::Cites, 1.0).await?;

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

        debug!("Cited {cited_id} from {citer_id}");
        Ok(())
    }

    /// Promote a memory to the next lifecycle tier.
    /// Working → Session → LongTerm. Cannot promote past LongTerm.
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
            MemoryTier::Working => MemoryTier::Session,
            MemoryTier::Session => MemoryTier::LongTerm,
            MemoryTier::LongTerm | MemoryTier::Archived => {
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

    /// Find memories below a depth threshold — candidates for archival by dream mode.
    pub async fn archive_candidates(
        &self,
        max_depth: f64,
        limit: i64,
    ) -> Result<Vec<Memory>, MemoryError> {
        let rows = sqlx::query(
            "SELECT * FROM memories WHERE tier != 'archived' AND depth_score < ?1 \
             ORDER BY depth_score ASC, access_count ASC LIMIT ?2",
        )
        .bind(max_depth)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_memory).collect()
    }

    /// Archive a memory (move to archived tier, set archived_at timestamp).
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

    /// Get all links involving a memory (as source or target).
    pub async fn get_links(&self, memory_id: &str) -> Result<Vec<MemoryLink>, MemoryError> {
        let rows =
            sqlx::query("SELECT * FROM memory_links WHERE source_id = ?1 OR target_id = ?1")
                .bind(memory_id)
                .fetch_all(&self.pool)
                .await?;

        rows.iter().map(row_to_link).collect()
    }

    /// Count total memories in the store.
    pub async fn count(&self) -> Result<i64, MemoryError> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM memories")
            .fetch_one(&self.pool)
            .await?;
        let cnt: i64 = row
            .try_get("cnt")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
        Ok(cnt)
    }

    // ── Session Persistence ──────────────────────────────────────────────────

    /// Start a new session. Returns the session ID.
    pub async fn start_session(&self, notes: Option<&str>) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Get boot count from previous sessions
        let row = sqlx::query("SELECT COALESCE(MAX(boot_count), 0) as bc FROM sessions")
            .fetch_one(&self.pool)
            .await?;
        let prev_boots: i64 = row.try_get("bc").unwrap_or(0);

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

    /// End the current session, persisting coordination state as JSON.
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

    /// Load the latest completed session's coordination state.
    /// Returns (session_id, boot_count, coordination_state_json) or None.
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
                let id: String = r
                    .try_get("id")
                    .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
                let boot_count: i64 = r
                    .try_get("boot_count")
                    .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
                let state: String = r
                    .try_get("coordination_state")
                    .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
                let started: String = r
                    .try_get("started_at")
                    .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
                let ended: Option<String> = r
                    .try_get("ended_at")
                    .map_err(|e| MemoryError::InvalidData(e.to_string()))?;

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

    /// Get the total boot count across all sessions.
    pub async fn boot_count(&self) -> Result<i64, MemoryError> {
        let row = sqlx::query("SELECT COALESCE(MAX(boot_count), 0) as bc FROM sessions")
            .fetch_one(&self.pool)
            .await?;
        let bc: i64 = row.try_get("bc").unwrap_or(0);
        Ok(bc)
    }
}

// ── Row Parsing ──────────────────────────────────────────────────────────────

fn parse_dt(s: &str) -> Result<DateTime<Utc>, MemoryError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| MemoryError::InvalidData(e.to_string()))
}

fn parse_optional_dt(s: Option<String>) -> Option<DateTime<Utc>> {
    s.and_then(|v| DateTime::parse_from_rfc3339(&v).ok().map(|dt| dt.with_timezone(&Utc)))
}

fn row_to_memory(row: &SqliteRow) -> Result<Memory, MemoryError> {
    let evidence_json: Option<String> = row
        .try_get("evidence")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let evidence: Vec<String> = evidence_json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();

    let category_str: String = row
        .try_get("category")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let tier_str: String = row
        .try_get("tier")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;

    let created_str: String = row
        .try_get("created_at")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let updated_str: String = row
        .try_get("updated_at")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let last_accessed_str: Option<String> = row
        .try_get("last_accessed_at")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let archived_str: Option<String> = row
        .try_get("archived_at")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;

    Ok(Memory {
        id: row
            .try_get("id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        mind_id: row
            .try_get("mind_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        role: row
            .try_get("role")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        vertical: row
            .try_get("vertical")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        category: category_str.parse().map_err(MemoryError::InvalidData)?,
        title: row
            .try_get("title")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        content: row
            .try_get("content")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        evidence,
        depth_score: row
            .try_get("depth_score")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        citation_count: row
            .try_get("citation_count")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        access_count: row
            .try_get("access_count")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        tier: tier_str.parse().map_err(MemoryError::InvalidData)?,
        created_at: parse_dt(&created_str)?,
        updated_at: parse_dt(&updated_str)?,
        last_accessed_at: parse_optional_dt(last_accessed_str),
        archived_at: parse_optional_dt(archived_str),
        session_id: row
            .try_get("session_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        task_id: row
            .try_get("task_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
    })
}

fn row_to_link(row: &SqliteRow) -> Result<MemoryLink, MemoryError> {
    let lt_str: String = row
        .try_get("link_type")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;
    let created_str: String = row
        .try_get("created_at")
        .map_err(|e| MemoryError::InvalidData(e.to_string()))?;

    Ok(MemoryLink {
        id: row
            .try_get("id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        source_id: row
            .try_get("source_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        target_id: row
            .try_get("target_id")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        link_type: lt_str.parse().map_err(MemoryError::InvalidData)?,
        strength: row
            .try_get("strength")
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?,
        created_at: parse_dt(&created_str)?,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

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
            content: "Cited memories grow deeper. Uncited memories fade. \
                      This is how Cortex learns what matters."
                .into(),
            evidence: vec!["observation-001".into(), "pattern-003".into()],
            tier: MemoryTier::Working,
            session_id: Some("session-001".into()),
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
        assert_eq!(mem.category, MemoryCategory::Learning);
        assert_eq!(mem.tier, MemoryTier::Working);
        assert_eq!(mem.evidence.len(), 2);
        assert_eq!(mem.access_count, 1); // get() increments access
    }

    #[tokio::test]
    async fn search_fts() {
        let store = test_store().await;
        store.store(test_memory()).await.unwrap();
        store
            .store(NewMemory {
                title: "Unrelated thing".into(),
                content: "This is about something else entirely.".into(),
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
                content: "We chose SQLite for memory because it is embedded.".into(),
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
                content: "I cite the depth scoring memory.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store.cite(&id2, &id1).await.unwrap();
        let mem = store.get(&id1).await.unwrap();

        assert_eq!(mem.citation_count, 1);
        assert!((mem.depth_score - 0.1).abs() < f64::EPSILON);

        // Cite again — depth compounds
        let id3 = store
            .store(NewMemory {
                title: "Second citer".into(),
                content: "Also cites depth scoring.".into(),
                ..test_memory()
            })
            .await
            .unwrap();
        store.cite(&id3, &id1).await.unwrap();
        let mem = store.get(&id1).await.unwrap();

        assert_eq!(mem.citation_count, 2);
        assert!((mem.depth_score - 0.2).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn promote_tiers() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();

        let t1 = store.promote(&id).await.unwrap();
        assert_eq!(t1, MemoryTier::Session);

        let t2 = store.promote(&id).await.unwrap();
        assert_eq!(t2, MemoryTier::LongTerm);

        // Cannot promote past LongTerm
        assert!(store.promote(&id).await.is_err());
    }

    #[tokio::test]
    async fn link_and_get_links() {
        let store = test_store().await;
        let id1 = store.store(test_memory()).await.unwrap();
        let id2 = store
            .store(NewMemory {
                title: "Related memory".into(),
                content: "This builds on the first.".into(),
                ..test_memory()
            })
            .await
            .unwrap();

        store
            .link(&id1, &id2, LinkType::BuildsOn, 0.8)
            .await
            .unwrap();
        let links = store.get_links(&id1).await.unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, LinkType::BuildsOn);
        assert!((links[0].strength - 0.8).abs() < f64::EPSILON);

        // Also visible from target side
        let links2 = store.get_links(&id2).await.unwrap();
        assert_eq!(links2.len(), 1);
    }

    #[tokio::test]
    async fn archive_candidates_and_archive() {
        let store = test_store().await;
        let id = store.store(test_memory()).await.unwrap();

        // depth_score starts at 0.0, so it is a candidate below 0.5
        let candidates = store.archive_candidates(0.5, 10).await.unwrap();
        assert_eq!(candidates.len(), 1);

        store.archive(&id).await.unwrap();
        let mem = store.get(&id).await.unwrap();
        assert_eq!(mem.tier, MemoryTier::Archived);
        assert!(mem.archived_at.is_some());

        // Archived memories no longer show as candidates
        let candidates = store.archive_candidates(0.5, 10).await.unwrap();
        assert!(candidates.is_empty());
    }

    #[tokio::test]
    async fn count() {
        let store = test_store().await;
        assert_eq!(store.count().await.unwrap(), 0);
        store.store(test_memory()).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn depth_caps_at_one() {
        let store = test_store().await;
        let target = store.store(test_memory()).await.unwrap();

        // Cite 15 times — depth should cap at 1.0
        for i in 0..15 {
            let citer = store
                .store(NewMemory {
                    title: format!("Citer {i}"),
                    content: format!("Citation number {i}"),
                    ..test_memory()
                })
                .await
                .unwrap();
            store.cite(&citer, &target).await.unwrap();
        }

        let mem = store.get(&target).await.unwrap();
        assert_eq!(mem.citation_count, 15);
        assert!((mem.depth_score - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn session_start_and_end() {
        let store = test_store().await;

        let sid = store.start_session(Some("test boot")).await.unwrap();
        assert!(!sid.is_empty());

        // Boot count should be 1
        assert_eq!(store.boot_count().await.unwrap(), 1);

        // Store a memory during the session
        store.store(test_memory()).await.unwrap();

        // End session with coordination state
        let state_json = r#"{"minds":[],"active_tasks":[]}"#;
        store.end_session(&sid, state_json).await.unwrap();

        // Load it back
        let saved = store.load_latest_session().await.unwrap().unwrap();
        assert_eq!(saved.id, sid);
        assert_eq!(saved.boot_count, 1);
        assert_eq!(saved.coordination_state_json, state_json);
        assert!(saved.ended_at.is_some());
    }

    #[tokio::test]
    async fn boot_count_increments() {
        let store = test_store().await;

        let s1 = store.start_session(None).await.unwrap();
        store.end_session(&s1, "{}").await.unwrap();

        let s2 = store.start_session(None).await.unwrap();
        store.end_session(&s2, "{}").await.unwrap();

        let s3 = store.start_session(None).await.unwrap();
        store.end_session(&s3, "{}").await.unwrap();

        assert_eq!(store.boot_count().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn load_latest_returns_most_recent() {
        let store = test_store().await;

        let s1 = store.start_session(None).await.unwrap();
        store.end_session(&s1, r#"{"v":1}"#).await.unwrap();

        let s2 = store.start_session(None).await.unwrap();
        store.end_session(&s2, r#"{"v":2}"#).await.unwrap();

        let saved = store.load_latest_session().await.unwrap().unwrap();
        assert_eq!(saved.id, s2);
        assert_eq!(saved.coordination_state_json, r#"{"v":2}"#);
    }

    #[tokio::test]
    async fn no_sessions_returns_none() {
        let store = test_store().await;
        let saved = store.load_latest_session().await.unwrap();
        assert!(saved.is_none());
    }
}
