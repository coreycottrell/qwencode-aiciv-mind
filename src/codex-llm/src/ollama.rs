//! Ollama client — native `/api/chat` endpoint with tool use.
//!
//! Retry policy: transient errors (HTTP 429/500/502/503/504 and connection errors)
//! are retried up to 3 times with exponential backoff (1s, 2s, 4s).
//! Non-retryable errors (4xx except 429, parse errors) fail immediately.

use async_trait::async_trait;

use crate::provider::LlmProvider;
use crate::rate_limiter::RateLimiter;
use codex_exec::ToolDefinition;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Truncate a string to at most `max` bytes, landing on a valid UTF-8 char boundary.
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

/// Configuration for the Ollama connection.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    /// Optional API key for cloud-hosted models (e.g., Ollama Cloud).
    /// When set, sent as `Authorization: Bearer <key>`.
    pub api_key: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".into(),
            model: "qwen2.5:7b".into(),
            temperature: 1.0,
            max_tokens: 4096,
            api_key: None,
        }
    }
}

/// Model routing — selects the right model for each role and task type.
///
/// Gemma 4 for orchestration/planning (Primary, TeamLead).
/// M2.7 for lightweight tasks (red team, memory extraction).
/// Configurable per-role model assignment.
#[derive(Debug, Clone)]
pub struct ModelRouter {
    /// Model for Primary minds (orchestration, planning).
    pub primary_model: String,
    /// Model for TeamLead minds (delegation, synthesis).
    pub team_lead_model: String,
    /// Model for Agent minds (execution, reasoning).
    pub agent_model: String,
    /// Lightweight model for red team, memory scoring, trivial planning.
    pub lightweight_model: String,
    /// Base URL for the model provider.
    pub base_url: String,
    /// Optional API key.
    pub api_key: Option<String>,
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self {
            primary_model: "qwen2.5:7b".into(),
            team_lead_model: "qwen2.5:7b".into(),
            agent_model: "qwen2.5:7b".into(),
            lightweight_model: "phi3:mini".into(),
            base_url: "http://localhost:11434".into(),
            api_key: None,
        }
    }
}

impl ModelRouter {
    /// Create a router for Ollama Cloud with Devstral Small 2 + M2.7.
    ///
    /// Devstral (Mistral) supports native structured tool calling with zero thinking overhead.
    /// Thinking models (Qwen 3, Gemma 3) waste output budget on chain-of-thought before tool calls.
    /// Non-thinking models with tool support are better for orchestration.
    pub fn cloud(api_key: impl Into<String>) -> Self {
        Self {
            primary_model: "devstral-small-2:24b".into(),
            team_lead_model: "devstral-small-2:24b".into(),
            agent_model: "devstral-small-2:24b".into(),
            lightweight_model: "minimax-m2.7".into(),
            base_url: "https://api.ollama.com".into(),
            api_key: Some(api_key.into()),
        }
    }

    /// Create a router from environment variables.
    ///
    /// Reads OLLAMA_API_KEY, CORTEX_PRIMARY_MODEL, CORTEX_AGENT_MODEL, etc.
    pub fn from_env() -> Self {
        let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| if api_key.is_some() {
                "https://api.ollama.com".into()
            } else {
                "http://localhost:11434".into()
            });

        Self {
            primary_model: std::env::var("CORTEX_PRIMARY_MODEL")
                .unwrap_or_else(|_| "devstral-small-2:24b".into()),
            team_lead_model: std::env::var("CORTEX_TEAM_LEAD_MODEL")
                .unwrap_or_else(|_| "devstral-small-2:24b".into()),
            agent_model: std::env::var("CORTEX_AGENT_MODEL")
                .unwrap_or_else(|_| "devstral-small-2:24b".into()),
            lightweight_model: std::env::var("CORTEX_LIGHTWEIGHT_MODEL")
                .unwrap_or_else(|_| "minimax-m2.7".into()),
            base_url,
            api_key,
        }
    }

    /// Get the OllamaConfig for a specific role.
    ///
    /// Thinking models (Qwen 3) consume generation budget for chain-of-thought,
    /// so cloud configs get a higher max_tokens to leave room for tool calls.
    pub fn config_for_role(&self, role: codex_roles::Role) -> OllamaConfig {
        let model = match role {
            codex_roles::Role::Primary => &self.primary_model,
            codex_roles::Role::TeamLead => &self.team_lead_model,
            codex_roles::Role::Agent => &self.agent_model,
        };

        // Thinking models need more tokens — thinking uses generation budget
        let max_tokens = if self.api_key.is_some() { 16384 } else { 4096 };

        OllamaConfig {
            base_url: self.base_url.clone(),
            model: model.clone(),
            temperature: 1.0,
            max_tokens,
            api_key: self.api_key.clone(),
        }
    }

    /// Get the OllamaConfig for lightweight tasks (red team, scoring).
    pub fn config_lightweight(&self) -> OllamaConfig {
        OllamaConfig {
            base_url: self.base_url.clone(),
            model: self.lightweight_model.clone(),
            temperature: 1.0,
            max_tokens: 2048,
            api_key: self.api_key.clone(),
        }
    }
}

impl OllamaConfig {
    /// Create a cloud config for models hosted on Ollama Cloud.
    pub fn cloud(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: "https://api.ollama.com".into(),
            model: model.into(),
            temperature: 1.0,
            max_tokens: 4096,
            api_key: Some(api_key.into()),
        }
    }

    /// Create config from environment, checking OLLAMA_API_KEY.
    pub fn from_env(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());
        Self {
            base_url: base_url.into(),
            model: model.into(),
            temperature: 1.0,
            max_tokens: 4096,
            api_key,
        }
    }
}

/// Ollama client using the native `/api/chat` endpoint.
///
/// Supports both local Ollama and Ollama Cloud with Bearer token auth.
/// Uses Ollama's native API (not OpenAI-compatible), since Ollama Cloud's
/// `/v1/` endpoint doesn't support all auth modes.
pub struct OllamaClient {
    config: OllamaConfig,
    http: Client,
    /// Optional rate limiter — tracks usage and provides circuit breaker protection.
    rate_limiter: Option<RateLimiter>,
}

/// Native Ollama API response (different from OpenAI format).
#[derive(Debug, Clone, Deserialize)]
struct NativeOllamaResponse {
    #[allow(dead_code)]
    model: Option<String>,
    message: Option<NativeMessage>,
    done: Option<bool>,
    done_reason: Option<String>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

/// Native Ollama message format.
#[derive(Debug, Clone, Deserialize)]
struct NativeMessage {
    #[allow(dead_code)]
    role: Option<String>,
    content: Option<String>,
    /// Chain-of-thought reasoning (some models like Qwen 3 emit this).
    #[serde(default)]
    #[allow(dead_code)]
    thinking: Option<String>,
    tool_calls: Option<Vec<NativeToolCall>>,
}

/// Native Ollama tool call.
/// Some models (Qwen 3) include `id`, others don't — we generate if missing.
#[derive(Debug, Clone, Deserialize)]
struct NativeToolCall {
    /// Tool call ID (present in Qwen 3, absent in Gemma 3).
    id: Option<String>,
    function: NativeFunctionCall,
}

#[derive(Debug, Clone, Deserialize)]
struct NativeFunctionCall {
    name: String,
    arguments: serde_json::Value,
    /// Function index (some models include this).
    #[serde(default)]
    #[allow(dead_code)]
    index: Option<u32>,
}

/// Maximum number of retries for transient LLM errors.
const MAX_RETRIES: u32 = 3;

/// Base backoff duration in milliseconds (doubles each retry: 1s, 2s, 4s).
const BASE_BACKOFF_MS: u64 = 1000;

/// Request timeout for LLM calls (5 minutes — large prompts with tools take time).
const REQUEST_TIMEOUT_SECS: u64 = 300;

/// Connect timeout for the HTTP client.
const CONNECT_TIMEOUT_SECS: u64 = 10;

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        let http = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            config,
            http,
            rate_limiter: None,
        }
    }

    /// Attach a rate limiter to this client.
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(limiter);
        self
    }

    /// Get a reference to the rate limiter, if configured.
    pub fn rate_limiter(&self) -> Option<&RateLimiter> {
        self.rate_limiter.as_ref()
    }

    /// Send a chat request using Ollama's native `/api/chat` endpoint.
    ///
    /// Retries transient errors (HTTP 429/500/502/503/504, connection errors)
    /// up to 3 times with exponential backoff. Non-retryable errors fail immediately.
    ///
    /// The response is converted to the internal `ChatResponse` format
    /// (OpenAI-compatible) so the rest of the codebase doesn't change.
    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolSchema]>,
    ) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/api/chat", self.config.base_url);

        // Convert messages to native Ollama format — arguments must be objects, not strings
        let native_messages: Vec<serde_json::Value> = messages.iter().map(|msg| {
            let mut m = serde_json::json!({ "role": msg.role });
            if let Some(ref content) = msg.content {
                m["content"] = serde_json::Value::String(content.clone());
            }
            if let Some(ref tool_calls) = msg.tool_calls {
                let native_tcs: Vec<serde_json::Value> = tool_calls.iter().map(|tc| {
                    // Parse arguments string back to JSON object for native API
                    let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    serde_json::json!({
                        "function": {
                            "name": tc.function.name,
                            "arguments": args,
                        }
                    })
                }).collect();
                m["tool_calls"] = serde_json::Value::Array(native_tcs);
            }
            m
        }).collect();

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": native_messages,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            },
            "stream": false,
        });

        if let Some(tools) = tools {
            if !tools.is_empty() {
                body["tools"] = serde_json::to_value(tools).unwrap();
            }
        }

        debug!(model = %self.config.model, url = %url, msgs = messages.len(), cloud = self.config.api_key.is_some(), "Sending chat request");

        // Circuit breaker check — if open, fail fast
        if let Some(ref limiter) = self.rate_limiter {
            if let Err(secs) = limiter.check().await {
                warn!(remaining_secs = secs, "Circuit breaker OPEN — request blocked");
                return Err(LlmError::Api {
                    status: 429,
                    body: format!("Circuit breaker open — {secs}s remaining in cooldown"),
                });
            }
        }

        // Retry loop for transient errors
        let mut last_err: Option<LlmError> = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = std::time::Duration::from_millis(BASE_BACKOFF_MS * (1 << (attempt - 1)));
                warn!(
                    attempt = attempt,
                    max_retries = MAX_RETRIES,
                    backoff_ms = backoff.as_millis() as u64,
                    "LLM transient error — retrying after backoff"
                );
                tokio::time::sleep(backoff).await;
            }

            let request_start = std::time::Instant::now();

            let mut req = self.http.post(&url).json(&body);
            if let Some(ref key) = self.config.api_key {
                req = req.bearer_auth(key);
            }

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    let latency = request_start.elapsed().as_millis() as u64;
                    let err = LlmError::Connection(e.to_string());

                    // Record connection error
                    if let Some(ref limiter) = self.rate_limiter {
                        limiter.record(&self.config.model, 0, 0, 0, latency).await;
                    }

                    if err.is_retryable() && attempt < MAX_RETRIES {
                        warn!(attempt = attempt, error = %err, "Connection error (retryable)");
                        last_err = Some(err);
                        continue;
                    }
                    return Err(err);
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let latency = request_start.elapsed().as_millis() as u64;
                let resp_body = resp.text().await.unwrap_or_default();

                // Record error response
                if let Some(ref limiter) = self.rate_limiter {
                    limiter.record(&self.config.model, 0, 0, status.as_u16(), latency).await;
                }

                let err = LlmError::Api {
                    status: status.as_u16(),
                    body: resp_body,
                };
                if err.is_retryable() && attempt < MAX_RETRIES {
                    warn!(attempt = attempt, status = status.as_u16(), "API error (retryable)");
                    last_err = Some(err);
                    continue;
                }
                return Err(err);
            }

            let latency = request_start.elapsed().as_millis() as u64;

            // Parse native Ollama response
            let raw_text = resp.text().await
                .map_err(|e| LlmError::Parse(e.to_string()))?;
            debug!(raw_len = raw_text.len(), "Raw Ollama response received");

            let native: NativeOllamaResponse = serde_json::from_str(&raw_text)
                .map_err(|e| LlmError::Parse(format!("JSON parse error: {e}\nRaw: {}", safe_truncate(&raw_text, 500))))?;

            // Record successful response with token counts
            let tokens_in = native.prompt_eval_count.unwrap_or(0);
            let tokens_out = native.eval_count.unwrap_or(0);
            if let Some(ref limiter) = self.rate_limiter {
                limiter.record(&self.config.model, tokens_in, tokens_out, 200, latency).await;
            }

            // Log tool calls from native response before conversion
            if let Some(ref msg) = native.message {
                if let Some(ref tcs) = msg.tool_calls {
                    debug!(tool_calls = tcs.len(), "Native response contains tool calls");
                    for tc in tcs {
                        debug!(name = %tc.function.name, "Native tool call");
                    }
                }
                if let Some(ref thinking) = msg.thinking {
                    debug!(thinking_len = thinking.len(), "Native response contains thinking");
                }
            }

            // Convert to internal ChatResponse format
            let chat_resp = self.convert_native_response(native);

            if attempt > 0 {
                info!(attempt = attempt, "LLM call succeeded after retry");
            }

            if let Some(choice) = chat_resp.choices.first() {
                let tool_calls = choice.message.tool_calls.as_ref().map(|t| t.len()).unwrap_or(0);
                debug!(
                    finish = ?choice.finish_reason,
                    tool_calls = tool_calls,
                    content_len = choice.message.content.as_ref().map(|c| c.len()).unwrap_or(0),
                    "Chat response received"
                );
            }

            return Ok(chat_resp);
        }

        // All retries exhausted — return the last error
        Err(last_err.unwrap_or_else(|| LlmError::Connection("All retries exhausted".into())))
    }

    /// Convert a native Ollama response into our internal ChatResponse format.
    fn convert_native_response(&self, native: NativeOllamaResponse) -> ChatResponse {
        let msg = native.message.unwrap_or(NativeMessage {
            role: Some("assistant".into()),
            content: None,
            thinking: None,
            tool_calls: None,
        });

        // Convert native tool calls to our format (use model's ID if present, else generate)
        let tool_calls = msg.tool_calls.map(|calls| {
            calls
                .into_iter()
                .enumerate()
                .map(|(i, tc)| ToolCallMessage {
                    id: tc.id.unwrap_or_else(|| format!("call_{}", i)),
                    call_type: "function".into(),
                    function: FunctionCall {
                        name: tc.function.name,
                        arguments: serde_json::to_string(&tc.function.arguments)
                            .unwrap_or_default(),
                    },
                })
                .collect()
        });

        let finish_reason = native.done_reason.or_else(|| {
            if native.done.unwrap_or(false) { Some("stop".into()) } else { None }
        });

        let usage = Some(Usage {
            prompt_tokens: native.prompt_eval_count,
            completion_tokens: native.eval_count,
            total_tokens: match (native.prompt_eval_count, native.eval_count) {
                (Some(p), Some(c)) => Some(p + c),
                _ => None,
            },
        });

        ChatResponse {
            id: None,
            choices: vec![Choice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".into(),
                    content: msg.content,
                    tool_calls,
                    tool_call_id: None,
                },
                finish_reason,
            }],
            usage,
        }
    }

    /// Convert Cortex ToolDefinitions into OpenAI-compatible tool schemas.
    pub fn tool_schemas(definitions: &[ToolDefinition]) -> Vec<ToolSchema> {
        definitions
            .iter()
            .map(|d| ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: d.name.clone(),
                    description: d.description.clone(),
                    parameters: d.parameters.clone(),
                },
            })
            .collect()
    }

    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolSchema]>,
    ) -> Result<ChatResponse, LlmError> {
        self.chat(messages, tools).await
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

// ── Chat API Types (OpenAI-compatible) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant_tool_calls(calls: Vec<ToolCallMessage>) -> Self {
        Self {
            role: "assistant".into(),
            content: None,
            tool_calls: Some(calls),
            tool_call_id: None,
        }
    }

    /// Create an assistant message with both content (including `<think>` tags)
    /// and tool calls. Preserving content across turns prevents 35-40% degradation
    /// in M2.7's interleaved thinking mode.
    pub fn assistant_with_tool_calls(content: Option<String>, calls: Vec<ToolCallMessage>) -> Self {
        Self {
            role: "assistant".into(),
            content,
            tool_calls: Some(calls),
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("API error (status {status}): {body}")]
    Api { status: u16, body: String },

    #[error("Parse error: {0}")]
    Parse(String),
}

impl LlmError {
    /// Whether this error is transient and worth retrying.
    ///
    /// Retryable: connection errors, HTTP 429 (rate limit), 500/502/503/504 (server errors).
    /// Non-retryable: HTTP 4xx (except 429), parse errors.
    pub fn is_retryable(&self) -> bool {
        match self {
            // Connection errors are often transient (DNS blip, timeout, reset)
            LlmError::Connection(_) => true,
            // Server-side transient errors
            LlmError::Api { status, .. } => matches!(status, 429 | 500 | 502 | 503 | 504),
            // Parse errors won't improve on retry
            LlmError::Parse(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_serialization() {
        let msg = ChatMessage::system("You are a Cortex mind.");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("You are a Cortex mind."));
    }

    #[test]
    fn tool_schema_conversion() {
        let defs = vec![ToolDefinition {
            name: "bash".into(),
            description: "Execute a shell command".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }),
            mutates: true,
        }];

        let schemas = OllamaClient::tool_schemas(&defs);
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].function.name, "bash");
        assert_eq!(schemas[0].tool_type, "function");
    }

    #[test]
    fn tool_call_message_roundtrip() {
        let tc = ToolCallMessage {
            id: "call_1".into(),
            call_type: "function".into(),
            function: FunctionCall {
                name: "bash".into(),
                arguments: r#"{"command":"ls"}"#.into(),
            },
        };

        let json = serde_json::to_string(&tc).unwrap();
        let back: ToolCallMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.function.name, "bash");
    }

    #[test]
    fn default_config() {
        let cfg = OllamaConfig::default();
        assert!(cfg.base_url.contains("11434"));
        assert_eq!(cfg.model, "qwen2.5:7b");
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn cloud_config() {
        let cfg = OllamaConfig::cloud("gemma4:cloud", "test-key-123");
        assert!(cfg.base_url.contains("api.ollama.com"));
        assert_eq!(cfg.model, "gemma4:cloud");
        assert_eq!(cfg.api_key.as_deref(), Some("test-key-123"));
    }

    #[test]
    fn model_router_default() {
        let router = ModelRouter::default();
        assert_eq!(router.primary_model, "qwen2.5:7b");
        assert_eq!(router.lightweight_model, "phi3:mini");
        assert!(router.api_key.is_none());
    }

    #[test]
    fn model_router_cloud() {
        let router = ModelRouter::cloud("cloud-key-abc");
        assert_eq!(router.primary_model, "devstral-small-2:24b");
        assert_eq!(router.team_lead_model, "devstral-small-2:24b");
        assert_eq!(router.agent_model, "devstral-small-2:24b");
        assert_eq!(router.lightweight_model, "minimax-m2.7");
        assert!(router.base_url.contains("api.ollama.com"));
        assert_eq!(router.api_key.as_deref(), Some("cloud-key-abc"));
    }

    #[test]
    fn model_router_config_for_role() {
        use codex_roles::Role;
        let router = ModelRouter::cloud("key");

        let primary_cfg = router.config_for_role(Role::Primary);
        assert_eq!(primary_cfg.model, "devstral-small-2:24b");
        assert!(primary_cfg.api_key.is_some());
        assert!(primary_cfg.base_url.contains("api.ollama.com"));

        let agent_cfg = router.config_for_role(Role::Agent);
        assert_eq!(agent_cfg.model, "devstral-small-2:24b");

        let light_cfg = router.config_lightweight();
        assert_eq!(light_cfg.model, "minimax-m2.7");
        assert_eq!(light_cfg.temperature, 1.0);
    }
}
