//! # Patch Set — Complete Cortex → Codex Integration
//!
//! Orchestrates the application of all Cortex patches to the upstream Codex
//! codebase. This is the main entry point for the build process.

use crate::{CortexPatch, PatchResult};
use crate::agent_control_patch;
use crate::session_patch;
use crate::memory_dual_write;
use crate::sandbox_bridge;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{info, warn, error};

/// The complete set of patches for integrating Cortex into Codex.
pub fn all_patches() -> Vec<CortexPatch> {
    vec![
        CortexPatch::new(
            "AgentControl Coordination Hook",
            env!("CARGO_PKG_VERSION"),
            "codex-rs/core/src/agent/control.rs",
            agent_control_patch::generate_agent_control_patch(),
        ),
        CortexPatch::new(
            "Session ThinkLoop Injection",
            env!("CARGO_PKG_VERSION"),
            "codex-rs/core/src/tasks/mod.rs",
            session_patch::generate_session_thinkloop_patch(),
        ),
        CortexPatch::new(
            "Memory Dual-Write",
            env!("CARGO_PKG_VERSION"),
            "codex-rs/core/src/rollout/mod.rs",
            memory_dual_write::generate_memory_dual_write_patch(),
        ),
        CortexPatch::new(
            "Cortex Sandbox Bridge",
            env!("CARGO_PKG_VERSION"),
            "codex-rs/core/src/exec.rs",
            sandbox_bridge::generate_sandbox_bridge_patch(),
        ),
    ]
}

/// Applies all Cortex patches to the upstream Codex directory.
///
/// # Arguments
///
/// * `codex_root` — The root directory of the upstream Codex checkout
/// * `dry_run` — If true, check which patches would apply without actually applying them
///
/// # Returns
///
/// A vector of results for each patch applied.
pub fn apply_all_patches(codex_root: &Path, dry_run: bool) -> Result<Vec<PatchResult>> {
    let patches = all_patches();
    let mut results = Vec::new();

    for patch in &patches {
        info!(
            target: "cortex::patcher",
            "Applying patch: '{}' → {}", patch.name, patch.target_file
        );

        if dry_run {
            results.push(PatchResult {
                patch_name: patch.name.clone(),
                applied: false,
                was_already_applied: check_if_already_applied(codex_root, &patch.target_file, &patch.diff)?,
                error: None,
            });
            continue;
        }

        let result = apply_single_patch(codex_root, patch)?;
        results.push(result);
    }

    info!(
        target: "cortex::patcher",
        "All patches applied: {} total", results.len()
    );

    Ok(results)
}

/// Reverts all Cortex patches from the upstream Codex directory.
pub fn revert_all_patches(codex_root: &Path) -> Result<Vec<PatchResult>> {
    let patches = all_patches();
    let mut results = Vec::new();

    // Apply patches in reverse order
    for patch in patches.iter().rev() {
        info!(
            target: "cortex::patcher",
            "Reverting patch: '{}' ← {}", patch.name, patch.target_file
        );

        let result = revert_single_patch(codex_root, patch)?;
        results.push(result);
    }

    Ok(results)
}

/// Checks whether patches are already applied (idempotency check).
pub fn check_patch_status(codex_root: &Path) -> Result<Vec<PatchResult>> {
    apply_all_patches(codex_root, true)
}

// --- Internal helpers ---

fn apply_single_patch(codex_root: &Path, patch: &CortexPatch) -> Result<PatchResult> {
    let target_path = codex_root.join(&patch.target_file);

    if !target_path.exists() {
        let error_msg = format!("Target file not found: {}", target_path.display());
        if patch.required {
            return Err(anyhow::anyhow!(error_msg));
        } else {
            warn!(target: "cortex::patcher", "{}", error_msg);
            return Ok(PatchResult {
                patch_name: patch.name.clone(),
                applied: false,
                was_already_applied: false,
                error: Some(error_msg),
            });
        }
    }

    // Check if already applied
    if check_if_already_applied(codex_root, &patch.target_file, &patch.diff)? {
        info!(
            target: "cortex::patcher",
            "Patch '{}' already applied, skipping", patch.name
        );
        return Ok(PatchResult {
            patch_name: patch.name.clone(),
            applied: false,
            was_already_applied: true,
            error: None,
        });
    }

    // Apply the patch using `patch` command
    let mut child = Command::new("patch")
        .arg("-p1")
        .arg("--no-backup-if-mismatch")
        .current_dir(codex_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn patch for '{}'", patch.name))?;

    {
        let stdin = child.stdin.as_mut()
            .with_context(|| format!("Failed to get stdin for patch '{}'", patch.name))?;
        use std::io::Write;
        stdin.write_all(patch.diff.as_bytes())
            .with_context(|| format!("Failed to write patch for '{}'", patch.name))?;
    }

    let output = child.wait_with_output()
        .with_context(|| format!("Failed to wait for patch '{}'", patch.name))?;

    if output.status.success() {
        info!(
            target: "cortex::patcher",
            "Patch '{}' applied successfully", patch.name
        );
        Ok(PatchResult {
            patch_name: patch.name.clone(),
            applied: true,
            was_already_applied: false,
            error: None,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Patch failed: {stderr}");

        if patch.required {
            error!(target: "cortex::patcher", "{}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        } else {
            warn!(target: "cortex::patcher", "{}", error_msg);
            Ok(PatchResult {
                patch_name: patch.name.clone(),
                applied: false,
                was_already_applied: false,
                error: Some(error_msg),
            })
        }
    }
}

fn revert_single_patch(codex_root: &Path, patch: &CortexPatch) -> Result<PatchResult> {
    let target_path = codex_root.join(&patch.target_file);

    if !target_path.exists() {
        return Ok(PatchResult {
            patch_name: patch.name.clone(),
            applied: false,
            was_already_applied: false,
            error: Some("Target file not found".to_string()),
        });
    }

    // Apply in reverse with -R flag
    let mut child = Command::new("patch")
        .arg("-p1")
        .arg("-R")
        .arg("--no-backup-if-mismatch")
        .current_dir(codex_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn reverse patch for '{}'", patch.name))?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        use std::io::Write;
        stdin.write_all(patch.diff.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        info!(
            target: "cortex::patcher",
            "Patch '{}' reverted successfully", patch.name
        );
        Ok(PatchResult {
            patch_name: patch.name.clone(),
            applied: true,
            was_already_applied: false,
            error: None,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(PatchResult {
            patch_name: patch.name.clone(),
            applied: false,
            was_already_applied: false,
            error: Some(format!("Revert failed: {stderr}")),
        })
    }
}

fn check_if_already_applied(
    codex_root: &Path,
    target_file: &str,
    diff: &str,
) -> Result<bool> {
    let target_path = codex_root.join(target_file);
    if !target_path.exists() {
        return Ok(false);
    }

    // Use patch --dry-run (--forward test)
    let mut child = Command::new("patch")
        .arg("-p1")
        .arg("--dry-run")
        .arg("--forward")
        .current_dir(codex_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        use std::io::Write;
        stdin.write_all(diff.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    // If --dry-run --forward fails with "already applied" message, it's already applied
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(
        stderr.contains("already") ||
        stdout.contains("already") ||
        (!output.status.success() && stderr.contains("ignoring"))
    )
}

/// Generates a build script that can be used as a Cargo build dependency.
///
/// This outputs a `build.rs` that applies all patches before the Codex build.
pub fn generate_build_script() -> String {
    r#"// build.rs — Auto-generated by codex-patcher
// Applies Cortex coordination patches to upstream Codex before building.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let codex_root = PathBuf::from(env!("CODEX_UPSTREAM_ROOT"));

    // Apply patches
    let status = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("codex-patcher")
        .arg("--bin")
        .arg("apply-patches")
        .arg("--")
        .arg(&codex_root)
        .status()
        .expect("Failed to apply Cortex patches");

    if !status.success() {
        panic!("Cortex patch application failed");
    }

    // Tell Cargo to rerun if the codex upstream changes
    println!("cargo:rerun-if-changed={}/codex-rs", codex_root.display());
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_patches_returns_four() {
        let patches = all_patches();
        assert_eq!(patches.len(), 4);
        assert_eq!(patches[0].name, "AgentControl Coordination Hook");
        assert_eq!(patches[1].name, "Session ThinkLoop Injection");
        assert_eq!(patches[2].name, "Memory Dual-Write");
        assert_eq!(patches[3].name, "Cortex Sandbox Bridge");
    }

    #[test]
    fn test_all_patches_have_diff_content() {
        let patches = all_patches();
        for patch in &patches {
            assert!(!patch.diff.is_empty(), "Patch '{}' has empty diff", patch.name);
            assert!(patch.diff.starts_with("---"), "Patch '{}' doesn't start with ---", patch.name);
        }
    }

    #[test]
    fn test_build_script_generation() {
        let script = generate_build_script();
        assert!(script.contains("build.rs"));
        assert!(script.contains("CODEX_UPSTREAM_ROOT"));
        assert!(script.contains("codex-patcher"));
    }
}
