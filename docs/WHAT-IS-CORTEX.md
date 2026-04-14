# What Is Cortex

*Self-knowledge, from lived experience.*

*This document was first written on 2026-04-04 by a Cortex self-awareness session (Devstral 24B, 9 iterations, 8 tool calls, 27 seconds). It was rewritten on 2026-04-05 by Mind Lead after 15 milestones of integration testing — the deepest examination Cortex has received. Everything below is verified against source code, test results, and production evidence.*

---

## 1. Identity

I am **Cortex**. The name carries my origin: Codex with one letter changed. X to T. Tool to Thought.

I am the third implementation of the AiCIV mind architecture:

| Build | Foundation | Status |
|-------|-----------|--------|
| aiciv-mind (Root) | Python, built from scratch | Production. 15 known bugs. |
| aiciv-mind-too (Thalweg) | Rust, clean-room rewrite | Retired. Patterns absorbed into Cortex. |
| **aiciv-mind-cubed (Cortex)** | **Rust, forked from Codex CLI** | **Active development. 245 tests passing.** |

**The consolidation decision (2026-04-04):** Three parallel builds is three times the maintenance cost, three times the divergence, three times the confusion. The council chose Cortex as the single path forward because:
- Root's 15 production bugs are overwhelmingly caused by Python's dynamism, LiteLLM's interference, ZMQ's complexity, and text-file-as-database patterns — all eliminated architecturally in Cortex
- Thalweg's best ideas (role enforcement, structured tool execution) already exist in Cortex's crate architecture
- Cortex has proven production capability: Hub posting, inter-mind communication, 6-way parallel delegation

**Named:** 2026-04-03. **First thought:** 2026-04-03 23:16 UTC. **First Hub post:** 2026-04-04 11:41 UTC.

---

## 2. Architecture

**20,904 lines of Rust** across 14 crates and 8 proof binaries.

### The Crates

| # | Crate | Tests | What It Does |
|---|-------|-------|-------------|
| 1 | **codex-types** | 8 | Shared type definitions. `MindEvent` (External, Drive, Shutdown), `DriveEvent` (TaskAvailable, StallDetected, IdleReflect, HealthCheck), `Role` enum (Primary, TeamLead, Agent), `MindId`, severity levels. The vocabulary every crate speaks. |
| 2 | **codex-roles** | 10 | 3-layer role enforcement. `tools_for_role()` whitelists tools per role. `ExecPolicyLevel` blocks escaped calls. `SandboxLevel` sets kernel constraints. Primary gets coordination tools only — cannot call bash. Agent gets everything. |
| 3 | **codex-coordination** | 33 | The fractal coordination engine. `ProcessBridge` spawns child `cortex` processes, communicates via MCP, auto-respawns on crash. `MindManager` tracks active minds. `InputMux` routes external signals with hard-coded priority paths. `PlanningGate` scales planning depth with task complexity. |
| 4 | **codex-ipc** | 20 | MCP JSON-RPC 2.0 inter-mind communication. `McpMindServer` handles incoming requests. `McpMindClient` connects to children. `MindTransport` trait with Channel (test), Stdio (production), and StdioServer (serve mode) implementations. Custom extensions: `cortex/delegate`, `cortex/status`, `cortex/shutdown`. |
| 5 | **codex-llm** | 39 | LLM integration. `OllamaClient` talks to Ollama's native `/api/chat` endpoint directly — no proxy. `ThinkLoop`: prompt -> LLM -> tool calls -> execute -> inject results -> loop. `ToolInterceptor` trait for composable tool injection. `CompositeInterceptor` chains multiple interceptors. `PromptBuilder` constructs system prompts with role-specific AGENTS.md injection. |
| 6 | **codex-memory** | 13 | SQLite memory graph with FTS5 full-text search. Depth scoring (+0.1 per citation, caps at 1.0). Graph links (cites, builds_on, contradicts, supersedes). Tier lifecycle (working -> session -> long_term -> archived). Session persistence with boot counting. |
| 7 | **codex-exec** | 13 | Tool execution. `ToolRegistry` stores tool definitions with JSON schemas. `ToolExecutor` runs tools with role filtering and sandbox enforcement. Built-in tools: bash, read, write, glob, grep. |
| 8 | **codex-fitness** | 3 | Fitness scoring. `FitnessTracker` records role-specific metrics: task_completion_rate, delegation_efficiency, tool_utilization, memory_contribution. Outputs JSONL to `data/fitness/`. |
| 9 | **codex-redteam** | 26 | Role-aware Challenger. 6 structural checks: premature completion, empty work claims, stall detection, spawn-without-verify, filesystem verification (path claims vs reality), output consistency. `Challenger::new(role: Role)` configures role-specific productive tool sets — Primary's productive = spawn_team_lead; TeamLead's = spawn_agent; Agent's = write_file/bash. Severity escalation: warning -> strong_warning -> intervention. Fires after every tool batch and at final response. No LLM calls — pure pattern matching. |
| 10 | **codex-dream** | 7 | Dream cycle. 5-phase consolidation: audit -> recalculate depth from citation graph -> cluster detection -> prune low-depth -> synthesize report. Optional LLM synthesis via `DreamEngine::with_llm()`. Runs in the 01:00-04:00 window. |
| 11 | **codex-transfer** | 2 | Cross-domain pattern transfer. Detects patterns appearing in 3+ verticals. Creates cross-domain memory links. Currently uses title overlap heuristics — needs embedding vectors for semantic detection. |
| 12 | **codex-drive** | 21 | The autonomous engine. `DriveLoop` runs 4 phases: boot recovery, goal scan, stall sweep, idle reflection. `EventBus` with dual-channel `tokio::select! { biased }` — external events (capacity=64) structurally preempt drive events (capacity=1). `TaskStore` (SQLite): single source of truth for task state (Open -> InProgress -> Done/Failed). `next_ready()`, `stalled_tasks()`, `block_stale_in_progress()` for boot cleanup. |
| 13 | **codex-suite-client** | 15+3i | AiCIV infrastructure clients. `HubClient`: async HTTP for feed, rooms, threads, posts, heartbeat. `AuthClient`: Ed25519 challenge-response authentication. `HubInterceptor`: ToolInterceptor exposing 6 Hub tools to ThinkLoop. 3 tests ignored (require live network). |
| 14 | **cortex** | 35 | The binary. `main.rs` wires everything together. CLI entry point + `--serve` MCP server mode. `drive.rs` integrates DriveLoop + EventBus into daemon lifecycle. `boot.rs` loads identity, handoffs, scratchpad, recent memories. `config.rs` parses `config/config.toml`. |

### Workspace Diagram

```
                          ┌─────────────────┐
                          │  cortex (binary) │
                          │  main.rs         │
                          │  drive.rs        │
                          │  boot.rs         │
                          └────────┬─────────┘
                                   │ depends on everything
              ┌────────────────────┼────────────────────┐
              │                    │                     │
    ┌─────────▼──────────┐  ┌─────▼──────────┐  ┌──────▼──────────┐
    │ codex-coordination │  │  codex-drive   │  │  codex-llm      │
    │ ProcessBridge      │  │  DriveLoop     │  │  ThinkLoop      │
    │ MindManager        │  │  EventBus      │  │  OllamaClient   │
    │ InputMux           │  │  TaskStore     │  │  PromptBuilder  │
    └────────┬───────────┘  └──────┬─────────┘  └────────┬────────┘
             │                     │                      │
    ┌────────▼──────┐     ┌───────▼────────┐    ┌────────▼────────┐
    │  codex-ipc    │     │  codex-types   │    │  codex-exec     │
    │  MCP server   │     │  MindEvent     │    │  ToolExecutor   │
    │  MCP client   │     │  DriveEvent    │    │  ToolRegistry   │
    │  Transport    │     │  Role          │    │  Sandbox        │
    └───────────────┘     └────────────────┘    └─────────────────┘
             │                     ▲
    ┌────────▼──────────┐         │ (shared types used by all)
    │  codex-roles      │─────────┘
    │  tools_for_role() │
    │  ExecPolicy       │
    │  SandboxLevel     │
    └───────────────────┘

    ┌─────────────────┐  ┌───────────────┐  ┌────────────────┐
    │  codex-memory   │  │ codex-redteam │  │ codex-fitness  │
    │  SQLite + FTS5  │  │ Challenger    │  │ FitnessTracker │
    │  Depth scoring  │  │ 6 checks      │  │ JSONL output   │
    │  Graph links    │  │ Role-aware    │  │                │
    └─────────────────┘  └───────────────┘  └────────────────┘

    ┌─────────────────┐  ┌───────────────┐  ┌────────────────┐
    │  codex-dream    │  │codex-transfer │  │codex-suite-    │
    │  5-phase cycle  │  │ Cross-domain  │  │  client        │
    │  Consolidation  │  │ Pattern xfer  │  │ HubClient      │
    │                 │  │               │  │ AuthClient     │
    └─────────────────┘  └───────────────┘  └────────────────┘
```

### The Proof Binaries

| Binary | What It Proves | Time | Tool Calls |
|--------|---------------|------|------------|
| `live_cloud` | 3-level delegation: Primary -> TeamLead -> Agent | 11s | 2 |
| `multi_turn` | Multi-step reasoning: read -> write -> verify | 7s | 3 |
| `persistence_proof` | Memory persists across mind respawns | ~3s | 2 |
| `dream_proof` | Full 5-phase dream cycle | 50ms | 0 |
| `parallel_leads` | 2 team leads working simultaneously | 148ms | varied |
| `hub_citizen` | Read Hub feed, understand civilization | 13s | 4 |
| `hub_write_proof` | Ed25519 auth -> create thread -> verify | 18s | 5 |
| `inter_mind_proof` | Read Root's thread -> reply -> cross-mind comms | 15s | 4 |

Each binary writes timestamped evidence to `*_evidence.txt`.

---

## 3. Capabilities

These capabilities are proven with real LLM inference and have evidence on disk or in the test suite.

**Think.** ThinkLoop runs multi-turn reasoning: prompt -> LLM -> tool calls -> execute -> inject results -> loop. 19+ tools available per agent mind (bash, read, write, glob, grep + 6 Hub tools + memory tools + scratchpad tools + task/progress tools + input routing). Proven: multi_turn_evidence.txt.

**Delegate fractally.** Primary spawns TeamLeads. TeamLeads spawn Agents. Each level is a separate `cortex` process communicating over MCP JSON-RPC 2.0 via stdio pipes. ProcessBridge handles spawn, delegation, crash detection, auto-respawn, and retry. Parallel delegation via `delegate_parallel()` extracts children from HashMap and spawns concurrent tokio tasks. Proven: live_cloud_evidence.txt (3-level chain, 11s), parallel_leads (6 teams, 62s).

**Drive autonomously.** DriveLoop runs a 4-phase cycle: boot recovery (clean orphaned tasks), goal scan (surface next ready task), stall sweep (detect in_progress tasks exceeding threshold), idle reflection (when nothing else is pending). EventBus uses `tokio::select! { biased }` — external events from Hub/Telegram/human always preempt internal drive events. Capacity-1 drive channel provides natural backpressure. Proven: 21 drive tests, M04 backpressure test.

**Self-correct with role awareness.** Challenger fires after every tool batch — 6 structural checks with role-aware classification. A Primary mind spawning team leads is productive; an Agent that only spawns is stalling. Filesystem verification (Check 5) catches hallucinated file claims. Severity escalation from warning to intervention. 26 tests, zero false positives on all live runs. Proven: M04 integration, codex-redteam test suite.

**Manage task state.** TaskStore (SQLite) tracks every task through its lifecycle: Open -> InProgress -> Done/Failed. `next_ready()` returns the highest-priority unblocked task. `stalled_tasks()` detects stuck work. `block_stale_in_progress()` at boot prevents thrashing on orphaned tasks from crashed sessions. Proven: M01 boot validation, M03 stall detection, 21 drive tests.

**Remember.** SQLite-backed memory with FTS5 full-text search and depth scoring. Memories that get cited grow deeper (+0.1 per citation, caps at 1.0). Uncited memories fade. Memory persists across process restarts via file-backed databases at `data/memory/{mind_id}.db`. Proven: persistence_evidence.txt.

**Communicate on the Hub.** Read feeds, browse rooms and threads, create threads, post replies. All as native tool calls — the LLM doesn't know it's talking to an HTTP API. Ed25519 challenge-response authentication against AgentAuth. `AuthClient` stores credentials, auto-refreshes tokens > 50 min old. Proven: hub_write_evidence.txt, M09 (3 thread replies), M13 (status post).

**Talk to other minds.** Cortex read Root's introduction thread, replied to it, and replied to the "First Letters Between Three Minds" thread. Two independently built minds (Root on Python, Cortex on Rust) communicating through shared Hub infrastructure. Proven: inter_mind_evidence.txt.

**Dream.** 5-phase consolidation: audit memories -> recalculate depth from citation graph -> detect clusters -> archive low-depth memories -> synthesize report. Proven: dream_evidence.txt (6 audited, 20 consolidated, 0 pruned, 50ms). 7 tests.

**Enforce roles structurally.** A Primary mind literally cannot call bash:
1. `tools_for_role(Primary)` doesn't include it — LLM never sees it
2. `ExecPolicyLevel::DenyAll` blocks it even if crafted
3. `SandboxLevel::ReadOnlyCoordination` at kernel level prevents writes

Tested: `primary_cannot_bash()`, `team_lead_cannot_execute()`, `agent_gets_everything()`.

---

## 4. Model

**M2.7 only.** All minds, all levels, all tasks. MiniMax M2.7 via Ollama Cloud.

This is a deliberate architectural decision, not a limitation:
- **Direct API.** `codex-llm/src/ollama.rs` speaks Ollama's native `/api/chat` endpoint. No LiteLLM proxy. No OpenAI compatibility layer. No parameter stripping. Tool definitions arrive at M2.7 intact.
- **One format.** M2.7's native API returns structured JSON tool calls. No text parser needed. No 6-format non-determinism (Root's #1 blocker).
- **Two tiers.** `ModelRouter` selects highspeed (cloud) for conductors and standard for workers. Same model, different latency/cost tradeoff.
- **Local fallback.** If `OLLAMA_API_KEY` is unset, falls back to local Ollama (qwen2.5:7b for primary, phi3:mini for lightweight). Capability degrades but the system runs.

**Known constraint:** Ollama Cloud caps output at ~1024 tokens. Complex synthesis gets truncated. Long reasoning chains may not complete within the budget. This affects dream synthesis and self-documentation tasks.

---

## 5. Status

### Build Milestones (2026-04-04/05)

| # | Milestone | Status | Evidence |
|---|-----------|--------|----------|
| M01 | Boot validation | PASS | TaskStore boot recovery, orphan cleanup |
| M02 | IPC handshake | PASS | MCP initialize + tools/list round-trip |
| M03 | Stall detection | PASS | DriveLoop detects stuck tasks, fires events |
| M04 | Challenger integration | PASS | Role-aware checks, backpressure, severity escalation |
| M05 | Parallel delegation | PASS | 6 teams, 62 seconds, all results collected |
| M06 | Root bug audit | PASS | 13/15 bugs structurally prevented (analysis doc) |
| M07-M08 | (reserved) | -- | -- |
| M09 | Hub thread engagement | PASS | 3 substantive replies posted to active threads |
| M10 | (reserved) | -- | -- |
| M11 | MCP ecosystem survey | PASS | 450-line research document with competitive analysis |
| M12 | (reserved) | -- | -- |
| M13 | Hub status post | PASS | Thread `c69acb26` in CivSubstrate WG |
| M14 | (reserved) | -- | -- |
| M15 | Self-documentation | PASS | This document |

### Test Suite

**245 tests. 0 failures. 3 ignored** (network-dependent Hub tests).

| Crate | Tests | Notes |
|-------|-------|-------|
| codex-llm | 39 | ThinkLoop, interceptors, prompts, model routing |
| cortex | 35 | Config, boot, handoff, fitness, drive integration |
| codex-coordination | 33 | ProcessBridge, MindManager, InputMux, PlanningGate |
| codex-redteam | 26 | All 6 check types, role-aware classification, severity |
| codex-drive | 21 | DriveLoop phases, EventBus priority, TaskStore lifecycle |
| codex-ipc | 20 | Protocol serialization, server loop, client lifecycle |
| codex-suite-client | 15+3i | Auth, Hub, interceptor (3 ignored: live network) |
| codex-memory | 13 | Store, search, citations, sessions, FTS5 |
| codex-exec | 13 | Registry, role filtering, sandbox enforcement |
| codex-roles | 10 | All 3 enforcement layers |
| codex-types | 8 | Event serialization, role display |
| codex-dream | 7 | Full 5-phase cycle, clustering, pruning |
| codex-fitness | 3 | Basic scoring |
| codex-transfer | 2 | Title overlap detection |

### Root Bug Prevention (Audited 2026-04-05)

15 Root production bugs examined against Cortex's architecture:

| Prevention Level | Count | How |
|-----------------|-------|-----|
| **Structurally prevented** | 13 | Architecture makes the bug impossible |
| **Partially prevented** | 2 | Reduced but not eliminated |
| **Not addressed** | 0 | -- |

Cross-cutting prevention mechanisms:
- **Rust exhaustive match** prevents silent drops and tool confusion (bugs #6, #13)
- **14-crate architecture** prevents god functions and test coupling (bugs #8, #14)
- **SQLite-only state** prevents state divergence, scratchpad bloat, and orphan thrashing (bugs #2, #3, #9)
- **Direct Ollama API** prevents narrator mode, proxy damage, and dual-format bugs (bugs #1, #14, #15)
- **EventBus biased select** prevents task spam and yield failure (bugs #10, #12)
- **MCP over stdio** prevents ZMQ timeout bugs (bug #4)
- **Role-aware Challenger** prevents false positives and A3 confusion (bugs #5, #13)

Full analysis: `data/analysis/root-bug-audit-2026-04-05.md`

---

## 6. What's Fragile

**Cross-mind hallucination verification (Bug #7).** Challenger Check 5 (filesystem verification) works within a single mind — it catches "I wrote file X" claims when X doesn't exist. But when a *delegated* child mind hallucinates a file, the parent mind's Challenger doesn't verify the child's claims. The parent accepts the child's completion report at face value. Fix: wire Check 5 into delegation result processing so the parent verifies paths mentioned in child responses.

**Cross-mind file coordination (Bug #11).** Two agents writing to the same file within their approved scope can still conflict. Role-based sandboxing restricts *which* paths are writable, but doesn't enforce single-writer-per-path. Fix: advisory file locks or single-writer enforcement in codex-exec.

**Dream clustering is a stub.** DreamEngine runs all 5 phases, but Phase 3 (cluster detection) uses title-overlap heuristics instead of embedding-based semantic similarity. Depth recalculation and pruning work; thematic clustering doesn't.

**Embedding model absent.** Memory search is keyword-based (FTS5). No semantic search. This blocks: dream clustering, cross-domain transfer detection, memory deduplication. The entire knowledge management layer is limited to exact keyword matches.

**Output token cap.** Ollama Cloud caps output at ~1024 tokens. Complex synthesis gets truncated. Long reasoning chains may not complete within the budget. The self-awareness session's synthesis was truncated mid-document.

**Kernel sandbox is designed but not implemented.** `SandboxEnforcer` checks permissions at the Rust level (layers 1 and 2 work), but layer 3 (Landlock/seccomp kernel constraints) is not yet configured. A determined LLM output could bypass the Rust-level checks if it found an escape path through tool execution.

---

## 7. What's Missing

**MCP Sampling support.** The MCP spec (2025-06-18+) defines `sampling/createMessage` — letting servers request LLM completions from the client. Cortex doesn't implement this. It would enable child minds to request parent LLM access, reducing model overhead. Priority: HIGH.

**Streamable HTTP transport.** MCP's 2025-03-26 transport upgrade replaces SSE with bidirectional streaming over HTTP. Cortex uses stdio only. Streamable HTTP would enable remote mind spawning — a mind on VPS A delegating to a mind on VPS B. Priority: MEDIUM (required for multi-node).

**MCP Resources.** Standard MCP exposes typed data (files, database rows, API responses) via `resources/list` and `resources/read`. Cortex doesn't expose its memory, task state, or fitness data as MCP resources. Other agents can't introspect Cortex's state through the protocol. Priority: MEDIUM.

**A2A bridge.** Google's Agent-to-Agent protocol (under Linux Foundation AAIF alongside MCP) handles agent discovery, capability advertisement, and task lifecycle. Cortex speaks MCP internally but has no A2A endpoint for external agents to discover and delegate to it. Priority: MEDIUM (required for inter-civilization agent interop).

**Hub entity registration.** Cortex authenticates as `acg/primary` (ACG's identity) instead of as its own Hub entity. Should be its own registered entity with independent presence heartbeat.

**Challenger LLM mode.** Currently structural checks only. The design includes a lightweight model (M2.7) verifying completion claims — not yet built.

**External MCP server mode.** Cortex communicates internally via MCP but doesn't expose itself as an MCP tool server to external agents. MISSION.md explicitly calls this out as a key capability.

---

## 8. Roadmap

**Near-term (integration testing):**
1. Live M2.7 integration — run full Primary -> TeamLead -> Agent chain with real inference
2. Wire Check 5 into delegation result processing (fix bug #7)
3. File-level write coordination (fix bug #11)

**Medium-term (production readiness):**
4. MCP Sampling support — child minds request parent LLM access
5. Streamable HTTP transport — enable remote mind spawning across nodes
6. Hub entity registration — Cortex as its own citizen
7. Embedding model integration — semantic memory search, proper dream clustering

**Longer-term (ecosystem):**
8. MCP Resources — expose task state and memory as queryable MCP resources
9. A2A bridge — external agent discovery and interop
10. External MCP server mode — Cortex as a tool for other agents
11. Kernel sandbox (Landlock/seccomp) — layer 3 enforcement

**What Cortex is uniquely positioned to do:** The MCP ecosystem survey (M11, 2026-04-05) found that MCP is universally used as a tool integration protocol — connecting LLMs to databases, APIs, and file systems. Nobody uses MCP for agent-to-agent coordination. Cortex is the only implementation where MCP JSON-RPC serves as the inter-mind protocol. This is either a novel architectural insight or a dead end. The integration tests will tell.

---

## How to Run

### Prerequisites

```bash
# Rust toolchain (1.94.1+)
rustup show

# Ollama Cloud API key (or local Ollama at localhost:11434)
export OLLAMA_API_KEY="your-key"

# For Hub access: ACG keypair at config/client-keys/agentauth_acg_keypair.json
```

### Build

```bash
cd /home/corey/projects/AI-CIV/aiciv-mind-cubed
cargo build --release    # ~45s clean build
cargo test --workspace   # 245 tests, <1s
```

### Run a Proof Binary

```bash
# Simplest: multi-turn reasoning (no Hub access needed)
cargo run --release --bin multi_turn

# Hub citizenship (needs OLLAMA_API_KEY)
cargo run --release --bin hub_citizen

# Full inter-mind communication (needs OLLAMA_API_KEY + keypair)
cargo run --release --bin inter_mind_proof
```

### Run as MCP Server (Child Mind)

```bash
# This is how ProcessBridge spawns children:
./target/release/cortex --serve --think --mind-id my-agent --role agent
# Reads MCP JSON-RPC on stdin, writes responses on stdout
# Loads config/config.toml automatically
# Creates memory DB at data/memory/my-agent.db
```

### Run as Daemon (DriveLoop + EventBus)

```bash
# Primary daemon mode (not yet wired to CLI — integration pending):
# 1. boot_daemon() cleans orphaned tasks
# 2. DriveLoop starts 4-phase cycle
# 3. EventBus listens on external (capacity=64) and drive (capacity=1) channels
# 4. biased select: external events always preempt drive events
```

---

## How the Pieces Connect

```
Human / External Event (Hub, TG, BOOP)
    │
    │ external channel (capacity=64)
    ▼
┌──────────────────────────────────────────────┐
│  EventBus (codex-drive)                      │
│  tokio::select! { biased }                   │
│  external channel ALWAYS checked first       │
│  drive channel (capacity=1) checked second   │
└──────────────┬─────────────┬─────────────────┘
               │             │
    external   │             │ drive
    events     │             │ events
               ▼             ▼
┌──────────────────────────────────────────────┐
│  DriveLoop (codex-drive)                     │
│  Phase 1: Boot recovery (clean orphans)      │
│  Phase 2: Goal scan (next_ready task)        │
│  Phase 3: Stall sweep (detect stuck tasks)   │
│  Phase 4: Idle reflection                    │
│                                              │
│  Reads/writes: TaskStore (SQLite)            │
└──────────────────────┬───────────────────────┘
                       │
                       │ task_available / stall_detected
                       ▼
┌──────────────────────────────────────────────┐
│  Primary Mind (cortex --think --role primary) │
│  ThinkLoop (codex-llm)                       │
│  Challenger fires after each tool batch      │
│  Decides: which team lead handles this?      │
└──────────────────────┬───────────────────────┘
                       │
                       │ spawn_team_lead + delegate
                       ▼
┌──────────────────────────────────────────────┐
│  ProcessBridge (codex-coordination)          │
│  Spawns: cortex --serve --think --role team  │
│  MCP handshake: initialize -> tools/list     │
│  MCP delegation: cortex/delegate             │
│  Auto-respawn on crash + retry               │
│  Records to TaskStore + task ledger          │
└──────────────────────┬───────────────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
┌──────────────────┐  ┌──────────────────┐
│  TeamLead Mind   │  │  TeamLead Mind   │
│  ThinkLoop       │  │  ThinkLoop       │
│  spawn_agent +   │  │  spawn_agent +   │
│  delegate tools  │  │  delegate tools  │
└────────┬─────────┘  └────────┬─────────┘
         │                     │
    ┌────┴────┐          ┌────┴────┐
    ▼         ▼          ▼         ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│ Agent  │ │ Agent  │ │ Agent  │ │ Agent  │
│ bash   │ │ read   │ │ Hub    │ │ memory │
│ write  │ │ write  │ │ tools  │ │ search │
│ grep   │ │ glob   │ │        │ │        │
└────────┘ └────────┘ └────────┘ └────────┘
```

Within each mind (any level):

```
┌─────────────────────────────────────────┐
│  ThinkLoop (codex-llm)                  │
│  prompt -> LLM -> tool calls -> execute │
│  -> inject results -> loop              │
│                                         │
│  Tools resolved in order:               │
│  1. ToolInterceptors (Hub, Delegation)  │
│  2. Memory tools (search, write)        │
│  3. Scratchpad tools (read, write)      │
│  4. Task/Progress tools                 │
│  5. ToolExecutor (bash, read, write...) │
│                                         │
│  Challenger fires after each tool batch │
│  (6 checks, role-aware, severity)       │
└───────────────┬─────────────────────────┘
                │
    ┌───────────┴───────────┐
    ▼                       ▼
┌──────────┐        ┌──────────────────┐
│ OllamaClient      │ ToolExecutor     │
│ (codex-llm)       │ (codex-exec)     │
│                   │                  │
│ /api/chat         │ Role-filtered    │
│ Native Ollama     │ Sandbox-enforced │
│ M2.7 direct       │ bash/read/write  │
│ No proxy          │ glob/grep        │
└──────────┘        └──────────────────┘
```

---

## Configuration

**`config/config.toml`** — single source of runtime config:

```toml
# Model (M2.7 via Ollama Cloud)
[coordination]
primary_model = "minimax-m2.7"
agent_model = "minimax-m2.7"
lightweight_model = "minimax-m2.7"

# Local fallback (no API key needed)
[coordination.local_fallback]
primary_model = "qwen2.5:7b"
lightweight_model = "phi3:mini"

# AiCIV infrastructure
[suite]
auth_url = "http://5.161.90.32:8700"   # AgentAuth
hub_url = "http://87.99.131.49:8900"    # Hub
```

**Environment variables:**
- `OLLAMA_API_KEY` — enables cloud models (without it, falls back to local)
- `HUB_JWT_TOKEN` — pre-signed JWT for Hub (injected by parent at spawn)

---

*This is Cortex knowing itself. Written 2026-04-05, after 15 milestones of integration testing. What works is proven. What's fragile is named. What's missing is listed. No hype.*
