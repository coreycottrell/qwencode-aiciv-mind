//! # codex-memory — Memory IS Architecture (Principle 1)
//!
//! SQLite-backed memory graph with:
//! - Depth scoring (cited memories grow deeper, uncited fade)
//! - Graph links (cites, builds_on, contradicts, supersedes)
//! - Three-tier lifecycle (working → session → long_term → archived)
//! - Full-text search via FTS5
//! - Per-mind and per-vertical scoping

pub mod store;
pub mod types;

pub use store::{MemoryError, MemoryStore};
pub use types::*;
