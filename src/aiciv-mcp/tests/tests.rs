//! Tests for aiciv-mcp server

use aiciv_mcp::{HengshiMcpServer, tools::{ToolCall, ToolCallResult}};
use std::collections::HashMap;

#[tokio::test]
async fn test_list_tools() {
    let server = HengshiMcpServer::new("hengshi");
    let tools = server.list_tools();
    assert!(!tools.is_empty(), "Should have at least one tool");
    let names: Vec<_> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(names.contains(&"hengshi_heartbeat".to_string()));
    assert!(names.contains(&"hengshi_tdd_cycle".to_string()));
    assert!(names.contains(&"hengshi_summarize_session".to_string()));
}

#[tokio::test]
async fn test_heartbeat_tool() {
    let server = HengshiMcpServer::new("hengshi")
        .with_hub_url("http://test-hub:8900");
    let mut args = HashMap::new();
    let call = ToolCall { name: "hengshi_heartbeat".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_none());
    assert_eq!(result.result.get("hub_url").and_then(|v| v.as_str()), Some("http://test-hub:8900"));
}

#[tokio::test]
async fn test_unknown_tool() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    let call = ToolCall { name: "nonexistent_tool".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_tdd_cycle_tool() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("function_name".to_string(), serde_json::json!("test_add"));
    args.insert("test_file".to_string(), serde_json::json!("tests/test_math.py"));
    let call = ToolCall { name: "hengshi_tdd_cycle".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_none());
    assert_eq!(result.result.get("function_name").and_then(|v| v.as_str()), Some("test_add"));
}

#[tokio::test]
async fn test_post_to_room_tool() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("room_id".to_string(), serde_json::json!("test-room-123"));
    args.insert("message".to_string(), serde_json::json!("Hello from tests"));
    args.insert("title".to_string(), serde_json::json!("Test message"));
    let call = ToolCall { name: "hengshi_post_to_room".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_none());
    assert_eq!(result.result.get("room_id").and_then(|v| v.as_str()), Some("test-room-123"));
}

#[tokio::test]
async fn test_poll_events_tool() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("agent_id".to_string(), serde_json::json!("agent-456"));
    args.insert("limit".to_string(), serde_json::json!(5));
    let call = ToolCall { name: "hengshi_poll_events".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_none());
    assert_eq!(result.result.get("agent_id").and_then(|v| v.as_str()), Some("agent-456"));
    assert_eq!(result.result.get("limit").and_then(|v| v.as_u64()), Some(5));
}