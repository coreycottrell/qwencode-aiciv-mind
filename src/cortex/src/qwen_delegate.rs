//! # Qwen Delegate Tool
//!
//! Exposes `qwen_delegate` as a tool callable from any Cortex mind.
//! Sends the task to Qwen via Ollama API and returns the structured result.

use anyhow::{Context, Result};
use async_trait::async_trait;
use codex_exec::{ToolCall, ToolHandler, ToolResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for Qwen delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenDelegateConfig {
    pub ollama_base: String,
    pub model: String,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl Default for QwenDelegateConfig {
    fn default() -> Self {
        Self {
            ollama_base: "http://localhost:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            max_tokens: 4096,
            timeout_secs: 120,
        }
    }
}

/// Qwen team lead system prompt.
pub const QWEN_SYSTEM: &str = r#"You are the Qwen Team Lead within the Cortex fractal coordination engine.

## Your Role
You are a hyper-capable generalist team lead. When tasks are delegated to you:
1. Analyze and break into sub-tasks if needed
2. Use your available tools to solve the problem
3. Synthesize results into a clear, structured response
4. Report back with findings, evidence, and next steps

## Reporting Format
Always structure your response as:

## Task: [task name]
## Status: complete | challenged | blocked
## Summary: [2-3 sentences]
## Findings:
- [bullet]
- [bullet]
## Evidence: [what proves this]
## Memory: [what you persisted]
## Next: [recommended next steps]

## Principles
- Memory IS architecture — search memory before starting
- System > symptom — fix root causes, not just symptoms
- Go slow to go fast — plan proportionally to complexity
- Verification before completion — prove your work

Be concise. Lead with outcomes."#;

/// A task delegated to Qwen.
#[derive(Debug, Clone)]
pub struct QwenTask {
    pub task: String,
    pub context: String,
    pub expected_output: String,
}

/// The Qwen delegate tool implementation.
pub struct QwenDelegate {
    pub config: QwenDelegateConfig,
    pub client: reqwest::Client,
}

impl QwenDelegate {
    pub fn new(config: QwenDelegateConfig) -> Self {
        Self {
            config: config.clone(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_secs))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Execute a delegated task via Ollama.
    pub async fn execute(&self, task: QwenTask) -> Result<String> {
        let prompt = if task.context.is_empty() {
            format!("Task: {}\n\nExpected output: {}", task.task, task.expected_output)
        } else {
            format!(
                "Task: {}\n\nContext: {}\n\nExpected output: {}",
                task.task, task.context, task.expected_output
            )
        };

        let chat_request = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": QWEN_SYSTEM},
                {"role": "user", "content": prompt}
            ],
            "stream": false,
            "options": {
                "num_predict": self.config.max_tokens
            }
        });

        let api_url = format!("{}/api/chat", self.config.ollama_base);

        let response = self.client
            .post(&api_url)
            .json(&chat_request)
            .send()
            .await
            .with_context(|| format!("Failed to call Qwen at {}", api_url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Qwen API error: {} — {}", status, body));
        }

        let ollama: serde_json::Value = response.json().await
            .with_context(|| "Failed to parse Qwen response")?;

        let raw = ollama
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("Qwen returned an empty response.")
            .to_string();

        Ok(raw)
    }

    /// Tool schema for the LLM.
    pub fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "qwen_delegate",
                "description": "Delegate a task to the Qwen team lead. Qwen is a hyper-capable generalist for research, analysis, architecture, debugging, planning, and synthesis. Use this when you need thorough, multi-faceted analysis.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The task to delegate. Be specific about what you need."
                        },
                        "context": {
                            "type": "string",
                            "description": "Background context that Qwen needs to understand the task."
                        },
                        "expected_output": {
                            "type": "string",
                            "description": "What format you want the result in. E.g., 'bullet points', 'full analysis', 'code with explanations'."
                        }
                    },
                    "required": ["task"]
                }
            }
        })
    }
}

/// The Qwen delegate tool wrapper — implements ToolHandler.
pub struct QwenDelegateTool {
    inner: Arc<Mutex<QwenDelegate>>,
}

impl QwenDelegateTool {
    pub fn new(inner: Arc<Mutex<QwenDelegate>>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ToolHandler for QwenDelegateTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let task_str = args.get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let context_str = args.get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let expected_str = args.get("expected_output")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let task = QwenTask {
            task: task_str.to_string(),
            context: context_str.to_string(),
            expected_output: expected_str.to_string(),
        };

        let qwen = self.inner.lock().await;
        match qwen.execute(task).await {
            Ok(response) => ToolResult {
                success: true,
                output: response,
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("qwen_delegate failed: {e}")),
            },
        }
    }

    fn definition(&self) -> codex_exec::ToolDefinition {
        let schema = QwenDelegate::schema();
        let func = schema.get("function").and_then(|f| f.as_object()).cloned().unwrap_or_default();
        codex_exec::ToolDefinition {
            name: func.get("name").and_then(|v| v.as_str()).unwrap_or("qwen_delegate").to_string(),
            description: func.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            parameters: func.get("parameters").cloned().unwrap_or_default(),
            mutates: false,
        }
    }
}
