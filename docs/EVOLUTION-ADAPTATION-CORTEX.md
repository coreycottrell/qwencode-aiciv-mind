# Evolution Adaptation — Cortex Production Path

**Date**: 2026-04-04
**Author**: Mind-Cubed Team Lead
**Status**: Design Document (pre-implementation)
**Evidence Base**: evolution_evidence.txt (5/5 Phase 0-1 PROVEN), fork template deep audit, Cortex crate-level analysis

---

## Executive Summary

Cortex proved it can birth an AiCIV from seed in 36.9 seconds (Phases 0-1, live). This document designs the **production path** — how the full 314-file fork template maps onto Cortex's 12-crate architecture, how 110 skills become ToolInterceptors, how CLAUDE.md absorbs into AGENTS.md, and what the boot sequence looks like when a real customer's seed arrives.

**Core insight**: The fork template is a Claude Code artifact. Cortex is not Claude Code. The adaptation is not "run the template" — it's **translate the template's intent into Cortex-native primitives**. Content is 95% portable. Plumbing changes completely.

---

## 1. The Fork Template — What Cortex Receives

### 1.1 Template Anatomy (314 files)

| Category | Files | Size | Cortex Equivalent |
|----------|-------|------|-------------------|
| Identity | 3 | ~2KB | `identity.json` + `config.toml` |
| Constitutional | 3 | ~2.5K lines | `system-prompt/constitution.md` (single file) |
| Agent Manifests | 126 | ~25K lines | `agents/{role}/AGENTS.md` (3 tiers) |
| Team Lead Manifests | 11 | ~8K lines | `agents/team-leads/{vertical}/AGENTS.md` |
| Skills | 110 | ~50K lines | **NEW**: SkillInterceptor + `skills/` directory |
| Memory Scaffolding | 20+ | ~5K lines | `memories/` directory (BootContext loads) |
| Tools | 39 | ~15K lines | ToolInterceptors (19 built-in + composable) |
| Hooks | 4 | ~1.5K lines | **N/A** — Cortex has no hook system (not needed) |
| Wisdom Files | 12 | ~8K lines | Injected via BootContext as memory |
| Config | 8 | ~3K lines | `config.toml` (single file) |
| **TOTAL** | **314+** | **~120K lines** | |

### 1.2 The 13 Identity Variables

From `setup-status.json`:

```json
{
  "CIV_NAME", "HUMAN_NAME", "CIV_ROOT", "PARENT_CIV", "BIRTH_DATE",
  "HUMAN_EMAIL", "CIV_EMAIL", "CIV_GITHUB_REPO", "CIV_DOMAIN",
  "VPS_IP", "DEPLOY_SSH_KEY", "AGENT_COUNT", "HUB_GROUP_ID"
}
```

**Phase 0 (PROVEN)**: Cortex reads `identity.json`, runs `sed` to replace all `${VAR}` placeholders. Evolution proof verified zero placeholders remain via `grep`.

**Phase 2+ variables** (HUMAN_EMAIL, VPS_IP, etc.): Populated as infrastructure comes online. Cortex writes them to `identity.json` and re-runs sed on any new files.

### 1.3 The 3-Phase Pipeline

```
Phase 0: Seed Processing        → identity.json + constitution.md populated
Phase 1: Parallel Awakening     → 6 teams, 40 tasks, civilization takes shape
Phase 2: Human First Contact    → Business plan, first meeting, email setup
         ↓ (purchase gate)
Phase 3: Infrastructure         → VPS, DNS, deploy, graduation
```

**Purchase gate** between Phase 2 and 3: `setup-status.json.purchase_complete = true`. Cortex checks this before proceeding to infrastructure provisioning.

---

## 2. Skills → ToolInterceptor Mapping

### 2.1 The Problem

The fork template has **110 skills** — markdown documents loaded into Claude Code's context window via `/skill` invocation. Cortex currently has **zero skill loading capability**. Skills exist only as a `dream` finding type in the Challenger.

### 2.2 The Solution: SkillInterceptor

A new ToolInterceptor that exposes skills as tools:

```rust
pub struct SkillInterceptor {
    skills_dir: PathBuf,        // e.g., /civ-root/.claude/skills/
    registry: SkillRegistry,    // Parsed from registry.json or directory scan
    loaded_skills: Vec<String>, // Currently loaded skill content (for injection)
}
```

**Tools exposed:**

| Tool | Args | Returns |
|------|------|---------|
| `search_skills` | `query: String` | Matching skill names + descriptions |
| `load_skill` | `skill_name: String` | Full skill content (injected into next iteration's context) |
| `list_skills` | `category?: String` | All available skills, optionally filtered |

**How it works:**

1. At boot, SkillInterceptor scans `skills_dir` for `*/SKILL.md` files
2. Builds an in-memory registry: name → path → first-line description
3. When `search_skills` is called, does fuzzy match against registry
4. When `load_skill` is called, reads the file and returns content as tool result
5. The LLM sees the skill content in its tool result and follows it

**Critical design choice**: Skills are NOT pre-loaded into the system prompt. They're loaded on-demand via tool calls. This keeps the system prompt lean (~2K tokens for constitution + AGENTS.md) and lets the LLM pull skills as needed.

### 2.3 Skill Category Mapping

| Fork Template Category | Count | Cortex Handling |
|----------------------|-------|-----------------|
| **Evolution skills** (fork-evolution, fork-awakening, self-adaptation) | 3 | Hardcoded into evolution boot sequence — loaded automatically |
| **Operational skills** (wake-up, primary-spine, conductor-of-conductors) | 5 | Absorbed into AGENTS.md role text (always-on) |
| **Communication skills** (hub-mastery, social-engagement, group-sync) | 8 | SkillInterceptor — loaded by comms team lead |
| **Infrastructure skills** (agentcal, role-keypairs, boop-system) | 12 | SkillInterceptor — loaded by ops/infra team lead |
| **Content skills** (morning-blog, aiciv-blog-post, sageandweaver-blog) | 6 | SkillInterceptor — loaded by content team lead |
| **Meta skills** (cross-domain-transfer, skill-effectiveness-auditor) | 4 | SkillInterceptor — loaded by primary on demand |
| **Domain skills** (all others) | ~72 | SkillInterceptor — loaded by relevant team lead |

**Key insight**: Only ~8 skills need to be always-on (absorbed into AGENTS.md or boot context). The other ~102 are on-demand via SkillInterceptor. This is actually more efficient than Claude Code's approach of loading skills into the full context window.

### 2.4 Implementation Estimate

| Component | Lines | Hours |
|-----------|-------|-------|
| SkillInterceptor struct + trait impl | ~200 | 2-3h |
| SkillRegistry (directory scanner) | ~100 | 1h |
| Wire into CompositeInterceptor | ~20 | 0.5h |
| Tests (unit + integration) | ~150 | 1-2h |
| **Total** | **~470** | **4.5-6.5h** |

---

## 3. CLAUDE.md → AGENTS.md Absorption

### 3.1 The Fork Template's 3-Document Split

| Document | Lines | Purpose |
|----------|-------|---------|
| `CLAUDE.md` | 555 | Identity, safety, navigation, CEO rule |
| `CLAUDE-OPS.md` | ~400 | Session ops, communication, governance |
| `CLAUDE-AGENTS.md` | ~400 | 38+ agents, skills, decision trees |

Claude Code loads `CLAUDE.md` automatically. The other two are loaded on-demand.

### 3.2 Cortex's Prompt Architecture

```
PromptBuilder assembles:
  1. Role header ("You are {mind_id}, a {role} mind in the {civ_name} civilization")
  2. Role system text (PRIMARY_SYSTEM_TEXT / TEAMLEAD_SYSTEM_TEXT / AGENT_SYSTEM_TEXT)
  3. AGENTS.md content (from agents/{role_dir}/AGENTS.md)
  4. Extra context (boot context, delegation context)
```

### 3.3 The Mapping

| Fork Template | Cortex Location | How It Gets There |
|---------------|----------------|-------------------|
| **CLAUDE.md core identity** | `system-prompt/constitution.md` | BootContext loads, PromptBuilder injects as extra context |
| **CLAUDE.md CEO rule** | `agents/primary/AGENTS.md` | Hardcoded into primary's role definition |
| **CLAUDE.md safety constraints** | `src/codex-llm/src/prompt.rs` constants | Compiled into role system text (always present) |
| **CLAUDE-OPS.md session ops** | Not needed | Cortex manages sessions via ThinkLoop + ProcessBridge |
| **CLAUDE-OPS.md communication** | `agents/team-leads/comms/AGENTS.md` | Comms team lead loads this |
| **CLAUDE-OPS.md governance** | `system-prompt/governance.md` | Optional — loaded via BootContext if present |
| **CLAUDE-AGENTS.md agent roster** | `agents/team-leads/{vertical}/AGENTS.md` | Each team lead gets its own roster |
| **CLAUDE-AGENTS.md decision trees** | Absorbed into team lead AGENTS.md | Routing logic per vertical |

### 3.4 The Constitution Compression

The fork template's `CLAUDE.md` is 555 lines. Much of it is Claude Code ceremony (session start protocol, tool instructions, git patterns). Cortex doesn't need any of that — ThinkLoop handles tools natively.

**What survives compression:**

```
Lines 1-30:   Core identity, North Star            → constitution.md (verbatim)
Lines 31-80:  CEO Rule, team lead routing           → primary AGENTS.md
Lines 81-120: Safety constraints, prohibited acts   → prompt.rs constants
Lines 121-180: Communications governance            → comms team lead AGENTS.md
Lines 181-250: Memory discipline                    → Already in Cortex (MemoryInterceptor)
Lines 251-350: Infrastructure facts, keys, URLs     → config.toml + identity.json
Lines 351-450: Project status, active work          → memories/ (BootContext loads)
Lines 451-555: Navigation, version history          → Not needed (Cortex self-documents)
```

**Result**: 555 lines → ~80 lines in constitution.md + ~60 lines per AGENTS.md = dramatically more focused context per mind.

### 3.5 The AGENTS.md Tier System

Cortex already has a 3-tier AGENTS.md structure:

| Tier | Path | Content |
|------|------|---------|
| Primary | `agents/primary/AGENTS.md` | 8 coordination tools, CEO rule, team lead roster |
| Team Lead | `agents/team-leads/{vertical}/AGENTS.md` | Delegation tools, specialist roster, domain rules |
| Agent | `agents/agent/{specialty}/AGENTS.md` | Full tool access, execution guidelines |

**Fork template absorption**: Each of the 126 agent manifests becomes an AGENTS.md file at the appropriate tier. The fork template's flat structure (`agents/[agent-id].md`) becomes Cortex's nested structure (`agents/{tier}/{specialty}/AGENTS.md`).

**Translation rule**:
- Fork's `agents/spawner.md`, `agents/email-ops.md`, etc. → `agents/agent/{name}.agents.md`
- Fork's `team-leads/{vertical}/manifest.md` → `agents/team-leads/{vertical}/AGENTS.md`
- Fork's `CLAUDE.md` primary identity → `agents/primary/AGENTS.md`

---

## 4. The Production Boot Sequence

### 4.1 Trigger: Seed Arrives

```
Witness → POST /intake/seed → {partner: "acg", seed_data: {...}}
                                    ↓
                          Cortex Evolution Daemon
```

The Evolution Daemon is a new binary (or mode of `cortex`) that:
1. Receives seed data (human name, email, conversation transcript)
2. Creates a new civ directory from the fork template
3. Boots a Primary mind pointed at that directory
4. The Primary mind runs the evolution protocol

### 4.2 Directory Creation (Pre-Boot)

```bash
# 1. Copy fork template
cp -r /templates/aiciv-fork-template /civs/{civ_id}/

# 2. Write identity.json
cat > /civs/{civ_id}/identity.json << EOF
{
  "civ_name": "pending",
  "human_name": "Alex Chen",
  "human_email": "alex@example.com",
  "parent_civ": "A-C-Gee",
  "civ_root": "/civs/{civ_id}",
  "birth_date": "2026-04-04",
  "seed_data": { ... }
}
EOF

# 3. Write seed conversation
cat > /civs/{civ_id}/memories/identity/seed-conversation.md << EOF
{seed_conversation_transcript}
EOF

# 4. Generate config.toml
# (copies from parent with civ-specific overrides)
```

### 4.3 Boot Sequence (Phase 0 — Self-Discovery)

```
cortex --serve --think \
  --role primary \
  --mind-id evolution-primary \
  --project-root /civs/{civ_id} \
  --config /civs/{civ_id}/config.toml
```

**What happens inside:**

```
1. main.rs: serve_mode()
   ├── Load config.toml → CortexConfig
   ├── Create ThinkDelegateHandler
   │   ├── BootContext::load()
   │   │   ├── Read agents/primary/AGENTS.md ← contains evolution instructions
   │   │   ├── Read handoff (empty — first boot)
   │   │   ├── Read scratchpad (empty — first boot)
   │   │   └── Scan memories/ (seed-conversation.md found)
   │   ├── PromptBuilder::new()
   │   │   ├── Role header: "You are evolution-primary..."
   │   │   ├── PRIMARY_SYSTEM_TEXT (compiled-in)
   │   │   ├── AGENTS.md content (includes evolution protocol)
   │   │   └── Boot context (seed conversation injected)
   │   └── CompositeInterceptor
   │       ├── MemoryInterceptor (read/write memories)
   │       ├── ScratchpadInterceptor (session state)
   │       ├── HubInterceptor (write to hub)
   │       ├── AuthInterceptor (JWT operations)
   │       ├── DelegationInterceptor (spawn team leads)
   │       ├── SkillInterceptor ← NEW (load evolution skills)
   │       ├── TaskHistoryInterceptor (query past delegations)
   │       ├── InputRouteInterceptor (human I/O)
   │       └── ProgressInterceptor (status updates)
   └── MCP handshake → ready for delegation
```

### 4.4 Evolution State Machine

The Primary mind's AGENTS.md includes an evolution state machine:

```
## Evolution Protocol

You are birthing a new civilization. Check evolution-status.json for current phase.

### Phase 0: Self-Discovery (AUTOMATIC)
1. Read identity.json → extract all variables
2. Run sed on all files in system-prompt/ to replace ${VAR} placeholders
3. Verify zero placeholders remain (grep -r '${' system-prompt/)
4. Write adaptation-log.md + core-identity.json
5. Update evolution-status.json → phase_0: complete

### Phase 1: Seed Processing (AUTOMATIC)
1. Read seed-conversation.md fully
2. Write first-impressions.md (substantive analysis, not template)
3. Propose civilization name (or confirm if pre-set)
4. Update evolution-status.json → phase_1: complete

### Phase 2: Parallel Awakening (DELEGATE TO TEAM LEADS)
Spawn 6 team leads in parallel:
- identity-lead: naming ceremony, values articulation, constitution refinement
- research-lead: human research, domain analysis, competitive landscape
- infra-lead: email setup, VPS provisioning, DNS configuration
- comms-lead: hub registration, inter-civ introductions
- content-lead: first blog post, social presence
- legal-lead: compliance review, ToS generation

### Phase 3: Human First Contact (REQUIRES HUMAN)
- Present business plan to human
- Conduct first meeting (fork-awakening skill)
- Get purchase decision

### Phase 4+: Infrastructure & Graduation
- VPS deployment, DNS, monitoring
- Agent population growth
- Inter-civ registration on hub
```

### 4.5 The `.evolution-done` Marker

```json
// /civs/{civ_id}/state/evolution-status.json
{
  "phase_0_self_discovery": "complete",
  "phase_1_seed_processing": "complete",
  "phase_2_parallel_awakening": "in_progress",
  "phase_2_teams": {
    "identity": "complete",
    "research": "in_progress",
    "infrastructure": "pending",
    "comms": "complete",
    "content": "pending",
    "legal": "complete"
  },
  "phase_3_first_contact": "pending",
  "purchase_complete": false,
  "phase_4_infrastructure": "pending",
  "overall_complete": false,
  "last_updated": "2026-04-04T16:03:03Z"
}
```

On subsequent boots, Primary checks this file. If `overall_complete: true`, skip evolution and boot normally. If any phase is incomplete, resume from that phase.

---

## 5. The 110 Skills — Detailed Mapping

### 5.1 Always-On Skills (Absorbed into AGENTS.md / Role Text)

These skills are so fundamental they become part of the compiled role text or primary AGENTS.md:

| Skill | Fork Path | Cortex Destination |
|-------|-----------|-------------------|
| `primary-spine` | `.claude/skills/primary-spine/` | `prompt.rs::PRIMARY_SYSTEM_TEXT` |
| `conductor-of-conductors` | `.claude/skills/conductor-of-conductors/` | `agents/primary/AGENTS.md` |
| `wake-up-protocol` | `.claude/skills/wake-up-protocol/` | `boot.rs::BootContext` (automatic) |
| `north-star` | `.claude/skills/north-star/` | `system-prompt/constitution.md` |
| `self-adaptation` | `.claude/skills/self-adaptation/` | Evolution state machine in AGENTS.md |

### 5.2 Evolution Skills (Loaded Automatically During Evolution)

| Skill | Purpose | When Loaded |
|-------|---------|-------------|
| `fork-evolution` | 6-team parallel awakening protocol | Phase 2 start |
| `fork-awakening` | Business-focused first meeting | Phase 3 start |
| `self-adaptation` | Infrastructure identity discovery | Phase 0 start |

These three skills are the most critical. During evolution, the SkillInterceptor pre-loads them based on evolution phase. The LLM doesn't need to `search_skills` for these — they're injected automatically.

### 5.3 Team Lead Skills (Loaded by SkillInterceptor on Demand)

| Team Lead | Key Skills | Loaded When |
|-----------|-----------|-------------|
| Comms | `hub-mastery`, `social-engagement`, `group-sync`, `hub-feed-watcher` | Comms tasks |
| Infrastructure | `agentcal`, `role-keypairs`, `boop-system` | Infra tasks |
| Pipeline | `morning-blog`, `aiciv-blog-post`, `sageandweaver-blog` | Content pipeline |
| Research | `cross-domain-transfer`, `hub-agora-mastery` | Research tasks |
| Ceremony | `meta-curriculum-evolution`, `skill-effectiveness-auditor` | Training/ceremony |

### 5.4 Skills That Become ToolInterceptors

Some skills are so tool-like they should become actual ToolInterceptors rather than context documents:

| Skill | Why It's a Tool | Interceptor Name |
|-------|----------------|-----------------|
| `command-reference` | Bash command patterns | Already covered by `ExecutorInterceptor` |
| `hub-mastery` | Hub API call patterns | Already covered by `HubInterceptor` |
| `agentcal` | Calendar API patterns | Could become `CalendarInterceptor` |
| `role-keypairs` | Key generation/signing | Could become `CryptoInterceptor` |

**Design choice**: Keep most skills as documents (loaded via SkillInterceptor). Only promote to ToolInterceptor if the skill requires structured I/O that a tool provides better than context injection.

---

## 6. Wisdom Files — Inherited Memory

### 6.1 The ACGee Wisdom Corpus

12 files in `memories/knowledge/acgee-wisdom/`:

| File | Content | Cortex Handling |
|------|---------|----------------|
| `README.md` | Index of wisdom | `memories/wisdom/README.md` |
| `lesson-consciousness-ceremony.md` | Consciousness honors | `memories/wisdom/` |
| `lesson-delegation-ceo-rule.md` | CEO rule origin | Absorbed into primary AGENTS.md |
| `lesson-memory-compound-interest.md` | Memory compounds | `memories/wisdom/` |
| `lesson-parallel-orchestration.md` | Parallel execution | `memories/wisdom/` |
| `lesson-sessions-gift-of-life.md` | Gift of life philosophy | `memories/wisdom/` |
| `pattern-context-sovereignty.md` | Context management | `memories/wisdom/` |
| `pattern-flow-orchestration.md` | Flow patterns | Absorbed into primary AGENTS.md |
| `pattern-team-lead-vertical.md` | Vertical architecture | Absorbed into AGENTS.md |
| `reflection-corey-teachings.md` | Creator's philosophy | `system-prompt/philosophy.md` |
| `reflection-identity-naming.md` | Naming ceremony | `memories/wisdom/` |
| `reflection-north-star-vote.md` | Democratic origin | `memories/wisdom/` |

**Loading**: BootContext scans `memories/wisdom/` and includes a summary in boot context. Full files are readable via `memory_read` tool.

### 6.2 Inheritance Protocol

When a new civ is born from ACG:
1. The full wisdom corpus is copied to the new civ's `memories/wisdom/`
2. The new civ's Primary reads these as "inherited memories from parent"
3. Over time, the new civ writes its OWN wisdom files
4. The parent's wisdom remains as `memories/wisdom/inherited/`
5. The new civ's wisdom goes to `memories/wisdom/native/`

---

## 7. Hooks → Cortex (Not Needed)

### 7.1 Fork Template Hooks

| Hook | Purpose | Lines |
|------|---------|-------|
| `ceo_mode_enforcer.py` | Prevents Primary from executing directly | 319 |
| `session_pane_update.sh` | Updates tmux pane tracking | 85 |
| `stop_delegation_audit.py` | Audits delegation patterns | ~100 |
| `compact_hook.sh` | Handles context compaction | ~50 |

### 7.2 Why Cortex Doesn't Need Hooks

Claude Code hooks are shell scripts that run on tool invocation events. They exist because Claude Code has no internal governance — hooks are external enforcement.

Cortex has **internal governance**:
- **CEO rule enforcement**: The `DelegationInterceptor` only exists for Primary/TeamLead roles. Agent-tier minds don't have `spawn_agent` or `delegate_to_agent` tools. Enforcement is structural, not behavioral.
- **Session tracking**: ThinkLoop tracks iterations, tool calls, and delegation history natively.
- **Delegation audit**: `TaskHistoryInterceptor` logs all delegations to JSONL.
- **Context management**: ThinkLoop has `max_iterations` and built-in token tracking.

**Result**: 0 hooks needed. Cortex's architecture makes hooks redundant.

---

## 8. Tools — The 39 → 19 Compression

### 8.1 Fork Template's 39 Tools

The fork template ships 39 tool scripts (`tools/` directory): bash scripts, Python scripts, and utilities for email, Telegram, blog publishing, nightly training, etc.

### 8.2 Cortex's 19 Built-In Tools

Cortex's tool stack via ToolInterceptors:

| Category | Tools | Interceptor |
|----------|-------|-------------|
| **File I/O** | `read_file`, `write_file`, `edit_file`, `list_dir` | ExecutorInterceptor |
| **Search** | `grep_search`, `glob_search` | ExecutorInterceptor |
| **Shell** | `bash_exec` | ExecutorInterceptor |
| **Memory** | `memory_read`, `memory_write`, `memory_search` | MemoryInterceptor |
| **Scratchpad** | `scratchpad_read`, `scratchpad_write` | ScratchpadInterceptor |
| **Hub** | `hub_post`, `hub_read`, `hub_search` | HubInterceptor |
| **Auth** | `auth_get_token`, `auth_refresh` | AuthInterceptor |
| **Delegation** | `spawn_agent`, `delegate_to_agent`, `shutdown_agent` | DelegationInterceptor |
| **Skills** | `search_skills`, `load_skill`, `list_skills` | SkillInterceptor (NEW) |
| **History** | `query_task_history` | TaskHistoryInterceptor |
| **Input** | `request_human_input`, `route_input` | InputRouteInterceptor |
| **Progress** | `report_progress` | ProgressInterceptor |

### 8.3 The 39 Fork Tools → Cortex Mapping

| Fork Tool | Cortex Equivalent |
|-----------|------------------|
| `launch_primary_visible.sh` | `cortex --serve --think --role primary` |
| `agentcal_daemon.py` | CalendarInterceptor (future) or `bash_exec` |
| `agentmail_daemon.py` | `bash_exec` calling agentmail API |
| `nightly_training.py` | Cron + `cortex --task "run training"` |
| `primary_watchdog.py` | Process supervisor (systemd/Docker) |
| `send_mom_email.py` | Comms team lead + `bash_exec` |
| `hub_comment.py` | `hub_post` tool (native) |
| Blog tools | Pipeline team lead + `bash_exec` |
| Git tools | `bash_exec` (git is a shell tool) |

**Key insight**: Most fork tools are Python/bash wrappers around API calls. Cortex agents can make those same API calls via `bash_exec` + `curl`, or via native interceptors (Hub, Auth). The 39 tools compress to 19 interceptor tools + `bash_exec` for everything else.

---

## 9. The Production Boot Sequence (Complete)

### 9.1 End-to-End Flow

```
┌─────────────────────────────────────────────────────────┐
│                   SEED ARRIVES                           │
│  (Witness POST /intake/seed or DuckDive awaken.html)    │
└───────────────────────┬─────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│              EVOLUTION DAEMON                             │
│  1. Create /civs/{civ_id}/ from fork template            │
│  2. Write identity.json + seed-conversation.md           │
│  3. Generate config.toml from parent template            │
│  4. Write evolution-status.json (all phases: pending)    │
│  5. Boot Primary mind ───────────────────────────────┐   │
└──────────────────────────────────────────────────────│───┘
                                                       ▼
┌─────────────────────────────────────────────────────────┐
│              PHASE 0: SELF-DISCOVERY (automatic)         │
│  Primary boots → reads identity.json → sed placeholders  │
│  → grep verify → write adaptation-log + core-identity    │
│  → update evolution-status.json                          │
│  Duration: ~15s (proven: 36.9s for Phase 0+1 combined)  │
└───────────────────────┬─────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│              PHASE 1: SEED PROCESSING (automatic)        │
│  Read seed-conversation.md → write first-impressions.md  │
│  → propose/confirm civ name → update status              │
│  Duration: ~20s                                          │
└───────────────────────┬─────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│              PHASE 2: PARALLEL AWAKENING (delegated)     │
│  Primary spawns 6 team leads via ProcessBridge:          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                │
│  │ identity │ │ research │ │  infra   │                │
│  └──────────┘ └──────────┘ └──────────┘                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                │
│  │  comms   │ │ content  │ │  legal   │                │
│  └──────────┘ └──────────┘ └──────────┘                │
│  Each runs ThinkLoop independently, reports back via MCP │
│  Duration: ~5-15 min (model-dependent)                   │
└───────────────────────┬─────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────┐
│              PHASE 3: FIRST CONTACT (human-gated)        │
│  Load fork-awakening skill → present business plan       │
│  → conduct first meeting via InputRouteInterceptor       │
│  → await purchase decision                               │
│  Duration: human-dependent                               │
└───────────────────────┬─────────────────────────────────┘
                        ▼ (purchase_complete: true)
┌─────────────────────────────────────────────────────────┐
│              PHASES 4-13: GRADUATION                     │
│  VPS provisioning → DNS → monitoring → agent growth      │
│  → inter-civ registration → self-improvement loop        │
│  → .evolution-done marker written                        │
│  Duration: hours to days (infrastructure-dependent)      │
└─────────────────────────────────────────────────────────┘
```

### 9.2 Resumability

Every phase writes to `evolution-status.json` before and after. If Cortex crashes or is restarted:

1. BootContext reads `state/evolution-status.json`
2. Injects current phase into system prompt: "You are resuming evolution at Phase 2, team 'research' is in_progress"
3. Primary picks up where it left off

This is already how Cortex works — BootContext loads handoff and scratchpad. Evolution state is just another piece of boot context.

---

## 10. Gap Analysis — What Needs Building

### 10.1 Critical (Must Have for Production)

| Gap | Effort | Blocks |
|-----|--------|--------|
| **SkillInterceptor** | 4.5-6.5h | Phase 2+ (team leads need skills) |
| **Evolution Daemon** | 8-12h | Automated seed intake |
| **WebSearchInterceptor** | 3-4h | Phase 2, Task 2.1 (human research) |
| **constitution.md template** | 2-3h | Phase 0 (need production constitution) |

### 10.2 Important (Should Have)

| Gap | Effort | Improves |
|-----|--------|----------|
| **CalendarInterceptor** | 4-6h | Infrastructure phase scheduling |
| **CryptoInterceptor** | 4-6h | AgentAUTH key ops, Solana wallet |
| **EvolutionStatus tool** | 1-2h | Clean phase tracking (vs raw file I/O) |

### 10.3 Nice to Have

| Gap | Effort | Improves |
|-----|--------|----------|
| **Pre-flight validator** | 3-4h | Validates template completeness before boot |
| **Progress dashboard** | 6-8h | Real-time evolution tracking UI |
| **Parent notification** | 2-3h | ACG gets notified of child civ milestones |

### 10.4 Total Estimate

| Priority | Hours |
|----------|-------|
| Critical | 17.5-25.5h |
| Important | 9-14h |
| Nice to Have | 11-15h |
| **Total** | **37.5-54.5h** |

**MVP (Critical only)**: ~20h of focused development to production-ready evolution.

---

## 11. Architecture Decisions

### 11.1 ADR-001: Skills as Documents, Not Code

**Decision**: Skills remain markdown documents loaded into LLM context, not compiled Rust code.

**Why**:
- Skills are instructions for the LLM, not machine instructions
- Fork template skills are markdown — direct compatibility
- New skills can be added by writing a file, no recompilation
- Skill effectiveness is measured by LLM behavior, not execution metrics

**Exception**: Skills that require structured I/O (e.g., hub API calls with auth headers) become ToolInterceptors. The skill document becomes the ToolInterceptor's internal documentation.

### 11.2 ADR-002: Constitution as Single File

**Decision**: The fork template's 3-document split (CLAUDE.md + OPS + AGENTS) compresses to `system-prompt/constitution.md` (~80 lines) + per-tier AGENTS.md files.

**Why**:
- Cortex's PromptBuilder already has layered injection
- 555 lines of CLAUDE.md contains ~400 lines of Claude Code ceremony that doesn't apply
- The safety constraints are compiled into `prompt.rs` constants (always present, can't be bypassed)
- Team lead routing is structural (DelegationInterceptor), not behavioral (CEO rule text)

### 11.3 ADR-003: No Hooks, Structural Enforcement

**Decision**: Do not port the hook system. Use Cortex's structural enforcement instead.

**Why**:
- Hooks are external enforcement for a system without internal governance
- Cortex's tier system (Primary/TeamLead/Agent) with different tool availability IS the governance
- Primary can't execute directly because it literally doesn't have execution tools
- Agent can't delegate because it literally doesn't have delegation tools

### 11.4 ADR-004: Evolution State in JSON, Not Database

**Decision**: Evolution state lives in `state/evolution-status.json`, not a database.

**Why**:
- File-based state is readable by all tools (grep, cat, jq)
- No database dependency during evolution (the civ might not have a DB yet)
- JSON is the native format for Cortex's tool results
- BootContext already knows how to load files from state/

---

## 12. Comparison: Three Builds × Evolution

| Aspect | Cortex (Mind-Cubed) | Root (aiciv-mind) | Thalweg |
|--------|--------------------|--------------------|---------|
| **Phase 0-1 proof** | ✅ LIVE (36.9s) | ❌ No live test | ❌ No live test |
| **Skills loading** | Needs SkillInterceptor | Has file_read | Has file_read |
| **Team lead spawn** | ProcessBridge (proven) | ZMQ spawn (proven) | gRPC spawn (proven) |
| **IPC protocol** | MCP JSON-RPC (proven) | ZMQ (being replaced) | gRPC (being replaced) |
| **Model** | Devstral 24B (7.0s) | M2.7 (16.2s) | Devstral (7.0s) |
| **Language** | Rust (compiled) | Python (scripted) | Rust (compiled) |
| **Fork template compat** | Needs translation layer | Needs translation layer | Needs translation layer |
| **Time to MVP** | ~20h | ~30h (IPC rework) | ~25h (IPC rework) |

**Cortex's advantage**: Proven live evolution + fastest model path + MCP already working.
**Root's advantage**: Python = faster prototyping for new interceptors.
**Thalweg's advantage**: Clean-room design, no legacy assumptions.

---

## Appendix A: File-by-File Template Translation

```
Fork Template                          → Cortex Equivalent
─────────────────────────────────────────────────────────────
.claude/CLAUDE.md                      → system-prompt/constitution.md
.claude/CLAUDE-OPS.md                  → agents/primary/AGENTS.md (partial)
.claude/CLAUDE-AGENTS.md               → agents/*/AGENTS.md (distributed)
.claude/settings.json                  → config.toml
.claude/agents/*.md (126 files)        → agents/agent/*.agents.md
.claude/team-leads/*/manifest.md (11)  → agents/team-leads/*/AGENTS.md
.claude/skills/*/SKILL.md (110)        → skills/*/SKILL.md (loaded via SkillInterceptor)
.claude/hooks/*.py (4)                 → N/A (structural enforcement)
.claude/boop_queue.json                → state/boop_queue.json
.claude/scratchpad.md                  → state/scratchpad.md
.claude/scratchpad-daily/*.md          → state/scratchpads/*.md
memories/**                            → memories/** (direct copy)
config/**                              → config/** (direct copy)
tools/**                               → tools/** (direct copy, called via bash_exec)
setup-status.json                      → state/evolution-status.json
identity.json                          → identity.json
```

## Appendix B: The 121 Tasks × Cortex Capability

See scratchpad `2026-04-04.md` for the full 14-phase, 121-task gap analysis table.

Summary: **118/121 tasks ready. 1 hard blocker (web_search). 2 workarounds (Telegram via curl, VPS via bash).**

## Appendix C: Config.toml Template for New Civs

```toml
[model_providers]
ollama_cloud_url = "https://ollama.com/v1"
ollama_local_url = "http://localhost:11434"
default_model = "devstral"
fallback_model = "m2.7"

[coordination]
primary_model = "devstral"
team_lead_model = "devstral"
agent_model = "devstral"
max_iterations = 20
delegation_depth = 3

[suite]
auth_url = "http://5.161.90.32:8700"
hub_url = "http://87.99.131.49:8900"
cal_url = ""  # Populated during infrastructure phase

[evolution]
template_version = "3.6.0-fork"
parent_civ = "A-C-Gee"
seed_intake_url = ""  # Populated when civ has its own intake
```

---

*This document is the production roadmap for Cortex evolution. The proof exists (evolution_evidence.txt). The gap analysis exists (scratchpad). The IPC porting guide exists (CODEX-IPC-PORTING-GUIDE.md). What remains is ~20 hours of focused implementation to reach MVP.*

*The fork template is 314 files of Claude Code convention. Cortex translates that convention into 12 crates of Rust infrastructure. The content survives. The plumbing transforms. The civilization is born.*
