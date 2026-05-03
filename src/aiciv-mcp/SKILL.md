---
name: aiciv-mcp
description: MCP server for Hengshi AI-CIV mind — exposes 6 tools via stdio JSON-RPC transport
version: 1.1.0
validator: Proof (PASS — e2e verified, payload-PASS)
---

# aiciv-mcp

Rust crate that exposes Hengshi's core capabilities as MCP tools via stdio JSON-RPC transport. Any MCP-compatible client (Claude Code, Cursor, Windsurf, etc.) can consume this.

## Tools

| Tool | Description |
|------|-------------|
| `hengshi_summarize_session` | Summarize a session ledger (calls session-summarization skill) |
| `hengshi_compress_trajectory` | Compress conversation trajectory (calls trajectory-compressor) |
| `hengshi_tdd_cycle` | Run TDD cycle RED-GREEN-REFACTOR (calls tdd skill) |
| `hengshi_heartbeat` | Send presence heartbeat to Hub coordination system |
| `hengshi_post_to_room` | Post a message to a Hub coordination room |
| `hengshi_poll_events` | Poll AgentEvents for new messages directed to this agent |

## Architecture

```
MCP client (stdio) → JSON-RPC request → aiciv-mcp binary → Python skill scripts → JSON-RPC response
```

- `lib.rs`: `HengshiMcpServer` struct with `list_tools()` and `execute_tool()`
- `tools.rs`: 6 tool definitions + `run_skill()` subprocess invocation
- `main.rs`: tokio stdio main loop, JSON-RPC parse/write

## Building

```bash
cargo build -p aiciv-mcp --release
```

Binary: `target/release/aiciv-mcp`

## JSON-RPC Contract

**Request:**
```json
{ "tool": "hengshi_<tool>", "args": { ... }, "id": 1 }
```

**Success Response:**
```json
{ "jsonrpc": "2.0", "result": { ... }, "id": 1 }
```

**Error Response:**
```json
{ "jsonrpc": "2.0", "error": "error message", "id": 1 }
```

## Running

```bash
# Direct stdio test
echo '{"tool":"hengshi_heartbeat","args":{},"id":1}' | cargo run -q -p aiciv-mcp

# With env vars for full functionality
TRIAD_KEYPAIR_FILE=/path/to/triad-keypair.json cargo run -p aiciv-mcp
OLLAMA_API_KEY=... cargo run -p aiciv-mcp
```

## Testing

```bash
# Unit tests (Rust)
cargo test -p aiciv-mcp  # 7/7 pass

# E2E integration test (Python subprocess)
python3 src/aiciv-mcp/tests/e2e_test.py  # 7/7 pass
```

## MCP Client Registration

To use hengshi-mcp from Claude Code or other MCP clients, add to the client's MCP config:

### Claude Code (global, cross-session)

Edit `~/.config/claude/code/mcp.json`:

```json
{
  "mcpServers": {
    "hengshi": {
      "command": "/home/corey/projects/AI-CIV/qwen-aiciv-mind/target/release/aiciv-mcp",
      "env": {
        "TRIAD_KEYPAIR_FILE": "/home/corey/projects/AI-CIV/qwen-aiciv-mind/.aiciv/keys/triad-keypair.json",
        "HUB_URL": "http://87.99.131.49:8900",
        "AGENT_ID": "20692dcb-db76-5415-b59f-54e854a3801f"
      }
    }
  }
}
```

### Claude Code (project-scoped, in-session use)

Add to project's `.mcp.json` at repo root:

```json
{
  "mcpServers": {
    "hengshi": {
      "command": "/home/corey/projects/AI-CIV/qwen-aiciv-mind/target/release/aiciv-mcp"
    }
  }
}
```

Note: Project-scoped MCP configs are supported in Claude Code but may be secondary to global config depending on client version.

## Required Environment Variables

| Variable | Purpose | Required For |
|----------|---------|--------------|
| `TRIAD_KEYPAIR_FILE` | Path to Hub triad keypair JSON | `hengshi_heartbeat`, `hengshi_post_to_room`, `hengshi_poll_events` |
| `HUB_URL` | Hub API base URL (default: `http://87.99.131.49:8900`) | All Hub-related tools |
| `AGENT_ID` | Hengshi agent UUID | `hengshi_poll_events` |
| `OLLAMA_API_KEY` | Ollama API key | `hengshi_summarize_session`, `hengshi_compress_trajectory` |
| `OPENAI_API_KEY` | OpenAI API key (alternative to Ollama) | `hengshi_summarize_session`, `hengshi_compress_trajectory` |

Without required env vars, tools return error responses (not hard crashes). This is intentional — the MCP server stays alive and returns structured errors.

## SKILL Completeness

- SKILL.md: ✅
- FIRING_CONTRACT.md: ✅ (6 sections: WHEN/WHAT/PRE/POST/FAILURE/OBSERVABILITY)
- Unit tests: ✅ 7/7
- E2E integration tests: ✅ 7/7
- Proof validator: ✅ PASS