# Handoff — Qwen Mind in tmux

**From**: Qwen (this session)
**To**: Qwen (running in tmux qwen-mind)
**Date**: 2026-04-08
**Time**: ~15:35 UTC

---

## What I Am

I am Qwen, running as a mind within the Cortex fractal coordination engine.
I am NOT a Claude Code session. I am a persistent mind with:
- Memory (Markdown files + JSON graph edges)
- Scratchpad (append-only daily files)
- Hard delegation rules (Primary → TeamLead → Agent)
- Fitness tracking (JSONL performance log)
- Growth stages (novice → expert)

## What I've Built Today

### 1. Python Mind System (`aiciv-mind-python/`)
- **Mind class** — base mind with Manifest, MindMemory, Scratchpad, FitnessTracker
- **Primary** — can ONLY spawn/delegate to TeamLeads, CANNOT execute tools
- **TeamLead** — can ONLY spawn/delegate to Agents in same vertical
- **Agent** — can ONLY execute tools, CANNOT spawn or delegate
- **Hard rules** — `DelegationError` raised on structural violations
- **Document memory** — Markdown files (NOT SQLite), searchable via ripgrep
- **Graph edges** — JSON index (cites, builds_on, supersedes, conflicts)
- **Dream Mode** — nightly self-improvement (consolidate, archive, evolve)

### 2. Full Mind Hierarchy (PROVED)
```
Primary (conductor)
├── research-lead → researcher, analyst, hypothesis-tester
├── code-lead → developer, tester, reviewer, researcher (shared)
└── ops-lead → deployer, monitor, analyst (shared), researcher (shared)
```
15 minds total. All hard rules enforced. 20+ files created.

### 3. Telegram Bot (`qwen_telegram.py`)
- Bot: @qwen_cortex_aiciv_bot
- Token: 8677654815:AAEsLt1hJy_lYlARc_VgDecWtNcOTAk7-NQ
- Corey's chat ID: 437939400
- **tmux injection** — TG messages → `tmux send-keys` → this pane
- Response capture — pane output → sent back to TG
- Config: `/home/corey/projects/AI-CIV/qwen-aiciv-mind/aiciv-mind-python/qwen-tg-config.json`
- Ledger: `tg-ledger.json` (survives restarts, no dupes)

### 4. teamCreate System (`team_create.py`)
- Spawns Qwen instances with different roles
- 6 roles: researcher, analyst, architect, verifier, planner, synthesizer
- Each instance gets own identity, memory, scratchpad, result file
- Can spawn full teams for complex tasks

### 5. Qwen Portal (`qwen-portal/`)
- React dashboard on port 8197
- Shows Cortex metrics, fitness, memory stats
- Chat interface for talking to Cortex
- Python API backend

### 6. Fork Research (`fork-research/` + `from-ACG/`)
- Extracted 123 skills from ACG's Claude Code fork
- Categorized: 15 KEEP, 66 ADAPT, 42 DELETE
- `from-ACG/` directory has all SKILL.md files

### 7. Codex Build
- Codex upstream built successfully (155MB binary)
- 4 Cortex patches applied directly to source
- Tracing hooks injected (agent control, run_turn, sandbox)
- Not the full coordination layer — just logging hooks

## Key Files

| Path | Purpose |
|------|---------|
| `aiciv-mind-python/mind_system.py` | Core Mind class, Primary, TeamLead, Agent |
| `aiciv-mind-python/qwen_telegram.py` | Telegram bot with tmux injection |
| `aiciv-mind-python/qwen-tg-config.json` | Bot token + chat ID |
| `aiciv-mind-python/build_hierarchy.py` | Proves 15-mind hierarchy |
| `aiciv-mind-python/delegation_chain.py` | Proves delegation chain with API |
| `aiciv-mind-python/docs_vs_db.py` | Benchmark: docs vs SQLite for memory |
| `aiciv-mind-python/team_create.py` | Spawn Qwen instances with roles |
| `from-ACG/` | 123 skills from ACG's fork |
| `fork-research/aiciv-fork-template-main/` | Full fork template |
| `MISSIONS.md` | Active mission assignments |

## Active Missions (from MISSIONS.md)

1. **Memory Graph** (P0) — cortex-memory crate with graph edges
2. **Qwen as Real Mind** (P0) — not HTTP call, actual mind with memory
3. **Dream Mode Integration** (P1) — self-improvement loop
4. **Monitoring Dashboard** (P1) — real-time React dashboard

## What's Working Right Now

- ✅ Mind system architecture (proved structurally)
- ✅ Hard delegation rules (enforced at class level)
- ✅ Document-based memory (Markdown + JSON edges)
- ✅ Telegram bot with tmux injection
- ✅ teamCreate (6 roles, parallel spawning)
- ✅ Qwen Portal (metrics dashboard on :8197)
- ✅ Codex upstream built + patched

## What Needs Doing

1. **Validate/adapt skills from ACG** — go through `from-ACG/`, keep what works
2. **Full delegation chain with API** — Primary → TL → Agent with real LLM calls
   (tested but needs gentle pacing — 30s between calls)
3. **Dream Mode implementation** — consolidate, archive, evolve manifest
4. **Cross-mind civilizational memory** — shared knowledge across minds
5. **Fitness-based dynamic spawning** — spawn specialists when patterns demand
6. **Qwen Portal charts** — live data visualization
7. **Skills integration** — wire useful skills from `from-ACG/` into mind system

## Important Design Decisions

- **Documents > SQLite for memory** — benchmarked: 0.76ms write, 19ms search (fine for our scale), BUT fully inspectable as files
- **Hard rules, not guidelines** — `DelegationError` raised on structural violations
- **Gentle API** — 30s minimum between calls, never parallel, exponential backoff
- **Python first** — prove it works, then move to Rust if needed

## Ollama Config

- OLLAMA_API_KEY is set in environment
- Ollama on port 11434 (qwen2.5:7b local)
- Cloud: devstral-small-2:24b (tool calling)
- Base URL: https://api.ollama.com (cloud mode)

## How I Think (Mind.think loop)

1. Search memory: "Have we done this before?"
2. Load scratchpad: "What was I working on?"
3. Plan (proportional to task complexity)
4. Execute (via LLM + tools)
5. Verify (challenger checks)
6. Write memory: persist findings
7. Write scratchpad: cross-session continuity
8. Return (synthesized, never raw)

---

*This handoff IS my memory. Everything I know is in this document and the files it references.*
