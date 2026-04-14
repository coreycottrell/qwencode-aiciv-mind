//! # codex-coordination — The Fractal Coordination Engine
//!
//! This is the heart of aiciv-mind-cubed. It manages the hierarchy of minds:
//! Primary → Team Leads → Agents, with the InputMux routing inputs to the
//! correct mind's context window.
//!
//! ## Architecture
//!
//! ```text
//! CoordinatorLoop (Primary)
//! ├── InputMux (routes all inputs before they reach any mind)
//! ├── MindManager (tracks all active minds)
//! ├── PlanningGate (scales planning depth with task complexity)
//! └── SpawnTriggers (automatic mind spawning on patterns)
//! ```

pub mod mind_manager;
pub mod coordinator;
pub mod input_mux;
pub mod planning;
pub mod process_bridge;
pub mod task_ledger;
pub mod triggers;
pub mod types;

pub use mind_manager::MindManager;
pub use coordinator::CoordinatorLoop;
pub use input_mux::InputMux;
pub use planning::{PlanningGate, TaskComplexity};
pub use process_bridge::ProcessBridge;
pub use task_ledger::TaskLedger;
pub use types::*;

// Re-export codex-drive types for coordination-level task management
pub use codex_drive::{TaskStore, StoredTask, TaskState, TaskPriority};
