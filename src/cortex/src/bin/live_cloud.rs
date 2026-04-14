//! # live_cloud — Real Ollama Cloud Integration Test
//!
//! Exercises the full 3-level delegation chain with REAL LLM inference:
//!
//! ```text
//! Primary (this process)
//! └── cortex --serve --think --mind-id research-lead --role team-lead
//!     └── cortex --serve --think --mind-id researcher --role agent
//! ```
//!
//! Primary delegates to TeamLead via ProcessBridge.
//! TeamLead THINKS (real LLM call via Ollama Cloud), decides to spawn+delegate to Agent.
//! Agent THINKS (real LLM call), executes tools, returns result.
//! Result flows back up the chain.
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! cargo build --release
//! cargo run --release --bin live_cloud
//! ```

use codex_coordination::ProcessBridge;
use codex_coordination::types::MindId;
use codex_roles::Role;
use tracing::info;

/// Load KEY=VALUE pairs from a .env file into the process environment.
fn load_dotenv() {
    // Walk up from cwd to find .env
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
                        // Only set if not already in env (env takes precedence)
                        if std::env::var(key).is_err() {
                            // SAFETY: called before any threads are spawned (main thread only, before tokio runtime)
                            unsafe { std::env::set_var(key, value); }
                        }
                    }
                }
                eprintln!("[live_cloud] Loaded .env from {}", env_file.display());
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
    // Load .env before anything else
    load_dotenv();

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   CORTEX LIVE CLOUD TEST — Real Ollama Inference        ║");
    println!("║   3-Level: Primary → TeamLead (thinks) → Agent (thinks) ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Check for OLLAMA_API_KEY
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        eprintln!("  export OLLAMA_API_KEY=\"your-key\"");
        eprintln!("  cargo run --release --bin live_cloud");
        std::process::exit(1);
    }
    info!("OLLAMA_API_KEY found ({}...)", &api_key.as_ref().unwrap()[..8.min(api_key.as_ref().unwrap().len())]);

    // Find the cortex binary
    let cortex_exe = std::env::current_exe()
        .expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        eprintln!("  Run: cargo build --release");
        std::process::exit(1);
    }
    info!(exe = %cortex_main.display(), "Cortex binary found");

    // ═══════ STAGE 1: Spawn thinking TeamLead ═══════

    info!("Stage 1: Spawning thinking team lead");

    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let lead_id = MindId("research-lead".into());

    match bridge.spawn_thinking(&lead_id, Role::TeamLead).await {
        Ok(()) => info!("research-lead spawned (thinking mode, DelegationInterceptor active)"),
        Err(e) => {
            eprintln!("FAILED to spawn team lead: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 2: List tools (should include delegation tools) ═══════

    info!("Stage 2: Checking team lead tools");

    match bridge.list_tools(&lead_id).await {
        Ok(tools) => {
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            info!("Team lead has {} tools: {:?}", tools.len(), names);
        }
        Err(e) => {
            eprintln!("FAILED to list tools: {e}");
        }
    }

    // ═══════ STAGE 3: Delegate a real task ═══════

    info!("Stage 3: Delegating task — team lead will THINK with real LLM");

    let task_description = "List the files in the current directory and report what you find. \
        If you have the ability to spawn agents, spawn one called 'researcher' and delegate \
        the file listing task to it.";

    match bridge.delegate(
        &lead_id,
        "task-live-001",
        task_description,
        Some("This is a live integration test of the Cortex fractal coordination engine."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   TEAM LEAD RESPONSE                                    ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations: {iterations}");
            println!("Tool calls: {tool_calls}");
            println!("Completed:  {:?}", result.completed);
            println!();
            println!("{response}");
            println!();

            // Write evidence to disk
            let evidence_path = "live_cloud_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX LIVE CLOUD TEST — {}\n\
                 ==========================================\n\
                 Task: {task_description}\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Completed: {:?}\n\
                 \n\
                 Response:\n{response}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                result.completed,
            )).expect("Failed to write evidence");
            info!(path = evidence_path, "Evidence written to disk");
        }
        Err(e) => {
            eprintln!("DELEGATION FAILED: {e}");
        }
    }

    // ═══════ STAGE 4: Shutdown ═══════

    info!("Stage 4: Shutting down");
    bridge.shutdown_all().await;
    info!("All minds shut down. Test complete.");
}
