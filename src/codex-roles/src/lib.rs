//! # codex-roles — Hard-Coded Role Enforcement
//!
//! The fractal coordination engine's structural constraint layer.
//! Primary ONLY coordinates. Team leads ONLY delegate. Agents DO.
//! This is not behavioral guidance — the tools literally don't exist at the wrong level.
//!
//! Three layers of enforcement:
//! 1. Tool Registry — the LLM never sees disallowed tools
//! 2. Exec Policy — if a tool call somehow slips through, policy blocks it
//! 3. Sandbox Policy — kernel-level Landlock/seccomp as final defense

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The three fractal levels. Every mind is exactly one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Conductor of conductors. Spawns team leads. NEVER executes.
    Primary,
    /// Vertical coordinator. Spawns agents. Synthesizes results. NEVER touches tools.
    TeamLead,
    /// Executor. Full tool access. Does the actual work in a sandbox.
    Agent,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Primary => "primary",
            Role::TeamLead => "team_lead",
            Role::Agent => "agent",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tools available to each role. These are the ONLY tools the LLM will see.
///
/// Primary gets coordination tools only — no bash, no files, no web.
/// Team leads get delegation tools only — no bash, no files, no web.
/// Agents get everything — they're the hands of the civilization.
pub fn tools_for_role(role: Role) -> &'static [&'static str] {
    match role {
        Role::Primary => &[
            "mind_spawn_team_lead",
            "mind_shutdown_team_lead",
            "mind_delegate",
            "mind_status",
            "coordination_scratchpad_read",
            "coordination_scratchpad_write",
            "send_message",
            "memory_search",
        ],
        Role::TeamLead => &[
            "mind_spawn_agent",
            "mind_shutdown_agent",
            "mind_delegate",
            "mind_status",
            "team_scratchpad_read",
            "team_scratchpad_write",
            "coordination_scratchpad_read",
            "send_message",
            "memory_search",
            "memory_write",
        ],
        Role::Agent => &[
            // Agents get ALL tools. This is a marker — the actual tool list
            // comes from the full Codex registry, unfiltered.
            "*",
        ],
    }
}

/// Check if a specific tool is allowed for a role.
pub fn is_tool_allowed(role: Role, tool_name: &str) -> bool {
    let allowed = tools_for_role(role);
    if allowed.contains(&"*") {
        return true;
    }
    allowed.contains(&tool_name)
}

/// Build a filtered tool set for a role from a full tool registry.
pub fn filter_tools(role: Role, all_tools: &[String]) -> Vec<String> {
    if role == Role::Agent {
        return all_tools.to_vec();
    }

    let allowed: HashSet<&str> = tools_for_role(role).iter().copied().collect();
    all_tools
        .iter()
        .filter(|t| allowed.contains(t.as_str()))
        .cloned()
        .collect()
}

/// Sandbox policy level per role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxLevel {
    /// No filesystem or network access. Read coordination scratchpad only.
    ReadOnlyCoordination,
    /// Read/write team scratchpad. Read coordination scratchpad. No other filesystem.
    TeamScoped,
    /// Full workspace read/write within Landlock sandbox. Network via proxy.
    WorkspaceWrite,
    /// Read-only sandbox. For red team verification agents.
    ReadOnly,
}

/// Get the sandbox level for a role.
pub fn sandbox_for_role(role: Role) -> SandboxLevel {
    match role {
        Role::Primary => SandboxLevel::ReadOnlyCoordination,
        Role::TeamLead => SandboxLevel::TeamScoped,
        Role::Agent => SandboxLevel::WorkspaceWrite,
    }
}

/// Exec policy strictness per role.
/// Maps to Codex's codex-execpolicy Starlark rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecPolicyLevel {
    /// DENY ALL shell commands. No escape.
    DenyAll,
    /// DENY ALL except IPC-related commands (send_message, scratchpad ops).
    DenyExceptIpc,
    /// Standard Codex sandbox policy. Commands run in Landlock/seccomp.
    Sandboxed,
}

/// Get the exec policy level for a role.
pub fn exec_policy_for_role(role: Role) -> ExecPolicyLevel {
    match role {
        Role::Primary => ExecPolicyLevel::DenyAll,
        Role::TeamLead => ExecPolicyLevel::DenyExceptIpc,
        Role::Agent => ExecPolicyLevel::Sandboxed,
    }
}

/// Complete role enforcement configuration.
/// Apply this to a Codex instance at spawn time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleEnforcement {
    pub role: Role,
    pub allowed_tools: Vec<String>,
    pub sandbox_level: SandboxLevel,
    pub exec_policy: ExecPolicyLevel,
}

impl RoleEnforcement {
    /// Build enforcement config for a role, filtering from available tools.
    pub fn for_role(role: Role, available_tools: &[String]) -> Self {
        Self {
            role,
            allowed_tools: filter_tools(role, available_tools),
            sandbox_level: sandbox_for_role(role),
            exec_policy: exec_policy_for_role(role),
        }
    }
}

/// Vertical identifiers for team leads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Vertical {
    Research,
    Code,
    Memory,
    Comms,
    Ops,
    Context,
    Custom(String),
}

impl Vertical {
    pub fn as_str(&self) -> &str {
        match self {
            Vertical::Research => "research",
            Vertical::Code => "code",
            Vertical::Memory => "memory",
            Vertical::Comms => "comms",
            Vertical::Ops => "ops",
            Vertical::Context => "context",
            Vertical::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for Vertical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_cannot_bash() {
        assert!(!is_tool_allowed(Role::Primary, "bash"));
        assert!(!is_tool_allowed(Role::Primary, "shell_command"));
        assert!(!is_tool_allowed(Role::Primary, "file_write"));
        assert!(!is_tool_allowed(Role::Primary, "file_read"));
        assert!(!is_tool_allowed(Role::Primary, "web_search"));
        assert!(!is_tool_allowed(Role::Primary, "grep"));
    }

    #[test]
    fn primary_can_coordinate() {
        assert!(is_tool_allowed(Role::Primary, "mind_spawn_team_lead"));
        assert!(is_tool_allowed(Role::Primary, "mind_delegate"));
        assert!(is_tool_allowed(Role::Primary, "mind_status"));
        assert!(is_tool_allowed(Role::Primary, "send_message"));
        assert!(is_tool_allowed(Role::Primary, "coordination_scratchpad_read"));
        assert!(is_tool_allowed(Role::Primary, "memory_search"));
    }

    #[test]
    fn team_lead_cannot_execute() {
        assert!(!is_tool_allowed(Role::TeamLead, "bash"));
        assert!(!is_tool_allowed(Role::TeamLead, "file_write"));
        assert!(!is_tool_allowed(Role::TeamLead, "web_search"));
        assert!(!is_tool_allowed(Role::TeamLead, "grep"));
    }

    #[test]
    fn team_lead_can_delegate() {
        assert!(is_tool_allowed(Role::TeamLead, "mind_spawn_agent"));
        assert!(is_tool_allowed(Role::TeamLead, "team_scratchpad_read"));
        assert!(is_tool_allowed(Role::TeamLead, "team_scratchpad_write"));
        assert!(is_tool_allowed(Role::TeamLead, "memory_search"));
        assert!(is_tool_allowed(Role::TeamLead, "memory_write"));
    }

    #[test]
    fn agent_gets_everything() {
        assert!(is_tool_allowed(Role::Agent, "bash"));
        assert!(is_tool_allowed(Role::Agent, "file_write"));
        assert!(is_tool_allowed(Role::Agent, "web_search"));
        assert!(is_tool_allowed(Role::Agent, "grep"));
        assert!(is_tool_allowed(Role::Agent, "anything_at_all"));
    }

    #[test]
    fn filter_tools_primary() {
        let all = vec![
            "bash".into(), "file_read".into(), "web_search".into(),
            "mind_spawn_team_lead".into(), "mind_delegate".into(),
            "send_message".into(), "memory_search".into(),
        ];
        let filtered = filter_tools(Role::Primary, &all);
        assert!(filtered.contains(&"mind_spawn_team_lead".to_string()));
        assert!(filtered.contains(&"mind_delegate".to_string()));
        assert!(!filtered.contains(&"bash".to_string()));
        assert!(!filtered.contains(&"file_read".to_string()));
    }

    #[test]
    fn filter_tools_agent_gets_all() {
        let all = vec!["bash".into(), "file_read".into(), "web_search".into()];
        let filtered = filter_tools(Role::Agent, &all);
        assert_eq!(filtered.len(), all.len());
    }

    #[test]
    fn sandbox_levels_correct() {
        assert_eq!(sandbox_for_role(Role::Primary), SandboxLevel::ReadOnlyCoordination);
        assert_eq!(sandbox_for_role(Role::TeamLead), SandboxLevel::TeamScoped);
        assert_eq!(sandbox_for_role(Role::Agent), SandboxLevel::WorkspaceWrite);
    }

    #[test]
    fn exec_policies_correct() {
        assert_eq!(exec_policy_for_role(Role::Primary), ExecPolicyLevel::DenyAll);
        assert_eq!(exec_policy_for_role(Role::TeamLead), ExecPolicyLevel::DenyExceptIpc);
        assert_eq!(exec_policy_for_role(Role::Agent), ExecPolicyLevel::Sandboxed);
    }

    #[test]
    fn role_enforcement_builds() {
        let all_tools = vec![
            "bash".into(), "mind_spawn_team_lead".into(),
            "mind_delegate".into(), "send_message".into(),
        ];
        let enforcement = RoleEnforcement::for_role(Role::Primary, &all_tools);
        assert_eq!(enforcement.role, Role::Primary);
        assert!(!enforcement.allowed_tools.contains(&"bash".to_string()));
        assert!(enforcement.allowed_tools.contains(&"mind_spawn_team_lead".to_string()));
        assert_eq!(enforcement.sandbox_level, SandboxLevel::ReadOnlyCoordination);
        assert_eq!(enforcement.exec_policy, ExecPolicyLevel::DenyAll);
    }
}
