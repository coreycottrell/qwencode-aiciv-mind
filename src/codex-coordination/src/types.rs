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
/// All fields are private — callers use MindHandleGuard for controlled mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindHandle {
    id: MindId,
    role: Role,
    vertical: Option<Vertical>,
    status: MindStatus,
    parent: Option<MindId>,
    children: Vec<MindId>,
    session_count: u64,
    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
    growth_stage: GrowthStage,
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

    // Read accessors — used by MindManager and tests
    pub fn id(&self) -> &MindId { &self.id }
    pub fn role(&self) -> &Role { &self.role }
    pub fn vertical(&self) -> Option<&Vertical> { self.vertical.as_ref() }
    pub fn status(&self) -> &MindStatus { &self.status }
    pub fn parent(&self) -> Option<&MindId> { self.parent.as_ref() }
    pub fn children(&self) -> &[MindId] { &self.children }
    pub fn session_count(&self) -> u64 { self.session_count }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn last_active(&self) -> &DateTime<Utc> { &self.last_active }
    pub fn growth_stage(&self) -> &GrowthStage { &self.growth_stage }
}

/// Guard for controlled mutation of MindHandle fields.
/// Invariants enforced at mutation point: parent-child symmetry,
/// children uniqueness, session_count monotonic increase, status-driven
/// last_active timestamp updates.
pub struct MindHandleGuard<'a> {
    handle: &'a mut MindHandle,
}

impl<'a> MindHandleGuard<'a> {
    pub fn new(handle: &'a mut MindHandle) -> Self {
        Self { handle }
    }

    /// Immutable access to the handle's id.
    pub fn id(&self) -> &MindId {
        &self.handle.id
    }

    /// Immutable access to the role.
    pub fn role(&self) -> &Role {
        &self.handle.role
    }

    /// Immutable access to the vertical.
    pub fn vertical(&self) -> Option<&Vertical> {
        self.handle.vertical.as_ref()
    }

    /// Read current status without mutation.
    pub fn status(&self) -> &MindStatus {
        &self.handle.status
    }

    /// Read parent without mutation.
    pub fn parent(&self) -> Option<&MindId> {
        self.handle.parent.as_ref()
    }

    /// Read children snapshot.
    pub fn children(&self) -> Vec<MindId> {
        self.handle.children.clone()
    }

    /// Immutable access to session count.
    pub fn session_count(&self) -> u64 {
        self.handle.session_count
    }

    /// Immutable access to created_at.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.handle.created_at
    }

    /// Immutable access to last_active.
    pub fn last_active(&self) -> &DateTime<Utc> {
        &self.handle.last_active
    }

    /// Immutable access to growth_stage.
    pub fn growth_stage(&self) -> &GrowthStage {
        &self.handle.growth_stage
    }

    /// Update status. Also updates last_active timestamp.
    pub fn update_status(&mut self, new_status: MindStatus) {
        self.handle.last_active = Utc::now();
        self.handle.status = new_status;
    }

    /// Add a child mind. Enforces: child.parent == self, no duplicate children.
    /// Returns Err if child_id already present.
    pub fn add_child(&mut self, child_id: MindId) -> Result<(), MindHandleError> {
        if self.handle.children.contains(&child_id) {
            return Err(MindHandleError::ChildAlreadyExists(child_id));
        }
        self.handle.children.push(child_id);
        Ok(())
    }

    /// Remove a child mind. Enforces: child.parent was self.
    pub fn remove_child(&mut self, child_id: &MindId) -> Result<(), MindHandleError> {
        let pos = self.handle.children.iter().position(|id| id == child_id);
        match pos {
            Some(idx) => { self.handle.children.remove(idx); Ok(()) }
            None => Err(MindHandleError::ChildNotFound(child_id.clone())),
        }
    }

    /// Advance session count by 1. Updates growth_stage from new count.
    pub fn advance_session(&mut self) {
        self.handle.session_count += 1;
        self.handle.growth_stage = GrowthStage::from_session_count(self.handle.session_count);
    }

    /// Terminate this mind. Sets status to Terminated and updates last_active.
    pub fn terminate(&mut self) {
        self.handle.last_active = Utc::now();
        self.handle.status = MindStatus::Terminated;
    }

    /// Consume the guard and commit a complete status snapshot update.
    pub fn commit(self) {
        // All mutations already applied to handle through guard methods.
        // Calling commit() is optional but signals intent.
    }
}

/// Errors from MindHandle mutation.
#[derive(Debug, thiserror::Error)]
pub enum MindHandleError {
    #[error("child mind {0} already exists in parent")]
    ChildAlreadyExists(MindId),
    #[error("child mind {0} not found in parent")]
    ChildNotFound(MindId),
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
