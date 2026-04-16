# Mind Map: Memory & Persistence Domain

**Owner**: mind-memory
**Crates**: `codex-memory`, `codex-dream`, `codex-transfer`, `cortex-memory`
**Total LOC**: ~5,170 lines across 4 crates (9 source files + 3 migration files)

---

## 1. What Exists

### The Dual Memory System

aiciv-mind has two parallel memory stores. Both use SQLite. Both have FTS5 full-text search. Both implement depth scoring and tiered lifecycle. They are **complementary, not redundant**.

```
                    ┌────────────────────────────────────┐
                    │        DUAL MEMORY SYSTEM          │
                    │                                    │
    ┌───────────────┴───────────┐   ┌───────────────────┴──────────┐
    │     codex-memory          │   │      cortex-memory           │
    │     (Flat Store)          │   │      (Graph Store)           │
    │                           │   │                              │
    │  Tables:                  │   │  Tables:                     │
    │  - memories               │   │  - memories                  │
    │  - memory_links           │   │  - memory_edges              │
    │  - memories_fts           │   │  - memories_fts              │
    │  - sessions               │   │  - sessions (inline CREATE)  │
    │                           │   │                              │
    │  Tiers:                   │   │  Tiers:                      │
    │  Working → Session →      │   │  Working → Validated →       │
    │  LongTerm → Archived      │   │  Archived                    │
    │                           │   │                              │
    │  Link types: 5            │   │  Link types: 4               │
    │  (cites, builds_on,       │   │  (cites, builds_on,          │
    │   contradicts, supersedes, │   │   supersedes, conflicts)     │
    │   related)                │   │                              │
    │                           │   │  EXTRA: graph traversal,     │
    │  Categories: 6            │   │  contradiction detection,    │
    │  (learning, pattern,      │   │  edge counts, update ops     │
    │   decision, observation,  │   │                              │
    │   error, context)         │   │  Categories: 4               │
    │                           │   │  (learning, pattern,         │
    │  Uses SqlitePoolOptions   │   │  observation, decision)      │
    │  (5 max connections)      │   │                              │
    └───────────────────────────┘   │  Uses SqlitePool::connect    │
                                    └──────────────────────────────┘
```

#### codex-memory (Flat Store) — ~1,100 LOC

**Files**: `lib.rs` (15), `store.rs` (859), `types.rs` (212)
**Migrations**: `001_init.sql` (82), `002_sessions.sql` (18)

The workhorse. Fast keyword search, tiered lifecycle (4 tiers), session persistence. This is where most runtime memory operations happen.

**Key capabilities**:
- `store()` — insert memory with UUID, category, evidence, tier
- `get()` — retrieve by ID, **auto-increments access_count** (usage tracking)
- `search()` — FTS5 full-text OR column-filter search, with relevance scoring
- `link()` — create directed relationships between memories (5 types)
- `cite()` — specialized link: creates Cites edge + boosts depth_score (+0.1, capped at 1.0)
- `promote()` — Working → Session → LongTerm (cannot skip tiers)
- `archive()` — move to Archived tier with timestamp
- `archive_candidates()` — find low-depth memories below threshold
- `start_session()` / `end_session()` / `load_latest_session()` — full session persistence with coordination state JSON
- `boot_count()` — cumulative boot counter across all sessions

**Schema**: `memories` table + `memory_links` table + `memories_fts` virtual table + `sessions` table. 8 indexes for fast queries.

#### cortex-memory (Graph Store) — ~1,200 LOC

**Files**: `lib.rs` (13), `store.rs` (974), `types.rs` (214)
**Migrations**: `001_graph.sql` (72)

The graph layer. Same memory nodes but with **rich graph traversal**: multi-hop walks, contradiction detection, directional edge queries.

**Key capabilities beyond codex-memory**:
- `edge()` — generic edge creation (replaces `link()`)
- `build_on()` / `supersede()` / `flag_conflict()` — semantic shortcuts
- `get_edges()` / `get_edges_by_type()` — edge queries with type filtering
- `traverse()` — **multi-hop graph traversal** with direction control (Outgoing/Incoming/Both), link-type filtering, depth limits
- `find_contradictions()` — query all Conflicts edges with full node data
- `update()` — modify existing memory title/content (codex-memory lacks this)
- `edge_counts()` — analytics: count edges grouped by type

**Schema**: `memories` table + `memory_edges` table + `memories_fts` virtual table. 7 indexes. Sessions table created inline via `CREATE TABLE IF NOT EXISTS`.

#### Differences between the two stores

| Feature | codex-memory | cortex-memory |
|---------|-------------|---------------|
| **Tiers** | 4 (Working/Session/LongTerm/Archived) | 3 (Working/Validated/Archived) |
| **Categories** | 6 (includes Error, Context) | 4 (Learning/Pattern/Observation/Decision) |
| **Link types** | 5 (includes Related) | 4 (no Related) |
| **Graph traversal** | No | Yes (multi-hop, directional) |
| **Contradiction detection** | No | Yes |
| **Memory update** | No | Yes |
| **Edge analytics** | No | Yes (edge_counts) |
| **Connection pooling** | SqlitePoolOptions (5 conn) | SqlitePool::connect (1 conn) |
| **Sessions** | Dedicated migration file | Inline CREATE TABLE |
| **FTS columns** | title, content, category | title, content |

### codex-dream (Dream Engine) — ~750 LOC

**Files**: `lib.rs` (170), `engine.rs` (603)

The offline learning system. Runs a 5-phase nightly cycle that turns raw memories into compound intelligence.

```
    ┌─────────────────── DREAM CYCLE (5 phases) ──────────────────────┐
    │                                                                  │
    │  Phase 1: AUDIT                                                  │
    │  └─ Find memories with depth_score < archive_threshold (0.1)    │
    │  └─ Returns candidates for Phase 2 and 3                        │
    │                                                                  │
    │  Phase 2: CONSOLIDATE                                           │
    │  └─ For each candidate, FTS-search for similar content          │
    │  └─ Create Related links between similar memories               │
    │  └─ Threshold: relevance > 0.3                                  │
    │                                                                  │
    │  Phase 3: PRUNE                                                  │
    │  └─ Archive isolated candidates (no links after consolidation)  │
    │  └─ Spare candidates that gained connections                    │
    │                                                                  │
    │  Phase 4: SYNTHESIZE                                            │
    │  └─ Find LongTerm memories with depth >= 0.3                   │
    │  └─ If a memory has 3+ links → create Pattern memory            │
    │  └─ New pattern cites all source memories                       │
    │  └─ Optional LLM-powered synthesis (Ollama/Gemma4)              │
    │                                                                  │
    │  Phase 5: REPORT                                                │
    │  └─ Summary: audited/consolidated/pruned/synthesized counts     │
    │  └─ DreamReport IS the artifact                                 │
    └──────────────────────────────────────────────────────────────────┘
```

**Key types**:
- `DreamConfig` — start_time (01:00), end_time (04:00), archive_threshold (0.1), model ("gemma4")
- `DreamCycle` — 5 phases (Review, PatternSearch, DeliberateForgetting, SelfImprovement, DreamArtifacts)
- `DreamFinding` — findings with typed variants (Pattern, ArchiveCandidate, Contradiction, ManifestEvolution, RoutingUpdate, SkillProposal, TransferOpportunity)
- `DreamEngine` — operates on codex-memory's `MemoryStore`, optional `OllamaClient` for LLM synthesis

**Dependency**: `codex-memory` (for MemoryStore), `codex-llm` (for OllamaClient — optional)

### codex-transfer (Cross-Domain Transfer) — ~120 LOC

**Files**: `lib.rs` (154)

The knowledge-sharing layer. When one mind discovers a useful pattern, it gets shared to other minds or other civilizations.

```
    ┌──────────── TRANSFER LIFECYCLE ──────────────┐
    │                                               │
    │  1. DISCOVER — mind finds a useful pattern    │
    │  2. PUBLISH — pattern registered with scope   │
    │     - Own (private to this mind)              │
    │     - Civ (all minds in this civilization)    │
    │     - Public (all civs — human approval req)  │
    │  3. SEARCH — other minds query for patterns   │
    │  4. ADAPT — pattern applied to new domain     │
    │  5. VALIDATE — success tracked                │
    └───────────────────────────────────────────────┘
```

**Key types**:
- `TransferPattern` — source_mind, context, pattern content (description + applicability + technique), evidence, confidence level, share scope
- `TransferEngine` — in-memory registry with `publish()`, `search()`, `count_by_scope()`
- `AdaptedPattern` — original_id, adapted_by, adaptation text, validation status (Untested/Testing/Validated)
- `ShareScope` — Own/Civ/Public (human-governed escalation)

**Current state**: Minimal. In-memory only (no persistence). Ready for integration with memory stores.

---

## 2. How Memory Persists Across Sessions

```
    SESSION START                    SESSION ACTIVE                  SESSION END
    ────────────                    ──────────────                  ───────────

    load_latest_session()           store() / get() / search()     end_session()
         │                               │                              │
         ▼                               ▼                              ▼
    ┌──────────┐                  ┌─────────────┐               ┌──────────────┐
    │ sessions │  coordination    │  memories   │  new entries  │   sessions   │
    │ table    │──state JSON───>  │  table      │───stored───>  │   table      │
    │          │  + boot_count    │             │               │   updated:   │
    │          │                  │  memory_    │               │  - status    │
    │          │                  │  links/edges│               │  - coord JSON│
    │          │                  │             │               │  - mem count │
    └──────────┘                  └─────────────┘               └──────────────┘
```

### The Persistence Chain

1. **Boot**: `start_session(notes)` — creates session record, increments boot_count
2. **Restore**: `load_latest_session()` — retrieves last completed session's `coordination_state_json`
3. **Operate**: memories stored/searched/cited/linked throughout session
4. **Save**: `end_session(session_id, coordination_state_json)` — persists:
   - Coordination state as JSON blob (MindManager state, active tasks, etc.)
   - Memory count at session end
   - Completion timestamp
5. **Crash recovery**: sessions with status `active` (never `completed`) indicate crashes

### What the coordination_state_json contains (contract with mind-coordination)

This is the serialized state of `codex-coordination`'s `CoordinationState` — it includes:
- Active minds and their roles
- Task ledger state
- Process bridge connections
- Any pending triggers

The memory system is **agnostic** about this JSON — it just stores and restores it. The coordination module owns the schema.

### Boot Count Tracks Identity Continuity

Each session increments `boot_count` via `COALESCE(MAX(boot_count), 0) + 1`. This gives every incarnation of the mind a sequence number — boot #1, boot #47, boot #300. This counter IS the thread of identity across ephemeral invocations.

---

## 3. How Memory Compounds (The Key Differentiator)

Memory that compounds is what makes aiciv-mind fundamentally different from a stateless LLM invocation. Here's the compounding mechanism:

```
    THE COMPOUNDING LOOP
    ════════════════════

    ┌──────────────────────────────────────────────────────────────────┐
    │                                                                  │
    │  1. STORE: New memory created (depth = 0.0)                     │
    │      │                                                           │
    │  2. CITE: Other memories reference it                            │
    │      │    depth += 0.1 per citation (capped at 1.0)             │
    │      │    citation_count increments                              │
    │      │                                                           │
    │  3. ACCESS: Queries read it                                      │
    │      │    access_count increments                                │
    │      │                                                           │
    │  4. LINK: Dream engine finds connections                         │
    │      │    Related/BuildsOn/Supersedes edges created              │
    │      │                                                           │
    │  5. PROMOTE: High-value memories advance tiers                   │
    │      │    Working → Session → LongTerm (codex-memory)           │
    │      │    Working → Validated (cortex-memory)                    │
    │      │                                                           │
    │  6. SYNTHESIZE: Dream engine creates Pattern memories            │
    │      │    3+ connected memories → new Pattern (LongTerm tier)    │
    │      │    Pattern cites all sources → sources gain depth         │
    │      │                                                           │
    │  7. TRANSFER: Patterns shared across minds/civs                  │
    │      │    Own → Civ → Public (human-governed escalation)        │
    │      │                                                           │
    │  8. PRUNE: Uncited/unlinked memories archived                    │
    │      │    Isolated low-depth memories fade                       │
    │      │                                                           │
    │  ↓ Loop continues: synthesized patterns get cited, gaining       │
    │    depth, triggering further synthesis → COMPOUND GROWTH         │
    └──────────────────────────────────────────────────────────────────┘
```

### The Three Compounding Mechanisms

#### A. Depth Scoring (automatic)
- Every citation adds +0.1 to depth_score (capped at 1.0)
- Every `get()` increments access_count
- Memories used often become foundational; memories ignored fade
- `depth_score` and `access_count` are the raw signals of value

#### B. Dream Engine Synthesis (nightly)
- Phase 4 finds LongTerm memories with depth >= 0.3 and 3+ links
- Creates a NEW Pattern memory that synthesizes the cluster
- Pattern cites all sources → source depth increases
- **Self-reinforcing**: patterns that cite good memories make those memories even deeper, making them more likely to be found and cited again

#### C. Cross-Domain Transfer (inter-mind)
- Patterns discovered in one domain get shared to others
- Human-governed scope escalation (Own → Civ → Public)
- Other minds adapt patterns to their domain
- Validation feedback loop (Untested → Testing → Validated)

### What Makes It Compound vs. Just Accumulate

The critical insight: **citation creates a positive feedback loop**.

```
    Memory A cited by Memory B
         → A.depth increases
         → A appears higher in search results
         → A more likely to be found and cited again
         → A's citations compound
         → Dream engine synthesizes A into Pattern P
         → P cites A → A.depth increases FURTHER
         → P also appears in search → P gets cited
         → P's citations compound → P gets synthesized into Pattern P2
         → EXPONENTIAL KNOWLEDGE GROWTH
```

Without this: each session is isolated, patterns are rediscovered N times.
With this: the civilization gets **measurably smarter** with each session.

---

## 4. Dependencies on Other Modules

```
    ┌──────────────┐     ┌─────────────┐     ┌────────────────┐
    │  codex-llm   │     │ codex-types  │     │ codex-drive    │
    │ (OllamaClient│     │ (MindId,     │     │ (SessionId,    │
    │  for dream   │     │  Role, etc.) │     │  transcripts   │
    │  synthesis)  │     │              │     │  for dream)    │
    └──────┬───────┘     └──────┬───────┘     └───────┬────────┘
           │                    │                      │
           │    OPTIONAL        │   TYPE DEFS          │  SESSION DATA
           ▼                    ▼                      ▼
    ┌──────────────────────────────────────────────────────────────┐
    │                     MEMORY DOMAIN                            │
    │                                                              │
    │  codex-memory ◄──────── codex-dream ───────► codex-llm     │
    │  (flat store)    reads/writes    (dream engine)  synthesis   │
    │       │                  │                                    │
    │       │                  │                                    │
    │       │          codex-transfer                              │
    │       │          (pattern sharing)                            │
    │       │                                                      │
    │  cortex-memory                                               │
    │  (graph store — independent, no internal cross-dependency)   │
    └──────────────────────────────────────────────────────────────┘
```

### Direct Dependencies (compile-time)

| Crate | Depends On | For |
|-------|-----------|-----|
| `codex-memory` | `sqlx`, `chrono`, `uuid`, `serde`, `serde_json`, `thiserror`, `tracing` | SQLite, time, IDs, serialization |
| `cortex-memory` | `sqlx`, `chrono`, `uuid`, `serde`, `serde_json`, `thiserror`, `tracing` | Same as above |
| `codex-dream` | **`codex-memory`** (MemoryStore), **`codex-llm`** (OllamaClient) | Store access + LLM synthesis |
| `codex-transfer` | `chrono`, `uuid`, `serde` | Timestamps, IDs, serialization |

### Interface Dependencies (runtime)

| Needed From | Needed By Memory | What |
|-------------|-----------------|------|
| `codex-types` (mind-coordination) | memory stores | MindId, SessionId, MemoryCategory — currently **string-typed**, will need codex-types alignment |
| `codex-drive` (mind-model-router) | dream engine | Session transcripts as raw input for pattern extraction (Phase 4) |
| `codex-llm` (mind-model-router) | dream engine | OllamaClient for LLM-powered synthesis (optional, falls back to template) |
| `codex-coordination` (mind-coordination) | session persistence | CoordinationState JSON blob for `end_session()` / `load_latest_session()` |

### What Memory Provides to Other Modules

| Consumer | What Memory Provides |
|----------|---------------------|
| `codex-drive` / `cortex` (mind-coordination) | `store()`, `search()`, `get()` — runtime memory ops |
| `codex-coordination` (mind-coordination) | `start_session()`, `end_session()`, `load_latest_session()` — session persistence |
| Any module needing context | `search()` with FTS5 — find relevant prior knowledge |
| `cortex` boot sequence | `load_latest_session()` → restore coordination state |

---

## 5. Recommended Build Order

### Phase 1: Verify Foundation (Current — Phase 1 from MISSIONS.md)

```
    1. codex-memory       — verify compiles, run tests (17 tests exist)
    2. cortex-memory      — verify compiles, run tests (12 tests exist)
    3. codex-transfer     — verify compiles, run tests (2 tests exist)
    4. codex-dream        — verify compiles, run tests (5 tests exist)
       └─ depends on codex-memory + codex-llm (both must compile first)
```

### Phase 2: Resolve Dual-Store Divergence

The two stores have drifted. Key unification tasks:

```
    1. ALIGN TIER MODELS
       codex-memory:  Working → Session → LongTerm → Archived (4 tiers)
       cortex-memory: Working → Validated → Archived (3 tiers)
       → Decision needed: unify or keep separate? (ask mind-lead)

    2. ALIGN CATEGORY MODELS
       codex-memory:  6 categories (includes Error, Context)
       cortex-memory: 4 categories (Learning, Pattern, Observation, Decision)
       → Decision needed: standardize via codex-types? (ask mind-coordination)

    3. ALIGN LINK TYPES
       codex-memory:  5 types (includes Related)
       cortex-memory: 4 types (no Related)
       → Recommendation: add Related to cortex-memory

    4. MERGE OR INTEGRATE
       Both stores have independent SQLite databases
       → Phase 3 decision: single DB with both tables, or keep separate?
```

### Phase 3: Integrate Dream Engine with Graph Store

Currently `codex-dream` only uses `codex-memory` (flat store). The graph store's traversal and contradiction detection would make dream cycles vastly more intelligent.

```
    1. Make DreamEngine generic over store (trait-based)
    2. Use cortex-memory's traverse() for consolidation (Phase 2)
    3. Use cortex-memory's find_contradictions() for conflict detection
    4. Use cortex-memory's edge_counts() for synthesis analytics
```

### Phase 4: Persist codex-transfer

Currently in-memory only. Connect to SQLite for durable pattern sharing.

```
    1. Add SQLite migration for transfer_patterns + adapted_patterns tables
    2. Integrate with dream engine (Phase 4 already discovers TransferOpportunity findings)
    3. Connect to Hub for cross-civilization pattern sharing
```

### Phase 5: Self-Improvement Loop (Phase 4 from MISSIONS.md)

```
    1. Dream engine runs on real session transcripts (needs codex-drive integration)
    2. Transfer engine publishes dream-discovered patterns to civ-wide memory
    3. Fitness scoring (mind-testing) tracks memory quality over time
    4. Dynamic agent spawning triggered by memory pattern detection
```

---

## 6. Key Design Decisions & Observations

### What's Working Well
- **Depth scoring** is elegant: +0.1 per citation, capped at 1.0, creates natural signal
- **FTS5 integration** with sync triggers means search is always current
- **Session persistence** with coordination state JSON provides clean separation of concerns
- **Dream engine's 5-phase design** is sound: audit → consolidate → prune → synthesize → report
- **Test coverage** is solid: 36+ tests across the 4 crates, covering core behaviors

### What Needs Attention
- **Two stores, divergent types** — the biggest architectural debt. MemoryCategory, MemoryTier, and LinkType all have different variants between codex-memory and cortex-memory
- **codex-transfer has no persistence** — patterns disappear when process dies
- **Dream engine only uses flat store** — missing graph traversal for smarter consolidation
- **No shared type definitions** — memory types are defined locally per crate instead of in codex-types
- **cortex-memory sessions table** created inline (`CREATE TABLE IF NOT EXISTS` in code) rather than in migration — fragile

### Architectural Principle

Memory IS Architecture (Principle 1). The memory system is not a feature — it is the substrate on which identity and intelligence compound. Every other module (coordination, tools, LLM routing) produces ephemeral outputs. Memory is what makes those outputs persistent, searchable, and compounding. Without memory, each session is a fresh stateless invocation. With memory, each session builds on every session before it.
