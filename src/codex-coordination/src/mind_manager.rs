//! # MindManager — The Registry and Factory for All Active Minds
//!
//! Manages the lifecycle of minds in the fractal hierarchy:
//! - Spawn team leads (persistent ThreadManager instances)
//! - Spawn agents (ephemeral codex exec instances)
//! - Track status, delegate tasks, route results
//! - Enforce role constraints at spawn time

use std::collections::HashMap;

use chrono::Utc;
use codex_roles::{Role, RoleEnforcement, Vertical};
use tracing::{info, warn};

use crate::types::*;

/// Manages all active minds in the coordination hierarchy.
pub struct MindManager {
    /// All active minds indexed by ID.
    minds: HashMap<MindId, MindHandle>,
    /// Active tasks being processed.
    active_tasks: HashMap<String, Task>,
    /// The primary mind's ID.
    primary_id: Option<MindId>,
    /// Base directory for AGENTS.md templates.
    agents_dir: String,
    /// Base directory for scratchpads.
    scratchpads_dir: String,
}

impl MindManager {
    pub fn new(agents_dir: String, scratchpads_dir: String) -> Self {
        Self {
            minds: HashMap::new(),
            active_tasks: HashMap::new(),
            primary_id: None,
            agents_dir,
            scratchpads_dir,
        }
    }

    /// Initialize the Primary mind. Called once at startup.
    pub fn init_primary(&mut self) -> MindId {
        let handle = MindHandle::new(Role::Primary, None, None);
        let id = handle.id.clone();
        self.primary_id = Some(id.clone());
        self.minds.insert(id.clone(), handle);
        info!(mind_id = %id, "Primary mind initialized");
        id
    }

    /// Spawn a Team Lead mind for a specific vertical.
    ///
    /// In Codex terms: creates a new ThreadManager with a persistent thread,
    /// role-filtered tool registry, and vertical-specific AGENTS.md.
    pub fn spawn_team_lead(
        &mut self,
        vertical: Vertical,
        objective: &str,
    ) -> Result<MindId, CoordinationError> {
        // Validate: only Primary can spawn team leads
        let primary_id = self.primary_id.as_ref()
            .ok_or(CoordinationError::NoPrimary)?;

        // Check if this vertical already has an active lead
        if self.find_team_lead(&vertical).is_some() {
            return Err(CoordinationError::DuplicateVertical(vertical.to_string()));
        }

        let mut handle = MindHandle::new(
            Role::TeamLead,
            Some(vertical.clone()),
            Some(primary_id.clone()),
        );
        handle.status = MindStatus::Idle;

        let id = handle.id.clone();

        // Register as child of Primary
        if let Some(primary) = self.minds.get_mut(primary_id) {
            primary.children.push(id.clone());
        }

        // Build role enforcement
        let _enforcement = RoleEnforcement::for_role(
            Role::TeamLead,
            &self.all_tool_names(),
        );

        // Build the AGENTS.md path for this vertical
        let _agents_md = format!("{}/team-lead/{}.agents.md", self.agents_dir, vertical);

        // Build scratchpad paths
        let _team_scratchpad = format!("{}/teams/{}.md", self.scratchpads_dir, vertical);
        let _coordination_scratchpad = format!("{}/coordination.md", self.scratchpads_dir);

        info!(
            mind_id = %id,
            vertical = %vertical,
            objective = objective,
            "Team lead spawned"
        );

        self.minds.insert(id.clone(), handle);
        Ok(id)
    }

    /// Spawn an Agent mind under a specific team lead.
    ///
    /// In Codex terms: launches `codex exec` with sandbox, ephemeral mode,
    /// and agent-specific AGENTS.md.
    pub fn spawn_agent(
        &mut self,
        parent_id: &MindId,
        agent_type: &str,
        task: &str,
    ) -> Result<MindId, CoordinationError> {
        // Validate: parent must be a team lead
        let parent = self.minds.get(parent_id)
            .ok_or_else(|| CoordinationError::MindNotFound(parent_id.clone()))?;

        if parent.role != Role::TeamLead {
            return Err(CoordinationError::InvalidSpawnRole {
                spawner: parent.role,
                attempted: Role::Agent,
            });
        }

        let vertical = parent.vertical.clone();
        let mut handle = MindHandle::new(
            Role::Agent,
            vertical,
            Some(parent_id.clone()),
        );
        handle.status = MindStatus::Active {
            task_id: uuid::Uuid::new_v4().to_string(),
        };

        let id = handle.id.clone();

        // Register as child of team lead
        if let Some(parent) = self.minds.get_mut(parent_id) {
            parent.children.push(id.clone());
        }

        // Build the AGENTS.md path
        let _agents_md = format!("{}/agent/{}.agents.md", self.agents_dir, agent_type);

        info!(
            mind_id = %id,
            parent = %parent_id,
            agent_type = agent_type,
            task = task,
            "Agent spawned"
        );

        self.minds.insert(id.clone(), handle);
        Ok(id)
    }

    /// Delegate a task from one mind to another.
    pub fn delegate(
        &mut self,
        from: &MindId,
        to: &MindId,
        task_description: &str,
    ) -> Result<Task, CoordinationError> {
        // Validate both minds exist
        if !self.minds.contains_key(from) {
            return Err(CoordinationError::MindNotFound(from.clone()));
        }
        if !self.minds.contains_key(to) {
            return Err(CoordinationError::MindNotFound(to.clone()));
        }

        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            description: task_description.to_string(),
            source: from.clone(),
            target: Some(to.clone()),
            complexity: None,
            created_at: Utc::now(),
        };

        // Update target status
        if let Some(target) = self.minds.get_mut(to) {
            target.status = MindStatus::Active {
                task_id: task.id.clone(),
            };
            target.last_active = Utc::now();
        }

        self.active_tasks.insert(task.id.clone(), task.clone());
        info!(
            task_id = task.id,
            from = %from,
            to = %to,
            "Task delegated"
        );

        Ok(task)
    }

    /// Record a task result and update mind status.
    pub fn complete_task(&mut self, result: TaskResult) -> Result<(), CoordinationError> {
        let task = self.active_tasks.remove(&result.task_id)
            .ok_or_else(|| CoordinationError::TaskNotFound(result.task_id.clone()))?;

        // Update the completing mind's status
        if let Some(mind) = self.minds.get_mut(&result.mind_id) {
            mind.status = MindStatus::Idle;
            mind.last_active = Utc::now();
        }

        // Notify the source mind
        if let Some(source) = self.minds.get_mut(&task.source) {
            if let MindStatus::WaitingForResult { .. } = &source.status {
                source.status = MindStatus::Idle;
            }
        }

        info!(
            task_id = result.task_id,
            mind_id = %result.mind_id,
            "Task completed"
        );

        Ok(())
    }

    /// Shutdown a specific mind gracefully.
    pub fn shutdown_mind(&mut self, mind_id: &MindId) -> Result<(), CoordinationError> {
        // First pass: read-only — collect children and parent info
        let (children_ids, parent_id) = {
            let mind = self.minds.get(mind_id)
                .ok_or_else(|| CoordinationError::MindNotFound(mind_id.clone()))?;
            (mind.children.clone(), mind.parent.clone())
        };

        // Check for active children (read-only access)
        let active_children: Vec<_> = children_ids.iter()
            .filter(|c| {
                self.minds.get(*c)
                    .is_some_and(|m| m.status != MindStatus::Terminated)
            })
            .cloned()
            .collect();

        if !active_children.is_empty() {
            warn!(
                mind_id = %mind_id,
                active_children = ?active_children,
                "Cannot shutdown mind with active children"
            );
            return Err(CoordinationError::ActiveChildren {
                mind_id: mind_id.clone(),
                children: active_children,
            });
        }

        // Second pass: mutate — terminate the mind
        if let Some(mind) = self.minds.get_mut(mind_id) {
            mind.status = MindStatus::Terminated;
        }

        // Third pass: mutate — remove from parent's children list
        if let Some(parent_id) = &parent_id {
            if let Some(parent) = self.minds.get_mut(parent_id) {
                parent.children.retain(|c| c != mind_id);
            }
        }

        info!(mind_id = %mind_id, "Mind shutdown");
        Ok(())
    }

    /// Get a snapshot of the full coordination state.
    pub fn coordination_state(&self) -> CoordinationState {
        CoordinationState {
            minds: self.minds.values().cloned().collect(),
            active_tasks: self.active_tasks.values().cloned().collect(),
            pending_results: Vec::new(),
            snapshot_at: Utc::now(),
        }
    }

    /// Find the team lead for a given vertical.
    pub fn find_team_lead(&self, vertical: &Vertical) -> Option<&MindHandle> {
        self.minds.values().find(|m| {
            m.role == Role::TeamLead
                && m.vertical.as_ref() == Some(vertical)
                && m.status != MindStatus::Terminated
        })
    }

    /// Get a mind handle by ID.
    pub fn get_mind(&self, id: &MindId) -> Option<&MindHandle> {
        self.minds.get(id)
    }

    /// Count active minds by role.
    pub fn count_by_role(&self, role: Role) -> usize {
        self.minds.values()
            .filter(|m| m.role == role && m.status != MindStatus::Terminated)
            .count()
    }

    /// List all tool names (for filtering). In production, this comes from Codex's ToolRegistry.
    fn all_tool_names(&self) -> Vec<String> {
        vec![
            // Coordination tools
            "mind_spawn_team_lead".into(),
            "mind_shutdown_team_lead".into(),
            "mind_spawn_agent".into(),
            "mind_shutdown_agent".into(),
            "mind_delegate".into(),
            "mind_status".into(),
            "send_message".into(),
            "memory_search".into(),
            "memory_write".into(),
            "coordination_scratchpad_read".into(),
            "coordination_scratchpad_write".into(),
            "team_scratchpad_read".into(),
            "team_scratchpad_write".into(),
            // Standard Codex tools (agents get these)
            "bash".into(),
            "file_read".into(),
            "file_write".into(),
            "grep".into(),
            "glob".into(),
            "web_search".into(),
            "web_fetch".into(),
            "git".into(),
        ]
    }
}

/// Errors from the coordination engine.
#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    #[error("No primary mind initialized")]
    NoPrimary,

    #[error("Mind not found: {0}")]
    MindNotFound(MindId),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Vertical already has an active team lead: {0}")]
    DuplicateVertical(String),

    #[error("Role {spawner} cannot spawn {attempted}")]
    InvalidSpawnRole { spawner: Role, attempted: Role },

    #[error("Cannot shutdown mind {mind_id} with active children: {children:?}")]
    ActiveChildren { mind_id: MindId, children: Vec<MindId> },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> MindManager {
        MindManager::new("agents".into(), "scratchpads".into())
    }

    #[test]
    fn init_primary() {
        let mut mm = make_manager();
        let id = mm.init_primary();
        assert_eq!(id.as_str(), "primary");
        assert_eq!(mm.count_by_role(Role::Primary), 1);
    }

    #[test]
    fn spawn_team_lead() {
        let mut mm = make_manager();
        mm.init_primary();
        let id = mm.spawn_team_lead(Vertical::Research, "Research all the things").unwrap();
        assert_eq!(id.as_str(), "research-lead");
        assert_eq!(mm.count_by_role(Role::TeamLead), 1);

        // Primary should have the team lead as a child
        let primary = mm.get_mind(&MindId("primary".into())).unwrap();
        assert!(primary.children.contains(&id));
    }

    #[test]
    fn cannot_spawn_duplicate_vertical() {
        let mut mm = make_manager();
        mm.init_primary();
        mm.spawn_team_lead(Vertical::Research, "first").unwrap();
        let err = mm.spawn_team_lead(Vertical::Research, "second");
        assert!(err.is_err());
    }

    #[test]
    fn spawn_agent_under_team_lead() {
        let mut mm = make_manager();
        mm.init_primary();
        let lead_id = mm.spawn_team_lead(Vertical::Code, "Write code").unwrap();
        let agent_id = mm.spawn_agent(&lead_id, "coder", "Implement feature X").unwrap();

        assert_eq!(mm.count_by_role(Role::Agent), 1);
        let lead = mm.get_mind(&lead_id).unwrap();
        assert!(lead.children.contains(&agent_id));
    }

    #[test]
    fn primary_cannot_spawn_agents() {
        let mut mm = make_manager();
        let primary_id = mm.init_primary();
        let err = mm.spawn_agent(&primary_id, "coder", "task");
        assert!(err.is_err());
    }

    #[test]
    fn delegate_task() {
        let mut mm = make_manager();
        let primary_id = mm.init_primary();
        let lead_id = mm.spawn_team_lead(Vertical::Research, "Research").unwrap();
        let task = mm.delegate(&primary_id, &lead_id, "Research topic X").unwrap();

        assert!(!task.id.is_empty());
        let lead = mm.get_mind(&lead_id).unwrap();
        assert!(matches!(lead.status, MindStatus::Active { .. }));
    }

    #[test]
    fn complete_task_flow() {
        let mut mm = make_manager();
        let primary_id = mm.init_primary();
        let lead_id = mm.spawn_team_lead(Vertical::Research, "Research").unwrap();
        let task = mm.delegate(&primary_id, &lead_id, "Research X").unwrap();

        let result = TaskResult {
            task_id: task.id.clone(),
            mind_id: lead_id.clone(),
            summary: "Found X".into(),
            evidence: vec!["Source A".into()],
            learnings: vec!["Pattern P".into()],
            completed_at: Utc::now(),
        };

        mm.complete_task(result).unwrap();
        let lead = mm.get_mind(&lead_id).unwrap();
        assert_eq!(lead.status, MindStatus::Idle);
    }

    #[test]
    fn cannot_shutdown_with_active_children() {
        let mut mm = make_manager();
        mm.init_primary();
        let lead_id = mm.spawn_team_lead(Vertical::Code, "Code").unwrap();
        let _agent_id = mm.spawn_agent(&lead_id, "coder", "task").unwrap();

        let err = mm.shutdown_mind(&lead_id);
        assert!(err.is_err());
    }

    #[test]
    fn shutdown_after_children_terminated() {
        let mut mm = make_manager();
        mm.init_primary();
        let lead_id = mm.spawn_team_lead(Vertical::Code, "Code").unwrap();
        let agent_id = mm.spawn_agent(&lead_id, "coder", "task").unwrap();

        mm.shutdown_mind(&agent_id).unwrap();
        mm.shutdown_mind(&lead_id).unwrap();

        assert_eq!(mm.count_by_role(Role::TeamLead), 0);
        assert_eq!(mm.count_by_role(Role::Agent), 0);
    }

    #[test]
    fn coordination_state_snapshot() {
        let mut mm = make_manager();
        mm.init_primary();
        mm.spawn_team_lead(Vertical::Research, "Research").unwrap();
        mm.spawn_team_lead(Vertical::Code, "Code").unwrap();

        let state = mm.coordination_state();
        assert_eq!(state.minds.len(), 3); // Primary + 2 leads
    }
}
