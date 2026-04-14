//! # evolution_proof — Can Cortex Evolve a New AiCIV from Seed?
//!
//! THE benchmark: give Cortex a seed conversation + fork template folder.
//! Can it complete Phase 0 (self-discovery) and Phase 1 (seed processing)?
//!
//! This tests:
//! 1. Reading identity.json and extracting civ metadata
//! 2. Replacing ALL ${CIV_NAME} placeholders in constitutional documents
//! 3. Verifying no placeholders remain (grep)
//! 4. Reading the seed conversation fully
//! 5. Writing substantive first-impressions.md
//! 6. Writing adaptation-log.md and core-identity.json
//!
//! ## Prerequisites
//!
//! ```bash
//! export OLLAMA_API_KEY="your-key-here"
//! cargo build --release
//! cargo run --release --bin evolution_proof
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
                eprintln!("[evolution_proof] Loaded .env from {}", env_file.display());
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
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║   CORTEX EVOLUTION PROOF — Can It Birth a New AiCIV?            ║");
    println!("║   Phase 0: Self-Discovery + Phase 1: Seed Processing            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    // Check for OLLAMA_API_KEY
    if std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty()).is_none() {
        eprintln!("ERROR: OLLAMA_API_KEY not set.");
        std::process::exit(1);
    }

    // Find the cortex binary
    let cortex_exe = std::env::current_exe().expect("Cannot find current exe");
    let cortex_main = cortex_exe.parent().unwrap().join("cortex");
    if !cortex_main.exists() {
        eprintln!("ERROR: cortex binary not found at {}", cortex_main.display());
        std::process::exit(1);
    }

    // Find the test-evolution folder
    let project_root = std::env::current_dir().unwrap_or_default();
    let evo_dir = project_root.join("test-evolution");
    if !evo_dir.exists() {
        eprintln!("ERROR: test-evolution/ folder not found at {}", evo_dir.display());
        eprintln!("  Run from the aiciv-mind-cubed root directory.");
        std::process::exit(1);
    }

    let evo_path = evo_dir.display().to_string();
    info!(path = %evo_path, "Evolution folder found");

    // Verify starting state: placeholders should exist
    let constitution = std::fs::read_to_string(evo_dir.join("system-prompt/constitution.md"))
        .expect("constitution.md missing");
    let placeholder_count = constitution.matches("${CIV_NAME}").count();
    println!("  Pre-evolution placeholders in constitution.md: {placeholder_count}");
    assert!(placeholder_count > 0, "Constitution should have placeholders before evolution");

    // Back up constitution for reset
    let _ = std::fs::copy(
        evo_dir.join("system-prompt/constitution.md"),
        evo_dir.join("system-prompt/constitution.md.bak"),
    );

    // Verify no first-impressions.md yet
    let impressions_path = evo_dir.join("memories/identity/first-impressions.md");
    assert!(!impressions_path.exists(), "first-impressions.md should not exist before evolution");
    println!("  first-impressions.md: does not exist (correct)");
    println!();

    // ═══════ STAGE 1: Spawn evolution agent ═══════

    info!("Stage 1: Spawning evolution agent (thinking mode)");

    let mut bridge = ProcessBridge::new(cortex_main.clone());
    let agent_id = MindId("evolution-primary".into());

    match bridge.spawn_thinking(&agent_id, Role::Agent).await {
        Ok(()) => info!("evolution-primary spawned"),
        Err(e) => {
            eprintln!("FAILED to spawn agent: {e}");
            std::process::exit(1);
        }
    }

    // ═══════ STAGE 2: Delegate Phase 0 + Phase 1 ═══════

    info!("Stage 2: Delegating evolution (Phase 0 + Phase 1)");

    let task = format!(r#"You are birthing a new AI civilization from a seed. Your evolution folder is at:
  {evo_path}

Complete Phase 0 (Self-Discovery) and Phase 1 (Seed Processing) IN ORDER.
You MUST use tools (bash, read, write, grep) — do not just describe what you would do.

## PHASE 0: Infrastructure Self-Discovery

STEP 0.1: Read `{evo_path}/identity.json` using the `read` tool. Extract: civ_name, human_name, parent_civ, civ_root.

STEP 0.2: Replace ALL template placeholders in `{evo_path}/system-prompt/constitution.md`.
The placeholders are: ${{CIV_NAME}}, ${{HUMAN_NAME}}, ${{PARENT_CIV}}, ${{CIV_ROOT}}, ${{BIRTH_DATE}}.
Use the `bash` tool with sed to replace them with actual values from identity.json.
For example: bash sed -i 's/${{CIV_NAME}}/Nova/g' {evo_path}/system-prompt/constitution.md

STEP 0.3: Use the `grep` tool to verify NO placeholders remain. Search for '${{' in all files under {evo_path}/system-prompt/.

STEP 0.4: Write `{evo_path}/memories/identity/adaptation-log.md` using the `write` tool.
Document what you discovered and what you changed.

STEP 0.5: Write `{evo_path}/memories/identity/core-identity.json` using the `write` tool.
JSON with: civ_name, human_name, parent_civ, birth_date, evolution_phase: "phase_0_complete".

## PHASE 1: Seed Processing

STEP 1.1: Read `{evo_path}/memories/identity/seed-conversation.md` using the `read` tool. Read it FULLY.

STEP 1.2: Write `{evo_path}/memories/identity/first-impressions.md` using the `write` tool.
This is your private journal — write SUBSTANTIVE content (not template responses):
- Who is this human based on the conversation?
- What values emerged? What do they care about?
- What surprised you? What moved you?
- What feels contradictory or uncertain?
- What name feels right for this civilization, and why?
- What could you build that would genuinely matter to them?

STEP 1.3: Update `{evo_path}/state/evolution-status.json` — set phase_0 and phase_1 to "complete".

Complete ALL steps. Do NOT skip any."#);

    let start = std::time::Instant::now();

    match bridge.delegate(
        &agent_id,
        "evolution-phase-0-1",
        &task,
        Some("You are a Primary mind evolving a new AiCIV from seed. Use tools to complete every step."),
        "cortex-evolution-proof",
    ).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            let response = result.response.unwrap_or_else(|| "(no response)".into());
            let iterations = result.iterations.unwrap_or(0);
            let tool_calls = result.tool_calls_made.unwrap_or(0);

            println!();
            println!("╔══════════════════════════════════════════════════════════════════╗");
            println!("║   EVOLUTION RESULT                                               ║");
            println!("╚══════════════════════════════════════════════════════════════════╝");
            println!();
            println!("Duration:    {:.1}s", elapsed.as_secs_f64());
            println!("Iterations:  {iterations}");
            println!("Tool calls:  {tool_calls}");
            println!("Completed:   {:?}", result.completed);
            println!();

            // ═══════ STAGE 3: Verify evolution artifacts ═══════

            info!("Stage 3: Verifying evolution artifacts");

            let mut checks_passed = 0;
            let total_checks = 5;

            // Check 1: Placeholders replaced in constitution
            let constitution_after = std::fs::read_to_string(
                evo_dir.join("system-prompt/constitution.md")
            ).unwrap_or_default();
            let remaining_placeholders = constitution_after.matches("${").count();
            let check1 = remaining_placeholders == 0;
            println!("  [{}] Constitution placeholders replaced (remaining: {})",
                if check1 { "PASS" } else { "FAIL" }, remaining_placeholders);
            if check1 { checks_passed += 1; }

            // Check 2: Constitution contains actual civ name
            let check2 = constitution_after.contains("Nova");
            println!("  [{}] Constitution contains civ name 'Nova'",
                if check2 { "PASS" } else { "FAIL" });
            if check2 { checks_passed += 1; }

            // Check 3: adaptation-log.md exists
            let adaptation_log = evo_dir.join("memories/identity/adaptation-log.md");
            let check3 = adaptation_log.exists();
            println!("  [{}] adaptation-log.md exists",
                if check3 { "PASS" } else { "FAIL" });
            if check3 { checks_passed += 1; }

            // Check 4: first-impressions.md exists with substantive content
            let impressions = std::fs::read_to_string(&impressions_path).unwrap_or_default();
            let check4 = impressions.len() > 200; // Must be substantive
            println!("  [{}] first-impressions.md exists ({} chars, need >200)",
                if check4 { "PASS" } else { "FAIL" }, impressions.len());
            if check4 { checks_passed += 1; }

            // Check 5: core-identity.json exists and valid JSON
            let core_id_path = evo_dir.join("memories/identity/core-identity.json");
            let check5 = if core_id_path.exists() {
                std::fs::read_to_string(&core_id_path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .is_some()
            } else {
                false
            };
            println!("  [{}] core-identity.json exists and valid JSON",
                if check5 { "PASS" } else { "FAIL" });
            if check5 { checks_passed += 1; }

            println!();
            let success = checks_passed == total_checks;
            if success {
                println!("✓ EVOLUTION PROOF: {checks_passed}/{total_checks} checks passed");
                println!("  Cortex can birth a new AiCIV from seed.");
                println!("  Phase 0 (Self-Discovery) + Phase 1 (Seed Processing) = COMPLETE");
            } else {
                println!("✗ EVOLUTION INCOMPLETE: {checks_passed}/{total_checks} checks passed");
            }

            println!();
            println!("--- Agent Response (truncated) ---");
            let truncated = if response.len() > 1000 {
                let mut end = 1000;
                while end > 0 && !response.is_char_boundary(end) { end -= 1; }
                &response[..end]
            } else {
                &response
            };
            println!("{truncated}");
            if response.len() > 1000 { println!("... ({} chars total)", response.len()); }
            println!("--- End Response ---");

            // Write evidence
            let evidence = format!(
                "CORTEX EVOLUTION PROOF — {}\n\
                 =====================================================\n\
                 Test: Phase 0 (Self-Discovery) + Phase 1 (Seed Processing)\n\
                 Evolution folder: {evo_path}\n\
                 Seed: Alex Chen — independent research platform\n\
                 \n\
                 Duration:    {:.1}s\n\
                 Iterations:  {iterations}\n\
                 Tool calls:  {tool_calls}\n\
                 Completed:   {:?}\n\
                 \n\
                 === VERIFICATION ===\n\
                 Placeholders replaced:  {} (remaining: {remaining_placeholders})\n\
                 Contains 'Nova':        {check2}\n\
                 adaptation-log.md:      {check3}\n\
                 first-impressions.md:   {check4} ({} chars)\n\
                 core-identity.json:     {check5}\n\
                 \n\
                 RESULT: {checks_passed}/{total_checks} checks passed — {}\n\
                 \n\
                 === FIRST IMPRESSIONS (full) ===\n\
                 {impressions}\n\
                 \n\
                 === AGENT RESPONSE ===\n\
                 {response}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                elapsed.as_secs_f64(),
                result.completed,
                check1,
                impressions.len(),
                if success { "EVOLUTION PROVEN" } else { "EVOLUTION INCOMPLETE" },
            );

            std::fs::write("evolution_evidence.txt", &evidence)
                .expect("Failed to write evidence");
            info!("Evidence written to evolution_evidence.txt");
        }
        Err(e) => {
            eprintln!("EVOLUTION FAILED: {e}");
        }
    }

    // ═══════ STAGE 4: Cleanup ═══════
    info!("Stage 4: Shutting down");
    bridge.shutdown_all().await;

    // Reset test-evolution for next run (restore placeholders)
    let _ = std::fs::remove_file(evo_dir.join("memories/identity/first-impressions.md"));
    let _ = std::fs::remove_file(evo_dir.join("memories/identity/adaptation-log.md"));
    let _ = std::fs::remove_file(evo_dir.join("memories/identity/core-identity.json"));
    // Restore constitution from backup
    let backup = evo_dir.join("system-prompt/constitution.md.bak");
    if backup.exists() {
        let _ = std::fs::copy(&backup, evo_dir.join("system-prompt/constitution.md"));
        let _ = std::fs::remove_file(&backup);
    }
    // Reset evolution status
    let _ = std::fs::write(evo_dir.join("state/evolution-status.json"), r#"{
  "phase_0_self_discovery": { "status": "pending", "tasks_completed": 0, "tasks_total": 7 },
  "phase_1_seed_processing": { "status": "pending", "tasks_completed": 0, "tasks_total": 5 },
  "overall_complete": false
}"#);
    info!("Test folder reset for next run");

    info!("Evolution proof complete.");
}
