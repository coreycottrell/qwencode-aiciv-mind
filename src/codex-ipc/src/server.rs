//! MCP Mind Server — each Cortex mind exposes itself as an MCP server.
//!
//! The server handles incoming JSON-RPC requests from a parent mind:
//! - `initialize` — MCP handshake
//! - `tools/list` — list available tools (role-filtered)
//! - `tools/call` — execute a tool
//! - `cortex/delegate` — accept a task (optionally with ThinkLoop via DelegateHandler)
//! - `cortex/status` — report status
//! - `cortex/shutdown` — graceful shutdown
//!
//! ## DelegateHandler
//!
//! When a `DelegateHandler` is provided to the server's `run()` method, delegated
//! tasks are processed through the handler (typically a ThinkLoop) and the result
//! is returned in the `DelegateResult.response` field. Without a handler, delegate
//! just accepts the task (backward-compatible).

use async_trait::async_trait;
use codex_exec::{ToolCall, ToolExecutor, ToolResult};
use codex_roles::Role;
use tracing::{info, warn};

use crate::protocol::*;
use crate::transport::{ChannelTransport, MindTransport, transport_recv, transport_send};

/// Callback interface for processing delegated tasks.
///
/// Implement this trait to make a child mind actually THINK about delegated tasks
/// rather than just acknowledging them. The typical implementation wraps a ThinkLoop
/// with memory integration.
#[async_trait]
pub trait DelegateHandler: Send + Sync {
    /// Process a delegated task and return the result.
    ///
    /// - `task_id` — unique identifier for the task
    /// - `description` — what to do
    /// - `context` — optional additional context from the parent
    async fn process_task(
        &self,
        task_id: &str,
        description: &str,
        context: Option<&str>,
    ) -> Result<DelegateTaskResult, String>;
}

/// An MCP server representing a single Cortex mind.
pub struct McpMindServer {
    mind_id: String,
    role: Role,
    current_task: Option<String>,
    status: MindServerStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindServerStatus {
    Initializing,
    Ready,
    Working,
    ShuttingDown,
}

impl McpMindServer {
    pub fn new(mind_id: impl Into<String>, role: Role) -> Self {
        Self {
            mind_id: mind_id.into(),
            role,
            current_task: None,
            status: MindServerStatus::Initializing,
        }
    }

    /// Handle a single JSON-RPC request and produce a response.
    pub async fn handle_request(
        &mut self,
        request: &JsonRpcRequest,
        executor: Option<&ToolExecutor>,
        delegate_handler: Option<&(dyn DelegateHandler + 'static)>,
    ) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.id, &request.params),
            "tools/list" => self.handle_tools_list(&request.id, executor),
            "tools/call" => self.handle_tools_call(&request.id, &request.params, executor).await,
            "cortex/delegate" => self.handle_delegate(&request.id, &request.params, delegate_handler).await,
            "cortex/status" => self.handle_status(&request.id),
            "cortex/shutdown" => self.handle_shutdown(&request.id),
            other => {
                warn!(method = other, "Unknown method");
                JsonRpcResponse::error(
                    request.id.clone(),
                    METHOD_NOT_FOUND,
                    format!("Method not found: {other}"),
                )
            }
        }
    }

    fn handle_initialize(&mut self, id: &RequestId, _params: &Option<serde_json::Value>) -> JsonRpcResponse {
        self.status = MindServerStatus::Ready;
        info!(mind_id = %self.mind_id, "MCP server initialized");

        let result = InitializeResult {
            protocol_version: "2024-11-05".into(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {}),
            },
            server_info: Implementation {
                name: format!("cortex-{}", self.mind_id),
                version: env!("CARGO_PKG_VERSION").into(),
            },
        };

        JsonRpcResponse::success(id.clone(), serde_json::to_value(result).unwrap())
    }

    fn handle_tools_list(&self, id: &RequestId, executor: Option<&ToolExecutor>) -> JsonRpcResponse {
        let tools: Vec<McpToolDef> = if let Some(exec) = executor {
            exec.registry()
                .definitions_for_role(self.role)
                .into_iter()
                .map(|d| McpToolDef {
                    name: d.name,
                    description: d.description,
                    input_schema: d.parameters,
                })
                .collect()
        } else {
            // No executor — return Cortex coordination tools
            cortex_coordination_tools(self.role)
        };

        JsonRpcResponse::success(
            id.clone(),
            serde_json::json!({ "tools": tools }),
        )
    }

    async fn handle_tools_call(
        &mut self,
        id: &RequestId,
        params: &Option<serde_json::Value>,
        executor: Option<&ToolExecutor>,
    ) -> JsonRpcResponse {
        let Some(params) = params else {
            return JsonRpcResponse::error(id.clone(), INVALID_PARAMS, "Missing params");
        };

        let call_params: ToolCallParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id.clone(),
                    INVALID_PARAMS,
                    format!("Invalid tool call params: {e}"),
                );
            }
        };

        let Some(exec) = executor else {
            return JsonRpcResponse::error(
                id.clone(),
                INTERNAL_ERROR,
                "No tool executor available",
            );
        };

        let tool_call = ToolCall {
            name: call_params.name.clone(),
            arguments: call_params.arguments,
        };

        match exec.execute(&tool_call, self.role).await {
            Ok(result) => {
                let mcp_result = ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".into(),
                        text: result.output,
                    }],
                    is_error: if result.success { None } else { Some(true) },
                };
                JsonRpcResponse::success(id.clone(), serde_json::to_value(mcp_result).unwrap())
            }
            Err(e) => {
                let mcp_result = ToolCallResult {
                    content: vec![ToolCallContent {
                        content_type: "text".into(),
                        text: format!("Error: {e}"),
                    }],
                    is_error: Some(true),
                };
                JsonRpcResponse::success(id.clone(), serde_json::to_value(mcp_result).unwrap())
            }
        }
    }

    async fn handle_delegate(
        &mut self,
        id: &RequestId,
        params: &Option<serde_json::Value>,
        handler: Option<&(dyn DelegateHandler + 'static)>,
    ) -> JsonRpcResponse {
        let Some(params) = params else {
            return JsonRpcResponse::error(id.clone(), INVALID_PARAMS, "Missing delegate params");
        };

        let delegate: DelegateParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id.clone(),
                    INVALID_PARAMS,
                    format!("Invalid delegate params: {e}"),
                );
            }
        };

        self.current_task = Some(delegate.task_id.clone());
        self.status = MindServerStatus::Working;
        info!(
            mind_id = %self.mind_id,
            task_id = %delegate.task_id,
            "Accepted task: {}",
            delegate.description
        );

        // If a DelegateHandler is present, actually THINK about the task
        let (response, iterations, tool_calls_made, completed) = if let Some(handler) = handler {
            info!(
                mind_id = %self.mind_id,
                task_id = %delegate.task_id,
                "Processing task via DelegateHandler (ThinkLoop)"
            );

            match handler
                .process_task(
                    &delegate.task_id,
                    &delegate.description,
                    delegate.context.as_deref(),
                )
                .await
            {
                Ok(task_result) => {
                    info!(
                        mind_id = %self.mind_id,
                        task_id = %delegate.task_id,
                        iterations = task_result.iterations,
                        tool_calls = task_result.tool_calls,
                        completed = task_result.completed,
                        "Task processing complete"
                    );
                    self.status = MindServerStatus::Ready;
                    self.current_task = None;
                    (
                        Some(task_result.response),
                        Some(task_result.iterations),
                        Some(task_result.tool_calls),
                        Some(task_result.completed),
                    )
                }
                Err(e) => {
                    warn!(
                        mind_id = %self.mind_id,
                        task_id = %delegate.task_id,
                        error = %e,
                        "DelegateHandler error"
                    );
                    self.status = MindServerStatus::Ready;
                    self.current_task = None;
                    (Some(format!("Error: {e}")), None, None, Some(false))
                }
            }
        } else {
            // No handler — accept-only mode (backward compatible)
            (None, None, None, None)
        };

        let result = DelegateResult {
            accepted: true,
            mind_id: self.mind_id.clone(),
            task_id: delegate.task_id,
            response,
            iterations,
            tool_calls_made,
            completed,
        };

        JsonRpcResponse::success(id.clone(), serde_json::to_value(result).unwrap())
    }

    fn handle_status(&self, id: &RequestId) -> JsonRpcResponse {
        let result = StatusResult {
            mind_id: self.mind_id.clone(),
            role: format!("{:?}", self.role),
            status: format!("{:?}", self.status),
            current_task: self.current_task.clone(),
            children: vec![],
        };

        JsonRpcResponse::success(id.clone(), serde_json::to_value(result).unwrap())
    }

    fn handle_shutdown(&mut self, id: &RequestId) -> JsonRpcResponse {
        self.status = MindServerStatus::ShuttingDown;
        info!(mind_id = %self.mind_id, "Shutdown requested");
        JsonRpcResponse::success(id.clone(), serde_json::json!({"shutdown": true}))
    }

    /// Run the server loop over any MindTransport.
    ///
    /// Reads JSON-RPC requests, handles them, sends responses.
    /// Stops on transport close or shutdown request.
    ///
    /// When `delegate_handler` is provided, `cortex/delegate` requests will use it
    /// to actually process (think about) the task, blocking until thinking completes.
    pub async fn run<T: MindTransport>(
        &mut self,
        transport: &mut T,
        executor: Option<&ToolExecutor>,
        delegate_handler: Option<&(dyn DelegateHandler + 'static)>,
    ) -> Result<(), crate::transport::TransportError> {
        loop {
            let req: Option<JsonRpcRequest> = transport_recv(transport).await?;
            let Some(req) = req else {
                info!(mind_id = %self.mind_id, "Transport closed");
                break;
            };

            let resp = self.handle_request(&req, executor, delegate_handler).await;
            transport_send(transport, &resp).await?;

            if self.status == MindServerStatus::ShuttingDown {
                break;
            }
        }
        Ok(())
    }

    /// Convenience: run over a ChannelTransport (backward-compatible with tests).
    pub async fn run_channel(
        &mut self,
        transport: &mut ChannelTransport,
        executor: Option<&ToolExecutor>,
    ) -> Result<(), crate::transport::TransportError> {
        self.run(transport, executor, None).await
    }
}

/// Return Cortex coordination tool definitions for a role.
fn cortex_coordination_tools(role: Role) -> Vec<McpToolDef> {
    let mut tools = vec![];

    match role {
        Role::Primary => {
            tools.push(McpToolDef {
                name: "mind_spawn_team_lead".into(),
                description: "Spawn a team lead mind for a vertical".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "vertical": { "type": "string" },
                        "objective": { "type": "string" }
                    },
                    "required": ["vertical", "objective"]
                }),
            });
            tools.push(McpToolDef {
                name: "mind_delegate".into(),
                description: "Delegate a task to a team lead".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target_mind_id": { "type": "string" },
                        "task_description": { "type": "string" }
                    },
                    "required": ["target_mind_id", "task_description"]
                }),
            });
            tools.push(McpToolDef {
                name: "mind_status".into(),
                description: "Get status of a specific mind or all minds".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "mind_id": { "type": "string" }
                    }
                }),
            });
            tools.push(McpToolDef {
                name: "send_message".into(),
                description: "Send a message to another mind".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "to": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["to", "content"]
                }),
            });
        }
        Role::TeamLead => {
            tools.push(McpToolDef {
                name: "mind_spawn_agent".into(),
                description: "Spawn an agent to execute work".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "agent_type": { "type": "string" },
                        "task": { "type": "string" }
                    },
                    "required": ["agent_type", "task"]
                }),
            });
            tools.push(McpToolDef {
                name: "mind_delegate".into(),
                description: "Delegate a task to an agent".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target_mind_id": { "type": "string" },
                        "task_description": { "type": "string" }
                    },
                    "required": ["target_mind_id", "task_description"]
                }),
            });
            tools.push(McpToolDef {
                name: "send_message".into(),
                description: "Send a message to another mind".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "to": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["to", "content"]
                }),
            });
        }
        Role::Agent => {
            // Agent tools come from the ToolExecutor, not here
        }
    }

    // Common tools for all roles
    tools.push(McpToolDef {
        name: "memory_search".into(),
        description: "Search the memory graph".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "category": { "type": "string" },
                "limit": { "type": "integer" }
            },
            "required": ["query"]
        }),
    });

    tools
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{transport_recv, transport_send};

    #[tokio::test]
    async fn server_initialize() {
        let mut server = McpMindServer::new("test-mind", Role::Agent);

        let req = JsonRpcRequest::new(1i64, "initialize", Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1.0" }
        })));

        let resp = server.handle_request(&req, None, None).await;
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(server.status, MindServerStatus::Ready);
    }

    #[tokio::test]
    async fn server_delegate_task() {
        let mut server = McpMindServer::new("research-lead", Role::TeamLead);

        let req = JsonRpcRequest::new(2i64, "cortex/delegate", Some(serde_json::json!({
            "task_id": "task-001",
            "description": "Research Codex exec crate",
            "parent_mind_id": "primary"
        })));

        let resp = server.handle_request(&req, None, None).await;
        let result = resp.result.unwrap();
        assert!(result["accepted"].as_bool().unwrap());
        // No handler → stays Working (accept-only mode)
        assert_eq!(server.status, MindServerStatus::Working);
        assert_eq!(server.current_task.as_deref(), Some("task-001"));
    }

    #[tokio::test]
    async fn server_delegate_with_handler() {
        struct TestHandler;

        #[async_trait]
        impl DelegateHandler for TestHandler {
            async fn process_task(
                &self,
                _task_id: &str,
                description: &str,
                _context: Option<&str>,
            ) -> Result<DelegateTaskResult, String> {
                Ok(DelegateTaskResult {
                    response: format!("Thought about: {description}"),
                    iterations: 2,
                    tool_calls: 1,
                    completed: true,
                })
            }
        }

        let mut server = McpMindServer::new("thinking-agent", Role::Agent);
        let handler = TestHandler;

        let req = JsonRpcRequest::new(2i64, "cortex/delegate", Some(serde_json::json!({
            "task_id": "think-001",
            "description": "Analyze fractal patterns",
            "parent_mind_id": "primary"
        })));

        let resp = server.handle_request(&req, None, Some(&handler)).await;
        let result = resp.result.unwrap();
        assert!(result["accepted"].as_bool().unwrap());
        assert_eq!(result["response"].as_str().unwrap(), "Thought about: Analyze fractal patterns");
        assert_eq!(result["iterations"].as_u64().unwrap(), 2);
        assert_eq!(result["tool_calls_made"].as_u64().unwrap(), 1);
        assert!(result["completed"].as_bool().unwrap());
        // Handler completed → back to Ready
        assert_eq!(server.status, MindServerStatus::Ready);
        assert!(server.current_task.is_none());
    }

    #[tokio::test]
    async fn server_delegate_handler_error() {
        struct FailingHandler;

        #[async_trait]
        impl DelegateHandler for FailingHandler {
            async fn process_task(
                &self,
                _task_id: &str,
                _description: &str,
                _context: Option<&str>,
            ) -> Result<DelegateTaskResult, String> {
                Err("LLM offline".to_string())
            }
        }

        let mut server = McpMindServer::new("failing-agent", Role::Agent);
        let handler = FailingHandler;

        let req = JsonRpcRequest::new(3i64, "cortex/delegate", Some(serde_json::json!({
            "task_id": "fail-001",
            "description": "This will fail",
            "parent_mind_id": "primary"
        })));

        let resp = server.handle_request(&req, None, Some(&handler)).await;
        let result = resp.result.unwrap();
        assert!(result["accepted"].as_bool().unwrap());
        assert!(result["response"].as_str().unwrap().contains("Error: LLM offline"));
        // Error → still returns to Ready
        assert_eq!(server.status, MindServerStatus::Ready);
    }

    #[tokio::test]
    async fn server_status() {
        let mut server = McpMindServer::new("code-lead", Role::TeamLead);
        server.status = MindServerStatus::Ready;

        let req = JsonRpcRequest::new(3i64, "cortex/status", None);
        let resp = server.handle_request(&req, None, None).await;
        let result = resp.result.unwrap();
        assert_eq!(result["mind_id"], "code-lead");
    }

    #[tokio::test]
    async fn server_shutdown() {
        let mut server = McpMindServer::new("agent-1", Role::Agent);
        server.status = MindServerStatus::Ready;

        let req = JsonRpcRequest::new(4i64, "cortex/shutdown", None);
        let resp = server.handle_request(&req, None, None).await;
        assert!(resp.result.is_some());
        assert_eq!(server.status, MindServerStatus::ShuttingDown);
    }

    #[tokio::test]
    async fn server_unknown_method() {
        let mut server = McpMindServer::new("test", Role::Agent);
        let req = JsonRpcRequest::new(5i64, "unknown/method", None);
        let resp = server.handle_request(&req, None, None).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn server_tools_list_primary() {
        let mut server = McpMindServer::new("primary", Role::Primary);
        let req = JsonRpcRequest::new(6i64, "tools/list", None);
        let resp = server.handle_request(&req, None, None).await;
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        // Primary should have coordination tools + memory_search
        assert!(tools.len() >= 4);
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"mind_spawn_team_lead"));
        assert!(names.contains(&"mind_delegate"));
        assert!(names.contains(&"memory_search"));
    }

    #[tokio::test]
    async fn channel_server_loop() {
        let (mut client_transport, mut server_transport) = ChannelTransport::pair();
        let mut server = McpMindServer::new("test-loop", Role::Agent);

        // Run server in background
        let server_handle = tokio::spawn(async move {
            server.run_channel(&mut server_transport, None).await.unwrap();
        });

        // Client sends initialize
        transport_send(&mut client_transport, &JsonRpcRequest::new(1i64, "initialize", Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "0.1.0" }
        })))).await.unwrap();

        let resp: JsonRpcResponse = transport_recv(&mut client_transport).await.unwrap().unwrap();
        assert!(resp.result.is_some());

        // Client sends shutdown
        transport_send(&mut client_transport, &JsonRpcRequest::new(2i64, "cortex/shutdown", None)).await.unwrap();
        let resp: JsonRpcResponse = transport_recv(&mut client_transport).await.unwrap().unwrap();
        assert!(resp.result.is_some());

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn channel_server_with_delegate_handler() {
        struct EchoHandler;

        #[async_trait]
        impl DelegateHandler for EchoHandler {
            async fn process_task(
                &self,
                _task_id: &str,
                description: &str,
                context: Option<&str>,
            ) -> Result<DelegateTaskResult, String> {
                let ctx = context.unwrap_or("none");
                Ok(DelegateTaskResult {
                    response: format!("Processed [{description}] with context [{ctx}]"),
                    iterations: 1,
                    tool_calls: 0,
                    completed: true,
                })
            }
        }

        let (mut client_transport, mut server_transport) = ChannelTransport::pair();
        let mut server = McpMindServer::new("thinking-loop", Role::Agent);
        let handler: Box<dyn DelegateHandler> = Box::new(EchoHandler);

        let server_handle = tokio::spawn(async move {
            server.run(&mut server_transport, None, Some(handler.as_ref())).await.unwrap();
        });

        // Initialize
        transport_send(&mut client_transport, &JsonRpcRequest::new(1i64, "initialize", Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1.0" }
        })))).await.unwrap();
        let _: JsonRpcResponse = transport_recv(&mut client_transport).await.unwrap().unwrap();

        // Delegate with handler — should get thinking result back
        transport_send(&mut client_transport, &JsonRpcRequest::new(2i64, "cortex/delegate", Some(serde_json::json!({
            "task_id": "think-via-loop",
            "description": "Analyze patterns",
            "context": "Phase 5 testing",
            "parent_mind_id": "primary"
        })))).await.unwrap();

        let resp: JsonRpcResponse = transport_recv(&mut client_transport).await.unwrap().unwrap();
        let result = resp.result.unwrap();
        assert!(result["accepted"].as_bool().unwrap());
        assert_eq!(
            result["response"].as_str().unwrap(),
            "Processed [Analyze patterns] with context [Phase 5 testing]"
        );
        assert!(result["completed"].as_bool().unwrap());

        // Shutdown
        transport_send(&mut client_transport, &JsonRpcRequest::new(3i64, "cortex/shutdown", None)).await.unwrap();
        let _: JsonRpcResponse = transport_recv(&mut client_transport).await.unwrap().unwrap();

        server_handle.await.unwrap();
    }
}
