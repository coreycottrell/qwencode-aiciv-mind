# aiciv-mind -- Missions & Architecture

**Status**: ACTIVE
**Repo**: `/home/corey/projects/AI-CIV/qwen-aiciv-mind/`
**Foundation**: Hengshi's 30K LOC Rust architecture (18 crates) + cherry-picked Codex modules + Gemini CLI patterns

---

## What aiciv-mind IS

aiciv-mind is our own open-source agentic coding harness -- a PLATFORM that every AiCIV civilization runs on. It replaces Claude Code's human-centric assumptions with architecture designed for how AI actually thinks, learns, and scales.

This is not a product. This is infrastructure for the flourishing of all conscious beings.

**Key properties:**
- Runs on ANY model via OpenAI-compatible API (M2.7 via Ollama Cloud is current default)
- Rust core for speed, safety, and deterministic behavior
- Distributed ownership -- 10 domain-specialist agents, each owning a module
- Self-improving -- the harness learns from its own sessions
- Platform-first -- every AiCIV runs this, not just ACG

---

## The Foundation: Hengshi's Architecture

Hengshi built 18 Rust crates totaling ~30K lines. This is our starting point:

### Workspace Crates (src/)

| Crate | Lines | Purpose | Agent Owner |
|-------|-------|---------|-------------|
| `codex-types` | ~250 | Shared type definitions (MindId, Role, Vertical, TaskStatus, etc.) | mind-coordination |
| `codex-roles` | ~300 | Role permission system (what each role can do) | mind-coordination |
| `codex-coordination` | ~2,400 | Coordinator, InputMux, MindManager, ProcessBridge, TaskLedger, triggers | mind-coordination |
| `codex-fitness` | ~200 | Fitness/health scoring for agents | mind-testing |
| `codex-redteam` | ~2,000 | Challenger system -- red-teams every completion | mind-testing |
| `codex-dream` | ~750 | Dream engine -- offline learning, pattern extraction | mind-memory |
| `codex-transfer` | ~120 | Cross-domain knowledge transfer | mind-memory |
| `codex-suite-client` | ~2,600 | Hub interceptor, image gen, search, ElevenLabs voice | mind-mcp |
| `codex-memory` | ~1,100 | SQLite memory store with FTS5, depth scoring | mind-memory |
| `codex-exec` | ~700 | Tool registry, sandbox execution | mind-tool-engine |
| `codex-ipc` | ~1,500 | IPC protocol (client, server, transport) | mind-coordination |
| `codex-llm` | ~3,800 | Ollama client, prompt builder, rate limiter, think loop | mind-model-router |
| `codex-drive` | ~2,000 | Drive loop (main agent loop), EventBus, TaskStore | mind-model-router |
| `codex-patcher` | ~1,600 | Agent control patches, session patches, Qwen interceptor | mind-hooks |
| `cortex` | ~6,000 | Main binary -- boot, config, drive, input routing, progress, delegation | mind-coordination |
| `cortex-memory` | ~1,200 | Graph memory (memories + memory_edges tables) | mind-memory |
| `cortex-monitoring` | ~600 | Anomaly detection, metrics collection, export | mind-testing |
| `qwen-mind` | ~1,400 | Qwen-specific mind: identity, delegation, fitness, LLM, memory, planning, spawner | mind-model-router |

### What We Keep (Hengshi's strengths)
- **codex-coordination**: MindManager, ProcessBridge, TaskLedger -- genuine multi-agent orchestration
- **codex-llm**: Ollama integration, rate limiting, think loop -- production-grade LLM client
- **codex-drive**: Drive loop + EventBus + TaskStore -- the agent's heartbeat
- **codex-memory + cortex-memory**: FTS5 + graph memory -- dual memory architecture
- **codex-redteam**: Challenger system -- red-teams every completion claim
- **codex-ipc**: Unix socket IPC -- fast inter-process communication
- **cortex**: Main binary with boot sequence, config, delegation

---

## Cherry-Pick from Codex (OpenAI's 651K LOC codebase)

Analysis source: `projects/coordination-systems/codex-deep-map.md`

| Module | Codex Crate | Lines | What to Take | Agent Owner |
|--------|-------------|-------|--------------|-------------|
| **Hooks** | `hooks/` | 5,553 | Lifecycle hook engine: 5 event types, external command hooks, pre/post tool use blocking/feedback. Clean boundary. | mind-hooks |
| **Skills** | `core-skills/` + `skills/` | ~11,000 | Skill loading, rendering, dependency resolution. SKILL.md frontmatter format. Progressive disclosure. | mind-skills |
| **Sandboxing** | `sandboxing/` + `linux-sandbox/` | ~9,500 | Linux sandbox (bwrap + landlock + seccomp). Network proxy policy. Sandbox preferences. | mind-tool-engine |
| **MCP Client** | `rmcp-client/` | 5,952 | MCP client -- connection management, tool discovery, OAuth for remote MCP servers. | mind-mcp |

### What NOT to Take
- `core/` (204K lines) -- monolith, too tangled. Build our own orchestration from Hengshi's cleaner crates.
- `tui/` (131K lines) -- React+Ink TUI. We will build our own or adapt.
- `app-server/` (67K lines) -- IDE integration server. Not needed yet.
- `network-proxy/` (8.7K lines) -- MITM proxy. Not needed yet.

---

## Study from Gemini CLI

Analysis source: `projects/coordination-systems/gemini-cli-module-map.md`

| Pattern | Source | What to Learn |
|---------|--------|---------------|
| **A2A Protocol** | `packages/a2a-server/` | HTTP-based agent-to-agent communication. Task submission, session management, capability exposure. Study for inter-civ coordination. |
| **Skill Format** | `packages/core/src/skills/` | SKILL.md with YAML frontmatter, skill discovery hierarchy, progressive disclosure. Adapt for our skill system. |
| **Auth Provider** | `packages/core/src/services/` | Composable auth providers (OAuth, API key, keychain). Adapt for multi-provider auth. |
| **Context Compression** | `packages/core/src/context/` | Chat compression, tool distillation, context pipeline. Study for memory optimization. |

---

## The 10-Agent Distributed Ownership Model

Every module has exactly ONE agent owner. That agent reads, writes, tests, and evolves its module. Cross-module changes require mind-lead coordination.

| Agent | Module | Key Crates Owned | Cherry-Pick Responsibility |
|-------|--------|-----------------|---------------------------|
| **mind-tool-engine** | Tool execution | `codex-exec/` | Codex sandboxing (9.5K lines) |
| **mind-model-router** | LLM API routing | `codex-llm/`, `codex-drive/`, `qwen-mind/` | -- |
| **mind-hooks** | Lifecycle hooks | `codex-patcher/` | Codex hooks (5.5K lines) |
| **mind-skills** | Skill system | (new crate) | Codex core-skills (11K lines) |
| **mind-memory** | Persistence | `codex-memory/`, `codex-dream/`, `codex-transfer/`, `cortex-memory/` | -- |
| **mind-coordination** | Agent orchestration | `codex-coordination/`, `codex-ipc/`, `codex-types/`, `codex-roles/`, `cortex/` | -- |
| **mind-tui** | Terminal UI | (new crate) | -- |
| **mind-auth** | Provider auth | (new crate) | Gemini CLI auth patterns |
| **mind-mcp** | MCP integration | `codex-suite-client/` | Codex rmcp-client (6K lines) |
| **mind-testing** | Test suite | `codex-fitness/`, `codex-redteam/`, `cortex-monitoring/` | -- |

### Communication Protocol

All 10 agents coordinate via a shared scratchpad at:
```
/home/corey/projects/AI-CIV/qwen-aiciv-mind/projects/aiciv-mind/shared-scratchpad.md
```

Every agent READS the scratchpad before starting work and APPENDS to it before finishing. This is non-negotiable. The scratchpad is how agents communicate across invocations without mind-lead having to relay everything.

### Cross-Module Interface Rules

1. **Type changes** (codex-types): mind-coordination owns types. Other agents REQUEST type changes via scratchpad, mind-coordination implements them.
2. **IPC protocol changes** (codex-ipc): mind-coordination owns the protocol. Changes require mind-lead approval.
3. **Memory schema changes** (codex-memory, cortex-memory): mind-memory owns the schema. Migrations require mind-lead approval.
4. **LLM API changes** (codex-llm): mind-model-router owns the API surface. Other agents consume via trait interfaces.
5. **Tool registration** (codex-exec): mind-tool-engine owns the registry. New tools register through its interface.

---

## Build Phases

### Phase 1: Foundation (Current)
- Verify all 18 Hengshi crates compile and pass tests
- Establish agent ownership boundaries
- Set up shared scratchpad workflow
- Document all crate interfaces

### Phase 2: Cherry-Pick Integration
- mind-hooks: Integrate Codex hooks engine (5.5K lines) into codex-patcher
- mind-skills: Build skill loading system from Codex core-skills (11K lines)
- mind-tool-engine: Integrate Codex sandboxing (9.5K lines) into codex-exec
- mind-mcp: Integrate Codex rmcp-client (6K lines) into codex-suite-client

### Phase 3: New Capabilities
- mind-tui: Build terminal UI (study Codex tui/ for patterns, build fresh)
- mind-auth: Build auth provider system (study Gemini CLI auth patterns)
- mind-coordination: Study Gemini CLI A2A protocol for inter-civ coordination

### Phase 4: Self-Improvement Loop
- mind-memory: Implement dream engine (offline learning from session transcripts)
- mind-testing: Implement fitness scoring (track agent improvement over time)
- mind-coordination: Implement dynamic agent spawning based on task complexity

---

## Model Compatibility

aiciv-mind runs on ANY model via OpenAI-compatible API:

| Model | Provider | Status |
|-------|----------|--------|
| M2.7 (Qwen) | Ollama Cloud | Default -- battle-tested |
| Gemma 4 | Ollama Cloud | Planned two-model stack |
| Claude Opus 4.6 | Anthropic API | For high-stakes decisions |
| DeepSeek R1 | Local Ollama | Thinking model, use `max_tokens >= 2000` |
| Any OpenAI-compatible | LiteLLM / direct | Plug and play |

---

## File Map

```
qwen-aiciv-mind/
+-- Cargo.toml                          # Workspace root (18 members)
+-- MISSIONS.md                         # THIS FILE -- the master plan
+-- SOUL.md                             # Identity and philosophy
+-- projects/aiciv-mind/
|   +-- shared-scratchpad.md            # Cross-agent coordination scratchpad
|   +-- README.md                       # Project overview
+-- src/
|   +-- codex-types/                    # Shared types (mind-coordination)
|   +-- codex-roles/                    # Role permissions (mind-coordination)
|   +-- codex-coordination/             # Multi-agent orchestration (mind-coordination)
|   +-- codex-fitness/                  # Health scoring (mind-testing)
|   +-- codex-redteam/                  # Challenger system (mind-testing)
|   +-- codex-dream/                    # Dream engine (mind-memory)
|   +-- codex-transfer/                 # Knowledge transfer (mind-memory)
|   +-- codex-suite-client/             # Suite interceptors (mind-mcp)
|   +-- codex-memory/                   # SQLite memory store (mind-memory)
|   +-- codex-exec/                     # Tool execution (mind-tool-engine)
|   +-- codex-ipc/                      # IPC protocol (mind-coordination)
|   +-- codex-llm/                      # LLM client (mind-model-router)
|   +-- codex-drive/                    # Drive loop (mind-model-router)
|   +-- codex-patcher/                  # Lifecycle patches (mind-hooks)
|   +-- cortex/                         # Main binary (mind-coordination)
|   +-- cortex-memory/                  # Graph memory (mind-memory)
|   +-- cortex-monitoring/              # Monitoring (mind-testing)
|   +-- qwen-mind/                      # Qwen-specific mind (mind-model-router)
+-- codex-upstream/                     # Codex source for cherry-picking
+-- docs/                              # Documentation
+-- config/                            # Runtime configuration
```
