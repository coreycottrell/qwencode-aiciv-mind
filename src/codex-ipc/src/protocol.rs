//! MCP/JSON-RPC 2.0 protocol types for inter-mind communication.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: RequestId, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC request ID — integer or string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Integer(i64),
    String(String),
}

impl From<i64> for RequestId {
    fn from(val: i64) -> Self {
        RequestId::Integer(val)
    }
}

impl From<String> for RequestId {
    fn from(val: String) -> Self {
        RequestId::String(val)
    }
}

impl From<&str> for RequestId {
    fn from(val: &str) -> Self {
        RequestId::String(val.to_string())
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::Integer(i) => write!(f, "{i}"),
            RequestId::String(s) => write!(f, "{s}"),
        }
    }
}

// ── MCP Standard Methods ────────────────────────────────────────────────────

/// MCP initialize request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}

/// MCP initialize result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

// ── Cortex-Specific Methods ─────────────────────────────────────────────────

/// Parameters for `cortex/delegate` — assign a task to a child mind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateParams {
    pub task_id: String,
    pub description: String,
    pub context: Option<String>,
    pub parent_mind_id: String,
}

/// Result of `cortex/delegate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateResult {
    pub accepted: bool,
    pub mind_id: String,
    pub task_id: String,
    /// When a DelegateHandler is present, this contains the thinking result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    /// Number of ThinkLoop iterations used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    /// Number of tool calls made during thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls_made: Option<u32>,
    /// Whether the ThinkLoop completed naturally (vs hitting max iterations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}

/// Result returned by a DelegateHandler after processing a task.
#[derive(Debug, Clone)]
pub struct DelegateTaskResult {
    pub response: String,
    pub iterations: u32,
    pub tool_calls: u32,
    pub completed: bool,
}

/// Parameters for `cortex/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusParams {}

/// Result of `cortex/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub mind_id: String,
    pub role: String,
    pub status: String,
    pub current_task: Option<String>,
    pub children: Vec<String>,
}

/// Result of a completed task via `cortex/result`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultMessage {
    pub task_id: String,
    pub mind_id: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub learnings: Vec<String>,
}

/// MCP tools/list result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP tools/call params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// MCP tools/call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolCallContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

// ── Error Codes ─────────────────────────────────────────────────────────────

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_request() {
        let req = JsonRpcRequest::new(1i64, "initialize", None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"initialize\""));
    }

    #[test]
    fn serialize_response_success() {
        let resp = JsonRpcResponse::success(1i64.into(), serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn serialize_response_error() {
        let resp = JsonRpcResponse::error(1i64.into(), METHOD_NOT_FOUND, "not found");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn request_id_variants() {
        let id_int: RequestId = 42i64.into();
        let id_str: RequestId = "abc".into();
        assert_eq!(format!("{id_int}"), "42");
        assert_eq!(format!("{id_str}"), "abc");
    }

    #[test]
    fn delegate_params_roundtrip() {
        let params = DelegateParams {
            task_id: "t-001".into(),
            description: "Research Codex internals".into(),
            context: Some("Phase 2 work".into()),
            parent_mind_id: "primary".into(),
        };
        let json = serde_json::to_value(&params).unwrap();
        let back: DelegateParams = serde_json::from_value(json).unwrap();
        assert_eq!(back.task_id, "t-001");
    }
}
