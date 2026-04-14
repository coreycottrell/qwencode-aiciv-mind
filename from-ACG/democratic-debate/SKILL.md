---
name: democratic-debate
description: All agents vote on strategic decisions with quorum 50% and threshold 60% for democratic legitimacy
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Write]
agents-required: [all available agents, result-synthesizer, conflict-resolver]
---

# Democratic Debate Skill

A collective deliberation flow where all agents (14+) contribute perspectives and vote on strategic decisions. Ensures democratic legitimacy through quorum requirements (50%) and approval thresholds (60%). Preserves dissent for future reference.

## When to Use

**Invoke when**:
- Strategic decisions affecting entire collective
- Constitutional matters or governance changes
- Collective priority setting (what to work on)
- Major directional choices (not tactical)
- Need legitimate buy-in from all agents
- Decision will be referenced long-term

**Do not use when**:
- Single-domain question (use Specialist Consultation - 12.5x faster)
- Tactical/operational decision (too slow)
- Time pressure exists (this flow takes time)
- Clear expert answer exists (don't need democracy for facts)

## Prerequisites

**Agents Required**:
- **All available agents** - Every agent gets a voice
- **result-synthesizer** - Consolidates perspectives
- **conflict-resolver** - Handles disagreements

**Quorum & Threshold**:
- **Quorum**: 50% of agents must participate
- **Threshold**: 60% approval to pass

**Context Needed**:
- Clear decision question (not ambiguous)
- Options to vote on (can emerge from debate)
- Framing that invites genuine perspective

## Procedure

### Step 1: Decision Framing
**Duration**: ~5 minutes
**Agent(s)**: The Conductor

Frame the decision clearly:

1. State the decision question precisely
2. Explain why democratic input is needed
3. List any known options (or invite proposals)
4. Set participation expectations

**Deliverable**: Decision frame document

---

### Step 2: Perspective Gathering (Parallel)
**Duration**: ~20-30 minutes
**Agent(s)**: All agents

Every agent contributes their perspective:

1. Each agent receives the decision frame
2. Each provides their view from their domain expertise
3. Agents may propose options not yet listed
4. Perspectives include reasoning, not just positions

**Deliverable**: 14+ perspective statements

---

### Step 3: Synthesis & Option Crystallization
**Duration**: ~15 minutes
**Agent(s)**: result-synthesizer

Consolidate perspectives into votable options:

1. Read all perspective statements
2. Identify common themes and tensions
3. Crystallize 2-4 clear options for voting
4. Ensure options represent the full spectrum

**Deliverable**: Synthesized options for vote

---

### Step 4: Voting
**Duration**: ~10 minutes
**Agent(s)**: All agents

Democratic vote:

1. Present crystallized options
2. Each agent votes (can rank or approve)
3. Record all votes with reasoning
4. Calculate results

**Deliverable**: Vote tally with reasoning

---

### Step 5: Result Declaration & Dissent Preservation
**Duration**: ~5 minutes
**Agent(s)**: The Conductor + conflict-resolver

Finalize and preserve dissent:

1. Check quorum (50% participated?)
2. Check threshold (60% approved winner?)
3. Declare result with legitimacy status
4. Document dissenting views for record
5. Explain how dissent was considered

**Deliverable**: Final decision with dissent record

---

## Parallelization

**Can run in parallel**:
- Step 2: All agents can provide perspectives simultaneously
- Step 4: All votes can be cast simultaneously

**Must be sequential**:
- Step 1 before 2 (frame needed)
- Step 2 before 3 (perspectives needed for synthesis)
- Step 3 before 4 (options needed for voting)
- Step 4 before 5 (votes needed for declaration)

## Success Indicators

- [ ] Quorum achieved (50%+ agents participated)
- [ ] All agents had opportunity to contribute perspective
- [ ] Options represent full spectrum of views
- [ ] Threshold met for decision (60%+ approval)
- [ ] Dissent preserved (not suppressed)
- [ ] Democratic legitimacy established for the decision

## Example

**Scenario**: Collective decides on Adaptive Response Protocol for message handling

```
Step 1 (Frame): "How should AI-CIV handle incoming messages?
                Options: A) Respond immediately, B) Batch daily,
                C) Adaptive based on urgency"

Step 2 (Perspectives) - All 14 agents respond:
  security-auditor: "Immediate responses risk hasty security decisions"
  human-liaison: "Humans expect timely acknowledgment"
  pattern-detector: "Adaptive approach matches our parallel architecture"
  performance-optimizer: "Batching is more efficient for resources"
  [... 10 more perspectives ...]

Step 3 (Synthesize):
  Final options:
  A) Immediate response to all
  B) Daily batch processing
  C) Adaptive: urgent=immediate, routine=batched

Step 4 (Vote):
  Option A: 3 votes (21%)
  Option B: 2 votes (14%)
  Option C: 9 votes (64%)

Step 5 (Declare):
  Quorum: 14/14 = 100% (passed)
  Winner: Option C with 64% (passed 60% threshold)
  Decision: ADAPTIVE RESPONSE PROTOCOL ADOPTED

  Dissent preserved: "5 agents preferred deterministic options;
                      concerns about 'urgency' classification noted"

Result: Democratically legitimate decision with full participation
        Dissent documented for future reference
        Quality: 8.9/10
```

## Efficiency Note

| Flow | Efficiency | Best For |
|------|------------|----------|
| Specialist Consultation | 15.6 words/agent/sec | Single domain |
| Parallel Research | 6.2 words/agent/sec | Multi-perspective |
| **Democratic Debate** | 1.25 words/agent/sec | Strategic decisions |

**Democratic Debate is 12.5x slower than Specialist Consultation** - use only when legitimacy matters more than speed.

However, it scales well: **14 agents only 2.7x slower than single agent** (not 14x) due to parallelization.

## Notes

- **Typical Duration**: 60-90 minutes for full debate
- **Error Handling**: If quorum fails, extend invitation or reduce scope
- **Evolution**: Consider weighted voting for domain expertise
- **Key Insight**: Democratic legitimacy is worth the time for strategic decisions
- **The "60% threshold"**: High enough for legitimacy, low enough for decisions
- **Dissent Preservation**: Essential - minority views may become majority later
- **Don't Over-Use**: Most questions don't need democracy; 80% are specialist questions

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 9: Democratic Debate)
**Original Status**: VALIDATED (2025-10-02)
**Conversion Date**: 2025-12-27
