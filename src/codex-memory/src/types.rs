//! Memory types — the shape of what Cortex remembers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single memory entry in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub mind_id: String,
    pub role: String,
    pub vertical: Option<String>,

    pub category: MemoryCategory,
    pub title: String,
    pub content: String,
    pub evidence: Vec<String>,

    pub depth_score: f64,
    pub citation_count: i64,
    pub access_count: i64,

    pub tier: MemoryTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,

    pub session_id: Option<String>,
    pub task_id: Option<String>,
}

/// What kind of thing is remembered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    /// A discovered pattern worth preserving.
    Learning,
    /// A recurring pattern (3+ occurrences).
    Pattern,
    /// A decision and its rationale.
    Decision,
    /// A factual observation.
    Observation,
    /// An error and how it was fixed.
    Error,
    /// Contextual state (session notes, scratchpad content).
    Context,
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Learning => write!(f, "learning"),
            Self::Pattern => write!(f, "pattern"),
            Self::Decision => write!(f, "decision"),
            Self::Observation => write!(f, "observation"),
            Self::Error => write!(f, "error"),
            Self::Context => write!(f, "context"),
        }
    }
}

impl std::str::FromStr for MemoryCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "learning" => Ok(Self::Learning),
            "pattern" => Ok(Self::Pattern),
            "decision" => Ok(Self::Decision),
            "observation" => Ok(Self::Observation),
            "error" => Ok(Self::Error),
            "context" => Ok(Self::Context),
            _ => Err(format!("Unknown category: {s}")),
        }
    }
}

/// Memory lifecycle tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    /// Current task context. Ephemeral.
    Working,
    /// Persists across turns within a session.
    Session,
    /// Persists across sessions. The real memory.
    LongTerm,
    /// Archived by dream mode. Still searchable but deprioritized.
    Archived,
}

impl std::fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Working => write!(f, "working"),
            Self::Session => write!(f, "session"),
            Self::LongTerm => write!(f, "long_term"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for MemoryTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "working" => Ok(Self::Working),
            "session" => Ok(Self::Session),
            "long_term" => Ok(Self::LongTerm),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown tier: {s}")),
        }
    }
}

/// A link between two memories in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub link_type: LinkType,
    pub strength: f64,
    pub created_at: DateTime<Utc>,
}

/// How two memories are related.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Source cites target as evidence.
    Cites,
    /// Source builds on/extends target.
    BuildsOn,
    /// Source contradicts target.
    Contradicts,
    /// Source replaces target (newer version).
    Supersedes,
    /// General relationship.
    Related,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cites => write!(f, "cites"),
            Self::BuildsOn => write!(f, "builds_on"),
            Self::Contradicts => write!(f, "contradicts"),
            Self::Supersedes => write!(f, "supersedes"),
            Self::Related => write!(f, "related"),
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cites" => Ok(Self::Cites),
            "builds_on" => Ok(Self::BuildsOn),
            "contradicts" => Ok(Self::Contradicts),
            "supersedes" => Ok(Self::Supersedes),
            "related" => Ok(Self::Related),
            _ => Err(format!("Unknown link type: {s}")),
        }
    }
}

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

/// Search results with relevance scoring.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory: Memory,
    pub relevance: f64,
}

/// Query parameters for memory search.
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub text: Option<String>,
    pub mind_id: Option<String>,
    pub category: Option<MemoryCategory>,
    pub tier: Option<MemoryTier>,
    pub min_depth: Option<f64>,
    pub limit: Option<i64>,
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
