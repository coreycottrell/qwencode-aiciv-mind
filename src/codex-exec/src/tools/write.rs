//! Write tool — write file contents.

use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct WriteTool;

#[async_trait]
impl ToolHandler for WriteTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args.get("file_path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("Missing 'file_path' parameter"),
        };

        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::err("Missing 'content' parameter"),
        };

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return ToolResult::err(format!("Failed to create directory: {e}"));
            }
        }

        match fs::write(path, content).await {
            Ok(()) => ToolResult::ok(format!("Wrote {} bytes to {path}", content.len())),
            Err(e) => ToolResult::err(format!("Failed to write '{path}': {e}")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write".into(),
            description: "Write content to a file (creates parent directories)".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["file_path", "content"],
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                }
            }),
            mutates: true,
        }
    }
}
