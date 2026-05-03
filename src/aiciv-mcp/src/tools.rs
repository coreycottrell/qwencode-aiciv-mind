//! aiciv-mcp tools — MCP tool definitions for Hengshi capabilities

use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    agent_id: &Option<String>,
) -> Result<ToolCallResult> {
    match call.name.as_str() {
        "hengshi_summarize_session" => {
            let session_ledger = call.arguments.get("session_ledger")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing session_ledger"))?;
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_summarize_session",
                "session_ledger": session_ledger,
                "note": "Summarization would call session-summarization skill"
            })))
        }
        "hengshi_compress_trajectory" => {
            let trajectory_file = call.arguments.get("trajectory_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing trajectory_file"))?;
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_compress_trajectory",
                "trajectory_file": trajectory_file,
                "note": "Compression would call trajectory-compressor skill"
            })))
        }
        "hengshi_tdd_cycle" => {
            let function_name = call.arguments.get("function_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing function_name"))?;
            let test_file = call.arguments.get("test_file")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing test_file"))?;
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_tdd_cycle",
                "function_name": function_name,
                "test_file": test_file,
                "note": "TDD cycle would call tdd skill"
            })))
        }
        "hengshi_heartbeat" => {
            let hub = hub_url.clone().unwrap_or_else(|| "http://87.99.131.49:8900".to_string());
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_heartbeat",
                "hub_url": hub,
                "note": "Heartbeat would use hub-triad skill"
            })))
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
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_post_to_room",
                "room_id": room_id,
                "message": message,
                "title": title,
                "note": "Post would use hub-triad skill"
            })))
        }
        "hengshi_poll_events" => {
            let agent_id = call.arguments.get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing agent_id"))?;
            let limit = call.arguments.get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;
            Ok(ToolCallResult::ok(json!({
                "status": "ok",
                "tool": "hengshi_poll_events",
                "agent_id": agent_id,
                "limit": limit,
                "events": [],
                "note": "Polling would use hub-triad skill"
            })))
        }
        _ => Ok(ToolCallResult::err("unknown tool")),
    }
}