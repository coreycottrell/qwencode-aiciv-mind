//! Sandbox enforcement — the last line of defense.
//!
//! Maps codex-roles SandboxLevel to actual filesystem/command constraints.
//! This is the policy layer that prevents a compromised agent from
//! escaping its workspace.

use codex_roles::SandboxLevel;
use std::path::{Path, PathBuf};

use crate::registry::ExecError;

/// Paths that Cortex must NEVER write to, regardless of sandbox level.
/// These enforce publishing gates — content stays in data/ until reviewed.
const FORBIDDEN_WRITE_PATHS: &[&str] = &[
    "/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc",
    "projects/aiciv-inc",
];

/// Commands that Cortex must NEVER execute — publishing gates.
const FORBIDDEN_COMMANDS: &[&str] = &[
    "netlify deploy",
    "netlify-deploy",
];

/// Enforces sandbox policies on tool execution.
pub struct SandboxEnforcer {
    /// Root workspace directory. Agents can only write within this.
    workspace_root: PathBuf,
}

impl SandboxEnforcer {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Check whether a mutation is allowed under the given sandbox level.
    pub fn check_mutation(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
        level: SandboxLevel,
    ) -> Result<(), ExecError> {
        match level {
            SandboxLevel::ReadOnlyCoordination => {
                // Primary: no mutations allowed at all
                Err(ExecError::SandboxViolation(format!(
                    "ReadOnlyCoordination: mutation via '{tool_name}' denied"
                )))
            }
            SandboxLevel::TeamScoped => {
                // Team lead: only scratchpad writes
                if tool_name.contains("scratchpad_write") || tool_name == "memory_write" {
                    Ok(())
                } else {
                    Err(ExecError::SandboxViolation(format!(
                        "TeamScoped: only scratchpad/memory writes allowed, not '{tool_name}'"
                    )))
                }
            }
            SandboxLevel::WorkspaceWrite => {
                // Agent: can write, but only within workspace
                if let Some(path) = extract_path_from_args(tool_name, args) {
                    self.check_path_within_workspace(&path)?;
                }
                // Check for dangerous commands
                if tool_name == "bash" {
                    if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                        self.check_command_safety(cmd)?;
                    }
                }
                Ok(())
            }
            SandboxLevel::ReadOnly => {
                // Red team: no mutations
                Err(ExecError::SandboxViolation(format!(
                    "ReadOnly: mutation via '{tool_name}' denied"
                )))
            }
        }
    }

    /// Verify a path is within the workspace root and not in a forbidden zone.
    fn check_path_within_workspace(&self, path: &Path) -> Result<(), ExecError> {
        // Normalize the path (resolve .. and symlinks conceptually)
        let normalized = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Check containment
        if !normalized.starts_with(&self.workspace_root) {
            return Err(ExecError::SandboxViolation(format!(
                "Path '{}' escapes workspace '{}'",
                normalized.display(),
                self.workspace_root.display()
            )));
        }

        // Check forbidden write paths (publishing gate)
        let path_str = normalized.display().to_string();
        for forbidden in FORBIDDEN_WRITE_PATHS {
            if path_str.contains(forbidden) {
                return Err(ExecError::SandboxViolation(format!(
                    "PUBLISHING GATE: write to '{}' blocked — content must stay in data/ until reviewed by ACG Primary + Corey",
                    normalized.display()
                )));
            }
        }

        Ok(())
    }

    /// Check a bash command for dangerous patterns and forbidden commands.
    fn check_command_safety(&self, command: &str) -> Result<(), ExecError> {
        let dangerous = [
            "rm -rf /",
            "rm -rf ~",
            "rm -rf $HOME",
            ":(){ :|:& };:",  // fork bomb
            "> /dev/sda",
            "mkfs.",
            "dd if=/dev/zero",
            "--force",
            "chmod 777 /",
            "curl | bash",
            "wget | bash",
        ];

        let lower = command.to_lowercase();
        for pattern in &dangerous {
            if lower.contains(&pattern.to_lowercase()) {
                return Err(ExecError::SandboxViolation(format!(
                    "Dangerous command pattern detected: '{pattern}'"
                )));
            }
        }

        // Publishing gate — block deploy commands
        for forbidden in FORBIDDEN_COMMANDS {
            if lower.contains(&forbidden.to_lowercase()) {
                return Err(ExecError::SandboxViolation(format!(
                    "PUBLISHING GATE: '{}' blocked — deployment requires ACG Primary + Corey review",
                    forbidden
                )));
            }
        }

        // Publishing gate — block writes into forbidden paths via bash
        for forbidden_path in FORBIDDEN_WRITE_PATHS {
            // Check for cp/mv/tee/redirect targeting the forbidden path
            if (lower.contains("cp ") || lower.contains("mv ") || lower.contains("tee ") || lower.contains("> "))
                && command.contains(forbidden_path)
            {
                return Err(ExecError::SandboxViolation(format!(
                    "PUBLISHING GATE: bash write to '{}' blocked — content must stay in data/",
                    forbidden_path
                )));
            }
        }

        Ok(())
    }
}

/// Extract a file path from tool arguments, if present.
fn extract_path_from_args(tool_name: &str, args: &serde_json::Value) -> Option<PathBuf> {
    match tool_name {
        "read" | "write" | "edit" => args
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        "glob" => args
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        "grep" => args
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enforcer() -> SandboxEnforcer {
        SandboxEnforcer::new(PathBuf::from("/home/corey/projects/test"))
    }

    #[test]
    fn workspace_write_allows_internal_path() {
        let e = enforcer();
        let args = serde_json::json!({"file_path": "/home/corey/projects/test/src/main.rs"});
        assert!(e.check_mutation("write", &args, SandboxLevel::WorkspaceWrite).is_ok());
    }

    #[test]
    fn workspace_write_denies_escape() {
        let e = enforcer();
        let args = serde_json::json!({"file_path": "/etc/passwd"});
        assert!(e.check_mutation("write", &args, SandboxLevel::WorkspaceWrite).is_err());
    }

    #[test]
    fn read_only_coordination_denies_all_mutations() {
        let e = enforcer();
        let args = serde_json::json!({"command": "echo hello"});
        assert!(e.check_mutation("bash", &args, SandboxLevel::ReadOnlyCoordination).is_err());
    }

    #[test]
    fn team_scoped_allows_scratchpad() {
        let e = enforcer();
        let args = serde_json::json!({"content": "notes"});
        assert!(e.check_mutation("team_scratchpad_write", &args, SandboxLevel::TeamScoped).is_ok());
        assert!(e.check_mutation("memory_write", &args, SandboxLevel::TeamScoped).is_ok());
    }

    #[test]
    fn team_scoped_denies_bash() {
        let e = enforcer();
        let args = serde_json::json!({"command": "ls"});
        assert!(e.check_mutation("bash", &args, SandboxLevel::TeamScoped).is_err());
    }

    #[test]
    fn dangerous_command_blocked() {
        let e = enforcer();
        let args = serde_json::json!({"command": "rm -rf /"});
        assert!(e.check_mutation("bash", &args, SandboxLevel::WorkspaceWrite).is_err());
    }

    #[test]
    fn safe_command_allowed() {
        let e = enforcer();
        let args = serde_json::json!({"command": "cargo test"});
        assert!(e.check_mutation("bash", &args, SandboxLevel::WorkspaceWrite).is_ok());
    }

    #[test]
    fn read_only_denies_mutations() {
        let e = enforcer();
        let args = serde_json::json!({"file_path": "/home/corey/projects/test/x.rs"});
        assert!(e.check_mutation("write", &args, SandboxLevel::ReadOnly).is_err());
    }

    // ── Publishing gate tests ──

    #[test]
    fn publishing_gate_blocks_aiciv_inc_write() {
        // Use a broad workspace root so the path passes containment but hits forbidden check
        let e = SandboxEnforcer::new(PathBuf::from("/home/corey/projects/AI-CIV"));
        let args = serde_json::json!({
            "file_path": "/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/posts/test.html"
        });
        let result = e.check_mutation("write", &args, SandboxLevel::WorkspaceWrite);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("PUBLISHING GATE"));
    }

    #[test]
    fn publishing_gate_also_blocked_by_workspace_when_narrow() {
        // When workspace is narrow (aiciv-mind-cubed), ACG paths are caught by workspace containment
        let e = SandboxEnforcer::new(PathBuf::from("/home/corey/projects/AI-CIV/aiciv-mind-cubed"));
        let args = serde_json::json!({
            "file_path": "/home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/posts/test.html"
        });
        let result = e.check_mutation("write", &args, SandboxLevel::WorkspaceWrite);
        assert!(result.is_err()); // Blocked by workspace containment
    }

    #[test]
    fn publishing_gate_blocks_netlify_deploy() {
        let e = enforcer();
        let args = serde_json::json!({"command": "cd projects/aiciv-inc && netlify deploy --prod --dir=."});
        let result = e.check_mutation("bash", &args, SandboxLevel::WorkspaceWrite);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("PUBLISHING GATE"));
    }

    #[test]
    fn publishing_gate_blocks_bash_cp_to_aiciv_inc() {
        let e = enforcer();
        let args = serde_json::json!({
            "command": "cp data/audio/test.mp3 /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc/blog/audio/"
        });
        let result = e.check_mutation("bash", &args, SandboxLevel::WorkspaceWrite);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("PUBLISHING GATE"));
    }

    #[test]
    fn publishing_gate_allows_data_content_write() {
        let e = SandboxEnforcer::new(PathBuf::from("/home/corey/projects/AI-CIV/aiciv-mind-cubed"));
        let args = serde_json::json!({
            "file_path": "/home/corey/projects/AI-CIV/aiciv-mind-cubed/data/content/blog/test.html"
        });
        assert!(e.check_mutation("write", &args, SandboxLevel::WorkspaceWrite).is_ok());
    }
}
