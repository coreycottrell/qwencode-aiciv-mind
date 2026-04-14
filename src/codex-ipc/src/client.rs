//! MCP Mind Client — connects to a child mind's MCP server.
//!
//! Used by Primary to talk to Team Leads, and by Team Leads to talk to Agents.
//! Generic over `MindTransport` — works over channels (testing) or stdio (production).

use std::sync::atomic::{AtomicI64, Ordering};
use tracing::info;

use crate::protocol::*;
use crate::transport::{MindTransport, transport_recv, transport_send};

/// An MCP client that connects to a child mind.
///
/// Generic over `T: MindTransport` — use `ChannelTransport` for testing
/// or `StdioTransport` for connecting to a spawned child process.
pub struct McpMindClient<T: MindTransport> {
    mind_id: String,
    transport: T,
    next_id: AtomicI64,
}

impl<T: MindTransport> McpMindClient<T> {
    pub fn new(mind_id: impl Into<String>, transport: T) -> Self {
        Self {
            mind_id: mind_id.into(),
            transport,
            next_id: AtomicI64::new(1),
        }
    }

    fn next_request_id(&self) -> RequestId {
        RequestId::Integer(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Perform the MCP initialize handshake.
    pub async fn initialize(&mut self) -> Result<InitializeResult, IpcError> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".into(),
            capabilities: ClientCapabilities {},
            client_info: Implementation {
                name: "cortex-client".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
        };

        let resp = self.call("initialize", Some(serde_json::to_value(params).unwrap())).await?;
        let result: InitializeResult = serde_json::from_value(resp)
            .map_err(|e| IpcError::Protocol(e.to_string()))?;

        info!(
            mind_id = %self.mind_id,
            server = %result.server_info.name,
            "Connected to mind MCP server"
        );

        Ok(result)
    }

    /// List available tools on the remote mind.
    pub async fn list_tools(&mut self) -> Result<Vec<McpToolDef>, IpcError> {
        let resp = self.call("tools/list", None).await?;
        let tools: Vec<McpToolDef> = resp
            .get("tools")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();
        Ok(tools)
    }

    /// Call a tool on the remote mind.
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallResult, IpcError> {
        let params = ToolCallParams {
            name: name.into(),
            arguments,
        };
        let resp = self.call("tools/call", Some(serde_json::to_value(params).unwrap())).await?;
        let result: ToolCallResult = serde_json::from_value(resp)
            .map_err(|e| IpcError::Protocol(e.to_string()))?;
        Ok(result)
    }

    /// Delegate a task to the remote mind.
    pub async fn delegate(
        &mut self,
        task_id: &str,
        description: &str,
        context: Option<&str>,
        parent_mind_id: &str,
    ) -> Result<DelegateResult, IpcError> {
        let params = DelegateParams {
            task_id: task_id.into(),
            description: description.into(),
            context: context.map(String::from),
            parent_mind_id: parent_mind_id.into(),
        };
        let resp = self.call("cortex/delegate", Some(serde_json::to_value(params).unwrap())).await?;
        let result: DelegateResult = serde_json::from_value(resp)
            .map_err(|e| IpcError::Protocol(e.to_string()))?;
        Ok(result)
    }

    /// Get the remote mind's status.
    pub async fn status(&mut self) -> Result<StatusResult, IpcError> {
        let resp = self.call("cortex/status", None).await?;
        let result: StatusResult = serde_json::from_value(resp)
            .map_err(|e| IpcError::Protocol(e.to_string()))?;
        Ok(result)
    }

    /// Request graceful shutdown of the remote mind.
    pub async fn shutdown(&mut self) -> Result<(), IpcError> {
        self.call("cortex/shutdown", None).await?;
        info!(mind_id = %self.mind_id, "Shutdown request sent");
        Ok(())
    }

    /// Low-level: send a JSON-RPC request and wait for response.
    async fn call(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, IpcError> {
        let id = self.next_request_id();
        let request = JsonRpcRequest::new(id.clone(), method, params);

        transport_send(&mut self.transport, &request).await
            .map_err(|e| IpcError::Transport(e.to_string()))?;

        let response: JsonRpcResponse = transport_recv(&mut self.transport).await
            .map_err(|e| IpcError::Transport(e.to_string()))?
            .ok_or_else(|| IpcError::Transport("Connection closed".into()))?;

        if let Some(error) = response.error {
            return Err(IpcError::Remote {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| IpcError::Protocol("No result in response".into()))
    }
}

/// IPC errors.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Remote error (code {code}): {message}")]
    Remote { code: i64, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::McpMindServer;
    use crate::transport::ChannelTransport;
    use codex_roles::Role;

    /// Helper: set up a client-server pair with the server running in background.
    async fn setup_pair(mind_id: &str, role: Role) -> (McpMindClient<ChannelTransport>, tokio::task::JoinHandle<()>) {
        let (client_transport, mut server_transport) = ChannelTransport::pair();
        let mind_id_owned = mind_id.to_string();

        let server_handle = tokio::spawn(async move {
            let mut server = McpMindServer::new(mind_id_owned, role);
            server.run_channel(&mut server_transport, None).await.unwrap();
        });

        let client = McpMindClient::new(mind_id, client_transport);
        (client, server_handle)
    }

    #[tokio::test]
    async fn client_initialize_and_list_tools() {
        let (mut client, server) = setup_pair("research-lead", Role::TeamLead).await;

        let init = client.initialize().await.unwrap();
        assert!(init.server_info.name.contains("research-lead"));

        let tools = client.list_tools().await.unwrap();
        assert!(!tools.is_empty());
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"mind_spawn_agent"));

        client.shutdown().await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn client_delegate_and_status() {
        let (mut client, server) = setup_pair("code-lead", Role::TeamLead).await;

        client.initialize().await.unwrap();

        let delegate = client.delegate("task-001", "Build feature X", None, "primary").await.unwrap();
        assert!(delegate.accepted);
        assert_eq!(delegate.task_id, "task-001");

        let status = client.status().await.unwrap();
        assert_eq!(status.mind_id, "code-lead");
        assert_eq!(status.current_task.as_deref(), Some("task-001"));

        client.shutdown().await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn client_full_lifecycle() {
        let (mut client, server) = setup_pair("agent-1", Role::Agent).await;

        // Initialize
        let init = client.initialize().await.unwrap();
        assert_eq!(init.protocol_version, "2024-11-05");

        // Delegate
        let delegate = client
            .delegate("t-002", "Grep for spawn points", Some("Research phase"), "research-lead")
            .await
            .unwrap();
        assert!(delegate.accepted);

        // Status check
        let status = client.status().await.unwrap();
        assert_eq!(status.current_task.as_deref(), Some("t-002"));

        // Shutdown
        client.shutdown().await.unwrap();
        server.await.unwrap();
    }
}
