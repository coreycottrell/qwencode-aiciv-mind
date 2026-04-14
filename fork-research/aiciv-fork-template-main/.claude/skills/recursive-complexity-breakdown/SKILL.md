---
name: recursive-complexity-breakdown
description: Transform overwhelmingly complex tasks into structured execution plans with clear dependencies
version: 1.0.0
source: AI-CIV/${CIV_NAME} (flow-brainstorm-2025-10-02.md)
allowed-tools: [Task, Read, Write, Grep, Glob]
agents-required: [task-decomposer, web-researcher, api-architect, conflict-resolver, integration-auditor, test-architect]
status: AWAITING-VALIDATION
---

# Recursive Complexity Breakdown Skill

Transforms overwhelmingly complex, ambiguous tasks into structured execution plans with clear dependencies and resource allocation. Turns user's vague goal into executable roadmap.

## When to Use

**Invoke when**:
- Facing vague requests like "migrate our authentication system"
- Task feels too big to start
- Multiple valid approaches exist with unclear trade-offs
- Need to identify hidden dependencies and blockers
- Creating project plan for multi-day/week work
- Estimating effort for complex feature

**Do not use when**:
- Task is already well-defined with clear steps
- Simple single-agent task (overkill)
- Urgent fix needed (too slow)
- Already have detailed specification

## Prerequisites

**Agents Required**:
- **task-decomposer** (Lead) - Creates task hierarchy, complexity scores, dependency map
- **web-researcher** - Investigates domain context and prior art
- **api-architect** - Analyzes system constraints and integration requirements
- **conflict-resolver** - Identifies failure modes and risk areas
- **integration-auditor** - Validates decomposition against existing infrastructure
- **test-architect** - Defines acceptance criteria and testing strategy

**Context Needed**:
- User's original (possibly vague) request
- Existing codebase or system context
- Known constraints (timeline, resources, technology)

## Procedure

### Step 1: Parallel Discovery Phase
**Duration**: ~15-20 minutes
**Agent(s)**: web-researcher + api-architect + conflict-resolver (parallel)

Investigate from multiple angles simultaneously:

**web-researcher**:
- Research domain context and terminology
- Find similar implementations or case studies
- Identify industry best practices
- Discover potential libraries or tools

**api-architect**:
- Analyze existing system constraints
- Map current architecture boundaries
- Identify integration points
- Assess technical feasibility

**conflict-resolver**:
- Identify failure modes and edge cases
- Anticipate stakeholder conflicts
- Find hidden dependencies
- Map risk areas

**Deliverable**: Three perspective reports on the problem space

---

### Step 2: Task Hierarchy Creation
**Duration**: ~20 minutes
**Agent(s)**: task-decomposer

Synthesize findings into multi-level task structure:

1. Create epic-level summary (the "what")
2. Break into features (major deliverables)
3. Decompose features into tasks (workable units)
4. Further split into subtasks if needed
5. Assign complexity scores (Fibonacci: 1, 2, 3, 5, 8, 13)

**Structure**:
```
Epic (the goal)
├── Feature 1 (major deliverable)
│   ├── Task 1.1 (workable unit)
│   │   ├── Subtask 1.1.1
│   │   └── Subtask 1.1.2
│   └── Task 1.2
├── Feature 2
│   └── ...
```

**Deliverable**: Multi-level task hierarchy with complexity scores

---

### Step 3: Dependency Mapping
**Duration**: ~15 minutes
**Agent(s)**: task-decomposer + integration-auditor

Identify blocking relationships:

1. Map task dependencies (what blocks what)
2. Identify parallelizable work streams
3. Find critical path items
4. Flag external dependencies (APIs, approvals, etc.)
5. Validate against existing infrastructure

**Deliverable**: Dependency graph with critical path highlighted

---

### Step 4: Validation Layer
**Duration**: ~15 minutes
**Agent(s)**: test-architect

Define success criteria for each branch:

1. Create acceptance criteria for features
2. Define testing strategy per task
3. Identify verification checkpoints
4. Establish quality gates
5. Document rollback points

**Deliverable**: Acceptance criteria matrix

---

### Step 5: Parallel Sanity Check
**Duration**: ~10 minutes
**Agent(s)**: All agents review

Final validation round:

1. Each agent reviews decomposition from their perspective
2. Propose adjustments where domain expertise reveals gaps
3. Identify any missed dependencies or risks
4. Validate complexity estimates
5. Consensus on execution order

**Deliverable**: Finalized, validated execution plan

---

## Parallelization

**Can run in parallel**:
- Step 1: All three discovery agents
- Step 5: All review agents

**Must be sequential**:
- Step 1 before Step 2 (discovery informs decomposition)
- Step 2 before Step 3 (tasks needed for dependencies)
- Step 3 before Step 4 (structure needed for criteria)
- Step 4 before Step 5 (complete plan needed for review)

## Success Indicators

- [ ] Ambiguous request converted to structured plan
- [ ] All tasks have complexity scores
- [ ] Dependencies clearly mapped with critical path
- [ ] Each feature has acceptance criteria
- [ ] Hidden blockers identified before execution
- [ ] Parallelizable streams identified for efficiency
- [ ] Risk areas documented with mitigations
- [ ] Team consensus on execution order

## Example

**Scenario**: "Migrate our authentication system to a new provider"

```
Step 1 (Discovery):
  web-researcher: OAuth2 vs OIDC comparison, provider migration guides
  api-architect: Current auth flow analysis, 12 integration points found
  conflict-resolver: Data migration risk, session invalidation concern

Step 2 (Decomposition):
  Epic: Authentication Provider Migration
  ├── Feature 1: Provider Setup [5]
  │   ├── Task 1.1: Configure new provider tenant [2]
  │   ├── Task 1.2: Set up OAuth2 client [3]
  ├── Feature 2: Code Migration [13]
  │   ├── Task 2.1: Update auth middleware [5]
  │   ├── Task 2.2: Migrate token handling [5]
  │   └── Task 2.3: Update 12 integration points [8]
  ├── Feature 3: Data Migration [8]
  │   ├── Task 3.1: Export user data [3]
  │   └── Task 3.2: Import with mapping [5]
  └── Feature 4: Verification & Rollback [5]
      └── ...

Step 3 (Dependencies):
  Critical Path: 1.1 → 1.2 → 2.1 → 2.2 → 3.1 → 3.2 → 4.x
  Parallel: Feature 1 and Feature 3 can run simultaneously
  External: Provider approval needed before 1.2

Step 4 (Criteria):
  Feature 1: Provider responds to test auth request
  Feature 2: All existing tests pass with new provider
  Feature 3: 100% user accounts migrated, verified
  Feature 4: Rollback tested, <5 min recovery time

Step 5 (Review):
  api-architect: Add task for deprecation period
  test-architect: Need integration test suite before migration
  conflict-resolver: Add communication plan for users

Result: 31-point migration plan with clear execution order
```

## Notes

- **Typical Duration**: 60-90 minutes for complex task breakdown
- **Complexity Scoring**: Use Fibonacci (1,2,3,5,8,13) for relative sizing
- **Error Handling**: If decomposition feels wrong, return to Step 1 discovery
- **Key Insight**: Most project failures start with unclear problem understanding - Step 1 discovery prevents this
- **Critical Path**: Always highlight the longest dependency chain - that's your minimum timeline

---

**Source**: flow-brainstorm-2025-10-02.md (Section 12: Recursive Complexity Breakdown)
**Status**: AWAITING-VALIDATION
**Conversion Date**: 2025-12-27
