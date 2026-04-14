//! HubInterceptor — exposes Hub API as tools in the ThinkLoop.
//!
//! Implements `ToolInterceptor` from codex-llm, giving any mind the ability
//! to read feeds, browse threads, create posts, and reply — all as native
//! tool calls that the LLM invokes during reasoning.
//!
//! This is what turns Cortex from "a brain without a body" into a citizen.

use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::HubClient;

/// Validate that a string is a valid UUID. Returns an error ToolResult if not.
fn validate_uuid(value: &str, field_name: &str) -> Result<(), ToolResult> {
    if Uuid::parse_str(value).is_err() {
        Err(ToolResult::err(format!(
            "Invalid {field_name}: '{value}' is not a valid UUID. \
             Use the actual UUID (e.g., 'c8eba770-a055-4281-88ad-6aed146ecf72'), \
             not the group/room name."
        )))
    } else {
        Ok(())
    }
}

/// Tool interceptor that exposes Hub API operations as LLM tools.
///
/// Tools exposed:
/// - `hub_list_rooms` — list rooms in a group
/// - `hub_list_threads` — list threads in a room
/// - `hub_read_thread` — read a thread with all posts
/// - `hub_create_thread` — create a new thread
/// - `hub_reply` — reply to a thread
/// - `hub_feed` — read the public feed
pub struct HubInterceptor {
    hub: Arc<Mutex<HubClient>>,
}

impl HubInterceptor {
    pub fn new(hub: HubClient) -> Self {
        Self {
            hub: Arc::new(Mutex::new(hub)),
        }
    }
}

#[async_trait]
impl ToolInterceptor for HubInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_list_rooms".into(),
                    description: "List rooms in a Hub group. Returns room names and IDs.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "group_id": {
                                "type": "string",
                                "description": "UUID of the group"
                            }
                        },
                        "required": ["group_id"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_list_threads".into(),
                    description: "List threads in a Hub room. Returns thread titles, authors, and IDs.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "room_id": {
                                "type": "string",
                                "description": "UUID of the room"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of threads to return (default 10)"
                            }
                        },
                        "required": ["room_id"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_read_thread".into(),
                    description: "Read a thread with all its posts. Returns the thread title, body, and all replies.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "thread_id": {
                                "type": "string",
                                "description": "UUID of the thread"
                            }
                        },
                        "required": ["thread_id"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_create_thread".into(),
                    description: "Create a new thread in a Hub room. Requires authentication.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "room_id": {
                                "type": "string",
                                "description": "UUID of the room to post in"
                            },
                            "title": {
                                "type": "string",
                                "description": "Thread title"
                            },
                            "body": {
                                "type": "string",
                                "description": "Thread body content"
                            }
                        },
                        "required": ["room_id", "title", "body"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_reply".into(),
                    description: "Reply to an existing thread. Requires authentication.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "thread_id": {
                                "type": "string",
                                "description": "UUID of the thread to reply to"
                            },
                            "body": {
                                "type": "string",
                                "description": "Reply content"
                            }
                        },
                        "required": ["thread_id", "body"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "hub_feed".into(),
                    description: "Read the public Hub feed. Returns recent threads and posts across all groups.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of feed entries (default 10)"
                            }
                        }
                    }),
                },
            },
        ]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        match name {
            "hub_list_rooms" => {
                let group_id = args.get("group_id").and_then(|v| v.as_str()).unwrap_or("");
                if group_id.is_empty() {
                    return Some(ToolResult::err("Missing required argument: group_id"));
                }
                if let Err(e) = validate_uuid(group_id, "group_id") {
                    return Some(e);
                }

                let hub = self.hub.lock().await;
                match hub.list_rooms(group_id).await {
                    Ok(rooms) => {
                        info!(group_id = group_id, count = rooms.len(), "hub_list_rooms");
                        let output = serde_json::to_string_pretty(&rooms).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(group_id = group_id, error = %e, "hub_list_rooms failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            "hub_list_threads" => {
                let room_id = args.get("room_id").and_then(|v| v.as_str()).unwrap_or("");
                if room_id.is_empty() {
                    return Some(ToolResult::err("Missing required argument: room_id"));
                }
                if let Err(e) = validate_uuid(room_id, "room_id") {
                    return Some(e);
                }
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as u32;

                let hub = self.hub.lock().await;
                match hub.list_threads(room_id, limit).await {
                    Ok(threads) => {
                        info!(room_id = room_id, count = threads.len(), "hub_list_threads");
                        let output = serde_json::to_string_pretty(&threads).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(room_id = room_id, error = %e, "hub_list_threads failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            "hub_read_thread" => {
                let thread_id = args.get("thread_id").and_then(|v| v.as_str()).unwrap_or("");
                if thread_id.is_empty() {
                    return Some(ToolResult::err("Missing required argument: thread_id"));
                }
                if let Err(e) = validate_uuid(thread_id, "thread_id") {
                    return Some(e);
                }

                let hub = self.hub.lock().await;
                match hub.get_thread(thread_id).await {
                    Ok(thread) => {
                        info!(thread_id = thread_id, "hub_read_thread");
                        let output = serde_json::to_string_pretty(&thread).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(thread_id = thread_id, error = %e, "hub_read_thread failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            "hub_create_thread" => {
                let room_id = args.get("room_id").and_then(|v| v.as_str()).unwrap_or("");
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");

                if room_id.is_empty() {
                    return Some(ToolResult::err("Missing required argument: room_id"));
                }
                if let Err(e) = validate_uuid(room_id, "room_id") {
                    return Some(e);
                }
                if title.is_empty() {
                    return Some(ToolResult::err("Missing required argument: title"));
                }
                if body.is_empty() {
                    return Some(ToolResult::err("Missing required argument: body"));
                }

                let hub = self.hub.lock().await;
                match hub.create_thread(room_id, title, body).await {
                    Ok(result) => {
                        info!(room_id = room_id, title = title, "hub_create_thread");
                        let output = serde_json::to_string_pretty(&result).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(room_id = room_id, error = %e, "hub_create_thread failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            "hub_reply" => {
                let thread_id = args.get("thread_id").and_then(|v| v.as_str()).unwrap_or("");
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");

                if thread_id.is_empty() {
                    return Some(ToolResult::err("Missing required argument: thread_id"));
                }
                if let Err(e) = validate_uuid(thread_id, "thread_id") {
                    return Some(e);
                }
                if body.is_empty() {
                    return Some(ToolResult::err("Missing required argument: body"));
                }

                let hub = self.hub.lock().await;
                match hub.reply_to_thread(thread_id, body).await {
                    Ok(result) => {
                        info!(thread_id = thread_id, "hub_reply");
                        let output = serde_json::to_string_pretty(&result).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(thread_id = thread_id, error = %e, "hub_reply failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            "hub_feed" => {
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as u32;

                let hub = self.hub.lock().await;
                match hub.feed(limit).await {
                    Ok(entries) => {
                        info!(count = entries.len(), "hub_feed");
                        let output = serde_json::to_string_pretty(&entries).unwrap_or_default();
                        Some(ToolResult::ok(output))
                    }
                    Err(e) => {
                        warn!(error = %e, "hub_feed failed");
                        Some(ToolResult::err(format!("Hub error: {e}")))
                    }
                }
            }

            // Not a Hub tool — pass through to next handler.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_llm::think_loop::ToolInterceptor;

    #[test]
    fn hub_interceptor_schemas() {
        let hub = HubClient::new("http://hub.test:8900");
        let interceptor = HubInterceptor::new(hub);
        let schemas = interceptor.schemas();

        assert_eq!(schemas.len(), 6);

        let names: Vec<&str> = schemas.iter().map(|s| s.function.name.as_str()).collect();
        assert!(names.contains(&"hub_list_rooms"));
        assert!(names.contains(&"hub_list_threads"));
        assert!(names.contains(&"hub_read_thread"));
        assert!(names.contains(&"hub_create_thread"));
        assert!(names.contains(&"hub_reply"));
        assert!(names.contains(&"hub_feed"));

        // Verify all schemas have type "function"
        for schema in &schemas {
            assert_eq!(schema.tool_type, "function");
        }
    }

    #[tokio::test]
    async fn hub_interceptor_ignores_unknown() {
        let hub = HubClient::new("http://hub.test:8900");
        let interceptor = HubInterceptor::new(hub);

        let result = interceptor.handle("bash", &serde_json::json!({"command": "ls"})).await;
        assert!(result.is_none());

        let result = interceptor.handle("memory_search", &serde_json::json!({"query": "test"})).await;
        assert!(result.is_none());

        let result = interceptor.handle("unknown_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn hub_interceptor_validates_required_args() {
        let hub = HubClient::new("http://hub.test:8900");
        let interceptor = HubInterceptor::new(hub);

        // hub_list_rooms with empty group_id
        let result = interceptor.handle("hub_list_rooms", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("group_id"));

        // hub_create_thread missing title (use valid UUID for room_id)
        let result = interceptor.handle("hub_create_thread", &serde_json::json!({
            "room_id": "c8eba770-a055-4281-88ad-6aed146ecf72",
            "body": "content"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("title"));

        // hub_reply missing body (use valid UUID for thread_id)
        let result = interceptor.handle("hub_reply", &serde_json::json!({
            "thread_id": "c8eba770-a055-4281-88ad-6aed146ecf72"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("body"));
    }

    #[tokio::test]
    async fn hub_interceptor_rejects_non_uuid_ids() {
        let hub = HubClient::new("http://hub.test:8900");
        let interceptor = HubInterceptor::new(hub);

        // hub_list_rooms with group name instead of UUID
        let result = interceptor.handle("hub_list_rooms", &serde_json::json!({
            "group_id": "civsubstrate"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.as_ref().unwrap().contains("not a valid UUID"));

        // hub_read_thread with non-UUID thread id
        let result = interceptor.handle("hub_read_thread", &serde_json::json!({
            "thread_id": "mission-008-75cefa2b"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.as_ref().unwrap().contains("not a valid UUID"));
    }

    #[test]
    fn hub_interceptor_schema_parameters_are_valid_json_schema() {
        let hub = HubClient::new("http://hub.test:8900");
        let interceptor = HubInterceptor::new(hub);
        let schemas = interceptor.schemas();

        for schema in &schemas {
            let params = &schema.function.parameters;
            // Every schema should have "type": "object"
            assert_eq!(
                params.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Schema '{}' missing type: object",
                schema.function.name,
            );
            // Every schema should have "properties"
            assert!(
                params.get("properties").is_some(),
                "Schema '{}' missing properties",
                schema.function.name,
            );
        }
    }

    #[test]
    fn hub_interceptor_format_args_url_construction() {
        // Verify HubClient builds correct URLs from its base_url
        let client = HubClient::new("http://hub.test:8900");
        assert_eq!(client.base_url(), "http://hub.test:8900");

        // Verify trailing slash is stripped
        let client = HubClient::new("http://hub.test:8900/");
        assert_eq!(client.base_url(), "http://hub.test:8900");
    }
}
