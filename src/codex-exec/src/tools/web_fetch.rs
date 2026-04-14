//! Web fetch tool — retrieve a URL's content via Ollama Cloud's native web_fetch API.
//!
//! Primary: Ollama Cloud `POST /api/web_fetch` (same API key as chat).
//! Fallback: Jina Reader (`r.jina.ai/{url}`).

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

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

pub struct WebFetchTool;

impl WebFetchTool {
    /// Primary: Ollama Cloud native web_fetch via curl.
    async fn ollama_fetch(&self, url: &str) -> Option<String> {
        let api_key = std::env::var("OLLAMA_API_KEY").ok()?;
        let body = serde_json::json!({ "url": url });

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(25),
            Command::new("curl")
                .arg("-s")
                .arg("-X").arg("POST")
                .arg("https://ollama.com/api/web_fetch")
                .arg("-H").arg(format!("Authorization: Bearer {api_key}"))
                .arg("-H").arg("Content-Type: application/json")
                .arg("-d").arg(body.to_string())
                .output(),
        )
        .await
        .ok()?
        .ok()?;

        if !output.status.success() {
            warn!("Ollama web_fetch curl failed");
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let resp: serde_json::Value = serde_json::from_str(&raw).ok()?;

        let title = resp.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let content = resp.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if content.is_empty() {
            return None;
        }

        let truncated = if content.len() > 8000 {
            format!("# {title}\n\n{}...\n\n[Content truncated at 8000 chars]", safe_truncate(content, 8000))
        } else {
            format!("# {title}\n\n{content}")
        };

        debug!(url = url, content_len = content.len(), "Ollama web_fetch completed");
        Some(truncated)
    }

    /// Fallback: Jina Reader (r.jina.ai/{url}).
    async fn jina_fallback(&self, url: &str) -> ToolResult {
        let jina_url = format!("https://r.jina.ai/{url}");

        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            Command::new("curl")
                .arg("-sL")
                .arg("--max-time").arg("25")
                .arg("-H").arg("Accept: text/markdown")
                .arg(&jina_url)
                .output(),
        )
        .await
        {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if output.status.success() && !stdout.trim().is_empty() {
                    let content = if stdout.len() > 8000 {
                        format!("{}...\n\n[Content truncated at 8000 chars]", safe_truncate(&stdout, 8000))
                    } else {
                        stdout.to_string()
                    };
                    ToolResult::ok(content)
                } else {
                    ToolResult::err("Jina fallback returned empty response")
                }
            }
            Ok(Err(e)) => ToolResult::err(format!("Failed to run Jina fetch: {e}")),
            Err(_) => ToolResult::err("Jina fetch timed out after 30s"),
        }
    }
}

#[async_trait]
impl ToolHandler for WebFetchTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return ToolResult::err("Missing 'url' parameter"),
        };

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return ToolResult::err("URL must start with http:// or https://");
        }

        // Primary: Ollama Cloud native web_fetch
        if let Some(content) = self.ollama_fetch(url).await {
            return ToolResult::ok(content);
        }

        // Fallback: Jina Reader
        warn!("Ollama web_fetch failed, falling back to Jina Reader");
        self.jina_fallback(url).await
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".into(),
            description: "Fetch a URL and return its content as clean text.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["url"],
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch (must start with http:// or https://)"
                    }
                }
            }),
            mutates: false,
        }
    }
}
