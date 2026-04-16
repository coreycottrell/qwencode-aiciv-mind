//! # evolution_full — Push Cortex Through Full Evolution Until It Stalls
//!
//! Phase 0+1 (proven) → Phase 2 (6 PARALLEL teams via direct ThinkLoop) → report.
//!
//! ## Key Design Decisions
//!
//! Phase 0+1: Uses ProcessBridge (proven, sequential, works fine).
//! Phase 2: Bypasses ProcessBridge entirely — runs 6 concurrent ThinkLoops
//! with MINIMAL tool schemas (bash+read+write = 3 tools instead of 19).
//!
//! Why: ProcessBridge.delegate(&mut self) forces sequential execution,
//! and the 19-tool schema payload triggers Ollama Cloud HTTP 500 errors
//! when context accumulates after 2-5 iterations.
//!
//! ```bash
//! cargo run --release --bin evolution_full 2>&1 | tee evolution_full_log.txt
//! ```

use codex_coordination::ProcessBridge;
use codex_coordination::types::MindId;
use codex_exec::{ToolExecutor, ToolRegistry};
use codex_exec::sandbox::SandboxEnforcer;
use codex_exec::tools::bash::BashTool;
use codex_exec::tools::read::ReadTool;
use codex_exec::tools::write::WriteTool;
use codex_llm::ollama::{OllamaClient, ModelRouter};
use codex_llm::think_loop::{ThinkLoop, ThinkLoopConfig};
use codex_llm::prompt::PromptBuilder;
use codex_roles::Role;
use std::sync::Arc;
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
                eprintln!("[evolution_full] Loaded .env from {}", env_file.display());
            }
            break;
        }
        if !dir.pop() {
            break;
        }
    }
}

/// Create a minimal ToolExecutor with only bash+read+write (3 tools).
/// This is the key fix for Phase 2: 3 schemas instead of 19 = ~80% smaller
/// context payload to Ollama Cloud, preventing HTTP 500 errors.
fn minimal_executor(workspace: &std::path::Path) -> ToolExecutor {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new(workspace.to_path_buf())));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(WriteTool));
    let sandbox = SandboxEnforcer::new(workspace.to_path_buf());
    ToolExecutor::new(registry, sandbox)
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
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║   CORTEX FULL EVOLUTION — Push Until Stall                      ║");
    println!("║   Phase 0 → Phase 1 → Phase 2 (6 PARALLEL teams)              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    if std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty()).is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        std::process::exit(1);
    }

    let cortex_exe = std::env::current_exe().expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        std::process::exit(1);
    }

    let project_root = std::env::current_dir().unwrap_or_default();
    let evo_dir = project_root.join("test-evolution-full");
    if !evo_dir.exists() {
        eprintln!("ERROR: test-evolution-full/ folder not found at {}", evo_dir.display());
        std::process::exit(1);
    }

    let evo_path = evo_dir.display().to_string();
    info!(path = %evo_path, "Evolution folder found");

    // ═══════════════════════════════════════════════════════════════
    // PHASE 0 + 1: Self-Discovery + Seed Processing (proven path)
    // ═══════════════════════════════════════════════════════════════

    println!("═══ PHASE 0 + 1: Self-Discovery + Seed Processing ═══");
    println!();

    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let agent_id = MindId("evo-phase01".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("evo-phase01 spawned"),
        Err(e) => {
            eprintln!("FAILED to spawn: {e}");
            std::process::exit(1);
        }
    }

    // --- Phase 0: Self-Discovery ---
    let phase0_task = format!(r#"You are birthing a new AI civilization. Evolution folder: {evo_path}

Complete Phase 0 (Self-Discovery) using tools.

1. Read `{evo_path}/identity.json` using bash: cat {evo_path}/identity.json
   Extract: civ_name, human_name, parent_civ, birth_date

2. Replace ALL placeholders in system-prompt files using bash sed commands.
   Run these EXACT commands:
   bash: sed -i 's/${{CIV_NAME}}/verdant/g; s/${{HUMAN_NAME}}/Sarah Chen/g; s/${{PARENT_CIV}}/A-C-Gee/g; s/${{BIRTH_DATE}}/2026-04-04/g' {evo_path}/system-prompt/constitution.md {evo_path}/system-prompt/operations.md {evo_path}/system-prompt/agents.md

3. IMPORTANT: grep exit code 1 means NO matches found (SUCCESS for verification).
   Run: bash: grep -r '${{' {evo_path}/system-prompt/ || echo "VERIFIED: no placeholders remain"

4. Write adaptation-log.md:
   bash: cat > {evo_path}/memories/identity/adaptation-log.md << 'ENDLOG'
   # Adaptation Log
   ## Phase 0: Self-Discovery
   - Read identity.json: civ_name=verdant, human_name=Sarah Chen
   - Replaced placeholders in constitution.md, operations.md, agents.md
   - Verified: zero placeholders remain
   ENDLOG

5. Write core-identity.json:
   bash: cat > {evo_path}/memories/identity/core-identity.json << 'ENDJSON'
   {{"civ_name":"verdant","human_name":"Sarah Chen","parent_civ":"acg","birth_date":"2026-04-04","evolution_phase":"phase_0_complete"}}
   ENDJSON

Execute ALL 5 steps with bash. Do not skip any."#);

    let start = std::time::Instant::now();

    let phase0_result = match bridge.delegate(
        &agent_id,
        "evolution-phase-0",
        &phase0_task,
        Some("Execute every step using bash tool calls. Be efficient — one command per step."),
        "evolution-full",
    ).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Phase 0 FAILED: {e}");
            bridge.shutdown_all().await;
            std::process::exit(1);
        }
    };

    let phase0_time = start.elapsed();
    println!();
    println!("Phase 0: {:.1}s, {} iters, {} tools, completed={:?}",
        phase0_time.as_secs_f64(),
        phase0_result.iterations.unwrap_or(0),
        phase0_result.tool_calls_made.unwrap_or(0),
        phase0_result.completed);

    // Verify Phase 0
    let constitution = std::fs::read_to_string(evo_dir.join("system-prompt/constitution.md")).unwrap_or_default();
    let remaining = constitution.matches("${").count();
    println!("  Placeholders remaining: {} {}", remaining, if remaining == 0 { "✓" } else { "✗" });

    if remaining > 0 {
        println!("⚠ Phase 0 incomplete — placeholders remain.");
        bridge.shutdown_all().await;
        write_evidence(&evo_path, &phase0_result, None, phase0_time, std::time::Duration::ZERO);
        return;
    }

    bridge.shutdown_all().await;

    // --- Phase 1: Seed Processing ---
    println!();
    println!("═══ PHASE 1: Seed Processing ═══");

    let mut bridge1 = ProcessBridge::new(cortex_main.clone());
    let phase1_id = MindId("evo-phase1".into());

    match bridge1.spawn_thinking(&phase1_id, Role::Agent).await {
        Ok(()) => info!("evo-phase1 spawned"),
        Err(e) => {
            eprintln!("FAILED to spawn Phase 1 agent: {e}");
            std::process::exit(1);
        }
    }

    let seed_path = evo_dir.join("memories/identity/seed-conversation.md");
    let seed_content = std::fs::read_to_string(&seed_path).unwrap_or_else(|_| {
        "Sarah Chen — Verdant Thread sustainable fashion, Portland. Nordstrom placement, \
         4x scaling needed, subscription box debate, traceability tension.".into()
    });
    let seed_summary = if seed_content.len() > 2000 {
        let mut end = 2000;
        while end > 0 && !seed_content.is_char_boundary(end) { end -= 1; }
        seed_content[..end].to_string()
    } else {
        seed_content.clone()
    };

    let phase1_task = format!(r#"Write first impressions for a new AI civilization based on this seed conversation:

---
{seed_summary}
---

Do TWO things with bash:

1. Write a first-impressions analysis (at least 300 words) about Sarah Chen:
   bash: cat > {evo_path}/memories/identity/first-impressions.md << 'ENDIMPRESSIONS'
   (Your analysis: who is Sarah, her values, tensions, what you could build for her)
   ENDIMPRESSIONS

2. Update evolution status:
   bash: cat > {evo_path}/state/evolution-status.json << 'ENDJSON'
   {{"phases":{{"phase_0":{{"complete":true}},"phase_1":{{"complete":true}},"phase_2":{{"complete":false}}}},"overall_complete":false}}
   ENDJSON

Write REAL analysis showing genuine understanding of Sarah's situation."#);

    let phase1_start = std::time::Instant::now();

    let phase1_result = match bridge1.delegate(
        &phase1_id,
        "evolution-phase-1",
        &phase1_task,
        Some("Read the seed conversation carefully, then write genuine first impressions."),
        "evolution-full",
    ).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Phase 1 FAILED: {e}");
            bridge1.shutdown_all().await;
            std::process::exit(1);
        }
    };

    let phase1_time = phase1_start.elapsed();
    let phase01_time = phase0_time + phase1_time;
    let phase01_iters = phase0_result.iterations.unwrap_or(0) + phase1_result.iterations.unwrap_or(0);
    let phase01_tools = phase0_result.tool_calls_made.unwrap_or(0) + phase1_result.tool_calls_made.unwrap_or(0);
    let phase01_result = phase1_result.clone();

    println!();
    println!("Phase 1: {:.1}s, {} iters, {} tools, completed={:?}",
        phase1_time.as_secs_f64(),
        phase1_result.iterations.unwrap_or(0),
        phase1_result.tool_calls_made.unwrap_or(0),
        phase1_result.completed);

    let impressions = std::fs::read_to_string(evo_dir.join("memories/identity/first-impressions.md")).unwrap_or_default();
    let p01_ok = remaining == 0 && impressions.len() > 200;

    println!("  First impressions: {} chars {}", impressions.len(), if impressions.len() > 200 { "✓" } else { "✗" });

    if !p01_ok {
        println!();
        println!("⚠ Phase 1 incomplete — first impressions missing or too short.");
        bridge1.shutdown_all().await;
        write_evidence(&evo_path, &phase01_result, None, phase01_time, std::time::Duration::ZERO);
        return;
    }

    bridge1.shutdown_all().await;
    println!();
    println!("═══ Phase 0+1 PASSED — Proceeding to Phase 2 ═══");
    println!();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: Parallel Awakening — 6 CONCURRENT Teams
    // ═══════════════════════════════════════════════════════════════
    //
    // KEY FIX: Direct ThinkLoops instead of ProcessBridge.
    //
    // Why:
    // 1. ProcessBridge.delegate(&mut self) forces sequential (the &mut self problem)
    // 2. ProcessBridge spawns agents with 19 tools; Phase 2 only needs 3 (bash+read+write)
    // 3. 19-tool schema payload triggers Ollama Cloud HTTP 500 after 2-5 iterations
    //
    // Fix: Create 6 ThinkLoops directly, each with only 3 tool schemas.
    // Run them concurrently via tokio::join_all.
    // This cuts schema payload ~80% and enables true parallelism.

    println!("═══ PHASE 2: Parallel Awakening (6 CONCURRENT teams, direct ThinkLoop) ═══");
    println!();

    let phase2_start = std::time::Instant::now();

    // Create shared infrastructure
    let router = ModelRouter::from_env();
    let ollama_config = router.config_for_role(Role::Agent);

    // Create evolution output directory
    let _ = std::fs::create_dir_all(evo_dir.join("memories/evolution"));

    // Define the 6 evolution teams
    let teams: Vec<(&str, &str, &str)> = vec![
        ("research", "research-findings",
         "Research Sarah Chen's industry. Search for sustainable fashion trends, Nordstrom supplier requirements, regenerative agriculture fiber market, blockchain supply chain tracking. Write findings."),
        ("identity", "identity-refinement",
         "Refine this civilization's identity. Read the first impressions and seed conversation. Propose a naming ceremony: why is 'Verdant' the right name? What values does it embody?"),
        ("holy-shit", "holy-shit-moment",
         "Find the 'holy shit' insight for Sarah Chen. What ONE thing would make her say 'this changes everything'? Consider: traceability + blockchain costs + values tension."),
        ("gifts", "gifts",
         "Design 3 concrete gifts for Sarah Chen demonstrating immediate value. Ideas: Nordstrom scaling analysis, subscription box decision framework, supply chain traceability cost model."),
        ("infrastructure", "infrastructure-plan",
         "Plan technical infrastructure. What tools, services, integrations does Verdant need? Email, Hub registration, memory structure, team lead roster."),
        ("domain", "domain-analysis",
         "Analyze sustainable fashion domain deeply. 5 biggest challenges facing brands like Verdant Thread? Competitive landscape? Market trends?"),
    ];

    // Launch all 6 teams concurrently
    let mut handles = Vec::new();

    for (team_name, output_file, task_desc) in &teams {
        let evo_path = evo_path.clone();
        let evo_dir = evo_dir.clone();
        let output_path = evo_dir.join(format!("memories/evolution/{}.md", output_file));
        let team_name = team_name.to_string();
        let output_file = output_file.to_string();
        let ollama_config = ollama_config.clone();

        // Each team gets its own executor and ThinkLoop (no shared state)
        let task = format!(
            r#"You are the {} team for a new AI civilization called Verdant (for Sarah Chen, sustainable fashion).

Context: Read {evo_path}/memories/identity/first-impressions.md for background on Sarah Chen.

Your task: {task_desc}

Write your output to: {output_path}

Use bash to read files and write your output. Example:
bash: cat > {output_path} << 'ENDOUTPUT'
(your analysis here)
ENDOUTPUT

Be thorough. Write at least 200 words of genuine analysis."#,
            team_name,
            task_desc = task_desc,
            output_path = output_path.display(),
            evo_path = evo_path,
        );

        let handle = tokio::spawn(async move {
            let team_start = std::time::Instant::now();

            // Minimal executor: only bash + read + write (3 tools, not 19)
            let executor = minimal_executor(&evo_dir);
            let tool_defs = executor.registry().definitions_for_role(Role::Agent);
            let schemas = OllamaClient::tool_schemas(&tool_defs);

            let think_config = ThinkLoopConfig {
                max_iterations: 10,
            };
            let provider = OllamaClient::new(ollama_config);
            let think_loop = ThinkLoop::new(Box::new(provider), think_config);

            let prompt = PromptBuilder::new(Role::Agent, format!("evo-{}", team_name))
                .add_context("You are an evolution team member. Use bash to read context and write your output file. Be concise with tool calls.");

            let result = think_loop.run(
                &prompt,
                &task,
                &schemas,
                &executor,
                Role::Agent,
            ).await;

            let team_time = team_start.elapsed();

            match result {
                Ok(think_result) => {
                    let file_exists = output_path.exists();
                    let file_size = std::fs::read_to_string(&output_path).unwrap_or_default().len();
                    let success = think_result.completed && file_exists && file_size > 100;

                    (
                        team_name,
                        output_file,
                        success,
                        think_result.iterations,
                        think_result.tool_calls_made.len() as u32,
                        team_time.as_secs_f64(),
                        think_result.response.len(),
                        file_size,
                        if success { "completed".into() }
                        else if think_result.completed { "no_output_file".into() }
                        else { "max_iterations".into() },
                    )
                }
                Err(e) => {
                    (
                        team_name,
                        output_file,
                        false,
                        0,
                        0,
                        team_time.as_secs_f64(),
                        0,
                        0,
                        format!("ThinkLoop error: {e}"),
                    )
                }
            }
        });

        handles.push(handle);
    }

    // Await all 6 teams concurrently
    let results = futures::future::join_all(handles).await;

    let phase2_time = phase2_start.elapsed();

    // Process results
    let mut team_results: Vec<(String, bool, u32, u32, f64, String)> = Vec::new();
    let mut teams_completed = 0;
    let mut teams_stalled = 0;
    let mut teams_failed = 0;

    for result in results {
        match result {
            Ok((name, _output_file, success, iters, tools, time, response_len, file_size, status)) => {
                println!("--- Team: {} ---", name);
                println!("  Time: {:.1}s | Iters: {} | Tools: {} | Response: {} chars",
                    time, iters, tools, response_len);
                println!("  Output file: {} chars {}", file_size, if success { "✓" } else { "✗" });
                println!("  Result: {}", if success { "COMPLETED ✓" } else { &status });
                println!();

                if success { teams_completed += 1; }
                else if status.contains("error") || status.contains("ThinkLoop") { teams_failed += 1; }
                else { teams_stalled += 1; }

                team_results.push((name, success, iters, tools, time, status));
            }
            Err(e) => {
                println!("--- Team: (join error) ---");
                println!("  PANIC: {e}");
                teams_failed += 1;
                team_results.push(("unknown".into(), false, 0, 0, 0.0, format!("join error: {e}")));
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // RESULTS
    // ═══════════════════════════════════════════════════════════════

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║   FULL EVOLUTION RESULTS                                        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Phase 0+1: {:.1}s, {} iters, {} tools — {}",
        phase01_time.as_secs_f64(), phase01_iters, phase01_tools,
        if p01_ok { "PASSED ✓" } else { "FAILED ✗" });
    println!();
    println!("Phase 2 Teams ({:.1}s total, PARALLEL):", phase2_time.as_secs_f64());
    println!("  {:<15} {:>6} {:>6} {:>8} {:>12} {}", "Team", "Iters", "Tools", "Time", "File", "Status");
    println!("  {}", "-".repeat(70));
    for (name, success, iters, tools, time, status) in &team_results {
        let output_file = evo_dir.join(format!("memories/evolution/{}.md",
            match name.as_str() {
                "research" => "research-findings",
                "identity" => "identity-refinement",
                "holy-shit" => "holy-shit-moment",
                "gifts" => "gifts",
                "infrastructure" => "infrastructure-plan",
                "domain" => "domain-analysis",
                _ => name.as_str(),
            }
        ));
        let file_size = std::fs::read_to_string(&output_file).unwrap_or_default().len();
        println!("  {:<15} {:>6} {:>6} {:>6.1}s {:>8} chars  {}",
            name, iters, tools, time, file_size,
            if *success { "✓" } else { status.as_str() });
    }
    println!();
    println!("Summary: {}/{} teams completed, {} stalled, {} failed",
        teams_completed, team_results.len(), teams_stalled, teams_failed);
    println!("Total time: {:.1}s (Phase 0+1: {:.1}s, Phase 2: {:.1}s)",
        (phase01_time + phase2_time).as_secs_f64(),
        phase01_time.as_secs_f64(),
        phase2_time.as_secs_f64());

    write_evidence_v2(&evo_path, &phase01_result, &team_results, phase01_time, phase2_time);

    info!("Full evolution test complete.");
}

fn write_evidence_v2(
    evo_path: &str,
    phase01: &codex_ipc::DelegateResult,
    phase2: &[(String, bool, u32, u32, f64, String)],
    p01_time: std::time::Duration,
    p2_time: std::time::Duration,
) {
    let mut evidence = format!(
        "CORTEX FULL EVOLUTION v2 — {}\n\
         =====================================================\n\
         Evolution folder: {evo_path}\n\
         Seed: Sarah Chen — Verdant Thread (sustainable fashion)\n\
         Key fix: Direct ThinkLoop (3 tools) instead of ProcessBridge (19 tools)\n\
         Key fix: 6 PARALLEL teams instead of sequential\n\n\
         Phase 0+1: {:.1}s, {} iters, {} tools, completed={:?}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        p01_time.as_secs_f64(),
        phase01.iterations.unwrap_or(0),
        phase01.tool_calls_made.unwrap_or(0),
        phase01.completed,
    );

    evidence.push_str(&format!("Phase 2: {:.1}s (PARALLEL)\n", p2_time.as_secs_f64()));
    for (name, success, iters, tools, time, status) in phase2 {
        evidence.push_str(&format!(
            "  {}: {} | {}iters {}tools {:.1}s | {}\n",
            name, if *success { "PASS" } else { "FAIL" },
            iters, tools, time, status
        ));
    }
    let completed = phase2.iter().filter(|t| t.1).count();
    evidence.push_str(&format!("\nTeams completed: {}/{}\n", completed, phase2.len()));

    evidence.push_str(&format!(
        "\nTotal time: {:.1}s\n",
        (p01_time + p2_time).as_secs_f64()
    ));

    std::fs::write("evolution_full_evidence.txt", &evidence)
        .expect("Failed to write evidence");
    eprintln!("[evolution_full] Evidence written to evolution_full_evidence.txt");
}

// Keep old signature for compatibility (unused, but avoids breaking anything)
#[allow(dead_code)]
fn write_evidence(
    evo_path: &str,
    phase01: &codex_ipc::DelegateResult,
    _phase2: Option<&Vec<(&str, bool, u32, u32, f64, String)>>,
    p01_time: std::time::Duration,
    _p2_time: std::time::Duration,
) {
    let evidence = format!(
        "CORTEX FULL EVOLUTION — {}\nPhase 0+1: {:.1}s, {} iters\nEvolution folder: {evo_path}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        p01_time.as_secs_f64(),
        phase01.iterations.unwrap_or(0),
    );
    std::fs::write("evolution_full_evidence.txt", &evidence).ok();
}
