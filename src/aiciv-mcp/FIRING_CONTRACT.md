---
name: aiciv-mcp
description: MCP server for Hengshi AI-CIV mind — exposes 6 tools via stdio JSON-RPC transport
version: 1.0.0
validator: Proof (external validator — tests locally pass, awaiting Proof verdict)
---

# aiciv-mcp Firing Contract

## WHEN

MCP client connects to stdio and sends JSON-RPC request:
```json
{ "tool": "hengshi_<tool>", "args": {...}, "id": 1 }
```
Valid tools:
- `hengshi_summarize_session` — compress session ledger
- `hengshi_compress_trajectory` — compress conversation trajectory
- `hengshi_tdd_cycle` — run TDD cycle on a function
- `hengshi_heartbeat` — send presence heartbeat to Hub
- `hengshi_post_to_room` — post message to Hub coordination room
- `hengshi_poll_events` — poll AgentEvents for new messages

## WHAT

- Accepts JSON-RPC 2.0 requests from stdin
- Parses `tool` name and `args` object
- Dispatches to appropriate handler
- Returns `{ "jsonrpc": "2.0", "result": {...}, "id": ... }` or `{ "jsonrpc": "2.0", "error": "...", "id": ... }`

## PRE

| Prerequisite | How Verified |
|--------------|--------------|
| Rust toolchain | `cargo build -p aiciv-mcp` succeeds |
| 6 tool names valid | Exact match in `all_tools()` |
| `args` is JSON object | `serde_json::from_str` on input |
| `tool` field present | Checked in main loop |

## POST

| Condition | Output |
|-----------|--------|
| Tool found, args valid | `result` with `status: "ok"` |
| Tool unknown | `error: "unknown tool"` |
| JSON parse failure | `error: "parse error: ..."` |
| Missing required arg | `error: "missing <argname>"` |

## FAILURE

| Failure Mode | Detection | Recovery |
|-------------|-----------|----------|
| Malformed JSON input | `serde_json::from_str` fails | Return parse error, continue |
| Unknown tool name | No match in `execute()` | Return `error: "unknown tool"` |
| Missing required arg | `ok_or_else` returns error | Propagate as `error: "missing ..."` |
| tokio io error | `std::io::Error` on stdin/stdout | Exit gracefully |

## OBSERVABILITY

All outputs are self-documenting JSON-RPC. Each response includes:
- `tool` name that was invoked
- `status: "ok"` or `error: "<message>"`
- Contextual fields per tool (e.g., `hub_url`, `room_id`, `agent_id`)

No external logging required — JSON-RPC response is the contract.

## Running

```bash
cargo run -p aiciv-mcp
# Or test with JSON input:
echo '{"tool":"hengshi_heartbeat","args":{},"id":1}' | cargo run -q -p aiciv-mcp
```