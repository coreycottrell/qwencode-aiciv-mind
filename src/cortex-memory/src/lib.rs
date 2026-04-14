//! cortex-memory — SQLite-backed memory graph for Cortex minds.
//!
//! Provides graph-native memory with:
//! - Memory nodes with depth scoring and lifecycle tiers
//! - Graph edges: cites, builds_on, supersedes, conflicts
//! - FTS5 full-text search
//! - Contradiction detection
//! - Automatic archival of low-depth memories

pub mod store;
pub mod types;

pub use store::MemoryStore;
pub use types::*;
