//! # ProcessBridge — Runtime Layer for Multi-Mind Process Management
//!
//! Maps MindIds to live child processes connected via MCP over stdio.
//! The MindManager tracks logical state; ProcessBridge manages the real processes.
//!
//! ## Architecture
//!
//! ```text
//! MindManager (logical state)
//! └── ProcessBridge (runtime)
//!     ├── MindId("research-lead") → ChildMind { process, mcp_client }
//!     ├── MindId("code-lead")     → ChildMind { process, mcp_client }
//!     └── MindId("agent-abc123")  → ChildMind { process, mcp_client }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use codex_drive::{TaskStore, StoredTask, TaskState, TaskPriority};
use codex_ipc::{McpMindClient, StdioTransport};
use codex_ipc::client::IpcError;
use codex_roles::Role;
use tokio::process::Child;
use tokio::sync::watch;
use tracing::{info, warn, error};

use crate::task_ledger::TaskLedger;
use crate::types::MindId;

/// A live child mind process with its MCP client connection.
pub struct ChildMind {
    /// The tokio child process handle.
    process: Child,
    /// MCP client connected to the child's stdin/stdout.
    client: McpMindClient<StdioTransport>,
    /// The role this child was spawned with.
    role: Role,
    /// Whether the MCP handshake has completed.
    initialized: bool,
    /// Whether the child was spawned with `--think` (ThinkLoop enabled).
    thinking: bool,
}

/// Runtime bridge between MindManager (logical) and actual OS processes.
pub struct ProcessBridge {
    /// Path to the cortex binary.
    cortex_exe: PathBuf,
    /// Active child mind processes.
    children: HashMap<MindId, ChildMind>,
    /// Optional persistent task ledger (append-only JSONL audit trail).
    ledger: Option<TaskLedger>,
    /// Optional SQLite-backed task store (queryable state, dependency tracking).
    task_store: Option<TaskStore>,
    /// Optional completion sender — notifies DriveLoop when tasks complete.
    completion_tx: Option<Arc<watch::Sender<Option<String>>>>,
}

/// Errors from process bridge operations.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Failed to spawn process: {0}")]
    Spawn(String),

    #[error("IPC error: {0}")]
    Ipc(#[from] IpcError),

    #[error("Mind not found in bridge: {0}")]
    NotFound(MindId),

    #[error("Mind already exists in bridge: {0}")]
    AlreadyExists(MindId),

    #[error("Process error: {0}")]
    Process(String),
}

impl ProcessBridge {
    /// Create a new ProcessBridge.
    ///
    /// `cortex_exe` is the path to the cortex binary that will be spawned
    /// with `--serve` for each child mind.
    pub fn new(cortex_exe: PathBuf) -> Self {
        Self {
            cortex_exe,
            children: HashMap::new(),
            ledger: None,
            task_store: None,
            completion_tx: None,
        }
    }

    /// Attach a persistent task ledger. Delegations and results will be logged.
    pub fn with_ledger(mut self, ledger: TaskLedger) -> Self {
        self.ledger = Some(ledger);
        self
    }

    /// Attach a SQLite-backed task store for queryable state and dependency tracking.
    pub fn with_task_store(mut self, store: TaskStore) -> Self {
        self.task_store = Some(store);
        self
    }

    /// Attach a completion sender — notifies DriveLoop when delegated tasks complete.
    pub fn with_completion_sender(mut self, tx: Arc<watch::Sender<Option<String>>>) -> Self {
        self.completion_tx = Some(tx);
        self
    }

    /// Spawn a new child mind process and establish MCP connection.
    ///
    /// Launches `cortex --serve --mind-id <id> --role <role>` as a child process,
    /// connects via StdioTransport, and performs the MCP initialize handshake.
    pub async fn spawn(
        &mut self,
        mind_id: &MindId,
        role: Role,
    ) -> Result<(), BridgeError> {
        if self.children.contains_key(mind_id) {
            return Err(BridgeError::AlreadyExists(mind_id.clone()));
        }

        let role_str = match role {
            Role::Primary => "primary",
            Role::TeamLead => "team-lead",
            Role::Agent => "agent",
        };

        info!(
            mind_id = %mind_id,
            role = role_str,
            exe = %self.cortex_exe.display(),
            "Spawning child mind process"
        );

        let mut child = tokio::process::Command::new(&self.cortex_exe)
            .arg("--serve")
            .arg("--mind-id")
            .arg(mind_id.as_str())
            .arg("--role")
            .arg(role_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Child logs go to parent's stderr
            .spawn()
            .map_err(|e| BridgeError::Spawn(e.to_string()))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| BridgeError::Spawn("No stdout from child".into()))?;
        let stdin = child.stdin.take()
            .ok_or_else(|| BridgeError::Spawn("No stdin to child".into()))?;

        let transport = StdioTransport::new(stdout, stdin);
        let mut client = McpMindClient::new(mind_id.as_str(), transport);

        // Perform MCP initialize handshake
        let init_result = client.initialize().await?;
        info!(
            mind_id = %mind_id,
            server_name = %init_result.server_info.name,
            protocol = %init_result.protocol_version,
            "Child mind MCP handshake complete"
        );

        self.children.insert(mind_id.clone(), ChildMind {
            process: child,
            client,
            role,
            initialized: true,
            thinking: false,
        });

        Ok(())
    }

    /// Spawn a new child mind process with ThinkLoop enabled.
    ///
    /// Same as `spawn()` but adds the `--think` flag, which tells the child
    /// to use ThinkLoop for processing delegated tasks. The delegate response
    /// will include the ThinkLoop's output in `DelegateResult.response`.
    pub async fn spawn_thinking(
        &mut self,
        mind_id: &MindId,
        role: Role,
    ) -> Result<(), BridgeError> {
        if self.children.contains_key(mind_id) {
            return Err(BridgeError::AlreadyExists(mind_id.clone()));
        }

        let role_str = match role {
            Role::Primary => "primary",
            Role::TeamLead => "team-lead",
            Role::Agent => "agent",
        };

        info!(
            mind_id = %mind_id,
            role = role_str,
            exe = %self.cortex_exe.display(),
            "Spawning THINKING child mind process"
        );

        let mut child = tokio::process::Command::new(&self.cortex_exe)
            .arg("--serve")
            .arg("--think")
            .arg("--mind-id")
            .arg(mind_id.as_str())
            .arg("--role")
            .arg(role_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| BridgeError::Spawn(e.to_string()))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| BridgeError::Spawn("No stdout from child".into()))?;
        let stdin = child.stdin.take()
            .ok_or_else(|| BridgeError::Spawn("No stdin to child".into()))?;

        let transport = StdioTransport::new(stdout, stdin);
        let mut client = McpMindClient::new(mind_id.as_str(), transport);

        let init_result = client.initialize().await?;
        info!(
            mind_id = %mind_id,
            server_name = %init_result.server_info.name,
            protocol = %init_result.protocol_version,
            "Thinking child mind MCP handshake complete"
        );

        self.children.insert(mind_id.clone(), ChildMind {
            process: child,
            client,
            role,
            initialized: true,
            thinking: true,
        });

        Ok(())
    }

    /// Delegate a task to a child mind via MCP.
    pub async fn delegate(
        &mut self,
        mind_id: &MindId,
        task_id: &str,
        description: &str,
        context: Option<&str>,
        parent_mind_id: &str,
    ) -> Result<codex_ipc::DelegateResult, BridgeError> {
        // Record delegation in ledger (JSONL audit trail)
        if let Some(ref ledger) = self.ledger {
            ledger.record_delegation(task_id, &mind_id.0, parent_mind_id, description);
        }

        // Record delegation in TaskStore (SQLite state)
        if let Some(ref store) = self.task_store {
            let task = StoredTask::new(
                task_id,
                description,
                TaskPriority::Normal,
                Some(parent_mind_id),
            );
            if let Err(e) = store.insert(&task).await {
                warn!(task_id, error = %e, "Failed to insert task into TaskStore");
            } else if let Err(e) = store.assign(task_id, &mind_id.0).await {
                warn!(task_id, error = %e, "Failed to assign task in TaskStore");
            }
        }

        let child = self.children.get_mut(mind_id)
            .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;

        let result = match child.client
            .delegate(task_id, description, context, parent_mind_id)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // On transport error (broken pipe / child crash), try to respawn once
                if matches!(e, codex_ipc::client::IpcError::Transport(_)) {
                    warn!(
                        mind_id = %mind_id,
                        error = %e,
                        "Child mind crashed during delegation — attempting respawn"
                    );
                    let role = child.role;
                    let thinking = child.thinking;

                    // Remove the dead child (kill if still running)
                    if let Some(mut dead) = self.children.remove(mind_id) {
                        let _ = dead.process.kill().await;
                    }

                    // Respawn
                    if thinking {
                        self.spawn_thinking(mind_id, role).await?;
                    } else {
                        self.spawn(mind_id, role).await?;
                    }
                    info!(mind_id = %mind_id, "Respawned child mind — retrying delegation");

                    // Retry once
                    let child = self.children.get_mut(mind_id)
                        .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;
                    child.client
                        .delegate(task_id, description, context, parent_mind_id)
                        .await?
                } else {
                    return Err(e.into());
                }
            }
        };

        info!(
            mind_id = %mind_id,
            task_id = task_id,
            accepted = result.accepted,
            "Task delegated via MCP"
        );

        // Record completion in ledger (JSONL audit trail)
        if let Some(ref ledger) = self.ledger {
            let succeeded = result.completed.unwrap_or(result.accepted);
            let summary = result.response.as_deref();
            ledger.record_completion(
                task_id,
                &mind_id.0,
                parent_mind_id,
                description,
                succeeded,
                result.iterations,
                result.tool_calls_made,
                summary,
            );
        }

        // Record completion in TaskStore (SQLite state + dependency unblocking)
        if let Some(ref store) = self.task_store {
            let succeeded = result.completed.unwrap_or(result.accepted);
            let summary = result.response.as_deref().map(|s| s.to_string());
            let iterations = result.iterations.map(|v| v as i32);
            let tool_calls_count = result.tool_calls_made.map(|v| v as i32);
            if succeeded {
                if let Err(e) = store.complete(task_id, iterations, tool_calls_count, summary.as_deref()).await {
                    warn!(task_id, error = %e, "Failed to complete task in TaskStore");
                }
            } else {
                if let Err(e) = store.set_state(task_id, TaskState::Failed).await {
                    warn!(task_id, error = %e, "Failed to mark task failed in TaskStore");
                }
            }

            // Notify DriveLoop that a task completed
            if let Some(ref tx) = self.completion_tx {
                let _ = tx.send(Some(task_id.to_string()));
            }
        }

        Ok(result)
    }

    /// Get the status of a child mind via MCP.
    pub async fn status(
        &mut self,
        mind_id: &MindId,
    ) -> Result<codex_ipc::StatusResult, BridgeError> {
        let child = self.children.get_mut(mind_id)
            .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;

        let result = child.client.status().await?;
        Ok(result)
    }

    /// List tools available on a child mind via MCP.
    pub async fn list_tools(
        &mut self,
        mind_id: &MindId,
    ) -> Result<Vec<codex_ipc::McpToolDef>, BridgeError> {
        let child = self.children.get_mut(mind_id)
            .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;

        let tools = child.client.list_tools().await?;
        Ok(tools)
    }

    /// Call a tool on a child mind via MCP.
    pub async fn call_tool(
        &mut self,
        mind_id: &MindId,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<codex_ipc::ToolCallResult, BridgeError> {
        let child = self.children.get_mut(mind_id)
            .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;

        let result = child.client.call_tool(tool_name, arguments).await?;
        Ok(result)
    }

    /// Gracefully shutdown a child mind via MCP, then wait for the process to exit.
    pub async fn shutdown(
        &mut self,
        mind_id: &MindId,
    ) -> Result<(), BridgeError> {
        let mut child = self.children.remove(mind_id)
            .ok_or_else(|| BridgeError::NotFound(mind_id.clone()))?;

        // Send MCP shutdown request
        if let Err(e) = child.client.shutdown().await {
            warn!(mind_id = %mind_id, error = %e, "MCP shutdown request failed, killing process");
            let _ = child.process.kill().await;
            return Ok(());
        }

        // Wait for the child process to exit
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            child.process.wait(),
        ).await {
            Ok(Ok(status)) => {
                info!(
                    mind_id = %mind_id,
                    exit_code = ?status.code(),
                    "Child mind process exited"
                );
            }
            Ok(Err(e)) => {
                error!(mind_id = %mind_id, error = %e, "Error waiting for child exit");
            }
            Err(_) => {
                warn!(mind_id = %mind_id, "Child process didn't exit in 10s, killing");
                let _ = child.process.kill().await;
            }
        }

        Ok(())
    }

    /// Shutdown all child minds gracefully.
    pub async fn shutdown_all(&mut self) {
        let ids: Vec<MindId> = self.children.keys().cloned().collect();
        for id in ids {
            if let Err(e) = self.shutdown(&id).await {
                error!(mind_id = %id, error = %e, "Error shutting down child mind");
            }
        }
    }

    /// Get the number of active child processes.
    pub fn active_count(&self) -> usize {
        self.children.len()
    }

    /// Check if a mind has an active process in the bridge.
    pub fn is_active(&self, mind_id: &MindId) -> bool {
        self.children.contains_key(mind_id)
    }

    /// Get all active mind IDs.
    pub fn active_minds(&self) -> Vec<MindId> {
        self.children.keys().cloned().collect()
    }

    /// Delegate tasks to multiple children CONCURRENTLY.
    ///
    /// Temporarily extracts children from the HashMap, spawns concurrent
    /// delegation tasks, then returns children to the HashMap.
    ///
    /// This bypasses the `&mut self` constraint on `delegate()` by moving
    /// children out of shared state for the duration of concurrent work.
    ///
    /// Returns results in arbitrary order (whoever finishes first).
    /// Each result is tagged with its MindId.
    pub async fn delegate_parallel(
        &mut self,
        tasks: Vec<(MindId, String, String, Option<String>, String)>,
    ) -> Vec<(MindId, Result<codex_ipc::DelegateResult, BridgeError>)> {
        // Extract children we need (temporarily remove from HashMap)
        let mut work_items: Vec<(MindId, ChildMind, String, String, Option<String>, String)> = Vec::new();
        let mut not_found: Vec<MindId> = Vec::new();

        for (mind_id, task_id, desc, ctx, parent) in tasks {
            if let Some(child) = self.children.remove(&mind_id) {
                work_items.push((mind_id, child, task_id, desc, ctx, parent));
            } else {
                not_found.push(mind_id);
            }
        }

        // Spawn concurrent delegation tasks
        let handles: Vec<_> = work_items
            .into_iter()
            .map(|(mind_id, mut child, task_id, desc, ctx, parent)| {
                tokio::spawn(async move {
                    let result = child
                        .client
                        .delegate(&task_id, &desc, ctx.as_deref(), &parent)
                        .await
                        .map_err(BridgeError::from);
                    (mind_id, child, result)
                })
            })
            .collect();

        // Collect results and return children to the HashMap
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((mind_id, child, result)) => {
                    self.children.insert(mind_id.clone(), child);
                    results.push((mind_id, result));
                }
                Err(e) => {
                    // JoinError — task panicked (shouldn't happen)
                    error!(error = %e, "Concurrent delegation task panicked");
                    results.push((
                        MindId("unknown".into()),
                        Err(BridgeError::Process(e.to_string())),
                    ));
                }
            }
        }

        // Add not-found errors
        for mind_id in not_found {
            warn!(mind_id = %mind_id, "Mind not found for parallel delegation");
            results.push((mind_id.clone(), Err(BridgeError::NotFound(mind_id))));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_creation() {
        let bridge = ProcessBridge::new(PathBuf::from("/usr/local/bin/cortex"));
        assert_eq!(bridge.active_count(), 0);
        assert!(bridge.active_minds().is_empty());
    }

    #[test]
    fn bridge_not_found() {
        let bridge = ProcessBridge::new(PathBuf::from("/usr/local/bin/cortex"));
        let id = MindId("nonexistent".into());
        assert!(!bridge.is_active(&id));
    }
}
