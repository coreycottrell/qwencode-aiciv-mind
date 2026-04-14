---
name: specialist-consultation
description: Route question to single specialist agent for expert-depth answer - fastest flow (80% of cases)
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Grep, Glob]
agents-required: [varies by domain]
---

# Specialist Consultation Skill

The simplest and most efficient coordination flow: identify the right specialist, route the question, get expert answer. This handles 80% of coordination tasks and is 12.5x more efficient than Democratic Debate for single-domain questions.

## When to Use

**Invoke when**:
- Domain-specific question with clear owner
- Need expert depth rather than breadth
- Single clear specialist exists for the domain
- 80% of all coordination tasks fit this pattern
- Speed matters (fastest flow available)
- Question doesn't require multiple perspectives

**Do not use when**:
- Question spans multiple domains (use Parallel Research)
- Strategic decision needing collective input (use Democratic Debate)
- No clear specialist exists (may need new agent or parallel approach)
- Need diverse viewpoints (single expert = single perspective)

## Prerequisites

**Agents Required**:
- **One domain specialist** - The expert for this question type

**Common Specialist Mappings**:
| Question Type | Specialist |
|--------------|------------|
| Security | security-auditor |
| Performance | performance-optimizer |
| API design | api-architect |
| Testing | test-architect |
| Code quality | refactoring-specialist |
| Patterns | pattern-detector |
| Legacy code | code-archaeologist |
| Documentation | doc-synthesizer |
| UX/Features | feature-designer |
| Naming | naming-consultant |

**Context Needed**:
- Clear question
- Domain classification
- Any relevant context files

## Procedure

### Step 1: Domain Classification
**Duration**: ~1 minute
**Agent(s)**: The Conductor

Identify the question's domain:

1. Parse the question for domain signals
2. Match to specialist activation triggers
3. Verify single specialist is appropriate
4. If uncertain, check `.claude/templates/ACTIVATION-TRIGGERS.md`

**Deliverable**: Identified specialist

---

### Step 2: Context Preparation
**Duration**: ~2 minutes
**Agent(s)**: The Conductor

Gather relevant context:

1. Identify files the specialist might need
2. Extract relevant snippets
3. Frame the question clearly
4. Note any constraints or preferences

**Deliverable**: Question with context

---

### Step 3: Specialist Invocation
**Duration**: ~30-60 seconds
**Agent(s)**: The identified specialist

Route to specialist and get answer:

1. Invoke specialist with question + context
2. Specialist applies domain expertise
3. Specialist returns expert answer
4. Include confidence level

**Deliverable**: Expert answer from specialist

---

### Step 4: Validation (Optional)
**Duration**: ~1 minute
**Agent(s)**: The Conductor

Quick sanity check:

1. Does answer address the question?
2. Is confidence appropriate?
3. Any follow-up needed?

**Deliverable**: Validated answer (or follow-up question)

---

## Parallelization

**Can run in parallel**:
- Multiple independent Specialist Consultations can run simultaneously
- If you have 3 unrelated questions, invoke 3 specialists in parallel

**Must be sequential**:
- Steps 1-3 for a single question (classification -> context -> invoke)

## Success Indicators

- [ ] Correct specialist identified
- [ ] Expert answer addresses the question directly
- [ ] Answer demonstrates domain depth (not generic)
- [ ] Confidence level included
- [ ] Faster than alternative flows would have been

## Example

**Scenario**: "Is our authentication implementation secure?"

```
Step 1 (Classify): Security domain -> security-auditor

Step 2 (Context):
  - Authentication code files identified
  - Current auth flow documented
  - Question framed: "Audit auth implementation for vulnerabilities"

Step 3 (Invoke): security-auditor responds in 45 seconds:
  "Authentication implementation review:
   - Password hashing: Using bcrypt with cost 12
   - Session management: Secure, uses httpOnly cookies
   - CSRF: Protected via double-submit pattern
   - JWT: Expiry too long (24h), recommend 1h + refresh

   Confidence: 85% (would need pen test for higher)

   P0 Fix: Reduce JWT expiry to 1 hour"

Step 4 (Validate): Answer addresses question, actionable fix provided

Result: Expert security audit in 45 seconds
        Quality: 9.0/10
        Efficiency: 15.6 words/agent/second
```

## Efficiency Comparison

| Flow | Time | Efficiency | When to Use |
|------|------|------------|-------------|
| **Specialist Consultation** | 45 sec | 15.6 words/agent/sec | Single domain (80% of cases) |
| Parallel Research | 90 sec | 6.2 words/agent/sec | Multi-perspective needed |
| Democratic Debate | 3+ min | 1.25 words/agent/sec | Strategic decisions |

**12.5x more efficient than Democratic Debate** for single-domain questions.

## Notes

- **Typical Duration**: 30-90 seconds
- **Error Handling**: If specialist can't answer, escalate to Parallel Research
- **Evolution**: Improve domain classification with pattern learning
- **Key Insight**: 80% of questions have a single clear expert
- **The "12.5x efficiency" metric**: Validated comparison vs. Democratic Debate
- **Don't Over-Coordinate**: Simple questions deserve simple routing
- **Give Specialists Experience**: Even "easy" questions help them grow

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 8: Specialist Consultation)
**Original Status**: VALIDATED (2025-10-02)
**Conversion Date**: 2025-12-27
