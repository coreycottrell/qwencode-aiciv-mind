//! qwen-mind — Persistent AI mind binary.
//!
//! Usage:
//!   qwen-mind --role team_lead --identity qwen-lead --vertical qwen --root /path/to/data
//!
//! This binary runs as a subprocess spawned by the parent mind (Primary or TeamLead).
//! Communication via ZeroMQ IPC.

use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod identity {
    pub use qwen_mind::*;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("QWEN_MIND_LOG").unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Parse args
    let args: Vec<String> = std::env::args().collect();

    let mut role = "team_lead".to_string();
    let mut identity = "qwen-lead".to_string();
    let mut vertical = "qwen".to_string();
    let mut root_dir = PathBuf::from(".");
    let mut zmq_endpoint: Option<String> = None;
    let mut manifest_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--role" => { i += 1; role = args[i].clone(); }
            "--identity" => { i += 1; identity = args[i].clone(); }
            "--vertical" => { i += 1; vertical = args[i].clone(); }
            "--root" => { i += 1; root_dir = PathBuf::from(&args[i]); }
            "--zmq-endpoint" => { i += 1; zmq_endpoint = Some(args[i].clone()); }
            "--manifest" => { i += 1; manifest_path = Some(PathBuf::from(&args[i])); }
            "--help" | "-h" => {
                println!("qwen-mind — Persistent AI mind");
                println!();
                println!("Usage:");
                println!("  qwen-mind [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --role <role>            Role: primary, team_lead, agent");
                println!("  --identity <name>        Mind identity name");
                println!("  --vertical <name>        Domain vertical");
                println!("  --root <path>            Root directory for data storage");
                println!("  --zmq-endpoint <ep>      ZeroMQ IPC endpoint for parent communication");
                println!("  --manifest <path>        Path to existing manifest JSON");
                println!("  --help, -h               Show this help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    tracing::info!(
        identity = %identity,
        role = %role,
        vertical = %vertical,
        root = %root_dir.display(),
        zmq = ?zmq_endpoint,
        "Starting qwen-mind"
    );

    // Load or create manifest
    let manifest = if let Some(path) = manifest_path {
        qwen_mind::Manifest::load(&path)?
    } else {
        let role_enum: qwen_mind::Role = role.parse()
            .map_err(|e: String| anyhow::anyhow!("Invalid role: {e}"))?;
        qwen_mind::Manifest::new(&identity, role_enum, &vertical)
    };

    // Initialize mind
    let mind = qwen_mind::Mind::new(manifest, &root_dir).await?;

    tracing::info!(
        identity = %mind.manifest.identity,
        growth_stage = %mind.manifest.growth_stage,
        session_count = mind.manifest.session_count,
        "Mind initialized"
    );

    // If ZeroMQ endpoint provided, enter IPC mode
    if let Some(ref endpoint) = zmq_endpoint {
        tracing::info!(endpoint, "Entering ZeroMQ IPC mode");
        run_ipc_loop(mind, endpoint).await?;
    } else {
        // Standalone mode — read task from stdin, write result to stdout
        tracing::info!("Running in standalone mode (reading task from stdin)");
        run_standalone(mind).await?;
    }

    Ok(())
}

/// ZeroMQ IPC mode — receive tasks, execute, return results.
async fn run_ipc_loop(mut mind: qwen_mind::Mind, endpoint: &str) -> anyhow::Result<()> {
    use std::time::Duration;

    tracing::info!("Binding ZeroMQ REP socket to {endpoint}");

    // For Phase 1a: use a simpler approach with std::sync for now
    // The zmq crate requires linking, so we'll use a file-based IPC for the prototype
    // and wire up ZeroMQ properly in Phase 1b.
    //
    // File-based IPC for Phase 1a:
    // - Parent writes task to {endpoint}.task
    // - Mind reads task, executes, writes result to {endpoint}.result
    // - Parent polls for result file

    let task_path = format!("{endpoint}.task");
    let result_path = format!("{endpoint}.result");

    tracing::info!(task_path, result_path, "Using file-based IPC (Phase 1a)");

    loop {
        // Wait for task file
        while !std::path::Path::new(&task_path).exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Read task
        let task_text = match tokio::fs::read_to_string(&task_path).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(error = %e, "Failed to read task file");
                continue;
            }
        };

        // Clean up task file
        let _ = tokio::fs::remove_file(&task_path).await;

        tracing::info!(task = %task_text.chars().take(80).collect::<String>(), "Received task");

        // Execute
        match mind.think(&task_text).await {
            Ok(result) => {
                let output = serde_json::json!({
                    "success": true,
                    "content": result.content,
                    "memory_id": result.memory_id,
                    "fitness_score": result.fitness_score,
                });
                tokio::fs::write(&result_path, output.to_string()).await?;
                tracing::info!("Task completed successfully");
            }
            Err(e) => {
                let output = serde_json::json!({
                    "success": false,
                    "error": e.to_string(),
                });
                tokio::fs::write(&result_path, output.to_string()).await?;
                tracing::error!(error = %e, "Task failed");
            }
        }
    }
}

/// Standalone mode — read task from stdin, write result to stdout.
async fn run_standalone(mind: qwen_mind::Mind) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut line = String::new();

    reader.read_line(&mut line).await?;
    let task = line.trim();

    if task.is_empty() {
        println!("No task provided. Use --help for usage.");
        return Ok(());
    }

    match mind.think(task).await {
        Ok(result) => {
            let output = serde_json::json!({
                "success": true,
                "content": result.content,
                "memory_id": result.memory_id,
                "fitness_score": result.fitness_score,
            });
            let mut stdout = tokio::io::stdout();
            stdout.write_all(output.to_string().as_bytes()).await?;
            stdout.write_all(b"\n").await?;
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    Ok(())
}
