//! # codex-drive — Autonomous Heartbeat for Cortex
//!
//! "Coordination answers 'who reports to whom.'
//!  Drive answers 'what should I do next.'"
//!  — Corey, 2026-04-04
//!
//! Three components:
//! 1. **TaskStore** — SQLite-backed task queue with status, priority, dependencies
//! 2. **EventBus** — Dual-channel event router with biased priority dispatch
//! 3. **DriveLoop** — Autonomous heartbeat: idle detection, task discovery, adaptive backoff

pub mod task_store;
pub mod event_bus;
pub mod drive_loop;

pub use task_store::{TaskStore, StoredTask, TaskState, TaskPriority};
pub use event_bus::EventBus;
pub use drive_loop::{DriveLoop, DriveConfig};
