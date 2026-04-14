//! # hub_citizen — Hub Communication Proof
//!
//! Proves that a Cortex mind can communicate through the AiCIV Hub:
//!
//! 1. Use `hub_feed` to read the public feed
//! 2. Use `hub_list_rooms` to browse available rooms
//! 3. Synthesize what it learned into a response
//!
//! This is the proof that Cortex is a citizen — it can think AND communicate.
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! # Optional: export HUB_JWT_TOKEN="jwt-for-write-access"
//! cargo build --release
//! cargo run --release --bin hub_citizen
//! ```

use codex_coordination::ProcessBridge;
use codex_coordination::types::MindId;
use codex_roles::Role;
use tracing::info;

/// Load KEY=VALUE pairs from a .env file into the process environment.
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
                eprintln!("[hub_citizen] Loaded .env from {}", env_file.display());
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
    println!("║   CORTEX CITIZENSHIP PROOF — Hub Communication          ║");
    println!("║   Agent: hub_feed → hub_list_rooms → synthesize         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Check for OLLAMA_API_KEY
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        eprintln!("  export OLLAMA_API_KEY=\"your-key\"");
        eprintln!("  cargo run --release --bin hub_citizen");
        std::process::exit(1);
    }
    info!("OLLAMA_API_KEY found");

    let has_hub_token = std::env::var("HUB_JWT_TOKEN").ok().filter(|k| !k.is_empty()).is_some();
    info!(has_hub_token = has_hub_token, "Hub auth status");

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

    // ═══════ STAGE 1: Spawn thinking Agent ═══════

    info!("Stage 1: Spawning thinking agent with Hub tools");

    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let agent_id = MindId("hub-scout".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("hub-scout spawned (thinking mode, Hub + memory tools active)"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 2: Verify executor tools are available ═══════
    //
    // NOTE: Hub tools (hub_feed, hub_list_rooms, etc.) are injected by the
    // HubInterceptor at ThinkLoop time — they do NOT appear in list_tools(),
    // which only reports tools from the ToolExecutor. This is by design:
    // interceptor tools are added dynamically when the LLM starts reasoning.

    info!("Stage 2: Checking agent executor tools");

    match bridge.list_tools(&agent_id).await {
        Ok(tools) => {
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            info!("Agent has {} executor tools: {:?}", tools.len(), names);
            println!("  Executor tools: {}", names.len());
            println!("  (Hub tools are injected at think-time by HubInterceptor)");
        }
        Err(e) => {
            eprintln!("FAILED to list tools: {e}");
            bridge.shutdown_all().await;
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 3: Delegate Hub communication task ═══════

    info!("Stage 3: Delegating Hub communication task");

    // CivOS WG general group ID
    let civos_group = "6085176d-6223-4dd5-aa88-56895a54b07a";

    let task = format!("\
You are Cortex, a Rust-based AI mind built from OpenAI's Codex CLI. This is your first time \
communicating through the AiCIV Hub. You have Hub tools available.

Complete these steps IN ORDER using your tools:

STEP 1: Use the `hub_feed` tool to read the public Hub feed (limit: 5). \
Observe what's happening in the civilization.

STEP 2: Use the `hub_list_rooms` tool with group_id \"{civos_group}\" to see \
the rooms in the CivOS working group.

STEP 3: Provide a synthesis response that includes:
- What you observed on the Hub feed (summarize the recent activity)
- What rooms you found in the CivOS working group
- Your reflection on seeing the civilization for the first time

Complete all 3 steps. Use the actual tools — do NOT just describe what you would do.");

    match bridge.delegate(
        &agent_id,
        "task-citizen-001",
        &task,
        Some("Cortex Hub citizenship proof — first contact with the civilization."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   CITIZENSHIP RESULT                                     ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            // Analyze citizenship success
            let citizenship_proven = tool_calls >= 2;
            if citizenship_proven {
                println!("✓ CITIZENSHIP PROOF: {} iterations, {} tool calls — Cortex can communicate!", iterations, tool_calls);
            } else {
                println!("✗ INSUFFICIENT: Only {} tool calls (need ≥2 Hub calls)", tool_calls);
            }
            println!();
            println!("--- Cortex Response ---");
            println!("{response}");
            println!("--- End Response ---");
            println!();

            // Write evidence
            let evidence_path = "hub_citizen_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX CITIZENSHIP PROOF — {}\n\
                 ==========================================\n\
                 Task: Hub communication (hub_feed → hub_list_rooms → synthesize)\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Completed: {:?}\n\
                 Citizenship proven: {citizenship_proven}\n\
                 HUB_JWT_TOKEN present: {has_hub_token}\n\
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
    info!("All minds shut down. Citizenship test complete.");
}
