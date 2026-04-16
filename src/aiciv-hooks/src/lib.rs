//! # aiciv-hooks — Runtime Lifecycle Hook Engine
//!
//! Event dispatch system for intercepting and modifying behavior at key points
//! in the aiciv-mind lifecycle. Inspired by Codex's 5,553-line hooks module,
//! extended with aiciv-mind-specific events.
//!
//! ## Architecture
//!
//! - **Dependency inversion**: hooks depends ONLY on codex-types. Consumers
//!   (codex-exec, codex-drive, cortex) call INTO hooks, not the other way around.
//! - **HookHandler trait**: implement for in-process hooks or use
//!   ExternalCommandHandler for shell-command hooks.
//! - **HookDispatcher**: central bus — registers handlers, fires events,
//!   collects responses.
//!
//! ## Hook Event Types
//!
//! 5 from Codex + 4 aiciv-mind extensions:
//! - SessionStart, PreToolUse, PostToolUse, Stop, UserPromptSubmit
//! - PreDelegation, PostDelegation, MemoryWrite, DriveEvent

pub mod config;
pub mod dispatcher;
pub mod handler;
pub mod types;

pub use config::HookConfig;
pub use dispatcher::{Decision, HookDispatcher};
pub use handler::{ExternalCommandHandler, HookHandler};
pub use types::{HookEvent, HookEventType, HookResponse};
