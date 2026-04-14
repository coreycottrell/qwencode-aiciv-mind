//! Read tool — read file contents.

use async_trait::async_trait;
use tokio::fs;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct ReadTool;

#[async_trait]
impl ToolHandler for ReadTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args.get("file_path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("Missing 'file_path' parameter"),
        };

        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(2000) as usize;

        match fs::read_to_string(path).await {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = offset.min(lines.len());
                let end = (start + limit).min(lines.len());

                let mut output = String::new();
                for (i, line) in lines[start..end].iter().enumerate() {
                    let line_num = start + i + 1;
                    output.push_str(&format!("{line_num:>6}\t{line}\n"));
                }
                ToolResult::ok(output)
            }
            Err(e) => ToolResult::err(format!("Failed to read '{path}': {e}")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read".into(),
            description: "Read a file's contents with line numbers".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["file_path"],
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line number to start from (0-indexed, default: 0)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max lines to read (default: 2000)"
                    }
                }
            }),
            mutates: false,
        }
    }
}
