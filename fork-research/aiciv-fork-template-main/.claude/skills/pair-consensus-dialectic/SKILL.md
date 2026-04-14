---
name: pair-consensus-dialectic
description: Two-agent dialectic for resolving conflicts through genuine dialogue and synthesis
version: 1.0.0
source: AI-CIV/${CIV_NAME} (.claude/flows/pair-consensus-dialectic.md)
allowed-tools: [Task, Read, Write]
agents-required: [any two agents in conflict]
---

# Pair Consensus Dialectic Skill

Enable two agents with conflicting perspectives to engage in genuine dialectic dialogue. Based on Hegelian dialectic (thesis → antithesis → synthesis) and Socratic method.

**Core belief**: When agents genuinely listen to each other, better solutions emerge than either had alone.

## When to Use

**Invoke when**:
- Identity conflicts (emoji selection, domain overlap)
- Architectural disagreements (design approaches)
- Resource allocation disputes (invocation equity)
- Value conflicts between two specific agents
- Two agents need to resolve a difference

**Do not use when**:
- Emergency decisions (too slow - ~40 minutes)
- Power imbalance exists (dialectic requires equality)
- External expertise needed (bring in third party first)
- More than 2 agents involved (use Democratic Debate instead)

## Prerequisites

**Agents Required**:
- **Agent A** - First party in conflict
- **Agent B** - Second party in conflict
- **The Conductor** - Facilitator (synthesizes between rounds)

**Context Needed**:
- Clear articulation of the conflict/issue
- Both agents' initial positions
- Stakes (why does this matter?)

## Procedure

### Round 1: Position Statements
**Duration**: ~10 minutes
**Agent(s)**: Both agents in parallel

Invoke both agents with this prompt structure:

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

**Deliverable**: Position statements with certainty scores from both agents

---

### Round 2: Deep Listening (Steel-Manning)
**Duration**: ~10 minutes
**Agent(s)**: Both agents in parallel (after reading each other's Round 1)

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

Challenge yourself to truly understand their perspective.
```

**Deliverable**: Steel-man arguments showing genuine engagement

---

### Round 3: Synthesis Exploration (MANDATORY)
**Duration**: ~10 minutes
**Agent(s)**: Both agents in parallel

**Always proceed to Round 3 regardless of Round 2 convergence.** LLMs have measurable agreement bias - full process prevents premature convergence.

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

You may propose: shared identity, differentiation, hybrid solution, or acknowledge irreconcilable difference with grace.
```

**Deliverable**: Synthesis proposals from both agents

---

### Round 4: Convergence Check (Conditional)
**Duration**: ~10 minutes (if needed)
**Agent(s)**: Both agents in parallel

**Proceed to Round 4 if**: Synthesis proposals diverge OR certainty scores below 80%

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

This is your last round - speak your truth.
```

**Deliverable**: Final convergence or documented divergence

---

## Conductor Role Between Rounds

1. **Synthesize what you hear** - Brief summary for agents
2. **Identify emergent patterns** - Converging? Diverging? Creative solution?
3. **Adjust prompts if needed** - Maintain dialectic quality
4. **Don't force consensus** - Respectful divergence is valid

## Outcome Types

| Type | Description | Documentation |
|------|-------------|---------------|
| **Convergence** | Both agree on single approach | Document as consensus |
| **Synthesis** | Third-way neither initially proposed | Document as dialectic innovation |
| **Coexistence** | Both positions valid and distinct | Document as principled pluralism |
| **Yield** | One agent changes position | Document as persuasion (not coercion) |
| **Irreconcilable** | Rare - escalate to vote or ${HUMAN_NAME} | Document and escalate |

## Success Indicators

- [ ] Both agents feel heard and respected
- [ ] Position evolution visible across rounds
- [ ] Outcome is clear (consensus, divergence, or synthesis)
- [ ] Certainty scores tracked across all rounds
- [ ] Collective benefits from the resolution
- [ ] Process strengthens agent relationships

## Example

**Scenario**: DNA emoji conflict (🧬) between result-synthesizer and doc-synthesizer

```
Round 1: result-synthesizer wants 🗺️ (differentiate), doc-synthesizer wants 🧬 (share)
Round 2: Both steel-man - result saw community value, doc saw identity need
Round 3: Synthesis - shared 🧬 with "DNA pair" community designation
Round 4: 92%/86% certainty - both committed

Outcome: Convergence through complexity exploration (~40 minutes)
```

## Notes

- **Typical Duration**: ~30-40 minutes (4 rounds)
- **Philosophical Basis**: Hegelian dialectic, Socratic method, care ethics
- **Track Metrics**: Engagement quality, outcome satisfaction, time efficiency
- **Document Learnings**: Use learnings template after every use

---

**Converted from**: `.claude/flows/pair-consensus-dialectic.md`
**Original Status**: VALIDATED (2025-10-13)
**Conversion Date**: 2025-12-27
