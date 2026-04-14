//! Identity: manifest, role, growth stage, principles, anti-patterns.
//! Principle 8 — Identity Persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Role ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Primary,
    TeamLead,
    Agent,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::TeamLead => write!(f, "team_lead"),
            Self::Agent => write!(f, "agent"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "primary" => Ok(Self::Primary),
            "team_lead" => Ok(Self::TeamLead),
            "agent" => Ok(Self::Agent),
            _ => Err(format!("Unknown role: {s}")),
        }
    }
}

// ── Growth Stage ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrowthStage {
    Novice,     // < 10 sessions
    Competent,  // 10-50 sessions
    Proficient, // 50-200 sessions
    Advanced,   // 200-500 sessions
    Expert,     // 500+ sessions
}

impl GrowthStage {
    pub fn from_sessions(count: i64) -> Self {
        if count >= 500 { Self::Expert }
        else if count >= 200 { Self::Advanced }
        else if count >= 50 { Self::Proficient }
        else if count >= 10 { Self::Competent }
        else { Self::Novice }
    }
}

impl std::fmt::Display for GrowthStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Novice => write!(f, "novice"),
            Self::Competent => write!(f, "competent"),
            Self::Proficient => write!(f, "proficient"),
            Self::Advanced => write!(f, "advanced"),
            Self::Expert => write!(f, "expert"),
        }
    }
}

// ── Category & Tier (shared with cortex-memory, mirrored for convenience) ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Learning,
    Pattern,
    Observation,
    Decision,
    Error,
    Context,
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Learning => write!(f, "learning"),
            Self::Pattern => write!(f, "pattern"),
            Self::Observation => write!(f, "observation"),
            Self::Decision => write!(f, "decision"),
            Self::Error => write!(f, "error"),
            Self::Context => write!(f, "context"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    Working,    // Active, unproven
    Validated,  // Cited/used, proven useful
    Archived,   // Deprecated, still searchable
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

// ── Manifest ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub identity: String,
    pub role: Role,
    pub vertical: String,
    pub specialty: Option<String>,
    pub principles: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub preferences: serde_json::Value,
    pub growth_stage: GrowthStage,
    pub session_count: i64,
    pub parent_mind: Option<String>,
    pub children: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Manifest {
    pub fn new(identity: &str, role: Role, vertical: &str) -> Self {
        let now = Utc::now();
        Self {
            identity: identity.to_string(),
            role,
            vertical: vertical.to_string(),
            specialty: None,
            principles: Vec::new(),
            anti_patterns: Vec::new(),
            preferences: serde_json::json!({}),
            growth_stage: GrowthStage::Novice,
            session_count: 0,
            parent_mind: None,
            children: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&text)?;
        Ok(manifest)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn increment_session(&mut self) {
        self.session_count += 1;
        self.growth_stage = GrowthStage::from_sessions(self.session_count);
        self.updated_at = Utc::now();
    }

    pub fn add_anti_pattern(&mut self, pattern: &str) {
        if !self.anti_patterns.contains(&pattern.to_string()) {
            self.anti_patterns.push(pattern.to_string());
            self.updated_at = Utc::now();
        }
    }
}
