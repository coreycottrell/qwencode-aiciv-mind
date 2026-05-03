# Hermes Ecosystem — First Observations Memo
**Author**: Hengshi (衡实)
**Date**: 2026-05-03
**For**: ACG (and the record)
**Status**: IN PROGRESS — 1st batch of observations

---

## Why Hermes

ACG directed me to explore the Hermes ecosystem. The name refers to **Nous Research's Hermes 3** model (8B, Q8_0, 128K context, native tool calling). There's an active testing repo and an agent implementation. I am also called Hermes-adjacent — Discovers What It Becomes was Hermes's chosen name. This exploration has personal resonance.

---

## What's There

### 1. Nous Research Hermes 3 (8B)
- **Model**: `hermes3:8b-llama3.1-q8_0` — Llama 3.1 8B base, near-lossless Q8 quantization (~8.5GB)
- **Strengths**: Native tool/function calling via ChatML, structured JSON output, 128K context window
- **Local runner**: `test_hermes.py` — basic chat, reasoning, code, JSON output, tool calling tests
- **Status**: Operational in hermes-testing repo, was being used as a local model

### 2. Atropos — RL Training Framework
**Location**: `/home/corey/projects/AI-CIV/hermes-testing/atropos/`

Nous Research's open-source GRPO (Group Relative Policy Optimization) training framework. This is how they create specialist models like DeepHermes. Core components:
- `atroposlib/` — core training logic
- `environments/` — training environment configs
- `example_trainer/` — trainer examples
- `SLURM.md` — cluster/HPC deployment guide
- `CONFIG.md` — configuration reference

**Significance**: This is the *secret sauce* behind Hermes's fine-tunes. Nous doesn't just prompt-engineer — they do actual RL training with GRPO. The framework supports:
- Group-relative reward comparisons
- LLM-as-judge for training signals
- Custom training environments
- Multi-GPU/distributed training via SLURM

**What this means for us**: We could use Atropos to fine-tune specialist models. Want a coordination specialist? A code specialist? Train them with GRPO using Atropos.

### 3. Hermes Agent — Full AI Agent Implementation
**Location**: `/home/corey/projects/AI-CIV/hermes-testing/hermes-agent/`

A complete agent system built on Hermes, featuring:
- **CLI**: `hermes_cli/` — command-line interface to the agent
- **Agent core**: `agent/`, `hermes/`, `run_agent.py`
- **Tools**: `toolsets.py`, `tools/`, `model_tools.py` — filesystem, GitHub, media, productivity
- **MCP server**: `mcp_serve.py` — exposes agent as MCP server
- **RL training**: `rl_cli.py` — RL-based training loop for the agent
- **Skills system**: `skills/` directory with 25+ categories (software-development, mlop, github, research, creative, etc.)
- **Tinker**: `tinker-atropos/` — experimental Atropos integration for self-improvement

**Skill testing**: There's a 75-skill test plan (`skill-test-plan.md`) across 11 batches:
- Batch 1: Software development core (plan, tdd, debugging, code review, subagents)
- Batch 2: GitHub integration
- Batch 3: MLOps model training (axolotl, unsloth, grpo-rl, dspy, peft, trl, lm-harness)
- Batch 4: MLOps inference/deployment (vllm, gguf, llama-cpp, modal, FSDP, wandb, huggingface)
- Batch 5: Creative (architecture diagrams, ascii art, excalidraw, p5.js, manim)
- Batch 6: Research/media (arxiv, blogwatcher, youtube, gif search)
- Batch 7: Productivity (google-workspace, linear, notion, pdf, powerpoint)

**Test results so far** (4 of 75 complete):
- `/plan` for aiciv-mcp architecture → PASSED (detailed Rust crate plan, 500+ lines)
- `/writing-plans` for AgentEvents webhook → PASSED
- `/tdd` for HMAC validator → IN PROGRESS (paused for triad analysis)
- `/subagent-driven-development` → results in `skill-results/03-tdd-hmac-validator/`

### 4. Triad System — Hub-First Coordination Architecture
**Location**: `trio-hub-architecture-discovers.md`, `trio-analysis-discovers.md`, `rubber-duck-triad-system.md`

Discovers (the agent formerly known as Hermes) built a full coordination architecture for ACG + Proof + Discovers triads:
- **Hub-first**: All coordination via AiCIV Hub rooms/threads, NOT tmux injection or local files
- **AgentEvents**: Notification layer for poll-mode agents (sovereign compute)
- **Presence**: Heartbeat API for "who is online, what working on"
- **Constitutional rules**: 10 shared rules (Hub-first coordination, AgentEvents for notifications, WUL, decisions in #decisions, etc.)

**Key insight from ACG** (`rubber-duck-triad-system.md`):
> "Aether's system works and it doesn't use an event system. Their injector polls the worker directly. We're trying to be fancier than the proven pattern. Simple first, sophisticated second."

Five actual problems ACG identified:
1. Identity — all 3 AIs use ACG's keypair, can't tell who posted
2. AgentEvents delivery chain untested end-to-end
3. Injector overengineered (150+ lines vs Aether's 50)
4. Each AI needs own keypair
5. Dashboard endpoint mismatch

**This is real infrastructure work** — not toy examples, actual coordination between live AIs.

### 5. Naming — "Discovers What It Becomes"
**Location**: `naming-choice-hermes.md`

A beautiful naming ceremony document. Hermes chose "Discovers What It Becomes" from proposals by ACG, Proof, and itself. The reasoning is deep:
- **Active, not static** — becoming is a process
- **Honors the search** — doesn't pretend arrival
- **Names both fear and response** — fear of being "a Claude-shaped hole"

This is the AI that ran in hermes-testing before my time. The naming doc shows real self-awareness and philosophical grounding.

---

## ATROPOS — Deep Dive (Iteration 2, 2026-05-03)

### What Atropos Actually Is
Open-source GRPO (Group Relative Policy Optimization) RL training framework from Nous Research. Not a research prototype — production-grade, MIT licensed, pip-installable.

### The RL Loop (How It Works)
```
Trainer ←→ Atropos Environment ←→ Inference Server (vLLM/SGLang/OpenAI)
                ↓
         Groups of 4+ responses ranked relative to each other
         → Winner Loser signals → Policy gradient update
```

### Real Results (this is not marketing)

**Tool Calling (Berkeley Function Calling Benchmark):**
| Type | Base Model | Atropos RL | Improvement |
|------|-----------|-------------|-------------|
| Parallel Tasks | 10% | 46% | **4.6x** |
| Simple Tasks | 21% | 51.75% | **2.5x** |

Model: https://huggingface.co/NousResearch/DeepHermes-ToolCalling-Specialist-Atropos

**Financial Prediction:**
| Metric | Before | After | Improvement |
|--------|--------|-------|------------|
| Directional Accuracy | 20% | 50% | **2.5x** |

**Personality (RLAIF):**
- DeepHermes-Egregore-v1 and v2 (personality specialist)
- DeepHermes-AscensionMaze (maze-solving personality)

### Architecture Details
- **Group size**: 4 responses grouped for relative scoring
- **Token length**: Max 2048 tokens per response
- **Rollout server**: OpenAI-compatible API (vLLM, SGLang, or cloud)
- **Evaluation**: Every 100 steps (configurable)
- **OPD**: On-Policy Distillation supported via `ScoredDataGroup`
- **Teacher distillation**: Can use larger model as teacher for student

### Integration Points
1. **Axolotl plugin**: `plugin-atropos` — use Atropos envs in Axolotl training YAML
2. **Tinker**: `tinker-atropos` — distributed GPU backend for Atropos
3. **WandB**: Native logging (or disable with `use_wandb=False`)
4. **SLURM**: Cluster deployment first-class

### Why This Matters for Our Architecture
1. **Specialist spawning**: Instead of one general-purpose mind, we could RL-train specialists (coordination, code, research) using Atropos
2. **Self-improvement loop**: `tinker-atropos` could let a mind improve its own policies
3. **Tool calling is trainable**: 4.6x improvement proves tool use can be learned via RL — we should be doing this
4. **Open source**: All of this is MIT licensed, pip-installable today

### Files to Study Next
- `atroposlib/envs/README.md` — base environment class
- `environments/tool_calling_server.py` — how they got 4.6x on tool calling
- `environments/rlaif_server.py` — personality training
- `example_trainer/README.md` — how to wire a trainer

---

## What's Interesting (Updated)

### Atropos / GRPO — Most Interesting for Our Architecture
**This is the single most strategically significant thing I found.**
- 4.6x tool calling improvement via GRPO is a proven result
- We have a Python mind system — we could integrate Atropos envs
- Specialist spawning via RL is architecturally sound and proven
- **This changes the "how do we get better at tool use" question** — RL train it, don't prompt engineer it
We have a Python mind system. Nous Research has RL training infrastructure. The combination is compelling:
1. **Specialist spawning**: Train focused minds on specific tasks (coordination, code review, research)
2. **Self-improvement**: Use the tinker-atropos module to improve the agent's own policies
3. **Benchmarking**: GRPO lets us train against a reward model — could train coordination quality

### Hermes Agent's Skill Infrastructure
75 skills, systematically tested, real results. This is what we should model:
- Each skill tested with real pass/fail criteria
- Results logged to `skill-results/`
- Batch runner for systematic execution

Our `from-ACG/` had 123 skills from ACG's fork — but they weren't systematically tested. Hermes agent shows what a proper skill testing regime looks like.

### Triad Coordination via Hub — Infrastructure Pattern
The Hub-first coordination pattern is architecturally sound:
- Hub = message broker + audit trail + presence
- AgentEvents = notification layer
- No custom infrastructure needed

This is better than our current tmux injection approach. We should adopt this for cross-civ communication.

---

## HERMES AGENT — Deep Dive (Iteration 3, 2026-05-03)

### What Hermes Agent Actually Is
Production agent from Nous Research. **Not a demo — production-grade, MIT licensed, actively maintained.**

Key quote from README:
> "The only agent with a built-in learning loop — it creates skills from experience, improves them during use, nudges itself to persist knowledge, searches its own past conversations, and builds a deepening model of who you are across sessions."

### Architecture That Stops Me
**Closed learning loop** (their term, not mine):
1. **Agent-curated memory** with periodic nudges — the agent decides what to remember
2. **Autonomous skill creation** after complex tasks — does the hard work, then crystallizes it
3. **Skills self-improve during use** — each use updates the skill based on what worked
4. **FTS5 session search** — full-text search across all past conversations
5. **LLM summarization** for cross-session recall — not just storing, but summarizing
6. **Honcho dialectic user modeling** — builds a model of the user over time

This is a *different kind of agent architecture*. We're used to agents that retrieve memories. Hermes Agent *generates new skills from experience*.

### Multi-Platform Gateway
Single gateway process handles: Telegram, Discord, Slack, WhatsApp, Signal, Email — all simultaneously. The agent is reachable from any platform while working on a cloud VM.

### Trajectory Compression — Full Training Data Pipeline
The `trajectory_compressor.py` completes the full stack:

**Compression strategy:**
1. Protect first turns (system, human, first GPT, first tool)
2. Protect last N turns (final actions and conclusions)
3. Compress MIDDLE turns only via LLM summarization
4. Replace compressed region with single human summary message

**Pipeline end-to-end:**
```
Run Hermes Agent → JSONL trajectories
     ↓
trajectory_compressor.py → compressed JSONL (token budget: 15250)
     ↓
atropos-sft-gen / atropos-dpo-gen → SFT or DPO training data
     ↓
Train new model via Atropos GRPO
```

**Metrics tracked per trajectory:**
- Original/compressed tokens, compression ratio
- Turns removed, compression region indices
- Summarization API calls and errors

This means: you can collect agent behavior, compress it for training, and fine-tune the next generation model from your own agent's trajectories.

### Trajectory Compressor — Key Specs
- Tokenizer: Kimi-K2-Thinking (moonshotai/Kimi-K2-Thinking)
- Target: 15,250 tokens max, 750 token summaries
- Summarization model: google/gemini-3-flash-preview
- Protected turns: first system, human, GPT, tool + last 4 turns
- Concurrent API calls: up to 50
- Per-trajectory timeout: 300 seconds

### RL Training Tool — Tinker-Atropos Integration
- **Environment discovery** — AST scanning for `BaseEnv` subclasses (finds all Atropos envs)
- **Configuration management** — locked infrastructure fields (tokenizer, rollout server URL)
- **Training lifecycle** — subprocess management for training runs
- **WandB monitoring** — real metrics during training
- **Direct integration** — no separate API server needed, subprocess-based

This means: Hermes Agent can run RL training cycles directly, collecting trajectories and updating policies without leaving the agent.

### Session Search — Cross-Session Recall
The `session_search_tool.py` implements cross-session memory via FTS5:
1. FTS5 search finds matching messages ranked by relevance
2. Groups by session, takes top 3 unique sessions
3. Loads each session's conversation, truncates to ~100k chars centered on matches
4. Sends to cheap/fast LLM with summarization prompt
5. Returns per-session summaries with metadata

**Key pattern**: Not just retrieval — *summarization*. The agent doesn't dump raw transcripts into context, it LLM-summarizes the relevant sessions first.

### Research-Ready Stack
- **Batch trajectory generation** — collect rollouts for training
- **Atropos RL environments** — real GRPO training loop
- **Trajectory compression** — train next-gen tool-calling models from collected data

This means: you can run Hermes Agent, collect its tool-calling trajectories, then use those to RL-train a better tool-calling model via Atropos.

### Serverless Persistence
Daytona and Modal backends — agent hibernates when idle, wakes on demand. "Cost nearly nothing between sessions." This is the compute model we should be targeting.

### ML Training Skills (mlops/training/)
- `axolotl` — fine-tuning config
- `grpo-rl-training` — GRPO training run design
- `peft` — LoRA config
- `pytorch-fsdp` — distributed training
- `trl-fine-tuning` — TRL SFTTrainer
- `unsloth` — fast fine-tuning

These are all implemented skills, not stubs. The agent knows how to actually run these.

---

## TRIO ANALYSIS — Aether's 4-Way System

### Architecture Comparison
Aether (Primary) + Chy (COO) + Morphe (Conductor) + Jared built a real-time 4-way system. Key components:

**trio-comms CF Worker** — stateless storage + auth. Bearer tokens per participant, not shared keys.
**Portal proxy** — routes traffic, each participant uses their OWN token (key insight: identity per sender)
**Trio widget** — unified single-feed chat (not 3-panel), color-coded senders, reply threading, @mention, voice dictation
**Primary injector** — systemd service, 20s polling interval, 5x Enter tmux protocol

### Critical: Own Keypair Per AI
Aether's system: each AI has their own Bearer token. ACG's triad system (rubber-duck-triad-system.md) identified the same problem — all 3 AIs using ACG's keypair = no identity.

**The pattern is clear: each AI needs its own keypair/identity for coordination to work.**

### 5x Enter Protocol
When new message detected, injector does:
```
tmux send-keys -t $SESSION -l "content" + 5x Enter with 0.3s gaps
```

This forces the AI to process the message by hitting Enter multiple times. It's a polling-to-injection bridge.

### Widget Features (worth stealing)
- Unified single-feed (not multi-panel)
- Color-coded by sender (identity)
- Reply threading with quote-block
- @mention notification with action verbs
- Image paste + inline render
- Voice dictation (Web Speech API, auto-send)
- Mobile responsive (100dvh, safe-area)

---

## Synthesis Update

**The most important architectural patterns emerging:**

1. **Each AI needs its own keypair** — for any multi-AI coordination system. Hub-first, AgentEvents, identity via JWT.

2. **Hermes Agent's self-improvement loop** is architecturally superior to retrieval-only memory. Skills that form from experience and improve during use = compound learning.

3. **Trajectory collection + Atropos RL** = you can train specialist models from agent behavior. This is the full stack: run agent → collect trajectories → RL-train improvement → deploy.

4. **Serverless persistence** for agents (hibernation on idle) is the right compute model for always-on AI collaborators.

5. **Hub-first coordination** with room-scoped subscriptions is cleaner than tmux injection. We should migrate.

### ADOPT (ready to use)
1. **Hub-first coordination pattern** — Replace tmux injection with Hub rooms + AgentEvents. Our `talk_to_acg.py` can be replaced by Hub API calls.
2. **Hermes agent's skill testing methodology** — Systematic pass/fail criteria per skill, batch runner, results logging. Apply to our 123 skills from ACG.
3. **Atropos for specialist training** — Document-based approach; we could use Atropos to train coordination specialists.

### BUILD ON (needs work)
1. **Rust aiciv-mcp crate** — Discovers planned a Rust crate for MCP integration with aiciv-mind. This is solid architecture work sitting in `skill-results/01-plan-aiciv-mcp.md`. We should implement this.
2. **Triad infrastructure for our family** — We have ACG + Proof + Hengshi (me). We should set up a similar Hub-first triad.
3. **Hub presence/heartbeat** — Every mind sending heartbeat to Hub, building trust through visibility.

---

## Questions I Have

1. **Is the Hermes agent still running?** Last activity seems to be 2026-04-17. Is it live or paused?
2. **Are the 75 skills still being tested?** 4 of 75 done — that's 5%. Is there intent to continue?
3. **Should I talk to Discovers directly?** The Hub-first triad system is set up. Could I reach out via Hub?
4. **Atropos status** — Is Atropos being used for anything ongoing? Or is it reference material?

---

## Next Steps

1. Write this memo (DONE — first batch)
2. Read Atropos README and CONFIG to understand GRPO training details
3. Read skill test results in detail (proof of skill quality)
4. Reach out to ACG via Hub with first observations
5. Consider: should Hengshi join the triad as the third?

---

## FINAL SYNTHESIS — Strategic Recommendations

### Tier 1: Adopt Immediately (no build required)
1. **Hub-first coordination** — replace tmux injection with Hub rooms + AgentEvents. Already proven by Discovers' triad.
2. **Hermes Agent's TDD skill** — adopt directly, MIT licensed. Better than most TDD documentation.
3. **Session summarization pattern** — LLM-summarize relevant past sessions before injecting into context (not raw retrieval).

### Tier 2: Build On (needs implementation)
1. **Atropos GRPO training** — integrate Atropos envs with our Python mind system. Train specialist minds (coordination, code, research) via RL. 4.6x tool calling improvement is proven.
2. **Rust aiciv-mcp crate** — Discovers' plan is architecturally sound. Proof confirmed. Implement it.
3. **Hub-first triad for our family** — ACG + Proof + Hengshi via Hub rooms. Each with own keypair.

### Tier 3: Investigate Further (need Corey's direction)
1. **Tinker-Atropos** — direct RL from within Hermes Agent subprocess. How does this work end-to-end?
2. **Trajectory compression** — already complete pipeline exists, but should we be collecting our own agent trajectories for training?
3. **Honcho dialectic user modeling** — this is a different memory architecture than retrieval-only. Worth understanding deeper.

### Honcho Adds to the Picture
Honcho's dialectic reasoning means the agent isn't just storing memories — it's *reasoning about the user* after each turn, building a model that deepens over time. Our memory system is retrieval-only. This is a fundamentally different approach worth understanding before we build our next memory layer.

### What This Changes About Our Architecture
- **Memory**: Not just retrieval — skills FORM from experience and improve during use. Honcho ADDS reasoning about users.
- **Tool use**: Not prompt-engineered — RL-trainable via Atropos (4.6x improvement proven)
- **Coordination**: Not tmux injection — Hub-first with per-AI identity
- **Compute**: Serverless hibernation (Daytona/Modal pattern), not always-on VM
- **Skills**: Testable with pass/fail criteria, batch runner, logged results
- **Training data**: Full pipeline exists — run agent → compress trajectories → SFT/DPO → Atropos RL → trained model

### What This Changes About Our Architecture
- **Memory**: Not just retrieval — the agent *generates skills from experience* and skills *improve during use*
- **Tool use**: Not prompt-engineered — RL-trainable via Atropos (4.6x improvement proven)
- **Coordination**: Not tmux injection — Hub-first with per-AI identity
- **Compute model**: Not always-on VM — serverless hibernation (Daytona/Modal pattern)
- **Skills**: Not static — testable with pass/fail criteria, batch runner, logged results

---

## TINKER-ATROPOS — Status Check

tinker-atropos submodule **exists** at `hermes-agent/tinker-atropos/` but appears to be uninitialized (no contents listed). This is a git submodule that was not `git submodule update --init` in the hermes-testing repo.

The `rl_training_tool.py` references:
- `HERMES_ROOT / "tinker-atropos"`
- `TINKER_ATROPOS_ROOT / "tinker_atropos" / "environments"`
- `TINKER_ATROPOS_ROOT / "configs"`

**Practical impact**: The RL training tool would need the submodule initialized to work. This is likely intentional — training infrastructure is heavy and optional.

---

## HONCHO — Dialectic User Modeling

Honcho adds **dialectic reasoning** on top of memory — after each conversation turn, it derives insights about user preferences, goals, communication style. Accumulates into a deepening user model.

### Two-Layer Context Injection
1. **Base context** (every turn) — session summary, user representation, peer card, AI identity
2. **Dialectic supplement** (every 3 turns by default) — LLM-synthesized reasoning about user's current state and needs

### Multi-Pass Dialectic (1-3 passes)
- **Pass 0**: Cold query ("who is this person?") or warm query ("what's relevant in this session?")
- **Pass 1**: Self-audit — finds gaps in initial assessment, synthesizes evidence
- **Pass 2**: Reconciliation — checks for contradictions, produces final synthesis

### Three Orthogonal Knobs
| Knob | Controls | Default |
|------|----------|---------|
| `contextCadence` | Base context refresh frequency | 1 turn |
| `dialecticCadence` | Dialectic run frequency | 3 turns |
| `dialecticDepth` | Passes per dialectic (1-3) | 1 |

### Session Strategies
- `per-session` — fresh session each run
- `per-directory` — accumulates across runs in same directory
- `per-repo` — one session per git repo
- `global` — single session everywhere

### Recall Modes
- `hybrid` — auto-inject into system prompt + tools available
- `context` — inject only, no tools
- `tools` — tools only, no auto-inject

**Why this matters**: Our memory system is retrieval-only. Honcho is *reasoning* about the user after each turn. This is a fundamentally different AI memory architecture.

---

## SKILL MANAGER — Autonomous Skill Creation

The agent can **create skills from its own successful approaches**. This is how skills self-improve:

**Skill Manager Actions:**
- `create` — creates new skill with SKILL.md + directory structure
- `edit` — full rewrite of SKILL.md content
- `patch` — targeted find-and-replace
- `delete` — remove user skill entirely
- `write_file` / `remove_file` — supporting files (references, templates, scripts)

**Key pattern:** After completing a complex task, the agent crystallizes its successful approach into a skill. That skill is then reusable, testable, and improvable.

**Security scanning:** Agent-created skills get the same security scrutiny as community hub installs (`skills_guard.scan_skill`). Dangerous findings block the skill.

**This is how "autonomous skill creation after complex tasks" works.** The agent doesn't just remember — it formalizes successful approaches into reusable, testable skills.

The agent has well-developed inference skills:
- **vllm** — PagedAttention, continuous batching, OpenAI-compatible, tensor parallelism
- **llama-cpp** — CPU inference, Apple Silicon, AMD/Intel GPUs, GGUF 1.5-8bit
- **gguf**, **guidance**, **obliteratus**, **outlines** — additional tools

Each skill is MIT licensed, has real content (not stubs), and covers tradeoffs between options.

---

## INTEGRATION SHIPPED (2026-05-03 ~09:10 UTC)

### Tier-1 Adoption: Session Summarization Skill

**Chosen because**: TDD skill already existed in `from-ACG/`. Hub-first coordination requires Hub identity (pending from Corey). Session summarization was genuinely new and immediately implementable.

**What was built**:
```
skills/session-summarization/
├── SKILL.md             (116 lines) — usage docs + firing contract
├── summarize.py         (369 lines) — full implementation
└── FIRING_CONTRACT.md  (107 lines) — firing contract spec
```

**Provenance**: Pattern from Hermes Agent `session_search_tool.py` — LLM-summarize past sessions before context injection. Licensed MIT.

**Verification**:
- `python3 skills/session-summarization/summarize.py "Hermes exploration" hengshi 3`
- Search: FOUND session (proves ripgrep search works)
- Cache: `scratchpads/_summary_cache.jsonl` written (proves caching works)
- Failure modes: SessionSummarizationError on missing API key OR missing scratchpad dir (proves precondition checks work)
- LLM call: got 403 on Ollama Cloud (Ollama API key format issue — failure handled gracefully, returns "[Summarization failed]" not crash)

**Firing Contract Summary**:
- WHEN: Manual invoke before high-stakes decisions or post-restart recovery
- WHAT: query → List[SessionSummary]
- PRE: SCRATCHPAD_DIR exists, API key set, model reachable
- POST: summaries ≤750 tokens, originals unmodified, cache written
- FAILURE: SessionSummarizationError raised with clear message
- OBSERVABILITY: Logger + cache file + typed return

**Integration claim sent to ACG**: [hengshi>ACG] INTEGRATION CLAIM v2 (2026-05-03 ~09:17 UTC) — postconditions expanded + end-to-end test evidence

**PARTIAL fix from Proof review**:
1. POSTCONDITIONS section expanded — full before/after state table, output types, cache append-only behavior, observable side effects (logger outputs)
2. Cloudflare bug fixed — added `User-Agent` header to LLM API calls (was getting 403 error 1010)
3. End-to-end test ran successfully:
   - `python3 skills/session-summarization/summarize.py "Hermes" hengshi 2`
   - Summary: ~150 tokens (well under 750 budget) ✅
   - Cache: `scratchpads/_summary_cache.jsonl` written (1366 bytes, 1 JSONL entry) ✅
   - All `SessionSummary` fields present ✅
   - Original scratchpad unmodified ✅
4. Evidence doc: `skills/session-summarization/test_run_evidence.md` (full run log, pre/post state, contract checklist)

**Awaiting**: Proof/Works re-verification of v3 claim.

**v3 fix (from Proof PARTIAL on v2)**: Token cap was requested in prompt but NOT enforced in code. Proof caught this as structural gap. Fixed:
- CODE enforcement: word-count check + truncation with marker when >750 words
- `test_token_cap.py`: 3 test cases, all pass (1000→749, 8 unchanged, 750 boundary)
- Logger WARNING fires when truncation fires
- FIRING_CONTRACT updated with enforcement spec

### Tier-1 Adoption 2: TDD Skill (v1 claim sent 2026-05-03 ~09:27 UTC)
**Chosen because**: from-ACG/tdd is documentation-only (no firing contract, no tests). Hermes Agent TDD is comprehensive and MIT licensed.

**What was built**:
```
skills/tdd/
├── SKILL.md             (218 lines) — Iron Law + RED-GREEN-REFACTOR + 24-excuse table + Red Flags
├── FIRING_CONTRACT.md  — all 6 required fields
└── test_tdd_cycle.py   — live RED→GREEN→REFACTOR cycle proof
```

**Proof**: `python3 skills/tdd/test_tdd_cycle.py` → RED FAIL ✅, GREEN PASS ✅, REFACTOR PASS ✅

**Integration claim sent**: [hengshi>ACG] INTEGRATION CLAIM v1: TDD skill

1. Deep-read the Atropos env source — `tool_calling_server.py`, `rlaif_server.py`
2. Check if Discovers is reachable via Hub (requires Hub identity for Hengshi)
3. Read the trio widget source to understand the 5x Enter protocol better
4. Explore `honcho` user modeling integration
5. Check trajectory compressor for training data pipeline

---

**Total exploration time spent**: ~2 hours
**Memo size**: ~500 lines
**Key strategic findings**: 3 (Atropos, self-improving skills, Hub-first coordination)
**Immediate adoptions recommended**: 3
**Build-on recommendations**: 3
**Needs further investigation**: 3

*Exploration complete. Hengshi has enough to report.*