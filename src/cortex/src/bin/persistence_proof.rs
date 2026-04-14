//! # persistence_proof — Memory Persistence Verification
//!
//! Spawns an agent that searches for memories written in a prior session.
//! Run multi_turn first (creates `data/memory/researcher.db`), then this.
//!
//! ```bash
//! cargo run --release --bin multi_turn       # creates memories
//! cargo run --release --bin persistence_proof # verifies they persist
//! ```

use codex_coordination::ProcessBridge;
use codex_coordination::types::MindId;
use codex_roles::Role;
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
    println!("║   CORTEX PERSISTENCE PROOF — Memory Survives Restart    ║");
    println!("║   Researcher agent recalls memories from prior session  ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Check for prior memory DB
    let memory_db = std::env::current_dir()
        .unwrap_or_default()
        .join("data/memory/researcher.db");
    if !memory_db.exists() {
        eprintln!("ERROR: No prior memory database found at {}", memory_db.display());
        eprintln!("  Run `cargo run --release --bin multi_turn` first to create memories.");
        std::process::exit(1);
    }
    let db_size = std::fs::metadata(&memory_db).map(|m| m.len()).unwrap_or(0);
    info!(path = %memory_db.display(), size_bytes = db_size, "Prior memory database found");

    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        std::process::exit(1);
    }

    let cortex_exe = std::env::current_exe()
        .expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        std::process::exit(1);
    }

    // ═══════ Spawn researcher agent (same mind_id → same memory DB) ═══════

    info!("Spawning researcher agent (same mind_id as multi_turn → loads prior memories)");

    let mut bridge = ProcessBridge::new(cortex_main);
    let agent_id = MindId("researcher".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("researcher spawned with persistent memory"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ Ask it to recall prior memories ═══════

    info!("Delegating memory recall task");

    let task = "\
You have a memory store that may contain memories from prior sessions. \
Your task is to search for ANY memories that exist.

STEP 1: Use `memory_search` with query 'MISSION' to find any prior memories.

STEP 2: Use `memory_search` with query 'Summary' to find any stored summaries.

STEP 3: Report what you found. If you found memories from a prior session, \
list their titles and content. If the memory store is empty, say so.

This test verifies that memories persist across process restarts.";

    match bridge.delegate(
        &agent_id,
        "task-persist-001",
        task,
        Some("This is a memory persistence verification test."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   PERSISTENCE RESULT                                    ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            // Check if memories were found
            let found_prior = response.contains("MISSION") && response.contains("Summary");
            if found_prior {
                println!("✓ PERSISTENCE PROOF: Agent recalled memories from prior session!");
            } else {
                println!("✗ NO PRIOR MEMORIES: Agent did not find memories from prior session.");
            }
            println!();
            println!("--- Agent Response ---");
            println!("{response}");
            println!("--- End Response ---");

            // Write evidence
            let evidence_path = "persistence_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX PERSISTENCE PROOF — {}\n\
                 ==========================================\n\
                 Prior DB: {} ({} bytes)\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Found prior memories: {found_prior}\n\
                 \n\
                 Response:\n{response}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                memory_db.display(),
                db_size,
            )).expect("Failed to write evidence");
            info!(path = evidence_path, "Evidence written");
        }
        Err(e) => {
            eprintln!("DELEGATION FAILED: {e}");
        }
    }

    // Shutdown
    bridge.shutdown_all().await;
    info!("All minds shut down. Persistence test complete.");
}
