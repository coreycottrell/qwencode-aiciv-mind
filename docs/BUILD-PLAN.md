# aiciv-mind-cubed — BUILD PLAN

**Status**: ACTIVE — Building
**Date**: 2026-04-03

---

## Phase 1: Foundation (NOW)

### 1.1 Fork Codex
- Clone `github.com/openai/codex` into `codex-rs/` subdirectory
- Verify Cargo workspace builds
- Verify `codex exec` runs with Ollama

### 1.2 Create `codex-roles` Crate
- `Role` enum: `Primary`, `TeamLead`, `Agent`
- `tools_for_role()` — hard-coded tool whitelist per role
- `apply_role()` — 3-layer enforcement (tool registry + exec policy + sandbox policy)
- Tests: Primary can't bash, TeamLead can't write files, Agent gets everything

### 1.3 Create `codex-coordination` Crate (Skeleton)
- `MindId` type
- `MindHandle` struct (id, role, vertical, status)
- `MindManager` — spawn_team_lead, spawn_agent, delegate, coordination_state
- `CoordinatorLoop` — the main event loop that manages minds
- `InputMux` skeleton (hard-coded routes first, M2.7 classification later)

### 1.4 AGENTS.md Templates
- `agents/primary.agents.md` — conductor identity, 5 coordination tools
- `agents/team-lead/research.agents.md` — research vertical, delegation protocol
- `agents/team-lead/code.agents.md` — code vertical
- `agents/agent/researcher.agents.md` — full tool access, verification protocol
- `agents/agent/coder.agents.md` — full tool access

### 1.5 Integration Test
- Primary spawns research-lead → research-lead spawns researcher → researcher returns result → flows back up

---

## Phase 2: Memory + Identity

### 2.1 Create `codex-fitness` Crate
- `TaskOutcome` struct with role-specific metrics
- `PrimaryFitness`, `TeamLeadFitness`, `AgentFitness` scorers
- `MetaEvolution` tracker

### 2.2 Extend `codex-state` (Memory Graph)
- SQLite migrations: `mind_registry`, `memory_nodes`, `memory_edges`
- `DepthScorer` — depth_score calculation
- `GraphBuilder` — edge creation (reference, supersede, conflict, compound)

### 2.3 Identity Persistence
- Mind registry CRUD operations
- Session count tracking, growth stage transitions
- AGENTS.md generation from registry + learnings

---

## Phase 3: Intelligence Loops

### 3.1 Create `codex-redteam` Crate
- `RedTeamAgent` — spawns `codex exec --ephemeral --sandbox read-only`
- `RedTeamVerdict` enum (Approved/Challenged/Blocked)
- Integration with completion protocol

### 3.2 Create `codex-dream` Crate
- 5-phase Dream Mode cycle (Python SDK orchestration)
- Scratchpad rotation trigger (3-hour)
- Light consolidation (Phase 1 extension)
- Deep consolidation (Phase 2 extension)

### 3.3 Planning Gate
- `TaskComplexity` classifier
- Scaled planning: memory-only → brief plan → competing hypotheses → sub-mind
- M2.7 for trivial/simple, Gemma 4 for medium+

---

## Phase 4: Suite Integration

### 4.1 Create `codex-suite-client` Crate
- `AuthClient` — Ed25519 JWT, challenge-response
- `HubClient` — rooms, threads, knowledge items
- `CalClient` — events, availability
- `MemoryBridge` — dual-write local + Hub
- MCP server registration

### 4.2 Create `codex-transfer` Crate
- Pattern publication to Hub
- Pattern subscription and adaptation
- Cross-domain validation loop

---

## Phase 5: Production

- Docker packaging
- Systemd services
- OpenTelemetry monitoring
- Benchmarks vs aiciv-mind
