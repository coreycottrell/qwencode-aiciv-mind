//! SearchInterceptor — exposes web search and fetch as tools in the ThinkLoop.
//!
//! Implements `ToolInterceptor` from codex-llm, giving any mind the ability
//! to search the web and fetch URL content — all as native tool calls that
//! the LLM invokes during reasoning.
//!
//! Primary: Ollama Cloud `web_search` + `web_fetch` API (same key as chat).
//! Fallback: DuckDuckGo (search) / Jina Reader (fetch).
//!
//! This is what turns Cortex from "a brain without the internet" into a researcher.

use async_trait::async_trait;
use codex_exec::ToolResult;
use codex_llm::think_loop::ToolInterceptor;
use codex_llm::ollama::{ToolSchema, FunctionSchema};
use tokio::process::Command;
use tracing::{debug, warn};

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

/// Tool interceptor that exposes web search and fetch as LLM tools.
///
/// Tools exposed:
/// - `web_search` — search the web via Ollama Cloud (DDG fallback)
/// - `web_fetch` — fetch a URL's content via Ollama Cloud (Jina fallback)
pub struct SearchInterceptor {
    /// Ollama API key (read from env at construction time).
    api_key: Option<String>,
}

impl SearchInterceptor {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("OLLAMA_API_KEY").ok(),
        }
    }

    /// Primary: Ollama Cloud native web_search via curl.
    async fn ollama_search(&self, query: &str, max_results: u64) -> Option<String> {
        let api_key = self.api_key.as_ref()?;
        let body = serde_json::json!({
            "query": query,
            "max_results": max_results,
        });

        // SECURITY: API key is passed via env var, NOT as a curl argument —
        // prevents exposure via `ps aux` process argument lists.
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            Command::new("sh")
                .arg("-c")
                .arg("curl -s -X POST \"$OLLAMA_SEARCH_URL\" -H \"Authorization: Bearer $OLLAMA_API_KEY\" -H 'Content-Type: application/json' -d \"$OLLAMA_SEARCH_BODY\"")
                .env("OLLAMA_API_KEY", api_key)
                .env("OLLAMA_SEARCH_URL", "https://ollama.com/api/web_search")
                .env("OLLAMA_SEARCH_BODY", body.to_string())
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

    /// Primary: Ollama Cloud native web_fetch via curl.
    async fn ollama_fetch(&self, url: &str) -> Option<String> {
        let api_key = self.api_key.as_ref()?;
        let body = serde_json::json!({ "url": url });

        // SECURITY: API key is passed via env var, NOT as a curl argument —
        // prevents exposure via `ps aux` process argument lists.
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(25),
            Command::new("sh")
                .arg("-c")
                .arg("curl -s -X POST \"$OLLAMA_FETCH_URL\" -H \"Authorization: Bearer $OLLAMA_API_KEY\" -H 'Content-Type: application/json' -d \"$OLLAMA_FETCH_BODY\"")
                .env("OLLAMA_API_KEY", api_key)
                .env("OLLAMA_FETCH_URL", "https://ollama.com/api/web_fetch")
                .env("OLLAMA_FETCH_BODY", body.to_string())
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
impl ToolInterceptor for SearchInterceptor {
    fn schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "web_search".into(),
                    description: "Search the web for information. Returns titles, URLs, and \
                        content snippets. Use when you need current information about people, \
                        companies, technologies, or events.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "Maximum number of results to return (default: 8)"
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
            ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: "web_fetch".into(),
                    description: "Fetch and read a specific URL. Returns clean text content of \
                        the page. Use when you have a URL and need to read its contents.".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to fetch (must start with http:// or https://)"
                            }
                        },
                        "required": ["url"]
                    }),
                },
            },
        ]
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        match name {
            "web_search" => {
                let query = match args.get("query").and_then(|v| v.as_str()) {
                    Some(q) if !q.is_empty() => q,
                    _ => return Some(ToolResult::err("Missing required argument: query")),
                };

                let max_results = args
                    .get("max_results")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(8);

                // Primary: Ollama Cloud
                if let Some(results) = self.ollama_search(query, max_results).await {
                    return Some(ToolResult::ok(results));
                }

                // Fallback: DuckDuckGo
                warn!("Ollama web_search failed, falling back to DDG");
                Some(self.ddg_fallback(query, max_results).await)
            }

            "web_fetch" => {
                let url = match args.get("url").and_then(|v| v.as_str()) {
                    Some(u) if !u.is_empty() => u,
                    _ => return Some(ToolResult::err("Missing required argument: url")),
                };

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Some(ToolResult::err(
                        "URL must start with http:// or https://",
                    ));
                }

                // Primary: Ollama Cloud
                if let Some(content) = self.ollama_fetch(url).await {
                    return Some(ToolResult::ok(content));
                }

                // Fallback: Jina Reader
                warn!("Ollama web_fetch failed, falling back to Jina Reader");
                Some(self.jina_fallback(url).await)
            }

            // Not a search tool — pass through to next handler.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_llm::think_loop::ToolInterceptor;

    #[test]
    fn search_interceptor_schemas() {
        let interceptor = SearchInterceptor::new();
        let schemas = interceptor.schemas();

        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> = schemas.iter().map(|s| s.function.name.as_str()).collect();
        assert!(names.contains(&"web_search"));
        assert!(names.contains(&"web_fetch"));

        // Verify all schemas have type "function"
        for schema in &schemas {
            assert_eq!(schema.tool_type, "function");
        }
    }

    #[tokio::test]
    async fn search_interceptor_ignores_unknown() {
        let interceptor = SearchInterceptor::new();

        let result = interceptor.handle("bash", &serde_json::json!({"command": "ls"})).await;
        assert!(result.is_none());

        let result = interceptor.handle("hub_feed", &serde_json::json!({})).await;
        assert!(result.is_none());

        let result = interceptor.handle("unknown_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn search_interceptor_validates_required_args() {
        let interceptor = SearchInterceptor::new();

        // web_search with empty args
        let result = interceptor.handle("web_search", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("query"));

        // web_fetch with empty args
        let result = interceptor.handle("web_fetch", &serde_json::json!({})).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("url"));
    }

    #[tokio::test]
    async fn search_interceptor_rejects_invalid_urls() {
        let interceptor = SearchInterceptor::new();

        // web_fetch with non-http URL
        let result = interceptor.handle("web_fetch", &serde_json::json!({
            "url": "ftp://example.com/file"
        })).await;
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.error.unwrap().contains("http://"));
    }

    #[test]
    fn search_interceptor_schema_parameters_are_valid_json_schema() {
        let interceptor = SearchInterceptor::new();
        let schemas = interceptor.schemas();

        for schema in &schemas {
            let params = &schema.function.parameters;
            assert_eq!(
                params.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Schema '{}' missing type: object",
                schema.function.name,
            );
            assert!(
                params.get("properties").is_some(),
                "Schema '{}' missing properties",
                schema.function.name,
            );
        }
    }
}
