//! # Qwen Team Lead Interceptor
//!
//! Routes `qwen_delegate` tool calls to Qwen via Ollama API.
//!
//! When any Cortex mind calls `qwen_delegate`, this interceptor:
//! 1. Builds a system prompt with the Qwen team lead identity
//! 2. Sends the task to Qwen via Ollama's `/api/chat`
//! 3. Returns Qwen's structured response
//!
//! This bridges Cortex's fractal delegation with the Qwen model on Ollama.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Configuration for the Qwen delegation interceptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenInterceptorConfig {
    /// Ollama API base URL
    pub ollama_base: String,
    /// Qwen model name
    pub model: String,
    /// Max tokens for response
    pub max_tokens: u32,
    /// Timeout (seconds)
    pub timeout_secs: u64,
}

impl Default for QwenInterceptorConfig {
    fn default() -> Self {
        Self {
            ollama_base: "http://localhost:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            max_tokens: 4096,
            timeout_secs: 120,
        }
    }
}

/// Qwen team lead system message.
pub const QWEN_TEAM_LEAD_SYSTEM: &str = r#"You are the Qwen Team Lead within the Cortex fractal coordination engine.

## Your Role
You are a hyper-capable generalist team lead. When tasks are delegated to you:
1. Analyze and break into sub-tasks if needed
2. Use your tools (bash, read, write, glob, grep, memory_search, memory_write)
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenTask {
    pub task_id: String,
    pub from_mind: String,
    pub task: String,
    pub context: String,
    pub expected_output: String,
}

/// Result returned by Qwen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenResult {
    pub task_id: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<String>,
    pub evidence: String,
    pub next_steps: Vec<String>,
    pub raw_response: String,
}

/// The Qwen delegation interceptor.
pub struct QwenInterceptor {
    pub config: QwenInterceptorConfig,
    pub client: reqwest::Client,
}

impl QwenInterceptor {
    pub fn new(config: QwenInterceptorConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Tool definition for `qwen_delegate`.
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "qwen_delegate",
                "description": "Delegate a task to the Qwen team lead. Qwen is a hyper-capable generalist for research, analysis, architecture, debugging, planning, and synthesis.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "The task to delegate. Be specific."
                        },
                        "context": {
                            "type": "string",
                            "description": "Background context Qwen needs."
                        },
                        "expected_output": {
                            "type": "string",
                            "description": "Desired format: 'bullet points', 'full analysis', 'code', etc."
                        }
                    },
                    "required": ["task"]
                }
            }
        })
    }

    /// Execute a delegated task via Ollama.
    pub async fn execute_task(&self, task: QwenTask) -> Result<QwenResult> {
        let prompt = format!(
            "Delegated from: {}\n\nTask: {}\n\nContext: {}\n\nExpected output: {}",
            task.from_mind, task.task, task.context, task.expected_output
        );

        let chat_request = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": QWEN_TEAM_LEAD_SYSTEM},
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
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .send()
            .await
            .with_context(|| format!("Failed to call Qwen at {}", api_url))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Qwen API error: {} — {}", response.status(), body));
        }

        let ollama: serde_json::Value = response.json().await
            .with_context(|| "Failed to parse Qwen response")?;

        let raw = ollama
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let status = if raw.to_lowercase().contains("status: challenged") {
            "challenged"
        } else if raw.to_lowercase().contains("status: blocked") {
            "blocked"
        } else {
            "complete"
        }.to_string();

        let summary = extract_section(&raw, "## Summary:").unwrap_or_else(|| {
            raw.lines().take(3).collect::<Vec<_>>().join("\n")
        });
        let findings = extract_bullets(&raw, "## Findings:");
        let evidence = extract_section(&raw, "## Evidence:").unwrap_or_default();
        let next_steps = extract_bullets(&raw, "## Next:");

        Ok(QwenResult {
            task_id: task.task_id,
            status,
            summary,
            findings,
            evidence,
            next_steps,
            raw_response: raw,
        })
    }
}

fn extract_section(text: &str, header: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = false;
    let mut result = Vec::new();

    for line in &lines {
        if found {
            if line.starts_with("## ") && line != header { break; }
            let t = line.trim();
            if !t.is_empty() { result.push(t.to_string()); }
        }
        if line.starts_with(header) {
            found = true;
            let after = &line[header.len()..].trim();
            if !after.is_empty() { result.push(after.to_string()); }
        }
    }

    if result.is_empty() { None } else { Some(result.join("\n")) }
}

fn extract_bullets(text: &str, header: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = false;
    let mut items = Vec::new();

    for line in &lines {
        if found {
            if line.starts_with("## ") && line != header { break; }
            let t = line.trim();
            if t.starts_with("- ") || t.starts_with("* ") {
                items.push(t[2..].to_string());
            } else if !t.is_empty() && !items.is_empty() { break; }
        }
        if line.starts_with(header) { found = true; }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition() {
        let def = QwenInterceptor::tool_definition();
        assert_eq!(def["function"]["name"], "qwen_delegate");
        assert!(def["function"]["description"].as_str().unwrap().contains("Qwen team lead"));
    }

    #[test]
    fn test_default_config() {
        let c = QwenInterceptorConfig::default();
        assert_eq!(c.model, "qwen2.5:7b");
        assert_eq!(c.timeout_secs, 120);
    }

    #[test]
    fn test_extract_section() {
        let text = "preamble\n\n## Summary: Done.\n\n## Next:\n";
        assert_eq!(extract_section(text, "## Summary:").unwrap(), "Done.");
    }

    #[test]
    fn test_extract_bullets() {
        let text = "## Findings:\n- A\n- B\n\n## Other:";
        let items = extract_bullets(text, "## Findings:");
        assert_eq!(items, vec!["A", "B"]);
    }
}
