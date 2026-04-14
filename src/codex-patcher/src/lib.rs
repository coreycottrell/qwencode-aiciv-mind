//! # codex-patcher
//!
//! Applies Cortex coordination patches to upstream OpenAI Codex CLI.
//!
//! ## Architecture
//!
//! Rather than maintaining a full fork of Codex, this crate applies targeted
//! patches that inject Cortex's coordination layer into Codex's existing
//! architecture. This gives us upstream updates + our fractal intelligence.
//!
//! ## Three Core Patches
//!
//! 1. **AgentControl Coordination Hook** — Intercepts spawn decisions to enforce
//!    the Primary → TeamLead → Agent fractal hierarchy.
//!
//! 2. **Session ThinkLoop Injection** — Wraps Codex's simple prompt→response
//!    with Cortex's tool-intercepting ThinkLoop (memory, Challenger, red team).
//!
//! 3. **Memory Dual-Write** — When Codex writes rollouts, also persist to
//!    Cortex's SQLite graph store with FTS5, depth scoring, and graph links.
//!
//! ## Patch Format
//!
//! Patches are stored as unified diffs in `patches/codex/` and applied at
//! build time. Each patch is idempotent and version-tagged.

pub mod agent_control_patch;
pub mod session_patch;
pub mod memory_dual_write;
pub mod patch_set;
pub mod sandbox_bridge;

/// A single patch with metadata for application
#[derive(Debug, Clone)]
pub struct CortexPatch {
    /// Human-readable name
    pub name: String,
    /// Semver of the patch
    pub version: String,
    /// Which upstream file this patch targets
    pub target_file: String,
    /// The unified diff content
    pub diff: String,
    /// Whether this patch is required (if it fails, build should fail)
    pub required: bool,
}

impl CortexPatch {
    pub fn new(name: &str, version: &str, target_file: &str, diff: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            target_file: target_file.to_string(),
            diff: diff.to_string(),
            required: true,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Result of applying a single patch
#[derive(Debug, Clone)]
pub struct PatchResult {
    pub patch_name: String,
    pub applied: bool,
    pub was_already_applied: bool,
    pub error: Option<String>,
}
