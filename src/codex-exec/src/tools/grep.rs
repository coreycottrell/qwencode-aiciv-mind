//! Grep tool — search file contents by regex.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct GrepTool {
    workspace_root: PathBuf,
}

impl GrepTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolHandler for GrepTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("Missing 'pattern' parameter"),
        };

        let search_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.workspace_root.clone());

        let output_mode = args
            .get("output_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("content");

        let mut cmd = Command::new("grep");
        cmd.arg("-rn") // recursive + line numbers
            .arg("--include=*.rs")
            .arg("--include=*.toml")
            .arg("--include=*.sql")
            .arg("--include=*.md")
            .arg("--include=*.json")
            .arg("--include=*.py")
            .arg("--include=*.ts")
            .arg("--include=*.js");

        if output_mode == "files_with_matches" {
            cmd.arg("-l");
        } else if output_mode == "count" {
            cmd.arg("-c");
        }

        // Context lines
        if let Some(ctx) = args.get("context").and_then(|v| v.as_u64()) {
            cmd.arg(format!("-C{ctx}"));
        }

        cmd.arg("-E") // extended regex
            .arg(pattern)
            .arg(&search_path);

        match cmd.output().await {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.is_empty() {
                    ToolResult::ok("No matches found")
                } else {
                    // Limit output to prevent flooding
                    let lines: Vec<&str> = stdout.lines().collect();
                    let limit = args
                        .get("head_limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(100) as usize;
                    let truncated = lines.len() > limit;
                    let output: String = lines[..limit.min(lines.len())].join("\n");
                    if truncated {
                        ToolResult::ok(format!(
                            "{output}\n\n[... truncated, showing {limit} of {} matches]",
                            lines.len()
                        ))
                    } else {
                        ToolResult::ok(output)
                    }
                }
            }
            Err(e) => ToolResult::err(format!("Grep failed: {e}")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "grep".into(),
            description: "Search file contents with regex patterns".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory to search (default: workspace root)"
                    },
                    "output_mode": {
                        "type": "string",
                        "enum": ["content", "files_with_matches", "count"],
                        "description": "Output format (default: content)"
                    },
                    "context": {
                        "type": "integer",
                        "description": "Lines of context around matches"
                    },
                    "head_limit": {
                        "type": "integer",
                        "description": "Max output lines (default: 100)"
                    }
                }
            }),
            mutates: false,
        }
    }
}
