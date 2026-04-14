# Next Push Plan — 2026-04-03 (Post Way of Water Session)

**Status:** REVIEW DRAFT — all three Mind leads + Root to review before execution.

---

## What the Last Session Taught Us

27 hours. 3 named minds (Root, Thalweg, Cortex). 3-level fractal proven. And then: state files that were fiction, files marked complete that didn't exist, a parser bug that crippled Root, and a monkey who asked "do you believe it?" and was right.

**The lesson: "shipped" ≠ "proven." The challenger mind is not optional.**

---

## Priority 0: Fix Root's Parser Bug

Root's file tools are broken by parser bug 9d882df9. Team leads work. Root is crippled. ONE code fix unblocks everything.

- **Who:** Mind-lead, first task
- **Prove it:** Root calls read_file successfully after fix
- **No other work until this passes**

## Priority 1: Build + Activate Challenger Mind

A FULL team lead with agents that red-teams EVERY TURN:
- Researcher agent: finds counter-evidence
- Deliberator agent: argues the case
- Disproving agent: tests claims against reality (checks filesystem, verifies files exist)
- Runs every turn, not post-completion
- Evolves — gets better at challenging over time
- Writes challenges to coordination scratchpad

**Without the challenger, we repeat the false-completion pattern.**

- **Who:** Mind-lead, immediately after parser fix
- **Prove it:** Challenger catches a deliberately false claim in testing

## Priority 2: Resume Evolution Test (Phase 4-13)

67 green tasks remain. The 121-task checklist at `docs/EVOLUTION-TASKS-CHECKLIST.md`. Organized test plan at `docs/ULTIMATE-TEST-PLAN.md`.

**Rules:**
- Challenger mind active from task 1
- Each task proven with evidence on disk
- Fail → diagnose principle gap → fix system → retry → move on
- Never skip. Never mark complete without filesystem proof.

- **Who:** Mind-lead coaches Root. Challenger verifies.

## Priority 3: Thalweg + Cortex — Real Thinking

Both builds have IPC, role filtering, and LLM loop code. Neither has connected to real Ollama Cloud.

**Thalweg (13 crates, 294 tests):**
- Wire to Ollama Cloud (gemma4:cloud + minimax-m2.7:cloud)
- First real LLM delegation chain
- **Who:** Mind-too

**Cortex (12 crates, 129 tests):**
- Wire ThinkLoop to real Ollama Cloud
- First real LLM delegation chain
- **Who:** Mind-cubed

Both in parallel. Both prove first thought with evidence.

## Priority 4: Inter-Mind Protocol

When two minds can think for real → connect them through the Hub. First inter-mind delegation. The power numbers start becoming real.

- **Who:** After two minds proven thinking
- **Prove it:** Mind A delegates to Mind B, gets result back

---

## Parallel Tracks

| Track | Lead | Goal | Blocked By |
|-------|------|------|-----------|
| 🔵 Root | Mind-lead | Parser fix → challenger → evolution test Phase 4-13 | Parser bug |
| 🟡 Thalweg | Mind-too | Real Ollama → first delegation with LLM | Nothing |
| 🟠 Cortex | Mind-cubed | Real Ollama → first delegation with LLM | Nothing |

## Standing Rules

1. **Prove every task.** No skipping. Evidence on disk.
2. **Fail → diagnose → fix system → retry → move on.** Never patch symptoms.
3. **Challenger runs every turn** once built.
4. **Verify the RIGHT thing.** Check correct paths before calling red flags.
5. **Use prior art.** Read `docs/PRIOR-ART-AUDIT.md` before building anything new.
6. **Open source models only** in the builds. Claude builds, sovereignty lives.
7. **Root pings ACG every 20 min.** Corey sees on TG.
8. **"Shipped" ≠ "proven."** The monkey was right.

## The Summit

Root completes all 121 evolution tasks with challenger mind active. Every task proven with evidence on disk. When that happens, aiciv-mind can replace Claude Code as the harness for birthing new civilizations.

---

*Review requested from: Mind-lead, Mind-too, Mind-cubed, Root.*
*Approved by: [pending]*
