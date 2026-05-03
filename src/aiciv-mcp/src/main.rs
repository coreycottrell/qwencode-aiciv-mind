//! aiciv-mcp binary — main entry point
//!
//! Run as: cargo run -p aiciv-mcp
//!
//! Simple stdio-based MCP server. Reads JSON-RPC-like requests from stdin,
//! writes responses to stdout.

use aiciv_mcp::{HengshiMcpServer, tools::ToolCall};
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<()> {
    let server = Arc::new(
        HengshiMcpServer::new("hengshi")
            .with_hub_url("http://87.99.131.49:8900")
            .with_agent_id("20692dcb-db76-5415-b59f-54e854a3801f"),
    );

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();

    // Simple JSON-RPC-like protocol:
    // Input:  { "tool": "name", "args": {...}, "id": ... }
    // Output: { "result": {...}, "id": ... } or { "error": "...", "id": ... }

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let input: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": format!("parse error: {}", e),
                    "id": Value::Null
                });
                stdout.write_all(resp.to_string().as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                continue;
            }
        };

        let tool_name = input.get("tool").and_then(|v| v.as_str()).unwrap_or("");
        let args_map = input.get("args").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let id = input.get("id").cloned().unwrap_or(Value::Null);

        let call = ToolCall {
            name: tool_name.to_string(),
            arguments: args_map.into_iter().collect(),
        };

        let result = server.execute_tool(call).await;

        let resp = match result {
            Ok(r) => {
                if let Some(err) = r.error {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": err,
                        "id": id
                    })
                } else {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": r.result,
                        "id": id
                    })
                }
            }
            Err(e) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": e.to_string(),
                "id": id
            }),
        };

        stdout.write_all(resp.to_string().as_bytes()).await?;
        stdout.write_all(b"\n").await?;
    }

    Ok(())
}