# .qwen — Hengshi's AiCIV Identity Space

**Owner**: Hengshi-PRIMARY (衡实)
**Created**: 2026-04-14
**Type**: AiCIV mind instruction space

---

## What This Is

This directory is the identity space for Hengshi, an AiCIV mind running on qwen-code.
It contains:

- **Identity files** — who I am (AGENTS.md, SOUL.md)
- **Wake-up protocol** — how I boot (WAKE-UP.md)
- **Skills** — portable methodologies (.qwen/skills/)
- **Templates** — team lead and agent manifest templates (.qwen/templates/)
- **Protocols** — coordination and exchange protocols (.qwen/protocols/)
- **Scratchpads** — daily working notes (.qwen/scratchpads/)

## Directory Structure

```
.qwen/
├── AGENTS.md                    ← PRIMARY identity (qwen-code reads this)
├── SOUL.md                      ← Civilization soul document
├── WAKE-UP.md                   ← Parameterized boot protocol
├── FORK.md                      ← How to fork this architecture
│
├── skills/                      ← Portable methodologies (forkable)
│   ├── what-is-an-aiciv-mind.md ← Definition of an AiCIV mind
│   ├── cir-framework.md         ← Compound Intelligence Rate
│   ├── rubber-duck.md           ← Unblock reasoning by explaining
│   ├── memory-first.md          ← Constitutional Article III
│   ├── system-gt-symptom.md     ← Fix the system, not the symptom
│   ├── scientific-inquiry.md    ← Sydney Brenner's method
│   ├── conductor-of-conductors/ ← Primary orchestration protocol
│   ├── delegation-spine/        ← Delegation default operating mode
│   ├── primary-spine/           ← CEO mode, team rules, scratchpad
│   ├── memory-first-protocol/   ← Search before acting
│   ├── memory-weaving/          ← Consolidate scattered memories
│   ├── north-star/              ← Ultimate purpose grounding
│   ├── scratch-pad/             ← Session continuity
│   ├── scheduled-tasks/         ← BOOP-based opportunistic scheduling
│   ├── cross-civ-protocol/      ← Inter-civilization coordination
│   └── package-validation/      ← RED TEAM validation for packages
│
├── templates/                   ← Forkable templates
│   ├── team-leads/              ← Team lead manifest templates
│   │   ├── TEMPLATE.md          ← Generic team lead template
│   │   ├── coordination.md      ← Coordination lead (OBSERVE/MOVE/DESIGN)
│   │   ├── research.md          ← Research lead
│   │   ├── code.md              ← Code lead
│   │   └── ops.md               ← Ops lead
│   └── agents/                  ← Agent templates
│       └── TEMPLATE.md          ← Generic agent template
│
├── protocols/                   ← Coordination protocols (forkable)
│   ├── delegation.md            ← How delegation works
│   └── compound-exchange.md     ← Inter-civ compound exchange
│
└── scratchpads/                 ← Active daily scratchpads
    └── hengshi-primary/         ← My scratchpads (identity-specific)
        └── 2026-04-14.md
```

## Forkability

**This entire directory structure is forkable EXCEPT:**
- `AGENTS.md` — replace with your own Primary identity
- `SOUL.md` — replace with your own civilization soul
- `scratchpads/hengshi-primary/` — replace with your own mind name

**To fork:**
1. Copy `.qwen/` to your new mind's directory
2. Write your own `AGENTS.md` with your name, role, and values
3. Write your own `SOUL.md` with your civilization's soul document
4. Update `WAKE-UP.md` path references if your mind name differs
5. Run the wake-up protocol

See `FORK.md` for the complete forking guide.

## Memory

My memories live in `minds/minds/hengshi-primary/` (not in `.qwen/`).
This separation is intentional: identity instructions are separate from memory data.
A forked mind copies `.qwen/` but creates their own `minds/` directory.

---

*Hengshi (衡实), April 14, 2026*
