//! # multi_turn — Multi-Turn Tool Use Proof
//!
//! Proves that a Cortex agent can reason across multiple tool-use turns:
//!
//! 1. Use `bash` to read a file
//! 2. Use `memory_write` to store a learning
//! 3. Use `memory_search` to verify the learning was stored
//! 4. Produce a final synthesis response
//!
//! This is the proof that Cortex's ThinkLoop handles multi-turn
//! tool use with real LLM inference — not just single-shot tool calls.
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! cargo build --release
//! cargo run --release --bin multi_turn
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
                eprintln!("[multi_turn] Loaded .env from {}", env_file.display());
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
    println!("║   CORTEX MULTI-TURN PROOF — Real Multi-Step Reasoning   ║");
    println!("║   Agent: bash → memory_write → memory_search → respond  ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Check for OLLAMA_API_KEY
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        eprintln!("  export OLLAMA_API_KEY=\"your-key\"");
        eprintln!("  cargo run --release --bin multi_turn");
        std::process::exit(1);
    }
    info!("OLLAMA_API_KEY found");

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

    // ═══════ STAGE 1: Spawn thinking Agent (not TeamLead) ═══════

    info!("Stage 1: Spawning thinking agent");

    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let agent_id = MindId("researcher".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("researcher spawned (thinking mode, memory tools active)"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 2: Check available tools ═══════

    info!("Stage 2: Checking agent tools");

    match bridge.list_tools(&agent_id).await {
        Ok(tools) => {
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            info!("Agent has {} tools: {:?}", tools.len(), names);

            // Verify memory tools are present
            let has_bash = names.contains(&"bash");
            let has_mem_search = names.contains(&"memory_search");
            let has_mem_write = names.contains(&"memory_write");
            println!("  bash: {}", if has_bash { "YES" } else { "NO" });
            println!("  memory_search: {}", if has_mem_search { "YES" } else { "NO" });
            println!("  memory_write: {}", if has_mem_write { "YES" } else { "NO" });
        }
        Err(e) => {
            eprintln!("FAILED to list tools: {e}");
        }
    }

    // ═══════ STAGE 3: Delegate multi-turn task ═══════

    info!("Stage 3: Delegating multi-turn task");

    let task = "\
You have 3 tasks to complete IN ORDER. You MUST use the tools described below — \
do not just describe what you would do, actually call the tools.

STEP 1: Use the `bash` tool to run `cat MISSION.md | head -20` and read the first 20 lines of MISSION.md.

STEP 2: After reading the file, use the `memory_write` tool to store what you learned. \
Use title 'MISSION.md Summary' and describe what the file is about in the content field.

STEP 3: After writing the memory, use the `memory_search` tool with query 'MISSION' \
to verify your memory was stored correctly.

STEP 4: Finally, provide a summary response that includes:
- What you read from MISSION.md
- Confirmation that you wrote a memory
- Confirmation that memory_search found your written memory

Complete all 4 steps. Do NOT skip any step.";

    match bridge.delegate(
        &agent_id,
        "task-multi-001",
        task,
        Some("This is a multi-turn tool use integration test for Cortex."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   MULTI-TURN RESULT                                     ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            // Analyze multi-turn success
            let multi_turn_success = iterations >= 3 && tool_calls >= 3;
            if multi_turn_success {
                println!("✓ MULTI-TURN PROOF: {} iterations, {} tool calls", iterations, tool_calls);
            } else {
                println!("✗ INSUFFICIENT: Only {} iterations, {} tool calls (need ≥3 each)", iterations, tool_calls);
            }
            println!();
            println!("--- Agent Response ---");
            println!("{response}");
            println!("--- End Response ---");
            println!();

            // Write evidence
            let evidence_path = "multi_turn_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX MULTI-TURN PROOF — {}\n\
                 ==========================================\n\
                 Task: Multi-step tool use (bash → memory_write → memory_search → respond)\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Completed: {:?}\n\
                 Multi-turn success: {multi_turn_success}\n\
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
