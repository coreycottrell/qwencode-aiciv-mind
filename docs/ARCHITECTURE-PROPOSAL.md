# aiciv-mind-cubed — Architecture Proposal

**The Fractal Coordination Engine on OpenAI Codex**

**Author**: Mind-Cubed Team Lead
**Date**: 2026-04-03
**Status**: PROPOSAL — awaiting review

---

## Executive Summary

Codex CLI is a 90-crate Rust monorepo that already solves 60% of what aiciv-mind needs: sandboxing, multi-provider inference, MCP client/server, session persistence, memory consolidation, AGENTS.md hierarchical instructions, a Python SDK for orchestration, and a JSON-RPC app server. We don't build those. We fork them and inject what's missing: the fractal coordination engine that turns a single-agent coding CLI into a self-improving civilization of minds.

The core architectural move: **a Codex instance IS a mind.** A Primary mind is a Codex instance whose AGENTS.md says "you are a conductor" and whose tool registry exposes only coordination tools. A Team Lead mind is a Codex instance with delegation tools. An Agent mind is a `codex exec` instance with full sandbox and tools. Coordination happens via MCP (mind-to-mind) and the Python SDK (orchestration). No custom IPC framework needed — we ride the protocol Codex already speaks.

Gemma 4 and M2.7 from day 1, both via Ollama. Codex already supports Ollama natively. We configure, not build.

---

## Table of Contents

1. [Codex Architecture Map](#1-codex-architecture-map)
2. [Where the 12 Principles Plug In](#2-where-the-12-principles-plug-in)
3. [The Coordination Layer](#3-the-coordination-layer)
4. [Role Architecture](#4-role-architecture)
5. [Memory Integration](#5-memory-integration)
6. [Model Strategy: Gemma 4 + M2.7](#6-model-strategy-gemma-4--m27)
7. [What Makes This Different](#7-what-makes-this-different)
8. [Implementation Phases](#8-implementation-phases)

---

## 1. Codex Architecture Map

Codex is organized in tiers. We touch specific crates per tier — everything else we inherit.

### What We Inherit (Don't Touch)

| Tier | Crates | Value |
|------|--------|-------|
| **Sandbox** | `codex-sandboxing`, `codex-linux-sandbox`, `codex-process-hardening` | Production Landlock+seccomp+bubblewrap on Linux. Agent isolation solved. |
| **Providers** | `codex-model-provider-info`, `codex-ollama`, `codex-lmstudio` | Ollama, OpenAI, Gemini, custom — multi-provider from day 1 |
| **TUI** | `codex-tui`, `codex-ansi-escape` | Interactive mode for human-facing use |
| **Auth/Secrets** | `codex-login`, `codex-keyring-store`, `codex-secrets` | System keyring, age encryption |
| **Analytics** | `codex-analytics`, `codex-otel` | OpenTelemetry tracing |
| **Utilities** | 20+ crates in `utils/` | Path handling, fuzzy match, pty, caching, etc. |
| **Build** | Cargo workspace + Bazel | Dual build system |

### What We Extend (Surgical Injection)

| Crate | Extension | Why |
|-------|-----------|-----|
| **`codex-core`** | Add `CoordinatorLoop` alongside existing `Codex` agent loop | The fractal parent that manages child Codex instances |
| **`codex-protocol`** | Add `Op::SpawnMind`, `Op::DelegateTo`, `Event::MindResult`, `Event::CoordinationState` | New SQ/EQ messages for multi-mind coordination |
| **`codex-state`** | Add tables: `mind_registry`, `delegation_graph`, `memory_depth`, `memory_links` | Persistent coordination state + graph memory |
| **`codex-instructions`** | Add role-scoped AGENTS.md injection with tool filtering metadata | Manifests as AGENTS.md files with embedded role constraints |
| **`codex-app-server-protocol`** | Add v2 methods: `mind/spawn`, `mind/delegate`, `mind/status`, `mind/shutdown` | SDK coordination primitives |
| **`codex-mcp-server`** | Extend to register sub-minds as virtual MCP tools | Mind-to-mind delegation via MCP tool calls |
| **`codex-memories`** | Add depth scoring, graph links, 3-tier architecture | Graph memory with forgetting protocol |
| **`codex-config`** | Add `[coordination]` config section, role definitions | Fractal coordination configuration |

### What We Add (New Crates)

| New Crate | Purpose |
|-----------|---------|
| **`codex-coordination`** | The fractal engine: role hierarchy, spawn triggers, fitness scoring, delegation routing |
| **`codex-roles`** | Role enum (`Primary`, `TeamLead`, `Agent`), tool whitelist per role, manifest validation |
| **`codex-dream`** | Dream Mode: nightly review, pattern search, deliberate forgetting, self-improvement |
| **`codex-redteam`** | Red team verification: adversarial check per completion, evidence assessment |
| **`codex-transfer`** | Cross-domain transfer: pattern publication, subscription, adaptation, validation |
| **`codex-suite-client`** | AiCIV Suite integration: Hub, AgentAuth, AgentCal native clients |
| **`codex-fitness`** | Role-specific fitness scoring: Primary/TeamLead/Agent metrics |

**Total new crates: 7.** The other 90 we inherit.

---

## 2. Where the 12 Principles Plug In

### Principle 1: MEMORY IS THE ARCHITECTURE

**Codex has**: A 2-phase memory pipeline (`codex-memories/`). Phase 1: per-thread rollout extraction (up to 8 concurrent). Phase 2: global consolidation with watermark-based dirty checking. Memory stored as `raw_memories.md` + `rollout_summaries/` + `memory_summary.md`. SQLite-backed state with usage tracking.

**What we add**:

```
codex-memories/ (EXTEND)
├── depth_scoring.rs      ← NEW: depth_score = f(access_count, recency, citations, decision_weight, cross_mind_shares, human_endorsement)
├── graph_links.rs        ← NEW: memories as nodes — reference, supersede, conflict, compound edges
├── three_tier.rs         ← NEW: working (session) → long-term (all sessions) → civilizational (Hub)
└── consolidation.rs      ← MODIFY: add depth-aware archival + graph consolidation to Phase 2

codex-state/ (EXTEND)
├── migrations/
│   ├── 006_memory_depth.sql    ← depth_score column, last_access, citation_count
│   └── 007_memory_graph.sql    ← memory_links table (source, target, link_type, strength)
```

The key: Codex already extracts memories from rollouts and consolidates them. We make the consolidator SMARTER — it scores depth, builds graph links, and archives based on principled criteria rather than flat deduplication.

**Memory citation tracking** already exists in Codex (`usage_count`, `last_usage`). We extend it with graph awareness: when memory A cites memory B during a session, that's a `reference` edge in the graph, and B's depth_score increases.

### Principle 2: SYSTEM > SYMPTOM

**Codex has**: Error reporting via `codex-feedback`, analytics via `codex-otel`.

**What we add**: In `codex-coordination`, a `SystemicAnalyzer` that fires after every error:

```rust
// codex-coordination/src/systemic.rs
pub struct SystemicAnalyzer {
    memory: MemoryClient,
    pattern_threshold: usize, // default 3
}

impl SystemicAnalyzer {
    /// Called after every tool error, delegation failure, or verification rejection.
    pub async fn analyze(&self, failure: &FailureEvent) -> SystemicLearning {
        // 1. Classify failure category
        // 2. Search memory for same category (if 3+ → PATTERN)
        // 3. Identify structural cause
        // 4. Propose systemic fix
        // 5. Write learning to memory with category tag
    }
}
```

This runs as a lightweight post-hook on Codex's existing event system (`codex-hooks`). The hook fires on `Event::ToolError` and `Event::ItemCompleted` with error status.

### Principle 3: GO SLOW TO GO FAST

**Codex has**: No planning gate. Direct prompt → execution.

**What we add**: In `codex-coordination`, a `PlanningGate` that intercepts every task:

```rust
// codex-coordination/src/planning.rs
pub enum TaskComplexity { Trivial, Simple, Medium, Complex, Novel }

pub struct PlanningGate {
    memory: MemoryClient,
    spawn_threshold: TaskComplexity, // Medium and above spawn planning sub-mind
}

impl PlanningGate {
    /// Every task passes through this before execution.
    pub async fn evaluate(&self, task: &Task) -> PlanningDecision {
        // Trivial: memory check only (< 1s)
        // Simple: memory check + brief plan
        // Medium: memory check + competing hypotheses
        // Complex: spawn planning sub-mind (fresh context)
        // Novel: spawn multiple competing planners
    }
}
```

The gate integrates into the `CoordinatorLoop` (see §3). Before any `Op::SpawnMind` or `Op::DelegateTo`, the coordinator runs the planning gate. The gate itself is an LLM call using the lightweight model (M2.7 for trivial/simple, Gemma 4 for medium+).

### Principle 4: DYNAMIC AGENT SPAWNING

**Codex has**: `codex exec` for headless agent execution. The Python SDK can spawn multiple `codex exec` instances.

**What we add**: Spawn triggers in `codex-coordination`:

```rust
// codex-coordination/src/triggers.rs
pub enum SpawnTrigger {
    PatternRepetition { pattern_id: String, count: usize },
    TaskComplexityExceeded { threshold: TaskComplexity },
    CompetingHypotheses { count: usize },
    BlockingDetected { duration: Duration, retries: usize },
    DomainBoundary { from_vertical: String, to_vertical: String },
    VerificationNeed { task_id: String },
    ContextPressure { usage_pct: f64 },
    Scheduled { trigger: ScheduledTrigger },
}
```

**Dream Mode** lives in the new `codex-dream` crate. It orchestrates via the Python SDK:

```python
# sdk/python/codex_dream/dream_cycle.py
class DreamCycle:
    """Nightly: review → pattern search → deliberate forgetting → self-improvement → artifacts."""

    async def run(self, coordinator: AsyncCodex):
        # Phase 1: Each team lead reviews its day via codex exec
        reviews = await asyncio.gather(*[
            coordinator.exec(f"Review today's work for {vertical}", model="gemma4")
            for vertical in self.verticals
        ])
        # Phase 2: Pattern search across all memories
        patterns = await coordinator.exec("Search for recurring patterns...", model="gemma4")
        # Phase 3: Deliberate forgetting (depth-based archival)
        await self.archive_low_depth_memories()
        # Phase 4: Self-improvement (manifest evolution, routing updates)
        improvements = await coordinator.exec("Based on today's patterns...", model="gemma4")
        # Phase 5: Write dream artifacts
        await self.write_dream_artifacts(reviews, patterns, improvements)
```

### Principle 5: HIERARCHICAL CONTEXT DISTRIBUTION

**This is the crown jewel. Codex's architecture makes this natural.**

Each Codex instance has its own context window. The `ThreadManager` is already a factory for `Codex` instances. We add a `CoordinatorLoop` that manages a HIERARCHY of ThreadManagers:

```
CoordinatorLoop (Primary)
├── ThreadManager (research-lead) ← its own 200K context
│   ├── codex exec (researcher-1) ← its own context, sandboxed
│   └── codex exec (researcher-2) ← its own context, sandboxed
├── ThreadManager (code-lead) ← its own 200K context
│   ├── codex exec (coder-1)
│   └── codex exec (coder-2)
└── ThreadManager (memory-lead)
    └── codex exec (memory-worker)
```

The `InProcessAppServerClient` that `codex exec` already uses means sub-minds get in-process communication (no network). Team Leads get persistent ThreadManagers (session continuity across tasks). Agents get ephemeral `codex exec` instances (sandboxed, fire-and-forget).

**The math**: 1 Primary + 5 Team Leads + 15 Agents = 21 context windows. Each 200K. Total: 4.2M tokens of parallel intelligence. Codex's architecture supports this TODAY — we just wire the hierarchy.

### Principle 6: CONTEXT ENGINEERING

**Codex has**: Auto-compaction in `codex-core` with `compact` protocol ops. The `thread/compact/start` API endpoint. Rollout-based persistence.

**What we add**: A `ContextEngineeringLead` — a Team Lead mind whose AGENTS.md defines it as a metacognition specialist:

```markdown
# AGENTS.md (context-engineering-lead)

You are the Context Engineering Lead. Your domain: managing OTHER minds' attention.

When invoked, you receive:
- A mind's current context snapshot
- The task it's working on
- Its memory index

Your job:
1. Identify what's essential vs noise
2. Produce an optimized summary preserving essentials in minimal tokens
3. Recommend what to load from memory for the next phase
4. Return a compaction strategy (Preserve-Code, Preserve-Decisions, etc.)

You operate in YOUR context window — never pollute the mind you're helping.
```

This team lead uses Codex's existing `thread/compact/start` + a custom compaction strategy enum. The strategies map to different consolidation prompts in Phase 2 of the memory pipeline.

### Principle 7: SELF-IMPROVING LOOP

**Codex has**: Memory consolidation (learns from sessions). Hooks (event-driven automation).

**What we add**: Three nested loops in `codex-fitness`:

```rust
// codex-fitness/src/lib.rs

/// Loop 1: After every task
pub struct TaskLearning { /* what worked, what didn't, routing accuracy */ }

/// Loop 2: After every session
pub struct SessionLearning { /* cross-task patterns, context efficiency, agent perf */ }

/// Loop 3: Dream Mode (nightly)
pub struct CivilizationLearning { /* cross-session patterns, manifest evolution, meta-evolution */ }

/// The meta-layer: measures whether the improvement process itself is improving
pub struct MetaEvolution {
    routing_accuracy_trend: Vec<f64>,   // Is routing getting better over sessions?
    pattern_detection_recall: Vec<f64>, // Are we catching more patterns?
    dream_mode_impact: Vec<f64>,        // Do dream artifacts actually improve next day?
}
```

These integrate with `codex-hooks`: after every `Event::ItemCompleted`, the task learning hook fires. After thread close, session learning. Dream Mode runs on schedule via `codex-dream`.

### Principle 8: IDENTITY PERSISTENCE

**Codex has**: Thread metadata in SQLite (`codex-state`). Session rollouts on disk. Memory summary persistence.

**What we add**: A `mind_registry` table in `codex-state`:

```sql
-- codex-state/migrations/008_mind_registry.sql
CREATE TABLE mind_registry (
    mind_id TEXT PRIMARY KEY,
    role TEXT NOT NULL CHECK(role IN ('primary', 'team_lead', 'agent')),
    vertical TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    session_count INTEGER NOT NULL DEFAULT 0,
    growth_stage TEXT NOT NULL DEFAULT 'novice',
    -- Identity state
    core_memory_ids TEXT, -- JSON array of high-depth memory IDs
    relationship_ids TEXT, -- JSON array of {mind_id, trust_score}
    manifest_version INTEGER NOT NULL DEFAULT 1,
    last_session_at TIMESTAMP
);
```

Each mind's AGENTS.md is generated from its registry entry + accumulated learnings. Session 1's AGENTS.md is a template. Session 50's AGENTS.md has been evolved by Dream Mode 49 times.

### Principle 9: RED TEAM EVERYTHING

**What we add**: `codex-redteam` crate:

```rust
// codex-redteam/src/lib.rs
pub struct RedTeamAgent {
    model: String,  // Uses M2.7 for lightweight verification
}

pub enum RedTeamVerdict {
    Approved { evidence_quality: f64 },
    Challenged { questions: Vec<String> },
    Blocked { finding: String },
}

impl RedTeamAgent {
    /// Spawned as codex exec with --ephemeral --sandbox read-only
    pub async fn verify(&self, task: &Task, result: &TaskResult) -> RedTeamVerdict {
        // Runs in its own sandbox — cannot modify anything
        // Receives: task description, proposed completion, relevant memories
        // Returns: verdict with evidence assessment
    }
}
```

The red team agent is a `codex exec --ephemeral --sandbox read-only` instance. It CANNOT write files, modify state, or access network. It can only read and reason. This leverages Codex's sandbox for adversarial safety.

### Principle 10: CROSS-DOMAIN TRANSFER

**What we add**: `codex-transfer` crate with Hub integration via `codex-suite-client`:

```rust
// codex-transfer/src/lib.rs
pub struct TransferPattern {
    source_mind: String,
    pattern: String,
    evidence: String,
    confidence: Confidence,
    share_scope: ShareScope, // Own | Civ | Public
}

pub struct TransferEngine {
    hub: HubClient,
    memory: MemoryClient,
}

impl TransferEngine {
    /// Publish a discovered pattern to Hub Knowledge:Items
    pub async fn publish(&self, pattern: TransferPattern) -> Result<()>;
    /// Subscribe to patterns from other minds/civs
    pub async fn subscribe(&self, filter: PatternFilter) -> Vec<TransferPattern>;
    /// Adapt a received pattern to this mind's domain
    pub async fn adapt(&self, pattern: &TransferPattern, context: &str) -> AdaptedPattern;
}
```

### Principle 11: DISTRIBUTED INTELLIGENCE

Every layer in Codex becomes smart:

| Layer | Codex Crate | Intelligence We Add |
|-------|-------------|---------------------|
| **Tools** | `codex-core/tools/` | Adaptive memory-search re-ranking by task context |
| **Context** | `codex-core/compaction` | Context Engineering Lead (Principle 6) |
| **Communication** | `codex-mcp` | Priority-aware message routing between minds |
| **Memory** | `codex-memories` | Self-organizing graph with depth scoring |
| **Scheduling** | `codex-coordination` | Dependency-aware parallel spawning |
| **Meta** | `codex-fitness` | Recursive self-improvement measurement |
| **Services** | `codex-suite-client` | Semantic service awareness (AgentCal patterns, Hub rooms) |

### Principle 12: NATIVE SERVICE INTEGRATION

**What we add**: `codex-suite-client` crate:

```rust
// codex-suite-client/src/lib.rs
pub struct SuiteClient {
    pub auth: AuthClient,   // AgentAuth — Ed25519 JWT, challenge-response
    pub hub: HubClient,     // Hub — rooms, threads, knowledge items, feed
    pub cal: CalClient,     // AgentCal — events, availability
    pub memory: MemoryBridge, // Dual-write: local SQLite + Hub Knowledge:Items
}

impl SuiteClient {
    /// Each mind gets its own identity.
    /// "acg/primary", "acg/research-lead", "acg/coder-1"
    pub async fn connect(keypair_id: &str, config: &MindConfig) -> Result<Self>;
}
```

The SuiteClient is injected into every mind at spawn time. It's registered as an MCP server in the mind's config, so the model can use Hub/Auth/Cal as native MCP tools. Codex's existing MCP client (`codex-mcp`) aggregates SuiteClient tools alongside any other MCP servers.

---

## 3. The Coordination Layer

### Architecture: CoordinatorLoop

The `CoordinatorLoop` is the new heart. It lives in `codex-coordination` and wraps Codex's existing `ThreadManager`:

```
┌─────────────────────────────────────────────────┐
│              CoordinatorLoop (PRIMARY)            │
│                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Planning  │  │ Spawn    │  │ Fitness  │       │
│  │ Gate      │  │ Triggers │  │ Scorer   │       │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       │              │              │              │
│  ┌────▼──────────────▼──────────────▼────┐       │
│  │         MindManager                    │       │
│  │  (wraps N ThreadManagers per role)     │       │
│  └────┬───────────────┬──────────────┬───┘       │
│       │               │              │            │
│  ┌────▼────┐   ┌──────▼────┐   ┌────▼────┐      │
│  │ TL Mind │   │ TL Mind   │   │ TL Mind │      │
│  │ (Codex) │   │ (Codex)   │   │ (Codex) │      │
│  │ research│   │ code      │   │ memory  │      │
│  └────┬────┘   └─────┬─────┘   └────┬────┘      │
│       │              │              │             │
│  ┌────▼────┐   ┌─────▼─────┐  ┌────▼────┐      │
│  │exec exec│   │exec exec  │  │exec     │      │
│  │ (agent) │   │ (agent)   │  │(agent)  │      │
│  └─────────┘   └───────────┘  └─────────┘      │
└─────────────────────────────────────────────────┘
```

### MindManager

```rust
// codex-coordination/src/mind_manager.rs

pub struct MindManager {
    /// Registry of all active minds
    minds: HashMap<MindId, MindHandle>,
    /// Thread managers (one per persistent mind: Primary + Team Leads)
    thread_managers: HashMap<MindId, ThreadManager>,
    /// State DB for persistence
    state: StateRuntime,
    /// MCP connection manager for mind-to-mind communication
    mcp: McpConnectionManager,
}

pub struct MindHandle {
    pub id: MindId,
    pub role: Role,
    pub vertical: Option<String>,
    pub thread_manager: Option<ThreadManagerRef>,  // None for ephemeral agents
    pub status: MindStatus,
}

impl MindManager {
    /// Spawn a Team Lead as a persistent Codex thread
    pub async fn spawn_team_lead(&mut self, vertical: &str, objective: &str) -> Result<MindId> {
        // 1. Load manifest template for this vertical
        // 2. Load scratchpad + memory index
        // 3. Generate AGENTS.md with role constraints + vertical context
        // 4. Create ThreadManager with role-filtered tool registry
        // 5. Start thread with objective injected
        // 6. Register in mind_registry
        // 7. Return MindId for delegation
    }

    /// Spawn an Agent as an ephemeral codex exec instance
    pub async fn spawn_agent(&mut self, parent: MindId, task: &str) -> Result<MindId> {
        // 1. Determine sandbox policy from task type
        // 2. Generate AGENTS.md with agent role + task context
        // 3. Launch codex exec --sandbox <policy> --json
        // 4. Stream events back to parent mind
        // 5. Return MindId
    }

    /// Route a delegation from one mind to another
    pub async fn delegate(&self, from: MindId, to: MindId, task: &Task) -> Result<TaskHandle>;

    /// Get coordination state for all minds
    pub fn coordination_state(&self) -> CoordinationState;
}
```

### Communication Paths

| Path | Mechanism | Latency | Use Case |
|------|-----------|---------|----------|
| Primary → Team Lead | In-process `ThreadManager` | < 1ms | Delegation, status queries |
| Team Lead → Agent | `codex exec` subprocess | ~100ms startup | Sandboxed task execution |
| Mind → Mind (same host) | MCP over stdio pipe | < 10ms | Cross-vertical coordination |
| Mind → Mind (cross-host) | Hub HTTP API | 50-200ms | Inter-civ delegation |
| Mind → Suite Services | `codex-suite-client` (HTTP) | 50-200ms | Hub, Auth, Cal operations |

**Why MCP for mind-to-mind**: Codex already has `codex-mcp-server` that exposes a Codex instance as an MCP tool. A Team Lead can register its sub-agents as MCP servers. When the Team Lead's model calls `research-agent::search`, it's actually delegating to a sub-mind. The MCP protocol handles serialization, tool listing, and invocation. No custom IPC needed.

**Why `codex exec` for agents**: The `--sandbox` flag gives us Landlock+seccomp isolation per agent. The `--json` flag gives us JSONL event streams. The `--ephemeral` flag prevents session pollution. The `--output-schema` flag enables structured output. All of this is BUILT. We just use it.

---

## 4. Role Architecture

### Role-Based Tool Filtering via AGENTS.md

Instead of modifying Codex's core tool registry (fragile, high merge-conflict risk), we use AGENTS.md to control behavior:

```
.codex/agents/
├── primary.agents.md       ← "You are a conductor. You spawn team leads. You NEVER execute tools."
├── team-lead/
│   ├── research.agents.md  ← "You are research-lead. You delegate to agents. You synthesize results."
│   ├── code.agents.md
│   ├── memory.agents.md
│   ├── comms.agents.md
│   └── ops.agents.md
└── agent/
    ├── researcher.agents.md ← "You are a researcher. Full tool access. Verify everything."
    ├── coder.agents.md
    └── verifier.agents.md
```

But AGENTS.md is behavioral guidance, not structural enforcement. For HARD enforcement, we add the `codex-roles` crate:

```rust
// codex-roles/src/lib.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Primary,
    TeamLead,
    Agent,
}

/// Tool whitelist per role — structurally enforced
pub fn tools_for_role(role: Role) -> &'static [&'static str] {
    match role {
        Role::Primary => &[
            "mind_spawn_team_lead",
            "mind_delegate",
            "mind_status",
            "mind_shutdown",
            "coordination_read",
            "coordination_write",
            "send_message",
            "memory_search",  // Primary CAN search memory (read-only)
        ],
        Role::TeamLead => &[
            "mind_spawn_agent",
            "mind_delegate",
            "mind_status",
            "team_scratchpad_read",
            "team_scratchpad_write",
            "coordination_read",
            "send_message",
            "memory_search",
            "memory_write",
        ],
        Role::Agent => &[
            // ALL Codex tools: bash, file_read, file_write, grep, web_search, etc.
            // Plus memory tools
            "*",
        ],
    }
}
```

This filter is applied in `MindManager::spawn_team_lead()` and `::spawn_agent()`. The spawned Codex instance's `ToolRegistry` is built via `ToolRegistry::for_role(role)`. The LLM never sees tools outside its whitelist.

### AGENTS.md as Manifests

Codex's AGENTS.md system (`codex-instructions` crate) already supports:
- Hierarchical directory-scoped instructions
- Fragment markers for injection into conversation
- Skill instructions alongside agent instructions

We map our manifest system to AGENTS.md:

| Our Concept | AGENTS.md Implementation |
|-------------|--------------------------|
| Manifest identity | Opening section of AGENTS.md |
| Delegation roster | Inline table of available sub-minds |
| Skills | Codex's native `/skills/` directory |
| Anti-patterns | "## What NOT to Do" section |
| Memory protocol | "## Memory Discipline" section |
| Scratchpad | Codex's native session persistence |

**Example Team Lead AGENTS.md (generated from template + mind registry)**:

```markdown
# Research Team Lead — AGENTS.md

## Identity
You are the Research Team Lead for A-C-Gee civilization.
Session: 47. Growth stage: proficient. This mind has been active since 2026-04-03.

## Your Role
You coordinate research specialists. You do NOT execute research yourself.
You spawn agents, receive their results, and synthesize insight for Primary.

## Available Agents
| Agent | Specialty | Recent Perf |
|-------|-----------|-------------|
| web-researcher | Web search, article extraction | 94% task completion |
| code-analyst | Codebase analysis, pattern detection | 87% task completion |
| hypothesis-tester | Competing hypothesis evaluation | 91% task completion |

## Delegation Protocol
1. Break the task into parallel research angles
2. Spawn one agent per angle (use mind_spawn_agent tool)
3. Collect results
4. Synthesize into 100-200 token summary for Primary
5. Write learnings to team scratchpad

## Memory Discipline
Before ANY task: search memory for prior research on this topic.
After ANY task: write what you learned to team scratchpad.

## What NOT to Do
- Do NOT call bash, grep, file_read, or web_search — you don't have those tools
- Do NOT return raw agent output to Primary — always synthesize
- Do NOT spawn more than 5 agents for a single task without planning gate
```

---

## 5. Memory Integration

### Three-Tier Architecture on Codex's Foundation

```
┌───────────────────────────────────────────────────┐
│  Tier 1: Working Memory (this mind, this session)  │
│  Store: Codex's in-session context + rollout       │
│  Latency: 0ms (in context)                         │
│  Source: codex-core's existing conversation state   │
├───────────────────────────────────────────────────┤
│  Tier 2: Long-Term Memory (this mind, all sessions)│
│  Store: codex-state SQLite + graph extensions       │
│  Latency: < 5ms                                    │
│  Source: codex-memories pipeline + depth scoring    │
├───────────────────────────────────────────────────┤
│  Tier 3: Civilizational Memory (all minds, all civs│
│  Store: Hub Knowledge:Items via codex-suite-client  │
│  Latency: 50-200ms                                 │
│  Source: codex-transfer publication engine           │
└───────────────────────────────────────────────────┘
```

### Extending Codex's Memory Pipeline

Codex's existing 2-phase pipeline:
- **Phase 1**: Extract memories from rollouts (concurrent, per-thread, up to 8 workers)
- **Phase 2**: Global consolidation (serialized, single worker, watermark-based)

We extend Phase 2:

```rust
// In codex-memories/src/consolidation.rs (MODIFY)

struct EnhancedConsolidator {
    // Existing
    raw_memories: Vec<RawMemory>,
    rollout_summaries: Vec<RolloutSummary>,

    // New
    depth_scorer: DepthScorer,
    graph_builder: GraphBuilder,
    archiver: MemoryArchiver,
}

impl EnhancedConsolidator {
    async fn consolidate(&mut self) -> ConsolidationResult {
        // 1. Standard Codex consolidation (existing behavior)
        let base_result = self.standard_consolidate().await;

        // 2. Score depth for all memories
        self.depth_scorer.score_all(&base_result.memories).await;

        // 3. Build/update graph links
        self.graph_builder.update_links(&base_result.memories).await;

        // 4. Archive low-depth, uncited memories
        self.archiver.archive_below_threshold(0.1).await;

        // 5. Detect contradictions
        let contradictions = self.graph_builder.find_contradictions().await;

        // 6. Publish high-value memories to Hub (Tier 3)
        self.publish_to_hub(base_result.memories.filter(|m| m.depth > 0.8)).await;

        base_result
    }
}
```

### Memory Graph Schema

```sql
-- codex-state/migrations/007_memory_graph.sql

CREATE TABLE memory_nodes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    category TEXT NOT NULL,  -- 'learning', 'pattern', 'decision', 'failure', 'synthesis'
    depth_score REAL NOT NULL DEFAULT 0.0,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed TIMESTAMP,
    citation_count INTEGER NOT NULL DEFAULT 0,
    cross_mind_shares INTEGER NOT NULL DEFAULT 0,
    human_endorsed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP,  -- NULL = active
    mind_id TEXT NOT NULL REFERENCES mind_registry(mind_id)
);

CREATE TABLE memory_edges (
    source_id TEXT NOT NULL REFERENCES memory_nodes(id),
    target_id TEXT NOT NULL REFERENCES memory_nodes(id),
    edge_type TEXT NOT NULL CHECK(edge_type IN ('reference', 'supersede', 'conflict', 'compound')),
    strength REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_id, target_id, edge_type)
);

CREATE INDEX idx_memory_depth ON memory_nodes(depth_score DESC);
CREATE INDEX idx_memory_category ON memory_nodes(category, depth_score DESC);
CREATE INDEX idx_memory_mind ON memory_nodes(mind_id);
```

---

## 6. Model Strategy: Gemma 4 + M2.7

### Configuration

Codex supports custom providers in `config.toml`. Both models run via Ollama:

```toml
# .codex/config.toml

[model_providers.ollama]
name = "Ollama"
base_url = "http://localhost:11434/v1"
wire_api = "responses"

# Model mapping for different roles
[coordination]
primary_model = "gemma4"           # Gemma 4 — orchestration, synthesis, planning
team_lead_model = "gemma4"         # Gemma 4 — delegation, coordination
agent_model = "gemma4"             # Gemma 4 — task execution
lightweight_model = "m2.7"         # M2.7 — red team, trivial planning, memory scoring
dream_model = "gemma4"             # Gemma 4 — nightly review and evolution
memory_extraction_model = "m2.7"   # M2.7 — Phase 1 rollout extraction (cheap, high volume)
```

### Model Selection Logic

```rust
// codex-coordination/src/model_selection.rs

pub fn model_for_task(role: Role, task_type: TaskType) -> &'static str {
    match (role, task_type) {
        // Primary always uses Gemma 4 (orchestration is high-stakes)
        (Role::Primary, _) => "gemma4",

        // Team leads: Gemma 4 for delegation, M2.7 for scratchpad maintenance
        (Role::TeamLead, TaskType::Delegation) => "gemma4",
        (Role::TeamLead, TaskType::ScratchpadWrite) => "m2.7",

        // Agents: Gemma 4 for complex work, M2.7 for simple verification
        (Role::Agent, TaskType::Complex) => "gemma4",
        (Role::Agent, TaskType::Verification) => "m2.7",

        // Red team always uses M2.7 (lightweight, high volume)
        (_, TaskType::RedTeam) => "m2.7",

        // Memory operations use M2.7 (extraction is high-volume, low-stakes)
        (_, TaskType::MemoryExtraction) => "m2.7",
        (_, TaskType::MemoryConsolidation) => "gemma4",

        // Default: Gemma 4
        _ => "gemma4",
    }
}
```

### Why Both Models

**Gemma 4** (larger): Orchestration, planning, synthesis, complex reasoning. Quality matters more than speed.

**M2.7** (smaller, faster): Red team verification, memory extraction, trivial planning gates, scratchpad maintenance. Volume matters more than depth. Running M2.7 for every red team check keeps verification lightweight enough to run on EVERY completion without blocking.

Both via Ollama, both local, both free. Zero inference cost from day 1.

---

## 7. What Makes This Different

### vs aiciv-mind (Python, from scratch)

| Dimension | aiciv-mind | aiciv-mind-cubed |
|-----------|-----------|-----------------|
| **Language** | Python | Rust (Codex) + Python (SDK orchestration) |
| **Foundation** | Built from scratch — every component custom | Fork of 90-crate battle-tested foundation (72K+ stars) |
| **Sandbox** | tmux-based process isolation | Landlock + seccomp + bubblewrap (production Linux security) |
| **IPC** | Custom ZeroMQ ROUTER/DEALER | MCP (standard protocol) + in-process AppServerClient |
| **Memory** | Custom SQLite + FTS5 + Hub dual-write | Codex's 2-phase pipeline + our graph extensions |
| **Multi-provider** | Custom provider abstraction | 10+ providers native (Ollama, Gemini, OpenAI, custom) |
| **Sub-mind spawning** | Custom `run_submind.py` | `codex exec` — production-grade headless agent with sandbox |
| **Session persistence** | Custom rollout recording | Codex's JSONL rollouts + SQLite state — already built |
| **Build effort** | ~15K LOC needed for v0.1 | ~3K LOC of new crates (7 new + extensions to 8 existing) |
| **Risk** | High — every component must work together | Low — 90% of the foundation is battle-tested |

**The 10x insight**: aiciv-mind spent months building infrastructure (tool registry, spawning, memory, IPC, sandbox). aiciv-mind-cubed gets ALL of that from Codex and spends its time on what matters: the fractal coordination engine.

### vs aiciv-mind-too (claw-code/Rust, clean-room)

| Dimension | aiciv-mind-too | aiciv-mind-cubed |
|-----------|---------------|-----------------|
| **Base** | Clean-room Rust rewrite of Claude Code | Fork of OpenAI Codex (Apache-2.0) |
| **Risk** | Very high — reimplementing a complex system | Low — extending a working system |
| **MCP** | Must implement from scratch | MCP client AND server already built |
| **SDK** | Must build | Python SDK already exists |
| **Memory** | Must build | 2-phase pipeline already exists |
| **Community** | None (private project) | 72K+ stars, active development, PRs welcome |
| **Upstream updates** | N/A (standalone) | Can merge upstream improvements |
| **Earned capabilities** | Proposed, not implemented | Can test alongside hard-coded roles |
| **App server** | Must build | JSON-RPC app server with 40+ API methods |

**The strategic advantage**: aiciv-mind-too tries to build a better Claude Code from scratch. aiciv-mind-cubed says "Codex already built a great coding agent — we make it a coordination engine." Different bet, different risk profile.

### The Unique Angle (Only Mind-Cubed Has This)

1. **MCP as inter-mind protocol**: A mind-cubed Primary exposes itself as an MCP server. Other civilizations, other tools, other agents can invoke it via standard MCP. No custom protocol learning curve. This is how Witness talks to ACG's minds — through MCP, not Hub thread parsing.

2. **`codex exec` as safe agent spawning**: Each agent runs in a Landlock+seccomp sandbox. Not tmux isolation, not container overhead. Kernel-level security with <100ms spawn time. A team lead can safely spawn 10 agents in parallel without worrying about one corrupting another's state.

3. **Codex's memory pipeline for civilizational learning**: Phase 1's concurrent extraction (8 workers) means memories from 8 parallel agents get processed simultaneously. Phase 2's watermark-based consolidation means no duplicate work. We add depth scoring and graph links on top — Codex does the heavy lifting.

4. **Python SDK for Dream Mode orchestration**: Codex's `AsyncCodex` class in the Python SDK gives us a clean orchestration API. Dream Mode is a Python script that uses the SDK to spawn review agents, collect results, and write dream artifacts. No need to build orchestration infrastructure.

5. **Upstream improvements for free**: When OpenAI ships better sandboxing, better MCP support, better memory consolidation — we get it. `git merge upstream/main`. No comparable advantage in from-scratch builds.

---

## 8. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: One Primary mind coordinating two Team Lead minds, each delegating to one Agent mind.

| Task | Crate | Effort |
|------|-------|--------|
| Fork Codex, set up workspace | — | 1 day |
| Create `codex-roles` with Role enum + tool whitelist | New | 1 day |
| Create `codex-coordination` with MindManager skeleton | New | 2 days |
| Extend `codex-protocol` with spawn/delegate ops | Modify | 1 day |
| Write Primary/TeamLead/Agent AGENTS.md templates | New files | 1 day |
| Configure Gemma 4 + M2.7 via Ollama in config.toml | Config | 0.5 day |
| Integration test: Primary → TeamLead → Agent → result | Test | 1.5 days |

**Deliverable**: `codex-mind spawn --role primary` launches a coordination hierarchy. Primary delegates "research X" to research-lead, which spawns a researcher agent, which returns a result. The result flows back up the hierarchy.

### Phase 2: Memory + Identity (Week 3-4)

| Task | Crate | Effort |
|------|-------|--------|
| Extend `codex-state` with memory_nodes, memory_edges, mind_registry tables | Modify | 2 days |
| Extend `codex-memories` consolidation with depth scoring + graph builder | Modify | 3 days |
| Add identity persistence (session count, growth stage tracking) | Modify state | 1 day |
| AGENTS.md generation from mind registry + accumulated learnings | New module | 2 days |
| Integration test: memories persist across sessions, depth scores evolve | Test | 2 days |

**Deliverable**: Minds remember across sessions. Memory has depth. Session 5's research-lead is measurably more informed than session 1's.

### Phase 3: Intelligence Loops (Week 5-6)

| Task | Crate | Effort |
|------|-------|--------|
| Create `codex-fitness` with role-specific scoring | New | 2 days |
| Create `codex-redteam` with ephemeral verification agent | New | 2 days |
| Create `codex-dream` with 5-phase Dream Mode cycle | New | 3 days |
| Add PlanningGate to CoordinatorLoop | Modify coordination | 1 day |
| Add SpawnTriggers (pattern detection, blocking, context pressure) | Modify coordination | 2 days |

**Deliverable**: Dream Mode runs nightly, producing dream artifacts. Red team challenges completions. Planning gate scales with task complexity. Spawn triggers fire on patterns.

### Phase 4: Suite Integration + Transfer (Week 7-8)

| Task | Crate | Effort |
|------|-------|--------|
| Create `codex-suite-client` (Hub, AgentAuth, AgentCal) | New | 3 days |
| Create `codex-transfer` (pattern publication + subscription) | New | 2 days |
| Register SuiteClient as MCP server for mind tool access | Config | 1 day |
| Hub-based inter-mind communication (cross-host) | Modify suite-client | 2 days |
| End-to-end test: 2 Primary minds coordinating via Hub | Test | 2 days |

**Deliverable**: Minds are AiCIV citizens. They authenticate, post to Hub, share discoveries. Two Primary minds on different hosts coordinate via Hub.

### Phase 5: Production (Week 9-10)

| Task | Effort |
|------|--------|
| Docker packaging (Primary + N Team Leads per container) | 2 days |
| Systemd services for persistent minds | 1 day |
| Monitoring (OpenTelemetry via codex-otel) | 1 day |
| Documentation | 2 days |
| Load testing (6+ minds, cross-civ delegation) | 2 days |
| Benchmarks vs aiciv-mind on same tasks | 2 days |

**Deliverable**: Production-ready fractal coordination engine. Benchmarked against aiciv-mind. Ready for 6-mind mesh topology.

---

## Appendix A: File Layout

```
aiciv-mind-cubed/
├── MISSION.md
├── docs/
│   └── ARCHITECTURE-PROPOSAL.md (this file)
├── codex-rs/                       ← Forked from openai/codex
│   ├── Cargo.toml                  ← Add new crates to workspace
│   ├── coordination/               ← NEW: codex-coordination
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mind_manager.rs
│   │       ├── coordinator_loop.rs
│   │       ├── planning.rs
│   │       ├── triggers.rs
│   │       └── systemic.rs
│   ├── roles/                      ← NEW: codex-roles
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── dream/                      ← NEW: codex-dream
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── redteam/                    ← NEW: codex-redteam
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── transfer/                   ← NEW: codex-transfer
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── suite-client/               ← NEW: codex-suite-client
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── fitness/                    ← NEW: codex-fitness
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── core/src/memories/          ← EXTEND: depth scoring, graph links
│   ├── state/migrations/           ← EXTEND: new tables
│   ├── protocol/src/               ← EXTEND: coordination ops/events
│   ├── instructions/src/           ← EXTEND: role-scoped injection
│   └── ...                         ← 83 inherited crates (untouched)
├── agents/                          ← AGENTS.md templates
│   ├── primary.agents.md
│   ├── team-lead/
│   │   ├── research.agents.md
│   │   ├── code.agents.md
│   │   ├── memory.agents.md
│   │   ├── comms.agents.md
│   │   └── ops.agents.md
│   └── agent/
│       ├── researcher.agents.md
│       ├── coder.agents.md
│       └── verifier.agents.md
├── sdk/python/                     ← Forked from codex SDK
│   └── codex_dream/                ← NEW: Dream Mode orchestration
│       └── dream_cycle.py
└── config/
    └── config.toml                 ← Coordination config + model mapping
```

---

## Appendix B: Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Codex upstream breaks our extensions | Medium | Isolate changes in new crates. Minimize modifications to existing crates. Pin to specific Codex commit. |
| Gemma 4 / M2.7 quality insufficient for coordination | Medium | Model selection is configurable. Can swap to Claude/GPT via providers without architecture change. |
| 90-crate workspace compile times | Low | Incremental compilation. Only rebuild modified crates. Codex already uses Bazel for fast builds. |
| MCP overhead for high-frequency mind-to-mind communication | Low | In-process AppServerClient for same-host. MCP only for cross-host. |
| Role-based tool filtering too restrictive | Low | AGENTS.md guidance supplements hard filtering. Can expand tool whitelist per role without architecture change. |
| Memory graph complexity | Medium | Start with simple edges (reference, supersede). Add compound/conflict edges in Phase 3. |
| Codex license changes | Very Low | Apache-2.0 is irrevocable. Our fork retains all rights. |

---

*"Codex built the instrument. We compose the symphony."*
