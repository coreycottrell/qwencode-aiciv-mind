//! # dream_proof — Dream Cycle with LLM Consolidation
//!
//! Runs the full 5-phase dream cycle on the persistent memory store,
//! using the LLM for intelligent synthesis.
//!
//! Run multi_turn first to populate the memory store.
//!
//! ```bash
//! cargo run --release --bin multi_turn    # creates memories
//! cargo run --release --bin dream_proof   # dreams about them
//! ```

use codex_dream::{DreamConfig, DreamEngine};
use codex_llm::ollama::{OllamaClient, ModelRouter};
use codex_memory::MemoryStore;
use tracing::info;

fn load_dotenv() {
    let mut dir = std::env::current_dir().unwrap_or_default();
    loop {
        let env_file = dir.join(".env");
        if env_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&env_file) {
                for line in contents.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();
                        if std::env::var(key).is_err() {
                            unsafe { std::env::set_var(key, value); }
                        }
                    }
                }
            }
            break;
        }
        if !dir.pop() {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    load_dotenv();

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   CORTEX DREAM PROOF — LLM-Powered Memory Consolidation║");
    println!("║   5-phase cycle: audit → consolidate → prune → synth   ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Open persistent memory
    let memory_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("data")
        .join("memory");
    let _ = std::fs::create_dir_all(&memory_dir);
    let memory_path = memory_dir.join("researcher.db");
    let memory_path_str = memory_path.display().to_string();

    if !memory_path.exists() {
        eprintln!("ERROR: No memory database at {}", memory_path_str);
        eprintln!("  Run `cargo run --release --bin multi_turn` first.");
        std::process::exit(1);
    }

    let store = MemoryStore::new(&memory_path_str).await
        .expect("Failed to open memory store");
    let count = store.count().await.unwrap_or(0);
    info!(path = %memory_path_str, memories = count, "Memory store opened");

    // Seed additional memories for a richer dream (3+ needed for synthesis)
    info!("Seeding additional memories for synthesis...");
    use codex_memory::{MemoryCategory, MemoryTier, NewMemory};

    let id1 = store.store(NewMemory {
        mind_id: "researcher".into(),
        role: "agent".into(),
        vertical: Some("research".into()),
        category: MemoryCategory::Learning,
        title: "Fractal coordination works".into(),
        content: "Primary delegates to TeamLead, TeamLead spawns Agent, results flow back. 3-level chain proven.".into(),
        evidence: vec!["live_cloud_evidence.txt".into()],
        tier: MemoryTier::LongTerm,
        session_id: None,
        task_id: None,
    }).await.expect("Failed to store memory");

    let id2 = store.store(NewMemory {
        mind_id: "researcher".into(),
        role: "agent".into(),
        vertical: Some("research".into()),
        category: MemoryCategory::Learning,
        title: "Devstral is the best tool-calling model".into(),
        content: "Gemma 3 lacks tool calling. Qwen 3 wastes budget on thinking. Devstral 24B has instant structured calls.".into(),
        evidence: vec!["model-selection-2026-04-03".into()],
        tier: MemoryTier::LongTerm,
        session_id: None,
        task_id: None,
    }).await.expect("Failed to store memory");

    let id3 = store.store(NewMemory {
        mind_id: "researcher".into(),
        role: "agent".into(),
        vertical: Some("research".into()),
        category: MemoryCategory::Decision,
        title: "Native Ollama API over OpenAI-compatible".into(),
        content: "Ollama Cloud /v1/ returns 401 with Bearer. Native /api/chat works. Tool args must be JSON objects, not strings.".into(),
        evidence: vec!["api-discovery-2026-04-03".into()],
        tier: MemoryTier::LongTerm,
        session_id: None,
        task_id: None,
    }).await.expect("Failed to store memory");

    // Create links so the dream engine finds clusters
    use codex_memory::LinkType;
    let _ = store.cite(&id2, &id1).await;
    let _ = store.cite(&id3, &id1).await;
    let _ = store.cite(&id3, &id2).await;
    let _ = store.link(&id2, &id1, LinkType::BuildsOn, 0.8).await;
    let _ = store.link(&id3, &id2, LinkType::Related, 0.7).await;

    let count_after = store.count().await.unwrap_or(0);
    info!(before = count, after = count_after, "Memory seeding complete");

    // Create LLM client for synthesis
    let router = ModelRouter::from_env();
    let llm_config = router.config_lightweight(); // Use lightweight model for dream
    info!(model = %llm_config.model, "LLM configured for dream synthesis");
    let llm = OllamaClient::new(llm_config);

    // Run dream cycle
    info!("Starting dream cycle with LLM synthesis...");
    let dream_config = DreamConfig::default();
    let engine = DreamEngine::with_llm(&store, dream_config, llm);

    match engine.run_cycle().await {
        Ok(report) => {
            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   DREAM REPORT                                          ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Audited:      {}", report.audited);
            println!("Consolidated: {}", report.consolidated);
            println!("Pruned:       {}", report.pruned);
            println!("Synthesized:  {}", report.synthesized);
            println!("Phases:       {}", report.cycle.phases.len());
            println!();

            if report.synthesized > 0 {
                println!("✓ DREAM PROOF: {} patterns synthesized with LLM!", report.synthesized);
            } else if report.consolidated > 0 {
                println!("✓ DREAM PROOF: {} memories consolidated (no synthesis needed)", report.consolidated);
            } else {
                println!("○ Dream cycle ran but no synthesis/consolidation occurred");
                println!("  (memories may need more connections for pattern detection)");
            }

            // Print findings
            for phase in &report.cycle.phases {
                for finding in &phase.findings {
                    println!("  [{:?}] {}", finding.priority, finding.description);
                }
            }

            let final_count = store.count().await.unwrap_or(0);
            println!();
            println!("Memory count: {} → {}", count_after, final_count);

            // Write evidence
            let evidence_path = "dream_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX DREAM PROOF — {}\n\
                 ==========================================\n\
                 Audited: {}\n\
                 Consolidated: {}\n\
                 Pruned: {}\n\
                 Synthesized: {}\n\
                 Memory count: {} → {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                report.audited, report.consolidated, report.pruned, report.synthesized,
                count_after, final_count,
            )).expect("Failed to write evidence");
            info!(path = evidence_path, "Evidence written");
        }
        Err(e) => {
            eprintln!("DREAM CYCLE FAILED: {e}");
        }
    }
}
