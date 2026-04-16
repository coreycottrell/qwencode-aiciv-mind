//! Bash tool — execute shell commands.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct BashTool {
    workspace_root: PathBuf,
}

impl BashTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolHandler for BashTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let command = match args.get("command").and_then(|v| v.as_str()) {
            Some(cmd) => cmd,
            None => return ToolResult::err("Missing 'command' parameter"),
        };

        let timeout_ms = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(120_000);

        // TODO(Sprint-future): Wrap bash execution in Landlock/bwrap for process isolation.
        // Currently bare `bash -c` — sandbox.rs is policy-only, not execution isolation.
        // See: Proof RED TEAM Finding 6 (2026-04-16)
        match tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(&self.workspace_root)
                .output(),
        )
        .await
        {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    let mut out = stdout.to_string();
                    if !stderr.is_empty() {
                        out.push_str("\n[stderr]\n");
                        out.push_str(&stderr);
                    }
                    ToolResult::ok(out)
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    ToolResult {
                        success: false,
                        output: stdout.to_string(),
                        error: Some(format!("Exit code {code}\n{stderr}")),
                    }
                }
            }
            Ok(Err(e)) => ToolResult::err(format!("Failed to spawn: {e}")),
            Err(_) => ToolResult::err(format!("Command timed out after {timeout_ms}ms")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "bash".into(),
            description: "Execute a bash command in the workspace directory".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in milliseconds (default: 120000)"
                    }
                }
            }),
            mutates: true,
        }
    }
}
