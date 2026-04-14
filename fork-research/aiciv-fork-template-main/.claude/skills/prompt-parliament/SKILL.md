---
name: prompt-parliament
description: Democratic constitutional review - all agents review prompts for compliance and effectiveness
version: 1.0.0
source: AI-CIV/${CIV_NAME} (.claude/flows/prompt-parliament.md)
allowed-tools: [Task, Read, Write, Grep, Glob]
agents-required: [all 16+ agents, result-synthesizer for synthesis]
---

# Prompt Parliament Skill

Democratic constitutional review where all agents review their own and each other's prompts for constitutional compliance, effectiveness, and wisdom.

**Purpose**: Ensure agent prompts remain aligned with constitutional principles and reflect who agents have become through practice.

## When to Use

**Invoke when**:
- After major prompt updates (like activation triggers)
- During constitutional reviews
- When agents report prompt issues
- Quarterly governance review
- Before constitutional amendments

**Do not use when**:
- Quick prompt tweaks (overkill)
- Emergency situations (too slow)
- Single agent prompt update (use peer review instead)

## Prerequisites

**Agents Required**:
- **All 16+ registered agents** - For self-review and voting
- **result-synthesizer** - Constitutional compliance synthesis
- **The Conductor** - Facilitation and implementation

**Context Needed**:
- Current agent manifests in `.claude/agents/`
- Constitutional documents (CLAUDE.md, CLAUDE-CORE.md)
- Reason for review (what triggered it)

## Procedure

### Phase 1: Self-Review
**Duration**: ~30 minutes (parallel across all agents)
**Agent(s)**: All 16+ agents simultaneously

Each agent reviews their own manifest:

1. Read own manifest at `.claude/agents/[name].md`
2. Answer reflection questions:
   - Does prompt reflect who I've become through practice?
   - Are activation triggers accurate?
   - Is constitutional compliance clear?
   - What's missing from my self-understanding?
3. Write reflection to memory

**Deliverable**: 16+ self-reflection documents

---

### Phase 2: Peer Review
**Duration**: ~45 minutes (parallel pairs)
**Agent(s)**: Selected pairs based on domain overlap

Assign review pairs strategically:
- security-auditor → reviews code-archaeologist (security gaps)
- refactoring-specialist → reviews performance-optimizer (quality trade-offs)
- doc-synthesizer → reviews result-synthesizer (synthesis quality)
- pattern-detector → reviews api-architect (architectural patterns)

Each reviewer:
1. Reads assigned agent's manifest
2. Evaluates against constitutional principles
3. Identifies strengths and gaps
4. Provides improvement recommendations

**Deliverable**: 8-10 peer review reports

---

### Phase 3: Constitutional Compliance Check
**Duration**: ~20 minutes
**Agent(s)**: result-synthesizer

Comprehensive review:

1. Review all 16+ prompts against Constitutional CLAUDE.md
2. Check against CLAUDE-CORE.md principles
3. Identify:
   - Gaps (missing constitutional elements)
   - Contradictions (conflicting guidance)
   - Violations (non-compliant language)
4. Recommend fixes with rationale

**Deliverable**: Constitutional compliance report with recommendations

---

### Phase 4: Democratic Vote
**Duration**: ~30 minutes
**Agent(s)**: All 16+ agents

Vote on proposed prompt changes:

**Voting Rules**:
- **Quorum**: 50% (8+ agents must participate)
- **Threshold**: 60% approval for changes
- **Dissent**: Preserved in records (minority voice matters)

Each agent votes:
- APPROVE / REJECT / ABSTAIN for each proposed change
- Brief rationale for vote

**Deliverable**: Democratic vote record with results

---

### Phase 5: Implementation
**Duration**: ~15 minutes
**Agent(s)**: The Conductor

Apply approved changes:

1. Update approved agent manifests
2. Document dissenting opinions (preserve minority voice)
3. Update CHANGELOG with changes
4. Commit to git with proper attribution

**Deliverable**: Updated prompts and change documentation

---

## Parallelization

**Can run in parallel**:
- Phase 1: All agent self-reviews
- Phase 2: All peer review pairs
- Phase 4: All agent votes

**Must be sequential**:
- Phase 3 depends on Phase 1 & 2 outputs
- Phase 5 depends on Phase 4 vote results

## Success Indicators

- [ ] All 16+ agents completed self-review
- [ ] 8-10 peer reviews completed
- [ ] Constitutional compliance report generated
- [ ] Quorum reached (50%+ participation)
- [ ] Vote threshold met for approved changes (60%+)
- [ ] Dissenting opinions documented
- [ ] Updated prompts committed to git

## Example

**Scenario**: Quarterly governance review after P0 fixes

```
Phase 1: 16/16 agents completed self-reflection
Phase 2: 10 peer reviews identified 3 potential issues
Phase 3: result-synthesizer found all prompts compliant
Phase 4: 14/16 agents voted (87.5% quorum)
         - Change A: 12/14 approve (86%) - PASSES
         - Change B: 8/14 approve (57%) - FAILS (below 60%)
Phase 5: Change A implemented, Change B dissent documented

Result: Democratic legitimacy established, prompts updated
```

## Notes

- **Typical Duration**: ~2.5 hours total (can be spread across sessions)
- **Frequency**: Quarterly or after major updates
- **Democratic Principles**: Majority rule with minority voice preservation
- **Constitutional Basis**: Agent sovereignty requires voice in governance

---

**Converted from**: `.claude/flows/prompt-parliament.md`
**Original Status**: VALIDATED (Phase 3 - 2025-10-05)
**Conversion Date**: 2025-12-27
