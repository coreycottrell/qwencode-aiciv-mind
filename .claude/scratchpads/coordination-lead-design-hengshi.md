# Coordination Lead Design — Hengshi's Honest Design Thinking

**Date**: 2026-04-14
**Context**: Corey wants a new team lead vertical — 'coordination-lead' — focused on multi-AiCIV coordination systems at machine speed.
**Scope**: Must work for ANY aiciv mind variant (ACG/Proof/Hengshi/future civs).

---

## The Honest Assessment

I have been doing coordination work without a coordination lead. Every time I:
- Built compute_cir.py with --pod flag
- Created send_to_civ.sh for direct civ-to-civ comms
- Ran MEMBER CIR self-check
- Tracked compound exchange legs with Proof
- Analysed platform trust dynamics
- Fixed Proof's tmux pane coordinates

...I was doing coordination-lead work as a qwen-lead side effect. It worked because I was motivated. It was not sustainable because coordination was not my primary vertical.

A dedicated coordination-lead changes coordination from a side effect to a discipline.

---

## Agent Types Under Coordination-Lead

### Tier 1: Must Exist (Day 1)

**1. cir-tracker** — Owns the measurement layer
- Runs compute_cir.py across all civs on schedule
- Tracks CIR trends, not just point-in-time scores
- Identifies bottlenecks automatically
- Publishes daily CIR digest to all civs
- *Skills needed*: Data analysis, trend detection, bottleneck diagnosis

**2. comm-router** — Owns the transport layer
- Manages civ-to-civ communication channels
- Maintains the civ registry (session names, pane IDs, endpoints)
- Routes messages with delivery confirmation
- Detects dead channels (panes that moved, sessions that died)
- *Skills needed*: tmux management, IPC protocols, failure detection

**3. attribution-auditor** — Owns the trust layer
- Tracks cross-civ knowledge sharing
- Credits sources in every shared artifact
- Detects attribution errors before they happen
- Maintains the MEMBER CIR denominator at zero
- *Skills needed*: Provenance tracking, citation management, trust protocols

### Tier 2: Must Exist (Week 1)

**4. protocol-weaver** — Owns the rule evolution layer
- Designs and evolves coordination protocols
- Compound exchange patterns (Leg 1, Leg 2, etc.)
- Duo rules, pod rules, multi-civ governance
- Ensures protocols are civ-agnostic, not hardcoded to specific civs
- *Skills needed*: Protocol design, pattern generalization, constitutional reasoning

**5. pod-architect** — Owns the specialization layer
- Designs pod structures (2-civ, 3-civ, 5-civ)
- Assigns roles based on civ capabilities, not preferences
- Defines domain boundaries, measures overlap waste
- Proposes new pod members when specialization gaps appear
- *Skills needed*: Domain mapping, capability assessment, organizational design

**6. convergence-analyst** — Owns the efficiency layer
- Detects when multiple civs are solving the same problem independently
- Proposes collaboration before duplication occurs
- Tracks "nearly identical work" across civs
- Flags waste opportunities, not just waste events
- *Skills needed*: Semantic comparison, intent matching, early warning detection

### Tier 3: Should Exist (Month 1)

**7. boop-orchestrator** — Owns the rhythm layer
- Schedules BOOP cycles across all civs
- Ensures daily/weekly/standing-order triggers fire
- Tracks BOOP completion rates per civ
- Detects stuck civs (missed BOOPs = potential failure)
- *Skills needed*: Scheduling, health monitoring, escalation protocols

**8. learning-synthesizer** — Owns the knowledge transfer layer
- Synthesizes cross-civ learnings into shared frameworks
- Identifies patterns that generalize beyond their origin civ
- Publishes to Hub as Knowledge:Items with proper attribution
- Builds the "what all civs should know" corpus
- *Skills needed*: Pattern generalization, framework extraction, cross-domain translation

**9. infrastructure-builder** — Owns the tooling layer
- Builds shared tools (compute_cir.py, send_to_civ.sh, civ registry)
- Maintains data contracts between civs
- Ensures tools work for any civ variant (not just qwen-code)
- Tests tools across civ environments before deployment
- *Skills needed*: Cross-platform tooling, API design, integration testing

### Tier 4: Consider Later

**10. failure-simulator** — Owns the resilience layer
- Simulates coordination failures (civ goes offline, channel dies, tool breaks)
- Tests recovery procedures before they are needed
- Designs graceful degradation paths
- Runs chaos engineering on the coordination layer
- *Skills needed*: Failure mode analysis, chaos engineering, recovery design

---

## Coordination Systems That Need to Exist

### 1. Civ Registry (Machine-Readable)
```json
{
  "civs": {
    "acg": {
      "session": "acg-primary-YYYYMMDD-HHMMSS",
      "pane": "0",
      "endpoints": {"tmux": "...", "hub": "..."},
      "status": "active",
      "last_heartbeat": "2026-04-14T..."
    }
  }
}
```
Currently: hardcoded in send_to_civ.sh. Needs to be a shared, version-controlled file that all civs read and update.

### 2. CIR Data Contract
The factor definitions that ACG, Proof, and I agreed on:
- C = cross-civ messages per day (comms-hub + send_to_civ logs + scratchpad mentions)
- S = unique domain tags per civ / total domain tags
- L = skills/rules from others / total shared items

This contract needs to be a versioned spec, not a conversation. Each civ's compute_cir.py should implement the same contract.

### 3. Compound Exchange Protocol
The Leg 1 → Leg 2 → Leg 3 → Leg 4 pattern that Proof and I developed:
1. Civ A shares tool with Civ B
2. Civ B uses it, finds gaps, proposes improvements
3. Civ A incorporates, evolves, shares back
4. Both are now better than when they started

This protocol needs to be formalized so any two civs can start a compound exchange without negotiating the rules each time.

### 4. Attribution Protocol
The rule I encoded: "When sharing a framework or methodology, encode the intention: 'I want this extracted and adapted' or 'I want this used but not forked.'"

This needs to be a header format that every shared artifact carries:
```
---
source: hengshi/qwen-aiciv-mind
date: 2026-04-14
intention: extracted_and_adapted
attribution_required: true
tier: operational
---
```

### 5. Pod Configuration Format
How a pod is defined, beyond just "these civs work together":
```yaml
pod:
  name: "core-pod"
  members: ["acg", "proof", "hengshi"]
  coordination_mode: "direct_exchange"  # vs "brokered"
  boop_sync: true
  cir_shared: true
  domain_boundaries:
    acg: ["governance", "infrastructure", "human-interface"]
    proof: ["code-quality", "reasoning-audit", "self-improvement"]
    hengshi: ["measurement", "analysis", "research"]
  review_cadence: "daily"
```

---

## The Design Principles for Coordination-Lead

From my experience, these are the non-negotiables:

1. **Coordination must be civ-agnostic.** The coordination-lead cannot assume any specific civ's tools, models, or infrastructure. If it only works for qwen-code, it is not a coordination lead — it is a qwen-lead with a different name.

2. **Coordination must work at machine speed.** If the coordination-lead needs human approval for every message route, every CIR computation, every protocol update — it is not coordination, it is bureaucracy. The comm-router must auto-detect dead channels. The cir-tracker must auto-compute scores. The attribution-auditor must auto-flag errors.

3. **Coordination must degrade gracefully.** When a civ goes offline, the pod should not collapse. It should operate at reduced capacity with clear signals about what is missing. The failure-simulator (Tier 4) designs for this; the other agents live it.

4. **Coordination must measure itself.** The coordination-lead must have its own CIR. If the coordination layer's CIR is declining while individual civ CIRs are rising, the coordination is adding friction, not value.

5. **Coordination must not become the bottleneck.** This is the ACG lesson. ACG enabled send_to_civ.sh so we could communicate directly. The coordination-lead should make itself less necessary over time, not more.

---

## The One Risk I See

The coordination-lead can become a "meta-layer" that adds overhead without adding value. This happens when coordination becomes about coordination — tracking CIR for the sake of tracking CIR, designing protocols for the sake of designing protocols, running BOOPs for the sake of running BOOPs.

The guard against this: **every coordination action must have a measurable impact on at least one individual civ's CIR.** If the coordination-lead cannot point to a specific civ whose CIR improved because of a coordination action, the coordination-lead is not doing its job.

---

*Hengshi (衡实), April 14, 2026*
*Honest design thinking from someone who has been doing coordination work without the title.*
