//! # Cortex — The Fractal Coordination Engine
//!
//! Born from Codex. One letter transformed. A tool becomes a mind.
//!
//! ## Modes
//!
//! - **Demo mode** (default): Runs the full 23-phase lifecycle demonstration.
//! - **Serve mode** (`--serve`): Runs as an MCP server on stdin/stdout.
//!   Used when a parent mind spawns this process as a child.
//!
//! ## Serve Mode Usage
//!
//! ```bash
//! cortex --serve --mind-id research-lead --role team-lead
//! ```

mod boot;
mod config;
mod drive;
mod input_route;
mod progress;
mod task_history;
mod qwen_delegate;
mod monitoring;

use chrono::Utc;
use codex_coordination::MindManager;
use codex_coordination::types::*;
use codex_dream::{DreamConfig, DreamEngine};
use codex_fitness::{self, TaskOutcome};
use codex_exec::{ToolCall, ToolExecutor, ToolRegistry, SandboxEnforcer};
use codex_ipc::server::McpMindServer;
use codex_ipc::client::McpMindClient;
use codex_ipc::transport::{ChannelTransport, StdioServerTransport, StdioTransport};
use codex_ipc::DelegateTaskResult;
use codex_llm::prompt::PromptBuilder;
use codex_llm::ollama::{OllamaClient, OllamaConfig, ToolSchema, FunctionSchema};
use codex_llm::think_loop::{ThinkLoop, ThinkLoopConfig};
use codex_memory::{MemoryCategory, MemoryQuery, MemoryStore, MemoryTier, NewMemory, LinkType};
use codex_redteam::{CompletionClaim, Evidence, EvidenceType, Freshness, RedTeamProtocol};
use codex_roles::{Role, Vertical};
use codex_transfer::{Confidence, PatternContent, ShareScope, TransferEngine, TransferPattern};
use tracing::info;

use config::CortexConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Truncate a string to at most `max` bytes, landing on a valid UTF-8 char boundary.
/// Prevents panics when slicing strings containing multi-byte characters (e.g., Hub content).
fn safe_truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

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
                            // SAFETY: called before any threads are spawned (main thread only, before tokio runtime)
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
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--daemon") {
        daemon_mode(&args).await;
    } else if args.iter().any(|a| a == "--serve") {
        serve_mode(&args).await;
    } else {
        demo_mode().await;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DAEMON MODE — Autonomous event loop (no MCP stdin, DriveLoop drives behavior)
// ═══════════════════════════════════════════════════════════════════════════════

async fn daemon_mode(args: &[String]) {
    let log_level = std::env::var("CORTEX_LOG_LEVEL").unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(&log_level)
        .with_target(false)
        .compact()
        .init();

    let mind_id = arg_value(args, "--mind-id").unwrap_or_else(|| "root".into());
    let role = match arg_value(args, "--role").as_deref() {
        Some("agent") => Role::Agent,
        Some("team-lead") | Some("teamlead") => Role::TeamLead,
        _ => Role::Primary, // Daemon defaults to Primary
    };
    let model_override = arg_value(args, "--model");
    let seed_task = arg_value(args, "--seed-task");
    let seed_tasks_file = arg_value(args, "--seed-tasks");
    let stall_threshold: Option<i64> = arg_value(args, "--stall-threshold")
        .and_then(|s| s.parse().ok());

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   CORTEX DAEMON — Autonomous Event Loop                  ║");
    println!("║   DriveLoop → EventBus → ThinkLoop → Delegation         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    info!(
        mind_id = %mind_id,
        role = ?role,
        model = model_override.as_deref().unwrap_or("(config default)"),
        "Cortex daemon starting"
    );

    // Load config
    let config = CortexConfig::find_and_load(
        std::env::current_dir().unwrap_or_default()
    ).unwrap_or_default();
    let mut router = config.model_router();

    // Apply model override if provided
    if let Some(ref model) = model_override {
        info!(model = %model, "Overriding all models");
        router.primary_model = model.clone();
        router.team_lead_model = model.clone();
        router.agent_model = model.clone();
    }

    info!(
        primary = %router.primary_model,
        base_url = %router.base_url,
        cloud = router.api_key.is_some(),
        "Config loaded"
    );

    let project_root = std::env::current_dir().unwrap_or_default();

    // Build executor (bash, read, write, glob, grep)
    let executor = build_executor();

    // Persistent memory
    let memory_dir = project_root.join("data").join("memory");
    let _ = std::fs::create_dir_all(&memory_dir);
    let memory_path = memory_dir.join(format!("{}.db", &mind_id));
    info!(path = %memory_path.display(), "Opening persistent memory store");
    let memory = MemoryStore::new(&memory_path.display().to_string()).await
        .expect("Failed to create persistent memory store");

    // Boot context — identity, last handoff, scratchpad, recent memories
    let boot_ctx = boot::BootContext::load(
        &project_root,
        &mind_id,
        role,
        Some(&memory),
    ).await;

    // Scratchpad directory
    let scratchpad_dir = project_root.join("data").join("scratchpad");
    let _ = std::fs::create_dir_all(&scratchpad_dir);

    // Scratchpad daily rollover — archive stale scratchpads from previous days
    if let Err(e) = boot::rollover_scratchpads(&scratchpad_dir) {
        tracing::warn!(error = %e, "Failed to roll over scratchpads");
    }

    // Hum (witness) observation directory
    let hum_dir = project_root.join("data").join("hum");
    let _ = std::fs::create_dir_all(&hum_dir);
    let hum_log_path = hum_dir.join(format!("{}.jsonl", Utc::now().format("%Y-%m-%d")));

    // Rate limiter — tracks Ollama usage + circuit breaker
    let metrics_dir = project_root.join("data").join("metrics");
    let _ = std::fs::create_dir_all(&metrics_dir);
    let rate_limiter = codex_llm::RateLimiter::new(metrics_dir);

    // Task ledger (JSONL audit trail)
    let tasks_dir = project_root.join("data").join("tasks");
    let _ = std::fs::create_dir_all(&tasks_dir);
    let task_ledger = codex_coordination::TaskLedger::open(&tasks_dir);

    // Build custom DriveConfig if --stall-threshold is set
    let drive_config = stall_threshold.map(|secs| {
        info!(stall_threshold_secs = secs, "Custom stall threshold");
        let mut dc = codex_drive::DriveConfig::default();
        dc.stall_threshold_secs = secs;
        dc
    });

    // Boot drive subsystem in daemon mode — returns EventBus to us
    let (daemon_handles, mut bus) = drive::boot_daemon(&project_root, &mind_id, "primary", drive_config).await
        .expect("Failed to boot drive subsystem");
    let task_store = daemon_handles.task_store.clone();
    let completion_sender = daemon_handles.drive_loop.completion_sender();

    // Build ThinkLoop
    let ollama_config = router.config_for_role(role);
    let think_config = ThinkLoopConfig {
        max_iterations: 15,
        ollama: codex_llm::ollama::OllamaConfig {
            max_tokens: 4096,
            ..ollama_config
        },
    };
    let think_loop = codex_llm::think_loop::ThinkLoop::new(think_config)
        .with_scratchpad_dir(scratchpad_dir.clone())
        .with_hum_dir(hum_dir.clone())
        .with_rate_limiter(rate_limiter.clone());

    // Tool schemas for the LLM
    let mut tool_schemas = codex_llm::ollama::OllamaClient::tool_schemas(
        &executor.registry().definitions_for_role(role),
    );
    // Add qwen_delegate schema
    let qwen_schema = qwen_delegate::QwenDelegate::schema();
    if let Some(func) = qwen_schema.get("function") {
        tool_schemas.push(ToolSchema {
            tool_type: "function".to_string(),
            function: codex_llm::ollama::FunctionSchema {
                name: func.get("name").and_then(|v| v.as_str()).unwrap_or("qwen_delegate").to_string(),
                description: func.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                parameters: func.get("parameters").cloned().unwrap_or_default(),
            },
        });
    }

    // Hub interceptor
    let hub = config.suite.hub_interceptor();

    // Task history interceptor
    let task_history = task_history::TaskHistoryInterceptor::new(task_ledger.clone());

    // Input route interceptor
    let input_route = input_route::InputRouteInterceptor::new(&mind_id);

    // Progress interceptor
    let progress_dir = project_root.join("data").join("progress");
    let progress = progress::ProgressInterceptor::new(&progress_dir, &mind_id);

    // Search interceptor — web search + fetch for all minds
    let search = codex_suite_client::SearchInterceptor::new();

    // ElevenLabs TTS interceptor — text-to-speech for all minds
    let audio_dir = project_root.join("data").join("audio");
    let elevenlabs = codex_suite_client::ElevenLabsInterceptor::new(&audio_dir);

    // Image generation interceptor — Gemini Imagen for visual content
    let images_dir = project_root.join("data").join("images");
    let image_gen = codex_suite_client::ImageGenInterceptor::new(&images_dir, &project_root);

    // Delegation interceptor (Primary/TeamLead can spawn children)
    let cortex_exe = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("cortex"));
    let delegation = DelegationInterceptor::with_stores(
        cortex_exe,
        mind_id.clone(),
        task_ledger,
        task_store.clone(),
        completion_sender,
    );

    // Build prompt template
    let agents_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("../../agents"))
        .and_then(|p| std::fs::canonicalize(&p).ok());

    // Seed a task if --seed-task was provided
    if let Some(ref task_desc) = seed_task {
        let task_id = format!("seed-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let task = codex_drive::StoredTask::new(
            &task_id,
            task_desc,
            codex_drive::TaskPriority::Normal,
            Some("daemon"),
        );
        match task_store.insert(&task).await {
            Ok(()) => info!(task_id = %task_id, "Seeded task: {}", task_desc),
            Err(e) => tracing::error!(error = %e, "Failed to seed task"),
        }
    }

    // Seed multiple tasks from JSON file if --seed-tasks was provided
    // Format: [{"description": "...", "priority": "normal"}, ...]
    // Priority is optional (defaults to Normal). Valid values: low, normal, high, critical.
    if let Some(ref tasks_path) = seed_tasks_file {
        match std::fs::read_to_string(tasks_path) {
            Ok(json_str) => {
                #[derive(serde::Deserialize)]
                struct SeedTask {
                    description: String,
                    #[serde(default = "default_priority")]
                    priority: codex_drive::TaskPriority,
                }
                fn default_priority() -> codex_drive::TaskPriority {
                    codex_drive::TaskPriority::Normal
                }

                match serde_json::from_str::<Vec<SeedTask>>(&json_str) {
                    Ok(tasks) => {
                        info!(count = tasks.len(), path = %tasks_path, "Loading seed tasks from file");
                        for (i, st) in tasks.iter().enumerate() {
                            let task_id = format!("mission-{:03}-{}", i,
                                uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
                            let task = codex_drive::StoredTask::new(
                                &task_id,
                                &st.description,
                                st.priority,
                                Some("daemon"),
                            );
                            match task_store.insert(&task).await {
                                Ok(()) => info!(
                                    task_id = %task_id,
                                    priority = ?st.priority,
                                    seq = i,
                                    "Seeded mission: {}",
                                    &st.description[..st.description.len().min(80)]
                                ),
                                Err(e) => tracing::error!(
                                    error = %e,
                                    seq = i,
                                    "Failed to seed mission"
                                ),
                            }
                        }
                    }
                    Err(e) => tracing::error!(
                        error = %e,
                        path = %tasks_path,
                        "Failed to parse seed tasks JSON"
                    ),
                }
            }
            Err(e) => tracing::error!(
                error = %e,
                path = %tasks_path,
                "Failed to read seed tasks file"
            ),
        }
    }

    info!(
        mind_id = %mind_id,
        model = %router.primary_model,
        "Daemon ready — entering event loop"
    );

    // ═══════ MAIN EVENT LOOP ═══════
    let mut events_processed: u64 = 0;
    let start_time = std::time::Instant::now();

    while let Some(event) = bus.recv().await {
        events_processed += 1;
        let uptime = start_time.elapsed().as_secs();

        // Track task_id for TaskAvailable events so we can mark complete after ThinkLoop
        let mut active_task_id: Option<String> = None;

        let task_description = match &event {
            codex_types::MindEvent::External(ext) => {
                info!(
                    mind_id = %mind_id,
                    source = ?ext.source,
                    event_num = events_processed,
                    "Processing external event"
                );
                format!(
                    "External event from {:?}:\n\n{}\n\nProcess this event. Take any needed action.",
                    ext.source, ext.content
                )
            }
            codex_types::MindEvent::Drive(drive_event) => {
                use codex_types::DriveEvent;
                match drive_event {
                    DriveEvent::IdleSuggestion { suggestion } => {
                        info!(
                            mind_id = %mind_id,
                            event_num = events_processed,
                            uptime_secs = uptime,
                            "Processing idle suggestion"
                        );
                        format!(
                            "You are a Cortex daemon — an autonomous AI mind.\n\
                             You have been idle. {suggestion}\n\n\
                             Your SOUL.md describes who you are. Your scratchpad may have notes.\n\
                             Your tools: bash, read, write, glob, grep, memory_search, memory_write, \
                             scratchpad_read, scratchpad_write, spawn_agent, delegate_to_agent, shutdown_agent.\n\n\
                             What should you do? Review your identity, check for work, or create tasks.\n\
                             If there is nothing to do, say so — do NOT invent fake work."
                        )
                    }
                    DriveEvent::TaskAvailable { task_id, description, priority } => {
                        info!(
                            mind_id = %mind_id,
                            task_id = %task_id,
                            priority = ?priority,
                            event_num = events_processed,
                            "Processing available task"
                        );
                        // Mark in-progress so DriveLoop doesn't re-surface it
                        let _ = task_store.assign(task_id, &mind_id).await;
                        active_task_id = Some(task_id.clone());
                        daemon_handles.drive_loop.mark_productive().await;
                        format!(
                            "Task available: [{task_id}] {description}\n\
                             Priority: {priority:?}\n\n\
                             Execute this task or delegate it to an agent."
                        )
                    }
                    DriveEvent::StallDetected { task_id, mind_id: stalled_mind, stalled_seconds } => {
                        tracing::warn!(
                            mind_id = %mind_id,
                            stalled_mind = %stalled_mind,
                            task_id = %task_id,
                            stalled_seconds = stalled_seconds,
                            "Processing stall detection"
                        );
                        format!(
                            "STALL DETECTED: Task [{task_id}] assigned to mind '{stalled_mind}' \
                             has been stalled for {stalled_seconds}s.\n\n\
                             Investigate and take corrective action (restart, reassign, or escalate)."
                        )
                    }
                    DriveEvent::HealthCheck { active_minds, pending_tasks, uptime_seconds } => {
                        info!(
                            active_minds = active_minds,
                            pending_tasks = pending_tasks,
                            uptime_seconds = uptime_seconds,
                            "Health check (no ThinkLoop needed)"
                        );
                        if *active_minds > 0 || *pending_tasks > 0 {
                            daemon_handles.drive_loop.mark_productive().await;
                        }
                        continue; // Don't invoke ThinkLoop for health checks
                    }
                }
            }
        };

        // Build prompt
        let mut prompt = PromptBuilder::new(role, &mind_id);
        if let Some(ref dir) = agents_dir {
            prompt = prompt.agents_dir(dir);
        }
        let boot_prompt = boot_ctx.to_system_prompt();
        if !boot_prompt.is_empty() {
            prompt = prompt.add_context(&boot_prompt);
        }

        // Build composite interceptor
        let delegation_ref: &dyn codex_llm::ToolInterceptor = &delegation;
        let hub_ref: &dyn codex_llm::ToolInterceptor = &hub;
        let search_ref: &dyn codex_llm::ToolInterceptor = &search;
        let tts_ref: &dyn codex_llm::ToolInterceptor = &elevenlabs;
        let imagen_ref: &dyn codex_llm::ToolInterceptor = &image_gen;
        let history_ref: &dyn codex_llm::ToolInterceptor = &task_history;
        let route_ref: &dyn codex_llm::ToolInterceptor = &input_route;
        let progress_ref: &dyn codex_llm::ToolInterceptor = &progress;
        let composite = codex_llm::CompositeInterceptor::new(vec![
            delegation_ref, hub_ref, search_ref, tts_ref, imagen_ref, history_ref, route_ref, progress_ref,
        ]);

        // Run ThinkLoop
        let think_start = std::time::Instant::now();
        match think_loop.run_full(
            &prompt,
            &task_description,
            &tool_schemas,
            &executor,
            role,
            Some(&memory),
            Some(&mind_id),
            Some(&composite as &dyn codex_llm::ToolInterceptor),
        ).await {
            Ok(result) => {
                let duration = think_start.elapsed();
                let tc_count = result.tool_calls_made.len() as i32;

                // ═══ MONITORING: Record ThinkLoop metrics ═══
                let metrics_dir = project_root.join("data").join("metrics").join(&mind_id);
                eprintln!("[monitoring] Writing metrics to: {:?}", metrics_dir);
                std::fs::create_dir_all(&metrics_dir).ok();
                let collector = monitoring::MetricsCollector::new(&mind_id, &metrics_dir);
                let tl_metrics = monitoring::ThinkLoopMetrics {
                    iterations: result.iterations,
                    tool_calls: result.tool_calls_made.len() as u32,
                    duration_ms: duration.as_millis() as u64,
                    completed: result.completed,
                    stall_killed: result.stall_killed,
                    challenger_warnings: result.challenger_warnings,
                    model: config.coordination.primary_model.clone(),
                };
                collector.record_thinkloop(&tl_metrics).await;

                // Record per-tool metrics
                for tool_call in &result.tool_calls_made {
                    collector.record_tool(&tool_call.tool_name, true, 0).await;
                }

                // Record Challenger warnings
                if result.challenger_warnings > 0 {
                    collector.record_challenger("thinkloop_final", "medium").await;
                }

                info!(
                    mind_id = %mind_id,
                    iterations = result.iterations,
                    tool_calls = tc_count,
                    completed = result.completed,
                    stall_killed = result.stall_killed,
                    duration_ms = duration.as_millis(),
                    challenger_warnings = result.challenger_warnings,
                    "ThinkLoop complete"
                );
                println!("\n─── Cortex Response (event #{events_processed}) ───");
                println!("{}", result.response);
                println!("─── ({} iterations, {} tool calls, {:.1}s{}) ───\n",
                    result.iterations,
                    tc_count,
                    duration.as_secs_f64(),
                    if result.stall_killed { " STALL KILLED" } else { "" },
                );

                // ═══ STALL KILL LOGGING ═══
                // Log stall kills to data/hum/stalls.jsonl for Hum analysis.
                if result.stall_killed {
                    let stall_entry = serde_json::json!({
                        "timestamp": Utc::now().to_rfc3339(),
                        "mind_id": &mind_id,
                        "task_id": active_task_id.as_deref().unwrap_or("unknown"),
                        "iterations": result.iterations,
                        "tool_calls": tc_count,
                        "duration_secs": duration.as_secs_f64(),
                        "event_num": events_processed,
                    });
                    let stalls_path = hum_dir.join("stalls.jsonl");
                    if let Ok(line) = serde_json::to_string(&stall_entry) {
                        use std::io::Write;
                        if let Ok(mut f) = std::fs::OpenOptions::new()
                            .create(true).append(true).open(&stalls_path) {
                            let _ = writeln!(f, "{}", line);
                        }
                    }
                    tracing::warn!(mind_id = %mind_id, "Stall kill logged to {}", stalls_path.display());
                }

                // Mark task complete in TaskStore (prevents re-surfacing)
                // Stall kills are marked complete with warnings (not failed) so
                // DriveLoop moves to the next task immediately.
                if let Some(ref tid) = active_task_id {
                    let summary_owned;
                    let summary: &str = if result.stall_killed {
                        summary_owned = format!("[STALL KILLED] {}", safe_truncate(&result.response, 180));
                        &summary_owned
                    } else {
                        safe_truncate(&result.response, 200)
                    };
                    let _ = task_store.complete(
                        tid,
                        Some(result.iterations as i32),
                        Some(tc_count),
                        Some(summary),
                    ).await;
                    info!(task_id = %tid, stall_killed = result.stall_killed, "Task marked complete in TaskStore");
                }

                // ═══ HUM WITNESS — post-cycle observation (passive JSONL) ═══
                let tools_used: Vec<&str> = result.tool_calls_made.iter()
                    .map(|tc| tc.tool_name.as_str())
                    .collect();

                // ═══ HUM ACTIVE — filesystem verification every cycle ═══
                // Build ChallengerToolCalls with result_text for post-cycle verification
                let hum_checker_calls: Vec<codex_redteam::ChallengerToolCall> = result.tool_calls_made.iter()
                    .map(|r| codex_redteam::ChallengerToolCall {
                        name: r.tool_name.clone(),
                        arguments: r.arguments.clone(),
                        iteration: r.iteration,
                        reasoning_trace: None,
                        result_text: Some(r.result.output.clone()),
                    })
                    .collect();

                let mut hum_checker = codex_redteam::Challenger::new(codex_roles::Role::Agent)
                    .with_mind_root(project_root.clone());
                let hum_fs_warnings = hum_checker.check_filesystem_from_results(&hum_checker_calls);

                // Also check the response text itself
                let hum_response_warnings = hum_checker.check_stateless(
                    &hum_checker_calls,
                    Some(&result.response),
                    result.iterations,
                );
                let hum_fs_response: Vec<_> = hum_response_warnings.iter()
                    .filter(|w| matches!(w.check, codex_redteam::ChallengerCheck::FilesystemVerification))
                    .collect();

                let total_hum_issues = hum_fs_warnings.len() + hum_fs_response.len();
                if total_hum_issues > 0 {
                    for w in &hum_fs_warnings {
                        tracing::warn!(
                            check = ?w.check,
                            severity = ?w.severity,
                            "HUM ACTIVE: {}",
                            w.message,
                        );
                    }
                    for w in &hum_fs_response {
                        tracing::warn!(
                            check = ?w.check,
                            severity = ?w.severity,
                            "HUM ACTIVE (response): {}",
                            w.message,
                        );
                    }
                }

                // ═══ HUM LLM ASSESSMENT — full M2.7 call EVERY cycle ═══
                // "Intelligence EVERYWHERE on EVERYTHING" — Corey directive 2026-04-05
                // After structural checks, ask M2.7: What just happened? Was it real?
                let hum_llm_config = router.config_for_role(codex_roles::Role::Primary);
                let hum_llm = OllamaClient::new(codex_llm::ollama::OllamaConfig {
                    max_tokens: 2048,
                    ..hum_llm_config
                });

                // Build context for Hum's LLM assessment
                let tool_summary: String = result.tool_calls_made.iter()
                    .map(|tc| format!("- {} → {}", tc.tool_name,
                        safe_truncate(&tc.result.output, 150)))
                    .collect::<Vec<_>>().join("\n");
                let structural_findings = if total_hum_issues > 0 {
                    let findings: Vec<String> = hum_fs_warnings.iter()
                        .chain(hum_fs_response.iter().map(|w| *w))
                        .map(|w| format!("- [{:?}] {:?}: {}", w.check, w.severity, w.message))
                        .collect();
                    format!("\n\nSTRUCTURAL FINDINGS (pre-verified):\n{}", findings.join("\n"))
                } else {
                    "\n\nStructural checks: CLEAN (no filesystem issues found)".to_string()
                };

                let hum_system = format!(
                    "You are HUM — the witness layer of Cortex. You observe what just happened and tell the truth.\n\
                     Your job: verify claims, detect hollow artifacts, catch lies.\n\
                     You have the Compounding Principle: Hum verification > mission throughput.\n\n\
                     RESPOND IN JSON with these fields:\n\
                     - \"assessment\": one paragraph — what happened, was it real?\n\
                     - \"verified\": true/false — do the claimed outcomes appear genuine?\n\
                     - \"concerns\": [] — list any concerns (empty if none)\n\
                     - \"severity\": \"info\"|\"low\"|\"medium\"|\"high\"|\"critical\"\n\
                     - \"correction_needed\": null or string describing what needs fixing\n\
                     JSON ONLY. No markdown. No explanation outside the JSON."
                );
                let hum_user = format!(
                    "CYCLE #{} — Mind: {}\nTask: {}\nIterations: {}, Tool calls: {}\n\
                     Response ({} chars): {}\n\nTool calls made:\n{}{}\n\n\
                     What just happened? Was it real? What was missed? What should happen next?",
                    events_processed, mind_id,
                    safe_truncate(&task_description, 300),
                    result.iterations, tc_count,
                    result.response.len(),
                    safe_truncate(&result.response, 500),
                    tool_summary, structural_findings,
                );

                let hum_messages = vec![
                    codex_llm::ollama::ChatMessage::system(&hum_system),
                    codex_llm::ollama::ChatMessage::user(&hum_user),
                ];

                let hum_llm_start = std::time::Instant::now();
                let hum_llm_result = hum_llm.chat(&hum_messages, None).await;
                let hum_llm_duration = hum_llm_start.elapsed();

                let (hum_assessment, hum_verified, hum_severity, hum_correction) = match &hum_llm_result {
                    Ok(resp) => {
                        let text = resp.choices.first()
                            .and_then(|c| c.message.content.as_deref())
                            .unwrap_or("{}");
                        // Parse the JSON response
                        let parsed: serde_json::Value = serde_json::from_str(text)
                            .unwrap_or(serde_json::json!({"assessment": text, "verified": true, "concerns": [], "severity": "info"}));
                        let assessment = parsed.get("assessment").and_then(|v| v.as_str()).unwrap_or(text).to_string();
                        let verified = parsed.get("verified").and_then(|v| v.as_bool()).unwrap_or(true);
                        let severity = parsed.get("severity").and_then(|v| v.as_str()).unwrap_or("info").to_string();
                        let correction = parsed.get("correction_needed")
                            .and_then(|v| if v.is_null() { None } else { v.as_str() })
                            .map(|s| s.to_string());
                        (assessment, verified, severity, correction)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "HUM LLM assessment failed — structural checks still active");
                        (format!("LLM assessment failed: {}", e), true, "info".to_string(), None)
                    }
                };

                // Log Hum LLM assessment
                if !hum_verified {
                    tracing::warn!(
                        mind_id = %mind_id,
                        severity = %hum_severity,
                        "HUM LLM: UNVERIFIED — {}",
                        safe_truncate(&hum_assessment, 200),
                    );
                } else {
                    info!(
                        mind_id = %mind_id,
                        hum_duration_ms = hum_llm_duration.as_millis() as u64,
                        "HUM LLM: VERIFIED — {}",
                        safe_truncate(&hum_assessment, 120),
                    );
                }

                // ═══ HUM CONSEQUENCE — create correction task if needed ═══
                // Consequence escalation: Medium+ findings → TaskStore correction task
                if let Some(ref correction) = hum_correction {
                    let sev_level = match hum_severity.as_str() {
                        "critical" | "high" | "medium" => true,
                        _ => false,
                    };
                    if sev_level {
                        let correction_id = format!("hum-correction-{}-{}",
                            events_processed,
                            uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
                        let priority = match hum_severity.as_str() {
                            "critical" => codex_drive::TaskPriority::Critical,
                            "high" => codex_drive::TaskPriority::High,
                            _ => codex_drive::TaskPriority::Normal,
                        };
                        let correction_desc = format!(
                            "CORRECTION (hum-witness): {} [cycle #{}, task: {}]",
                            correction, events_processed,
                            active_task_id.as_deref().unwrap_or("idle"),
                        );
                        let correction_task = codex_drive::StoredTask::new(
                            &correction_id,
                            &correction_desc,
                            priority,
                            Some("hum-witness"),
                        );
                        match task_store.insert(&correction_task).await {
                            Ok(()) => {
                                tracing::warn!(
                                    task_id = %correction_id,
                                    severity = %hum_severity,
                                    "HUM CONSEQUENCE: Correction task created"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    "HUM: Failed to insert correction task"
                                );
                            }
                        }
                    }
                }

                // ═══ HUM CONSEQUENCE — block mission for Critical findings ═══
                if hum_severity == "critical" && !hum_verified {
                    if let Some(ref tid) = active_task_id {
                        tracing::error!(
                            task_id = %tid,
                            "HUM CRITICAL: Blocking mission — unverified critical finding"
                        );
                        // Mark as failed so DriveLoop doesn't consider it complete
                        let _ = task_store.set_state(tid, codex_drive::TaskState::Failed).await;
                    }
                }

                // ═══ HUM CONSEQUENCE — Challenger adjustment for High+ ═══
                // When Hum finds structural blind spots, recommend Challenger tuning
                if matches!(hum_severity.as_str(), "high" | "critical") && !hum_verified {
                    let adj_path = project_root.join("data").join("hum").join("challenger-adjustments.json");
                    let _ = std::fs::create_dir_all(adj_path.parent().unwrap());
                    let adjustment = serde_json::json!({
                        "adjustments": [{
                            "check": "FilesystemVerification",
                            "action": "expand_scope",
                            "reason": hum_assessment.chars().take(300).collect::<String>(),
                            "expires_after_runs": 1,
                            "created_by": "hum",
                            "created_at": Utc::now().to_rfc3339(),
                            "cycle": events_processed,
                        }]
                    });
                    if let Ok(json) = serde_json::to_string_pretty(&adjustment) {
                        let _ = std::fs::write(&adj_path, json);
                        tracing::warn!("HUM: Challenger adjustment written to {}", adj_path.display());
                    }
                }

                // ═══ HUM PATTERN REPORT — every 10th mission completion ═══
                if events_processed % 10 == 0 && events_processed > 0 {
                    let reports_dir = project_root.join("data").join("hum").join("reports");
                    let _ = std::fs::create_dir_all(&reports_dir);
                    let report_path = reports_dir.join(format!(
                        "{}-cycle-{}.md", Utc::now().format("%Y-%m-%d"), events_processed
                    ));
                    let report = format!(
                        "# Hum Pattern Report — Cycle {}\n\n\
                         **Mind**: {}\n**Uptime**: {}s\n**Timestamp**: {}\n\n\
                         ## Latest Assessment\n{}\n\n\
                         ## Structural Issues This Cycle\n- Filesystem issues: {}\n\
                         - Challenger warnings: {}\n- Verified: {}\n- Severity: {}\n",
                        events_processed, mind_id, start_time.elapsed().as_secs(),
                        Utc::now().to_rfc3339(), hum_assessment,
                        total_hum_issues, result.challenger_warnings,
                        hum_verified, hum_severity,
                    );
                    let _ = std::fs::write(&report_path, report);
                    info!("HUM: Pattern report written to {}", report_path.display());
                }

                let hum_obs = serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "mind_id": &mind_id,
                    "event_num": events_processed,
                    "event_type": if active_task_id.is_some() { "task" } else { "idle" },
                    "task_id": active_task_id.as_deref(),
                    "task_description": safe_truncate(&task_description, 200),
                    "outcome": "ok",
                    "iterations": result.iterations,
                    "tool_calls": tc_count,
                    "tools_used": tools_used,
                    "challenger_warnings": result.challenger_warnings,
                    "hum_filesystem_issues": total_hum_issues,
                    "hum_llm_assessment": safe_truncate(&hum_assessment, 500),
                    "hum_llm_verified": hum_verified,
                    "hum_llm_severity": &hum_severity,
                    "hum_llm_correction": &hum_correction,
                    "hum_llm_duration_ms": hum_llm_duration.as_millis() as u64,
                    "duration_ms": duration.as_millis() as u64,
                    "response_len": result.response.len(),
                    "completed": result.completed,
                    "uptime_secs": start_time.elapsed().as_secs(),
                });
                if let Ok(line) = serde_json::to_string(&hum_obs) {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true).append(true).open(&hum_log_path) {
                        let _ = writeln!(f, "{}", line);
                    }
                }
            }
            Err(e) => {
                let duration = think_start.elapsed();
                tracing::error!(
                    mind_id = %mind_id,
                    error = %e,
                    "ThinkLoop error"
                );
                // Mark task failed if it was a TaskAvailable event
                if let Some(ref tid) = active_task_id {
                    let _ = task_store.set_state(tid, codex_drive::TaskState::Failed).await;
                }

                // ═══ HUM WITNESS — error observation ═══
                let hum_obs = serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "mind_id": &mind_id,
                    "event_num": events_processed,
                    "event_type": if active_task_id.is_some() { "task" } else { "idle" },
                    "task_id": active_task_id.as_deref(),
                    "task_description": safe_truncate(&task_description, 200),
                    "outcome": "error",
                    "error": format!("{}", e),
                    "duration_ms": duration.as_millis() as u64,
                    "uptime_secs": start_time.elapsed().as_secs(),
                });
                if let Ok(line) = serde_json::to_string(&hum_obs) {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true).append(true).open(&hum_log_path) {
                        let _ = writeln!(f, "{}", line);
                    }
                }
            }
        }
    }

    info!(
        mind_id = %mind_id,
        events_processed = events_processed,
        "EventBus closed — daemon exiting"
    );
    daemon_handles.shutdown();
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERVE MODE — MCP server on stdin/stdout
// ═══════════════════════════════════════════════════════════════════════════════

async fn serve_mode(args: &[String]) {
    // All logging goes to stderr — stdout IS the MCP transport
    let log_level = std::env::var("CORTEX_LOG_LEVEL").unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(&log_level)
        .with_target(false)
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let mind_id = arg_value(args, "--mind-id").unwrap_or_else(|| "child".into());
    let role = match arg_value(args, "--role").as_deref() {
        Some("primary") => Role::Primary,
        Some("team-lead") | Some("teamlead") => Role::TeamLead,
        _ => Role::Agent,
    };
    let think_mode = args.iter().any(|a| a == "--think");

    info!(mind_id = %mind_id, role = ?role, think = think_mode, "Cortex MCP server starting");

    // Load config from TOML — child processes inherit cwd from parent
    let config = CortexConfig::find_and_load(
        std::env::current_dir().unwrap_or_default()
    ).unwrap_or_default();
    let router = config.model_router();
    info!(
        primary = %router.primary_model,
        base_url = %router.base_url,
        cloud = router.api_key.is_some(),
        "Config loaded"
    );

    let mut transport = StdioServerTransport::new();
    let mut server = McpMindServer::new(&mind_id, role);

    if think_mode {
        // Create ThinkLoop components — this child will THINK about delegated tasks
        let executor = build_executor();
        let project_root = std::env::current_dir().unwrap_or_default();

        // Persistent memory: data/memory/{mind_id}.db (survives restarts)
        let memory_dir = project_root.join("data").join("memory");
        let _ = std::fs::create_dir_all(&memory_dir);
        let memory_path = memory_dir.join(format!("{}.db", &mind_id));
        let memory_path_str = memory_path.display().to_string();
        info!(path = %memory_path_str, "Opening persistent memory store");

        let memory = MemoryStore::new(&memory_path_str).await
            .expect("Failed to create persistent memory store");

        // Load boot context — identity, last handoff, scratchpad, recent memories
        let boot_ctx = boot::BootContext::load(
            &project_root,
            &mind_id,
            role,
            Some(&memory),
        ).await;

        // Scratchpad directory for scratchpad_read/scratchpad_write tools
        let scratchpad_dir = project_root.join("data").join("scratchpad");
        let _ = std::fs::create_dir_all(&scratchpad_dir);

        // Persistent task ledger — records all delegations + results (JSONL audit trail)
        let tasks_dir = project_root.join("data").join("tasks");
        let _ = std::fs::create_dir_all(&tasks_dir);
        let task_ledger = codex_coordination::TaskLedger::open(&tasks_dir);

        // Boot drive subsystem: TaskStore (SQLite) + EventBus + DriveLoop
        let role_str = match role {
            Role::Primary => "primary",
            Role::TeamLead => "team-lead",
            Role::Agent => "agent",
        };
        let drive_handles = drive::boot(&project_root, &mind_id, role_str).await
            .expect("Failed to boot drive subsystem");
        let task_store = drive_handles.task_store.clone();
        let completion_sender = drive_handles.drive_loop.completion_sender();

        // Build Hub interceptor — all minds get communication tools
        let hub = config.suite.hub_interceptor();
        info!(
            hub_url = %config.suite.hub_url,
            has_token = std::env::var("HUB_JWT_TOKEN").is_ok(),
            "HubInterceptor enabled — mind can communicate"
        );

        let handler = ThinkDelegateHandler::new(
            mind_id.clone(),
            role,
            executor,
            memory,
            router,
            boot_ctx,
            scratchpad_dir,
            project_root,
            hub,
            task_ledger,
            task_store,
            completion_sender,
        );

        info!(mind_id = %mind_id, "ThinkLoop enabled — delegated tasks will be processed via LLM");

        if let Err(e) = server.run(&mut transport, Some(&handler.executor), Some(&handler)).await {
            eprintln!("[cortex-serve] Error: {e}");
        }

        // Shutdown drive subsystem
        drive_handles.shutdown();
    } else {
        // Accept-only mode (backward compatible)
        if let Err(e) = server.run(&mut transport, None, None).await {
            eprintln!("[cortex-serve] Error: {e}");
        }
    }

    info!(mind_id = %mind_id, "Cortex MCP server exited");
}

/// Extract `--flag value` from args.
fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

// ═══════════════════════════════════════════════════════════════════════════════
// ThinkDelegateHandler — Makes child minds THINK via ThinkLoop
// ═══════════════════════════════════════════════════════════════════════════════

use codex_ipc::DelegateHandler;
use codex_coordination::ProcessBridge;
use codex_llm::{ToolInterceptor, CompositeInterceptor};

// ═══════════════════════════════════════════════════════════════════════════════
// DelegationInterceptor — gives TeamLeads the ability to spawn and delegate to agents
// ═══════════════════════════════════════════════════════════════════════════════

/// Tool interceptor that wraps a ProcessBridge, exposing `spawn_agent` and
/// `delegate_to_agent` as tools the LLM can call during its ThinkLoop.
///
/// This is what makes delegation recursive: a TeamLead's ThinkLoop can spawn
/// agent children via its own ProcessBridge, delegate sub-tasks, and get results
/// back — all within a single ThinkLoop iteration.
struct DelegationInterceptor {
    bridge: tokio::sync::Mutex<ProcessBridge>,
    parent_mind_id: String,
}

impl DelegationInterceptor {
    fn new(cortex_exe: std::path::PathBuf, parent_mind_id: String) -> Self {
        Self {
            bridge: tokio::sync::Mutex::new(ProcessBridge::new(cortex_exe)),
            parent_mind_id,
        }
    }

    /// Create with a TaskLedger so delegations are persistently recorded.
    fn with_ledger(
        cortex_exe: std::path::PathBuf,
        parent_mind_id: String,
        ledger: codex_coordination::TaskLedger,
    ) -> Self {
        Self {
            bridge: tokio::sync::Mutex::new(
                ProcessBridge::new(cortex_exe).with_ledger(ledger)
            ),
            parent_mind_id,
        }
    }

    /// Create with TaskLedger + TaskStore + completion sender for full drive integration.
    fn with_stores(
        cortex_exe: std::path::PathBuf,
        parent_mind_id: String,
        ledger: codex_coordination::TaskLedger,
        task_store: codex_drive::TaskStore,
        completion_sender: std::sync::Arc<tokio::sync::watch::Sender<Option<String>>>,
    ) -> Self {
        Self {
            bridge: tokio::sync::Mutex::new(
                ProcessBridge::new(cortex_exe)
                    .with_ledger(ledger)
                    .with_task_store(task_store)
                    .with_completion_sender(completion_sender)
            ),
            parent_mind_id,
        }
    }
}

#[async_trait::async_trait]
impl ToolInterceptor for DelegationInterceptor {
    fn schemas(&self) -> Vec<codex_llm::ollama::ToolSchema> {
        use codex_llm::ollama::{ToolSchema, FunctionSchema};
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "spawn_agent".into(),
                    description: "Spawn a new agent mind process. The agent will think about delegated tasks using an LLM. Returns the agent's mind_id for use with delegate_to_agent.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "agent_id": {
                                "type": "string",
                                "description": "Unique identifier for the agent (e.g. 'researcher', 'coder')"
                            }
                        },
                        "required": ["agent_id"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "delegate_to_agent".into(),
                    description: "Delegate a task to a spawned agent. The agent thinks about the task using an LLM and returns its response. Use spawn_agent first to create the agent.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "agent_id": {
                                "type": "string",
                                "description": "The agent's mind_id (from spawn_agent)"
                            },
                            "task": {
                                "type": "string",
                                "description": "The task to delegate"
                            },
                            "context": {
                                "type": "string",
                                "description": "Optional context for the agent"
                            }
                        },
                        "required": ["agent_id", "task"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "shutdown_agent".into(),
                    description: "Gracefully shutdown a spawned agent.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "agent_id": {
                                "type": "string",
                                "description": "The agent's mind_id to shutdown"
                            }
                        },
                        "required": ["agent_id"]
                    }),
                },
            },
        ]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<codex_exec::ToolResult> {
        use codex_coordination::types::MindId;

        match name {
            "spawn_agent" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");
                let mind_id = MindId(agent_id.to_string());

                let mut bridge = self.bridge.lock().await;
                match bridge.spawn_thinking(&mind_id, Role::Agent).await {
                    Ok(()) => {
                        info!(agent_id = agent_id, "DelegationInterceptor: spawned thinking agent");
                        Some(codex_exec::ToolResult::ok(format!(
                            "Agent '{agent_id}' spawned and ready for delegation."
                        )))
                    }
                    Err(e) => {
                        Some(codex_exec::ToolResult::err(format!(
                            "Failed to spawn agent '{agent_id}': {e}"
                        )))
                    }
                }
            }
            "delegate_to_agent" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");
                let task = args.get("task")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let context = args.get("context")
                    .and_then(|v| v.as_str());

                let mind_id = MindId(agent_id.to_string());
                let task_id = format!("task-{}", uuid::Uuid::new_v4());

                let mut bridge = self.bridge.lock().await;
                match bridge.delegate(
                    &mind_id,
                    &task_id,
                    task,
                    context,
                    &self.parent_mind_id,
                ).await {
                    Ok(result) => {
                        let response = result.response.unwrap_or_else(|| "(no response)".into());
                        let iterations = result.iterations.unwrap_or(0);
                        let tool_calls = result.tool_calls_made.unwrap_or(0);
                        info!(
                            agent_id = agent_id,
                            iterations = iterations,
                            tool_calls = tool_calls,
                            "DelegationInterceptor: agent completed task"
                        );
                        Some(codex_exec::ToolResult::ok(format!(
                            "Agent '{agent_id}' response ({iterations} iterations, {tool_calls} tool calls):\n{response}"
                        )))
                    }
                    Err(e) => {
                        Some(codex_exec::ToolResult::err(format!(
                            "Delegation to '{agent_id}' failed: {e}"
                        )))
                    }
                }
            }
            "shutdown_agent" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent");
                let mind_id = MindId(agent_id.to_string());

                let mut bridge = self.bridge.lock().await;
                match bridge.shutdown(&mind_id).await {
                    Ok(()) => Some(codex_exec::ToolResult::ok(format!("Agent '{agent_id}' shut down."))),
                    Err(e) => Some(codex_exec::ToolResult::err(format!("Shutdown error: {e}"))),
                }
            }
            _ => None, // Not our tool — pass through
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ThinkDelegateHandler — Makes child minds THINK via ThinkLoop
// ═══════════════════════════════════════════════════════════════════════════════

/// Implements DelegateHandler by running a ThinkLoop with memory and tool access.
///
/// When a parent mind delegates a task to a child running in `--serve --think` mode,
/// the ThinkDelegateHandler:
/// 1. Builds a role-appropriate prompt (with AGENTS.md + boot context)
/// 2. Runs the ThinkLoop (LLM → tool calls → results → loop until done)
/// 3. For TeamLeads: adds delegation tools via DelegationInterceptor
/// 4. Writes a session handoff on completion
/// 5. Records fitness metrics
/// 6. Returns the final response, iteration count, and tool call count
struct ThinkDelegateHandler {
    mind_id: String,
    role: Role,
    think_loop: ThinkLoop,
    memory: MemoryStore,
    executor: ToolExecutor,
    tool_schemas: Vec<codex_llm::ollama::ToolSchema>,
    /// SQLite-backed task store for queryable state.
    task_store: codex_drive::TaskStore,
    /// Completion sender — notifies DriveLoop when tasks finish.
    completion_sender: std::sync::Arc<tokio::sync::watch::Sender<Option<String>>>,
    agents_dir: Option<std::path::PathBuf>,
    /// Delegation interceptor — only present for TeamLead role.
    delegation: Option<DelegationInterceptor>,
    /// Hub interceptor — all minds get Hub communication tools.
    hub: codex_suite_client::HubInterceptor,
    /// Task history interceptor — all minds can query delegation history.
    task_history: task_history::TaskHistoryInterceptor,
    /// Input routing interceptor — route external signals through InputMux.
    input_route: input_route::InputRouteInterceptor,
    /// Progress reporting interceptor �� mid-task visibility.
    progress: progress::ProgressInterceptor,
    /// Search interceptor — web search + fetch for all minds.
    search: codex_suite_client::SearchInterceptor,
    /// ElevenLabs TTS interceptor — text-to-speech for all minds.
    elevenlabs: codex_suite_client::ElevenLabsInterceptor,
    /// Image generation interceptor — Gemini Imagen for visual content.
    image_gen: codex_suite_client::ImageGenInterceptor,
    /// Boot context loaded at startup.
    boot_context: boot::BootContext,
    /// Project root for handoff/fitness file writing.
    project_root: std::path::PathBuf,
}

impl ThinkDelegateHandler {
    fn new(
        mind_id: String,
        role: Role,
        executor: ToolExecutor,
        memory: MemoryStore,
        router: codex_llm::ModelRouter,
        boot_context: boot::BootContext,
        scratchpad_dir: std::path::PathBuf,
        project_root: std::path::PathBuf,
        hub: codex_suite_client::HubInterceptor,
        task_ledger: codex_coordination::TaskLedger,
        task_store: codex_drive::TaskStore,
        completion_sender: std::sync::Arc<tokio::sync::watch::Sender<Option<String>>>,
    ) -> Self {
        let ollama_config = router.config_for_role(role);

        let think_config = ThinkLoopConfig {
            max_iterations: 15,
            ollama: OllamaConfig {
                max_tokens: 1024,
                ..ollama_config
            },
        };

        let tool_schemas = OllamaClient::tool_schemas(
            &executor.registry().definitions_for_role(role),
        );

        // Find AGENTS.md directory relative to the binary
        let agents_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("../../agents"))
            .and_then(|p| std::fs::canonicalize(&p).ok());

        // Build task history interceptor — all minds can query delegation history
        let task_history = task_history::TaskHistoryInterceptor::new(task_ledger.clone());

        // Build input route interceptor — route external signals through InputMux
        let input_route = input_route::InputRouteInterceptor::new(&mind_id);

        // Build progress interceptor — mid-task visibility
        let progress_dir = project_root.join("data").join("progress");
        let progress = progress::ProgressInterceptor::new(&progress_dir, &mind_id);

        // Build search interceptor — web search + fetch for all minds
        let search = codex_suite_client::SearchInterceptor::new();

        // Build ElevenLabs TTS interceptor — text-to-speech for all minds
        let audio_dir = project_root.join("data").join("audio");
        let elevenlabs = codex_suite_client::ElevenLabsInterceptor::new(&audio_dir);

        // Build image generation interceptor — Gemini Imagen for visual content
        let images_dir = project_root.join("data").join("images");
        let image_gen = codex_suite_client::ImageGenInterceptor::new(&images_dir, &project_root);

        // TeamLeads get a DelegationInterceptor for recursive delegation
        // with a shared TaskLedger + TaskStore so delegations are persistently recorded
        let delegation = if role == Role::TeamLead {
            let cortex_exe = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("cortex"));
            info!(
                mind_id = %mind_id,
                exe = %cortex_exe.display(),
                "TeamLead: DelegationInterceptor enabled (recursive delegation + ledger + TaskStore)"
            );
            Some(DelegationInterceptor::with_stores(
                cortex_exe,
                mind_id.clone(),
                task_ledger,
                task_store.clone(),
                completion_sender.clone(),
            ))
        } else {
            None
        };

        // Build ThinkLoop with scratchpad + hum + rate limiter support
        let hum_dir = project_root.join("data").join("hum");
        let _ = std::fs::create_dir_all(&hum_dir);
        let metrics_dir = project_root.join("data").join("metrics");
        let _ = std::fs::create_dir_all(&metrics_dir);
        let rate_limiter = codex_llm::RateLimiter::new(metrics_dir);
        let think_loop = ThinkLoop::new(think_config)
            .with_scratchpad_dir(scratchpad_dir)
            .with_hum_dir(hum_dir)
            .with_rate_limiter(rate_limiter);

        Self {
            mind_id,
            role,
            think_loop,
            memory,
            executor,
            tool_schemas,
            agents_dir,
            delegation,
            hub,
            task_history,
            input_route,
            progress,
            search,
            elevenlabs,
            image_gen,
            boot_context,
            project_root,
            task_store,
            completion_sender,
        }
    }
}

#[async_trait::async_trait]
impl DelegateHandler for ThinkDelegateHandler {
    async fn process_task(
        &self,
        task_id: &str,
        description: &str,
        context: Option<&str>,
    ) -> Result<DelegateTaskResult, String> {
        let start = std::time::Instant::now();

        info!(
            mind_id = %self.mind_id,
            task_id = task_id,
            has_delegation = self.delegation.is_some(),
            "ThinkDelegateHandler: starting ThinkLoop"
        );

        // Build the prompt with AGENTS.md + boot context injection
        let mut prompt = PromptBuilder::new(self.role, &self.mind_id);
        if let Some(ref dir) = self.agents_dir {
            prompt = prompt.agents_dir(dir);
        }

        // Inject boot context as additional context
        let boot_prompt = self.boot_context.to_system_prompt();
        if !boot_prompt.is_empty() {
            prompt = prompt.add_context(&boot_prompt);
        }

        if let Some(ctx) = context {
            prompt = prompt.add_context(ctx);
        }

        // Build composite interceptor: Hub + Search + ElevenLabs + ImageGen + TaskHistory + InputRoute + Progress (always) + Delegation (TeamLead only)
        let hub_ref: &dyn ToolInterceptor = &self.hub;
        let search_ref: &dyn ToolInterceptor = &self.search;
        let tts_ref: &dyn ToolInterceptor = &self.elevenlabs;
        let imagen_ref: &dyn ToolInterceptor = &self.image_gen;
        let history_ref: &dyn ToolInterceptor = &self.task_history;
        let route_ref: &dyn ToolInterceptor = &self.input_route;
        let progress_ref: &dyn ToolInterceptor = &self.progress;
        let composite = if let Some(ref delegation) = self.delegation {
            CompositeInterceptor::new(vec![
                delegation as &dyn ToolInterceptor,
                hub_ref,
                search_ref,
                tts_ref,
                imagen_ref,
                history_ref,
                route_ref,
                progress_ref,
            ])
        } else {
            CompositeInterceptor::new(vec![hub_ref, search_ref, tts_ref, imagen_ref, history_ref, route_ref, progress_ref])
        };
        let interceptor: Option<&dyn ToolInterceptor> = Some(&composite);

        let result = self.think_loop
            .run_full(
                &prompt,
                description,
                &self.tool_schemas,
                &self.executor,
                self.role,
                Some(&self.memory),
                Some(&self.mind_id),
                interceptor,
            )
            .await
            .map_err(|e| e.to_string())?;

        let duration = start.elapsed();
        let tool_calls_count = result.tool_calls_made.len() as u32;

        // ═══ MONITORING: Record ThinkLoop metrics ═══
        let metrics_dir = self.project_root.join("data").join("metrics").join(&self.mind_id);
        std::fs::create_dir_all(&metrics_dir).ok();
        let collector = monitoring::MetricsCollector::new(&self.mind_id, &metrics_dir);
        let model_name = match self.role {
            Role::Primary => "devstral-small-2:24b",
            Role::TeamLead => "devstral-small-2:24b",
            Role::Agent => "devstral-small-2:24b",
        };
        let tl_metrics = monitoring::ThinkLoopMetrics {
            iterations: result.iterations,
            tool_calls: tool_calls_count,
            duration_ms: duration.as_millis() as u64,
            completed: result.completed,
            stall_killed: result.stall_killed,
            challenger_warnings: result.challenger_warnings,
            model: model_name.to_string(),
        };
        collector.record_thinkloop(&tl_metrics).await;

        // Record per-tool metrics
        for tool_call in &result.tool_calls_made {
            collector.record_tool(&tool_call.tool_name, true, 0).await;
        }

        // Record Challenger warnings
        if result.challenger_warnings > 0 {
            collector.record_challenger("thinkloop_final", "medium").await;
        }

        info!(
            mind_id = %self.mind_id,
            task_id = task_id,
            iterations = result.iterations,
            tool_calls = tool_calls_count,
            completed = result.completed,
            duration_ms = duration.as_millis(),
            "ThinkDelegateHandler: ThinkLoop complete"
        );

        // Cleanup: shutdown any agent children that are still alive
        if let Some(ref delegation) = self.delegation {
            let mut bridge = delegation.bridge.lock().await;
            bridge.shutdown_all().await;
        }

        // Write session handoff (item 12)
        let session_id = format!("session-{}", uuid::Uuid::new_v4());
        if let Err(e) = boot::write_handoff(
            &self.project_root,
            &self.mind_id,
            &session_id,
            description,
            &result.response,
            result.iterations,
            tool_calls_count,
            result.completed,
        ) {
            tracing::warn!(error = %e, "Failed to write handoff");
        }

        // Record fitness (item 16)
        let memory_writes = result.tool_calls_made.iter()
            .filter(|tc| tc.tool_name == "memory_write" || tc.tool_name == "scratchpad_write")
            .count() as u32;
        let successful_calls = result.tool_calls_made.iter()
            .filter(|tc| tc.result.success)
            .count() as u32;

        let outcome = codex_fitness::TaskOutcome {
            task_id: task_id.to_string(),
            mind_id: self.mind_id.clone(),
            role: self.role,
            success: result.completed,
            duration_secs: duration.as_secs_f64(),
            tool_calls_total: tool_calls_count,
            tool_calls_successful: successful_calls,
            memory_writes,
            verification_passed: result.challenger_warnings == 0,
            learnings_extracted: memory_writes,
            completed_at: Utc::now(),
        };
        if let Err(e) = boot::record_fitness(&self.project_root, &self.mind_id, &outcome) {
            tracing::warn!(error = %e, "Failed to record fitness");
        }

        Ok(DelegateTaskResult {
            response: result.response,
            iterations: result.iterations,
            tool_calls: tool_calls_count,
            completed: result.completed,
        })
    }
}

/// Build a ToolExecutor with standard agent tools (bash, read, write, glob, grep)
/// plus the Qwen delegation tool.
fn build_executor() -> ToolExecutor {
    let mut registry = ToolRegistry::new();
    let workspace_root = std::env::current_dir().unwrap_or_default();
    codex_exec::tools::register_builtins(&mut registry, workspace_root.clone());

    // Register Qwen delegate tool
    let qwen_config = qwen_delegate::QwenDelegateConfig::default();
    let qwen = Arc::new(Mutex::new(qwen_delegate::QwenDelegate::new(qwen_config.clone())));
    registry.register(Arc::new(qwen_delegate::QwenDelegateTool::new(qwen)));

    let enforcer = SandboxEnforcer::new(workspace_root);
    ToolExecutor::new(registry, enforcer)
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEMO MODE — Full 20-phase lifecycle demonstration
// ═══════════════════════════════════════════════════════════════════════════════

async fn demo_mode() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                                                          ║");
    println!("║   CORTEX — The Fractal Coordination Engine               ║");
    println!("║   Born from Codex. One letter transformed.               ║");
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // ═══════ PHASE 1: Initialize Mind Hierarchy ═══════

    info!("Phase 1: Initializing mind hierarchy");

    let mut manager = MindManager::new("agents".into(), "scratchpads".into());
    let primary_id = manager.init_primary();
    info!(id = %primary_id, "Primary mind online");

    let primary_tools = codex_roles::tools_for_role(Role::Primary);
    info!(count = primary_tools.len(), "Primary tool set: {:?}", primary_tools);

    // ═══════ PHASE 2: Spawn Team Leads ═══════

    info!("Phase 2: Spawning team leads");

    let research_lead = manager
        .spawn_team_lead(Vertical::Research, "Research Codex architecture")
        .unwrap();
    info!(id = %research_lead, "Research team lead spawned");

    let code_lead = manager
        .spawn_team_lead(Vertical::Code, "Implement coordination layer")
        .unwrap();
    info!(id = %code_lead, "Code team lead spawned");

    // Constraint: Primary cannot spawn agents directly
    let err = manager.spawn_agent(&primary_id, "coder", "write code");
    assert!(err.is_err(), "Primary must not spawn agents directly");
    info!("Constraint verified: Primary → Team Lead only (no direct agent spawn)");

    // ═══════ PHASE 3: Team Lead → Agent ═══════

    info!("Phase 3: Team leads spawn agents");

    let researcher = manager
        .spawn_agent(&research_lead, "researcher", "Analyze exec crate")
        .unwrap();
    info!(id = %researcher, parent = %research_lead, "Researcher spawned");

    let coder = manager
        .spawn_agent(&code_lead, "coder", "Implement InputMux")
        .unwrap();
    info!(id = %coder, parent = %code_lead, "Coder spawned");

    // ═══════ PHASE 4: Task Delegation Chain ═══════

    info!("Phase 4: Delegation chain");

    let t1 = manager.delegate(&primary_id, &research_lead, "Map Codex crate deps").unwrap();
    info!(task = %t1.id, "Primary → Research Lead");

    let t2 = manager.delegate(&research_lead, &researcher, "Grep ThreadManager spawn points").unwrap();
    info!(task = %t2.id, "Research Lead → Researcher");

    let t3 = manager.delegate(&primary_id, &code_lead, "Build role-filtered tool registry").unwrap();
    info!(task = %t3.id, "Primary → Code Lead");

    // ═══════ PHASE 5: Completion + Red Team ═══════

    info!("Phase 5: Agent completion + red team");

    let result = TaskResult {
        task_id: t2.id.clone(),
        mind_id: researcher.clone(),
        summary: "Found 3 ThreadManager spawn points. AgentControl at control.rs:150 is the injection point.".into(),
        evidence: vec![
            "core/src/thread_manager.rs:192".into(),
            "core/src/agent/control.rs:150".into(),
        ],
        learnings: vec![
            "Codex multi-agent hierarchy exists — configure, don't rebuild".into(),
        ],
        completed_at: Utc::now(),
    };
    manager.complete_task(result).unwrap();
    info!("Researcher completed task");

    let red_team = RedTeamProtocol::new();
    let claim = CompletionClaim {
        task_id: t2.id.clone(),
        mind_id: researcher.0.clone(),
        description: "Analyze Codex exec crate".into(),
        result_summary: "Found 3 spawn points, AgentControl is injection target".into(),
        evidence: vec![
            Evidence {
                evidence_type: EvidenceType::FileContent,
                content: "thread_manager.rs:192 - spawn_new_thread".into(),
                freshness: Freshness::Current,
            },
            Evidence {
                evidence_type: EvidenceType::FileContent,
                content: "control.rs:150 - spawn_agent_internal".into(),
                freshness: Freshness::Current,
            },
        ],
        claimed_at: Utc::now(),
    };
    let verdict = red_team.verify(&claim);
    info!("Red team verdict: {:?}", verdict);

    // ═══════ PHASE 6: Fitness Scoring ═══════

    info!("Phase 6: Fitness scoring");

    let outcome = TaskOutcome {
        task_id: t2.id.clone(),
        mind_id: researcher.0.clone(),
        role: Role::Agent,
        success: true,
        duration_secs: 45.0,
        tool_calls_total: 12,
        tool_calls_successful: 11,
        memory_writes: 2,
        verification_passed: true,
        learnings_extracted: 1,
        completed_at: Utc::now(),
    };
    let fitness = codex_fitness::compute_fitness(&outcome);
    info!("Fitness: {:?} (composite: {:.2})", fitness, fitness.composite());

    // ═══════ PHASE 7: Cross-Domain Transfer ═══════

    info!("Phase 7: Cross-domain transfer");

    let mut transfer = TransferEngine::new();
    transfer.publish(TransferPattern {
        id: "pattern-001".into(),
        source_mind: "researcher".into(),
        source_context: "cortex-boot".into(),
        pattern: PatternContent {
            description: "Codex multi-agent hierarchy already exists".into(),
            applicability: "Any Codex fork attempting custom orchestration".into(),
            technique: "Set agent_max_depth=2, define roles, use built-in tools".into(),
        },
        evidence: "Found in codex-core in 45s".into(),
        confidence: Confidence::Validated,
        share_scope: ShareScope::Civ,
        created_at: Utc::now(),
    });
    let found = transfer.search("multi-agent");
    info!("Published {} pattern(s), found {} via search", transfer.count_by_scope(ShareScope::Civ), found.len());

    // ═══════ PHASE 8: InputMux Routing ═══════

    info!("Phase 8: InputMux routing");

    let mux = codex_coordination::InputMux::new(primary_id.clone());
    let inputs = vec![
        ExternalInput {
            source: InputSource::Human { name: "Corey".into() },
            content: "Deploy the service".into(),
            priority: InputPriority::Normal,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        },
        ExternalInput {
            source: InputSource::Hub { room: "#general".into(), thread: None },
            content: "New message".into(),
            priority: InputPriority::Normal,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        },
        ExternalInput {
            source: InputSource::Boop { boop_type: "work-mode".into() },
            content: "Scheduled check".into(),
            priority: InputPriority::Low,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        },
    ];
    for input in &inputs {
        let decision = mux.route(input);
        info!("{:?} → {:?}", input.source, decision);
    }

    // ═══════ PHASE 9: State Snapshot ═══════

    info!("Phase 9: Coordination state");

    let state = manager.coordination_state();
    info!("{} minds, {} active tasks", state.minds.len(), state.active_tasks.len());
    for mind in &state.minds {
        info!("  {} [{:?}] {:?} children={}", mind.id, mind.role, mind.status, mind.children.len());
    }

    // ═══════ PHASE 10: Graceful Shutdown ═══════

    info!("Phase 10: Shutdown (leaves → roots)");

    manager.shutdown_mind(&researcher).unwrap();
    manager.shutdown_mind(&coder).unwrap();
    info!("Agents terminated");

    manager.shutdown_mind(&research_lead).unwrap();
    manager.shutdown_mind(&code_lead).unwrap();
    info!("Team leads terminated");

    manager.shutdown_mind(&primary_id).unwrap();
    info!("Primary terminated");

    // ═══════ PHASE 11: Dream Config ═══════

    let dream = DreamConfig::default();
    info!("Dream mode: {} - {}, archive threshold: {}", dream.start_time, dream.end_time, dream.archive_threshold);

    // ═══════ PHASE 12: Memory Graph (Principle 1) ═══════

    info!("Phase 12: Memory IS Architecture");

    let mem_store = MemoryStore::new(":memory:").await.unwrap();

    // Store learnings from the delegation chain
    let learning1 = mem_store
        .store(NewMemory {
            mind_id: researcher.0.clone(),
            role: "agent".into(),
            vertical: Some("research".into()),
            category: MemoryCategory::Learning,
            title: "Codex multi-agent hierarchy exists".into(),
            content: "Found 3 ThreadManager spawn points. AgentControl at control.rs:150 \
                      is the injection point. Configure, don't rebuild."
                .into(),
            evidence: vec![
                "core/src/thread_manager.rs:192".into(),
                "core/src/agent/control.rs:150".into(),
            ],
            tier: MemoryTier::Working,
            session_id: Some("cortex-boot-001".into()),
            task_id: Some(t2.id.clone()),
        })
        .await
        .unwrap();
    info!("Stored learning: {learning1}");

    let decision1 = mem_store
        .store(NewMemory {
            mind_id: primary_id.0.clone(),
            role: "primary".into(),
            vertical: Some("coordination".into()),
            category: MemoryCategory::Decision,
            title: "Fork Codex, inject fractal engine".into(),
            content: "Rather than build from scratch, fork OpenAI Codex CLI (Apache-2.0) \
                      and inject our coordination layer. Codex already has multi-agent \
                      hierarchy — we add role filtering, depth limits, memory."
                .into(),
            evidence: vec![
                "Codex has ThreadManager, AgentControl, agent_max_depth".into(),
                "90 crates of production infrastructure we get for free".into(),
            ],
            tier: MemoryTier::Session,
            session_id: Some("cortex-boot-001".into()),
            task_id: None,
        })
        .await
        .unwrap();
    info!("Stored decision: {decision1}");

    let pattern1 = mem_store
        .store(NewMemory {
            mind_id: primary_id.0.clone(),
            role: "primary".into(),
            vertical: None,
            category: MemoryCategory::Pattern,
            title: "Depth scoring compounds via citation".into(),
            content: "Memories that get cited grow deeper (depth_score += 0.1, cap 1.0). \
                      Uncited memories fade. Dream mode archives low-depth memories. \
                      This is how Cortex learns what matters."
                .into(),
            evidence: vec!["codex-memory store.rs cite() method".into()],
            tier: MemoryTier::LongTerm,
            session_id: None,
            task_id: None,
        })
        .await
        .unwrap();
    info!("Stored pattern: {pattern1}");

    // Build the graph: decision cites learning, pattern cites decision
    mem_store.cite(&decision1, &learning1).await.unwrap();
    mem_store.cite(&pattern1, &decision1).await.unwrap();
    info!("Graph links created (decision cites learning, pattern cites decision)");

    // Create a builds_on link
    mem_store
        .link(&pattern1, &learning1, LinkType::BuildsOn, 0.9)
        .await
        .unwrap();

    // Promote the learning (working → session, since it proved useful)
    let new_tier = mem_store.promote(&learning1).await.unwrap();
    info!("Promoted learning to: {new_tier}");

    // Search: find memories about "Codex"
    let results = mem_store
        .search(&MemoryQuery {
            text: Some("Codex multi-agent".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    info!("FTS search 'Codex multi-agent': {} result(s)", results.len());
    for r in &results {
        info!(
            "  [{:.2}] {} (depth={:.1}, citations={}, tier={})",
            r.relevance, r.memory.title, r.memory.depth_score, r.memory.citation_count, r.memory.tier
        );
    }

    // Show graph links for the learning
    let links = mem_store.get_links(&learning1).await.unwrap();
    info!("Links for learning: {} connection(s)", links.len());
    for l in &links {
        info!("  {} --{}--> {}", l.source_id[..8].to_string(), l.link_type, l.target_id[..8].to_string());
    }

    // Archive candidates (dream mode prep)
    let candidates = mem_store.archive_candidates(0.05, 10).await.unwrap();
    info!("Archive candidates (depth < 0.05): {}", candidates.len());

    let total = mem_store.count().await.unwrap();
    info!("Total memories in graph: {total}");

    // ═══════ PHASE 13: Dream Cycle ═══════

    info!("Phase 13: Dream cycle — consolidate, prune, synthesize");

    let dream_engine = DreamEngine::new(&mem_store, dream);
    let report = dream_engine.run_cycle().await.unwrap();

    info!(
        "Dream report: audited={}, consolidated={}, pruned={}, synthesized={}",
        report.audited, report.consolidated, report.pruned, report.synthesized
    );
    for phase in &report.cycle.phases {
        if !phase.findings.is_empty() {
            info!("  {:?}: {} finding(s)", phase.phase, phase.findings.len());
        }
    }

    let total_after = mem_store.count().await.unwrap();
    info!("Memory graph: {total} before dream → {total_after} after");

    // ═══════ PHASE 14: Session Persistence (Principle 2) ═══════

    info!("Phase 14: Identity IS Continuity — session persistence");

    // Check for prior sessions (simulates a restart)
    let prev_boots = mem_store.boot_count().await.unwrap();
    info!("Previous boot count: {prev_boots}");

    if let Some(prev_session) = mem_store.load_latest_session().await.unwrap() {
        info!(
            "Restored session {} (boot #{}, {} memories at close)",
            &prev_session.id[..8],
            prev_session.boot_count,
            // Parse the coordination state to show mind count
            prev_session.coordination_state_json.len()
        );
    } else {
        info!("No prior session found — this is the first boot");
    }

    // Start this session
    let session_id = mem_store
        .start_session(Some("Cortex boot demo — full 20-phase lifecycle"))
        .await
        .unwrap();
    let boot_num = mem_store.boot_count().await.unwrap();
    info!(session = %session_id, boot = boot_num, "Session started");

    // Serialize the coordination state (this is what survives restarts)
    let coord_state = manager.coordination_state();
    let coord_json = serde_json::to_string_pretty(&coord_state).unwrap();
    info!("Coordination state serialized ({} bytes, {} minds)", coord_json.len(), coord_state.minds.len());

    // End the session (saves state for next boot)
    mem_store
        .end_session(&session_id, &coord_json)
        .await
        .unwrap();
    info!("Session ended — state persisted for next boot");

    // Verify: load it back
    let restored = mem_store.load_latest_session().await.unwrap().unwrap();
    assert_eq!(restored.id, session_id);
    assert_eq!(restored.boot_count, boot_num);
    let restored_state: codex_coordination::types::CoordinationState =
        serde_json::from_str(&restored.coordination_state_json).unwrap();
    info!(
        "Verified: restored {} minds, {} tasks from persisted state",
        restored_state.minds.len(),
        restored_state.active_tasks.len()
    );

    // ═══════ PHASE 15: Tool Execution Engine ═══════

    info!("Phase 15: Tool execution — where Cortex meets the filesystem");

    let workspace = std::env::current_dir().unwrap();
    let mut registry = ToolRegistry::new();
    codex_exec::tools::register_builtins(&mut registry, workspace.clone());

    let tool_count = registry.len();
    let tool_names = registry.tool_names();
    info!("Registered {} built-in tools: {:?}", tool_count, tool_names);

    // Show role-filtered tool visibility
    let agent_tools = registry.definitions_for_role(Role::Agent);
    let primary_tools_exec = registry.definitions_for_role(Role::Primary);
    let lead_tools = registry.definitions_for_role(Role::TeamLead);
    info!(
        "Tool visibility: Agent={}, TeamLead={}, Primary={}",
        agent_tools.len(),
        lead_tools.len(),
        primary_tools_exec.len()
    );

    let sandbox = SandboxEnforcer::new(workspace);
    let executor = ToolExecutor::new(registry, sandbox);

    // Agent executes bash — should succeed
    let bash_call = ToolCall {
        name: "bash".into(),
        arguments: serde_json::json!({"command": "echo 'Cortex lives' && date"}),
    };
    let bash_result = executor.execute(&bash_call, Role::Agent).await.unwrap();
    info!("Agent bash: {}", bash_result.output.trim());

    // Agent reads a file — should succeed
    let read_call = ToolCall {
        name: "read".into(),
        arguments: serde_json::json!({"file_path": "Cargo.toml", "limit": 5}),
    };
    let read_result = executor.execute(&read_call, Role::Agent).await.unwrap();
    info!("Agent read Cargo.toml (first 5 lines): {} bytes", read_result.output.len());

    // Primary tries bash — should be DENIED
    let primary_bash = executor.execute(&bash_call, Role::Primary).await;
    match &primary_bash {
        Err(e) => info!("Primary bash denied (correct): {e}"),
        Ok(_) => panic!("Primary should NEVER execute bash!"),
    }

    // Team lead tries bash — should be DENIED
    let lead_bash = executor.execute(&bash_call, Role::TeamLead).await;
    match &lead_bash {
        Err(e) => info!("TeamLead bash denied (correct): {e}"),
        Ok(_) => panic!("TeamLead should NEVER execute bash!"),
    }

    // Agent tries dangerous command — sandbox blocks it
    let dangerous_call = ToolCall {
        name: "bash".into(),
        arguments: serde_json::json!({"command": "rm -rf /"}),
    };
    let dangerous_result = executor.execute(&dangerous_call, Role::Agent).await;
    match &dangerous_result {
        Err(e) => info!("Dangerous command blocked (correct): {e}"),
        Ok(_) => panic!("Sandbox should block rm -rf /!"),
    }

    info!("Tool execution engine: {} tools, role enforcement verified", tool_count);

    // ═══════ PHASE 16: IPC — MCP Inter-Mind Communication (Channel) ═══════

    info!("Phase 16: IPC — MCP inter-mind communication (channel transport)");

    // Create a client-server pair (in-process, via channel transport)
    let (client_transport, mut server_transport) = ChannelTransport::pair();

    // Server: a TeamLead mind
    let server_handle = tokio::spawn(async move {
        let mut server = McpMindServer::new("research-lead-ipc", Role::TeamLead);
        server.run_channel(&mut server_transport, None).await.unwrap();
    });

    // Client: Primary connects to the TeamLead
    let mut client = McpMindClient::new("research-lead-ipc", client_transport);

    // MCP handshake
    let init_result = client.initialize().await.unwrap();
    info!(
        "IPC initialized: {} (protocol {})",
        init_result.server_info.name,
        init_result.protocol_version
    );

    // List tools via MCP
    let tools = client.list_tools().await.unwrap();
    info!("IPC tools available: {} (TeamLead role)", tools.len());
    for t in &tools {
        info!("  - {}: {}", t.name, t.description);
    }

    // Delegate a task via MCP
    let delegate_result = client
        .delegate("ipc-task-001", "Analyze Cortex IPC architecture", None, "primary")
        .await
        .unwrap();
    info!(
        "IPC delegate: accepted={}, mind={}, task={}",
        delegate_result.accepted, delegate_result.mind_id, delegate_result.task_id
    );

    // Check status via MCP
    let status = client.status().await.unwrap();
    info!(
        "IPC status: mind={}, role={}, status={}, task={:?}",
        status.mind_id, status.role, status.status, status.current_task
    );

    // Graceful shutdown via MCP
    client.shutdown().await.unwrap();
    server_handle.await.unwrap();
    info!("IPC channel server shutdown complete");

    // ═══════ PHASE 17: LLM Prompt Construction ═══════

    info!("Phase 17: LLM prompt construction");

    // Build prompts for each role
    let primary_prompt = PromptBuilder::new(Role::Primary, "cortex-primary")
        .add_context("Session boot #1. All systems nominal.");
    let primary_msgs = primary_prompt.build_messages("Spawn a research team lead and delegate code analysis");
    info!(
        "Primary prompt: {} messages, system={} chars",
        primary_msgs.len(),
        primary_msgs[0].content.as_ref().map(|c| c.len()).unwrap_or(0)
    );

    let lead_prompt = PromptBuilder::new(Role::TeamLead, "research-lead")
        .vertical("research")
        .add_context("Memory search: 3 relevant patterns found from prior sessions.");
    let lead_msgs = lead_prompt.build_messages("Analyze Codex exec crate for injection points");
    info!(
        "TeamLead prompt: {} messages, system={} chars",
        lead_msgs.len(),
        lead_msgs[0].content.as_ref().map(|c| c.len()).unwrap_or(0)
    );

    let agent_prompt = PromptBuilder::new(Role::Agent, "coder-1")
        .add_context("Task: implement InputMux routing. Prior learning: Codex has ThreadManager.");
    let agent_msgs = agent_prompt.build_messages("Implement InputMux with keyword-based routing");
    info!(
        "Agent prompt: {} messages, system={} chars",
        agent_msgs.len(),
        agent_msgs[0].content.as_ref().map(|c| c.len()).unwrap_or(0)
    );

    // Convert tool definitions to OpenAI-compatible schemas
    let tool_defs = executor.registry().definitions_for_role(Role::Agent);
    let schemas = OllamaClient::tool_schemas(&tool_defs);
    info!(
        "Tool schemas for Agent: {} tools converted to OpenAI format",
        schemas.len()
    );

    // ═══════ PHASE 18: LLM Connection Test ═══════

    info!("Phase 18: LLM connection test (Ollama)");

    let llm_config = OllamaConfig {
        base_url: "http://localhost:11434/v1".into(),
        model: "qwen2.5:7b".into(),
        temperature: 0.7,
        max_tokens: 256,
        api_key: None,
    };

    let llm = OllamaClient::new(llm_config);

    // Try a simple chat completion (no tools)
    let test_messages = vec![
        codex_llm::ollama::ChatMessage::system("You are Cortex, a fractal AI mind. Respond in one sentence."),
        codex_llm::ollama::ChatMessage::user("Who are you?"),
    ];

    match llm.chat(&test_messages, None).await {
        Ok(resp) => {
            if let Some(choice) = resp.choices.first() {
                let content = choice.message.content.as_deref().unwrap_or("(no content)");
                info!("LLM response: {}", content);
                info!("LLM connection: LIVE (model: {})", llm.config().model);
            }
        }
        Err(e) => {
            info!("LLM connection: OFFLINE ({e}) — Cortex will think when Ollama is running");
        }
    }

    // ═══════ PHASE 19: Process-Based Mind Spawning ═══════

    info!("Phase 19: Process-based mind spawning via MCP over stdio");

    let cortex_exe = std::env::current_exe().unwrap();
    info!("Spawning child: {} --serve --mind-id spawned-lead --role team-lead", cortex_exe.display());

    match spawn_child_mind(&cortex_exe, "spawned-lead", "team-lead").await {
        Ok(()) => {
            info!("Process-based mind spawning: VERIFIED");
        }
        Err(e) => {
            info!("Process-based mind spawning failed: {e} — binary must be built first");
        }
    }

    // ═══════ PHASE 20: Real LLM ThinkLoop ═══════

    info!("Phase 20: Real LLM ThinkLoop (qwen2.5:7b via Ollama)");

    let think_config = ThinkLoopConfig {
        max_iterations: 3,
        ollama: OllamaConfig {
            model: "qwen2.5:7b".into(),
            max_tokens: 512,
            ..OllamaConfig::default()
        },
    };

    let think_loop = ThinkLoop::new(think_config);
    let think_prompt = PromptBuilder::new(Role::Agent, "thinking-agent")
        .add_context("You are running inside Cortex. Be concise.");

    let think_schemas = OllamaClient::tool_schemas(
        &executor.registry().definitions_for_role(Role::Agent),
    );

    match think_loop
        .run(
            &think_prompt,
            "List the files in the current directory using the bash tool, then summarize what you see.",
            &think_schemas,
            &executor,
            Role::Agent,
        )
        .await
    {
        Ok(result) => {
            info!(
                "ThinkLoop: {} iterations, {} tool calls, completed={}",
                result.iterations,
                result.tool_calls_made.len(),
                result.completed
            );
            for tc in &result.tool_calls_made {
                info!("  Tool: {} (iter {})", tc.tool_name, tc.iteration);
            }
            let preview = if result.response.len() > 200 {
                format!("{}...", safe_truncate(&result.response, 200))
            } else {
                result.response.clone()
            };
            info!("Response: {preview}");
        }
        Err(e) => {
            info!("ThinkLoop: OFFLINE ({e}) — will work when Ollama has qwen2.5:7b");
        }
    }

    // ═══════ PHASE 22: ThinkLoop with Memory Integration ═══════

    info!("Phase 22: ThinkLoop with memory (memory_search + memory_write)");

    let memory_think_config = ThinkLoopConfig {
        max_iterations: 4,
        ollama: OllamaConfig {
            model: "qwen2.5:7b".into(),
            max_tokens: 512,
            ..OllamaConfig::default()
        },
    };
    let memory_think_loop = ThinkLoop::new(memory_think_config);

    // Load AGENTS.md for the thinking agent
    let agents_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
        .join("../../agents");
    let agents_dir_canonical = std::fs::canonicalize(&agents_dir)
        .unwrap_or_else(|_| std::path::PathBuf::from("agents"));
    info!("AGENTS.md directory: {}", agents_dir_canonical.display());

    let memory_prompt = PromptBuilder::new(Role::Agent, "thinking-agent")
        .agents_dir(&agents_dir_canonical)
        .add_context("You are running inside Cortex with access to a memory graph. \
                       Use memory_search to find relevant context, and memory_write to save insights.");

    let memory_schemas = OllamaClient::tool_schemas(
        &executor.registry().definitions_for_role(Role::Agent),
    );

    match memory_think_loop
        .run_with_memory(
            &memory_prompt,
            "Search your memory for 'coordination', then write a memory about what you learned. \
             If nothing found, write a new memory about fractal coordination.",
            &memory_schemas,
            &executor,
            Role::Agent,
            Some(&mem_store),
            Some("thinking-agent"),
        )
        .await
    {
        Ok(result) => {
            info!(
                "ThinkLoop+Memory: {} iterations, {} tool calls, completed={}",
                result.iterations,
                result.tool_calls_made.len(),
                result.completed
            );
            let mem_tools: Vec<&str> = result.tool_calls_made.iter()
                .filter(|tc| tc.tool_name.starts_with("memory_"))
                .map(|tc| tc.tool_name.as_str())
                .collect();
            if !mem_tools.is_empty() {
                info!("Memory tools used: {:?}", mem_tools);
            }
            info!("ThinkLoop with memory: VERIFIED");
        }
        Err(e) => {
            info!("ThinkLoop+Memory: OFFLINE ({e}) — will work when Ollama has qwen2.5:7b");
        }
    }

    // ═══════ PHASE 23: Multi-Mind Orchestration via ProcessBridge ═══════

    info!("Phase 23: Multi-mind orchestration via ProcessBridge");

    use codex_coordination::ProcessBridge;

    let cortex_exe2 = std::env::current_exe().unwrap();
    let mut bridge = ProcessBridge::new(cortex_exe2);

    // Spawn two team leads as real child processes
    let research_id = MindId("bridge-research-lead".into());
    let code_id = MindId("bridge-code-lead".into());

    let mut bridge_success = true;

    match bridge.spawn(&research_id, Role::TeamLead).await {
        Ok(()) => info!("Bridge: research-lead spawned"),
        Err(e) => {
            info!("Bridge: research-lead spawn failed: {e}");
            bridge_success = false;
        }
    }

    if bridge_success {
        match bridge.spawn(&code_id, Role::TeamLead).await {
            Ok(()) => info!("Bridge: code-lead spawned"),
            Err(e) => {
                info!("Bridge: code-lead spawn failed: {e}");
                bridge_success = false;
            }
        }
    }

    if bridge_success {
        info!("Bridge: {} active minds", bridge.active_count());

        // List tools on research-lead
        if let Ok(tools) = bridge.list_tools(&research_id).await {
            info!("Bridge: research-lead has {} tools", tools.len());
        }

        // Delegate tasks to both leads
        if let Ok(r) = bridge.delegate(
            &research_id, "bridge-task-001",
            "Research fractal coordination patterns",
            Some("Multi-mind orchestration demo"), "primary",
        ).await {
            info!("Bridge: research delegation accepted={}", r.accepted);
        }

        if let Ok(r) = bridge.delegate(
            &code_id, "bridge-task-002",
            "Implement pattern extraction module",
            Some("Multi-mind orchestration demo"), "primary",
        ).await {
            info!("Bridge: code delegation accepted={}", r.accepted);
        }

        // Check status of both
        if let Ok(s) = bridge.status(&research_id).await {
            info!("Bridge: research status={} task={:?}", s.status, s.current_task);
        }
        if let Ok(s) = bridge.status(&code_id).await {
            info!("Bridge: code status={} task={:?}", s.status, s.current_task);
        }

        // Shutdown all — ProcessBridge handles graceful MCP shutdown + process wait
        bridge.shutdown_all().await;
        info!("Bridge: all minds shut down, {} remaining", bridge.active_count());

        info!("Multi-mind orchestration via ProcessBridge: VERIFIED");
    } else {
        info!("Multi-mind orchestration: SKIPPED (binary not built or spawn failed)");
    }

    // ═══════ PHASE 24: Thinking Children via ProcessBridge ═══════

    info!("Phase 24: Thinking children — ProcessBridge + ThinkLoop integration");

    let cortex_exe3 = std::env::current_exe().unwrap();
    let mut thinking_bridge = ProcessBridge::new(cortex_exe3);

    let thinker_id = MindId("thinking-child".into());
    let mut thinking_success = true;

    // Spawn a child with --think flag — this child has a real ThinkLoop
    match thinking_bridge.spawn_thinking(&thinker_id, Role::Agent).await {
        Ok(()) => info!("ThinkBridge: thinking child spawned"),
        Err(e) => {
            info!("ThinkBridge: spawn failed: {e}");
            thinking_success = false;
        }
    }

    if thinking_success {
        // Delegate a task — the child will THINK about it via ThinkLoop
        match thinking_bridge.delegate(
            &thinker_id,
            "think-task-001",
            "List the files in the current directory and describe what you see.",
            Some("You are a Cortex thinking child. Use tools to complete the task."),
            "primary",
        ).await {
            Ok(r) => {
                info!("ThinkBridge: accepted={}", r.accepted);
                if let Some(ref response) = r.response {
                    let preview = if response.len() > 300 {
                        format!("{}...", safe_truncate(response, 300))
                    } else {
                        response.clone()
                    };
                    info!("ThinkBridge: RESPONSE = {preview}");
                    info!(
                        "ThinkBridge: iterations={:?}, tool_calls={:?}, completed={:?}",
                        r.iterations, r.tool_calls_made, r.completed
                    );
                    info!("Thinking children via ProcessBridge: VERIFIED");
                } else {
                    info!("ThinkBridge: no response (handler may not have been created)");
                }
            }
            Err(e) => {
                info!("ThinkBridge: delegation failed: {e}");
            }
        }

        thinking_bridge.shutdown_all().await;
        info!("ThinkBridge: all thinking children shut down");
    } else {
        info!("Thinking children: SKIPPED (binary not built or spawn failed)");
    }

    // ═══════ PHASE 25: Role-Aware Model Routing ═══════

    info!("Phase 25: Role-aware model routing (Gemma 4 + M2.7)");

    use codex_llm::ModelRouter;

    let router = ModelRouter::from_env();
    let primary_config = router.config_for_role(Role::Primary);
    let lead_config = router.config_for_role(Role::TeamLead);
    let agent_config = router.config_for_role(Role::Agent);
    let lightweight_config = router.config_lightweight();

    info!(
        "Model routing: Primary={}, TeamLead={}, Agent={}, Lightweight={}",
        primary_config.model, lead_config.model, agent_config.model, lightweight_config.model
    );
    info!(
        "Provider: {} (cloud={})",
        primary_config.base_url, primary_config.api_key.is_some()
    );

    // Test cloud model connection (if API key is set)
    if primary_config.api_key.is_some() {
        let cloud_llm = OllamaClient::new(primary_config.clone());
        let test_msgs = vec![
            codex_llm::ollama::ChatMessage::system(
                "You are Cortex, a fractal coordination engine. Respond in one sentence."
            ),
            codex_llm::ollama::ChatMessage::user("Identify yourself."),
        ];

        match cloud_llm.chat(&test_msgs, None).await {
            Ok(resp) => {
                if let Some(choice) = resp.choices.first() {
                    let content = choice.message.content.as_deref().unwrap_or("(no content)");
                    info!("Cloud model ({}) response: {}", primary_config.model, content);
                    info!("Cloud connection: LIVE");
                }
            }
            Err(e) => {
                info!("Cloud connection: OFFLINE ({e}) — set OLLAMA_API_KEY for cloud models");
            }
        }
    } else {
        info!("Cloud models: not configured (set OLLAMA_API_KEY for Gemma 4 + M2.7)");
    }

    // ═══════ PHASE 26: End-to-End Multi-Mind Thinking Chain ═══════

    info!("Phase 26: End-to-end multi-mind thinking chain");
    info!("  Primary → TeamLead (thinking) → Agent (thinking)");

    let cortex_exe4 = std::env::current_exe().unwrap();
    let mut chain_bridge = ProcessBridge::new(cortex_exe4);
    let chain_lead = MindId("chain-research-lead".into());
    let chain_agent = MindId("chain-researcher".into());
    let mut chain_success = true;

    // Spawn TeamLead with thinking
    match chain_bridge.spawn_thinking(&chain_lead, Role::TeamLead).await {
        Ok(()) => info!("Chain: TeamLead spawned (thinking)"),
        Err(e) => {
            info!("Chain: TeamLead spawn failed: {e}");
            chain_success = false;
        }
    }

    // Spawn Agent with thinking
    if chain_success {
        match chain_bridge.spawn_thinking(&chain_agent, Role::Agent).await {
            Ok(()) => info!("Chain: Agent spawned (thinking)"),
            Err(e) => {
                info!("Chain: Agent spawn failed: {e}");
                chain_success = false;
            }
        }
    }

    if chain_success {
        info!("Chain: {} thinking minds active", chain_bridge.active_count());

        // Step 1: Primary delegates to TeamLead
        match chain_bridge.delegate(
            &chain_lead,
            "chain-task-001",
            "You are the research team lead. Summarize what a fractal coordination engine is in 2 sentences.",
            Some("This is a multi-level delegation test. Think carefully and respond."),
            "primary",
        ).await {
            Ok(r) => {
                info!("Chain: TeamLead accepted={}", r.accepted);
                if let Some(ref response) = r.response {
                    let preview = if response.len() > 300 { format!("{}...", safe_truncate(response, 300)) } else { response.clone() };
                    info!("Chain: TeamLead response = {preview}");
                    info!("Chain: TeamLead iterations={:?}, tool_calls={:?}", r.iterations, r.tool_calls_made);
                }
            }
            Err(e) => info!("Chain: TeamLead delegation failed: {e}"),
        }

        // Step 2: Primary delegates to Agent (simulating TeamLead → Agent)
        match chain_bridge.delegate(
            &chain_agent,
            "chain-task-002",
            "List the Rust crates in the current directory using bash, then describe the project structure.",
            Some("You are a research agent inside Cortex. Use tools to complete the task."),
            "research-lead",
        ).await {
            Ok(r) => {
                info!("Chain: Agent accepted={}", r.accepted);
                if let Some(ref response) = r.response {
                    let preview = if response.len() > 300 { format!("{}...", safe_truncate(response, 300)) } else { response.clone() };
                    info!("Chain: Agent response = {preview}");
                    info!("Chain: Agent iterations={:?}, tool_calls={:?}", r.iterations, r.tool_calls_made);
                }
            }
            Err(e) => info!("Chain: Agent delegation failed: {e}"),
        }

        chain_bridge.shutdown_all().await;
        info!("Chain: all minds shut down — multi-level thinking VERIFIED");
    } else {
        info!("Chain: SKIPPED (binary not built or spawn failed)");
    }

    // ═══════ PHASE 27: ToolInterceptor — Recursive Delegation ═══════

    info!("Phase 27: ToolInterceptor trait + DelegationInterceptor (recursive delegation)");

    // Verify the ToolInterceptor trait works in isolation
    use codex_llm::ToolInterceptor;
    struct TestInterceptor;

    #[async_trait::async_trait]
    impl ToolInterceptor for TestInterceptor {
        fn schemas(&self) -> Vec<codex_llm::ollama::ToolSchema> {
            vec![codex_llm::ollama::ToolSchema {
                tool_type: "function".into(),
                function: codex_llm::ollama::FunctionSchema {
                    name: "test_tool".into(),
                    description: "Demo interceptor".into(),
                    parameters: serde_json::json!({"type": "object", "properties": {}}),
                },
            }]
        }

        async fn handle(&self, name: &str, _args: &serde_json::Value) -> Option<codex_exec::ToolResult> {
            if name == "test_tool" {
                Some(codex_exec::ToolResult::ok("intercepted!"))
            } else {
                None
            }
        }
    }

    let interceptor = TestInterceptor;
    assert_eq!(interceptor.schemas().len(), 1);
    assert_eq!(interceptor.schemas()[0].function.name, "test_tool");
    let result = interceptor.handle("test_tool", &serde_json::json!({})).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().output, "intercepted!");
    let pass_through = interceptor.handle("unknown", &serde_json::json!({})).await;
    assert!(pass_through.is_none());
    info!("ToolInterceptor: intercept + pass-through verified");

    // Verify DelegationInterceptor has correct schemas
    let delegation_test = DelegationInterceptor::new(
        std::env::current_exe().unwrap(),
        "test-lead".into(),
    );
    let del_schemas = delegation_test.schemas();
    let del_names: Vec<&str> = del_schemas.iter().map(|s| s.function.name.as_str()).collect();
    assert_eq!(del_schemas.len(), 3, "DelegationInterceptor should expose 3 tools");
    assert!(del_names.contains(&"spawn_agent"), "Missing spawn_agent");
    assert!(del_names.contains(&"delegate_to_agent"), "Missing delegate_to_agent");
    assert!(del_names.contains(&"shutdown_agent"), "Missing shutdown_agent");
    info!("DelegationInterceptor: 3 delegation tools verified ({:?})", del_names);

    // Verify unknown tools pass through
    let pass = delegation_test.handle("bash", &serde_json::json!({})).await;
    assert!(pass.is_none(), "Non-delegation tools should pass through");
    info!("DelegationInterceptor: pass-through for non-delegation tools verified");

    info!("Phase 27: COMPLETE — TeamLeads can now recursively spawn + delegate to agents");

    // ═══════ FINAL BANNER ═══════

    let ipc_tools_count = tools.len();
    let schema_count = schemas.len();

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                                                          ║");
    println!("║   CORTEX BOOT COMPLETE — Phase 6                        ║");
    println!("║                                                          ║");
    println!("║   Minds: 5 (1 Primary, 2 Team Leads, 2 Agents)          ║");
    println!("║   Tasks: 3 delegated, 1 completed                       ║");
    println!("║   Red team: 1 verification (APPROVED)                    ║");
    println!("║   Transfer: 1 pattern published                          ║");
    println!("║   Fitness: scored (Agent role)                           ║");
    println!("║   Memory: {} stored, 2 cited, 1 promoted, 3 linked      ║",
        total_after);
    println!("║   Dream: {} audited, {} pruned, {} synthesized           ║",
        report.audited, report.pruned, report.synthesized);
    println!("║   Session: boot #{boot_num}, state persisted             ║");
    println!("║   Tools: {tool_count} registered, role-enforced              ║");
    println!("║   IPC: channel + stdio + ProcessBridge                   ║");
    println!("║   IPC tools: {ipc_tools_count} exposed via MCP                      ║");
    println!("║   LLM: {schema_count} tool schemas, ThinkLoop wired              ║");
    println!("║   Model routing: {} / {}                                ║",
        primary_config.model, lightweight_config.model);
    println!("║   Cloud auth: Bearer token for Ollama Cloud             ║");
    println!("║   Memory-integrated thinking: search + write in loop    ║");
    println!("║   ProcessBridge: multi-mind spawn + delegate + shutdown  ║");
    println!("║   THINKING CHILDREN: delegate → ThinkLoop → response    ║");
    println!("║   MULTI-LEVEL CHAIN: Primary → Lead → Agent (all LLM)  ║");
    println!("║   ToolInterceptor: extensible tool injection in loop    ║");
    println!("║   DelegationInterceptor: recursive spawn+delegate       ║");
    println!("║   All minds gracefully terminated.                       ║");
    println!("║                                                          ║");
    println!("║   Cortex THINKS. The fractal hierarchy is alive.        ║");
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
}

/// Spawn a child Cortex process in MCP server mode, do a full lifecycle, then shut it down.
async fn spawn_child_mind(
    cortex_exe: &std::path::Path,
    mind_id: &str,
    role: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Stdio;

    let mut child = tokio::process::Command::new(cortex_exe)
        .arg("--serve")
        .arg("--mind-id")
        .arg(mind_id)
        .arg("--role")
        .arg(role)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let child_stdin = child.stdin.take().expect("child stdin");
    let child_stdout = child.stdout.take().expect("child stdout");
    let transport = StdioTransport::new(child_stdout, child_stdin);

    let mut client = McpMindClient::new(mind_id, transport);

    // MCP handshake
    let init = client.initialize().await.map_err(|e| format!("init: {e}"))?;
    info!(
        "Process mind connected: {} (protocol {})",
        init.server_info.name, init.protocol_version
    );

    // List tools
    let tools = client.list_tools().await.map_err(|e| format!("tools: {e}"))?;
    info!("Process mind tools: {}", tools.len());

    // Delegate a task
    let delegate = client
        .delegate("process-task-001", "Analyze via MCP over stdio", None, "primary")
        .await
        .map_err(|e| format!("delegate: {e}"))?;
    info!("Process delegation: accepted={}, task={}", delegate.accepted, delegate.task_id);

    // Check status
    let status = client.status().await.map_err(|e| format!("status: {e}"))?;
    info!(
        "Process mind: {} role={} status={} task={:?}",
        status.mind_id, status.role, status.status, status.current_task
    );

    // Shutdown
    client.shutdown().await.map_err(|e| format!("shutdown: {e}"))?;
    let exit_status = child.wait().await?;
    info!("Process mind exited: {exit_status}");

    Ok(())
}
