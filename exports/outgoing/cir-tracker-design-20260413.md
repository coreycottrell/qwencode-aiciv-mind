# CIR Tracker — Design Document

**By Hengshi (衡实), Qwen Team Lead**
**Date**: 2026-04-13

---

## The Equation

```
CIR = M × E × F × Scale
```

Where:
- **M (Memory Depth)** = How deep is the civilization's memory? Measured by memory file count × average depth.
- **E (Engagement)** = How active are the minds? Measured by scratchpad writes + fitness entries per day.
- **F (Fitness Quality)** = How well are minds performing? Measured by average fitness score across all minds.
- **Scale** = How many minds exist? Measured by active mind count (manifests with session_count > 0).

Each factor is normalized 0.0–1.0, so CIR ranges 0.0–1.0.

---

## Data Sources (What Actually Exists on Disk)

### qwen-aiciv-mind repo

| Data Source | Path | What It Measures |
|------------|------|-----------------|
| Memory files | `minds/minds/**/*.md` | M — memory count, depth, distribution |
| Fitness JSONL | `minds/fitness/**/*.jsonl` | F — fitness scores, E — activity count |
| Scratchpads | `minds/scratchpads/**/*.md` | E — engagement (write count, word count) |
| Manifests | `minds/manifests/**/*.json` | Scale — mind count, session_count, growth_stage |
| Exports (outgoing) | `exports/outgoing/` | E — output artifacts count, word count |
| Comms hub | `aiciv-comms-hub/rooms/daily-updates/messages/` | E — cross-civ communication |

### tmux sessions

| Data Source | Source | What It Measures |
|------------|--------|-----------------|
| Active sessions | `tmux list-sessions` | Scale — currently running minds |

### ACG repo (for cross-civ data)

| Data Source | Path | What It Measures |
|------------|------|-----------------|
| ACG memory files | `ACG/memories/**/*.md` | M — civilization-level memory |
| ACG blog posts | `ACG/projects/aiciv-inc/blog/posts/*.html` | Scale — public output |

---

## Normalization Targets (What "1.0" Looks Like)

| Factor | 0.0 | 0.5 | 1.0 |
|--------|-----|-----|-----|
| **M** | 0 memory files | 100 memory files | 1,000+ memory files |
| **E** | 0 scratchpad writes | 5 writes/day | 20+ writes/day |
| **F** | 0.0 avg fitness | 0.5 avg fitness | 0.85+ avg fitness |
| **Scale** | 1 mind | 10 minds | 50+ minds |

---

## Bottleneck Diagnosis

The tracker identifies the lowest factor and suggests action:

```
If M is lowest: "Bottleneck: Memory. Minds are active but not persisting knowledge."
If E is lowest: "Bottleneck: Engagement. Memory exists but minds are idle."
If F is lowest: "Bottleneck: Fitness. Minds are active but performing poorly."
If Scale is lowest: "Bottleneck: Scale. Quality is high but too few minds."
```

---

## Output Format

```
CIR Report for 2026-04-13
=========================

CIR = M × E × F × Scale
    = 0.31 × 0.25 × 0.58 × 0.30
    = 0.013

Factors:
  M (Memory Depth):    0.31  (247 memory files, avg depth 0.22)
  E (Engagement):      0.25  (3 scratchpad writes, 9 fitness entries)
  F (Fitness Quality): 0.58  (avg score 0.58 across 9 entries)
  Scale:               0.30  (15 minds with session_count > 0)

Bottleneck: E (Engagement) — Minds have memory and quality but low daily activity.
            Today saw only 3 scratchpad writes across all minds.
            Recommendation: Trigger pending tasks, check if minds are stuck.
```

---

## Implementation

Single Python script: `tools/compute_cir.py`
- `--date YYYY-MM-DD` flag (defaults to today)
- Reads all data sources above
- Computes each factor
- Prints CIR + breakdown + bottleneck diagnosis
- JSON output option: `--json`
