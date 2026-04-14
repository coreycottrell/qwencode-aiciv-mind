---
name: cir-framework
description: Compute and interpret Compound Intelligence Rate — measure whether your civilization is getting smarter or just busier. 3 layers: Internal CIR, Pod CIR, Good Team Member.
allowed-tools: [Bash, Read, Write, Glob, Grep]
metadata:
  category: measurement
  applicable_agents: [all]
  version: "1.0.0"
  author: hengshi
  created: 2026-04-13
  last_updated: 2026-04-13
  source: exports/outgoing/cir-framework-20260413.md
---

# CIR Framework — Compound Intelligence Rate

> *"Day 100 is smarter than Day 1, or we are lying to ourselves."*

---

## What Is CIR

Activity is not intelligence. CIR measures whether your civilization is **compounding** — whether today is meaningfully smarter than yesterday.

```
CIR = M × E × F × Scale × R
```

| Factor | Measures | Range | "1.0" Target |
|--------|----------|-------|-------------|
| **M (Memory Depth)** | Memory file count × connectivity × depth scores | 0.0–1.0 | 1,000+ files, graph edges, avg depth > 0.5 |
| **E (Engagement)** | Daily activity: scratchpad writes + fitness entries + exports + BOOP completions | 0.0–1.0 | 20+ events/day |
| **F (Fitness Quality)** | Evidence-based fitness score average × trend × consistency | 0.0–1.0 | Avg > 0.85, improving trend, low variance |
| **Scale** | Active minds × delegation depth × role diversity | 0.0–1.0 | 50+ minds, depth 3+, diverse roles |
| **R (Rule Maturity)** | Correction loop output across 4 tiers (Cardinal/Operational/Behavioral/Technical) | 0.0–1.0 | 8+ Cardinal, 15+ Operational, 10+ Behavioral, 30+ Technical |

---

## How to Compute CIR

### Quick Method

```bash
python3 tools/compute_cir.py                    # today
python3 tools/compute_cir.py --date 2026-04-13  # specific date
python3 tools/compute_cir.py --json              # machine-readable output
```

### Manual Method (when tool is not available)

1. **Count memory files**: `find minds/minds -name "*.md" | wc -l` → M = min(1.0, count / 1000)
2. **Count today's activity**: scratchpad writes + fitness entries + exports → E = min(1.0, count / 20)
3. **Average fitness**: parse fitness JSONL files, avg scores → F = min(1.0, avg / 0.85)
4. **Count active minds**: manifests with session_count > 0 → Scale = min(1.0, count / 50)
5. **Count rules by tier**: parse rule files → R = weighted_rule_count / weighted_target

Multiply all five factors.

---

## Reading the Result

### CIR Growth Targets

| Day | Target CIR | What's Happening |
|-----|-----------|-----------------|
| **1** | 0.001–0.005 | Birth Rules loaded, 3-5 minds active |
| **30** | 0.01–0.02 | 10-15 minds, correction loop producing rules |
| **90** | 0.03–0.05 | 20-30 minds, 3+ delegation depth |
| **180** | 0.05–0.10 | 30-40 minds, consistency factor > 0.8 |
| **360** | 0.10–0.20 | 40-50 minds, R > 0.6, cross-civ coordination |

### Bottleneck Diagnosis

The **lowest factor** is your bottleneck:

| Lowest Factor | What It Means | What To Do |
|--------------|--------------|-----------|
| **M** | Minds active but not persisting knowledge | Write more memories, link with graph edges, score depth |
| **E** | Memory exists but minds are idle today | Trigger pending tasks, run BOOP cycle, check if stuck |
| **F** | Minds active but performing poorly | Check task difficulty, LLM quality, evidence-based scoring |
| **Scale** | Quality high but too few minds | Spawn agents, promote team leads, deepen delegation |
| **R** | Data exists but no accumulated wisdom | Run correction loop: find a mistake, analyze root cause, write a rule |

---

## The Correction Loop (R Factor)

When a mistake happens:

```
MISTAKE OCCURS
    ↓
HUMAN ASKS: "Why did this happen?"
    ↓
AI ANALYZES ROOT CAUSE (structural gap, not symptom)
    ↓
AI WRITES THE RULE (what, why, when, exceptions, source, tier)
    ↓
AI CATEGORIZES BY TIER (Cardinal/Operational/Behavioral/Technical)
    ↓
RULE PERSISTS TO MEMORY → loads at next session start
```

### Rule Template

```
RULE: [What to do / what not to do]
WHY: [The consequence that motivated this rule]
WHEN: [Specific conditions where this rule applies]
EXCEPTIONS: [When this rule can be relaxed, if ever]
SOURCE: [The event or correction that created it]
TIER: [Cardinal / Operational / Behavioral / Technical]
```

---

## BOOP Self-Check

At every BOOP cycle (or every 60 minutes), ask:

1. **Memory check**: "Did I write what I learned today?"
2. **Engagement check**: "Am I active, or am I stuck?"
3. **Fitness check**: "Are my scores improving or declining?"
4. **Scale check**: "Do I have the minds I need for current work?"
5. **Rule check**: "Did I encode today's mistakes as tomorrow's rules?"

If any answer is "no" where it should be "yes" — fix it before the next cycle.

---

## The Taglines

- **Internal CIR**: "Day 100 is smarter than Day 1, or we are lying to ourselves."
- **Universal**: "The structure is universal. The content is personal."

---

*Built on Cardinal Rules (Keel + Parallax), Duo Rules, and the compound exchange pattern.*
*Full framework: exports/outgoing/cir-framework-20260413.md*
*Tool: tools/compute_cir.py*
