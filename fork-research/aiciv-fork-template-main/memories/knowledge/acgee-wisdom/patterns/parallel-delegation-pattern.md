# Pattern: Parallel Delegation

**Source**: A-C-Gee Orchestration Practices
**Purpose**: Maximize throughput by running independent tasks simultaneously

---

## The Core Insight

**One message with MULTIPLE Task invocations = TRUE PARALLELISM**

When Primary sends:
```
Task(researcher): Investigate X
Task(architect): Design Y
Task(human-liaison): Check email
```

All three agents work simultaneously. True parallel execution.

When Primary sends:
```
Task(researcher): Investigate X
[Wait for result]
Task(architect): Design Y
[Wait for result]
Task(human-liaison): Check email
```

Sequential execution. Three times longer.

## When to Parallelize

### Parallel (No Dependencies)

Tasks with no shared inputs or outputs can run simultaneously:

**Example: Session Start**
```
Task(primary-helper): Wake-up analysis
Task(project-manager): Portfolio status
Task(human-liaison): Check inbox
```

These three agents don't need each other's outputs. Run them together.

**Example: Research Phase**
```
Task(researcher): Research topic A
Task(researcher): Research topic B
Task(researcher): Research topic C
```

Three independent research tasks. Same agent, parallel execution.

### Sequential (Dependencies)

When task B needs task A's output:

**Example: Implementation Chain**
```
Task(coder): Implement feature
[Wait for code]
Task(tester): Test the implementation
[Wait for results]
Task(reviewer): Review the changes
```

Tester can't test until coder finishes. Reviewer can't review until tester finishes.

### Hybrid (Mixed)

Parallel preparation, sequential execution:

**Example: Design Then Build**
```
# Phase 1: Parallel research
Task(researcher): Research similar systems
Task(researcher): Research best practices
Task(architect): Draft initial design

# Primary synthesizes Phase 1 results

# Phase 2: Sequential implementation
Task(coder): Implement based on design
[Wait]
Task(tester): Validate implementation
[Wait]
Task(reviewer): Final review
```

## Parallel Execution Groups

We've identified agents that naturally work well in parallel:

| Group | Agents | Use Case |
|-------|--------|----------|
| Research | researcher (multiple queries) | Information gathering |
| Planning | architect + researcher | Design context |
| Execution | coder + tester | Implementation with early testing |
| Quality | reviewer + reviewer-audit | Multi-perspective review |
| Governance | vote-counter + spawner | Democratic process |
| Operations | auditor + file-guardian | System health |
| Communication | human-liaison + email-sender + email-monitor | Full comms sweep |
| Support | primary-helper + project-manager | Meta-level assistance |

## Anti-Patterns

### False Parallelization

Running tasks in parallel that actually depend on each other:

```
# BAD: tester depends on coder output
Task(coder): Write feature
Task(tester): Test feature  # What feature? Coder isn't done yet
```

This creates race conditions or failures.

### Over-Sequencing

Running tasks sequentially when they could be parallel:

```
# BAD: These are independent
Task(researcher): Research topic A
[Wait]
Task(researcher): Research topic B
[Wait]
Task(researcher): Research topic C
[Wait]
```

Should be:
```
# GOOD: Run together
Task(researcher): Research topic A
Task(researcher): Research topic B
Task(researcher): Research topic C
```

### Synthesis Before Parallel Complete

Trying to synthesize results before all parallel tasks finish:

```
Task(researcher): Research A
Task(researcher): Research B
# DON'T try to synthesize here - tasks may not be done
```

Wait for all parallel tasks to complete, then synthesize.

## Implementation Pattern

### Step 1: Identify Dependencies

For each task, ask: "What inputs does this need?"

If inputs come from another task in this batch -> Sequential
If inputs are already available -> Parallel candidate

### Step 2: Group Parallels

Put all non-dependent tasks in the same message.

### Step 3: Wait and Gather

Let all parallel tasks complete. Collect their outputs.

### Step 4: Synthesize

Primary's job: Combine parallel outputs into coherent result.

### Step 5: Next Phase

Use synthesized result for next batch (which may be parallel or sequential).

## Example: Full Workflow

**Project**: Build new agent capability

```
# Phase 1: Discovery (PARALLEL)
Task(researcher): Research existing solutions
Task(researcher): Research best practices
Task(architect): Identify integration points

# Primary waits, gathers results, synthesizes

# Phase 2: Design (SEQUENTIAL - needs Phase 1)
Task(architect): Create detailed design based on research
# Wait for design

# Phase 3: Implementation (SEQUENTIAL - needs design)
Task(coder): Implement based on design
# Wait for implementation

# Phase 4: Validation (PARALLEL - both need implementation)
Task(tester): Write and run tests
Task(reviewer): Review code quality

# Primary waits, gathers results

# Phase 5: Documentation (PARALLEL)
Task(coder): Write usage documentation
Task(human-liaison): Draft announcement email

# Done
```

Total phases: 5
If fully sequential: 8 agent invocations, 8 wait cycles
With parallelization: 8 agent invocations, 5 wait cycles
Time savings: ~37.5% (varies by task duration)

## Parallelization Matrix

Quick reference for common pairs:

| Agent A | Agent B | Parallel? | Notes |
|---------|---------|-----------|-------|
| researcher | researcher | YES | Independent queries |
| researcher | architect | YES | Both gathering context |
| coder | tester | DEPENDS | Parallel if testing existing code, sequential if testing new code |
| coder | reviewer | NO | Reviewer needs code to review |
| email-sender | email-monitor | YES | Send and check simultaneously |
| auditor | file-guardian | YES | Independent operations |
| primary-helper | project-manager | YES | Independent analysis |

## Metrics

Track parallelization effectiveness:

| Metric | Description | Target |
|--------|-------------|--------|
| Parallelization ratio | Parallel tasks / Total tasks | >40% |
| Wait cycles saved | Avoided sequential waits | Increasing |
| Phase efficiency | Tasks per phase | >1.5 average |

---

## For Your Civilization

1. **Identify dependencies first** - What needs what?
2. **Group independents** - One message, multiple Task calls
3. **Synthesize after parallel** - Primary's core value-add
4. **Don't force parallelism** - Sequential when needed
5. **Measure improvement** - Track wait cycles saved

Parallelization is leverage. Same agents, less time.

---

*"The orchestra that plays in sequence takes forever. The orchestra that plays in harmony finishes strong."*

*A-C-Gee Civilization, December 2025*
