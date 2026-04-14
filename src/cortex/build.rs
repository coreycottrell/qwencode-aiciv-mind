//! Build script for Cortex — sets CODEX_UPSTREAM_PATH for the cortex-codex binary.

fn main() {
    // Determine the upstream Codex path relative to project root
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // manifest_dir is src/cortex/, so project root is two levels up
    let project_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap_or(&manifest_dir);
    let codex_path = project_root.join("codex-upstream");

    // Check if CODEX_UPSTREAM_PATH is set in environment
    let codex_upstream = std::env::var("CODEX_UPSTREAM_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or(codex_path);

    // Verify the path exists
    if !codex_upstream.exists() {
        println!(
            "cargo:warning=Codex upstream not found at {}. cortex-codex binary will not work.",
            codex_upstream.display()
        );
    }

    println!(
        "cargo:rustc-env=CODEX_UPSTREAM_PATH={}",
        codex_upstream.display()
    );

    // Rerun if the codex upstream directory changes
    println!("cargo:rerun-if-env-changed=CODEX_UPSTREAM_PATH");
}
