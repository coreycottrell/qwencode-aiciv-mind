//! Ollama client — with retry, exponential backoff, and fallback mode.
//! Principle 11 — Distributed Intelligence.

use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: Option<OllamaMessage>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
    min_delay_secs: u64,
    max_retries: u32,
}

impl OllamaClient {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let api_key = std::env::var("OLLAMA_API_KEY").ok();
        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| {
                if api_key.is_some() {
                    "devstral-small-2:24b".to_string()
                } else {
                    "qwen2.5:7b".to_string()
                }
            });
        let min_delay_secs: u64 = std::env::var("OLLAMA_MIN_DELAY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Self::new(&base_url, api_key.as_deref(), &model, min_delay_secs)
    }

    pub fn new(base_url: &str, api_key: Option<&str>, model: &str, min_delay_secs: u64) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(180))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.map(|s| s.to_string()),
            model: model.to_string(),
            min_delay_secs,
            max_retries: 3,
        }
    }

    /// Chat with the LLM. Retries with exponential backoff on 500 errors.
    pub async fn chat(
        &self,
        system: &str,
        user: &str,
    ) -> Result<LlmResponse, LlmError> {
        let url = if self.api_key.is_some() {
            "https://api.ollama.com/api/chat".to_string()
        } else {
            format!("{}/api/chat", self.base_url)
        };

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.min_delay_secs * 2u64.pow(attempt as u32);
                tracing::warn!(attempt, delay, "Retrying Ollama API call");
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }

            let mut req = self.client.post(&url).json(&serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user}
                ],
                "stream": false,
                "options": {"num_predict": 4096}
            }));

            if let Some(ref key) = self.api_key {
                req = req.header("Authorization", format!("Bearer {key}"));
            }

            match req.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<OllamaResponse>().await {
                            Ok(body) => {
                                let content = body
                                    .message
                                    .map(|m| m.content)
                                    .unwrap_or_else(|| "Empty response.".to_string());
                                return Ok(LlmResponse {
                                    content,
                                    retries: attempt,
                                });
                            }
                            Err(e) => {
                                last_error = Some(format!("JSON parse error: {e}"));
                                continue;
                            }
                        }
                    } else if resp.status().is_server_error() {
                        // 500 — retry with backoff
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        last_error = Some(format!("Server error {status}: {body}"));
                        continue;
                    } else {
                        // 4xx — don't retry
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        return Err(LlmError::ApiError(format!("{status}: {body}")));
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Connection error: {e}"));
                    continue;
                }
            }
        }

        Err(LlmError::ExhaustedRetries(
            self.max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub retries: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Exhausted {0} retries: {1}")]
    ExhaustedRetries(u32, String),
}
