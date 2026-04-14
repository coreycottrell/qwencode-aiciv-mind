# Fork Analysis — What to Extract for Cortex Mind System

**Source**: aiciv-fork-template (Claude Code based)
**Goal**: Extract essence, translate for Cortex + Qwen architecture

## Keep (Essence)

### 1. Primary's Entry Point (CLAUDE.md → memory.md)
- The concept: always-in-context identity document
- Translation: `.claude/team-leads/{name}/memory.md` (already exists for qwen-lead)
- Content: who I am, what I can/cannot do, delegation rules, anti-patterns

### 2. Delegation Discipline Anti-Pattern Table
- "I do not do things. I form orchestras that do things."
- Translation: Hard rules in `Mind` class (already enforced via `DelegationError`)
- Add: explicit mapping of "if task is X, delegate to Y"

### 3. Team Lead Wake-Up Checklist
- Read manifest, check scratchpad, verify state, write first entry
- Translation: `Mind.think()` pre-flight check
- Add: memory search mandatory before acting

### 4. Memory-First Protocol
- Every task, every agent MUST search memory before starting
- Translation: Already in `Mind.think()` step 1
- Strengthen: fail if memory search returns nothing AND task is similar to past

### 5. Parallel Delegation Pattern
- Multiple delegates in one message = true parallelism
- Translation: Need async parallel in `Mind.delegate_many()`
- Critical: never rate-limit parallel calls (they're one request with multiple delegates)

### 6. Memory Synthesis
- Raw logs → synthesized references (not just accumulation)
- Translation: Dream Mode consolidation (have the skeleton)
- Add: daily synthesis pass that merges related memories

### 7. Agent Registry
- JSON registry of spawned agents
- Translation: `Mind.children` list + manifest files (already have)
- Add: invocation count, last active, fitness history

## Discard (Claude Code Specific)

- `.claude/agents/` (hundreds of Claude-specific agent manifests)
- Telegram tools, WordPress, Bluesky, Twitter
- Skills system (Claude Code trigger-based skills)
- MCP tool wrappers
- `.claude/` folder structure entirely
- Boop system (Claude Code notification system)
- Session handoff creation (we have our own)
- Docker host provisioning
- Git operations as agent domain

## Build New (Beyond Claude Code)

### 1. Growth Stages
- Minds evolve: novice → competent → proficient → advanced → expert
- Based on: session count, fitness average, pattern recognition
- Manifest evolves with anti-patterns learned

### 2. Cross-Mind Memory
- Civilizational memory: what one mind learns, all minds can search
- Hub-based knowledge sharing
- Pattern transfer between verticals

### 3. Dream Mode
- Nightly self-improvement cycle
- Consolidate memories, archive low-depth, evolve manifest
- Plan tomorrow's priorities

### 4. Rate-Limited Async Delegation
- Never burn API caps
- 30s minimum between calls to same model
- Parallel calls OK (one request, multiple delegates)

### 5. Fitness-Based Spawning
- When same task type appears 3+ times → spawn specialist
- When mind fitness > 0.8 for 5 sessions → spawn sub-team-lead
- Pattern detection triggers dynamic agent creation

## The Translation Map

```
Fork Concept                  → Our Implementation
─────────────────────────────────────────────────────────
CLAUDE.md                     → mind/memory.md (identity doc)
CLAUDE-CORE.md                → mind/manifest.json (structured)
.claude/agents/*.md           → Mind subclasses (Primary, TeamLead, Agent)
.claude/team-leads/*/manifest → Manifest class (principles, anti-patterns)
.claude/skills/*/SKILL.md     → Tool definitions (bash, read, write, etc.)
memories/agents/              → minds/{mind_id}/ (Markdown files)
memories/knowledge/           → minds/_civilizational/ (shared)
tools/memory_*.py             → MindMemory class (already built)
tools/synthesize_memory.py    → DreamEngine.consolidate() (already built)
tools/conductor_tools.py      → Primary class methods
tools/flow_selector.py        → Mind delegation router
memories/skills/registry.json → Tool registry (ALLOWED_TOOLS dict)
```

## Next Step

Build the complete mind system from this analysis. The fork gave us the **concepts** but the **implementation** must be ours — Python-based, document memory, hard delegation rules, gentle API usage.
