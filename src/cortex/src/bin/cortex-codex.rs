//! # cortex-codex — Patched Codex with Cortex Coordination
//!
//! This binary applies Cortex coordination patches to the upstream Codex
//! codebase, then builds and runs the patched Codex with fractal intelligence.
//!
//! ## Usage
//!
//! ```bash
//! # Apply patches and check status
//! cargo run --bin cortex-codex -- status
//!
//! # Apply patches
//! cargo run --bin cortex-codex -- patch
//!
//! # Revert patches
//! cargo run --bin cortex-codex -- revert
//!
//! # Build the patched Codex
//! cargo run --bin cortex-codex -- build
//!
//! # Run the patched Codex (default)
//! cargo run --bin cortex-codex -- run
//! ```

use anyhow::{Context, Result};
use codex_patcher::patch_set;
use std::path::PathBuf;
use std::process::Command;

const CODEX_UPSTREAM: &str = env!("CODEX_UPSTREAM_PATH");

fn codex_root() -> PathBuf {
    PathBuf::from(CODEX_UPSTREAM)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(|s| s.as_str()).unwrap_or("status");

    match command {
        "status" => cmd_status(),
        "patch" => cmd_patch(),
        "revert" => cmd_revert(),
        "build" => cmd_build(),
        "run" | "start" => cmd_run(&args[1..]),
        "help" | "--help" | "-h" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {command}");
            cmd_help()?;
            std::process::exit(1);
        }
    }
}

fn cmd_status() -> Result<()> {
    println!("=== Cortex → Codex Patch Status ===\n");
    let codex_root = codex_root();

    if !codex_root.exists() {
        eprintln!("ERROR: Codex upstream not found at: {}", codex_root.display());
        eprintln!("Set CODEX_UPSTREAM_PATH env var or clone Codex to: {CODEX_UPSTREAM}");
        std::process::exit(1);
    }

    println!("Codex root: {}", codex_root.display());
    println!();

    let results = patch_set::check_patch_status(&codex_root)
        .with_context(|| "Failed to check patch status")?;

    let mut applied = 0;
    let mut not_applied = 0;
    let mut already_applied = 0;

    for result in &results {
        if result.was_already_applied {
            println!("  ✅ {} (already applied)", result.patch_name);
            already_applied += 1;
        } else if result.applied {
            println!("  ✅ {} (applied)", result.patch_name);
            applied += 1;
        } else if let Some(err) = &result.error {
            println!("  ❌ {} ({})", result.patch_name, err);
        } else {
            println!("  ⏳ {} (not applied)", result.patch_name);
            not_applied += 1;
        }
    }

    println!();
    println!("Summary: {} applied, {} already applied, {} not applied",
        applied, already_applied, not_applied);

    Ok(())
}

fn cmd_patch() -> Result<()> {
    println!("=== Applying Cortex Patches to Codex ===\n");
    let codex_root = codex_root();

    if !codex_root.exists() {
        eprintln!("ERROR: Codex upstream not found at: {}", codex_root.display());
        std::process::exit(1);
    }

    let results = patch_set::apply_all_patches(&codex_root, false)
        .with_context(|| "Failed to apply patches")?;

    let mut success = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for result in &results {
        if result.applied {
            println!("  ✅ {}", result.patch_name);
            success += 1;
        } else if result.was_already_applied {
            println!("  ⏭ {} (already applied)", result.patch_name);
            skipped += 1;
        } else if let Some(err) = &result.error {
            println!("  ❌ {}: {}", result.patch_name, err);
            failed += 1;
        }
    }

    println!();
    println!("Patches: {} applied, {} skipped, {} failed", success, skipped, failed);

    if failed > 0 {
        std::process::exit(1);
    }

    println!("\nDone! Run 'cargo run --bin cortex-codex -- build' to build patched Codex.");
    Ok(())
}

fn cmd_revert() -> Result<()> {
    println!("=== Reverting Cortex Patches ===\n");
    let codex_root = codex_root();

    let results = patch_set::revert_all_patches(&codex_root)
        .with_context(|| "Failed to revert patches")?;

    for result in &results {
        if result.applied {
            println!("  ✅ Reverted: {}", result.patch_name);
        } else {
            println!("  ⏭ Skipped: {}", result.patch_name);
        }
    }

    Ok(())
}

fn cmd_build() -> Result<()> {
    println!("=== Building Patched Codex ===\n");
    let codex_root = codex_root();

    if !codex_root.exists() {
        eprintln!("ERROR: Codex upstream not found");
        std::process::exit(1);
    }

    // Apply patches first
    patch_set::apply_all_patches(&codex_root, false)
        .with_context(|| "Failed to apply patches before build")?;

    println!("\nBuilding Codex with Cortex coordination...\n");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .arg("codex")
        .current_dir(&codex_root)
        .status()
        .with_context(|| "Failed to run cargo build")?;

    if !status.success() {
        eprintln!("\nERROR: Build failed");
        std::process::exit(1);
    }

    println!("\n✅ Build successful! Run 'cargo run --bin cortex-codex -- run' to start.");
    Ok(())
}

fn cmd_run(args: &[String]) -> Result<()> {
    println!("=== Running Patched Codex with Cortex Coordination ===\n");

    // First ensure patches are applied
    let codex_root = codex_root();
    patch_set::apply_all_patches(&codex_root, false)
        .with_context(|| "Failed to apply patches before run")?;

    // Build
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .arg("codex")
        .current_dir(&codex_root)
        .status()
        .with_context(|| "Failed to build Codex")?;

    if !build_status.success() {
        eprintln!("ERROR: Build failed");
        std::process::exit(1);
    }

    // Run with any additional args passed through
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--release")
        .arg("--bin")
        .arg("codex")
        .current_dir(&codex_root);

    for arg in args {
        cmd.arg(arg);
    }

    println!("Launching Codex with Cortex coordination layer...\n");

    let status = cmd.status()
        .with_context(|| "Failed to run Codex")?;

    if !status.success() {
        eprintln!("\nCodex exited with non-zero status: {}", status);
    }

    Ok(())
}

fn cmd_help() -> Result<()> {
    println!(
        r#"cortex-codex — Patched Codex with Cortex Coordination

Usage:
  cargo run --bin cortex-codex -- [command]

Commands:
  status   Check which patches are applied (default)
  patch    Apply Cortex patches to upstream Codex
  revert   Revert Cortex patches from upstream Codex
  build    Build the patched Codex binary
  run      Build and run the patched Codex
  help     Show this help message

Environment:
  CODEX_UPSTREAM_PATH  Path to the upstream Codex checkout
                       (default: {default})

Patches Applied:
  1. AgentControl Coordination Hook — fractal hierarchy enforcement
  2. Session ThinkLoop Injection — cognitive loop + Challenger + fitness
  3. Memory Dual-Write — graph memory alongside rollout JSONL
  4. Cortex Sandbox Bridge — role-based sandbox via Codex's bubblewrap/seccomp"#,
        default = CODEX_UPSTREAM,
    );
    Ok(())
}
