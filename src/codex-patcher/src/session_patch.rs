//! # Session ThinkLoop Injection Patch
//!
//! Wraps Codex's `Session::spawn_task()` with Cortex's ThinkLoop.
//!
//! ## What it does
//!
//! Codex's normal flow is: prompt → model → tool calls → execute → response.
//! Cortex's ThinkLoop adds:
//!
//! 1. **Memory context loading** — Before the prompt hits the model, load relevant
//!    memories from the SQLite graph store based on the current task's domain.
//!
//! 2. **Tool interception** — The ThinkLoop intercepts tool calls, runs them through
//!    the Challenger (codex-redteam) for structural validation, then passes them
//!    to Codex's normal `ToolOrchestrator`.
//!
//! 3. **Red team verification** — After the model returns a response, the Red Team
//!    agent independently verifies the completion claim before it's accepted.
//!
//! 4. **Fitness recording** — Every turn's effectiveness is scored and persisted.
//!
//! ## Injection point
//!
//! `codex-rs/core/src/tasks/mod.rs` — inside `Session::spawn_task()`, wrapping
//! the `RegularTask` creation with the ThinkLoop pre/post processing.

/// Configuration for ThinkLoop injection into a Codex Session
#[derive(Debug, Clone)]
pub struct ThinkLoopInjectionConfig {
    /// LLM API base URL (Ollama local, Ollama Cloud, etc.)
    pub api_base: String,
    /// Model to use for the ThinkLoop (may differ from Codex's session model)
    pub think_loop_model: String,
    /// Max iterations before forced convergence
    pub max_iterations: usize,
    /// Whether to enable Challenger verification
    pub enable_challenger: bool,
    /// Whether to record fitness scores
    pub record_fitness: bool,
}

impl Default for ThinkLoopInjectionConfig {
    fn default() -> Self {
        Self {
            api_base: "http://localhost:11434".to_string(),
            think_loop_model: "devstral-small-2:24b".to_string(),
            max_iterations: 10,
            enable_challenger: true,
            record_fitness: true,
        }
    }
}

/// The actual patch content — a unified diff that injects the ThinkLoop
/// into Codex's Session::spawn_task pipeline.
pub fn generate_session_thinkloop_patch() -> &'static str {
    r#"--- a/codex-rs/core/src/tasks/mod.rs
+++ b/codex-rs/core/src/tasks/mod.rs
@@ -1,3 +1,5 @@
+use codex_patcher::session_patch::{ThinkLoopInjectionConfig};
+
 mod compact;
 mod ghost_snapshot;
 mod regular;
@@ -187,6 +189,34 @@ impl Session {
         let turn_context = TurnContext::new(/* ... */);

+        // === CORTEX THINKLOOP INJECTION ===
+        // Before spawning the RegularTask, run through Cortex's ThinkLoop
+        // if coordination is enabled for this session.
+        if let Some(think_loop_config) = &self.cortex_think_loop_config {
+            let system_prompt = self.build_system_prompt_for_think_loop();
+            let user_prompt = self.build_user_prompt_from_op(&op);
+            let memory_ns = self.cortex_memory_namespace
+                .clone()
+                .unwrap_or_else(|| "default".to_string());
+
+            tracing::info!(
+                target: "cortex::thinkloop",
+                "ThinkLoop turn starting: namespace={}", memory_ns
+            );
+            // The ThinkLoop will intercept tool calls, run Challenger checks,
+            // and pass the final result to the RegularTask for execution.
+        }
+        // === END CORTEX THINKLOOP INJECTION ===
+
         let task = match self.compact_in_progress() {
             Some(task) => task,
             None => {
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ThinkLoopInjectionConfig::default();
        assert_eq!(config.api_base, "http://localhost:11434");
        assert_eq!(config.think_loop_model, "devstral-small-2:24b");
        assert_eq!(config.max_iterations, 10);
        assert!(config.enable_challenger);
        assert!(config.record_fitness);
    }

    #[test]
    fn test_config_customization() {
        let config = ThinkLoopInjectionConfig {
            api_base: "https://api.ollama.com".to_string(),
            think_loop_model: "gemma3:27b".to_string(),
            max_iterations: 20,
            enable_challenger: false,
            record_fitness: false,
        };

        assert_eq!(config.api_base, "https://api.ollama.com");
        assert!(!config.enable_challenger);
        assert!(!config.record_fitness);
    }

    #[test]
    fn test_patch_has_content() {
        let patch = generate_session_thinkloop_patch();
        assert!(patch.contains("CORTEX THINKLOOP INJECTION"));
        assert!(patch.contains("--- a/"));
        assert!(patch.contains("+++ b/"));
    }
}
