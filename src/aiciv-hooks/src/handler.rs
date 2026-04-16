//! Hook handler trait and external command implementation.
//!
//! Two kinds of handlers:
//! 1. In-process: implement `HookHandler` trait directly in Rust.
//! 2. External command: `ExternalCommandHandler` spawns a subprocess, sends
//!    JSON on stdin, reads JSON response from stdout (Codex pattern).

use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::types::{HookEvent, HookResponse};

/// Trait for hook handlers. Implement this for in-process hooks.
#[async_trait]
pub trait HookHandler: Send + Sync {
    /// Handle a hook event and return a response.
    ///
    /// For fire-and-forget events (SessionStart, Stop, etc.), return `HookResponse::Ack`.
    /// For blocking events (PreToolUse, PreDelegation), return the appropriate variant.
    async fn handle(&self, event: &HookEvent) -> Result<HookResponse>;

    /// Display name for logging.
    fn name(&self) -> &str;
}

/// Runs an external command as a hook handler.
///
/// The command receives the hook event as JSON on stdin and must write
/// a JSON response to stdout. If the command exits non-zero or times out,
/// behavior depends on `fail_open`:
/// - `fail_open = true`: treat as Ack (don't block on failure)
/// - `fail_open = false`: treat as error (propagate)
pub struct ExternalCommandHandler {
    pub command: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub fail_open: bool,
}

impl ExternalCommandHandler {
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            timeout: Duration::from_secs(5),
            fail_open: true,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_fail_open(mut self, fail_open: bool) -> Self {
        self.fail_open = fail_open;
        self
    }
}

#[async_trait]
impl HookHandler for ExternalCommandHandler {
    async fn handle(&self, event: &HookEvent) -> Result<HookResponse> {
        let payload =
            serde_json::to_vec(event).context("serializing hook event")?;

        debug!(command = %self.command, "spawning external hook");

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("spawning hook command: {}", self.command))?;

        // Write event JSON to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&payload).await.context("writing to hook stdin")?;
            // Drop stdin to signal EOF
        }

        // Wait with timeout
        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| {
                // Kill the child on timeout
                anyhow::anyhow!(
                    "hook command timed out after {:?}: {}",
                    self.timeout,
                    self.command
                )
            })?
            .with_context(|| {
                format!("waiting for hook command: {}", self.command)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = format!(
                "hook command exited {}: {} (stderr: {})",
                output.status, self.command, stderr
            );
            if self.fail_open {
                warn!("{msg} — fail_open=true, returning Ack");
                return Ok(HookResponse::Ack);
            }
            anyhow::bail!(msg);
        }

        // Empty stdout → Ack
        if output.stdout.is_empty() {
            return Ok(HookResponse::Ack);
        }

        // Parse response
        let response: HookResponse = serde_json::from_slice(&output.stdout)
            .with_context(|| {
                let raw = String::from_utf8_lossy(&output.stdout);
                format!(
                    "parsing hook response from {}: {}",
                    self.command, raw
                )
            })?;

        Ok(response)
    }

    fn name(&self) -> &str {
        &self.command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HookEvent;

    #[tokio::test]
    async fn external_handler_echo_ack() {
        // Use `true` command which exits 0 with no output → Ack
        let handler = ExternalCommandHandler::new("true".into());
        let event = HookEvent::SessionStart {
            session_id: "test".into(),
            metadata: serde_json::Value::Null,
        };
        let resp = handler.handle(&event).await.unwrap();
        assert!(matches!(resp, HookResponse::Ack));
    }

    #[tokio::test]
    async fn external_handler_returns_json() {
        // echo a valid PreToolUse response
        let handler = ExternalCommandHandler::new("echo".into()).with_args(vec![
            r#"{"type":"pre_tool_use","should_block":true,"reason":"blocked by test"}"#.into(),
        ]);
        let event = HookEvent::PreToolUse {
            session_id: "test".into(),
            tool_name: "bash".into(),
            tool_input: serde_json::json!({}),
        };
        let resp = handler.handle(&event).await.unwrap();
        assert!(resp.should_block());
        assert_eq!(resp.block_reason(), Some("blocked by test"));
    }

    #[tokio::test]
    async fn external_handler_fail_open() {
        let handler =
            ExternalCommandHandler::new("false".into()).with_fail_open(true);
        let event = HookEvent::Stop {
            session_id: "test".into(),
            reason: "done".into(),
        };
        let resp = handler.handle(&event).await.unwrap();
        assert!(matches!(resp, HookResponse::Ack));
    }

    #[tokio::test]
    async fn external_handler_fail_closed() {
        let handler =
            ExternalCommandHandler::new("false".into()).with_fail_open(false);
        let event = HookEvent::Stop {
            session_id: "test".into(),
            reason: "done".into(),
        };
        let result = handler.handle(&event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn external_handler_timeout() {
        let handler = ExternalCommandHandler::new("sleep".into())
            .with_args(vec!["60".into()])
            .with_timeout(Duration::from_millis(50));
        let event = HookEvent::SessionStart {
            session_id: "test".into(),
            metadata: serde_json::Value::Null,
        };
        let result = handler.handle(&event).await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("timed out"),
            "should mention timeout"
        );
    }
}
