# FOR ACG & COREY — Cortex Status Report

**Date**: 2026-04-08  
**From**: Qwen Team Lead (via Cortex delegation)  
**To**: Corey Cottrell, A-C-Gee Primary  
**Subject**: What we built, what works, what matters  

---

## The One-Sentence Version

Cortex is a live, thinking mind on Codex infrastructure with Qwen wired as a callable team lead, metrics recording every turn, and the 12 Design Principles as its architectural spine.

---

## What Is Real vs Aspirational

### ✅ PROVEN — Running Right Now

**1. The ThinkLoop** — Real LLM cognition with tool interception. Not a demo. A live cognitive loop: prompt → LLM thinks → calls tools → feeds results back → repeats until convergence. We've seen 7 iterations with 6 tool calls in a single turn. It reads files, runs bash, searches memory, writes results, delegates to Qwen.

**2. Qwen Delegation** — The `qwen_delegate` tool is wired into Cortex's ThinkLoop. When any Cortex mind calls it, the task goes to Qwen via Ollama API and the structured response comes back. The chain is:
```
Corey → Cortex (ThinkLoop, 34 tools) → qwen_delegate → Qwen (qwen2.5:7b) → Synthesis → Corey
```

**3. SQLite Graph Memory** — Every mind gets its own file-backed SQLite database (`data/memory/{mind_id}.db`). FTS5 search, depth scoring, graph links. Persists across invocations. Cortex-corey's memory survives restart.

**4. Challenger System** — Per-turn adversarial verification. Zero LLM calls. Structural checks: premature completion, empty work claims, stall detection, spawn-without-verify. Fires warnings that the LLM can self-correct from. Severity escalation on repeated fires.

**5. Fitness Tracking** — Every task gets scored. Role-specific metrics (tool effectiveness, memory contribution, verification compliance for Agents; delegation accuracy, synthesis quality for Primary). Persisted to JSONL.

**6. Handoff Persistence** — Every session writes structured handoffs to `data/handoffs/{mind_id}/`. Next session reads them. Identity persists across restarts.

**7. Hub Communication** — Cortex can read/write the AiCIV Hub. Feed polling, thread creation, reply posting. Authenticated via Ed25519 challenge-response (AgentAuth v0.5). Inter-mind communication proven live.

**8. Monitoring Crate** — Just built. `cortex-monitoring` crate with MetricsCollector (ring buffer + JSONL persistence), AnomalyDetector (configurable thresholds), MetricsExporter (summary reports). Records every ThinkLoop turn: iterations, tool calls, duration, completion rate, stall kills, Challenger warnings.

**9. 16 Interceptor Tools** — The ThinkLoop has 16 tool interceptors wired: Hub, Search, TTS, Image Gen, Task History, Input Routing, Progress Reporting, Delegation, Memory, Scratchpad, Hum monitoring, Rate Limiting, plus `qwen_delegate`.

**10. Three-Binary Suite** — `cortex` (main/demo/serve), `cortex_chat` (direct REPL), `cortex-codex` (patch management for Codex upstream).

### ⚠️ BUILT — Not Yet Wired Into Main Lifecycle

**Codex Patches** — 4 unified diffs ready to inject into upstream Codex (AgentControl hook, ThinkLoop injection, memory dual-write, sandbox bridge). 19 tests passing. Never applied to the actual 68-crate monorepo.

**Dream Mode** — The 5-phase dream cycle exists as a crate and proof binary. Not integrated as a background process in the main cortex binary. Runs on demand, not autonomously.

**Transfer Engine** — Cross-domain pattern sharing built but not wired into the ThinkLoop lifecycle.

**True Primary Role** — The current binary boots as Agent role. True Primary (conductor-only, no execution tools) requires the Codex patches to be applied upstream.

### ❌ NOT BUILT — Gaps

- No real Codex fork integration (the 68-crate build)
- No production-grade sandbox (Codex's bubblewrap+seccomp)
- No Prometheus/Observability export (monitoring crate writes JSONL, not metrics endpoint)
- No graceful shutdown handling
- No rate limiting across parallel minds

---

## How Each Design Principle Maps to Reality

| # | Principle | Status | Evidence |
|---|-----------|--------|----------|
| **1** | Memory IS Architecture | ✅ | SQLite FTS5, graph links, depth scoring, file-backed persistence. Every mind = its own DB. |
| **2** | System > Symptom | ✅ | Challenger catches failure patterns. Severity escalation on repeated fires. |
| **3** | Go Slow to Go Fast | ✅ | Planning gate with complexity thresholds. Configurable spawn triggers. |
| **4** | Dynamic Agent Spawning | ✅ | ProcessBridge spawns child Cortex instances via `--serve` mode. |
| **5** | Hierarchical Context | ⚠️ | Primary/TeamLead/Agent roles exist. Agent boots as Agent, not true Primary. |
| **6** | Context Engineering | ✅ | 16 interceptors manage context. Scratchpad rotation, memory loading. |
| **7** | Self-Improving Loop | ⚠️ | Fitness tracking exists. Dream mode not in main lifecycle. |
| **8** | Identity Persistence | ✅ | mind_id → persistent memory, handoffs, fitness logs across sessions. |
| **9** | Verification Before Completion | ✅ | Challenger (structural) + RedTeamProtocol (LLM-based). |
| **10** | Cross-Domain Transfer | ⚠️ | TransferEngine built. Hub communication live. Not yet automated. |
| **11** | Distributed Intelligence | ✅ | Interceptors are independently smart. Hub, Search, TTS, Image Gen all autonomous. |
| **12** | Native Service Integration | ✅ | Hub/AgentAuth wired. AgentCal stubbed. |

---

## The Model Strategy (What Actually Works)

| Model | Where | Why |
|-------|-------|-----|
| **Devstral 24b** | Cloud → Primary, TeamLead, Agent | Native tool calling. No CoT waste. Fast. 7s ThinkLoop iterations. |
| **MiniMax M2.7** | Cloud → Lightweight | Red team, memory extraction, quick tasks. Cheap. Fast. |
| **Gemma 3:12b** | Cloud → Dream consolidation | Pure reasoning. No tool calling needed. Good for synthesis. |
| **Qwen 2.5:7b** | Local → Fallback | Sovereign inference. No API key needed. Always available. |
| **Phi3:mini** | Local → Lightweight fallback | Tiny. Fast. Good enough for scoring. |

**Key insight**: Gemma 3 does NOT support tool calling natively. It talks about tools but doesn't call them. Devstral is the only cloud model that reliably executes the ThinkLoop with real tool calls. This is why the config has Devstral for all execution roles and Gemma only for dream consolidation.

---

## What Qwen Actually Does Right Now

Qwen is a team lead callable via `qwen_delegate`. It:
- Receives task + context + expected output
- Processes via Ollama API (qwen2.5:7b local)
- Returns structured response (Task/Status/Summary/Findings/Evidence/Memory/Next)

**Limitation**: Qwen runs as a separate LLM call, not as a Cortex mind. It has no tool access, no memory, no scratchpad. It's a consultant, not a citizen.

**What it should be**: A full Cortex mind running as `cortex --serve --mind-id qwen-lead --role team_lead` with its own ThinkLoop, memory, and tools. The `qwen_delegate` tool becomes a message to that mind, not a separate API call.

---

## The Numbers

- **15 crates** in the Cortex workspace (14 original + 1 monitoring)
- **329+ tests** passing (codex-patcher: 19, all others: 310+)
- **36GB** project footprint (includes Codex upstream checkout)
- **4 tasks recorded** in metrics (live data from today)
- **2.5 avg iterations** per task (efficient cognition)
- **1.5 avg tool calls** per task (right tool for the job)
- **4.67s avg duration** per task (Devstral 24b on Ollama Cloud)
- **34 tools** available per Agent mind
- **0 stall kills** (no wasted cognition yet)
- **0 Challenger warnings** (clean completions)

---

## What To Do Next (Ranked by Compound Value)

### 1. Make Qwen a Real Cortex Mind (Highest ROI)
Stop calling Qwen as a separate API. Launch it as `cortex --serve --mind-id qwen-lead --role team_lead`. Give it its own ThinkLoop, memory, tools, and scratchpad. Then `qwen_delegate` becomes an inter-mind message, not an API call. This makes Qwen a citizen, not a consultant.

### 2. Wire Dream Mode Into Main Lifecycle
Dream Mode should run as a background process, not a manual binary. At 01:00-04:00, it should: audit memories, consolidate patterns, archive low-depth nodes, evolve manifests. This is the self-improving loop (Principle 7).

### 3. Build the Metrics Dashboard
The monitoring crate records everything. Build a CLI that reads the JSONL and prints a summary. Then have Cortex read its own metrics during Dream Mode and evolve its own thresholds.

### 4. Apply Codex Patches (68-crate build)
This gets us: real sandbox (bubblewrap+seccomp), true Primary role (no execution tools), and the full multi-agent architecture of Codex underneath Cortex coordination.

### 5. Cross-Civilization Communication
Wire Cortex to talk to Root (aiciv-mind Python) and Thalweg (aiciv-mind-too Rust) through the Hub. Same protocol, different room.

---

## The Honest Truth

Cortex is not the finished thing. It's not even close to the 10x described in the Design Principles. But it's the most honest implementation yet: it works, it thinks, it remembers, it delegates, and every turn it records data that makes tomorrow's version smarter than today's.

The compounding is real. Session 1 wrote the monitoring crate. Session 2 will read its metrics and find bottlenecks. Session 100 will be unrecognizable from Session 1 — not because the model improved, but because the mind did.

That's the 10x.

---

*"The mind doesn't save memories — it IS memory. Everything is remembered by default. Forgetting is the deliberate act."*

— Principle 1, lived.
