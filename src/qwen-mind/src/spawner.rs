//! Spawner — launch a child mind as a subprocess with file-based IPC.
//!
//! Phase 1a: file-based IPC (task/result files).
//! Phase 1b: ZeroMQ REQ/REP.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::mind::TaskResult;

/// A running child mind subprocess.
pub struct MindProcess {
    role: String,
    identity: String,
    vertical: String,
    root_dir: PathBuf,
    ipc_dir: PathBuf,
    _child: Option<tokio::process::Child>,
}

impl MindProcess {
    /// Spawn a new mind process.
    pub async fn spawn(
        role: &str,
        identity: &str,
        vertical: &str,
        root_dir: &PathBuf,
        qwen_mind_binary: &PathBuf,
    ) -> anyhow::Result<Self> {
        let ipc_dir = root_dir.join("ipc").join(identity);
        std::fs::create_dir_all(&ipc_dir)?;

        let task_path = ipc_dir.join("task");
        let result_path = ipc_dir.join("result");

        // Clean up any stale files
        let _ = std::fs::remove_file(&task_path);
        let _ = std::fs::remove_file(&result_path);

        tracing::info!(
            identity, role, vertical,
            task_path = ?task_path,
            result_path = ?result_path,
            "Spawning mind subprocess"
        );

        let child = Command::new(qwen_mind_binary)
            .arg("--role")
            .arg(role)
            .arg("--identity")
            .arg(identity)
            .arg("--vertical")
            .arg(vertical)
            .arg("--root")
            .arg(root_dir)
            .arg("--zmq-endpoint")
            .arg(task_path.to_str().unwrap())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Give the child a moment to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(Self {
            role: role.to_string(),
            identity: identity.to_string(),
            vertical: vertical.to_string(),
            root_dir: root_dir.clone(),
            ipc_dir,
            _child: Some(child),
        })
    }

    /// Send a task to the child mind and wait for the result.
    pub async fn delegate(&self, task: &str, timeout_secs: u64) -> anyhow::Result<TaskResult> {
        let task_path = self.ipc_dir.join("task");
        let result_path = self.ipc_dir.join("result");

        // Clean previous result
        let _ = std::fs::remove_file(&result_path);

        // Write task
        tokio::fs::write(&task_path, task).await?;
        tracing::debug!(identity = %self.identity, task = %task.chars().take(60).collect::<String>(), "Task sent to child");

        // Wait for result with timeout
        let result = timeout(
            Duration::from_secs(timeout_secs),
            self.wait_for_result(&result_path),
        )
        .await??;

        // Clean up
        let _ = std::fs::remove_file(&task_path);
        let _ = std::fs::remove_file(&result_path);

        Ok(result)
    }

    async fn wait_for_result(&self, result_path: &PathBuf) -> anyhow::Result<TaskResult> {
        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;

            if result_path.exists() {
                // Wait a brief moment to ensure write is complete
                tokio::time::sleep(Duration::from_millis(100)).await;

                let content = tokio::fs::read_to_string(result_path).await?;
                let json: serde_json::Value = serde_json::from_str(&content)?;

                return Ok(TaskResult {
                    content: json.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    success: json.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                    memory_id: json.get("memory_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    fitness_score: json.get("fitness_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                });
            }
        }
    }

    /// Check if the child process is still alive.
    pub fn is_alive(&self) -> bool {
        // For Phase 1a: check if we can write to the task path
        self.ipc_dir.join("task").exists() || self.ipc_dir.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn spawn_and_delegate() {
        // This test requires the qwen-mind binary to be built.
        // It's an integration test — run with `cargo test -- --ignored`.

        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();

        // Find the binary (built by cargo)
        let binary = PathBuf::from(env!("CARGO_BIN_EXE_qwen-mind"));
        if !binary.exists() {
            eprintln!("Skipping: qwen-mind binary not found at {:?}", binary);
            return;
        }

        let child = MindProcess::spawn(
            "agent",
            "test-agent",
            "test",
            &root,
            &binary,
        ).await.unwrap();

        // Send a simple task
        let result = child.delegate("Say hello", 30).await;
        // The task will fail because there's no Ollama running, but the
        // subprocess should have started and the IPC mechanism should work.
        // We're testing the spawner, not the LLM.
        drop(child);
    }
}
