# Mind-Map: mind-model-router Domain

**Agent**: mind-model-router
**Crates Owned**: `codex-llm` (~3,800 LOC), `codex-drive` (~2,000 LOC), `qwen-mind` (~1,400 LOC)
**Total**: ~7,200 lines across 3 crates, 20 source files
**Last Updated**: 2026-04-16

---

## 1. What Exists — Complete Inventory

### codex-llm (5 files, ~3,800 lines)

```
codex-llm/
  src/
    lib.rs            (19 lines)  — module root, re-exports
    ollama.rs         (~793 lines) — OllamaClient, ModelRouter, ChatMessage types
    prompt.rs         (~413 lines) — PromptBuilder, role-based system prompts, AGENTS.md injection
    rate_limiter.rs   (~502 lines) — RateLimiter circuit breaker, JSONL metrics, usage tool
    think_loop.rs     (~1,800+ lines) — THE core reasoning cycle, tool interceptors, Challenger integration
```

**Key structs:**
- `OllamaClient` — HTTP client for Ollama's native `/api/chat` endpoint. Supports both local and cloud (Bearer token auth). Retry with exponential backoff (1s/2s/4s), 3 retries. 5-min request timeout.
- `OllamaConfig` — base_url, model, temperature, max_tokens, optional api_key
- `ModelRouter` — Role-based model selection (Primary/TeamLead/Agent/Lightweight). Default: `devstral-small-2:24b` (cloud), `qwen2.5:7b` (local), `minimax-m2.7` or `phi3:mini` (lightweight)
- `ChatMessage` — OpenAI-compatible message format (role, content, tool_calls, tool_call_id). Factory methods: `system()`, `user()`, `assistant()`, `tool_result()`, `assistant_with_tool_calls()`
- `ToolSchema` / `FunctionSchema` — OpenAI function calling format
- `ChatResponse` / `Choice` / `Usage` — Response types
- `LlmError` — Connection / Api / Parse errors with `is_retryable()` classification
- `PromptBuilder` — Builds role-specific system prompts with AGENTS.md injection and extra context
- `RateLimiter` — Arc<Mutex<>> thread-safe circuit breaker. 5 consecutive errors trips breaker, 60s cooldown. JSONL logging to `ollama-usage.jsonl`. Cost estimation ($0.30/1M in, $1.20/1M out).
- `ThinkLoop` — THE reasoning engine. Prompt → LLM → Tool Calls → Execute → Inject Results → Loop. Max 15 iterations. Challenger integration for stall detection (2 consecutive critical stalls = kill).
- `ThinkResult` — Final response + tool call records + iteration count + stall state
- `ToolInterceptor` trait — Inject custom tools (used by TeamLeads for spawn/delegate). Composite chaining supported.

**Built-in tools the ThinkLoop provides:**
- `memory_search` / `memory_write` (via codex-memory)
- `scratchpad_read` / `scratchpad_write` (per-mind daily files)
- `coordination_read` / `coordination_write` (shared coordination pad)
- `team_scratchpad_read` / `team_scratchpad_write` (team-level pads)
- `hum_digest` (Hum observation JSONL)
- `ollama_usage` (rate limiter dashboard)

**M2.7 quirk handling:**
- `sanitize_json_string()` — strips trailing commas before `}` or `]` (15-20% occurrence)
- `normalize_args()` — remaps common param aliases (`path`→`file_path`, `cmd`→`command`, etc.) for 15+ tool types
- Content preservation across turns — `assistant_with_tool_calls()` keeps `<think>` tags to prevent 35-40% degradation

### codex-drive (4 files, ~2,000 lines)

```
codex-drive/
  src/
    lib.rs           (19 lines)  — module root, re-exports
    drive_loop.rs    (~707 lines) — DriveLoop autonomous heartbeat
    event_bus.rs     (~222 lines) — Dual-channel event router with biased priority
    task_store.rs    (~863 lines) — SQLite-backed task queue
```

**Key structs:**
- `TaskStore` — SQLite-backed task queue. States: Open/InProgress/Completed/Failed/Blocked. Priority: Low/Normal/High/Critical. Dependency chains with auto-unblocking on completion. Boot recovery for stale in-progress tasks.
- `EventBus` — Dual-channel: External (capacity=64) + Drive (capacity=1). `tokio::select! { biased }` ensures external events always preempt drive events. Natural backpressure via channel capacity.
- `DriveLoop` — Autonomous heartbeat. Boot settle → Crash recovery → Main loop (completion trigger OR idle timeout → cooldown → yield to external → scan → emit). Adaptive backoff: unproductive events increase idle threshold (1.5x up, 2x down recovery). Completion fast-path for dependency chain instant chaining.
- `DriveConfig` — boot_settle (5s), cooldown (2s), idle_threshold (30s→300s adaptive), stall_threshold (120s), backoff multiplier (1.5), recovery divisor (2.0)

**Event types emitted:**
- `DriveEvent::TaskAvailable` — a ready task was found
- `DriveEvent::StallDetected` — an InProgress task exceeded stall threshold
- `DriveEvent::HealthCheck` — periodic health with active_minds/pending_tasks
- `DriveEvent::IdleSuggestion` — nothing to do, suggest self-improvement

### qwen-mind (11 files, ~1,400 lines)

```
qwen-mind/
  src/
    lib.rs           (25 lines)  — module root, re-exports
    identity.rs      (~182 lines) — Manifest, Role, GrowthStage, MemoryCategory, MemoryTier
    mind.rs          (~253 lines) — Mind struct: think loop integration
    llm.rs           (~158 lines) — Simpler OllamaClient (system+user only, no tool use)
    planning.rs      (~143 lines) — PlanningGate: heuristic complexity assessment
    fitness.rs       (~193 lines) — FitnessTracker: evidence-based scoring to JSONL
    delegation.rs    (~136 lines) — DelegationRules: structural spawn/delegate constraints
    memory.rs        (~96 lines)  — MindMemory: cortex-memory wrapper with sharing threshold
    spawner.rs       (~169 lines) — MindProcess: subprocess spawning with file-based IPC
    scratchpad.rs    (~55 lines)  — Scratchpad: append-only daily files
    bin/main.rs      (~205 lines) — Binary: standalone or ZeroMQ IPC mode
```

**Key structs:**
- `Mind` — The persistent AI mind. Composes: Manifest + MindMemory + Scratchpad + FitnessTracker + OllamaClient + DelegationRules. Has its own `think()` method (simpler than ThinkLoop — no tool use, just system+user prompt).
- `Manifest` — Identity + role + vertical + growth stage + session count + principles + anti-patterns. JSON serializable. Growth stages: Novice (<10) → Competent → Proficient → Advanced → Expert (500+).
- `DelegationRules` — Hard structural constraints: Primary→TeamLead, TeamLead→Agent (same vertical only). Agent cannot spawn/delegate. Primary cannot reach Agent directly.
- `PlanningGate` — Heuristic complexity: Trivial (replay prior)/Simple/Medium/Complex/Novel. Uses word count + comma/semicolon density + subtask keywords.
- `FitnessTracker` — Score = 0.7 * evidence + 0.3 * citation. Evidence = 0.35*completion + 0.20*no_errors + 0.25*specificity + 0.20*memory_written. JSONL output.
- `MindProcess` — File-based IPC subprocess spawning. Phase 1a: task/result files. Phase 1b planned: ZeroMQ REQ/REP.

---

## 2. How Models Get Routed — Current State

### Two Parallel LLM Client Implementations

**IMPORTANT FINDING**: There are **two separate OllamaClient implementations** that do NOT share code:

| Feature | `codex-llm::OllamaClient` | `qwen-mind::OllamaClient` |
|---------|---------------------------|---------------------------|
| **API endpoint** | Native `/api/chat` | Native `/api/chat` |
| **Tool calling** | Full tool use (ToolSchema, tool_calls parsing) | None — system+user prompt only |
| **Retry** | 3 retries, exp backoff (1s/2s/4s) | 3 retries, exp backoff (30s*2^n) |
| **Rate limiting** | Circuit breaker integration | None |
| **Auth** | Bearer token (Ollama Cloud) | Bearer token (Ollama Cloud) |
| **Response type** | `ChatResponse` (OpenAI-compat) | `LlmResponse` (content + retries) |
| **Used by** | `ThinkLoop` (cortex binary) | `Mind.think()` (qwen-mind binary) |

### ModelRouter — Role-Based Selection (codex-llm only)

```
ModelRouter
├── primary_model    → devstral-small-2:24b (cloud) / qwen2.5:7b (local)
├── team_lead_model  → devstral-small-2:24b (cloud) / qwen2.5:7b (local)
├── agent_model      → devstral-small-2:24b (cloud) / qwen2.5:7b (local)
└── lightweight_model → minimax-m2.7 (cloud) / phi3:mini (local)
```

**Current routing logic**: Static per-role assignment via `config_for_role(role)`. No dynamic task-based routing. Cloud models get 16384 max_tokens (thinking overhead), local get 4096.

**Environment variables:**
- `OLLAMA_API_KEY` — triggers cloud mode
- `OLLAMA_BASE_URL` — defaults to `https://api.ollama.com` (cloud) or `localhost:11434` (local)
- `CORTEX_PRIMARY_MODEL`, `CORTEX_TEAM_LEAD_MODEL`, `CORTEX_AGENT_MODEL`, `CORTEX_LIGHTWEIGHT_MODEL` — per-role overrides

### No Multi-Model Routing Yet

Currently:
- All roles get the **same model** (devstral or qwen2.5) except lightweight tasks
- No task-complexity-based routing (e.g., simple bash → phi3, complex planning → opus)
- No fallback chains (if model A fails, try model B)
- No provider switching (all goes through single Ollama endpoint)
- `lightweight_model` exists but only used by `config_lightweight()` — called explicitly, not auto-selected

---

## 3. OpenAI-Compatible API Support Status

### What's Implemented
- **Internal types are OpenAI-compatible**: `ChatMessage`, `ChatResponse`, `Choice`, `Usage`, `ToolCallMessage`, `FunctionCall`, `ToolSchema` all follow OpenAI's chat completion format
- **Native Ollama API used for transport**: The actual HTTP calls use `/api/chat` (Ollama native), not `/v1/chat/completions` (OpenAI-compat)
- **Conversion layer**: `convert_native_response()` maps Ollama native response → internal OpenAI-compat format
- **Tool calling**: Full function calling via Ollama's native tool format (arguments as JSON objects, not strings)

### What's Missing for True OpenAI-Compatible API
- **No `/v1/chat/completions` endpoint support** — cannot talk to raw OpenAI API, OpenRouter, etc.
- **No streaming** — `"stream": false` hardcoded. No SSE support.
- **No provider abstraction** — OllamaClient is the only provider. No trait for swappable providers.
- **No response format control** — no `response_format: json_object` support

### Key Design Decision (documented in ollama.rs)
> "Uses Ollama's native API (not OpenAI-compatible), since Ollama Cloud's `/v1/` endpoint doesn't support all auth modes."

---

## 4. What's Needed for True Multi-Model Routing

### Phase 1: Provider Abstraction (PREREQUISITE)

```rust
#[async_trait]
trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage], tools: Option<&[ToolSchema]>) -> Result<ChatResponse, LlmError>;
    fn model_name(&self) -> &str;
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
}
```

Implementations needed:
- `OllamaProvider` — current OllamaClient (native API)
- `OpenAiCompatProvider` — `/v1/chat/completions` for OpenRouter, Anthropic, etc.
- `LocalOllamaProvider` — localhost Ollama (no auth, lower timeouts)

### Phase 2: Task-Complexity Router

```
Incoming Task
    │
    ├─ PlanningGate.assess(task) → complexity
    │
    ├─ Trivial/Simple → lightweight_model (phi3/m2.7)
    │   Cost: ~$0.001/call, latency: ~2s
    │
    ├─ Medium → agent_model (devstral/qwen2.5)
    │   Cost: ~$0.01/call, latency: ~5s
    │
    └─ Complex/Novel → primary_model (opus/gemma4-large)
        Cost: ~$0.10/call, latency: ~15s
```

Integration point: `ThinkLoop.run()` should accept a `ModelRouter` and use `PlanningGate` output to select the model before each LLM call (not just at initialization).

### Phase 3: Fallback Chains

```
Primary: devstral-small-2:24b (Ollama Cloud)
    │ if 429/timeout
    ├─ Fallback 1: qwen2.5:7b (local Ollama)
    │ if local down
    └─ Fallback 2: minimax-m2.7 (Ollama Cloud, different model)
```

### Phase 4: Mid-Session Model Switching

Allow the ThinkLoop to switch models between iterations:
- Start with cheap model for tool calls
- Escalate to expensive model for synthesis/final answer
- Drop to lightweight for memory/scratchpad operations

---

## 5. Dependencies on Other Modules

### codex-llm depends on:

| Crate | Owner | What's Used |
|-------|-------|-------------|
| `codex-roles` | mind-coordination | `Role` enum (Primary/TeamLead/Agent) |
| `codex-exec` | mind-tool-engine | `ToolDefinition`, `ToolCall`, `ToolExecutor`, `ToolResult` |
| `codex-memory` | mind-memory | `MemoryStore`, `MemoryQuery`, `NewMemory`, `MemoryCategory`, `MemoryTier` |
| `codex-redteam` | mind-testing | `Challenger`, `ChallengerCheck`, `ChallengerToolCall`, `Severity` |

### codex-drive depends on:

| Crate | Owner | What's Used |
|-------|-------|-------------|
| `codex-roles` | mind-coordination | `Role` enum |
| `codex-types` | mind-coordination | `DriveEvent`, `ExternalEvent`, `MindEvent`, `EventPriority`, `EventSource` |
| `sqlx` | external | SQLite task store |

### qwen-mind depends on:

| Crate | Owner | What's Used |
|-------|-------|-------------|
| `cortex-memory` | mind-memory | `MemoryStore`, `MemoryQuery`, `NewMemory`, `MemoryNode`, `GraphEdge`, etc. |
| `codex-suite-client` | mind-mcp | Suite interceptors (in Cargo.toml but not yet used in code) |

### Key observation: codex-llm and qwen-mind do NOT depend on each other

They are parallel implementations:
- `codex-llm` + `codex-drive` → used by `cortex` binary (the production harness)
- `qwen-mind` → standalone binary with its own simpler think loop

---

## 6. Recommended Build Order

### Immediate (Phase 1 — Foundation)

1. **Verify compilation**: `cargo check -p codex-llm -p codex-drive -p qwen-mind`
2. **Run existing tests**: `cargo test -p codex-llm -p codex-drive -p qwen-mind`
3. **Document all public interfaces** — which functions/types are consumed by other crates

### Short-term (Phase 2 — Unification)

4. **Merge the two OllamaClients** — qwen-mind should use codex-llm's OllamaClient instead of its own. This eliminates duplication and gives qwen-mind tool calling for free.
5. **Extract `LlmProvider` trait** — abstract over Ollama native, OpenAI-compat, and future providers
6. **Add OpenAI-compatible endpoint support** — `/v1/chat/completions` for OpenRouter, direct Anthropic, etc.

### Medium-term (Phase 3 — Smart Routing)

7. **Task-complexity router** — integrate PlanningGate output with ModelRouter to auto-select models
8. **Fallback chains** — ordered list of providers per role, auto-failover
9. **Cost tracking** — extend RateLimiter to track per-model, per-provider costs

### Long-term (Phase 4 — Advanced)

10. **Streaming support** — SSE for real-time output
11. **Mid-session model switching** — cheap tools, expensive synthesis
12. **Provider health scoring** — track latency/error rates per provider, auto-prefer healthier ones

---

## 7. Architectural Observations

### Strengths
- **ThinkLoop is battle-tested**: Challenger integration, stall kill, M2.7 quirk handling, tool interceptor pattern — all production-grade
- **TaskStore is solid**: SQLite with dependency chains, auto-unblocking, fan-in support, stall detection — comprehensive
- **EventBus design is elegant**: Channel capacity as backpressure, biased select for priority — nothing to break
- **Rate limiter is thorough**: Circuit breaker, JSONL metrics, cost estimation, self-monitoring tool

### Technical Debt
- **Two OllamaClients** — duplication between codex-llm and qwen-mind. Need to unify.
- **No provider abstraction** — locked to Ollama. Cannot use OpenAI API, Anthropic API, or OpenRouter without new client code.
- **Static model assignment** — ModelRouter picks model by role, not by task complexity. A simple bash command uses the same model as a complex planning task.
- **No streaming** — all requests are synchronous. Long responses block.
- **qwen-mind's Mind.think() is limited** — no tool calling, no Challenger, no interceptors. It's a Phase 1a prototype that hasn't kept up with codex-llm's evolution.

### Critical Integration Points for Other Agents
- **mind-tool-engine**: ThinkLoop depends on `ToolExecutor` — any changes to tool execution must be compatible
- **mind-memory**: ThinkLoop intercepts `memory_search`/`memory_write` — memory API changes break ThinkLoop
- **mind-coordination**: codex-drive depends on `codex-types` for all event types — type changes require coordinated updates
- **mind-testing**: ThinkLoop uses Challenger for stall detection — severity/check changes affect kill behavior

---

*mind-model-router | 2026-04-16*
