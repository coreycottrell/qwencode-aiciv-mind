//! Edit tool — surgical string replacement in files.
//!
//! Performs exact string replacements without rewriting the entire file.
//! The LLM specifies old_string and new_string; the tool finds and replaces.
//! If old_string appears multiple times and replace_all is false, the tool
//! errors asking for more context or the replace_all flag.

use async_trait::async_trait;
use tokio::fs;

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct EditTool;

#[async_trait]
impl ToolHandler for EditTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let path = match args.get("file_path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("Missing 'file_path' parameter"),
        };

        let old_string = match args.get("old_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::err("Missing 'old_string' parameter"),
        };

        let new_string = match args.get("new_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::err("Missing 'new_string' parameter"),
        };

        let replace_all = args
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Read the file
        let content = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return ToolResult::err(format!("Failed to read '{path}': {e}")),
        };

        // old_string must differ from new_string
        if old_string == new_string {
            return ToolResult::err("old_string and new_string are identical — nothing to change");
        }

        // Count occurrences
        let count = content.matches(old_string).count();

        if count == 0 {
            // Provide helpful context: show a snippet of the file
            let preview: String = content.lines().take(20).collect::<Vec<_>>().join("\n");
            return ToolResult::err(format!(
                "old_string not found in '{path}'. \
                 The file has {} lines. First 20 lines:\n{preview}",
                content.lines().count()
            ));
        }

        if count > 1 && !replace_all {
            // Find line numbers where old_string appears
            let mut locations = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.contains(old_string) {
                    locations.push(format!("  line {}: {}", i + 1, line.trim()));
                }
            }
            let loc_display = locations.join("\n");
            return ToolResult::err(format!(
                "old_string appears {count} times in '{path}'. \
                 Provide more surrounding context to make it unique, \
                 or set replace_all to true.\n\nOccurrences:\n{loc_display}"
            ));
        }

        // Perform the replacement
        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            // Exactly one occurrence — replacen(1) is safe
            content.replacen(old_string, new_string, 1)
        };

        // Write back
        match fs::write(path, &new_content).await {
            Ok(()) => {
                let verb = if replace_all && count > 1 {
                    format!("Replaced {count} occurrences")
                } else {
                    "Replaced 1 occurrence".to_string()
                };
                ToolResult::ok(format!("{verb} in {path}"))
            }
            Err(e) => ToolResult::err(format!("Failed to write '{path}': {e}")),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "edit".into(),
            description: "Perform exact string replacement in a file. \
                Finds old_string and replaces with new_string. \
                Fails if old_string is not found or is ambiguous (appears multiple times \
                without replace_all)."
                .into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["file_path", "old_string", "new_string"],
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact text to find and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The replacement text"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default: false). If false and old_string appears multiple times, the tool errors."
                    }
                }
            }),
            mutates: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs;

    async fn temp_file(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("codex-exec-edit-tests");
        fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join(name);
        fs::write(&path, content).await.unwrap();
        path
    }

    #[tokio::test]
    async fn edit_single_occurrence() {
        let path = temp_file("single.txt", "hello world\ngoodbye world\n").await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "hello world",
                "new_string": "hi world"
            }))
            .await;
        assert!(result.success, "Expected success: {:?}", result.error);
        let content = fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "hi world\ngoodbye world\n");
    }

    #[tokio::test]
    async fn edit_not_found() {
        let path = temp_file("notfound.txt", "hello world\n").await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "nonexistent",
                "new_string": "replacement"
            }))
            .await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn edit_ambiguous_without_replace_all() {
        let path = temp_file("ambiguous.txt", "foo bar\nfoo baz\n").await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "foo",
                "new_string": "qux"
            }))
            .await;
        assert!(!result.success);
        let err = result.error.unwrap();
        assert!(err.contains("2 times"), "Error: {err}");
        // File should be unchanged
        let content = fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "foo bar\nfoo baz\n");
    }

    #[tokio::test]
    async fn edit_replace_all() {
        let path = temp_file("replaceall.txt", "foo bar\nfoo baz\n").await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "foo",
                "new_string": "qux",
                "replace_all": true
            }))
            .await;
        assert!(result.success, "Expected success: {:?}", result.error);
        let content = fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "qux bar\nqux baz\n");
        assert!(result.output.contains("2 occurrences"));
    }

    #[tokio::test]
    async fn edit_identical_strings() {
        let path = temp_file("identical.txt", "hello\n").await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "hello",
                "new_string": "hello"
            }))
            .await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("identical"));
    }

    #[tokio::test]
    async fn edit_missing_file() {
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": "/tmp/codex-exec-edit-tests/nonexistent-file.txt",
                "old_string": "a",
                "new_string": "b"
            }))
            .await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Failed to read"));
    }

    #[tokio::test]
    async fn edit_multiline_replacement() {
        let path = temp_file(
            "multiline.txt",
            "fn main() {\n    println!(\"old\");\n}\n",
        )
        .await;
        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "file_path": path.to_str().unwrap(),
                "old_string": "    println!(\"old\");",
                "new_string": "    println!(\"new\");\n    println!(\"extra\");"
            }))
            .await;
        assert!(result.success, "Expected success: {:?}", result.error);
        let content = fs::read_to_string(&path).await.unwrap();
        assert_eq!(
            content,
            "fn main() {\n    println!(\"new\");\n    println!(\"extra\");\n}\n"
        );
    }
}
