use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Categories ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Learning,
    Pattern,
    Observation,
    Decision,
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Learning => write!(f, "learning"),
            Self::Pattern => write!(f, "pattern"),
            Self::Observation => write!(f, "observation"),
            Self::Decision => write!(f, "decision"),
        }
    }
}

impl std::str::FromStr for MemoryCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "learning" => Ok(Self::Learning),
            "pattern" => Ok(Self::Pattern),
            "observation" => Ok(Self::Observation),
            "decision" => Ok(Self::Decision),
            _ => Err(format!("Unknown category: {s}")),
        }
    }
}

// ── Lifecycle tiers ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    Working,
    Validated,
    Archived,
}

impl std::fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Working => write!(f, "working"),
            Self::Validated => write!(f, "validated"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for MemoryTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "working" => Ok(Self::Working),
            "validated" => Ok(Self::Validated),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown tier: {s}")),
        }
    }
}

// ── Memory node (full, retrieved from DB) ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: String,
    pub mind_id: String,
    pub role: String,
    pub vertical: Option<String>,
    pub category: MemoryCategory,
    pub title: String,
    pub content: String,
    pub evidence: Vec<String>,
    pub depth_score: f64,
    pub citation_count: i32,
    pub access_count: i32,
    pub tier: MemoryTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
}

// ── Graph edge ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub link_type: LinkType,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

// ── Link types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Cites,      // memory A references memory B
    BuildsOn,   // memory A was derived from memory B
    Supersedes, // memory A replaces memory B (B is outdated)
    Conflicts,  // memory A contradicts memory B (needs resolution)
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cites => write!(f, "cites"),
            Self::BuildsOn => write!(f, "builds_on"),
            Self::Supersedes => write!(f, "supersedes"),
            Self::Conflicts => write!(f, "conflicts"),
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cites" => Ok(Self::Cites),
            "builds_on" => Ok(Self::BuildsOn),
            "supersedes" => Ok(Self::Supersedes),
            "conflicts" => Ok(Self::Conflicts),
            _ => Err(format!("Unknown link type: {s}")),
        }
    }
}

// ── Input types ─────────────────────────────────────────────────────────────

/// Input for creating a new memory.
#[derive(Debug, Clone)]
pub struct NewMemory {
    pub mind_id: String,
    pub role: String,
    pub vertical: Option<String>,
    pub category: MemoryCategory,
    pub title: String,
    pub content: String,
    pub evidence: Vec<String>,
    pub tier: MemoryTier,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
}

/// Search query parameters.
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub text: Option<String>,
    pub mind_id: Option<String>,
    pub category: Option<MemoryCategory>,
    pub tier: Option<MemoryTier>,
    pub min_depth: Option<f64>,
    pub limit: Option<i64>,
}

/// Graph traversal query — find memories connected via edges.
#[derive(Debug, Clone)]
pub struct GraphQuery {
    pub memory_id: String,
    pub link_types: Option<Vec<LinkType>>,
    pub direction: TraversalDirection,
    pub max_depth: u32,
    pub limit: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TraversalDirection {
    #[default]
    Outgoing, // source → target
    Incoming, // target → source
    Both,
}

// ── Result types ────────────────────────────────────────────────────────────

/// A search result with relevance scoring.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory: MemoryNode,
    pub relevance: f64,
}

/// A traversed path through the memory graph.
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub path: Vec<MemoryNode>,
    pub edges: Vec<GraphEdge>,
    pub total_depth: u32,
}

/// A saved session restored from the database.
#[derive(Debug, Clone)]
pub struct SavedSession {
    pub id: String,
    pub boot_count: i64,
    pub coordination_state_json: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
