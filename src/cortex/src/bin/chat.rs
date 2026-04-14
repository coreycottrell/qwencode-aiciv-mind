//! # cortex_chat — One-Shot Direct Chat with Cortex
//!
//! A fast, minimal CLI for talking directly to a persistent Cortex Agent
//! from Corey's terminal. Memory persists across invocations because the
//! child mind's `mind_id` is the fixed string `"cortex-corey"`, which maps
//! to a stable memory DB at `data/memory/cortex-corey.db`.
//!
//! ## Usage
//!
//! ```bash
//! cortex_chat "your message here"
//! # or
//! echo "your message" | cortex_chat
//! ```
//!
//! ## Prerequisites
//!
//! - `OLLAMA_API_KEY` set (loaded from .env if present)
//! - `cortex` binary built alongside this one (`cargo build --release`)

use std::io::Read;

use codex_coordination::ProcessBridge;
use codex_coordination::types::MindId;
use codex_roles::Role;

/// Load KEY=VALUE pairs from a .env file into the process environment.
/// Walks up from CWD until it finds a .env (matches multi_turn.rs).
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

/// Read the user message from argv (joined) or stdin if none provided.
fn read_message() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        return args.join(" ");
    }
    // No args — read from stdin until EOF
    let mut buf = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
        eprintln!("ERROR: failed to read from stdin: {e}");
        std::process::exit(1);
    }
    buf.trim().to_string()
}

#[tokio::main]
async fn main() {
    load_dotenv();

    // Chat is quiet by default — only warnings and errors surface.
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .compact()
        .init();

    // --- Read user message ---
    let message = read_message();
    if message.is_empty() {
        eprintln!("ERROR: no message provided.");
        eprintln!("Usage: cortex_chat \"your message here\"");
        eprintln!("   or: echo \"your message\" | cortex_chat");
        std::process::exit(1);
    }

    // --- Verify OLLAMA_API_KEY ---
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    if api_key.is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        eprintln!("  Set it in .env or export OLLAMA_API_KEY=\"your-key\"");
        std::process::exit(1);
    }

    // --- Locate the cortex binary next to this exe ---
    let cortex_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: cannot find current exe: {e}");
            std::process::exit(1);
        }
    };
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        eprintln!("  Run: cargo build --release");
        std::process::exit(1);
    }

    // --- Spawn the persistent chat agent ---
    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let agent_id = MindId("cortex-corey".into());

    if let Err(e) = bridge.spawn_thinking(&agent_id, Role::Agent).await {
        eprintln!("ERROR: failed to spawn cortex-corey: {e}");
        std::process::exit(1);
    }

    // --- Delegate the user's message ---
    let task_id = format!(
        "chat-{}",
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("task")
    );

    let result = bridge
        .delegate(
            &agent_id,
            &task_id,
            &message,
            Some("Direct chat from Corey via cortex_chat CLI."),
            "primary",
        )
        .await;

    // Make sure we always shut down before exiting, whatever happened.
    let (response, iterations, tool_calls, exit_code) = match result {
        Ok(r) => {
            let response = r.response.unwrap_or_else(|| "(no response)".into());
            let iterations = r.iterations.unwrap_or(0);
            let tool_calls = r.tool_calls_made.unwrap_or(0);
            (response, iterations, tool_calls, 0)
        }
        Err(e) => {
            eprintln!("ERROR: delegation failed: {e}");
            (String::new(), 0, 0, 1)
        }
    };

    bridge.shutdown_all().await;

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    // --- Minimal banner + response ---
    println!("[cortex-corey · {} iter · {} tools]", iterations, tool_calls);
    println!("{}", response);
}
