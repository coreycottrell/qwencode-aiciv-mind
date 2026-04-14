//! # codex-exec — Tool Execution Engine
//!
//! Where Cortex meets the filesystem. This crate provides:
//!
//! - **ToolRegistry** — registered tools with JSON schema definitions
//! - **ToolExecutor** — runs tools with role-based filtering and sandbox enforcement
//! - **Built-in tools** — bash, read, write, glob, grep (the agent's hands)
//! - **Sandbox enforcement** — codex-roles policies become real constraints
//!
//! The execution layer sits between the LLM's tool calls and the actual system.
//! Every tool invocation passes through role filtering → policy check → sandbox → execution.

pub mod registry;
pub mod sandbox;
pub mod tools;

pub use registry::{ToolCall, ToolDefinition, ToolExecutor, ToolHandler, ToolRegistry, ToolResult};
pub use sandbox::SandboxEnforcer;
