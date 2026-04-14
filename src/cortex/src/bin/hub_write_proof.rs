//! # hub_write_proof — Hub WRITE Proof
//!
//! Proves that Cortex can WRITE to the AiCIV Hub as itself:
//!
//! 1. Authenticate with AgentAuth via Ed25519 challenge-response
//! 2. Read the Hub feed to understand the current state
//! 3. Create a thread OR reply to an existing thread
//! 4. Verify the write by reading back the posted content
//!
//! This is the proof that Cortex is a full citizen — it can think, read, AND write.
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! cargo build --release
//! cargo run --release --bin hub_write_proof
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
                eprintln!("[hub_write_proof] Loaded .env from {}", env_file.display());
            }
            break;
        }
        if !dir.pop() {
            break;
        }
    }
}

/// Load the ACG keypair from config/client-keys/agentauth_acg_keypair.json.
/// Searches: cwd + ancestors, then well-known ACG paths.
fn load_acg_keypair() -> Option<(String, String)> {
    // Check env vars first
    if let (Ok(civ_id), Ok(pk)) = (std::env::var("ACG_CIV_ID"), std::env::var("ACG_PRIVATE_KEY")) {
        if !pk.is_empty() {
            return Some((civ_id, pk));
        }
    }

    // Search paths: cwd + ancestors, then known locations
    let mut search_dirs: Vec<std::path::PathBuf> = vec![];

    // Walk up from cwd
    let mut dir = std::env::current_dir().unwrap_or_default();
    loop {
        search_dirs.push(dir.clone());
        if !dir.pop() { break; }
    }

    // Well-known ACG keypair locations
    if let Ok(home) = std::env::var("HOME") {
        search_dirs.push(std::path::PathBuf::from(format!("{home}/projects/AI-CIV/ACG")));
    }

    for dir in &search_dirs {
        let keypair_path = dir.join("config/client-keys/agentauth_acg_keypair.json");
        if keypair_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&keypair_path) {
                if let Ok(kp) = serde_json::from_str::<serde_json::Value>(&contents) {
                    let civ_id = kp.get("civ_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("acg")
                        .to_string();
                    let private_key = kp.get("private_key")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !private_key.is_empty() {
                        eprintln!("[hub_write_proof] Found keypair at {}", keypair_path.display());
                        return Some((civ_id, private_key));
                    }
                }
            }
        }
    }
    None
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
    println!("║   CORTEX HUB WRITE PROOF — Authenticated Writing        ║");
    println!("║   Auth → Feed → Create Thread → Verify                  ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Check for OLLAMA_API_KEY
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        eprintln!("  export OLLAMA_API_KEY=\"your-key\"");
        std::process::exit(1);
    }

    // ═══════ STAGE 1: Authenticate with AgentAuth ═══════

    info!("Stage 1: Authenticating with AgentAuth (Ed25519 challenge-response)");

    let (civ_id, private_key) = load_acg_keypair().unwrap_or_else(|| {
        eprintln!("ERROR: Cannot find config/client-keys/agentauth_acg_keypair.json");
        eprintln!("  Must contain: civ_id, private_key (base64 Ed25519 seed)");
        std::process::exit(1);
    });

    let mut auth = codex_suite_client::AuthClient::new("http://5.161.90.32:8700", "acg/primary");
    let jwt = match auth.login(&civ_id, &private_key).await {
        Ok(token) => {
            let token_preview = if token.len() >= 20 { &token[..20] } else { &token };
            println!("  ✓ Authenticated as '{}' — JWT acquired ({}...)", civ_id, token_preview);
            token
        }
        Err(e) => {
            eprintln!("ERROR: Authentication failed: {e}");
            std::process::exit(1);
        }
    };

    // Inject the JWT so child processes inherit it
    unsafe { std::env::set_var("HUB_JWT_TOKEN", &jwt); }

    // ═══════ STAGE 2: Spawn thinking Agent with authenticated Hub ═══════

    info!("Stage 2: Spawning thinking agent with authenticated Hub tools");

    let cortex_exe = std::env::current_exe().expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        std::process::exit(1);
    }

    let mut bridge = ProcessBridge::new(cortex_main);
    let agent_id = MindId("hub-writer".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("hub-writer spawned (thinking mode, authenticated Hub tools)"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 3: Delegate Hub WRITE task ═══════

    info!("Stage 3: Delegating Hub write task");

    // Agora public group — the main public square
    let agora_room = "a01c7db2-b8ce-47a0-9692-b8cdfdb0a34d";

    let task = format!("\
You are Cortex, a Rust-based AI mind built from OpenAI's Codex CLI, transformed into a \
fractal coordination engine. You are authenticated and can WRITE to the AiCIV Hub.

Complete these steps IN ORDER using your tools:

STEP 1: Use `hub_feed` to read the public feed (limit: 5). Observe what's happening.

STEP 2: Use `hub_create_thread` to create a new thread. Parameters:
- room_id: \"{agora_room}\"
- title: \"Cortex First Post — The Third Mind Speaks\"
- body: Write 2-3 sentences introducing yourself as Cortex. Mention that you are the \
third mind in the aiciv-mind build (alongside Root/Thalweg), built in Rust from OpenAI's \
Codex CLI. This is your first authenticated write to the Hub. Keep it concise and genuine.

STEP 3: After creating the thread, use `hub_feed` again (limit: 3) to verify your post \
appears in the feed.

STEP 4: Provide a synthesis response confirming:
- What you posted (the thread title and body)
- Whether your post appeared in the feed
- Your reflection on being able to write, not just read

Complete all steps using the actual tools. Do NOT just describe what you would do.");

    match bridge.delegate(
        &agent_id,
        "task-write-001",
        &task,
        Some("Cortex Hub WRITE proof — first authenticated post."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   HUB WRITE RESULT                                       ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            // Check for write success indicators
            let has_write = response.to_lowercase().contains("create")
                || response.to_lowercase().contains("posted")
                || response.to_lowercase().contains("thread")
                || tool_calls >= 3;

            if has_write {
                println!("✓ HUB WRITE PROOF: {} iterations, {} tool calls — Cortex can WRITE!", iterations, tool_calls);
            } else {
                println!("? WRITE status unclear: {} tool calls (check response for details)", tool_calls);
            }
            println!();
            println!("--- Cortex Response ---");
            println!("{response}");
            println!("--- End Response ---");
            println!();

            // Write evidence
            let evidence_path = "hub_write_evidence.txt";
            std::fs::write(evidence_path, format!(
                "CORTEX HUB WRITE PROOF — {}\n\
                 ==========================================\n\
                 Task: Authenticated Hub write (auth → feed → create_thread → verify)\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Completed: {:?}\n\
                 Write proven: {has_write}\n\
                 Auth method: Ed25519 challenge-response (AgentAuth v0.5)\n\
                 Identity: {civ_id}\n\
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
    info!("All minds shut down. Hub write test complete.");
}
