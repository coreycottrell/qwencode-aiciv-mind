//! DreamEngine — the nightly cycle that turns memories into intelligence.
//!
//! Five phases:
//! 1. Audit — find low-depth memories (archive candidates)
//! 2. Consolidate — find similar memories, create graph links
//! 3. Prune — archive candidates below threshold
//! 4. Synthesize — create pattern memories from clusters of 3+ linked memories
//! 5. Report — return what happened

use chrono::Utc;
use codex_llm::ollama::{ChatMessage, OllamaClient};
use codex_memory::{
    LinkType, Memory, MemoryCategory, MemoryQuery, MemoryStore, MemoryTier, NewMemory,
};
use tracing::{debug, info, warn};

use crate::{
    DreamConfig, DreamCycle, DreamFinding, DreamPhase, FindingPriority, FindingType, PhaseStatus,
};

/// Errors from the dream engine.
#[derive(Debug, thiserror::Error)]
pub enum DreamError {
    #[error("memory store error: {0}")]
    Memory(#[from] codex_memory::MemoryError),
}

/// Summary of a completed dream cycle.
#[derive(Debug, Clone)]
pub struct DreamReport {
    pub cycle: DreamCycle,
    pub audited: usize,
    pub consolidated: usize,
    pub pruned: usize,
    pub synthesized: usize,
}

/// The dream engine — operates on a MemoryStore to consolidate, prune, and synthesize.
///
/// When an `OllamaClient` is provided, the synthesis phase uses LLM inference
/// to generate intelligent pattern descriptions instead of templates.
pub struct DreamEngine<'a> {
    store: &'a MemoryStore,
    config: DreamConfig,
    /// Optional LLM client for intelligent synthesis.
    llm: Option<OllamaClient>,
}

impl<'a> DreamEngine<'a> {
    pub fn new(store: &'a MemoryStore, config: DreamConfig) -> Self {
        Self { store, config, llm: None }
    }

    /// Create a dream engine with LLM-powered synthesis.
    pub fn with_llm(store: &'a MemoryStore, config: DreamConfig, llm: OllamaClient) -> Self {
        Self { store, config, llm: Some(llm) }
    }

    /// Run the full 5-phase dream cycle.
    pub async fn run_cycle(&self) -> Result<DreamReport, DreamError> {
        let mut cycle = crate::new_dream_cycle();
        info!(cycle_id = %cycle.id, "Dream cycle starting");

        // Phase 1: Audit
        let candidates = self.run_audit(&mut cycle.phases[0]).await?;
        let audited = candidates.len();

        // Phase 2: Consolidate
        let consolidated = self.run_consolidate(&mut cycle.phases[1], &candidates).await?;

        // Phase 3: Prune
        let pruned = self.run_prune(&mut cycle.phases[2], &candidates).await?;

        // Phase 4: Synthesize
        let synthesized = self.run_synthesize(&mut cycle.phases[3]).await?;

        // Phase 5: Report (just marks complete — the DreamReport IS the artifact)
        self.run_report(&mut cycle.phases[4], audited, consolidated, pruned, synthesized);

        cycle.completed_at = Some(Utc::now());

        info!(
            cycle_id = %cycle.id,
            audited, consolidated, pruned, synthesized,
            "Dream cycle complete"
        );

        Ok(DreamReport {
            cycle,
            audited,
            consolidated,
            pruned,
            synthesized,
        })
    }

    /// Phase 1: Audit — find low-depth memories that are candidates for archival.
    async fn run_audit(
        &self,
        phase: &mut DreamPhase,
    ) -> Result<Vec<Memory>, DreamError> {
        phase.status = PhaseStatus::Running;
        phase.started_at = Some(Utc::now());

        let candidates = self
            .store
            .archive_candidates(self.config.archive_threshold, 100)
            .await?;

        for mem in &candidates {
            phase.findings.push(DreamFinding {
                finding_type: FindingType::ArchiveCandidate {
                    depth_score: mem.depth_score,
                },
                description: format!(
                    "Memory '{}' has depth {:.2} (threshold: {:.2})",
                    mem.title, mem.depth_score, self.config.archive_threshold
                ),
                action: Some("Archive if not consolidated".into()),
                priority: FindingPriority::Low,
            });
        }

        debug!(count = candidates.len(), "Audit found archive candidates");
        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(Utc::now());
        Ok(candidates)
    }

    /// Phase 2: Consolidate — find similar memories and link them.
    ///
    /// For each candidate, search for memories with similar content.
    /// If found, create a `Related` link (they cover the same ground).
    async fn run_consolidate(
        &self,
        phase: &mut DreamPhase,
        candidates: &[Memory],
    ) -> Result<usize, DreamError> {
        phase.status = PhaseStatus::Running;
        phase.started_at = Some(Utc::now());

        let mut links_created = 0;

        for mem in candidates {
            // Extract first significant word from title for search
            let search_term = mem
                .title
                .split_whitespace()
                .find(|w| w.len() > 3)
                .unwrap_or(&mem.title);

            let results = self
                .store
                .search(&MemoryQuery {
                    text: Some(search_term.to_string()),
                    ..Default::default()
                })
                .await?;

            // Link to similar memories (skip self)
            for result in &results {
                if result.memory.id != mem.id && result.relevance > 0.3 {
                    self.store
                        .link(&mem.id, &result.memory.id, LinkType::Related, result.relevance)
                        .await?;
                    links_created += 1;
                    debug!(
                        from = %mem.id[..8], to = %result.memory.id[..8],
                        relevance = result.relevance,
                        "Consolidated: linked similar memories"
                    );
                }
            }
        }

        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(Utc::now());
        Ok(links_created)
    }

    /// Phase 3: Prune — archive memories below threshold that weren't consolidated.
    async fn run_prune(
        &self,
        phase: &mut DreamPhase,
        candidates: &[Memory],
    ) -> Result<usize, DreamError> {
        phase.status = PhaseStatus::Running;
        phase.started_at = Some(Utc::now());

        let mut pruned = 0;

        for mem in candidates {
            // Check if consolidation gave this memory new links
            let links = self.store.get_links(&mem.id).await?;

            if links.is_empty() {
                // No connections — safe to archive
                self.store.archive(&mem.id).await?;
                pruned += 1;
                debug!(id = %mem.id[..8], title = %mem.title, "Pruned: archived isolated memory");
            } else {
                debug!(
                    id = %mem.id[..8], links = links.len(),
                    "Spared: memory has {} connection(s)", links.len()
                );
            }
        }

        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(Utc::now());
        Ok(pruned)
    }

    /// Phase 4: Synthesize — create pattern memories from clusters of well-linked memories.
    ///
    /// Finds memories with 3+ links (highly connected nodes) and creates a new
    /// "pattern" memory that synthesizes them, citing all sources.
    async fn run_synthesize(
        &self,
        phase: &mut DreamPhase,
    ) -> Result<usize, DreamError> {
        phase.status = PhaseStatus::Running;
        phase.started_at = Some(Utc::now());

        // Find long-term memories with high depth (the best synthesis sources)
        let deep_memories = self
            .store
            .search(&MemoryQuery {
                tier: Some(MemoryTier::LongTerm),
                min_depth: Some(0.3),
                limit: Some(50),
                ..Default::default()
            })
            .await?;

        let mut synthesized = 0;

        for result in &deep_memories {
            let links = self.store.get_links(&result.memory.id).await?;

            if links.len() >= 3 {
                // This memory is highly connected — synthesize a pattern from it
                let pattern_title = format!("Pattern: {}", result.memory.title);

                // Collect linked memory IDs for evidence
                let evidence: Vec<String> = links
                    .iter()
                    .take(5)
                    .map(|l| {
                        if l.source_id == result.memory.id {
                            l.target_id.clone()
                        } else {
                            l.source_id.clone()
                        }
                    })
                    .collect();

                // Generate synthesis content — LLM if available, template otherwise
                let synthesis_content = if let Some(ref llm) = self.llm {
                    self.llm_synthesize(llm, &result.memory, &links).await
                } else {
                    format!(
                        "Synthesized from {} with {} connections. \
                         Original insight: {}",
                        result.memory.title,
                        links.len(),
                        result.memory.content
                    )
                };

                let pattern_id = self
                    .store
                    .store(NewMemory {
                        mind_id: "dream-engine".into(),
                        role: "system".into(),
                        vertical: result.memory.vertical.clone(),
                        category: MemoryCategory::Pattern,
                        title: pattern_title.clone(),
                        content: synthesis_content,
                        evidence: evidence.clone(),
                        tier: MemoryTier::LongTerm,
                        session_id: None,
                        task_id: None,
                    })
                    .await?;

                // Cite the source memories
                self.store.cite(&pattern_id, &result.memory.id).await?;
                for eid in &evidence {
                    // Only cite if the evidence ID looks like a memory ID (not arbitrary text)
                    if eid.contains('-') && eid.len() > 30 {
                        let _ = self.store.cite(&pattern_id, eid).await;
                    }
                }

                phase.findings.push(DreamFinding {
                    finding_type: FindingType::Pattern {
                        occurrences: links.len(),
                    },
                    description: format!(
                        "Synthesized '{}' from {} connections",
                        pattern_title,
                        links.len()
                    ),
                    action: Some(format!("Created pattern memory {}", &pattern_id[..8])),
                    priority: FindingPriority::Medium,
                });

                synthesized += 1;
                debug!(
                    pattern = %pattern_id[..8],
                    source = %result.memory.id[..8],
                    links = links.len(),
                    "Synthesized pattern memory"
                );
            }
        }

        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(Utc::now());
        Ok(synthesized)
    }

    /// Use the LLM to generate an intelligent synthesis of a memory and its connections.
    async fn llm_synthesize(
        &self,
        llm: &OllamaClient,
        source: &Memory,
        links: &[codex_memory::MemoryLink],
    ) -> String {
        let prompt = format!(
            "You are a dream consolidation engine. Synthesize the following memory and its connections \
             into a concise pattern description (2-3 sentences max).\n\n\
             ## Source Memory\n\
             Title: {}\n\
             Content: {}\n\
             Category: {}\n\
             Connections: {} links\n\n\
             Produce ONLY the synthesis text. No headers, no bullet points, no meta-commentary.",
            source.title,
            source.content,
            source.category,
            links.len(),
        );

        let messages = vec![
            ChatMessage::system("You are a memory consolidation system. Be concise and precise."),
            ChatMessage::user(prompt),
        ];

        match llm.chat(&messages, None).await {
            Ok(resp) => {
                let text = resp.choices.first()
                    .and_then(|c| c.message.content.clone())
                    .unwrap_or_default();
                if text.is_empty() {
                    // Fallback to template
                    format!(
                        "Synthesized from {} with {} connections. Original insight: {}",
                        source.title, links.len(), source.content
                    )
                } else {
                    info!(title = %source.title, len = text.len(), "LLM synthesis generated");
                    text
                }
            }
            Err(e) => {
                warn!(error = %e, "LLM synthesis failed, using template");
                format!(
                    "Synthesized from {} with {} connections. Original insight: {}",
                    source.title, links.len(), source.content
                )
            }
        }
    }

    /// Phase 5: Report — summarize findings.
    fn run_report(
        &self,
        phase: &mut DreamPhase,
        audited: usize,
        consolidated: usize,
        pruned: usize,
        synthesized: usize,
    ) {
        phase.status = PhaseStatus::Running;
        phase.started_at = Some(Utc::now());

        phase.findings.push(DreamFinding {
            finding_type: FindingType::Pattern { occurrences: 0 },
            description: format!(
                "Dream cycle complete: audited={audited}, consolidated={consolidated}, \
                 pruned={pruned}, synthesized={synthesized}"
            ),
            action: None,
            priority: FindingPriority::Low,
        });

        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhaseType;

    async fn seeded_store() -> MemoryStore {
        let store = MemoryStore::new(":memory:").await.unwrap();

        // Create a mix of memories at different depths
        let deep = store
            .store(NewMemory {
                mind_id: "primary".into(),
                role: "primary".into(),
                vertical: Some("research".into()),
                category: MemoryCategory::Learning,
                title: "Fork strategy validated".into(),
                content: "Forking Codex and injecting our coordination layer works.".into(),
                evidence: vec!["cortex-boot-001".into()],
                tier: MemoryTier::LongTerm,
                session_id: None,
                task_id: None,
            })
            .await
            .unwrap();

        let _shallow1 = store
            .store(NewMemory {
                mind_id: "agent-1".into(),
                role: "agent".into(),
                vertical: Some("code".into()),
                category: MemoryCategory::Observation,
                title: "Compile takes 10s".into(),
                content: "The workspace compiles in about 10 seconds on dev.".into(),
                evidence: vec![],
                tier: MemoryTier::Working,
                session_id: Some("session-1".into()),
                task_id: None,
            })
            .await
            .unwrap();

        let _shallow2 = store
            .store(NewMemory {
                mind_id: "agent-2".into(),
                role: "agent".into(),
                vertical: Some("code".into()),
                category: MemoryCategory::Observation,
                title: "Compile speed observation".into(),
                content: "Compilation performance is roughly 10 seconds.".into(),
                evidence: vec![],
                tier: MemoryTier::Working,
                session_id: Some("session-1".into()),
                task_id: None,
            })
            .await
            .unwrap();

        let related1 = store
            .store(NewMemory {
                mind_id: "primary".into(),
                role: "primary".into(),
                vertical: Some("research".into()),
                category: MemoryCategory::Decision,
                title: "Use Codex for orchestration".into(),
                content: "Codex has built-in multi-agent hierarchy we can leverage.".into(),
                evidence: vec!["thread_manager.rs".into()],
                tier: MemoryTier::LongTerm,
                session_id: None,
                task_id: None,
            })
            .await
            .unwrap();

        let related2 = store
            .store(NewMemory {
                mind_id: "primary".into(),
                role: "primary".into(),
                vertical: Some("research".into()),
                category: MemoryCategory::Learning,
                title: "AgentControl is the injection point".into(),
                content: "control.rs:150 is where we hook in our role filtering.".into(),
                evidence: vec!["control.rs:150".into()],
                tier: MemoryTier::LongTerm,
                session_id: None,
                task_id: None,
            })
            .await
            .unwrap();

        // Build a cluster around the deep memory: 3+ links, 3+ citations (depth >= 0.3)
        store.cite(&related1, &deep).await.unwrap(); // deep depth → 0.1
        store.cite(&related2, &deep).await.unwrap(); // deep depth → 0.2
        store.cite(&related1, &related2).await.unwrap(); // related2 depth → 0.1
        // Third citation to push deep to 0.3 (synthesis threshold)
        store.cite(&_shallow1, &deep).await.unwrap(); // deep depth → 0.3
        store
            .link(&related2, &deep, LinkType::BuildsOn, 0.9)
            .await
            .unwrap();

        // shallow1 and shallow2 have no links — they're isolated

        store
    }

    #[tokio::test]
    async fn audit_finds_shallow_candidates() {
        let store = seeded_store().await;
        let engine = DreamEngine::new(&store, DreamConfig::default());

        let mut phase = DreamPhase {
            phase: PhaseType::Review,
            status: PhaseStatus::Pending,
            findings: vec![],
            started_at: None,
            completed_at: None,
        };

        let candidates = engine.run_audit(&mut phase).await.unwrap();

        // shallow1 and shallow2 have depth 0.0 (below threshold 0.1)
        assert!(candidates.len() >= 2, "Should find at least 2 shallow memories");
        assert_eq!(phase.status, PhaseStatus::Completed);
    }

    #[tokio::test]
    async fn consolidation_creates_links() {
        let store = seeded_store().await;
        let engine = DreamEngine::new(&store, DreamConfig::default());

        let mut audit_phase = DreamPhase {
            phase: PhaseType::Review,
            status: PhaseStatus::Pending,
            findings: vec![],
            started_at: None,
            completed_at: None,
        };
        let candidates = engine.run_audit(&mut audit_phase).await.unwrap();

        let mut consolidate_phase = DreamPhase {
            phase: PhaseType::PatternSearch,
            status: PhaseStatus::Pending,
            findings: vec![],
            started_at: None,
            completed_at: None,
        };
        let links = engine
            .run_consolidate(&mut consolidate_phase, &candidates)
            .await
            .unwrap();

        // shallow1 ("Compile takes 10s") and shallow2 ("Compile speed observation")
        // should find each other via FTS5 search on "Compile"
        assert!(links > 0, "Should create at least 1 consolidation link");
    }

    #[tokio::test]
    async fn synthesis_creates_pattern_memories() {
        let store = seeded_store().await;
        let engine = DreamEngine::new(&store, DreamConfig::default());

        let mut phase = DreamPhase {
            phase: PhaseType::SelfImprovement,
            status: PhaseStatus::Pending,
            findings: vec![],
            started_at: None,
            completed_at: None,
        };

        let count_before = store.count().await.unwrap();
        let synthesized = engine.run_synthesize(&mut phase).await.unwrap();
        let count_after = store.count().await.unwrap();

        // The "deep" memory has 3+ links (2 cites + 1 builds_on) → should synthesize
        assert!(synthesized > 0, "Should synthesize at least 1 pattern");
        assert!(
            count_after > count_before,
            "New pattern memory should be created"
        );
    }

    #[tokio::test]
    async fn full_dream_cycle() {
        let store = seeded_store().await;
        let engine = DreamEngine::new(&store, DreamConfig::default());

        let report = engine.run_cycle().await.unwrap();

        assert!(report.audited >= 2);
        assert!(report.pruned > 0 || report.consolidated > 0);
        assert!(report.cycle.completed_at.is_some());
        assert_eq!(report.cycle.phases.len(), 5);

        // All phases should be completed
        for phase in &report.cycle.phases {
            assert_eq!(phase.status, PhaseStatus::Completed);
        }
    }
}
