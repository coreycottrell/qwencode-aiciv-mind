---
name: parallel-research
description: Multiple agents research different aspects of a topic simultaneously for comprehensive coverage
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Grep, Glob, WebFetch, WebSearch]
agents-required: [web-researcher, pattern-detector, result-synthesizer]
---

# Parallel Research Skill

A multi-perspective research flow where 3-5 agents simultaneously investigate different angles of a topic. By dividing perspectives (not just workload), the collective achieves comprehensive coverage with minimal overlap and diverse insights.

## When to Use

**Invoke when**:
- Complex, multi-perspective questions need answering
- Comprehensive topic coverage required quickly
- Limited time for deep research (parallel > sequential)
- Need diverse viewpoints on same topic
- Research question has multiple valid angles
- 4x speedup would be valuable

**Do not use when**:
- Single clear domain (use Specialist Consultation - 12.5x more efficient)
- Need sequential depth on one angle
- Research requires dependent discoveries (each step needs prior)
- Simple factual lookup

## Prerequisites

**Agents Required**:
- **web-researcher** - External sources, industry context
- **pattern-detector** - Structural/architectural patterns
- **code-archaeologist** - Historical context, prior art
- **security-auditor** - Risk and vulnerability angle
- **result-synthesizer** - Consolidates all perspectives
- **2-4 domain specialists** - Based on topic (select relevant)

**Context Needed**:
- Clear research question
- Angle assignments (who researches what aspect)
- Synthesis criteria (what makes a good combined answer)

## Procedure

### Step 1: Angle Assignment
**Duration**: ~5 minutes
**Agent(s)**: The Conductor

Divide the research into non-overlapping angles:

1. Decompose question into 3-5 distinct perspectives
2. Assign each perspective to appropriate specialist
3. Ensure perspectives are complementary, not redundant
4. Brief each agent on their specific angle

**Example Angles**:
- Technical implementation
- Industry precedent
- Security implications
- User experience impact
- Historical context

**Deliverable**: Angle assignment matrix

---

### Step 2: Parallel Investigation
**Duration**: ~30-60 minutes (all run simultaneously)
**Agent(s)**: 3-5 assigned researchers

All agents research in parallel:

1. Each agent focuses ONLY on their assigned angle
2. Gather evidence, examples, insights
3. Note confidence levels
4. Flag cross-cutting discoveries for synthesis
5. Produce angle-specific report

**Deliverable**: 3-5 specialist research reports

---

### Step 3: Synthesis
**Duration**: ~15-20 minutes
**Agent(s)**: result-synthesizer

Consolidate perspectives:

1. Read all angle-specific reports
2. Identify cross-perspective themes
3. Note contradictions or tensions
4. Create unified answer that honors all angles
5. Produce confidence-weighted synthesis

**Deliverable**: Synthesized research findings

---

### Step 4: Quality Check
**Duration**: ~5 minutes
**Agent(s)**: The Conductor

Verify coverage and quality:

1. Calculate overlap percentage (target: <15%)
2. Assess coverage completeness
3. Rate overall quality
4. Document any gaps

**Deliverable**: Quality assessment

---

## Parallelization

**Can run in parallel**:
- Step 2 (the heart of this flow) - All research happens simultaneously
- This is where the 4x speedup comes from

**Must be sequential**:
- Step 1 before 2 (assignments needed)
- Step 2 before 3 (reports needed for synthesis)
- Step 3 before 4 (synthesis needed for quality check)

## Success Indicators

- [ ] 3-5 specialist research reports completed
- [ ] <15% overlap between reports (truly different perspectives)
- [ ] Synthesis addresses all angles coherently
- [ ] Quality rating 8.5+ across all reports
- [ ] Comprehensive coverage achieved (no major gaps)
- [ ] 3-4x faster than sequential research would have been

## Example

**Scenario**: Research "AI agent coordination patterns" across perspectives

```
Step 1 (Assign):
  - web-researcher: Industry examples and academic papers
  - pattern-detector: Structural patterns and architectures
  - code-archaeologist: Historical approaches and evolution
  - security-auditor: Security implications of multi-agent systems

Step 2 (Research) - 90 seconds total, all parallel:
  web-researcher: Found 7 industry implementations, 3 papers
  pattern-detector: Identified 4 core patterns (hub, mesh, hierarchy, swarm)
  code-archaeologist: Traced 15-year evolution from simple scripts
  security-auditor: Flagged 3 attack vectors, 2 mitigations

Step 3 (Synthesize):
  Unified finding: "Effective coordination requires..."
  Cross-theme: All reports emphasized trust/verification
  Tension: Hierarchy (efficient) vs. Mesh (resilient)

Step 4 (Quality):
  Overlap: 8% (excellent - truly different perspectives)
  Coverage: Comprehensive
  Quality: 9.2/10 average across agents
  Speed: 4x faster than sequential

Result: Comprehensive answer from 4 perspectives in 90 seconds
        Would have taken 6+ minutes sequentially
```

## Notes

- **Typical Duration**: 45-90 minutes for complex topics (mostly Step 2)
- **Error Handling**: If one angle fails, others still provide value
- **Evolution**: Can scale to 6-8 agents for very complex topics
- **Key Insight**: Divide PERSPECTIVES, not just workload
- **The "<10% overlap" metric**: Validates truly different angles
- **When NOT to use**: Single-domain questions (Specialist Consultation is 12.5x more efficient)
- **4x Speedup**: Parallel execution beats sequential even with synthesis overhead

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 7: Parallel Research)
**Original Status**: VALIDATED (2025-10-02)
**Conversion Date**: 2025-12-27
