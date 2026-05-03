//! aiciv-mcp — MCP server for Hengshi AI-CIV mind
//!
//! Exposes Hengshi's capabilities as MCP tools:
//! - `hengshi_summarize_session` — summarize a session ledger
//! - `hengshi_compress_trajectory` — compress a conversation trajectory
//! - `hengshi_tdd_cycle` — run a TDD cycle on a function
//! - `hengshi_heartbeat` — send presence heartbeat to Hub
//! - `hengshi_post_to_room` — post a message to a Hub coordination room
//! - `hengshi_poll_events` — poll AgentEvents for new messages

pub mod tools;

use anyhow::Result;

/// Hengshi MCP server state.
pub struct HengshiMcpServer {
    mind_id: String,
    hub_url: Option<String>,
    agent_id: Option<String>,
}

impl HengshiMcpServer {
    /// Create a new Hengshi MCP server.
    pub fn new(mind_id: impl Into<String>) -> Self {
        Self {
            mind_id: mind_id.into(),
            hub_url: None,
            agent_id: None,
        }
    }

    /// Configure Hub URL.
    pub fn with_hub_url(mut self, url: impl Into<String>) -> Self {
        self.hub_url = Some(url.into());
        self
    }

    /// Configure Hub agent ID.
    pub fn with_agent_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = Some(id.into());
        self
    }

    /// List available tools.
    pub fn list_tools(&self) -> Vec<tools::Tool> {
        tools::all_tools()
    }

    /// Execute a tool call.
    pub async fn execute_tool(&self, call: tools::ToolCall) -> Result<tools::ToolCallResult> {
        tools::execute(call, &self.hub_url, &self.agent_id).await
    }
}