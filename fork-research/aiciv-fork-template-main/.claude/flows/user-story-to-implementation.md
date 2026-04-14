# User Story to Implementation Pipeline

**Status**: VALIDATED (2025-12-27)
**Lead Agent**: feature-designer
**Pattern Type**: Sequential with Parallel Investigation Phases
**Validated By**: test-architect
**Test Scenario**: Cross-Collective Signing Tutorial

---

## Purpose

Transform a user need or feature request into a fully implemented, tested,
and documented solution with optimal UX.

---

## Agents Involved (AI-CIV Roster)

| Phase | Agent | Focus |
|-------|-------|-------|
| Lead (P1) | feature-designer | User research, UX design, acceptance criteria |
| Parallel (P2) | web-researcher | Existing solutions, industry patterns |
| Parallel (P2) | api-architect | Technical approach, API design |
| Synthesis (P3) | the-conductor | Conflict resolution, planning |
| Implementation (P4) | refactoring-specialist | Code implementation |
| Testing (P4) | test-architect | Test suite development |
| Documentation (P5) | doc-synthesizer | User docs, developer notes |

---

## Execution Phases

### Phase 1: Discovery & Design (Sequential)
**Agent**: feature-designer (lead)
**Duration**: 30-60 minutes

1. Review user need or feature request
2. Identify target user persona
3. Create user story with acceptance criteria
4. Design user journey/wireframes
5. Define UX specifications

**Output**: Feature Design Specification

**Handoff Criteria**: Acceptance criteria approved by human (if available)

---

### Phase 2: Parallel Investigation
**Agents**: web-researcher + api-architect
**Duration**: 30-60 minutes (parallel)

#### web-researcher Focus
- Research existing solutions and libraries
- Find industry best practices
- Identify accessibility patterns

#### api-architect Focus
- Analyze technical constraints
- Design implementation approach
- Identify potential risks

**Handoff Criteria**: Both agents complete findings documents

---

### Phase 3: Synthesis & Planning (Sequential)
**Agents**: the-conductor (lead) + feature-designer
**Duration**: 20-30 minutes

1. Merge UX design with technical findings
2. Resolve conflicts (UX vs technical constraints)
3. Create detailed implementation plan
4. Assign agent responsibilities

**Output**: Implementation Plan with file deliverables list

**Escalation**: If UX and technical requirements conflict unresolvably,
escalate to human for prioritization decision.

---

### Phase 4: Implementation & Testing Loop (Parallel)
**Agents**: refactoring-specialist + test-architect
**Duration**: 2-6 hours (parallel, iterative)

**Loop Structure**:
1. refactoring-specialist implements code
2. test-architect writes tests (parallel)
3. Run tests, identify failures
4. Iterate until all tests pass

**Handoff Criteria**: All acceptance criteria have passing tests

---

### Phase 5: Documentation & Handoff (Sequential)
**Agent**: doc-synthesizer
**Duration**: 1-3 hours

1. Create user-facing documentation
2. Write developer notes
3. Update system documentation
4. Create handoff summary

**Output**: Complete documentation package

---

## Time Budget

| Phase | Duration | Total Elapsed |
|-------|----------|---------------|
| Phase 1 | 30-60 min | 1 hour |
| Phase 2 | 30-60 min (parallel) | 1.5 hours |
| Phase 3 | 20-30 min | 2 hours |
| Phase 4 | 2-6 hours | 8 hours |
| Phase 5 | 1-3 hours | 11 hours |

**Total**: 8-12 hours for moderate feature

---

## When to Use

- New user-facing features
- Features requiring UX design decisions
- Complex features needing research phase
- Features requiring documentation

## When NOT to Use

- Internal tooling (skip UX phases, use Specialist Consultation)
- Bug fixes (use Fortress Protocol for security, direct implementation otherwise)
- Minor enhancements (use Specialist Consultation)

---

## Success Criteria

1. User story with acceptance criteria defined
2. Research phase complete with findings documented
3. No unresolved UX/technical conflicts
4. All acceptance criteria have passing tests
5. Documentation complete and reviewed
6. Handoff to production path clear

---

## Example Use Cases

**Validated**: Cross-Collective Signing Tutorial (2025-12-27)
- User story: "As a collective operator, I want a signing tutorial..."
- Acceptance criteria: Complete in 15 min, copy-paste code, error handling
- Deliverables: Python example, tutorial doc, test suite

**Future Candidates**:
- Dark mode implementation (original example from brainstorm)
- API v2.0 migration guide
- Dashboard multi-collective view

---

## Lessons Learned (from Validation)

1. **Agent mapping required** - Original "Code Monkey" mapped to refactoring-specialist
2. **Conductor implicit in synthesis** - Made explicit in Phase 3
3. **Parallel phases highly efficient** - 30-50% time savings
4. **Test-architect parallel to implementation** - Prevents "testing afterthought" anti-pattern
5. **Documentation as Phase 5** - Ensures shipping complete features

---

## Related Flows

- **Specialist Consultation**: For simple features without UX complexity
- **Parallel Research**: Phase 2 uses this pattern internally
- **Fortress Protocol**: Add before Phase 4 for security-critical features

---

## Caveats (from Validation)

1. **Agent mapping required** - Use the table above (original spec uses non-AI-CIV names)
2. **Escalation path needed** - Define how to escalate UX/technical conflicts to human
3. **Not for internal tooling** - Skip UX phases for developer-only tools
4. **Combine with Fortress Protocol** - For security-critical features

---

## Validation Report

Full validation documentation: `user-story-to-implementation-VALIDATION-REPORT.md`
