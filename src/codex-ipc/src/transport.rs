//! Transport layer — line-delimited JSON over stdio, child process, or channels.
//!
//! Each message is a single JSON line terminated by \n.
//! This matches MCP's stdio transport specification.
//!
//! ## Transport Types
//!
//! - `ChannelTransport` — in-process tokio channels (testing)
//! - `StdioTransport` — client-side, connects to a child process (production)
//! - `StdioServerTransport` — server-side, reads own stdin/stdout (production)

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tracing::debug;

/// Errors from the transport layer.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Connection closed")]
    Closed,
}

// ── Transport Trait ──────────────────────────────────────────────────────────

/// The transport trait — send and receive raw JSON strings.
///
/// All Cortex IPC (server, client) is generic over this trait,
/// enabling the same code to work over channels (testing) or stdio (production).
#[async_trait]
pub trait MindTransport: Send {
    /// Send a raw JSON string.
    async fn send_raw(&mut self, json: String) -> Result<(), TransportError>;
    /// Receive a raw JSON string. Returns None on EOF/close.
    async fn recv_raw(&mut self) -> Result<Option<String>, TransportError>;
}

/// Send a serializable message over any MindTransport.
pub async fn transport_send<T: MindTransport, M: Serialize + Sync>(
    transport: &mut T,
    msg: &M,
) -> Result<(), TransportError> {
    let json = serde_json::to_string(msg)?;
    debug!("IPC send: {json}");
    transport.send_raw(json).await
}

/// Receive a deserializable message from any MindTransport.
pub async fn transport_recv<T: MindTransport, M: DeserializeOwned>(
    transport: &mut T,
) -> Result<Option<M>, TransportError> {
    match transport.recv_raw().await? {
        Some(json) => {
            debug!("IPC recv: {json}");
            Ok(Some(serde_json::from_str(&json)?))
        }
        None => Ok(None),
    }
}

// ── Stdio Transport (client side — connects to child process) ────────────────

/// Client-side stdio transport — reads from a child process's stdout,
/// writes to its stdin. Used by a parent mind to talk to a spawned child mind.
pub struct StdioTransport {
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,
}

impl StdioTransport {
    pub fn new(stdout: ChildStdout, stdin: ChildStdin) -> Self {
        Self {
            reader: BufReader::new(stdout),
            writer: stdin,
        }
    }
}

#[async_trait]
impl MindTransport for StdioTransport {
    async fn send_raw(&mut self, json: String) -> Result<(), TransportError> {
        let mut line = json;
        line.push('\n');
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.flush().await?;
        Ok(())
    }

    async fn recv_raw(&mut self) -> Result<Option<String>, TransportError> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return Ok(None);
        }
        Ok(Some(trimmed))
    }
}

// ── Stdio Server Transport (server side — reads own stdin/stdout) ────────────

/// Server-side stdio transport — reads from the process's own stdin,
/// writes to its own stdout. Used when cortex runs in `--serve` mode.
pub struct StdioServerTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

impl StdioServerTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }
}

impl Default for StdioServerTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MindTransport for StdioServerTransport {
    async fn send_raw(&mut self, json: String) -> Result<(), TransportError> {
        let mut line = json;
        line.push('\n');
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.flush().await?;
        Ok(())
    }

    async fn recv_raw(&mut self) -> Result<Option<String>, TransportError> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return Ok(None);
        }
        Ok(Some(trimmed))
    }
}

// ── Channel Transport (in-process, for testing) ──────────────────────────────

/// In-process transport for testing — uses tokio channels instead of stdio.
pub struct ChannelTransport {
    tx: tokio::sync::mpsc::Sender<String>,
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl ChannelTransport {
    /// Create a pair of connected transports for testing.
    pub fn pair() -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::channel(32);
        let (tx2, rx2) = tokio::sync::mpsc::channel(32);
        (
            Self { tx: tx1, rx: rx2 },
            Self { tx: tx2, rx: rx1 },
        )
    }
}

#[async_trait]
impl MindTransport for ChannelTransport {
    async fn send_raw(&mut self, json: String) -> Result<(), TransportError> {
        self.tx
            .send(json)
            .await
            .map_err(|_| TransportError::Closed)?;
        Ok(())
    }

    async fn recv_raw(&mut self) -> Result<Option<String>, TransportError> {
        match self.rx.recv().await {
            Some(json) => Ok(Some(json)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JsonRpcRequest;

    #[tokio::test]
    async fn channel_transport_roundtrip() {
        let (mut a, mut b) = ChannelTransport::pair();

        let req = JsonRpcRequest::new(1i64, "test/method", Some(serde_json::json!({"key": "val"})));
        transport_send(&mut a, &req).await.unwrap();

        let received: JsonRpcRequest = transport_recv(&mut b).await.unwrap().unwrap();
        assert_eq!(received.method, "test/method");
        assert_eq!(received.id, crate::protocol::RequestId::Integer(1));
    }

    #[tokio::test]
    async fn channel_transport_multiple_messages() {
        let (mut a, mut b) = ChannelTransport::pair();

        for i in 0..5 {
            let req = JsonRpcRequest::new(i as i64, "ping", None);
            transport_send(&mut a, &req).await.unwrap();
        }

        for i in 0..5 {
            let req: JsonRpcRequest = transport_recv(&mut b).await.unwrap().unwrap();
            assert_eq!(req.id, crate::protocol::RequestId::Integer(i));
        }
    }
}
