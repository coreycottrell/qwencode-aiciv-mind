//! # InputRouteInterceptor — Route external signals through InputMux.
//!
//! Exposes an `input_route` tool that lets any mind ask "where should this
//! input go?" during ThinkLoop reasoning. The InputMux applies hard-coded
//! routing rules and priority escalation to return the target mind.
//!
//! This wires the existing InputMux into the live runtime without requiring
//! a full async event loop — the LLM explicitly routes signals it discovers
//! (e.g., from Hub feed, notifications) through the coordination substrate.

use async_trait::async_trait;
use codex_coordination::InputMux;
use codex_coordination::types::{ExternalInput, InputSource, InputPriority, RoutingDecision};
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};

/// Interceptor that exposes input routing to the ThinkLoop.
pub struct InputRouteInterceptor {
    mux: InputMux,
}

impl InputRouteInterceptor {
    pub fn new(primary_mind_id: &str) -> Self {
        Self {
            mux: InputMux::new(codex_coordination::types::MindId(primary_mind_id.into())),
        }
    }
}

#[async_trait]
impl ToolInterceptor for InputRouteInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "input_route".into(),
                description: "Route an external input through the InputMux. Given a source type and content, returns which mind should handle it (direct route to a team lead, or escalate to Primary). Use when you receive external signals (Hub notifications, timer events, messages) and need to decide where to route them.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_type": {
                            "type": "string",
                            "enum": ["hub", "telegram", "boop", "schedule", "human", "ipc"],
                            "description": "Type of external input source."
                        },
                        "source_id": {
                            "type": "string",
                            "description": "Source identifier: room name for hub, chat_id for telegram, boop_type for boop, trigger_name for schedule, name for human, sender for ipc."
                        },
                        "content": {
                            "type": "string",
                            "description": "The input content to route."
                        },
                        "priority": {
                            "type": "string",
                            "enum": ["low", "normal", "high", "critical"],
                            "description": "Input priority. Default: normal."
                        }
                    },
                    "required": ["source_type", "content"]
                }),
            },
        }]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        if name != "input_route" {
            return None;
        }

        let source_type = match args.get("source_type").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Some(ToolResult::err("Missing required parameter: source_type")),
        };

        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Some(ToolResult::err("Missing required parameter: content")),
        };

        let source_id = args.get("source_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let priority = match args.get("priority").and_then(|v| v.as_str()) {
            Some("low") => InputPriority::Low,
            Some("high") => InputPriority::High,
            Some("critical") => InputPriority::Critical,
            _ => InputPriority::Normal,
        };

        let source = match source_type {
            "hub" => InputSource::Hub {
                room: source_id.to_string(),
                thread: None,
            },
            "telegram" => InputSource::Telegram {
                chat_id: source_id.to_string(),
            },
            "boop" => InputSource::Boop {
                boop_type: source_id.to_string(),
            },
            "schedule" => InputSource::Schedule {
                trigger_name: source_id.to_string(),
            },
            "human" => InputSource::Human {
                name: source_id.to_string(),
            },
            "ipc" => InputSource::Ipc {
                sender: source_id.to_string(),
            },
            other => return Some(ToolResult::err(
                format!("Unknown source_type: {}. Use: hub, telegram, boop, schedule, human, ipc", other)
            )),
        };

        let input = ExternalInput {
            source,
            content: content.to_string(),
            priority,
            timestamp: chrono::Utc::now(),
            metadata: serde_json::Value::Null,
        };

        let decision = self.mux.route(&input);

        let result = match decision {
            RoutingDecision::Direct(mind_id) => {
                format!("ROUTE → {} (direct match)\nInput should be delegated to mind: {}", mind_id, mind_id)
            }
            RoutingDecision::Escalate => {
                format!("ESCALATE → Primary\nThis input requires Primary's attention. Handle it directly or delegate based on content.")
            }
            RoutingDecision::Drop { reason } => {
                format!("DROP — {}\nThis input can be safely ignored.", reason)
            }
        };

        Some(ToolResult::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hub_general_routes_to_comms() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "hub",
            "source_id": "general",
            "content": "New message in general"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("comms-lead"));
    }

    #[tokio::test]
    async fn hub_protocol_routes_to_research() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "hub",
            "source_id": "protocol",
            "content": "New protocol discussion"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("research-lead"));
    }

    #[tokio::test]
    async fn boop_routes_to_ops() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "boop",
            "source_id": "grounding",
            "content": "Grounding timer fired"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ops-lead"));
    }

    #[tokio::test]
    async fn human_input_escalates() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "human",
            "source_id": "Corey",
            "content": "Hey, check this out"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ESCALATE"));
        assert!(result.output.contains("Primary"));
    }

    #[tokio::test]
    async fn critical_priority_escalates() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "hub",
            "source_id": "general",
            "content": "URGENT: system down",
            "priority": "critical"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ESCALATE"));
    }

    #[tokio::test]
    async fn unknown_source_escalates() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "ipc",
            "source_id": "unknown-agent",
            "content": "some message"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("ESCALATE"));
    }

    #[tokio::test]
    async fn passthrough_other_tools() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("other_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn missing_source_type_errors() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "content": "test"
        })).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("source_type"));
    }

    #[tokio::test]
    async fn invalid_source_type_errors() {
        let interceptor = InputRouteInterceptor::new("primary");
        let result = interceptor.handle("input_route", &serde_json::json!({
            "source_type": "carrier_pigeon",
            "content": "test"
        })).await.unwrap();
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("Unknown source_type"));
    }
}
