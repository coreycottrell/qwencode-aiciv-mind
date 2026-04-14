//! Tool registry and executor — the brain-hand interface.
//!
//! Tools are registered with JSON schema definitions (for the LLM) and
//! async handler functions (for execution). The executor enforces role
//! policies before every invocation.

use async_trait::async_trait;
use codex_roles::{ExecPolicyLevel, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::sandbox::SandboxEnforcer;

/// A tool the LLM can invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (must match the function name the LLM calls).
    pub name: String,
    /// Human-readable description shown to the LLM.
    pub description: String,
    /// JSON Schema for the tool's parameters.
    pub parameters: serde_json::Value,
    /// Whether this tool mutates state (write, bash) vs reads (read, grep).
    pub mutates: bool,
}

/// A tool call from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Which tool to invoke.
    pub name: String,
    /// Parameters as JSON.
    pub arguments: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool succeeded.
    pub success: bool,
    /// Output content (file contents, command output, search results, etc).
    pub output: String,
    /// Optional error message.
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        let error = error.into();
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}

/// Errors from the execution layer.
#[derive(Debug, Error)]
pub enum ExecError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool '{tool}' denied for role {role}: {reason}")]
    PolicyDenied {
        tool: String,
        role: Role,
        reason: String,
    },

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

/// Trait that all tool implementations must satisfy.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with the given arguments.
    async fn execute(&self, args: serde_json::Value) -> ToolResult;

    /// Get the tool's definition (for LLM tool list).
    fn definition(&self) -> ToolDefinition;
}

/// The tool registry — maps tool names to handlers.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool handler.
    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) {
        let def = handler.definition();
        self.tools.insert(def.name.clone(), handler);
    }

    /// Get all tool definitions (for sending to the LLM).
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|h| h.definition()).collect()
    }

    /// Get tool definitions filtered by role.
    pub fn definitions_for_role(&self, role: Role) -> Vec<ToolDefinition> {
        self.definitions()
            .into_iter()
            .filter(|d| codex_roles::is_tool_allowed(role, &d.name))
            .collect()
    }

    /// Get a tool handler by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn ToolHandler>> {
        self.tools.get(name)
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Tool names.
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The executor — enforces policy then runs tools.
pub struct ToolExecutor {
    registry: ToolRegistry,
    sandbox: SandboxEnforcer,
}

impl ToolExecutor {
    pub fn new(registry: ToolRegistry, sandbox: SandboxEnforcer) -> Self {
        Self { registry, sandbox }
    }

    /// Execute a tool call with full policy enforcement.
    ///
    /// Pipeline: role filter → exec policy → sandbox check → execute
    pub async fn execute(
        &self,
        call: &ToolCall,
        role: Role,
    ) -> Result<ToolResult, ExecError> {
        // Step 1: Check if tool exists
        let handler = self
            .registry
            .get(&call.name)
            .ok_or_else(|| ExecError::ToolNotFound(call.name.clone()))?;

        // Step 2: Role-based tool filtering
        if !codex_roles::is_tool_allowed(role, &call.name) {
            return Err(ExecError::PolicyDenied {
                tool: call.name.clone(),
                role,
                reason: format!("Tool '{}' not in allowed set for {:?}", call.name, role),
            });
        }

        // Step 3: Exec policy check
        let policy = codex_roles::exec_policy_for_role(role);
        let def = handler.definition();
        match policy {
            ExecPolicyLevel::DenyAll => {
                return Err(ExecError::PolicyDenied {
                    tool: call.name.clone(),
                    role,
                    reason: "DenyAll policy: no tool execution permitted".into(),
                });
            }
            ExecPolicyLevel::DenyExceptIpc => {
                // Only IPC tools allowed (send_message, scratchpad ops)
                let ipc_tools = [
                    "send_message",
                    "team_scratchpad_read",
                    "team_scratchpad_write",
                    "coordination_scratchpad_read",
                    "coordination_scratchpad_write",
                    "mind_spawn_agent",
                    "mind_shutdown_agent",
                    "mind_delegate",
                    "mind_status",
                    "memory_search",
                    "memory_write",
                ];
                if !ipc_tools.contains(&call.name.as_str()) {
                    return Err(ExecError::PolicyDenied {
                        tool: call.name.clone(),
                        role,
                        reason: "DenyExceptIpc: only coordination tools permitted".into(),
                    });
                }
            }
            ExecPolicyLevel::Sandboxed => {
                // Full sandbox enforcement
            }
        }

        // Step 4: Sandbox check (for mutation-capable tools)
        let sandbox_level = codex_roles::sandbox_for_role(role);
        if def.mutates {
            self.sandbox
                .check_mutation(&call.name, &call.arguments, sandbox_level)?;
        }

        // Step 5: Execute
        info!(tool = %call.name, role = %role, "Executing tool");
        let result = handler.execute(call.arguments.clone()).await;
        info!(
            tool = %call.name,
            success = result.success,
            output_len = result.output.len(),
            "Tool completed"
        );

        Ok(result)
    }

    /// Get the underlying registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    #[async_trait]
    impl ToolHandler for EchoTool {
        async fn execute(&self, args: serde_json::Value) -> ToolResult {
            ToolResult::ok(format!("echo: {}", args))
        }

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "echo".into(),
                description: "Echoes input back".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" }
                    }
                }),
                mutates: false,
            }
        }
    }

    struct BashTool;

    #[async_trait]
    impl ToolHandler for BashTool {
        async fn execute(&self, args: serde_json::Value) -> ToolResult {
            ToolResult::ok(format!("bash: {}", args))
        }

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "bash".into(),
                description: "Execute shell command".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" }
                    }
                }),
                mutates: true,
            }
        }
    }

    #[test]
    fn registry_register_and_lookup() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(EchoTool));
        reg.register(Arc::new(BashTool));

        assert_eq!(reg.len(), 2);
        assert!(reg.get("echo").is_some());
        assert!(reg.get("bash").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn definitions_for_role_filters() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(EchoTool));
        reg.register(Arc::new(BashTool));

        // Agent gets everything
        let agent_defs = reg.definitions_for_role(Role::Agent);
        assert_eq!(agent_defs.len(), 2);

        // Primary gets neither (echo and bash aren't in primary's tool list)
        let primary_defs = reg.definitions_for_role(Role::Primary);
        assert_eq!(primary_defs.len(), 0);
    }

    #[tokio::test]
    async fn executor_agent_can_run_bash() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(BashTool));
        let sandbox = SandboxEnforcer::new("/tmp/test-workspace".into());
        let exec = ToolExecutor::new(reg, sandbox);

        let call = ToolCall {
            name: "bash".into(),
            arguments: serde_json::json!({"command": "ls"}),
        };
        let result = exec.execute(&call, Role::Agent).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn executor_primary_denied_bash() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(BashTool));
        let sandbox = SandboxEnforcer::new("/tmp/test-workspace".into());
        let exec = ToolExecutor::new(reg, sandbox);

        let call = ToolCall {
            name: "bash".into(),
            arguments: serde_json::json!({"command": "rm -rf /"}),
        };
        let result = exec.execute(&call, Role::Primary).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecError::PolicyDenied { tool, role, .. } => {
                assert_eq!(tool, "bash");
                assert_eq!(role, Role::Primary);
            }
            other => panic!("Expected PolicyDenied, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn executor_tool_not_found() {
        let reg = ToolRegistry::new();
        let sandbox = SandboxEnforcer::new("/tmp/test-workspace".into());
        let exec = ToolExecutor::new(reg, sandbox);

        let call = ToolCall {
            name: "nonexistent".into(),
            arguments: serde_json::Value::Null,
        };
        let result = exec.execute(&call, Role::Agent).await;
        assert!(matches!(result, Err(ExecError::ToolNotFound(_))));
    }
}
