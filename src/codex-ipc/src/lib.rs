//! # codex-ipc — MCP-Based Inter-Mind Communication
//!
//! Each Cortex mind runs as an MCP server (JSON-RPC 2.0 over stdio).
//! Other minds connect as MCP clients to invoke tools and exchange messages.
//!
//! ## Architecture
//!
//! ```text
//! Primary Mind (MCP Client)
//! ├── connects to → Research Lead (MCP Server process)
//! │   └── connects to → Researcher Agent (MCP Server process)
//! └── connects to → Code Lead (MCP Server process)
//!     └── connects to → Coder Agent (MCP Server process)
//! ```
//!
//! ## Transport Layer
//!
//! All IPC is generic over `MindTransport`:
//! - `ChannelTransport` — in-process tokio channels (testing)
//! - `StdioTransport` — client-side, connects to child process stdout/stdin
//! - `StdioServerTransport` — server-side, reads own stdin/stdout (`--serve` mode)

pub mod protocol;
pub mod server;
pub mod client;
pub mod transport;

pub use protocol::*;
pub use server::{McpMindServer, DelegateHandler};
pub use client::McpMindClient;
pub use transport::{
    MindTransport, StdioTransport, StdioServerTransport, ChannelTransport,
    transport_send, transport_recv,
};
