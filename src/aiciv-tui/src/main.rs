//! aiciv-tui — Minimal terminal chat UI for aiciv-mind.
//!
//! Usage:
//!   aiciv-tui [OPTIONS]
//!
//! Options:
//!   --model <NAME>       Model name (default: devstral-small-2:24b)
//!   --url <URL>          Ollama base URL (default: https://api.ollama.com or http://localhost:11434)
//!   --mind-id <ID>       Mind identifier (default: tui-agent)
//!   --workspace <PATH>   Workspace root for sandbox (default: current directory)

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tracing::info;

use codex_exec::tools::register_builtins;
use codex_exec::{SandboxEnforcer, ToolRegistry};
use codex_llm::ollama::{OllamaClient, OllamaConfig};
use codex_llm::prompt::PromptBuilder;
use codex_llm::think_loop::{ThinkLoop, ThinkLoopConfig};
use codex_roles::Role;

use aiciv_tui::app::App;

/// Parse a simple `--key value` pair from args.
fn parse_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (file-based to avoid polluting TUI)
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter("aiciv_tui=info,codex_llm=info")
        .init();

    let args: Vec<String> = std::env::args().collect();

    // Parse CLI arguments
    let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
    let default_url = if api_key.is_some() {
        "https://api.ollama.com"
    } else {
        "http://localhost:11434"
    };

    let model = parse_arg(&args, "--model")
        .unwrap_or_else(|| "devstral-small-2:24b".into());
    let base_url = parse_arg(&args, "--url")
        .unwrap_or_else(|| default_url.into());
    let mind_id = parse_arg(&args, "--mind-id")
        .unwrap_or_else(|| "tui-agent".into());
    let workspace = parse_arg(&args, "--workspace")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    info!(model = %model, url = %base_url, mind_id = %mind_id, "Starting aiciv-tui");

    // Create the LLM client
    let config = OllamaConfig {
        base_url,
        model: model.clone(),
        temperature: 1.0,
        max_tokens: 4096,
        api_key,
    };
    let client = OllamaClient::new(config);

    // Create the ThinkLoop
    let think_loop = ThinkLoop::new(
        Box::new(client),
        ThinkLoopConfig::default(),
    );

    // Create tool registry with built-in tools
    let mut registry = ToolRegistry::new();
    register_builtins(&mut registry, workspace.clone());

    // Get tool schemas for the LLM
    let definitions = registry.definitions_for_role(Role::Agent);
    let tool_schemas = codex_llm::ollama::OllamaClient::tool_schemas(&definitions);

    // Create the executor
    let sandbox = SandboxEnforcer::new(workspace);
    let executor = codex_exec::ToolExecutor::new(registry, sandbox);

    // Create the prompt builder
    let prompt_builder = PromptBuilder::new(Role::Agent, &mind_id);

    // Set up the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run the app
    let mut app = App::new(&model);
    let result = app
        .run(
            &mut terminal,
            &think_loop,
            &executor,
            &prompt_builder,
            Role::Agent,
            &tool_schemas,
        )
        .await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
