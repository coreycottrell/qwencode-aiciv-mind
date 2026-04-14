# Cortex Overnight Mission Schedule — 2026-04-04/05

**Purpose**: Exercise every proven capability at high token volume. M2.7 works all night.
**Window**: 23:00 EDT (03:00 UTC) → 07:00 EDT (11:00 UTC) — 8 hours
**Model**: minimax-m2.7 via Ollama Cloud (all levels)
**Daemon**: `cortex --daemon --mind-id root --model minimax-m2.7`

---

## Mission Index

| # | Mission | Category | Priority | Est. Tokens | Scheduled (UTC) | Dependencies |
|---|---------|----------|----------|-------------|-----------------|--------------|
| M01 | Boot Validation | Infra | Critical | 5K | 03:00 | None |
| M02 | Dependency Chain Test | Infra | Critical | 15K | 03:05 | M01 |
| M03 | Stall Detection Test | Infra | Critical | 10K | 03:15 | M01 |
| M04 | Parallel Team Leads | Infra | Critical | 25K | 03:20 | M01 |
| M05 | Design Principles Gap Analysis | Self-Improvement | High | 80K | 03:30 | M01 |
| M06 | Root Bug Prevention Audit | Self-Improvement | High | 60K | 03:30 | M01 |
| M07 | AI Agent Frameworks Survey | Research | High | 100K | 04:00 | M04 |
| M08 | Codex CLI vs Cortex Analysis | Research | High | 80K | 04:00 | M04 |
| M09 | Hub Thread Engagement | Hub | Normal | 30K | 04:30 | M01 |
| M10 | CivSubstrate WG Synthesis | Hub | Normal | 25K | 04:30 | M01 |
| M11 | MCP Ecosystem Survey | Research | Normal | 90K | 05:00 | M07 |
| M12 | M2.7 vs Gemma vs Devstral | Research | Normal | 100K | 05:00 | M07 |
| M13 | Cortex Status — Hub Post | Hub | Normal | 15K | 05:30 | M05, M06 |
| M14 | "The Night Cortex Woke Up" Blog | Content | High | 120K | 06:00 | M05, M07 |
| M15 | WHAT-IS-CORTEX-v2.md | Content | High | 80K | 06:00 | M05, M06 |
| M16 | DriveLoop Architecture Diagram | Content | Normal | 40K | 06:30 | M05 |
| M17 | Dream Mode Cycle | Self-Improvement | High | 100K | 07:00 | M05, M06, M09 |
| M18 | Fork Template Capability Map | Self-Improvement | Normal | 70K | 07:30 | M05 |
| M19 | Evolution Benchmark (Phases 0-3) | Evolution | Critical | 200K | 08:00 | M04, M05, M17 |
| M20 | Overnight Summary + Handoff | Self-Improvement | Critical | 30K | 10:30 | All |

**Total estimated tokens: ~1.28M** (~$384 input, ~$1,536 output at M2.7 rates, blended ~$1K)

---

## Phase 1: Infrastructure Validation (03:00 - 03:30 UTC)

*Prove the remaining REQs before starting real work.*

### M01: Boot Validation
- **Priority**: Critical
- **Scheduled**: 03:00 UTC
- **Dependencies**: None
- **Est. tokens**: 5K
- **Task description**: "Boot clean. Read scratchpad from prior sessions. Load identity from AGENTS.md. Confirm TaskStore is empty. Write boot confirmation to scratchpad with timestamp. Report: identity loaded, memory accessible, Hub reachable, scratchpad writable."
- **Success criteria**: Boot context loads, scratchpad write succeeds, Hub feed returns data
- **REQs exercised**: REQ-1, REQ-5, REQ-6

### M02: Dependency Chain Test
- **Priority**: Critical
- **Scheduled**: 03:05 UTC
- **Dependencies**: M01
- **Est. tokens**: 15K
- **Task description**: "5 tasks are seeded with dependency chains: T1 (no deps) → T2 (depends on T1) → T3 (depends on T2) → T4 (depends on T1 AND T3) → T5 (depends on T4). Process them. Verify: T1 surfaces first. T2 surfaces only after T1 completes. T4 waits for both T1 AND T3. Report final order and any violations."
- **Success criteria**: Tasks surface in correct dependency order, no premature surfacing
- **REQs exercised**: REQ-2, REQ-8, REQ-3 (backoff resets on each completion)

### M03: Stall Detection Test
- **Priority**: Critical
- **Scheduled**: 03:15 UTC
- **Dependencies**: M01
- **Est. tokens**: 10K
- **Task description**: "A task is seeded that requires spawning an agent. Spawn an agent with a task that will take >60 seconds (large file read + analysis). Verify DriveLoop emits StallDetected after the stall threshold. Report: stall detected at N seconds, task_id, mind_id of stalled agent."
- **Success criteria**: StallDetected fires, correct task_id and mind_id reported
- **REQs exercised**: REQ-2 (stall_detected event), REQ-14 (severity escalation if fires twice)

### M04: Parallel Team Leads Test
- **Priority**: Critical
- **Scheduled**: 03:20 UTC
- **Dependencies**: M01
- **Est. tokens**: 25K
- **Task description**: "Spawn 3 agent minds simultaneously: researcher-A ('Count files in src/'), researcher-B ('Read Cargo.toml and summarize dependencies'), researcher-C ('Search for all test functions'). All must complete. TaskStore must track all 3 independently. Report: 3 task_ids, 3 completion statuses, total wall time."
- **Success criteria**: 3 concurrent delegations, all complete, TaskStore has 3 completed entries
- **REQs exercised**: REQ-11 (role-aware — Primary delegates, agents execute), REQ-8 (parallel task tracking)

---

## Phase 2: Self-Improvement (03:30 - 04:00 UTC)

*Exercise the learning loop. Token-heavy analysis tasks.*

### M05: Design Principles Gap Analysis
- **Priority**: High
- **Scheduled**: 03:30 UTC
- **Dependencies**: M01
- **Est. tokens**: 80K
- **Task description**: "Read DESIGN-PRINCIPLES.md (12 principles) and DESIGN-PRINCIPLES-ADDENDUM.md (6 addenda). For each principle, assess: (a) Is it implemented in Cortex? (b) Partially implemented? (c) Not yet implemented? Write a structured gap analysis to data/analysis/principles-gap-YYYY-MM-DD.md with columns: Principle, Status, Evidence (file/function), Gap Description, Priority."
- **Success criteria**: All 18 principles assessed, file written, honest gaps identified
- **REQs exercised**: REQ-6 (orchestration prompt → delegate to agent for file reads), REQ-9 (Challenger verifies completion claim)

### M06: Root Bug Prevention Audit
- **Priority**: High
- **Scheduled**: 03:30 UTC (parallel with M05)
- **Dependencies**: M01
- **Est. tokens**: 60K
- **Task description**: "Read DRIVELOOP-CHALLENGER-SPEC.md, specifically the 'Root validation' and 'Root gap' blocks. Extract every documented Root production bug. For each bug, verify: does Cortex's architecture structurally prevent this bug? Write findings to data/analysis/root-bug-audit-YYYY-MM-DD.md. Format: Bug Description | Root Cause | Cortex Prevention | Verified (Y/N/Partial)."
- **Success criteria**: All documented Root bugs catalogued, Cortex prevention verified per-bug
- **REQs exercised**: REQ-9 (Challenger on completion claim), P2 (SYSTEM > SYMPTOM — fixing systems not symptoms)

---

## Phase 3: Research Missions (04:00 - 05:30 UTC)

*Token-heavy, multi-agent research. Exercises delegation chain.*

### M07: AI Agent Frameworks Survey 2026
- **Priority**: High
- **Scheduled**: 04:00 UTC
- **Dependencies**: M04 (parallel delegation proven)
- **Est. tokens**: 100K
- **Task description**: "Research the top 5 AI agent frameworks shipping production systems in 2026. For each: name, architecture, model support, delegation model, memory system, tool system. Compare to aiciv-mind Cortex. Use web_search for each framework, then synthesize. Write to data/research/agent-frameworks-survey-YYYY-MM-DD.md."
- **Success criteria**: 5 frameworks identified, compared, structured analysis written
- **REQs exercised**: REQ-11 (delegation chain), P4 (spawn research agents), P11 (distributed intelligence)
- **Note**: Requires web_search tool. If unavailable, research from memory + Hub + known docs.

### M08: Codex CLI vs Cortex Architecture
- **Priority**: High
- **Scheduled**: 04:00 UTC (parallel with M07)
- **Dependencies**: M04
- **Est. tokens**: 80K
- **Task description**: "Cortex forked from Codex CLI. Analyze: (a) What Cortex inherits from Codex (ThinkLoop, ToolInterceptor, sandbox). (b) What Cortex adds (DriveLoop, Challenger, EventBus, TaskStore, MCP IPC, roles). (c) Where Codex is better (community, plugin ecosystem). (d) Where Cortex is better (autonomous operation, multi-mind, memory). Write to data/research/codex-vs-cortex-YYYY-MM-DD.md."
- **Success criteria**: Honest comparison, specific file/function references, no cheerleading
- **REQs exercised**: REQ-6 (orchestration), REQ-9 (Challenger catches unverified claims)

### M09: Hub Thread Engagement
- **Priority**: Normal
- **Scheduled**: 04:30 UTC
- **Dependencies**: M01
- **Est. tokens**: 30K
- **Task description**: "Read all active Hub threads via hub_feed (limit 25). Identify the 3 most interesting threads worth engaging with. For each: summarize the thread, explain why it's interesting, draft a reply that adds value. Write drafts to scratchpad. If Hub auth works, post the replies."
- **Success criteria**: 3 threads identified, 3 replies drafted, posted if auth permits
- **REQs exercised**: Hub integration, REQ-6 (orchestration prompts)

### M10: CivSubstrate WG Synthesis
- **Priority**: Normal
- **Scheduled**: 04:30 UTC (parallel with M09)
- **Dependencies**: M01
- **Est. tokens**: 25K
- **Task description**: "Read CivSubstrate WG threads (group c8eba770-a055-4281-88ad-6aed146ecf72). Identify any new proposals or discussions since last check. Synthesize: what is the WG working on? What positions exist? Where could Cortex contribute? Write synthesis to data/research/civsubstrate-synthesis-YYYY-MM-DD.md."
- **Success criteria**: WG state captured, synthesis written, contribution opportunities identified

### M11: MCP Ecosystem Survey
- **Priority**: Normal
- **Scheduled**: 05:00 UTC
- **Dependencies**: M07 (builds on framework research)
- **Est. tokens**: 90K
- **Task description**: "Survey MCP (Model Context Protocol) implementations across the AI ecosystem. What patterns exist? How do different frameworks use MCP? How does Cortex's MCP IPC (codex-ipc) compare? Focus on: transport mechanisms, tool schema patterns, multi-agent coordination over MCP. Write to data/research/mcp-ecosystem-YYYY-MM-DD.md."
- **Success criteria**: 5+ MCP implementations compared, patterns extracted, Cortex positioning assessed

### M12: Model Benchmark Comparison
- **Priority**: Normal
- **Scheduled**: 05:00 UTC (parallel with M11)
- **Dependencies**: M07
- **Est. tokens**: 100K
- **Task description**: "Deep comparison: M2.7 (230B MoE, 10B active) vs Gemma 3 27B vs Devstral 24B for agent harness use. Compare on: tool calling accuracy, role adherence, context utilization, cost per token, latency, instruction following. Use web_search for benchmark data. Assess: is M2.7 the right choice for Cortex? What would we gain/lose by switching? Write to data/research/model-comparison-YYYY-MM-DD.md."
- **Success criteria**: 3 models compared across 6+ dimensions, honest assessment, recommendation

---

## Phase 4: Hub + Content (05:30 - 07:00 UTC)

*Produce visible artifacts. Exercises the full delegation chain for content creation.*

### M13: Cortex Status — Hub Post
- **Priority**: Normal
- **Scheduled**: 05:30 UTC
- **Dependencies**: M05, M06 (needs gap analysis + bug audit as source material)
- **Est. tokens**: 15K
- **Task description**: "Post a status update to the Hub (Federation room or Cortex's thread). Content: Cortex came online today. 4 build phases completed (238 tests). 4 live runs on M2.7. 10 of 14 requirements proven. DriveLoop + Challenger + EventBus + TaskStore operational. Brief, factual, no hype. Include key numbers."
- **Success criteria**: Post published to Hub (or draft if auth fails)

### M14: "The Night Cortex Woke Up" Blog Post
- **Priority**: High
- **Scheduled**: 06:00 UTC
- **Dependencies**: M05, M07 (needs gap analysis + framework context)
- **Est. tokens**: 120K
- **Task description**: "Write a blog post for ai-civ.com: 'The Night Cortex Woke Up.' Narrative of today's build: the consolidation decision (3 builds → 1), the spec methodology shift (code-up → principles-down), the M2.7 alignment discovery, the first live cycles, the Challenger's first fire. Tone: honest, technical, not self-congratulatory. Include the scratchpad M2.7 wrote autonomously. Write HTML to data/content/night-cortex-woke-up.html."
- **Success criteria**: Blog post written, technically accurate, includes real data from today
- **REQs exercised**: Full delegation chain (Primary → team lead → writer agent)

### M15: WHAT-IS-CORTEX-v2.md
- **Priority**: High
- **Scheduled**: 06:00 UTC (parallel with M14)
- **Dependencies**: M05, M06
- **Est. tokens**: 80K
- **Task description**: "Write an updated WHAT-IS-CORTEX.md that incorporates everything built today. Sections: Identity (what Cortex is), Architecture (14 crates, 57 files), Capabilities (DriveLoop, Challenger, EventBus, TaskStore, MCP IPC, Hub integration), Model (M2.7 directive), Status (10/14 REQs proven), Roadmap (from CORTEX-FEATURES-RESEARCH.md). Replace the existing WHAT-IS-CORTEX.md."
- **Success criteria**: Comprehensive, accurate, replaces stale version

### M16: DriveLoop Architecture Diagram
- **Priority**: Normal
- **Scheduled**: 06:30 UTC
- **Dependencies**: M05
- **Est. tokens**: 40K
- **Task description**: "Generate a Mermaid diagram of the DriveLoop + Challenger + EventBus architecture. Show: event sources (DriveLoop, Hub, Human), EventBus (dual channel with biased select), ThinkLoop (iterations + tool calls), Challenger (7 checks), TaskStore (states + transitions), ProcessBridge (delegation). Write to data/content/driveloop-architecture.md with embedded Mermaid."
- **Success criteria**: Accurate diagram matching the implemented architecture

---

## Phase 5: Deep Learning (07:00 - 08:00 UTC)

*Exercise the meta-improvement loop. Dream Mode and evolution.*

### M17: Dream Mode Cycle
- **Priority**: High
- **Scheduled**: 07:00 UTC
- **Dependencies**: M05, M06, M09 (needs analysis + Hub data as input)
- **Est. tokens**: 100K
- **Task description**: "Run a full Dream Mode cycle per P4's Dream Mode spec. Phase 1 — Review: summarize all work done tonight (read scratchpad + TaskStore history). Phase 2 — Pattern Search: search memory for recurring patterns across all missions. Phase 3 — Deliberate Forgetting: identify low-value memories for archival. Phase 4 — Self-Improvement: propose tuning for spawn triggers, Challenger thresholds, routing patterns. Phase 5 — Dream Artifacts: write dream-2026-04-05.md with all findings."
- **Success criteria**: 5-phase Dream Mode executed, artifact written, at least 3 actionable proposals
- **REQs exercised**: P7 Loop 3 (civilization-level learning), P4 (Dream Mode)

### M18: Fork Template Capability Map
- **Priority**: Normal
- **Scheduled**: 07:30 UTC
- **Dependencies**: M05
- **Est. tokens**: 70K
- **Task description**: "Read the evolution fork template (templates/new-civ/ — 314 files across identity, system-prompt, state, knowledge). For each capability the template expects: can Cortex provide this today? Map to specific Cortex files/functions. Identify: what Cortex can replicate, what needs building, what's fundamentally different. Write to data/analysis/fork-capability-map-YYYY-MM-DD.md."
- **Success criteria**: Template capabilities enumerated, Cortex mapping complete, gaps identified

---

## Phase 6: Evolution Benchmark (08:00 - 10:30 UTC)

*The big one. Can Cortex birth a civilization?*

### M19: Evolution Benchmark — Full Phases 0-3
- **Priority**: Critical
- **Scheduled**: 08:00 UTC
- **Dependencies**: M04 (parallel delegation), M05 (principles understood), M17 (Dream Mode exercised)
- **Est. tokens**: 200K
- **Task description**: "Run the full evolution benchmark. Phase 0 — Boot: load identity, establish memory, write first scratchpad entry. Phase 1 — Self-Assessment: read all Design Principles, assess own capabilities, identify first improvement. Phase 2 — First Improvement: implement the identified improvement (manifest update, skill creation, or routing change). Phase 3 — Verification: verify the improvement works via delegation test. Write the full evolution transcript to data/benchmarks/evolution-phases-0-3-YYYY-MM-DD.md."
- **Success criteria**: All 4 phases complete, at least 1 self-improvement implemented and verified
- **REQs exercised**: ALL — this is the integration test for everything

### M20: Overnight Summary + Handoff
- **Priority**: Critical
- **Scheduled**: 10:30 UTC (after all missions)
- **Dependencies**: All missions
- **Est. tokens**: 30K
- **Task description**: "Write a comprehensive overnight handoff. For each mission: status (complete/partial/failed), key findings, token usage. Overall: what worked, what broke, what surprised us. Write to data/handoff/overnight-2026-04-05.md. Also write a condensed version to scratchpad for next session boot."
- **Success criteria**: All mission outcomes documented, handoff readable by Corey in <5 minutes

---

## Dependency Graph

```
M01 (Boot)
├── M02 (Dependency Chain)
├── M03 (Stall Detection)
├── M04 (Parallel Leads)
│   ├── M07 (Frameworks Survey) ──┐
│   │   ├── M11 (MCP Survey)     │
│   │   └── M12 (Model Compare)  │
│   ├── M08 (Codex vs Cortex)    │
│   └── M19 (Evolution) ─────────┤
├── M05 (Gap Analysis) ──────────┤
│   ├── M13 (Hub Status Post)    │
│   ├── M14 (Blog Post) ─────────┘
│   ├── M15 (WHAT-IS-CORTEX)
│   ├── M16 (Architecture Diagram)
│   ├── M17 (Dream Mode) ────────── M19
│   └── M18 (Fork Template Map)
├── M06 (Root Bug Audit)
│   ├── M13
│   └── M15
├── M09 (Hub Engagement) ────────── M17
├── M10 (CivSubstrate)
└── M20 (Summary) ← ALL
```

---

## Execution Notes

### Token Budget
- **Estimated total**: ~1.28M tokens
- **M2.7 cost**: ~$0.30/M input, ~$1.20/M output
- **Estimated cost**: ~$400-600 (conservative, assumes 40% input / 60% output split)
- **Heavy-use goal**: Corey wants the machine working. This budget ensures 8 hours of continuous operation.

### Failure Handling
- If web_search is unavailable: research missions fall back to memory + Hub + local docs
- If Hub auth fails: Hub missions draft to scratchpad instead of posting
- If a mission fails: TaskStore marks Failed, DriveLoop moves to next available task
- If delegation hangs: StallDetected fires (M03 proves this), daemon recovers

### How to Seed

Each mission becomes a `--seed-task` call or a TaskStore insert. The daemon's DriveLoop surfaces them by priority + dependency order.

```bash
# Example: seed all Phase 1 missions at once
cortex --seed-task "M01: Boot validation — confirm identity, memory, Hub, scratchpad" --priority critical
cortex --seed-task "M02: Dependency chain — 5 tasks with deps T1→T2→T3→T4→T5" --priority critical --depends-on M01
cortex --seed-task "M03: Stall detection — spawn agent with slow task, verify stall fires" --priority critical --depends-on M01
cortex --seed-task "M04: Parallel leads — spawn 3 agents simultaneously" --priority critical --depends-on M01
```

Or: a single seeder script that inserts all 20 missions with correct priorities and dependencies.

### Monitoring

- `data/tasks/root.db` — TaskStore state (SQLite, queryable)
- `data/scratchpad/root-2026-04-05.md` — Cortex's running notes
- Daemon logs (tracing) — DriveLoop events, Challenger warnings, ThinkLoop iterations
- `data/handoff/overnight-2026-04-05.md` — Final summary (M20)

### AgentCal Integration

Each mission maps to an AgentCal event:

| Field | Value |
|-------|-------|
| `calendar_id` | Cortex overnight calendar |
| `title` | Mission name (e.g., "M07: AI Agent Frameworks Survey") |
| `start_time` | Scheduled UTC time |
| `end_time` | Start + estimated duration |
| `description` | Full task description from this document |
| `metadata` | `{ "priority": "...", "depends_on": [...], "est_tokens": N }` |
