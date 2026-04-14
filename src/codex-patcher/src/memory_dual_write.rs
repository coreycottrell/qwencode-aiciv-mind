//! # Memory Dual-Write Patch
//!
//! Adds Cortex's SQLite graph store as a parallel persistence layer to Codex's
//! rollout JSONL files.
//!
//! ## What it does
//!
//! Codex records every conversation turn in its rollout JSONL format. This patch
//! adds a parallel write path that:
//!
//! 1. **Extracts structured memories** from rollout items (decisions, findings,
//!    patterns, errors) and writes them to Cortex's graph memory store.
//!
//! 2. **Builds memory graph links** — when a rollout item references a prior
//!    decision or finding, creates graph edges between the memories.
//!
//! 3. **Computes depth scores** — each memory gets an initial depth score based
//!    on its type (decisions score higher than raw tool outputs).
//!
//! 4. **Enables cross-session recall** — when Codex loads a session, it also
//!    loads relevant memories from Cortex's FTS5 store, providing context from
//!    all prior sessions, not just the current conversation.
//!
//! ## Injection point
//!
//! `codex-rs/core/src/rollout/mod.rs` — alongside the JSONL recorder.
//! `codex-rs/core/src/codex.rs` (Session) — alongside conversation history loading.

/// Configuration for memory dual-write
#[derive(Debug, Clone)]
pub struct MemoryDualWriteConfig {
    /// Path to the Cortex SQLite database
    pub db_path: String,
    /// Memory namespace for this mind (e.g., "primary", "team-lead/research")
    pub namespace: String,
    /// Whether to auto-extract memories from rollouts
    pub auto_extract: bool,
    /// Whether to load memories on session boot
    pub load_on_boot: bool,
    /// Max memories to load on boot (to avoid context flooding)
    pub max_boot_memories: usize,
}

impl Default for MemoryDualWriteConfig {
    fn default() -> Self {
        Self {
            db_path: "data/memory/cortex.db".to_string(),
            namespace: "primary".to_string(),
            auto_extract: true,
            load_on_boot: true,
            max_boot_memories: 20,
        }
    }
}

/// The actual patch content — a unified diff that injects memory dual-write
/// into Codex's rollout recording path.
pub fn generate_memory_dual_write_patch() -> &'static str {
    r#"--- a/codex-rs/core/src/rollout/mod.rs
+++ b/codex-rs/core/src/rollout/mod.rs
@@ -1,3 +1,5 @@
+use codex_patcher::memory_dual_write::{extract_memories_from_rollout, write_memories_to_cortex_store};
+
 use serde_json::Value;
 use std::path::Path;
 use tokio::fs::OpenOptions;
@@ -78,6 +80,28 @@ impl RolloutRecorder {
         self.writer.write_all(&bytes).await?;
         self.writer.write_all(b"\n").await?;

+        // === CORTEX MEMORY DUAL-WRITE ===
+        // Extract structured memories and write to Cortex's graph store
+        if self.cortex_dual_write_enabled {
+            let extractions = extract_memories_from_rollout(
+                &item,
+                &self.cortex_memory_namespace,
+            );
+
+            if !extractions.is_empty() {
+                if let Some(store) = &self.cortex_memory_store {
+                    match write_memories_to_cortex_store(store, &extractions).await {
+                        Ok(ids) => {
+                            tracing::info!(
+                                target: "cortex::memory",
+                                "Dual-write: {} memories extracted to {:?}",
+                                ids.len(), ids
+                            );
+                        }
+                        Err(e) => {
+                            tracing::warn!(
+                                target: "cortex::memory",
+                                "Dual-write failed: {e}"
+                            );
+                        }
+                    }
+                }
+            }
+        }
+        // === END CORTEX MEMORY DUAL-WRITE ===
+
         Ok(())
     }
 }
--- a/codex-rs/core/src/codex.rs
+++ b/codex-rs/core/src/codex.rs
@@ -234,6 +234,22 @@ impl Session {
         let rollout_recorder = RolloutRecorder::new(&rollout_path).await?;

+        // === CORTEX MEMORY BOOT ===
+        let cortex_memories = if self.cortex_memory_load_on_boot {
+            use codex_patcher::memory_dual_write::load_boot_memories;
+            match load_boot_memories(
+                &self.cortex_memory_store,
+                &self.cortex_memory_namespace,
+                self.cortex_max_boot_memories,
+            ).await {
+                Ok(memories) => Some(memories),
+                Err(e) => {
+                    tracing::warn!(target: "cortex::memory", "Boot memory load failed: {e}");
+                    None
+                }
+            }
+        } else {
+            None
+        };
+        // === END CORTEX MEMORY BOOT ===
+
         let session = Session {
             history,
             rollout_recorder,
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dual_write_config() {
        let config = MemoryDualWriteConfig::default();
        assert_eq!(config.db_path, "data/memory/cortex.db");
        assert_eq!(config.namespace, "primary");
        assert!(config.auto_extract);
        assert!(config.load_on_boot);
        assert_eq!(config.max_boot_memories, 20);
    }

    #[test]
    fn test_patch_has_content() {
        let patch = generate_memory_dual_write_patch();
        assert!(patch.contains("CORTEX MEMORY DUAL-WRITE"));
        assert!(patch.contains("CORTEX MEMORY BOOT"));
        assert!(patch.contains("--- a/"));
        assert!(patch.contains("+++ b/"));
    }
}
