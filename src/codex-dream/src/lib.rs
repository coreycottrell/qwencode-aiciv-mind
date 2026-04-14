//! # codex-dream — Dream Mode (Principle 4)
//!
//! Five-phase nightly cycle:
//! 1. Audit — find low-depth memories (archive candidates)
//! 2. Consolidate — find similar memories, create graph links
//! 3. Prune — archive isolated low-depth memories
//! 4. Synthesize — create pattern memories from clusters of 3+ linked memories
//! 5. Report — return what happened
//!
//! The dream cycle is how Cortex develops judgment about what matters.
//! Memories that compound get preserved. Memories that don't get archived.

pub mod engine;

pub use engine::{DreamEngine, DreamError, DreamReport};

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

/// Configuration for Dream Mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamConfig {
    /// Start time for nightly dream cycle (default: 01:00).
    pub start_time: NaiveTime,
    /// End time (default: 04:00).
    pub end_time: NaiveTime,
    /// Minimum depth score for keeping a memory (below = archive candidate).
    pub archive_threshold: f64,
    /// Model to use for dream mode (default: gemma4).
    pub model: String,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            start_time: NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
            archive_threshold: 0.1,
            model: "gemma4".into(),
        }
    }
}

/// A dream cycle that the system runs overnight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamCycle {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub phases: Vec<DreamPhase>,
}

/// Individual phase within a dream cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamPhase {
    pub phase: PhaseType,
    pub status: PhaseStatus,
    pub findings: Vec<DreamFinding>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseType {
    Review,
    PatternSearch,
    DeliberateForgetting,
    SelfImprovement,
    DreamArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
}

/// A finding from any dream phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamFinding {
    pub finding_type: FindingType,
    pub description: String,
    pub action: Option<String>,
    pub priority: FindingPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    /// A recurring pattern worth codifying.
    Pattern { occurrences: usize },
    /// A memory that should be archived.
    ArchiveCandidate { depth_score: f64 },
    /// Two memories that contradict each other.
    Contradiction { memory_a: String, memory_b: String },
    /// A manifest that should be updated.
    ManifestEvolution { vertical: String, change: String },
    /// A routing pattern that should change.
    RoutingUpdate { from: String, to: String },
    /// A skill that should be created or evolved.
    SkillProposal { name: String },
    /// Cross-domain transfer opportunity.
    TransferOpportunity { from_domain: String, to_domain: String, pattern: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Build a new dream cycle with all 5 phases pending.
pub fn new_dream_cycle() -> DreamCycle {
    let phases = vec![
        PhaseType::Review,
        PhaseType::PatternSearch,
        PhaseType::DeliberateForgetting,
        PhaseType::SelfImprovement,
        PhaseType::DreamArtifacts,
    ];

    DreamCycle {
        id: uuid::Uuid::new_v4().to_string(),
        started_at: Utc::now(),
        completed_at: None,
        phases: phases.into_iter().map(|p| DreamPhase {
            phase: p,
            status: PhaseStatus::Pending,
            findings: Vec::new(),
            started_at: None,
            completed_at: None,
        }).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dream_cycle_has_five_phases() {
        let cycle = new_dream_cycle();
        assert_eq!(cycle.phases.len(), 5);
        assert_eq!(cycle.phases[0].phase, PhaseType::Review);
        assert_eq!(cycle.phases[4].phase, PhaseType::DreamArtifacts);
    }

    #[test]
    fn all_phases_start_pending() {
        let cycle = new_dream_cycle();
        for phase in &cycle.phases {
            assert_eq!(phase.status, PhaseStatus::Pending);
        }
    }

    #[test]
    fn default_config() {
        let config = DreamConfig::default();
        assert_eq!(config.model, "gemma4");
        assert!(config.archive_threshold > 0.0);
    }
}
