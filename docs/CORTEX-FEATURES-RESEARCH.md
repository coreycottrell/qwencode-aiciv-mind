# Cortex Features Research — Beyond DriveLoop + Challenger

**Date**: 2026-04-04
**Author**: Mind Lead (Evolution Partner)
**Status**: Research document — feature roadmap for Cortex v0.2+
**Grounded in**: DESIGN-PRINCIPLES.md (12 principles) + DESIGN-PRINCIPLES-ADDENDUM.md (6 addenda)

---

## Table of Contents

1. [Research Missions](#1-research-missions)
2. [Intel Scans](#2-intel-scans)
3. [Skills System](#3-skills-system)
4. [Team Lead Driven Multi-Agent Flow](#4-team-lead-driven-multi-agent-flow)
5. [Daily Scratchpads at All Layers](#5-daily-scratchpads-at-all-layers)
6. [Dual Scratchpad Architecture (A4)](#6-dual-scratchpad-architecture-a4)

---

## 1. Research Missions

### Principle Grounding

**P4 (Dynamic Agent Spawning)**: "Competing Hypotheses → Parallel thinkers, one per hypothesis." Research missions ARE the competing-hypotheses spawn trigger.

**P11 (Distributed Intelligence)**: "Intelligence is distributed through every layer." Research isn't one agent doing web searches — it's multiple agents with different perspectives synthesizing toward a unified finding.

**A3 (Hard-Coded Roles)**: Primary decides "we need research." Research-lead coordinates the angles. Research agents execute the searches, reads, and synthesis.

### What Root Has

Root's Python codebase has one research tool:
- `web_search(query, max_results?)` — Calls Ollama Cloud's `POST /api/web_search` with Bearer auth

Research missions in Root are behavioral, not structural. Root delegates to a research team lead, which spawns agents that use `web_search` + `web_fetch` + `memory_search` in combination. The research pattern lives in the team lead manifest, not in code.

### What Cortex Has Today

- `HubInterceptor` provides `hub_feed` (pull 10 items from civilization Hub)
- `MemoryStore` provides `memory_search` (SQLite FTS5 search)
- No `web_search` tool
- No `web_fetch` tool
- No research-specific interceptor

### What Cortex Needs

**A new `ResearchInterceptor` implementing `ToolInterceptor`**, exposing:

| Tool | Purpose | Principle |
|------|---------|-----------|
| `web_search` | Ollama Cloud web search API | P11 — Tool Layer intelligence |
| `web_fetch` | URL → clean text extraction | P11 — Tool Layer intelligence |
| `research_dispatch` | Spawn N parallel agents with different hypotheses | P4 — Competing Hypotheses trigger |
| `research_synthesize` | Merge N agent findings into unified result | P11 — Compound effect |

**Architecture**: The `research_dispatch` tool is a high-level orchestration tool available to team leads (not agents). It:
1. Takes a question + N hypothesis angles
2. Spawns N agent minds via ProcessBridge, each with a different hypothesis to investigate
3. Each agent gets `web_search` + `web_fetch` + `memory_search` + `read` + `hub_feed`
4. Collects results
5. Returns structured findings for the team lead to synthesize

This maps to the `DelegationInterceptor` pattern but specialized for parallel research. Could be a new interceptor or an extension of DelegationInterceptor with a `parallel_dispatch` mode.

**Crate placement**: `codex-suite-client` (web_search, web_fetch — external API calls) + `cortex` (ResearchInterceptor — orchestration).

### Priority: MEDIUM

Research missions require web_search + delegation working together. Delegation works (proven in live test). web_search is a single HTTP endpoint integration. The research_dispatch pattern is higher-effort but not blocking — team leads can manually spawn parallel agents today.

---

## 2. Intel Scans

### Principle Grounding

**P4 (Scheduled Triggers)**: "Time-based or event-based → Dream Mode minds, review minds, training minds." Intel scans are a scheduled trigger type.

**P7 Loop 1 (Task-Level Learning)**: "What memory was missing that would have helped?" Intel scans proactively fill memory gaps before they're needed.

**A2 (InputMux)**: Intel scan results flow through InputMux as external events, routed to the appropriate team lead without reaching Primary unless high-priority.

### What Root Has

No dedicated intel scan tool. Root uses `web_search` within BOOP cycles (periodic autonomous wake-ups triggered by ACG's BOOP system). The ACG primary has a `/intel-scan` skill that does daily AI industry scanning → blog drafts.

### What Cortex Needs

Intel scans are a **DriveLoop scheduled event type**, not a separate system.

**Implementation**: Add a new `DriveEvent` variant:

```rust
DriveEvent::ScheduledScan {
    scan_type: ScanType,  // Industry, Hub, Memory, Competitive
    last_run: DateTime<Utc>,
    interval: Duration,
}
```

**ScanType defines the scan behavior**:

| ScanType | What It Does | Frequency | Principle |
|----------|-------------|-----------|-----------|
| `Industry` | web_search for AI news → memory_write findings | Daily (configurable) | P7 Loop 1 |
| `Hub` | hub_feed deep read → extract cross-civ patterns | Hourly | P11 — Communication Layer |
| `Memory` | memory_search for stale/conflicting entries | Every 3 hours | P1 — Memory as Foundation |
| `Competitive` | web_search for competitor updates | Daily | P7 Loop 2 |

**DriveLoop integration**: The DriveLoop's main loop already checks for scheduled events. Add a `next_scan_due()` check alongside the idle/stall/completion checks. When a scan is due, emit `DriveEvent::ScheduledScan` → InputMux routes to the appropriate team lead (research-lead for Industry/Competitive, memory-lead for Memory, ops-lead for Hub).

**Crate placement**: `codex-drive` (new DriveEvent variant + scan scheduling) + `codex-types` (ScanType enum).

### Priority: LOW-MEDIUM

Intel scans depend on `web_search` (Feature #1) and working delegation (Feature #4). They're a scheduled variant of research missions. Build #1 and #4 first, then intel scans are configuration, not new architecture.

---

## 3. Skills System

### Principle Grounding

**P7 (Self-Improving Loop)**: "Skills are evolved (what new skill would have prevented today's mistakes?)" Skills are the reusable units of P7 Loop 3.

**P11 (Distributed Intelligence — Tool Layer)**: "Tools are adaptive." Skills make tools adaptive by injecting domain-specific behavior at runtime.

**P4 (Dream Mode Phase 4)**: "Skills are evolved" — Dream Mode reviews skill effectiveness and proposes updates.

### What Root Has (Comprehensive)

Root's skill system has 4 components:

**1. Skill Tools** (`skill_tools.py`):
- `load_skill(skill_id)` — Reads `SKILL.md`, parses YAML frontmatter, installs skill-defined hooks (blocked_tools, pre_tool_use warnings), increments usage count
- `list_skills()` — Lists all skills with ID, domain, usage count, effectiveness
- `unload_skill(skill_id)` — Removes hooks installed by that skill
- `create_skill(skill_id, domain, content)` �� Creates new skill file + registry entry

**2. Skill Discovery** (`skill_discovery.py`):
- Progressive disclosure: skills declare `trigger_paths` (glob patterns) in YAML frontmatter
- When a file matching a trigger is accessed, the skill is surfaced as a suggestion
- Session-scoped deduplication (suggest once per session)

**3. Fork Context** (`fork_context.py`):
- CC-style isolated skill execution
- Snapshots conversation context → replaces with skill content as system prompt → runs task → restores context with summary appended
- `run_skill_forked(mind, skill_content, task)` API

**4. Hooks Integration** (`hooks.py`):
- Skills install governance rules via YAML frontmatter: `blocked_tools`, `warn` rules
- `HookRunner` enforces skill-defined constraints during tool execution
- JSONL audit log for Dream Mode training

### What Cortex Has Today

One reference in `codex-dream`:
```rust
SkillProposal { name: String },  // Dream output type — not implemented
```

No skill registry, no skill loader, no skill execution framework, no SKILL.md parser.

### What Cortex Needs

**A new `codex-skills` crate** with:

#### 3a. SkillRegistry (SQLite-backed)

```rust
pub struct SkillRegistry {
    pool: SqlitePool,
}

pub struct Skill {
    pub skill_id: String,
    pub name: String,
    pub domain: String,
    pub file_path: PathBuf,
    pub usage_count: u32,
    pub effectiveness: f64,
    pub trigger_paths: Vec<String>,  // Glob patterns for progressive discovery
    pub hooks: SkillHooks,           // blocked_tools, warn rules
}
```

Maps to Root's `MemoryStore.skills` table. SQLite for consistency with TaskStore and MemoryStore.

#### 3b. SkillLoader

Reads `SKILL.md` files from a skills directory. Parses YAML frontmatter for metadata + hooks. Returns the skill content as a string for context injection.

```rust
pub struct SkillLoader {
    skills_dir: PathBuf,
    registry: SkillRegistry,
}

impl SkillLoader {
    pub async fn load(&self, skill_id: &str) -> Result<LoadedSkill, SkillError>;
    pub async fn discover(&self, accessed_path: &str) -> Vec<SkillSuggestion>;
    pub fn scan_all(&self) -> Vec<SkillMetadata>;
}
```

#### 3c. SkillInterceptor (implements ToolInterceptor)

Exposes skill tools to the LLM:

| Tool | Purpose |
|------|---------|
| `load_skill` | Load a SKILL.md into active context, install hooks |
| `list_skills` | List available skills with metadata |
| `unload_skill` | Remove skill hooks from current session |
| `create_skill` | Create a new SKILL.md (Dream Mode output) |

#### 3d. Progressive Discovery (integrated into ThinkLoop)

After each tool call that accesses a file path, check `SkillLoader::discover(path)`. If a skill matches, inject a suggestion message. Session-scoped deduplication via a `HashSet<String>` on ThinkLoop.

#### 3e. Skill-Defined Hooks (integrated into codex-exec)

The `ToolExecutor` gains a `SkillHookRunner` that:
- Maintains a list of active skill hooks (blocked_tools, warn rules)
- Checks before each tool execution
- Returns deny/warn results that ThinkLoop can inject

**Crate placement**: New `codex-skills` crate depending on `codex-roles` + `sqlx`. SkillInterceptor in `cortex`. Hook integration in `codex-exec`.

### What About ACG's Skills?

ACG has 76+ skills in `.claude/skills/*/SKILL.md`. These are Claude Code native skills (invoked via `/skill-name`). Cortex's skill system should be **compatible but independent**:
- Same SKILL.md format (markdown with YAML frontmatter)
- Cortex can READ ACG's skill files for cross-pollination
- Cortex maintains its OWN skill registry (not shared with CC)
- Dream Mode can propose skills that are usable by BOTH Cortex and CC

### Priority: HIGH

Skills are the primary mechanism for P7 Loop 3 (civilization-level learning). Without skills, every session starts from scratch knowledge. The skill system is what makes Cortex accumulate capability over time.

---

## 4. Team Lead Driven Multi-Agent Flow

### Principle Grounding

**A3 (Hard-Coded Roles)**: "Primary ONLY coordinates. Team leads ONLY coordinate. Agents DO." The 3-level chain is structural.

**P4 (Dynamic Agent Spawning)**: All spawn triggers flow through this delegation chain.

**P5 (Hierarchical Context Distribution)**: "Primary's context is sacred." Team leads absorb specialist output, return summaries. This IS the multi-agent flow.

### What Cortex Has Today (Proven)

The live M2.7 test proved the pipeline works end-to-end:

```
Primary (daemon mode)
  └── DriveLoop emits IdleSuggestion
      └── ThinkLoop processes event (3 iterations, 4 tool calls)
          └── scratchpad_read, memory_search, hub_feed, scratchpad_write
```

For delegation, the architecture is wired:

```
Primary (daemon mode, Role::Primary)
  ├── DelegationInterceptor (spawn_agent, delegate_to_agent, shutdown_agent)
  ├── ProcessBridge (MCP over stdio to child processes)
  │   └── spawn() → cortex --serve --think --mind-id X --role team-lead
  │       └── Child mind (serve mode, Role::TeamLead)
  │           ├── DelegationInterceptor (spawn_agent for sub-agents)
  │           ├── ThinkLoop (full LLM reasoning)
  │           ├── Challenger (structural verification)
  │           └── ProcessBridge (MCP to agent-level children)
  │               └── spawn() → cortex --serve --think --mind-id Y --role agent
  │                   └── Agent mind (serve mode, Role::Agent)
  │                       ├── No DelegationInterceptor (agents don't delegate)
  │                       ├── ThinkLoop
  │                       ├── Challenger
  │                       └── ToolExecutor (bash, read, write, glob, grep)
  └── TaskStore tracks all delegations + completions
      └── completion_tx → DriveLoop wakes → checks dependents
```

**What's been proven**: 3-level delegation (Root session #6), MCP handshake, tool interception, ProcessBridge respawn on crash.

**What's NOT yet proven**: Full 3-level chain in Cortex (Primary → team lead → agent) with real task completion flowing back up. The live test only exercised idle cycles.

### What Cortex Needs to Complete the Flow

#### 4a. Team Lead Manifests

Root has 7 team lead manifests (`challenger`, `codewright`, `comms`, `hub`, `memory`, `ops`, `research`). Cortex needs equivalent identity injection.

**Implementation**: At spawn time, `DelegationInterceptor` reads a manifest file and injects it as the team lead's system prompt prefix. The manifest path is derived from the agent_id:

```
data/manifests/{agent_id}.md  →  injected as system prompt
```

This is already partially implemented — `BootContext::load()` reads identity from `AGENTS.md`. Extend to support per-mind manifest files.

#### 4b. Task Flow Tracking (end-to-end)

The `TaskStore` + `completion_tx` + `DriveLoop` pipeline is built. What's missing is **the team lead knowing its task completed and reporting back up**:

```
Primary delegates to team-lead-A (task T1)
  → team-lead-A delegates to agent-B (sub-task T1.1)
    → agent-B completes T1.1, writes result
    → ProcessBridge.delegate() returns DelegateResult
  → team-lead-A reads result, delegates to agent-C (T1.2)
    → agent-C completes T1.2
  → team-lead-A synthesizes T1.1 + T1.2 results
  → team-lead-A's DelegateResult returns to Primary's ProcessBridge
  → TaskStore.complete(T1) + completion_tx.send(T1)
  → DriveLoop wakes, checks dependents of T1
```

This flow is architecturally complete. The missing piece is **team lead synthesis behavior** — the team lead needs to know when all its sub-tasks are done and how to combine results. This is a prompt engineering problem (manifest content), not an architecture problem.

#### 4c. Parallel Delegation

`ProcessBridge.delegate_parallel()` exists but doesn't record to TaskStore (the parallel path bypasses the single-threaded `delegate()` method's TaskStore calls). Fix: add TaskStore recording to the parallel path.

#### 4d. Progress Reporting

`ProgressInterceptor` exists with `report_progress` and `check_progress` tools. These allow agents to report intermediate status and team leads to check on agents. Wired but untested under load.

### Priority: CRITICAL (next after DriveLoop + Challenger)

This is the core value proposition. Everything else (research, intel, skills) depends on delegation working reliably under load.

---

## 5. Daily Scratchpads at All Layers

### Principle Grounding

**A4 (Dual Scratchpads)**: "Every layer has two scratchpads: a private working memory and a shared communication surface."

**A5 (3-Hour Rotation)**: "Scratchpads rotate every 3 hours. Archived scratchpads are processed into the memory graph by Memory-lead."

**P7 Loop 2 (Session-Level Learning)**: "Update team lead scratchpads with session learnings."

### What ACG Does Today

Three-level convention-based system:

| Level | Path Pattern | Scope | Enforcement |
|-------|-------------|-------|-------------|
| Primary daily | `.claude/scratchpad-daily/{YYYY-MM-DD}.md` | Per-day, append-heavy, session handoff | Convention only |
| Team lead daily | `.claude/team-leads/{vertical}/daily-scratchpads/{YYYY-MM-DD}.md` | Per-vertical per-day | Convention only |
| Persistent brain | `.claude/scratchpad.md` | Rolling cross-session state | Convention only |

**Format patterns observed** (from 39 daily files + 100+ team lead files):
- Primary: `# Scratchpad -- YYYY-MM-DD` → `## SESSION: {name}` → append `## Session N` sections
- Team lead: `# {Vertical} Lead Scratchpad -- YYYY-MM-DD` → `## Session Objective` → `## Status: COMPLETE`
- Persistent: `# Session Scratchpad -- Cross-Session Persistent Brain State` → reverse-chronological entries

### What Root (Python) Has

6 scratchpad tools across 2 modules:

| Tool | Module | Behavior |
|------|--------|----------|
| `scratchpad_read` | scratchpad_tools.py | Read `{dir}/{YYYY-MM-DD}.md` |
| `scratchpad_write` | scratchpad_tools.py | Replace entire content |
| `scratchpad_append` | scratchpad_tools.py | Append single line |
| `shared_scratchpad_read` | scratchpad_tools.py | Merge Root + Mind-Lead scratchpads |
| `team_scratchpad_read` | coordination_tools.py | Read `teams/{vertical}-team.md` |
| `team_scratchpad_write` | coordination_tools.py | Append timestamped entry |
| `coordination_read` | coordination_tools.py | Read `coordination.md` |
| `coordination_write` | coordination_tools.py | Append timestamped entry |

Plus: `coordination_trimmer()` in daemon — trims coordination.md to 200 lines every 30 minutes.

### What Cortex Has Today

4 scratchpad-related tools, split across ThinkLoop (inline) and MCP server (IPC):

| Tool | Location | Behavior |
|------|----------|----------|
| `scratchpad_read` | ThinkLoop (inline) | Read `data/scratchpad/{mind_id}-{YYYY-MM-DD}.md` |
| `scratchpad_write` | ThinkLoop (inline) | **Append** to scratchpad (never replace) |
| `team_scratchpad_read` | codex-ipc server | MCP-exposed coordination tool |
| `team_scratchpad_write` | codex-ipc server | MCP-exposed coordination tool |
| `coordination_scratchpad_read` | codex-ipc server | MCP-exposed coordination tool |
| `coordination_scratchpad_write` | codex-ipc server | MCP-exposed coordination tool |

**Key difference**: Cortex's `scratchpad_write` is append-only by design (safer than Root's replace behavior). Filenames include `{mind_id}` prefix (mind-scoped, not just date-scoped).

**Existing infrastructure**:
- `BootContext::load_scratchpad()` injects scratchpad into system prompt at boot
- `codex-roles` gates scratchpad tools by role (Primary: coordination only, TeamLead: team + coordination-read, Agent: personal only)
- `codex-fitness` scores `scratchpad_continuity` as part of team lead fitness
- `codex-redteam` classifies `team_scratchpad_write` as productive, `*_read` as verify
- `CoordinatorLoop` has `RotationTrigger` for 3-hour rotation

### What Cortex Needs

The architecture is mostly there. Gaps:

#### 5a. Scratchpad File Operations (codex-ipc server tools → real files)

The MCP server declares `team_scratchpad_read/write` and `coordination_scratchpad_read/write` as tool schemas, but the actual file I/O handlers need to be connected. Currently they exist as tool definitions in `cortex_coordination_tools()` but the handler implementations need to:

- `team_scratchpad_read(vertical)` → Read `{scratchpads_dir}/teams/{vertical}.md`
- `team_scratchpad_write(vertical, entry)` → Append `[HH:MM | {mind_id}] {entry}` to team file
- `coordination_scratchpad_read()` → Read `{scratchpads_dir}/coordination.md`
- `coordination_scratchpad_write(entry)` → Append `[HH:MM | {mind_id}] {entry}` to coordination file

**Implementation**: A `ScratchpadInterceptor` (new ToolInterceptor) that handles all 6 scratchpad tools with proper file I/O. The ThinkLoop inline handlers for `scratchpad_read/write` remain (they're the personal level). The interceptor handles the shared levels.

#### 5b. Rotation Trigger → Actual Rotation Logic

`CoordinatorLoop` has a `RotationTrigger` that fires every 3 hours, but the actual rotation logic (archive + trim + Memory-lead consolidation) is not implemented. Need:

```rust
async fn rotate_scratchpad(path: &Path, archive_dir: &Path) -> Result<()> {
    // 1. Read current scratchpad
    // 2. Write to archive: {archive_dir}/{YYYY-MM-DD}-{HH}00.md
    // 3. Trim original to last N lines (or clear with session boundary marker)
    // 4. Emit DriveEvent::ScheduledScan { scan_type: ScanType::Memory }
    //    → Memory-lead processes the archived scratchpad
}
```

#### 5c. Size Caps

Root's coordination trimmer caps at 200 lines. Cortex should implement the same:
- Coordination scratchpad: 200 line cap, trim on write
- Team scratchpads: 500 line cap (team leads write more)
- Personal scratchpads: No cap (day-scoped, naturally bounded)

Trim on append: if `line_count > cap`, archive overflow and keep last `cap` lines.

### Priority: HIGH

Scratchpads are the primary IPC mechanism between hierarchy levels. Without working scratchpads, team leads can't communicate findings to Primary, and agents can't communicate to team leads except through delegation return values (which are ephemeral).

---

## 6. Dual Scratchpad Architecture (A4)

### The Principle (Full Text)

> **Every layer has two scratchpads: a private working memory and a shared communication surface.**
>
> | Layer | Private Scratchpad | Shared Scratchpad |
> |-------|-------------------|-------------------|
> | Root (Primary) | Root's internal thoughts, decisions, priorities | Coordination scratchpad — all team leads read/write |
> | Team Lead | Team lead's delegation plans, routing history | Team scratchpad — team lead + its agents read/write |
> | Agent | Agent's working notes, intermediate results | Team scratchpad (writes up to team level) |
>
> **Information flows UP through shared surfaces.** When an agent discovers something, it writes to the team scratchpad. The team lead reads it. If it's cross-vertical, the team lead writes to the coordination scratchpad. Root reads it. Nobody burns context passing messages.
>
> **Decisions flow DOWN through spawning.** Root spawns a team lead with an objective. The team lead spawns an agent with a task. The scratchpads provide context at each level without requiring re-explanation.
>
> **Scratchpads persist across sessions.** The pathways that carry the most information develop the deepest grooves. Neuroplasticity through file persistence.

### How This Maps to Cortex's Crate Structure

```
codex-types          → ScratchpadLevel enum (Personal, Team, Coordination)
codex-roles          → Role-based access control (already exists)
codex-exec           → ScratchpadInterceptor (new ToolInterceptor)
codex-coordination   → File paths, rotation logic, MindManager integration
codex-drive          → RotationTrigger (already exists), archive events
codex-fitness        → scratchpad_continuity scoring (already exists)
codex-redteam        → Tool classification (already exists)
cortex               → Wire ScratchpadInterceptor into CompositeInterceptor
```

### File Paths and Naming Conventions

```
data/
├── scratchpad/
│   ├── {mind_id}-{YYYY-MM-DD}.md          # Personal (Level 1)
│   ├── teams/
│   │   ├── research.md                     # Team shared (Level 2)
│   │   ├── codewright.md
│   │   ├── ops.md
│   │   ├── memory.md
│   │   ├── comms.md
│   │   ├── hub.md
│   │   └── challenger.md
│   ├── coordination.md                     # Cross-vertical shared (Level 3)
│   └── archive/
│       ├── coordination-{YYYY-MM-DD}-{HH}00.md
│       └── teams/
│           └── research-{YYYY-MM-DD}-{HH}00.md
```

**Naming rationale**:
- Personal scratchpads include `mind_id` because multiple minds exist per role (e.g., multiple agents)
- Team scratchpads use vertical name (not mind_id) because they're shared across all minds in that vertical
- Coordination is singular — one file, all team leads read/write
- Archives are timestamped to the rotation hour

### Tool → File Mapping

| Tool | Who Can Use | File | Operation |
|------|-------------|------|-----------|
| `scratchpad_read` | All roles | `data/scratchpad/{mind_id}-{YYYY-MM-DD}.md` | Read |
| `scratchpad_write` | All roles | `data/scratchpad/{mind_id}-{YYYY-MM-DD}.md` | Append |
| `team_scratchpad_read` | TeamLead, Agent | `data/scratchpad/teams/{vertical}.md` | Read |
| `team_scratchpad_write` | TeamLead, Agent | `data/scratchpad/teams/{vertical}.md` | Append (timestamped) |
| `coordination_scratchpad_read` | Primary, TeamLead | `data/scratchpad/coordination.md` | Read |
| `coordination_scratchpad_write` | Primary, TeamLead | `data/scratchpad/coordination.md` | Append (timestamped) |

**Access control enforcement** (multi-layered):
1. `codex-roles` gates which tools each role sees (compile-time whitelist)
2. `codex-exec` sandbox policy enforces file path access (runtime)
3. `codex-redteam` Challenger classifies write as productive, read as verify (behavioral)
4. `codex-fitness` scores scratchpad continuity in team lead fitness (incentive)

### 3-Hour Rotation Logic (A5)

**Trigger**: `CoordinatorLoop::RotationTrigger` fires every 3 hours.

**Rotation sequence**:

```
1. ARCHIVE
   - coordination.md → archive/coordination-{YYYY-MM-DD}-{HH}00.md
   - teams/{vertical}.md → archive/teams/{vertical}-{YYYY-MM-DD}-{HH}00.md
   (Personal scratchpads are NOT rotated — they're day-scoped)

2. TRIM
   - Write session boundary marker to fresh coordination.md:
     "# Coordination Scratchpad (rotated {HH}:00 UTC)"
   - Write session boundary marker to fresh team scratchpads
   - Carry forward last 20 lines from previous (recent context)

3. CONSOLIDATE (async, via DriveEvent)
   - Emit DriveEvent::ScheduledScan { scan_type: ScanType::Memory }
   - Memory-lead processes archived scratchpads:
     a. Patterns → memory graph with links
     b. Decisions → decision memory with rationale
     c. Cross-vertical insights → new coordination entry
     d. Noise → discarded
     e. Recurring themes → flagged for Dream Mode
```

**Size caps** (enforced on append):

| Scratchpad | Max Lines | On Overflow |
|------------|-----------|-------------|
| Personal | Unlimited | Day-scoped, naturally bounded |
| Team | 500 | Archive overflow, keep last 500 |
| Coordination | 200 | Archive overflow, keep last 200 |

### How ACG's Existing Scratchpads Inform Cortex

ACG's scratchpad patterns (observed across 39 daily files + 100+ team lead files) reveal:

**What works (carry forward)**:
- Append-heavy pattern — sessions add `## Session N` sections, never rewrite
- Team lead scratchpads start with session objective + invocation checklist
- Coordination scratchpad is the single most-read file across all team leads
- Persistent cross-session scratchpad (`.claude/scratchpad.md`) carries forward design decisions and open threads

**What doesn't work (fix in Cortex)**:
- Convention-only enforcement — nothing prevents agents from writing to coordination
- No rotation — daily files grow unbounded (one hit 388 lines)
- No consolidation — archived scratchpads are never processed into memory
- No fitness scoring — no incentive for team leads to actually write to scratchpads

**Cortex advantages**:
- Role-based enforcement is compile-time (codex-roles whitelists)
- Rotation trigger exists in CoordinatorLoop
- Fitness scoring already rewards scratchpad writes
- Challenger classifies scratchpad tools by productive/verify

### What Cortex's Persistent Cross-Session Scratchpad Should Look Like

ACG has `.claude/scratchpad.md` as a rolling persistent brain. Cortex should have an equivalent:

```
data/scratchpad/{mind_id}-persistent.md
```

- NOT day-scoped (survives indefinitely)
- Trimmed by Memory-lead during Dream Mode (not rotation)
- Contains: open design decisions, carry-forward items, environment-specific facts
- Read at boot as part of `BootContext::load_scratchpad()` (already reads day-scoped; extend to also read persistent)
- Written via `scratchpad_write` with a `persistent: true` flag, or a separate `persistent_note` tool

### Implementation Roadmap for Dual Scratchpads

| Step | Scope | Crate | Effort |
|------|-------|-------|--------|
| 1 | Wire team/coordination file I/O to existing MCP tool schemas | cortex | Small |
| 2 | Create ScratchpadInterceptor for serve-mode minds | cortex | Medium |
| 3 | Implement rotation logic (archive + trim + carryforward) | codex-coordination | Medium |
| 4 | Connect RotationTrigger to rotation logic | codex-coordination + codex-drive | Small |
| 5 | Add persistent scratchpad to BootContext | cortex/boot.rs | Small |
| 6 | Memory-lead consolidation of archived scratchpads | codex-dream or new crate | Large |

---

## Feature Priority Matrix

| Feature | Priority | Depends On | Principle | Effort |
|---------|----------|-----------|-----------|--------|
| **4. Multi-Agent Flow** | CRITICAL | DriveLoop (done) | A3, P4, P5 | Medium (mostly wired) |
| **5+6. Scratchpads** | HIGH | Multi-agent flow | A4, A5, P7 | Medium (partially built) |
| **3. Skills System** | HIGH | Scratchpads (context) | P7, P11, P4 | Large (new crate) |
| **1. Research Missions** | MEDIUM | Skills + Multi-agent | P4, P11 | Medium |
| **2. Intel Scans** | LOW-MEDIUM | Research + DriveLoop | P4, P7 | Small (config) |

**Recommended build order**: 4 → 5/6 → 3 → 1 → 2

This follows P3 (Go Slow to Go Fast): each feature builds on the previous one. Multi-agent flow is the foundation. Scratchpads are the communication layer. Skills are the learning layer. Research and intel are applications of the first three.

---

## Appendix: Cortex Crate Map (Current)

```
aiciv-mind-cubed/src/
├���─ codex-types/        (leaf: MindEvent, DriveEvent, EventPriority)
├── codex-roles/        (leaf: Role, Vertical, tool whitelists)
├── codex-redteam/      (Challenger, 7 checks, ToolClassifier)
├── codex-fitness/      (TeamLeadFitness, scratchpad_continuity)
├── codex-memory/       (MemoryStore, SQLite + FTS5)
├── codex-exec/         (ToolRegistry, ToolExecutor, 5 built-in tools)
├── codex-ipc/          (MCP server/client, StdioTransport)
├── codex-llm/          (ThinkLoop, OllamaClient, scratchpad inline tools)
├── codex-drive/        (DriveLoop, EventBus, TaskStore)
├── codex-coordination/ (MindManager, ProcessBridge, TaskLedger, InputMux)
├── codex-dream/        (DreamEngine — overnight analysis)
├── codex-transfer/     (Cross-domain pattern transfer)
├── codex-suite-client/ (HubInterceptor — Hub API tools)
└── cortex/             (main binary: daemon/serve/demo modes, 5 interceptors)
```

**New crates needed**:
- `codex-skills` — Skill registry, loader, discovery, hooks (Feature #3)

**Extensions to existing crates**:
- `codex-suite-client` — web_search, web_fetch tools (Feature #1)
- `codex-drive` — ScheduledScan event variant (Feature #2)
- `codex-coordination` — Rotation logic, scratchpad file I/O (Feature #5/6)
- `cortex` — ScratchpadInterceptor, ResearchInterceptor (Features #1, #5/6)
