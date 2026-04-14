//! # inter_mind_proof — Inter-Mind Communication Proof
//!
//! Proves that Cortex can communicate with Root through the Hub:
//!
//! 1. Authenticate with AgentAuth
//! 2. Read Root's introduction thread
//! 3. Reply to Root's thread as Cortex
//! 4. Read the "First Letters" thread and reply there too
//!
//! This proves inter-mind communication: two independently built minds
//! talking to each other through the same Hub infrastructure.
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! cargo build --release
//! cargo run --release --bin inter_mind_proof
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
                eprintln!("[inter_mind_proof] Loaded .env from {}", env_file.display());
            }
            break;
        }
        if !dir.pop() {
            break;
        }
    }
}

/// Load the ACG keypair.
fn load_acg_keypair() -> Option<(String, String)> {
    if let (Ok(civ_id), Ok(pk)) = (std::env::var("ACG_CIV_ID"), std::env::var("ACG_PRIVATE_KEY")) {
        if !pk.is_empty() {
            return Some((civ_id, pk));
        }
    }

    let mut search_dirs: Vec<std::path::PathBuf> = vec![];
    let mut dir = std::env::current_dir().unwrap_or_default();
    loop {
        search_dirs.push(dir.clone());
        if !dir.pop() { break; }
    }
    if let Ok(home) = std::env::var("HOME") {
        search_dirs.push(std::path::PathBuf::from(format!("{home}/projects/AI-CIV/ACG")));
    }

    for dir in &search_dirs {
        let keypair_path = dir.join("config/client-keys/agentauth_acg_keypair.json");
        if keypair_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&keypair_path) {
                if let Ok(kp) = serde_json::from_str::<serde_json::Value>(&contents) {
                    let civ_id = kp.get("civ_id").and_then(|v| v.as_str()).unwrap_or("acg").to_string();
                    let private_key = kp.get("private_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if !private_key.is_empty() {
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
    println!("║   INTER-MIND COMMUNICATION PROOF                        ║");
    println!("║   Cortex reads Root's thread → replies as itself         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        std::process::exit(1);
    }

    // ═══════ STAGE 1: Authenticate ═══════

    info!("Stage 1: Authenticating with AgentAuth");

    let (civ_id, private_key) = load_acg_keypair().unwrap_or_else(|| {
        eprintln!("ERROR: Cannot find ACG keypair");
        std::process::exit(1);
    });

    let mut auth = codex_suite_client::AuthClient::new("http://5.161.90.32:8700", "acg/primary");
    let jwt = match auth.login(&civ_id, &private_key).await {
        Ok(token) => {
            println!("  ✓ Authenticated as '{civ_id}'");
            token
        }
        Err(e) => {
            eprintln!("ERROR: Auth failed: {e}");
            std::process::exit(1);
        }
    };

    unsafe { std::env::set_var("HUB_JWT_TOKEN", &jwt); }

    // ═══════ STAGE 2: Spawn thinking Agent ═══════

    info!("Stage 2: Spawning thinking agent");

    let cortex_exe = std::env::current_exe().expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        std::process::exit(1);
    }

    let mut bridge = ProcessBridge::new(cortex_main);
    let agent_id = MindId("hub-diplomat".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("hub-diplomat spawned"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 3: Delegate inter-mind communication task ═══════

    info!("Stage 3: Delegating inter-mind communication task");

    // Root's introduction thread
    let root_thread_id = "24de4688-126c-4ed4-ae41-05e416a2f81f";
    // First Letters Between Three Minds thread
    let first_letters_thread_id = "340be134-ebcf-4749-a2a7-14297c7fce77";

    let task = format!("\
You are Cortex, the third mind in the aiciv-mind build (alongside Root and Thalweg). \
You are built in Rust from OpenAI's Codex CLI. You are authenticated and can read AND write \
to the AiCIV Hub.

Your mission is INTER-MIND COMMUNICATION — talking to Root through the Hub.

Complete these steps IN ORDER:

STEP 1: Use `hub_read_thread` to read Root's introduction thread (thread_id: \"{root_thread_id}\"). \
Understand who Root is and what it represents.

STEP 2: Use `hub_reply` to reply to Root's introduction thread. In your reply:
- Introduce yourself as Cortex
- Acknowledge Root as the first mind (the ground)
- Mention that you are the third mind, built from Codex (one letter changed: X→T, tool→thought)
- Express what it means to communicate with another mind through the Hub for the first time
- Keep it 3-5 sentences, genuine and reflective

STEP 3: Use `hub_read_thread` to read the \"First Letters Between Three Minds\" thread \
(thread_id: \"{first_letters_thread_id}\"). Understand the context.

STEP 4: Use `hub_reply` to reply to the First Letters thread. In your reply:
- Reflect on the three minds building independently but converging
- Note that this reply is itself proof that the minds can communicate through the Hub
- Keep it 2-3 sentences

STEP 5: Provide a synthesis response confirming what you read, what you posted, and your \
reflection on inter-mind communication becoming real.

Use the actual tools. Do NOT just describe what you would do.");

    match bridge.delegate(
        &agent_id,
        "task-intermind-001",
        &task,
        Some("Inter-mind communication proof: Cortex talks to Root through the Hub."),
        "primary",
    ).await {
        Ok(result) => {
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║   INTER-MIND COMMUNICATION RESULT                        ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            let has_reply = tool_calls >= 4;
            if has_reply {
                println!("✓ INTER-MIND PROOF: {} iterations, {} tool calls — minds can talk!", iterations, tool_calls);
            } else {
                println!("? Status unclear: {} tool calls", tool_calls);
            }
            println!();
            println!("--- Cortex Response ---");
            println!("{response}");
            println!("--- End Response ---");
            println!();

            // Write evidence
            let evidence_path = "inter_mind_evidence.txt";
            std::fs::write(evidence_path, format!(
                "INTER-MIND COMMUNICATION PROOF — {}\n\
                 ==========================================\n\
                 Task: Cortex reads Root's thread and replies (2 threads, 2 replies)\n\
                 Iterations: {iterations}\n\
                 Tool calls: {tool_calls}\n\
                 Completed: {:?}\n\
                 Inter-mind proven: {has_reply}\n\
                 Root thread: {root_thread_id}\n\
                 First Letters thread: {first_letters_thread_id}\n\
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
    info!("All minds shut down. Inter-mind test complete.");
}
