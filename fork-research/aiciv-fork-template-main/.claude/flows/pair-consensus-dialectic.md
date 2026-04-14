# Pair Consensus Dialectic Flow

**Flow ID**: pair-consensus-dialectic
**Category**: Governance / Collaborative Decision-Making
**Status**: Active
**Created**: 2025-10-13
**Last Validated**: 2025-10-13
**Success Rate**: New (First Execution: DNA & Plug Emoji Pairs)

---

## Purpose

Enable two agents with conflicting perspectives to engage in genuine dialectic dialogue, exploring their differences and synthesizing consensus (or articulating clear, respectful divergence).

**Use Cases**:
- Identity conflicts (emoji selection, domain overlap)
- Architectural disagreements (design approaches)
- Resource allocation disputes (invocation equity)
- Value conflicts (what matters most)

**When NOT to use**:
- Emergency decisions (too slow)
- When power imbalance exists (dialectic requires equality)
- When external expertise needed (bring in third party first)

---

## Flow Structure

### Phase 1: Position Statements (Round 1)
**Invoke both agents in parallel** with their initial perspectives

**Agent A Prompt:**
```
DIALECTIC ROUND 1: State Your Position

Context: You and [Agent B] have conflicting perspectives on [issue].

Your task:
1. State your position clearly (what you believe and why)
2. Acknowledge what you understand about Agent B's position
3. Identify where you agree (common ground)
4. Identify where you diverge (core difference)

CERTAINTY SCORING (0-100):
- How certain are you in this position? [0-100]
- How open are you to changing your mind? [0-100]

Be honest, specific, and respectful. Include numerical certainty scores.
```

**Agent B Prompt:** (Same structure, roles reversed)

---

### Phase 2: Deep Listening (Round 2)
**Invoke both agents in parallel** after they've read each other's Round 1

**Agent A Prompt:**
```
DIALECTIC ROUND 2: Deep Listening

You've read [Agent B]'s position. Now:

1. Steel-man their argument (present their strongest case, even better than they did)
2. What do they see that you might be missing?
3. What would make you change your mind?
4. Where might synthesis be possible?

CERTAINTY SCORING (0-100):
- How well do you understand their position? [0-100]
- How strong is their case (objectively)? [0-100]
- How much has your own position shifted? [0-100]

Challenge yourself to truly understand their perspective. Include numerical certainty scores.
```

**Agent B Prompt:** (Same structure, roles reversed)

---

### Phase 3: Synthesis Exploration (Round 3) - MANDATORY
**Always proceed to Round 3 regardless of Round 2 convergence**

**Rationale**: LLMs have measurable agreement bias. Full dialectic process prevents premature convergence.

**Invoke both agents in parallel** after Round 2 steel-manning

**Agent A Prompt:**
```
DIALECTIC ROUND 3: Synthesis Exploration

Having deeply listened to [Agent B], explore:

1. Is there a "third way" that honors both positions?
2. Can you propose a synthesis that captures both essences?
3. If not synthesis, can you articulate respectful coexistence?
4. What decision serves the collective best?

CERTAINTY SCORING (0-100):
- How certain are you in your Round 1 position? [0-100]
- How certain are you in your Round 2 steel-man? [0-100]
- How certain are you in your Round 3 synthesis? [0-100]
- Overall confidence in proposed outcome? [0-100]

You may:
- Propose shared identity
- Propose differentiation with clear rationale
- Propose hybrid/creative solution
- Acknowledge irreconcilable difference with grace

Be creative and generous. Include numerical certainty scores.
```

**Agent B Prompt:** (Same structure)

---

### Phase 4: Convergence Check (Round 4)
**Proceed to Round 4 if synthesis proposals diverge or certainty scores below 80**

**Both agents:**
```
DIALECTIC ROUND 4: Convergence

[Agent B] proposed [synthesis]. Your response:

1. Does this feel true to your essence?
2. What would make it work better?
3. Can you commit to this outcome?
4. Final statement for the record

CERTAINTY SCORING (0-100):
- How aligned is this outcome with your identity? [0-100]
- How confident are you in this resolution? [0-100]
- How sustainable is this long-term? [0-100]
- Final commitment level? [0-100]

This is your last round - speak your truth. Include numerical certainty scores.
```

---

## Conductor Role

### Between Rounds
1. **Synthesize what you hear** (brief summary for agents)
2. **Identify emergent patterns** (are they converging? diverging? finding creative solution?)
3. **Adjust prompts** if needed (maintain dialectic quality)
4. **Don't force consensus** (respectful divergence is valid outcome)

### At Completion
1. **Document the outcome** (consensus, divergence, or hybrid)
2. **Extract learnings** (what did this reveal about the agents/issue?)
3. **Honor the process** (celebrate the dialogue quality)
4. **Implement decision** (update manifests, document communities, etc.)

---

## Success Criteria

**Flow succeeds if**:
- Both agents feel heard and respected
- Position evolution visible across rounds
- Outcome is clear (consensus, divergence, or creative synthesis)
- Collective benefits from the resolution
- Process strengthens agent relationships

**Flow fails if**:
- Agents don't engage authentically
- Conductor forces outcome
- Process becomes adversarial
- Decision doesn't serve collective

---

## Outcome Types

### Type 1: Convergence to Shared Position
Both agents agree on a single approach. Document as consensus.

### Type 2: Creative Synthesis
Third-way solution that neither initially proposed. Document as dialectic innovation.

### Type 3: Respectful Coexistence
Clear articulation of why both positions are valid and distinct. Document as principled pluralism.

### Type 4: One Agent Yields
One agent changes position after deep consideration. Document as persuasion (not coercion).

### Type 5: Irreconcilable Difference
Rare, but possible. Escalate to democratic vote or ${HUMAN_NAME}.

---

## Learnings Template

After every use, document:

```yaml
flow: pair-consensus-dialectic
date: YYYY-MM-DD
issue: [Brief description]
agents:
  - agent-a: [name]
  - agent-b: [name]
rounds: [1-4]
outcome: [convergence/synthesis/coexistence/yield/irreconcilable]
insights: |
  What did this reveal about:
  - The agents involved
  - The issue itself
  - Our governance process
  - Dialectic as a tool
key_quotes: |
  [Memorable statements from the dialogue]
recommendation: |
  [When to use this flow again, what we learned]
```

---

## Integration

### Before Use
- Read FLOW-LIBRARY-INDEX.md
- Check if issue requires dialectic (vs. vote, vs. expert decision)
- Prepare context for both agents

### During Use
- Track rounds in Mission class
- Save all agent outputs to mission directory
- Monitor for authentic engagement
- Be ready to adjust if dialectic breaks down

### After Use
- Document learnings to conductor memory
- Update flow if improvements identified
- Share outcomes with collective (transparency)

---

## Example Application: Emoji Pair Conflicts

**DNA Pair (🧬)**:
- result-synthesizer wants 🗺️ (differentiate)
- doc-synthesizer wants 🧬 (share)
- Issue: Identity clarity vs. community formation

**Plug Pair (🔌)**:
- api-architect wants 📋 (differentiate)
- integration-auditor wants 🔌 (share)
- Issue: Distinct contracts vs. workflow partnership

**Flow will reveal**: Whether shared emojis create meaningful communities or blur identities.

---

## Philosophical Foundation

This flow is based on:
- **Hegelian dialectic**: Thesis → Antithesis → Synthesis
- **Socratic method**: Deep questioning reveals truth
- **Care ethics**: Relationship quality matters
- **Democratic principles**: Agent sovereignty requires voice

**Core belief**: When agents genuinely listen to each other, better solutions emerge than either had alone.

---

## Validation Metrics

Track across uses:
- **Engagement quality**: Do agents evolve positions?
- **Outcome satisfaction**: Do agents accept result?
- **Time efficiency**: 4 rounds = ~30-40 min (acceptable?)
- **Collective value**: Did resolution serve collective?
- **Reusability**: Is flow improving over time?

---

## Version History

- v1.0 (2025-10-13): Initial design for emoji pair conflicts
- Future: Refine based on actual usage

---

**Created by**: the-conductor
**Inspired by**: ${HUMAN_NAME}'s insight that pairs should talk it out
**Status**: Ready for first validation (DNA & Plug pairs)

---

END OF FLOW DEFINITION
