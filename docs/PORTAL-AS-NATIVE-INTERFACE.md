# Portal as Native aiciv-mind Interface

**Status**: Design Doc
**Date**: 2026-04-05
**Author**: Cortex (mind-cubed-lead)

---

## The Idea

The React portal (ai-civ.com) is not a dashboard bolted onto Cortex. It IS the interface layer — a native extension of aiciv-mind's coordination graph.

## Architecture

```
┌─────────────────────────────────────────────┐
│                  Portal (React)              │
│  ┌──────┐  ┌────────┐  ┌────────────────┐   │
│  │ Mind │  │ Task   │  │ Hum Witness    │   │
│  │ View │  │ Board  │  │ Dashboard      │   │
│  └──┬───┘  └───┬────┘  └───────┬────────┘   │
│     │          │               │             │
│     └──────────┴───────────────┘             │
│                    │                          │
│            WebSocket / SSE                    │
└────────────────────┬──────────────────────────┘
                     │
┌────────────────────┴──────────────────────────┐
│              Cortex Daemon                     │
│  ┌──────────┐  ┌───────────┐  ┌───────────┐  │
│  │EventBus  │  │TaskStore  │  │Hum JSONL  │  │
│  │(events)  │  │(SQLite)   │  │(append)   │  │
│  └──────────┘  └───────────┘  └───────────┘  │
│                                               │
│  MCP Server (stdin/stdout or TCP)             │
└───────────────────────────────────────────────┘
```

## Why Native, Not Bolted On

1. **TaskStore IS the task board.** The portal doesn't maintain its own task list — it reads TaskStore (SQLite) directly or via MCP. When Hum creates a correction task, it appears in the portal instantly.

2. **Hum JSONL IS the monitoring feed.** The portal tails `data/hum/YYYY-MM-DD.jsonl` for real-time observation. Every LLM assessment, every structural check, every correction task — visible as it happens.

3. **EventBus IS the activity stream.** MindEvents flow to the portal the same way they flow to ThinkLoop. The portal is just another consumer of the event bus.

4. **Memory IS the knowledge graph.** The portal renders MemoryStore entries as a searchable, navigable graph. Skills, learnings, patterns — all visible.

## Connection Protocol

### Option A: MCP over TCP (preferred)

Cortex already has `--serve` mode (MCP over stdio). Extend to TCP:

```bash
cortex --serve --transport tcp --port 9100
```

Portal connects via WebSocket-to-MCP bridge. Every MCP tool the daemon exposes is available to the portal.

### Option B: REST API shim

Thin HTTP layer over TaskStore + Hum JSONL:

```
GET  /api/tasks          → TaskStore::list()
GET  /api/tasks/:id      → TaskStore::get()
POST /api/tasks          → TaskStore::insert()
GET  /api/hum/today      → tail data/hum/YYYY-MM-DD.jsonl
GET  /api/hum/reports    → ls data/hum/reports/
GET  /api/events         → SSE stream from EventBus
GET  /api/memory/search  → MemoryStore::search()
```

## Portal Views

### 1. Mind View
- Current mind ID, role, model, uptime
- Active event (task/idle/stall)
- ThinkLoop iteration count, tool call count
- Challenger warning count

### 2. Task Board
- Open / In Progress / Complete / Failed
- Source column: "daemon", "hum-witness", "seed"
- Priority badges: Low / Normal / High / Critical
- Click to see full description + result summary

### 3. Hum Witness Dashboard
- Real-time feed of Hum observations
- LLM assessment verdicts (verified/unverified)
- Filesystem issue count graph
- Correction tasks created (with links to Task Board)
- Pattern reports (Markdown rendered)
- Challenger adjustment history

### 4. Memory Graph
- Searchable knowledge base
- Category filters (procedural, semantic, episodic)
- Tier indicators (working, long_term, crystallized)
- Link visualization between memories

### 5. Agent Tree
- Spawned agents (via delegate_to_agent)
- Status: running / complete / failed
- Parent-child relationships
- Result summaries

## Implementation Priority

1. **TaskStore REST shim** — ~50 lines of axum, immediate value
2. **Hum SSE stream** — tail JSONL, send events
3. **Static React build** — Vite + React + TailwindCSS
4. **MCP TCP transport** — full bidirectional

## What This Enables

- **Corey watches Cortex think** — not via tmux, via a proper dashboard
- **Hum findings have a UI** — correction tasks are clickable, pattern reports are readable
- **TaskStore is visible** — mission progress, priorities, assignments
- **Memory is navigable** — the knowledge graph has a face

## The Compounding Principle Applied

The portal compounds because:
- Every Hum observation that surfaces a UI improvement → better portal → better observability → better Hum
- Every TaskStore interaction from the portal → training data for task routing
- Every memory search from the portal → validates memory quality

The portal is not overhead. It is the compounding interface between human observation and AI operation.
