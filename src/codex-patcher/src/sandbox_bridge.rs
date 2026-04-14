//! # Sandbox Bridge — Cortex Roles → Codex Sandboxing
//!
//! Maps Cortex's role-based sandbox levels to Codex's production-grade
//! sandboxing (bubblewrap + seccomp on Linux, Seatbelt on macOS).
//!
//! ## What it does
//!
//! Cortex defines three sandbox levels in codex-roles:
//! - `ReadOnlyCoordination` (Primary) — no mutations allowed
//! - `TeamScoped` (TeamLead) — only scratchpad/memory writes
//! - `WorkspaceWrite` (Agent) — full workspace access with path containment
//!
//! Codex has `SandboxPolicy` with variants:
//! - `DangerFullAccess` — no sandbox
//! - `ReadOnly` — read-only filesystem, optional network
//! - `WorkspaceWrite` — writable roots + read-only carveouts
//! - `ExternalSandbox` — defer to external sandbox
//!
//! This module maps between the two systems and generates the patch that
//! wires Cortex role enforcement through Codex's sandbox manager.

use serde::{Deserialize, Serialize};

/// Cortex sandbox level mapped to Codex sandbox policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CortexSandboxLevel {
    /// Primary: coordination only, no file mutations.
    /// Maps to Codex `SandboxPolicy::ReadOnly { network_access: false }`.
    ReadOnlyCoordination,

    /// TeamLead: can write scratchpads and memories but not workspace files.
    /// Maps to Codex `SandboxPolicy::WorkspaceWrite { writable_roots: [scratchpad, memory] }`.
    TeamScoped,

    /// Agent: full workspace access with path containment.
    /// Maps to Codex `SandboxPolicy::WorkspaceWrite { writable_roots: [workspace_root] }`.
    WorkspaceWrite,

    /// Red team: read-only, no network.
    /// Maps to Codex `SandboxPolicy::ReadOnly { network_access: false }`.
    ReadOnly,
}

impl CortexSandboxLevel {
    /// Generate the Codex sandbox policy JSON for this level.
    /// This is what gets passed to `SandboxManager::transform()`.
    pub fn to_codex_policy_json(&self, workspace_root: &str) -> String {
        match self {
            CortexSandboxLevel::ReadOnlyCoordination => {
                serde_json::json!({
                    "type": "ReadOnly",
                    "network_access": false
                })
                .to_string()
            }
            CortexSandboxLevel::TeamScoped => {
                serde_json::json!({
                    "type": "WorkspaceWrite",
                    "writable_roots": [
                        format!("{workspace_root}/.claude"),
                        format!("{workspace_root}/data/memory"),
                    ],
                    "network_access": true,
                    "exclude_tmpdir_env_var": false,
                    "exclude_slash_tmp": false,
                })
                .to_string()
            }
            CortexSandboxLevel::WorkspaceWrite => {
                serde_json::json!({
                    "type": "WorkspaceWrite",
                    "writable_roots": [workspace_root],
                    "network_access": true,
                    "exclude_tmpdir_env_var": false,
                    "exclude_slash_tmp": false,
                })
                .to_string()
            }
            CortexSandboxLevel::ReadOnly => {
                serde_json::json!({
                    "type": "ReadOnly",
                    "network_access": false
                })
                .to_string()
            }
        }
    }

    /// Whether this level allows file writes.
    pub fn allows_writes(&self) -> bool {
        match self {
            CortexSandboxLevel::ReadOnlyCoordination => false,
            CortexSandboxLevel::TeamScoped => true, // scratchpad/memory writes
            CortexSandboxLevel::WorkspaceWrite => true,
            CortexSandboxLevel::ReadOnly => false,
        }
    }

    /// Whether this level allows network access.
    pub fn allows_network(&self) -> bool {
        match self {
            CortexSandboxLevel::ReadOnlyCoordination => false,
            CortexSandboxLevel::TeamScoped => true,
            CortexSandboxLevel::WorkspaceWrite => true,
            CortexSandboxLevel::ReadOnly => false,
        }
    }
}

/// The actual patch content — a unified diff that replaces Cortex's
/// string-matching sandbox with Codex's production sandboxing.
pub fn generate_sandbox_bridge_patch() -> &'static str {
    r#"--- a/codex-rs/core/src/exec.rs
+++ b/codex-rs/core/src/exec.rs
@@ -1,3 +1,5 @@
+use codex_patcher::sandbox_bridge::CortexSandboxLevel;
+
 use crate::codex_thread::CodexThread;
 use crate::config::Config;
 use crate::exec::ExecRequest;
@@ -120,6 +122,30 @@ pub(crate) async fn process_exec_tool_call(
     let sandbox_policy = select_process_exec_tool_sandbox_type(session);

+    // === CORTEX SANDBOX BRIDGE ===
+    // Override Codex's sandbox policy with Cortex's role-based sandbox level.
+    // This replaces Cortex's string-matching SandboxEnforcer with Codex's
+    // production-grade bubblewrap + seccomp (Linux) or Seatbelt (macOS).
+    if let Some(cortex_role) = &session.cortex_role_override {
+        let cortex_level = match cortex_role.as_str() {
+            "primary" => CortexSandboxLevel::ReadOnlyCoordination,
+            "team_lead" => CortexSandboxLevel::TeamScoped,
+            "agent" => CortexSandboxLevel::WorkspaceWrite,
+            "red_team" => CortexSandboxLevel::ReadOnly,
+            _ => CortexSandboxLevel::WorkspaceWrite,
+        };
+
+        // Generate Codex-compatible sandbox policy
+        let workspace_root = session
+            .cwd()
+            .to_string_lossy()
+            .to_string();
+        let cortex_policy = cortex_level.to_codex_policy_json(&workspace_root);
+
+        tracing::info!(
+            target: "cortex::sandbox",
+            "Cortex sandbox override: role={} → level={:?} policy={}",
+            cortex_role, cortex_level, cortex_policy
+        );
+
+        // Parse the policy JSON and use it instead of the default
+        if let Ok(policy) = serde_json::from_str(&cortex_policy) {
+            sandbox_policy = policy;
+        }
+    }
+    // === END CORTEX SANDBOX BRIDGE ===
+
     let exec_request = build_exec_request(params, &sandbox_policy)?;

     // Execute the command through Codex's sandboxed execution pipeline.
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_only_coord_no_writes() {
        let level = CortexSandboxLevel::ReadOnlyCoordination;
        assert!(!level.allows_writes());
        assert!(!level.allows_network());

        let json = level.to_codex_policy_json("/workspace");
        assert!(json.contains("ReadOnly"));
        assert!(json.contains("false")); // network
    }

    #[test]
    fn test_team_scoped_limited_writes() {
        let level = CortexSandboxLevel::TeamScoped;
        assert!(level.allows_writes());
        assert!(level.allows_network());

        let json = level.to_codex_policy_json("/workspace");
        assert!(json.contains("WorkspaceWrite"));
        assert!(json.contains(".claude"));
        assert!(json.contains("data/memory"));
    }

    #[test]
    fn test_workspace_write_full_access() {
        let level = CortexSandboxLevel::WorkspaceWrite;
        assert!(level.allows_writes());
        assert!(level.allows_network());

        let json = level.to_codex_policy_json("/workspace");
        assert!(json.contains("WorkspaceWrite"));
        assert!(json.contains("/workspace"));
    }

    #[test]
    fn test_read_only_no_network() {
        let level = CortexSandboxLevel::ReadOnly;
        assert!(!level.allows_writes());
        assert!(!level.allows_network());

        let json = level.to_codex_policy_json("/workspace");
        assert!(json.contains("ReadOnly"));
    }

    #[test]
    fn test_patch_has_content() {
        let patch = generate_sandbox_bridge_patch();
        assert!(patch.contains("CORTEX SANDBOX BRIDGE"));
        assert!(patch.contains("CortexSandboxLevel"));
        assert!(patch.contains("--- a/"));
        assert!(patch.contains("+++ b/"));
    }
}
