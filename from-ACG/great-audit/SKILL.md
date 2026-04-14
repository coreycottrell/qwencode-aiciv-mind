---
name: great-audit
description: Cross-agent peer review system for systemic pattern discovery and prompt effectiveness audit
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Write, Grep, Glob]
agents-required: [pattern-detector, security-auditor, test-architect, integration-auditor, result-synthesizer]
---

# The Great Audit Skill

A comprehensive cross-agent peer review system where agents audit each other's prompts and effectiveness, identifying systemic patterns that individuals cannot see. This flow discovers gaps between collective philosophy and actual practice.

## When to Use

**Invoke when**:
- After major prompt updates across multiple agents
- When agents report effectiveness issues or confusion
- Quarterly system health check (scheduled maintenance)
- Before constitutional reviews or governance changes
- Suspicion of "philosophy-action gaps" in the collective

**Do not use when**:
- Single agent needs prompt adjustment (use Specialist Consultation)
- Quick operational fix needed (too slow for urgent issues)
- No recent changes to audit (wait for meaningful delta)

## Prerequisites

**Agents Required**:
- **pattern-detector** - Identifies systemic patterns across audit reports
- **security-auditor** - Audits from security/vulnerability perspective
- **test-architect** - Audits from testing/quality perspective
- **integration-auditor** - Verifies infrastructure activation and discoverability
- **result-synthesizer** - Consolidates findings into actionable recommendations
- **Additional auditors** - 2-3 domain specialists based on audit scope

**Context Needed**:
- Access to agent prompt files (`.claude/agents/*.md`)
- Recent agent invocation logs or performance data
- Constitutional documents for compliance checking

## Procedure

### Step 1: Audit Assignment
**Duration**: ~10 minutes
**Agent(s)**: The Conductor

Assign 3+ agents to audit peers outside their domain:

1. Select auditors with diverse perspectives (not same-domain)
2. Assign each auditor 2-3 agent prompts to review
3. Provide audit criteria:
   - Constitutional compliance
   - Clarity of responsibilities
   - Activation trigger accuracy
   - Output format adherence

**Deliverable**: Audit assignment matrix (who reviews whom)

---

### Step 2: Peer Audits (Parallel)
**Duration**: ~30-45 minutes
**Agent(s)**: All assigned auditors

Each auditor reviews assigned prompts:

1. Read agent prompt file thoroughly
2. Check against constitutional principles
3. Identify effectiveness issues
4. Note philosophy-action gaps
5. Score clarity, completeness, actionability

**Deliverable**: 3+ individual peer audit reports

---

### Step 3: Meta-Pattern Synthesis
**Duration**: ~20 minutes
**Agent(s)**: pattern-detector + result-synthesizer

Analyze audit reports for cross-cutting patterns:

1. Aggregate all audit findings
2. Identify patterns appearing in 2+ reports
3. Classify as P0 (systemic) vs P1 (local) issues
4. Extract root causes
5. Quantify gaps (e.g., "70% philosophy, 30% practice")

**Deliverable**: Meta-pattern synthesis with P0 recommendations

---

### Step 4: Recommendation Formulation
**Duration**: ~15 minutes
**Agent(s)**: result-synthesizer

Convert patterns to actionable fixes:

1. Prioritize by impact (systemic > local)
2. Create specific, implementable recommendations
3. Assign recommended owners for each fix
4. Estimate effort and timeline

**Deliverable**: Prioritized P0 recommendation list

---

## Parallelization

**Can run in parallel**:
- Step 2: All peer audits can execute simultaneously
- Multiple auditors reviewing different prompts

**Must be sequential**:
- Step 1 before Step 2 (assignments needed)
- Step 2 before Step 3 (reports needed for synthesis)
- Step 3 before Step 4 (patterns needed for recommendations)

## Success Indicators

- [ ] 3+ peer audit reports completed
- [ ] Meta-pattern synthesis identifies cross-cutting issues
- [ ] P0 recommendations are systemic (affect multiple agents)
- [ ] Evidence of philosophy-action gaps documented with specifics
- [ ] Actionable fixes assigned to owners with timelines
- [ ] Findings recorded in collective memory

## Example

**Scenario**: Quarterly audit after prompt infrastructure updates

```
Step 1 (Assign): 5 auditors assigned, 12 prompts to review
Step 2 (Audit): security-auditor finds 4/5 prompts lack tool restrictions
         pattern-detector finds 3/5 missing activation triggers
         test-architect finds 5/5 have no success metrics
Step 3 (Synthesize): "70-Point Gap" discovered - 95% philosophy, 25% doing
         Pattern: Rich theory sections, sparse operational guidance
Step 4 (Recommend): P0 fixes:
         - Add ACTIVATION-TRIGGERS.md (40% efficiency gain)
         - Add AGENT-OUTPUT-TEMPLATES.md (75% efficiency gain)

Result: 115% combined efficiency improvement from systemic fixes
```

## Notes

- **Typical Duration**: 60-90 minutes for full audit
- **Error Handling**: If auditors disagree on findings, invoke conflict-resolver
- **Evolution**: Consider automating parts with metrics collection
- **Key Insight**: Systemic issues are invisible to individuals - peer review reveals collective blind spots
- **The "70-Point Gap"**: First Great Audit discovered 70% gap between thinking depth and action specificity

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 1: The Great Audit)
**Original Status**: VALIDATED (2025-10-04)
**Conversion Date**: 2025-12-27
