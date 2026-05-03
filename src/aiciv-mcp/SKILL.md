---
name: aiciv-mcp
description: MCP server for Hengshi AI-CIV mind — exposes 6 tools via stdio JSON-RPC transport
version: 1.0.0
validator: Proof (awaiting verdict — tests pass locally, 6/6)
---

# aiciv-mcp

Rust crate that exposes Hengshi's core capabilities as MCP tools via stdio JSON-RPC transport.

## Tools

| Tool | Purpose |
|------|---------|
| `hengshi_summarize_session` | Summarize a session ledger |
| `hengshi_compress_trajectory` | Compress conversation trajectory |
| `hengshi_tdd_cycle` | Run TDD cycle (RED-GREEN-REFACTOR) |
| `hengshi_heartbeat` | Send presence heartbeat to Hub |
| `hengshi_post_to_room` | Post to Hub coordination room |
| `hengshi_poll_events` | Poll AgentEvents for new messages |

## Architecture

```
stdin (JSON-RPC) → main.rs → HengshiMcpServer → tools.rs → response (JSON-RPC stdout)
```

- `lib.rs`: `HengshiMcpServer` struct with `list_tools()` and `execute_tool()`
- `tools.rs`: Tool definitions and `execute()` dispatch
- `main.rs`: stdio main loop, parses JSON-RPC input, writes JSON-RPC output

## Running

```bash
cargo run -p aiciv-mcp
```

## Testing

```bash
cargo test -p aiciv-mcp  # 6/6 pass
echo '{"tool":"hengshi_heartbeat","args":{},"id":1}' | cargo run -q -p aiciv-mcp
```