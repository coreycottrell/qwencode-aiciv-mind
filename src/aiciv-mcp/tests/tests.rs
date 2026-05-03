//! Tests for aiciv-mcp server

use aiciv_mcp::*;
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
    assert!(names.contains(&"hengshi_poll_events".to_string()));
    assert!(names.contains(&"hengshi_post_to_room".to_string()));
    assert!(names.contains(&"hengshi_compress_trajectory".to_string()));
}

#[tokio::test]
async fn test_unknown_tool() {
    let server = HengshiMcpServer::new("hengshi");
    let args = HashMap::new();
    let call = tools::ToolCall { name: "nonexistent_tool".to_string(), arguments: args };
    let result = server.execute_tool(call).await.unwrap();
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_compress_trajectory_rejects_missing_arg() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("something".to_string(), serde_json::json!("value"));
    let call = tools::ToolCall { name: "hengshi_compress_trajectory".to_string(), arguments: args };
    let result = server.execute_tool(call).await;
    // Result::Err when missing required arg
    assert!(result.is_err() || result.unwrap().error.is_some());
}

#[tokio::test]
async fn test_summarize_session_rejects_missing_arg() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("not_session_ledger".to_string(), serde_json::json!("value"));
    let call = tools::ToolCall { name: "hengshi_summarize_session".to_string(), arguments: args };
    let result = server.execute_tool(call).await;
    assert!(result.is_err() || result.unwrap().error.is_some());
}

#[tokio::test]
async fn test_tdd_cycle_rejects_missing_args() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("function_name".to_string(), serde_json::json!("test_add"));
    let call = tools::ToolCall { name: "hengshi_tdd_cycle".to_string(), arguments: args };
    let result = server.execute_tool(call).await;
    assert!(result.is_err() || result.unwrap().error.is_some());
}

#[tokio::test]
async fn test_poll_events_rejects_missing_agent_id() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("limit".to_string(), serde_json::json!(5));
    let call = tools::ToolCall { name: "hengshi_poll_events".to_string(), arguments: args };
    let result = server.execute_tool(call).await;
    assert!(result.is_err() || result.unwrap().error.is_some());
}

#[tokio::test]
async fn test_post_to_room_rejects_missing_message() {
    let server = HengshiMcpServer::new("hengshi");
    let mut args = HashMap::new();
    args.insert("room_id".to_string(), serde_json::json!("test-room-123"));
    let call = tools::ToolCall { name: "hengshi_post_to_room".to_string(), arguments: args };
    let result = server.execute_tool(call).await;
    assert!(result.is_err() || result.unwrap().error.is_some());
}