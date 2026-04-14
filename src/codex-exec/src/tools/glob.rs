//! Glob tool — find files by pattern.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct GlobTool {
    workspace_root: PathBuf,
}

impl GlobTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolHandler for GlobTool {
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

        // Use find with -name for glob matching
        let output = Command::new("find")
            .arg(&search_path)
            .arg("-name")
            .arg(pattern)
            .arg("-type")
            .arg("f")
            .arg("-not")
            .arg("-path")
            .arg("*/target/*")
            .arg("-not")
            .arg("-path")
            .arg("*/.git/*")
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut paths: Vec<&str> = stdout.lines().collect();
                paths.sort();
                if paths.is_empty() {
                    ToolResult::ok("No files matched")
                } else {
                    ToolResult::ok(paths.join("\n"))
                }
            }
            Err(e) => ToolResult::err(format!("Glob failed: {e}")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "glob".into(),
            description: "Find files matching a glob pattern".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern (e.g. '*.rs', '*.toml')"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (default: workspace root)"
                    }
                }
            }),
            mutates: false,
        }
    }
}
