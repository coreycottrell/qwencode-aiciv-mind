//! Core types for the coordination engine.

use chrono::{DateTime, Utc};
use codex_roles::{Role, Vertical};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a mind instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MindId(pub String);

impl MindId {
    pub fn new(role: Role, vertical: Option<&Vertical>) -> Self {
        let base = match (role, vertical) {
            (Role::Primary, _) => "primary".to_string(),
            (Role::TeamLead, Some(v)) => format!("{}-lead", v),
            (Role::TeamLead, None) => format!("lead-{}", &Uuid::new_v4().to_string()[..8]),
            (Role::Agent, Some(v)) => format!("{}-agent-{}", v, &Uuid::new_v4().to_string()[..8]),
            (Role::Agent, None) => format!("agent-{}", &Uuid::new_v4().to_string()[..8]),
        };
        Self(base)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MindId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Current status of a mind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MindStatus {
    /// Starting up, loading context.
    Initializing,
    /// Ready and waiting for work.
    Idle,
    /// Actively processing a task.
    Active { task_id: String },
    /// Waiting for a sub-mind result.
    WaitingForResult { waiting_on: MindId },
    /// Shutting down gracefully.
    ShuttingDown,
    /// Terminated.
    Terminated,
}

/// A handle to an active mind. Used by MindManager to track state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindHandle {
    pub id: MindId,
    pub role: Role,
    pub vertical: Option<Vertical>,
    pub status: MindStatus,
    pub parent: Option<MindId>,
    pub children: Vec<MindId>,
    pub session_count: u64,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    /// Growth stage derived from session_count + fitness scores.
    pub growth_stage: GrowthStage,
}

impl MindHandle {
    pub fn new(role: Role, vertical: Option<Vertical>, parent: Option<MindId>) -> Self {
        let id = MindId::new(role, vertical.as_ref());
        let now = Utc::now();
        Self {
            id,
            role,
            vertical,
            status: MindStatus::Initializing,
            parent,
            children: Vec::new(),
            session_count: 0,
            created_at: now,
            last_active: now,
            growth_stage: GrowthStage::Novice,
        }
    }
}

/// Growth stages — measured, not declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrowthStage {
    /// < 10 sessions
    Novice,
    /// 10-50 sessions, consistent task completion
    Competent,
    /// 50-200 sessions, cross-domain transfer, teaching
    Proficient,
    /// 200-500 sessions, systematic self-improvement
    Advanced,
    /// 500+ sessions, generates novel approaches
    Expert,
}

impl GrowthStage {
    pub fn from_session_count(count: u64) -> Self {
        match count {
            0..10 => GrowthStage::Novice,
            10..50 => GrowthStage::Competent,
            50..200 => GrowthStage::Proficient,
            200..500 => GrowthStage::Advanced,
            _ => GrowthStage::Expert,
        }
    }
}

/// A task that flows through the coordination hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub source: MindId,
    pub target: Option<MindId>,
    pub complexity: Option<TaskComplexityLevel>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexityLevel {
    Trivial,
    Simple,
    Medium,
    Complex,
    Novel,
}

/// Result from a mind completing a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub mind_id: MindId,
    pub summary: String,
    pub evidence: Vec<String>,
    pub learnings: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

/// Routing decision from the InputMux.
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// Route directly to a specific mind.
    Direct(MindId),
    /// Escalate to Primary (requires executive attention).
    Escalate,
    /// Drop (noise, already handled, duplicate).
    Drop { reason: String },
}

/// Input from any external source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalInput {
    pub source: InputSource,
    pub content: String,
    pub priority: InputPriority,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Hub { room: String, thread: Option<String> },
    Telegram { chat_id: String },
    Boop { boop_type: String },
    SubMindResult { mind_id: String },
    Ipc { sender: String },
    Human { name: String },
    Schedule { trigger_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputPriority {
    /// Background noise. Handle if idle.
    Low,
    /// Standard work item.
    Normal,
    /// Should be handled soon.
    High,
    /// Requires immediate attention (Corey, cross-vertical conflict, critical alert).
    Critical,
}

/// Snapshot of the full coordination state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationState {
    pub minds: Vec<MindHandle>,
    pub active_tasks: Vec<Task>,
    pub pending_results: Vec<String>,
    pub snapshot_at: DateTime<Utc>,
}
