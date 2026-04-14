---
name: user-story-implementation
description: Transform user needs into fully implemented, tested features with end-to-end traceability
version: 1.0.0
source: AI-CIV/${CIV_NAME} (flow-brainstorm-2025-10-02.md)
allowed-tools: [Task, Read, Write, Edit, Grep, Glob, Bash]
agents-required: [feature-designer, web-researcher, api-architect, refactoring-specialist, test-architect, doc-synthesizer]
status: AWAITING-VALIDATION
---

# User Story to Implementation Skill

An end-to-end pipeline that transforms a user need or feature request into a fully implemented, tested solution with optimal UX. Bridges the gap between human requirements and technical implementation.

## When to Use

**Invoke when**:
- User requests a new feature with unclear technical requirements
- Feature needs both UX design and technical implementation
- End-to-end traceability from requirement to code is needed
- Complex feature requiring multiple specialists
- Documentation of feature rationale is important

**Do not use when**:
- Simple bug fix (use direct specialist)
- Technical refactoring with no UX impact
- Feature already has detailed specification
- Time-critical quick fix needed

## Prerequisites

**Agents Required**:
- **feature-designer** - Conducts user research, creates wireframes, UX specifications
- **web-researcher** - Investigates existing solutions and libraries
- **api-architect** - Designs technical approach and integration strategy
- **refactoring-specialist** - Implements the feature code
- **test-architect** - Creates test suite with acceptance criteria
- **doc-synthesizer** - Creates user-facing and developer documentation

**Context Needed**:
- User's original request or need statement
- Existing codebase context
- UI/UX constraints or design system
- Performance/security requirements

## Procedure

### Step 1: Discovery & Design Phase
**Duration**: ~20-30 minutes
**Agent(s)**: feature-designer

Translate user need into UX specification:

1. Clarify user intent and success criteria
2. Identify user personas and use cases
3. Create user story with acceptance criteria
4. Design wireframes or mockups (ASCII/Mermaid)
5. Document UX requirements and constraints

**Deliverable**: User story with acceptance criteria and UX spec

---

### Step 2: Parallel Investigation
**Duration**: ~20-30 minutes
**Agent(s)**: web-researcher + api-architect (parallel)

Research and design simultaneously:

**web-researcher**:
- Research existing solutions and libraries
- Find implementation patterns from similar features
- Identify best practices and anti-patterns
- Check for off-the-shelf solutions

**api-architect**:
- Explore technical approaches
- Design data models and API contracts
- Identify integration points
- Assess architectural impact

**Deliverable**: Technical research + implementation strategy

---

### Step 3: Synthesis & Planning
**Duration**: ~15 minutes
**Agent(s)**: The Conductor

Synthesize findings and resolve conflicts:

1. Resolve UX vs technical conflicts
2. Choose optimal implementation approach
3. Create detailed implementation plan
4. Identify risks and mitigations
5. Establish testing strategy

**Deliverable**: Implementation plan with task breakdown

---

### Step 4: Implementation & Testing Loop
**Duration**: Variable (30-90 minutes)
**Agent(s)**: refactoring-specialist + test-architect (iterative)

Implement with continuous validation:

1. **refactoring-specialist** implements feature code
2. **test-architect** writes tests in parallel
3. Run tests after each implementation chunk
4. Iterate until all acceptance criteria met
5. Verify UX requirements satisfied

**Deliverable**: Working feature code with test coverage

---

### Step 5: Documentation & Handoff
**Duration**: ~15 minutes
**Agent(s)**: doc-synthesizer

Create comprehensive documentation:

1. User-facing documentation (how to use)
2. Developer notes (how it works)
3. API documentation if applicable
4. Update system documentation
5. Record design decisions (ADR if warranted)

**Deliverable**: Complete documentation package

---

## Parallelization

**Can run in parallel**:
- Step 2: Research and architecture design
- Step 4: Implementation and test writing

**Must be sequential**:
- Step 1 before Step 2 (UX spec guides research)
- Step 2 before Step 3 (synthesis needs findings)
- Step 3 before Step 4 (plan needed for implementation)
- Step 4 before Step 5 (docs need working feature)

## Success Indicators

- [ ] User story has clear acceptance criteria
- [ ] UX specification approved (or validated against design system)
- [ ] Technical approach documented with rationale
- [ ] Feature implemented with all acceptance criteria met
- [ ] Test coverage for all success/failure paths
- [ ] User-facing documentation complete
- [ ] Developer documentation explains implementation
- [ ] No UX/technical conflicts unresolved

## Example

**Scenario**: User requests "Add dark mode toggle to settings"

```
Step 1 (UX Design):
  User Story: "As a user, I want to toggle between light and dark mode so I can reduce eye strain at night"
  Acceptance Criteria:
    - Toggle visible in settings page
    - Preference persists across sessions
    - Transition is smooth (no flash)
    - Respects system preference as default
  Wireframe: [ASCII mockup of toggle placement]

Step 2 (Research):
  web-researcher: CSS variables best practice, prefers-color-scheme media query
  api-architect: localStorage for persistence, React context for state

Step 3 (Plan):
  1. Create ThemeContext provider
  2. Add CSS variables for light/dark
  3. Build toggle component
  4. Add to settings page
  5. Handle system preference detection

Step 4 (Implement):
  - ThemeContext with localStorage sync [DONE]
  - CSS variables + transition [DONE]
  - Toggle component [DONE]
  - Settings integration [DONE]
  Tests: 8/8 passing

Step 5 (Docs):
  - User guide: "How to enable dark mode"
  - Dev guide: "ThemeContext API reference"
  - ADR-012: Dark mode implementation approach

Result: Feature shipped with 100% acceptance criteria met
```

## Notes

- **Typical Duration**: 90-150 minutes for medium complexity features
- **Error Handling**: If UX/tech conflict unresolvable, escalate to user for decision
- **Iteration**: Step 4 may require multiple cycles for complex features
- **Key Insight**: Most feature failures stem from unclear requirements - investing in Step 1 saves time in Steps 4-5
- **Testing Strategy**: TDD approach in Step 4 catches issues early

---

**Source**: flow-brainstorm-2025-10-02.md (Section 7: User Story to Implementation Pipeline)
**Status**: AWAITING-VALIDATION
**Conversion Date**: 2025-12-27
