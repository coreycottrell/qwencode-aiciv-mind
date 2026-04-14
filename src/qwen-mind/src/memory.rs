//! MindMemory — wrapper around cortex-memory with sharing threshold logic.
//! Phase 1a: local-only. Phase 1b: dual-write to Hub.

use cortex_memory::{MemoryStore, MemoryQuery, NewMemory as CortexNewMemory, MemoryNode, GraphEdge, GraphQuery};
use crate::identity::MemoryTier;

pub struct MindMemory {
    pub store: MemoryStore,
    sharing_threshold: f64,
}

impl MindMemory {
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let store = MemoryStore::new(db_path).await?;
        Ok(Self {
            store,
            sharing_threshold: 0.5,
        })
    }

    /// Store a memory and optionally link it to a parent.
    pub async fn store(
        &self,
        mem: CortexNewMemory,
    ) -> anyhow::Result<String> {
        let id = self.store.store(mem).await?;
        Ok(id)
    }

    /// Get a memory by ID.
    pub async fn get(&self, id: &str) -> anyhow::Result<MemoryNode> {
        Ok(self.store.get(id).await?)
    }

    /// Search memories.
    pub async fn search(&self, query: &MemoryQuery) -> anyhow::Result<Vec<cortex_memory::SearchResult>> {
        Ok(self.store.search(query).await?)
    }

    /// Create a graph edge.
    pub async fn edge(
        &self,
        source: &str,
        target: &str,
        link_type: cortex_memory::LinkType,
        weight: f64,
    ) -> anyhow::Result<String> {
        Ok(self.store.edge(source, target, link_type, weight).await?)
    }

    /// Traverse the graph.
    pub async fn traverse(&self, query: &GraphQuery) -> anyhow::Result<cortex_memory::TraversalResult> {
        Ok(self.store.traverse(query).await?)
    }

    /// Get edges for a memory.
    pub async fn get_edges(&self, memory_id: &str) -> anyhow::Result<Vec<GraphEdge>> {
        Ok(self.store.get_edges(memory_id).await?)
    }

    /// Promote a memory to the next tier.
    pub async fn promote(&self, id: &str) -> anyhow::Result<MemoryTier> {
        let tier = self.store.promote(id).await?;
        // Map cortex-memory tier to our tier
        match tier {
            cortex_memory::MemoryTier::Working => Ok(MemoryTier::Working),
            cortex_memory::MemoryTier::Validated => Ok(MemoryTier::Validated),
            cortex_memory::MemoryTier::Archived => Ok(MemoryTier::Archived),
        }
    }

    /// Archive a memory.
    pub async fn archive(&self, id: &str) -> anyhow::Result<()> {
        Ok(self.store.archive(id).await?)
    }

    /// Get archive candidates.
    pub async fn archive_candidates(&self, max_depth: f64, limit: i64) -> anyhow::Result<Vec<MemoryNode>> {
        Ok(self.store.archive_candidates(max_depth, limit).await?)
    }

    /// Count total memories.
    pub async fn count(&self) -> anyhow::Result<i64> {
        Ok(self.store.count().await?)
    }

    // Phase 1b: dual-write to Hub Knowledge:Items
    // pub async fn persist_with_hub(&self, mem: CortexNewMemory, share_scope: ShareScope) -> anyhow::Result<String> {
    //     let id = self.store.store(mem.clone()).await?;
    //     if mem.depth_score >= self.sharing_threshold || share_scope == ShareScope::Civ {
    //         // TODO: publish to Hub Knowledge:Item
    //     }
    //     Ok(id)
    // }
}
