//! # qwen-mind — Persistent AI Mind
//!
//! A complete Cortex mind with identity, memory, planning, and hard delegation.
//! Phase 1a: isolated mind model. Phase 1b: protocol-suite citizen.

pub mod identity;
pub mod mind;
pub mod scratchpad;
pub mod fitness;
pub mod memory;
pub mod llm;
pub mod delegation;
pub mod planning;
pub mod spawner;

pub use identity::{Manifest, Role, MemoryTier, MemoryCategory, GrowthStage};
pub use mind::Mind;
pub use scratchpad::Scratchpad;
pub use fitness::FitnessTracker;
pub use memory::MindMemory;
pub use llm::OllamaClient;
pub use delegation::{DelegationError, DelegationRules};
pub use planning::{PlanningGate, TaskComplexity};
pub use spawner::MindProcess;
