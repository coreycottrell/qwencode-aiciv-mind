# mind-mcp — MCP Integration Specialist Mind-Map

**Agent**: mind-mcp
**Owns**: `src/codex-suite-client/`
**Cherry-pick responsibility**: Codex `rmcp-client` (5,952 lines) + study `codex-mcp` (4,440 lines)
**Date**: 2026-04-16

---

## 1. What I Own Today

### 1.1 Crate: `codex-suite-client` (~2,600 lines, 5 files)

The suite client provides **native service integration** for aiciv-mind. Every mind gets a `SuiteClient` at birth — Hub, AgentAuth, AgentCal are not external services, they are the mind's native environment (Principle 12).

```
src/codex-suite-client/
├── Cargo.toml                          # Dependencies: reqwest, ed25519-dalek, codex-llm, codex-exec
├── src/
│   ├── lib.rs                          # ~635 lines — SuiteClient, AuthClient, HubClient, CalClient
│   ├── hub_interceptor.rs              # ~467 lines — 6 Hub tools as ToolInterceptor
│   ├── image_gen_interceptor.rs        # ~457 lines — 2 image gen tools (Gemini API)
│   ├── search_interceptor.rs           # ~409 lines — 2 web search/fetch tools (Ollama Cloud + fallbacks)
│   └── elevenlabs_interceptor.rs       # ~405 lines — 2 TTS tools (ElevenLabs API)
```

### 1.2 Core Types

#### SuiteClient (lib.rs:24)
```rust
pub struct SuiteClient {
    pub auth: AuthClient,    // AgentAuth — Ed25519 challenge-response JWT
    pub hub: HubClient,      // Hub API — rooms, threads, feed
    pub cal: CalClient,      // AgentCal — events, scheduling (stub)
    pub config: SuiteConfig, // URLs + keypair_id
}
```
- Constructed with `SuiteConfig { auth_url, hub_url, cal_url, keypair_id }`
- Each mind gets its own identity: "acg/primary", "acg/research-lead", etc.

#### AuthClient (lib.rs:65)
```rust
pub struct AuthClient {
    base_url: String,
    keypair_id: String,
    token: Option<String>,
    token_issued_at: Option<Instant>,
    credentials: Option<(String, String)>,  // (civ_id, private_key_b64)
    http: reqwest::Client,
}
```
**Auth flow**: POST /challenge → sign with Ed25519 → POST /verify → receive JWT (1hr TTL)
**Key methods**:
- `login(civ_id, private_key_b64)` — full challenge-response auth
- `refresh()` — re-authenticate with stored credentials
- `ensure_fresh()` — auto-refresh if token is >50 minutes old
- `with_token(token)` — inject pre-signed JWT at spawn time
- `is_token_fresh()` — staleness check

#### HubClient (lib.rs:228)
```rust
pub struct HubClient {
    base_url: String,
    http: reqwest::Client,
    token: Option<String>,
}
```
**Endpoints exposed**:
- `list_rooms(group_id)` → GET /api/v1/groups/{id}/rooms
- `list_threads(room_id, limit)` → GET /api/v2/rooms/{id}/threads/list
- `get_thread(thread_id)` → GET /api/v2/threads/{id}
- `create_thread(room_id, title, body)` → POST /api/v2/rooms/{id}/threads
- `reply_to_thread(thread_id, body)` → POST /api/v2/threads/{id}/posts
- `feed(limit)` → GET /api/v2/feed (paginated: `{items, next_cursor, has_more}`)
- `group_feed(group_id, limit)` → GET /api/v2/feed/group/{id}
- `heartbeat(actor_id)` → POST /api/v1/actors/{id}/heartbeat

#### CalClient (lib.rs:486) — **Stub**
Currently just holds `base_url`. No methods implemented yet.

#### SuiteError (lib.rs:500)
Four variants: `Auth(String)`, `Hub(String)`, `Cal(String)`, `Connection(String)`

### 1.3 Interceptors — The ToolInterceptor Pattern

All four interceptors implement the `ToolInterceptor` trait from `codex-llm::think_loop`:

```rust
#[async_trait]
pub trait ToolInterceptor: Send + Sync {
    fn schemas(&self) -> Vec<ToolSchema>;
    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult>;
}
```

**Pattern**: Each interceptor:
1. Returns tool schemas in OpenAI function-calling format via `schemas()`
2. Handles matching tool calls in `handle()`, returning `Some(ToolResult)` for its tools, `None` for unknown names (pass-through)
3. Uses `Arc<Mutex<Client>>` for shared state where needed
4. Validates all required arguments before proceeding
5. Returns structured `ToolResult::ok(output)` or `ToolResult::err(message)`

#### HubInterceptor (hub_interceptor.rs)
- **6 tools**: `hub_list_rooms`, `hub_list_threads`, `hub_read_thread`, `hub_create_thread`, `hub_reply`, `hub_feed`
- **UUID validation**: `validate_uuid()` prevents non-UUID strings (catches "civsubstrate" instead of the actual UUID)
- **Shared state**: `Arc<Mutex<HubClient>>`
- **Tests**: Schema count, unknown tool pass-through, required arg validation, UUID rejection, JSON schema validity, URL construction

#### ImageGenInterceptor (image_gen_interceptor.rs)
- **2 tools**: `generate_image`, `image_styles`
- **Backend**: Inline Python script using `google-genai` SDK with `gemini-3-pro-image-preview`
- **Style presets**: cortex, cyberpunk, minimal, professional, organic, infographic
- **API key**: `GEMINI_API_KEY` → `GOOGLE_API_KEY` fallback
- **Output**: PNG files to configurable output directory
- **Aspect ratios**: 1:1, 16:9, 9:16, 4:3, 3:2, 21:9
- **Timeout**: 120 seconds
- **Python venv**: Auto-discovers `.venv/bin/python3` in project root

#### SearchInterceptor (search_interceptor.rs)
- **2 tools**: `web_search`, `web_fetch`
- **Primary path**: Ollama Cloud API (`/api/web_search`, `/api/web_fetch`) using `OLLAMA_API_KEY`
- **Fallbacks**: DuckDuckGo (via `ddgs` Python package) for search, Jina Reader (`r.jina.ai`) for fetch
- **Content truncation**: 8000 chars max for fetched pages
- **Timeouts**: 20s (Ollama search), 25s (Ollama fetch), 30s (fallbacks)
- **URL validation**: Must start with `http://` or `https://`

#### ElevenLabsInterceptor (elevenlabs_interceptor.rs)
- **2 tools**: `tts_speak`, `tts_voices`
- **Backend**: ElevenLabs v1 API via curl (`POST /v1/text-to-speech/{voice_id}`)
- **Voice presets**: Daniel (A-C-Gee default), Adam (True Bearing), Matilda (Witness)
- **Voice resolution**: By name ("Daniel"), civ name ("acg", "witness", "true-bearing"), or raw voice ID
- **Default model**: `eleven_turbo_v2_5`
- **Text limit**: 5000 characters
- **File validation**: Rejects output <100 bytes (likely error response)
- **Timeout**: 180 seconds

### 1.4 Dependencies

```toml
[dependencies]
serde, serde_json       # Serialization
thiserror               # Error derives
chrono                  # Timestamps (image filenames)
async-trait             # ToolInterceptor trait
tokio                   # Async runtime + process spawning
reqwest                 # HTTP client
uuid                    # UUID validation in hub interceptor
tracing                 # Logging
ed25519-dalek           # Ed25519 signing for AgentAuth
base64                  # Key encoding
codex-llm               # ToolInterceptor, ToolSchema, FunctionSchema
codex-exec              # ToolResult
```

### 1.5 Test Coverage

Every interceptor has tests covering:
- Schema count and content validation
- Unknown tool pass-through (returns `None`)
- Required argument validation
- Input validation (UUIDs, URLs, text length, aspect ratios)
- Style/voice resolution
- JSON schema validity (every schema has `type: "object"` + `properties`)
- URL construction (trailing slash stripping)
- Live integration tests (`#[ignore]`) for auth login and Hub feed

**Total tests**: ~30 unit tests + 3 live integration tests

---

## 2. What I Need to Cherry-Pick: Codex `rmcp-client` (5,952 lines)

### 2.1 Architecture Overview

The `rmcp-client` is a **low-level MCP client** that wraps the upstream `rmcp` Rust SDK. It manages a single connection to one MCP server, handling:
- Transport creation (stdio subprocess or Streamable HTTP)
- OAuth lifecycle (credential storage, login flow, token refresh)
- Session recovery (automatic reconnection on 404)
- Tool/resource operations

```
rmcp-client/src/
├── lib.rs                      # 32 lines — module declarations, re-exports
├── rmcp_client.rs              # ~1,210 lines — core client: transport, init, tool ops, session recovery
├── oauth.rs                    # ~923 lines — OAuth credential storage (keyring + file fallback)
├── perform_oauth_login.rs      # ~657 lines — full OAuth authorization code flow
├── auth_status.rs              # ~374 lines — OAuth discovery (RFC 8414)
├── logging_client_handler.rs   # 137 lines — logs MCP notifications
├── program_resolver.rs         # 225 lines — cross-platform executable resolution
└── utils.rs                    # 215 lines — env var construction, HTTP headers
```

### 2.2 Key Types

#### RmcpClient (rmcp_client.rs:470)
```rust
pub struct RmcpClient {
    state: Mutex<ClientState>,              // Connecting or Ready
    transport_recipe: TransportRecipe,      // Recipe to recreate transport
    initialize_context: Mutex<Option<InitializeContext>>,
    session_recovery_lock: Mutex<()>,       // Serializes recovery attempts
}
```

#### ClientState (rmcp_client.rs:318)
```rust
enum ClientState {
    Connecting { transport: Option<PendingTransport> },
    Ready { service, oauth, _process_group_guard },
}
```

#### TransportRecipe (rmcp_client.rs:388)
Stores parameters needed to **recreate** a transport for automatic reconnection:
```rust
enum TransportRecipe {
    Stdio { program, args, env, env_vars, cwd },
    StreamableHttp { server_name, url, bearer_token, http_headers, env_http_headers, store_mode },
}
```

#### PendingTransport (rmcp_client.rs:304)
```rust
enum PendingTransport {
    ChildProcess { transport: TokioChildProcess, process_group_guard },
    StreamableHttp { transport },
    StreamableHttpWithOAuth { transport, oauth_persistor },
}
```

### 2.3 Connection Lifecycle

```
new_stdio_client() / new_streamable_http_client()
    → Creates TransportRecipe + PendingTransport
    → State: Connecting

initialize(params, timeout, elicitation_callback)
    → MCP handshake via service::serve_client()
    → State: Ready
    → Persists OAuth tokens if applicable

Operations: list_tools(), call_tool(), list_resources(), read_resource()
    → refresh_oauth_if_needed()     (proactive: 30s before expiry)
    → run_service_operation()       (delegates to RunningService)
    → persist_oauth_tokens()        (save any refreshed tokens)
    → On 404 "session expired" → reinitialize_after_session_expiry()
```

### 2.4 Session Recovery (Critical Pattern)

When `run_service_operation()` catches a 404 "session expired" from Streamable HTTP:

1. Acquires `session_recovery_lock` (prevents concurrent recovery)
2. Checks if another recovery already succeeded (Arc pointer comparison)
3. Recreates transport from stored `TransportRecipe`
4. Reconnects with saved `InitializeContext`
5. Replaces `ClientState::Ready`
6. **Retries the failed operation once** on the new connection

### 2.5 Transport Mechanisms

**Stdio**:
- Spawns child process via `tokio::process::Command`
- Clears environment, injects whitelisted vars only (PATH, HOME, SHELL on Unix)
- Creates process group (`process_group(0)`) for clean cleanup
- `ProcessGroupGuard` sends SIGTERM, then SIGKILL after 2s grace on drop
- Logs stderr in background task

**Streamable HTTP**:
- Uses `StreamableHttpClientTransport` from `rmcp` crate
- `StreamableHttpResponseClient` handles session ID headers, content type negotiation (SSE vs JSON), 404 detection
- Three auth modes:
  1. Static bearer token (from env var or config)
  2. OAuth with `AuthClient` wrapper (loads stored tokens, creates OAuthState)
  3. Fallback: plain bearer token if OAuth discovery fails

### 2.6 OAuth Support (Deep)

**Credential storage** (oauth.rs):
- Primary: OS keyring (macOS Keychain, Windows Credential Manager, Linux DBus)
- Fallback: `CODEX_HOME/.credentials.json` (chmod 600 on Unix)
- Key = `{server_name}|{sha256_prefix(canonical_payload)}`
- Token expiry tracked as absolute millisecond timestamp

**Login flow** (perform_oauth_login.rs):
- Starts local `tiny_http` callback server on `127.0.0.1:0`
- RFC 8414 OAuth discovery (`.well-known/oauth-authorization-server`)
- PKCE authorization code flow
- Opens browser for user auth (or returns URL for programmatic use)
- 300s default timeout for callback
- Persists tokens to keyring/file after exchange

**Proactive refresh**: Tokens refreshed 30s before expiry (`REFRESH_SKEW_MILLIS = 30_000`). Tokens persisted after every operation, not just explicit refresh.

### 2.7 Elicitation Support

```rust
pub type SendElicitation = Box<
    dyn Fn(RequestId, Elicitation) -> BoxFuture<'static, Result<ElicitationResponse>>
    + Send + Sync,
>;
```
Callback for routing MCP server requests for user input to the UI layer.

---

## 3. What I Need to Study: Codex `codex-mcp` (4,440 lines)

### 3.1 Architecture Overview

The **high-level MCP connection manager**. Manages multiple `RmcpClient` instances simultaneously. This is the layer between the agent and all its MCP servers.

```
codex-mcp/src/
├── lib.rs                      # 2 lines — module declarations
├── mcp/
│   ├── mod.rs                  # ~477 lines — McpConfig, tool qualification, snapshots
│   ├── auth.rs                 # ~300 lines — auth status, OAuth scope resolution
│   └── skill_dependencies.rs   # ~167 lines — auto-detect missing MCP dependencies
└── mcp_connection_manager.rs   # ~1,699 lines — the manager: multi-server, caching, sandbox
```

### 3.2 Key Types

#### McpConnectionManager (mcp_connection_manager.rs:581)
```rust
pub struct McpConnectionManager {
    clients: HashMap<String, AsyncManagedClient>,    // server_name → client
    server_origins: HashMap<String, String>,          // tool_name → server_name
    elicitation_requests: ElicitationRequestManager,
}
```

#### AsyncManagedClient (line 407)
```rust
struct AsyncManagedClient {
    client: Shared<BoxFuture<'static, Result<ManagedClient, StartupOutcomeError>>>,
    startup_snapshot: Option<Vec<ToolInfo>>,    // cached tools served during init
    startup_complete: Arc<AtomicBool>,
    tool_plugin_provenance: Arc<ToolPluginProvenance>,
}
```
Key insight: wraps startup as a `Shared` future so multiple callers can `.await` the same initialization. Tools served from snapshot cache while server connects.

#### ManagedClient (line 354)
```rust
struct ManagedClient {
    client: Arc<RmcpClient>,
    tools: Vec<ToolInfo>,
    tool_filter: ToolFilter,
    tool_timeout: Option<Duration>,
    server_supports_sandbox_state_capability: bool,
    codex_apps_tools_cache_context: Option<CodexAppsToolsCacheContext>,
}
```

#### ToolInfo (line 184)
```rust
pub struct ToolInfo {
    pub server_name: String,
    pub tool_name: String,
    pub tool_namespace: String,
    pub tool: Tool,           // rmcp Tool type
    pub connector_id: Option<String>,
    pub connector_name: Option<String>,
    pub plugin_display_names: Vec<String>,
    pub connector_description: Option<String>,
}
```

#### McpConfig (mod.rs:73)
```rust
pub struct McpConfig {
    pub configured_mcp_servers: HashMap<String, McpServerConfig>,
    pub approval_policy: Constrained<AskForApproval>,
    pub codex_home: PathBuf,
    pub mcp_oauth_credentials_store_mode: OAuthCredentialsStoreMode,
    pub mcp_oauth_callback_port: Option<u16>,
    pub skill_mcp_dependency_install_enabled: bool,
    pub codex_linux_sandbox_exe: Option<PathBuf>,
    pub apps_enabled: bool,
    // ... more fields
}
```

### 3.3 Manager Lifecycle

```
McpConnectionManager::new(config, auth_entries, sandbox_state, event_channel)
    → For each enabled server config:
        → Load startup snapshot from disk cache (if available)
        → Spawn async init future: make_rmcp_client() → start_server_task()
        → Wrap in Shared<BoxFuture> (multiple awaiters OK)
    → Spawn JoinSet monitoring all startup futures
    → Emit McpStartupUpdate / McpStartupComplete events
    → Returns (McpConnectionManager, CancellationToken)

start_server_task():
    → InitializeRequestParams with ProtocolVersion::V_2025_06_18
    → client.initialize(params, timeout, send_elicitation)
    → list_tools_for_client_uncached()
    → Check for codex/sandbox-state capability
    → Returns ManagedClient
```

### 3.4 Tool Name Qualification

Global uniqueness guaranteed by the `qualify_tools()` function:
- Regular servers: `mcp__{server_name}__{tool_name}`
- Name sanitization: only `[a-zA-Z0-9_-]+` allowed
- Truncation: max 64 chars (appends SHA1 suffix for uniqueness)
- Splitting: `split_qualified_tool_name("mcp__alpha__do_thing")` → `("alpha", "do_thing")`

### 3.5 Startup Snapshot Pattern

For servers with disk caches, tools can be served from cache while the server initializes:
1. On first connection, tools are cached to disk (keyed by SHA1 of auth identity)
2. On subsequent starts, `startup_snapshot` is loaded from cache
3. `list_all_tools()` returns snapshot tools when `startup_complete` is false
4. Once the server finishes connecting, real tools replace the snapshot

### 3.6 Sandbox State Propagation

Custom Codex extension to MCP:
- Capability: `codex/sandbox-state` (in server's `experimental` capabilities)
- Method: `codex/sandbox-state/update` (custom MCP request)
- Payload: `SandboxState { sandbox_policy, sandbox_exe, cwd, use_legacy_landlock }`
- Pushed on initial connection + whenever `notify_sandbox_state_change()` is called

### 3.7 Elicitation Management

`ElicitationRequestManager` routes MCP server requests for user input:
- Per `(server_name, request_id)` oneshot channels
- Approval policy enforcement: `AskForApproval::Never` → auto-decline
- Two request types: Form-based (JSON schema) or URL-based
- Runtime policy updates via `set_approval_policy()`

### 3.8 Skill Dependencies Auto-Detection

`collect_missing_mcp_dependencies()`:
- Scans skill metadata for `type: "mcp"` tool dependencies
- Builds canonical keys for deduplication
- Returns `HashMap<String, McpServerConfig>` of needed servers
- Supports both `stdio` and `streamable_http` transport types

---

## 4. Integration Points with Other Agents

### 4.1 What I Provide

| Consumer | Interface | What I Give |
|----------|-----------|-------------|
| **mind-tool-engine** (codex-exec) | `ToolInterceptor` trait | Suite interceptors register into tool registry. MCP-discovered tools would also register here. |
| **mind-model-router** (codex-llm) | `ToolSchema` + `ToolResult` | Interceptor schemas injected into LLM prompts. Tool results returned to think loop. |
| **mind-coordination** (cortex) | `SuiteClient` | Every mind gets a SuiteClient at boot. Hub, Auth, Cal are native capabilities. |

### 4.2 What I Need

| Provider | What I Need | Status |
|----------|-------------|--------|
| **mind-tool-engine** (codex-exec) | `ToolResult` type, tool registry interface for MCP-discovered tools | **HAVE** — already using `codex_exec::ToolResult` |
| **mind-model-router** (codex-llm) | `ToolInterceptor` trait, `ToolSchema`, `FunctionSchema` types | **HAVE** — already using `codex_llm::think_loop::ToolInterceptor` |
| **mind-auth** | OAuth credentials for remote MCP servers | **NEED** — not yet built. Currently Auth is AgentAuth Ed25519 only. |
| **mind-coordination** (codex-types) | `McpServerConfig`, `McpToolSpec` type definitions | **NEED** — types for MCP server configuration |

### 4.3 Current Integration Chain

```
LLM (codex-llm) → ThinkLoop → ToolInterceptor.schemas()    → adds to prompt
                              → ToolInterceptor.handle(name) → executes tool
                                                              → returns ToolResult
```

The interceptors sit in the think loop pipeline. When the LLM outputs a tool call:
1. ThinkLoop checks each registered interceptor's `handle()` in order
2. First interceptor to return `Some(ToolResult)` wins
3. If all return `None`, the call falls through to the standard tool executor (codex-exec)

---

## 5. Phase 2 Plan: MCP Client Integration

### 5.1 What to Build

Build an MCP client module that can connect to any external MCP server and expose its tools through the tool registry. This is the bridge between the aiciv-mind harness and the MCP ecosystem.

### 5.2 Cherry-Pick Strategy

#### Take from rmcp-client (adapted, not verbatim):

| Component | Codex Lines | Adaptation Needed |
|-----------|-------------|-------------------|
| Transport creation (stdio + HTTP) | ~400 | Replace OpenAI-specific env vars with aiciv config |
| Session recovery logic | ~200 | Keep — this is transport-agnostic |
| Process group management | ~100 | Keep — clean subprocess cleanup |
| OAuth credential storage | ~500 | Simplify — we have AgentAuth, only need OAuth for *remote* MCP servers |
| OAuth login flow | ~650 | Keep for remote MCP servers (e.g., GitHub MCP) |
| Auth status discovery | ~370 | Keep — RFC 8414 is standard |

#### Take from codex-mcp (patterns, not code):

| Pattern | Why |
|---------|-----|
| `AsyncManagedClient` + `Shared<BoxFuture>` | Non-blocking startup — serve cached tools while servers connect |
| Tool name qualification (`mcp__{server}__{tool}`) | Global uniqueness across multiple MCP servers |
| `ToolFilter` (allowlist/denylist) | Per-server tool filtering |
| Elicitation routing | Forward server input requests to UI |
| Skill dependency auto-detection | Auto-install missing MCP servers from skill metadata |

#### Do NOT take:

| Component | Why Not |
|-----------|---------|
| Codex Apps integration | ChatGPT-specific, not relevant |
| `codex/sandbox-state` capability | Codex-specific extension, we'd build our own if needed |
| Plugin provenance tracking | Codex plugin ecosystem, not relevant |
| Startup snapshot disk cache | Over-engineering for initial implementation |

### 5.3 Implementation Architecture

```
src/codex-suite-client/          (or new crate: src/aiciv-mcp/)
├── src/
│   ├── lib.rs                   # Existing — add MCP module re-exports
│   ├── mcp_client.rs            # NEW — AicivMcpClient (wraps rmcp)
│   ├── mcp_manager.rs           # NEW — multi-server manager
│   ├── mcp_transport.rs         # NEW — stdio + HTTP transport creation
│   ├── mcp_oauth.rs             # NEW — OAuth for remote MCP (cherry-picked)
│   ├── hub_interceptor.rs       # Existing
│   ├── image_gen_interceptor.rs # Existing
│   ├── search_interceptor.rs    # Existing
│   └── elevenlabs_interceptor.rs # Existing
```

**Or** create a separate `src/aiciv-mcp/` crate to keep suite-client focused on AiCIV services and MCP focused on external server connections. **Decision**: Recommend **separate crate** — different concerns (native AiCIV services vs. generic MCP protocol), different dependencies (rmcp SDK), different lifecycle.

### 5.4 Key Design Decisions

#### Decision 1: Separate crate vs. extend codex-suite-client

**Recommendation: New `aiciv-mcp` crate.**
- codex-suite-client is about AiCIV's own services (Hub, Auth, Cal)
- MCP client is about connecting to arbitrary external servers
- Different dependency tree (rmcp SDK, OAuth libs)
- Different ownership scope in the 10-agent model
- Mind-mcp owns both crates but they serve different purposes

#### Decision 2: rmcp SDK dependency

**Recommendation: Use the `rmcp` Rust crate directly.**
- Codex's rmcp-client already wraps it — proves it works
- Provides `TokioChildProcess`, `StreamableHttpClientTransport`, `RunningService`
- Handles JSON-RPC protocol, MCP handshake, tool/resource types
- We adapt the wrapper (transport creation, OAuth, session recovery), not the SDK

#### Decision 3: Tool registration bridge

**Recommendation: New `McpToolInterceptor` that implements `ToolInterceptor`.**
- Discovers tools from connected MCP servers
- Qualifies names: `mcp__{server}__{tool}`
- Registers into the think loop pipeline alongside existing interceptors
- When LLM calls an MCP-qualified tool, routes to the correct server via `call_tool()`

#### Decision 4: Config format

**Recommendation: JSON config (not TOML) — consistent with aiciv-mind's config/ directory.**
```json
{
  "mcp_servers": {
    "github": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" }
    },
    "filesystem": {
      "transport": "streamable_http",
      "url": "http://localhost:3000/mcp",
      "bearer_token": "${FS_TOKEN}"
    }
  }
}
```

### 5.5 Phased Build

**Phase 2a: Single-server MCP client** (smallest useful increment)
- Wrap `rmcp` SDK for stdio transport
- Connect to one MCP server, discover tools, expose via `ToolInterceptor`
- Test with `@modelcontextprotocol/server-filesystem`
- ~500 lines

**Phase 2b: Multi-server manager**
- HashMap of server name → client
- Tool name qualification
- Parallel startup
- ~400 lines

**Phase 2c: Streamable HTTP + OAuth**
- HTTP/SSE transport
- OAuth credential storage (simplified — AgentAuth for our servers, OAuth for remote)
- Session recovery on 404
- ~800 lines

**Phase 2d: Integration with tool registry**
- Bridge MCP-discovered tools into mind-tool-engine's registry
- Coordinate with mind-tool-engine for registration interface
- ~200 lines

---

## 6. Dependency Map

### 6.1 Current Dependency Graph

```
codex-suite-client
├── codex-llm        # ToolInterceptor, ToolSchema, FunctionSchema
├── codex-exec       # ToolResult
├── reqwest          # HTTP client
├── ed25519-dalek    # AgentAuth signing
├── base64           # Key encoding
├── uuid             # Hub UUID validation
├── chrono           # Timestamps
├── tokio            # Async + process spawning
├── async-trait      # Trait async methods
├── tracing          # Logging
├── serde/json       # Serialization
└── thiserror        # Error types
```

### 6.2 New Dependencies for MCP Client (Phase 2)

```
aiciv-mcp (new crate)
├── rmcp             # MCP SDK — transport, JSON-RPC, protocol types
├── codex-llm        # ToolInterceptor (for McpToolInterceptor)
├── codex-exec       # ToolResult
├── tokio            # Async runtime
├── serde/json       # Config parsing
├── tracing          # Logging
├── reqwest          # HTTP transport
├── keyring          # OAuth credential storage (optional)
└── tiny_http        # OAuth callback server (optional)
```

---

## 7. Risk Analysis

### 7.1 Cherry-Pick Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `rmcp` SDK version mismatch | Medium | Pin to same version Codex uses; check Cargo.lock |
| OAuth complexity for initial version | Low | Defer OAuth to Phase 2c; start with stdio only |
| Session recovery edge cases | Medium | Port Codex's tested logic; add our own integration tests |
| Process group cleanup on Linux | Low | Codex's `ProcessGroupGuard` is battle-tested |
| Tool name collisions across servers | Low | Codex's qualification scheme is proven |

### 7.2 Integration Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `ToolInterceptor` trait changes | Medium | Owned by mind-model-router; coordinate via scratchpad |
| Tool registry interface changes | Medium | Owned by mind-tool-engine; need stable registration API |
| Auth provider for remote MCP | Low | mind-auth will provide OAuth; AgentAuth covers our servers |
| Type definitions for MCP config | Low | Request `McpServerConfig` from mind-coordination via scratchpad |

---

## 8. Comparison: Current State vs. Target

| Capability | Current | Target (Post Phase 2) |
|------------|---------|----------------------|
| AiCIV Hub tools | 6 tools via HubInterceptor | Same |
| Image generation | 2 tools via ImageGenInterceptor | Same |
| Web search/fetch | 2 tools via SearchInterceptor | Same |
| Text-to-speech | 2 tools via ElevenLabsInterceptor | Same |
| External MCP servers | **None** | Connect to any MCP server, discover + expose tools |
| MCP transport: stdio | **None** | Spawn subprocess, manage lifecycle |
| MCP transport: HTTP/SSE | **None** | Streamable HTTP with session management |
| MCP OAuth | **None** | OAuth login flow for remote MCP servers |
| Session recovery | **None** | Auto-reconnect on session expiry |
| Multi-server management | **None** | Parallel server connections with tool aggregation |
| Tool name qualification | N/A (tools are hardcoded) | `mcp__{server}__{tool}` naming |
| Startup snapshots | **None** | Cached tools served during server init |

---

## 9. Codex Deep-Map Insights Relevant to MCP

### 9.1 Scale Context

From the deep-map: Codex's MCP subsystem is **14K lines across 3 crates** (rmcp-client 6K + codex-mcp 4.4K + mcp-server 3.5K). Our target is ~2K lines covering the client side only (no need for mcp-server — aiciv-mind doesn't need to be an MCP server initially).

### 9.2 Boundary Quality

The deep-map rates MCP boundary quality as **MODERATE**: "MCP connection management is clean. The rmcp-client is isolated. But MCP tool calls flow through core's tool execution pipeline."

For us, this is actually **better** than Codex's situation because:
- We have the clean `ToolInterceptor` trait as our integration point
- We don't have Codex's 204K-line `core` monolith
- Our tool execution pipeline (codex-exec) is 700 lines, not tangled

### 9.3 What Codex Does That We Don't Need

- **MCP server mode** (3,538 lines) — aiciv-mind doesn't need to expose itself as an MCP server (yet)
- **Codex Apps integration** — ChatGPT-specific connector marketplace
- **Plugin provenance** — maps connectors to plugin display names
- **Guardian auto-approval for MCP tools** — we can build simpler approval if needed

### 9.4 What Codex Does That We Should Study

- **Protocol version negotiation** (`ProtocolVersion::V_2025_06_18`) — MCP evolves fast
- **Tool timeout per-server** — some MCP servers are slow
- **Parallel tool call support flag** (`supports_parallel_tool_calls`) — not all servers support it
- **Resource and resource template listing** — beyond just tools

---

## 10. Open Questions

1. **New crate or extend existing?** Recommendation is new `aiciv-mcp` crate, but mind-lead should confirm.
2. **rmcp SDK version?** Need to check which version Codex uses and whether it's compatible with our Cargo workspace.
3. **Config location?** `config/mcp_servers.json` seems natural. Need mind-coordination to add `McpServerConfig` to codex-types.
4. **Priority?** MCP client is Phase 2 work per MISSIONS.md. Should I start foundation (types, config) now, or wait for all Phase 1 work to complete?
5. **CalClient implementation?** Currently a stub. Should I implement it as an interceptor (like Hub), or is it lower priority?

---

## 11. Summary

**Current state**: codex-suite-client provides 12 tools across 4 interceptors (Hub, ImageGen, Search, TTS), plus AuthClient, HubClient, and CalClient. Well-tested, production-ready for AiCIV's native services.

**Cherry-pick target**: Codex's rmcp-client (5,952 lines) provides a battle-tested MCP client with stdio/HTTP transport, OAuth, and session recovery. codex-mcp (4,440 lines) provides multi-server management patterns.

**Integration path**: New `aiciv-mcp` crate using rmcp SDK directly, adapting Codex's transport/OAuth/recovery patterns, exposing MCP-discovered tools through the existing `ToolInterceptor` pipeline.

**Estimated new code**: ~2,000 lines across 4 phases, leveraging rmcp SDK for protocol-level work.
