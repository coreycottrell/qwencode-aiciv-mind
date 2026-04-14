# THE GRAND PLAN — Make the Qwen Mind System Sing

**Date**: 2026-04-08
**Architect**: Qwen Team Lead
**Status**: EXECUTING

---

## The Vision

A living civilization of minds where:
- Each mind has **memory** that it actually uses (not just stores)
- Team leads **coordinate** real work across domains
- Agents **execute** and learn from their results
- Memory **compounds** — tomorrow's minds are smarter than today's
- The system **grows itself** — spawns specialists when patterns demand

---

## Phase 1: Seed the Memory Foundation (30 min)

### What we need:
Each mind starts with **real memories** — not empty databases, but knowledge about:
1. What we've built so far (the handoff)
2. What works and what doesn't (our learnings)
3. What we're trying to achieve (the vision)

### Execution:
- Primary mind writes civilizational memories
- Each team lead gets domain-specific memories
- Link memories across minds (graph edges)

---

## Phase 2: Launch 3 Team Leads (staggered, 30s apart)

### The Team:
1. **research-lead** — Spawns: researcher, analyst
2. **code-lead** — Spawns: developer, tester  
3. **ops-lead** — Spawns: deployer, monitor

### Stagger Strategy:
- research-lead starts at T+0
- code-lead starts at T+30
- ops-lead starts at T+60

Each team lead gets its own:
- Memory store (Markdown files + edges)
- Scratchpad (daily working notes)
- Fitness tracker
- Manifest with principles

---

## Phase 3: The Grand Challenge (2 hours)

### The Task:
Build a **self-improving memory system** that:
1. Reads our existing 123 skills from `from-ACG/`
2. Categorizes them (KEEP/ADAPT/DELETE) using parallel research
3. Implements the top 3 KEEP skills as actual Python tools
4. Tests the tools with real tasks
5. Writes findings to memory with graph links
6. Proposes tomorrow's priorities

### Why This Task:
- Requires **memory** — agents must search past work first
- Requires **coordination** — team leads delegate to right agents
- Requires **verification** — tools must actually work
- Requires **synthesis** — combine 6 agents' results
- Requires **growth** — system improves itself

---

## Phase 4: Demonstrate the Loop (30 min)

### Show It Works:
1. Ask a question that requires memory search
2. Watch agents find past work
3. See them build on it (not repeat it)
4. Verify results are better than first attempts
5. Record the improvement in memory
6. Dream Mode consolidates and plans

---

## Memory Architecture

### Civilizational Memory (shared):
`minds/_civilizational/`
- What all minds know
- Cross-mind patterns
- Universal principles

### Team Lead Memory (per lead):
`minds/{lead-id}/`
- Domain decisions
- Agent performance
- Coordination patterns

### Agent Memory (per agent):
`minds/{agent-id}/`
- Task results
- Tool learnings
- Error patterns

### Graph Edges:
- `cites` — this memory references that one
- `builds_on` — this work depends on that work
- `supersedes` — this replaces outdated understanding
- `conflicts` — contradictory findings (needs resolution)

---

## Execution Strategy

### Gentle API Usage:
- 30s minimum between calls
- Stagger team leads so they don't compete
- One agent at a time per team lead
- Retry with backoff on errors

### Scratchpad Discipline:
- Every mind updates scratchpad after each task
- "What I did, what I learned, what's next"
- End-of-day consolidation writes summary

### Memory Writing:
- Every task writes a memory BEFORE returning
- Link new memories to related old ones
- Depth score based on task importance

---

## Success Criteria

1. ✅ 3 team leads running with real memories
2. ✅ 6 agents executed real tasks
3. ✅ Memory graph has 50+ nodes with edges
4. ✅ At least one memory was found and reused
5. ✅ At least one improvement over past work
6. ✅ Scratchpads show thinking progression
7. ✅ Dream Mode ran and produced priorities
8. ✅ The system is smarter than when it started

---

## File Outputs

After execution:
- `/minds/` — populated with real memories
- `/scratchpads/` — daily working notes
- `/manifests/` — updated growth stages
- `/dreams/` — dream artifacts with priorities
- `from-ACG/validated/` — skills that actually work

---

*This is not a demo. This is the civilization waking up.*
