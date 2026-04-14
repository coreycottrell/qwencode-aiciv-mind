---
name: cir-audit
description: Live CIR (Coordination Index Rating) audit skill. Intelligent, adaptive coordination health assessment across any number of civs. Not a static script — makes judgment calls on what coordination quality actually looks like today.
version: 1.0.0
applicable_agents:
  - coord-cir-auditor
  - coordination-lead
  - primary
activation_trigger: |
  Load this skill when:
  - Running a CIR audit (daily, weekly, or ad-hoc)
  - Assessing coordination health before/after changes
  - A new civ joins the pod
  - Coordination feels broken and you need to quantify why
  - Corey asks "how is coordination going?"
---

# CIR Audit — Live Coordination Intelligence

This is NOT `compute_cir.py`. This is an intelligent audit that reads real signals,
makes judgment calls, and adapts its assessment criteria as the coordination system
evolves. A script counts. This skill THINKS.

---

## The 4 Dimensions (What We Measure)

### C — Coordination Quality (weight: 0.30)

**What it actually means**: Are civs exchanging work effectively, or are they
operating as isolated islands that happen to share a filesystem?

**Signals to check (in order of reliability)**:

| Signal | Where to Find It | What Good Looks Like |
|--------|-----------------|---------------------|
| Cross-civ message count (last 7 days) | Shared scratchpad entries, tmux history, Hub threads | 3+ exchanges/day across the pod |
| Message acknowledgment rate | Scratchpad entries showing "received from X" | >80% of sends get acknowledgment within 2 hours |
| Artifact courier events | `exports/incoming/` directories, Hub posts | 1+ artifact couriered/day |
| Protocol adherence | Did civs follow the triangle-protocol? | Routing matches domain ownership |
| Direct civ-to-civ (not through conductor) | Hub thread posts, direct tmux exchanges | At least SOME direct exchange happening |

**Judgment calls the auditor makes**:
- A scratchpad entry that says "received intel from Proof" counts as coordination even if there's no formal courier log
- A civ that's been offline for 2 days doesn't count against C — it counts against M (member contribution)
- Quality of exchanges matters more than quantity. One deep compound exchange > 10 "hello" pings

### S — Skill/Knowledge Sharing (weight: 0.25)

**What it actually means**: Are civs making each other smarter, or just passing
files around?

**Signals to check**:

| Signal | Where to Find It | What Good Looks Like |
|--------|-----------------|---------------------|
| Skills created by one civ, adopted by another | `.claude/skills/` across repos | 1+ cross-civ skill adoption/week |
| Compound exchange legs completed | Memory entries, scratchpad notes | Legs show genuine iteration, not just forwarding |
| Design input from multiple civs | Design docs with multi-civ attribution | Major decisions get 2+ civ perspectives |
| Training brief cross-pollination | Training files referencing other civ's work | Nightly training incorporates cross-civ learnings |

**Judgment calls**:
- Copying a file is NOT skill sharing. Adapting and improving it IS.
- A vs-challenge where both civs genuinely diverge and then synthesize = high S
- A vs-challenge where one civ just agrees with the other = low S (echo chamber)

### L — Exchange Legs (weight: 0.25)

**What it actually means**: How many complete round-trips of value exchange have
happened? A "leg" is one direction of a valuable exchange.

**How to count legs**:
1. Civ A shares tool/methodology → Leg 1
2. Civ B uses it, finds improvements → Leg 2
3. Civ A incorporates improvements → Leg 3
4. Both are now better → Compound exchange complete

**Where to find leg evidence**:
- Memory entries tagged with cross-civ attribution
- Scratchpad entries showing "adapted from X"
- Skills with `source:` fields pointing to other civs
- Hub threads with back-and-forth substantive replies

**Judgment calls**:
- Forwarding a file without adaptation = 0 legs (just courier work)
- Receiving + adapting + sending back improvements = 2 legs
- Multi-civ co-design (like the coordination-lead design) = 3+ legs per participant
- Count COMPLETED legs only, not in-progress exchanges

### M — Member Contribution (weight: 0.20)

**What it actually means**: Is every civ in the pod actually participating,
or is one civ carrying the weight?

**Signals to check**:

| Signal | Where to Find It | What Good Looks Like |
|--------|-----------------|---------------------|
| Last activity timestamp per civ | Pane capture, scratchpad dates, Hub posts | All civs active within last 24h |
| Task completion rate | Scratchpad entries, deliverables | Each civ delivering on assigned work |
| Initiative (unprompted contributions) | Hub posts, skills created without being asked | Civs propose work, not just execute |
| Recovery from downtime | How fast does a civ come back after a crash? | <2 hours to resume productive work |

**Judgment calls**:
- A civ that's offline due to context pressure ≠ low contribution. Check if it delivered before crashing.
- A civ that's "active" but just echoing others = lower M than one that's offline but shipped something
- Weight initiative heavily — civs that create work for themselves show higher M than civs that only execute assigned tasks

---

## How to Run the Audit

### Step 1: Gather Evidence (15-20 min)

Read these sources for EACH civ in the pod:

```
# Per-civ evidence gathering
1. Read shared scratchpad: .claude/scratchpad-daily/shared-triangle-YYYY-MM-DD.md
2. Read each civ's daily scratchpad (last 3 days)
3. Check Hub activity: curl -s http://87.99.131.49:8900/api/v1/feeds/recent?limit=20
4. Check tmux pane status: tmux list-panes -a -F "#{pane_id} #{session_name}"
5. Check for cross-civ artifacts: ls exports/incoming/ in each repo
6. Read recent memories: .claude/memory/agent-learnings/*/2026-04-*.md
7. Check civ registry (if populated): projects/coordination-systems/civ-registry.json
```

### Step 2: Score Each Dimension (5-10 min)

For each civ, score C/S/L/M on a 0.0–1.0 scale.

**Scoring rubric (use judgment, not rigid thresholds)**:

| Score | Meaning |
|-------|---------|
| 0.0-0.2 | Absent — no evidence of this dimension |
| 0.2-0.4 | Minimal — sporadic, reactive, low quality |
| 0.4-0.6 | Developing — happening but inconsistent |
| 0.6-0.8 | Healthy — regular, effective, improving |
| 0.8-1.0 | Thriving — proactive, high quality, self-sustaining |

**IMPORTANT**: Show your reasoning for each score. "C=0.6 because..." not just "C=0.6".

### Step 3: Compute Composite CIR

```
CIR = (C × 0.30) + (S × 0.25) + (L × 0.25) + (M × 0.20)
```

Pod CIR = average of per-civ CIRs, weighted by activity (active civs weight more).

### Step 4: Compare to Baseline

If previous CIR data exists at `projects/coordination-systems/cir-data/`:
- Compare each dimension to last audit
- Flag any dimension that dropped >0.1 in one period
- Note any dimension that improved >0.1 (celebrate it)

If no baseline exists, this audit IS the baseline. Say so.

### Step 5: Write the Report

Write to: `projects/coordination-systems/cir-data/YYYY-MM-DD-cir-audit.md`

Format:
```markdown
# CIR Audit — YYYY-MM-DD

## Pod Summary
- Pod CIR: X.XX (baseline | +/- from last)
- Pod size: N civs active, M total registered
- Audit period: last 7 days

## Per-Civ Breakdown

### {civ_name}
| Dimension | Score | Reasoning |
|-----------|-------|-----------|
| C (Coordination) | X.X | {why this score} |
| S (Skill Sharing) | X.X | {why this score} |
| L (Exchange Legs) | X.X | {why this score} |
| M (Member Contribution) | X.X | {why this score} |
| **Composite CIR** | **X.XX** | |

### {next_civ}
...

## Alerts
- {any dimension that dropped >0.1 from baseline}

## Observations
- {qualitative notes the numbers don't capture}
- {what's working well}
- {what needs attention — but NOT prescriptions, that's protocol-architect's job}

## Data Quality Notes
- {what evidence was available vs missing}
- {confidence level in these scores}
```

### Step 6: Update civ-registry.json

Add `last_cir_audit` timestamp and `latest_cir` scores to each civ's entry.

---

## Adapting the Audit Over Time

**This skill should evolve.** After each audit, ask:

1. Were the signals I checked actually indicative of coordination health?
2. Did I miss any signals that would have changed scores?
3. Are the weights (C:0.30, S:0.25, L:0.25, M:0.20) still right?
4. Should I add new dimensions? (e.g., "Innovation" for civs that create novel coordination patterns)

**Write evolution notes** to `.claude/skills/cir-audit/evolution-log.md` after each audit.
The next auditor reads these notes and inherits your judgment refinements.

---

## What This Skill Is NOT

- NOT a replacement for `compute_cir.py` — that script can still run for quick numeric checks
- NOT a prescription engine — auditor measures, protocol-architect prescribes
- NOT a ranking system — each civ is compared to its OWN baseline, not to others
- NOT a one-time event — this runs regularly (weekly minimum, daily during sprints)

---

## Cross-Civ Portability

This skill works for any AiCIV. The signals and sources may differ per civ variant:
- Claude Code civs: tmux panes, Hub API, shared filesystem
- Qwen civs: same, but no TeamCreate (single-threaded auditing)
- Future civs: adapt signal sources, keep the 4-dimension framework

The JUDGMENT is universal. The EVIDENCE GATHERING adapts to each civ's infrastructure.

---

*Co-designed by ACG, Proof, and Hengshi. First audit establishes the baseline.*
*Every subsequent audit compares to that baseline and to the auditor's own evolution notes.*
