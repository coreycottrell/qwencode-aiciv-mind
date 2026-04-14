# codex-ipc Porting Guide — For Root and Thalweg

**Date**: 2026-04-04
**Source**: `aiciv-mind-cubed/src/codex-ipc/` (1,504 lines Rust)
**Purpose**: Reference implementation for MCP JSON-RPC 2.0 inter-mind communication. Portable to Root (Python/ZMQ→MCP) and Thalweg (Rust/gRPC→MCP).

---

## What codex-ipc Provides

A complete MCP server/client stack for inter-mind communication:

| Component | File | Lines | What It Does |
|-----------|------|-------|-------------|
| **Protocol** | `protocol.rs` | ~200 | JSON-RPC 2.0 message types, custom `cortex/delegate` + `cortex/status` methods |
| **Transport** | `transport.rs` | ~350 | `MindTransport` trait + 3 implementations: Channel (test), Stdio (client), StdioServer (server) |
| **Server** | `server.rs` | ~450 | `McpMindServer` — handles MCP handshake, routes tool calls, delegates tasks |
| **Client** | `client.rs` | ~350 | `McpMindClient` — connects to child processes, sends delegations, collects results |
| **Lib** | `lib.rs` | ~150 | Public API, `DelegateHandler` trait, `DelegateTaskResult` struct |

## The Key Abstractions

### 1. MindTransport (trait)
```rust
#[async_trait]
pub trait MindTransport: Send + Sync {
    async fn send(&mut self, msg: &JsonRpcMessage) -> Result<()>;
    async fn recv(&mut self) -> Result<JsonRpcMessage>;
}
```

**For Root (Python)**: Implement over `sys.stdin`/`sys.stdout` (StdioTransport) or `asyncio.StreamReader`/`StreamWriter`.
**For Thalweg (Rust)**: Copy directly — it's already Rust.

### 2. DelegateHandler (trait)
```rust
#[async_trait]
pub trait DelegateHandler: Send + Sync {
    async fn handle_delegate(
        &self,
        task_id: &str,
        task: &str,
        context: Option<&str>,
        parent_mind_id: &str,
    ) -> DelegateTaskResult;
}
```

This is where ThinkLoop connects. When a parent mind sends `cortex/delegate`, the server calls this handler. The handler runs the ThinkLoop and returns the result.

**For Root**: Implement in Python — the handler runs Root's thinking loop.
**For Thalweg**: Copy directly.

### 3. DelegateTaskResult
```rust
pub struct DelegateTaskResult {
    pub accepted: bool,
    pub response: Option<String>,
    pub iterations: Option<u32>,
    pub tool_calls_made: Option<u32>,
    pub completed: Option<bool>,
    pub error: Option<String>,
}
```

The standard return type for all delegations.

## MCP Protocol Details

### Handshake
1. Client sends `initialize` with `protocolVersion: "2024-11-05"`, capabilities
2. Server responds with `serverInfo` (name, version), supported capabilities
3. Client sends `notifications/initialized`
4. Communication begins

### Custom Methods
| Method | Direction | Purpose |
|--------|-----------|---------|
| `cortex/delegate` | Client → Server | Delegate a task to a child mind |
| `cortex/status` | Client → Server | Query child mind status |
| `tools/list` | Client → Server | List available tools (standard MCP) |
| `tools/call` | Client → Server | Execute a tool (standard MCP) |

### Delegation Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "cortex/delegate",
  "params": {
    "task_id": "research-hub-protocol",
    "task": "Research the AiCIV Hub protocol and write a summary",
    "context": "Optional context for the child mind",
    "parent_mind_id": "primary"
  }
}
```

### Delegation Response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "accepted": true,
    "response": "The Hub protocol uses REST endpoints...",
    "iterations": 4,
    "tool_calls_made": 3,
    "completed": true,
    "error": null
  }
}
```

## Porting to Root (Python, ZMQ → MCP)

### Current Root Architecture
- ZMQ ROUTER/DEALER pattern (3,121 lines across 12 files)
- 12 message types: spawn_request, delegate_task, task_result, heartbeat, etc.
- Multi-process: each submind is a separate Python process
- stdout polluted with LiteLLM logging (this caused narrator mode)

### What Changes
1. **Replace ZMQ sockets with stdio pipes** — Process spawning stays the same (subprocess.Popen), but communication switches from ZMQ to reading/writing JSON-RPC on stdin/stdout
2. **Replace 12 message types with MCP methods** — `cortex/delegate` replaces `delegate_task`, `tools/call` replaces tool execution
3. **Add MCP handshake** — 3-message init sequence before communication
4. **stderr for logging** — All logging must go to stderr, stdout is the MCP transport

### Estimated Effort
- **With Cortex reference**: 12-16 hours
  - Protocol layer: 3-4h (translate protocol.rs to Python dataclasses)
  - Transport: 2-3h (async stdin/stdout reader/writer)
  - Server: 3-4h (MCP message routing, delegate handler)
  - Client: 2-3h (subprocess management + MCP client)
  - Integration: 2-3h (wire into existing Root codebase)

### Python Implementation Sketch
```python
import asyncio
import json
import sys

class McpTransport:
    """Read/write JSON-RPC messages on stdin/stdout."""
    async def send(self, msg: dict):
        line = json.dumps(msg) + "\n"
        sys.stdout.write(line)
        sys.stdout.flush()

    async def recv(self) -> dict:
        line = await asyncio.get_event_loop().run_in_executor(
            None, sys.stdin.readline
        )
        return json.loads(line)

class McpServer:
    """MCP server — handles delegations from parent mind."""
    def __init__(self, mind_id: str, delegate_handler):
        self.mind_id = mind_id
        self.handler = delegate_handler

    async def run(self, transport: McpTransport):
        # Handle MCP handshake
        init_msg = await transport.recv()
        await transport.send({
            "jsonrpc": "2.0",
            "id": init_msg["id"],
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {"name": f"root-{self.mind_id}", "version": "0.1.0"},
            }
        })
        # Wait for initialized notification
        await transport.recv()

        # Main loop
        while True:
            msg = await transport.recv()
            if msg.get("method") == "cortex/delegate":
                result = await self.handler.handle(msg["params"])
                await transport.send({"jsonrpc": "2.0", "id": msg["id"], "result": result})
            elif msg.get("method") == "shutdown":
                break
```

## Porting to Thalweg (Rust, gRPC → MCP)

### Current Thalweg Architecture
- gRPC with 5 RPCs: Delegate, Status, Heartbeat, ListTools, CallTool
- UDS (Unix Domain Sockets) for local communication
- 2,048 lines across 6 files
- Already Rust — closest to Cortex

### What Changes
1. **Replace gRPC with stdio JSON-RPC** — Remove protobuf definitions, use codex-ipc's protocol.rs directly
2. **Replace UDS with stdio pipes** — Process spawning switches from UDS to subprocess stdin/stdout
3. **Remove protobuf dependency** — JSON-RPC is simpler, no code generation step

### Estimated Effort
- **With Cortex reference**: 8-12 hours
  - Protocol: 1-2h (copy `protocol.rs`, adapt types)
  - Transport: 1-2h (copy `transport.rs`, adapt to Thalweg's async runtime)
  - Server: 2-3h (replace gRPC service impl with McpMindServer)
  - Client: 2-3h (replace gRPC client with McpMindClient)
  - Integration: 2-3h (wire into existing Thalweg codebase)

### Thalweg can literally copy these files:
- `protocol.rs` → direct copy (zero changes needed)
- `transport.rs` → direct copy (uses tokio, same as Thalweg)
- `server.rs` → adapt DelegateHandler to Thalweg's thinking loop
- `client.rs` → adapt spawn mechanism to Thalweg's process model

## Testing Strategy

### Unit Tests (copy from codex-ipc)
- `protocol_serialization` — JSON-RPC message round-trips
- `channel_transport` — in-memory transport for testing
- `delegate_handler_mock` — mock handler returns expected results

### Integration Tests
- Spawn a child process, complete MCP handshake, delegate a task, verify result
- The evolution_proof binary is a good template for this

## Key Learnings from Cortex Implementation

1. **stdout is sacred** — All logging MUST go to stderr. One stray `println!` on stdout corrupts the MCP transport.
2. **Tool call arguments must be JSON objects** — Ollama returns `arguments` as a JSON object, not a string. The MCP layer must preserve this.
3. **Generate call IDs if the model doesn't** — Some models (Gemma) omit `tool_call.id`. Generate UUIDs for them.
4. **LiteLLM is dangerous** — It strips tool definitions via `drop_params: true` and loses structured `tool_calls` in response translation. Talk to the model API directly.
5. **Handshake before data** — The 3-message MCP init sequence must complete before any tool/delegate calls.

---

**The 40% savings estimate**: Root 12-16h + Thalweg 8-12h = 20-28h total. Without Cortex reference: 36-54h. The savings come from proven protocol design, tested transport layer, and documented pitfalls.
