//! aiciv-mcp tools — MCP tool definitions for Hengshi capabilities

use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;

/// Tool definition matching MCP spec.
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool call from MCP client.
#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub arguments: std::collections::HashMap<String, Value>,
}

/// Tool call result returned to MCP client.
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub result: Value,
    pub error: Option<String>,
}

impl ToolCallResult {
    pub fn ok(result: Value) -> Self {
        Self { result, error: None }
    }
    pub fn err(error: impl Into<String>) -> Self {
        Self { result: Value::Null, error: Some(error.into()) }
    }
}

/// Return project root (workspace root = qwen-aiciv-mind/).
fn project_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = /path/to/qwen-aiciv-mind/src/aiciv-mcp
    // 1 parent: /path/to/qwen-aiciv-mind/src
    // 2 parents: /path/to/qwen-aiciv-mind  ← workspace root
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf()
}

/// Run a Python skill script and return its stdout as JSON.
async fn run_skill(script: &str, args: &[&str]) -> Result<String> {
    let root = project_root();
    let script_path = root.join(script);
    let mut cmd = Command::new("python3");
    cmd.arg(script_path);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        anyhow::bail!("skill script failed (exit {}): {} {}", output.status, stdout, stderr);
    }
    Ok(stdout.to_string())
}

/// All available tools.
pub fn all_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "hengshi_summarize_session".to_string(),
            description: "Summarize a session ledger — compresses conversation history into a concise summary".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_ledger": {
                        "type": "string",
                        "description": "Path to session ledger file"
                    }
                },
                "required": ["session_ledger"]
            }),
        },
        Tool {
            name: "hengshi_compress_trajectory".to_string(),
            description: "Compress a conversation trajectory using LLM-based summarization".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trajectory_file": {
                        "type": "string",
                        "description": "Path to trajectory file"
                    }
                },
                "required": ["trajectory_file"]
            }),
        },
        Tool {
            name: "hengshi_tdd_cycle".to_string(),
            description: "Run a TDD cycle on a function — RED phase, GREEN phase, REFACTOR".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "function_name": {
                        "type": "string",
                        "description": "Name of function to develop via TDD"
                    },
                    "test_file": {
                        "type": "string",
                        "description": "Path to test file"
                    }
                },
                "required": ["function_name", "test_file"]
            }),
        },
        Tool {
            name: "hengshi_heartbeat".to_string(),
            description: "Send presence heartbeat to Hub coordination system".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "hub_url": {
                        "type": "string",
                        "description": "Hub URL (optional, uses default)"
                    }
                }
            }),
        },
        Tool {
            name: "hengshi_post_to_room".to_string(),
            description: "Post a message to a Hub coordination room".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "room_id": {
                        "type": "string",
                        "description": "Room ID to post to"
                    },
                    "message": {
                        "type": "string",
                        "description": "Message body"
                    },
                    "title": {
                        "type": "string",
                        "description": "Message title"
                    }
                },
                "required": ["room_id", "message"]
            }),
        },
        Tool {
            name: "hengshi_poll_events".to_string(),
            description: "Poll AgentEvents for new messages directed to this agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "Agent ID to poll events for"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max events to return (default 10)"
                    }
                },
                "required": ["agent_id"]
            }),
        },
    ]
}

/// Execute a tool call.
pub async fn execute(
    call: ToolCall,
    hub_url: &Option<String>,
    _agent_id: &Option<String>,
) -> Result<ToolCallResult> {
    match call.name.as_str() {
        "hengshi_summarize_session" => {
            let session_ledger = call.arguments.get("session_ledger")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing session_ledger"))?;
            let script = "skills/session-summarization/summarize.py";
            let out = run_skill(script, &[session_ledger]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        "hengshi_compress_trajectory" => {
            let trajectory_file = call.arguments.get("trajectory_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing trajectory_file"))?;
            let script = "skills/trajectory-compressor/trajectory_compressor.py";
            let out = run_skill(script, &["compress", trajectory_file]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        "hengshi_tdd_cycle" => {
            let function_name = call.arguments.get("function_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing function_name"))?;
            let test_file = call.arguments.get("test_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing test_file"))?;
            let out = run_skill("skills/tdd/test_tdd_cycle.py", &[
                "--function", function_name,
                "--test-file", test_file,
            ]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        "hengshi_heartbeat" => {
            let hub = hub_url.clone().unwrap_or_else(|| "http://87.99.131.49:8900".to_string());
            let out = run_skill("skills/hub-triad/triad_client.py", &["heartbeat", &hub]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        "hengshi_post_to_room" => {
            let room_id = call.arguments.get("room_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing room_id"))?;
            let message = call.arguments.get("message")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing message"))?;
            let title = call.arguments.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("MCP message");
            let out = run_skill("skills/hub-triad/triad_client.py", &["post", message, room_id, title]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        "hengshi_poll_events" => {
            let agent_id = call.arguments.get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing agent_id"))?;
            let out = run_skill("skills/hub-triad/triad_client.py", &["poll", agent_id]).await?;
            match serde_json::from_str::<Value>(&out) {
                Ok(v) => Ok(ToolCallResult::ok(v)),
                Err(_) => Ok(ToolCallResult::ok(json!({"raw": out}))),
            }
        }
        _ => Ok(ToolCallResult::err("unknown tool")),
    }
}