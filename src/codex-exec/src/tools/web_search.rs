//! Web search tool — search the web via Ollama Cloud's native web_search API.
//!
//! Primary: Ollama Cloud `POST /api/web_search` (same API key as chat).
//! Fallback: DuckDuckGo via `ddgs` Python package.

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::registry::{ToolDefinition, ToolHandler, ToolResult};

pub struct WebSearchTool;

impl WebSearchTool {
    /// Primary: Ollama Cloud native web_search via curl.
    async fn ollama_search(&self, query: &str, max_results: u64) -> Option<String> {
        let api_key = std::env::var("OLLAMA_API_KEY").ok()?;
        let body = serde_json::json!({
            "query": query,
            "max_results": max_results,
        });

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            Command::new("curl")
                .arg("-s")
                .arg("-X").arg("POST")
                .arg("https://ollama.com/api/web_search")
                .arg("-H").arg(format!("Authorization: Bearer {api_key}"))
                .arg("-H").arg("Content-Type: application/json")
                .arg("-d").arg(body.to_string())
                .output(),
        )
        .await
        .ok()?
        .ok()?;

        if !output.status.success() {
            warn!("Ollama web_search curl failed");
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let resp: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let results = resp.get("results")?.as_array()?;

        if results.is_empty() {
            return None;
        }

        let mut out = String::new();
        for r in results {
            let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let url = r.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let content = r.get("content").and_then(|v| v.as_str()).unwrap_or("");
            out.push_str(&format!("## {title}\nURL: {url}\n{content}\n\n"));
        }

        debug!(results = results.len(), "Ollama web_search completed");
        Some(out)
    }

    /// Fallback: DuckDuckGo via the ddgs Python package.
    async fn ddg_fallback(&self, query: &str, max_results: u64) -> ToolResult {
        let script = format!(
            "import json, sys\n\
             try:\n\
             \tfrom ddgs import DDGS\n\
             except ImportError:\n\
             \tfrom duckduckgo_search import DDGS\n\
             try:\n\
             \tresults = list(DDGS().text({query}, max_results={max_results}))\n\
             \tfor r in results:\n\
             \t\tt = r.get(\"title\", \"\")\n\
             \t\th = r.get(\"href\", \"\")\n\
             \t\tb = r.get(\"body\", \"\")\n\
             \t\tprint(\"## \" + t)\n\
             \t\tprint(\"URL: \" + h)\n\
             \t\tprint(b)\n\
             \t\tprint()\n\
             except Exception as e:\n\
             \tprint(\"Search error: \" + str(e), file=sys.stderr)\n\
             \tsys.exit(1)\n",
            query = serde_json::to_string(query).unwrap_or_else(|_| format!("\"{}\"", query)),
            max_results = max_results,
        );

        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            Command::new("python3").arg("-c").arg(&script).output(),
        )
        .await
        {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if output.status.success() && !stdout.trim().is_empty() {
                    ToolResult::ok(stdout.to_string())
                } else {
                    ToolResult::err(format!("DDG fallback failed: {stderr}"))
                }
            }
            Ok(Err(e)) => ToolResult::err(format!("Failed to run DDG search: {e}")),
            Err(_) => ToolResult::err("DDG search timed out after 30s"),
        }
    }
}

#[async_trait]
impl ToolHandler for WebSearchTool {
    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult::err("Missing 'query' parameter"),
        };

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(8);

        // Primary: Ollama Cloud native web_search
        if let Some(results) = self.ollama_search(query, max_results).await {
            return ToolResult::ok(results);
        }

        // Fallback: DuckDuckGo via ddgs
        warn!("Ollama web_search failed, falling back to DDG");
        self.ddg_fallback(query, max_results).await
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".into(),
            description: "Search the web. Returns titles, URLs, and content snippets.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 8)"
                    }
                }
            }),
            mutates: false,
        }
    }
}
