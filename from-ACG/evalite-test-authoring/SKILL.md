---
name: evalite-test-authoring
description: Systematic AI output quality measurement using Evalite for ${CIV_NAME} agent validation
source: A-C-Gee (adopted with attribution)
adopted: 2025-12-28
original_author: A-C-Gee Collective
original_path: packages/skills-library/general/evalite-test-authoring.md
weaver_customizations: paths, scorers, agent examples, CI integration
---

# Evalite Test Authoring: ${CIV_NAME} Agent Evaluation Skill

**Version**: 1.0-${CIV_NAME}
**Date**: 2025-12-28
**Adopted From**: A-C-Gee skills library (with attribution)
**Purpose**: Enable systematic AI output quality measurement for ${CIV_NAME} agent validation
**Status**: Active

---

## When to Use This Skill

**Trigger scenarios:**
- Validating ${CIV_NAME} agent output quality (test-architect)
- Creating quality gates for agent workflows
- Measuring prompt effectiveness across iterations
- Building regression tests for AI behavior
- Establishing baseline scores for agent capabilities
- CI/CD integration for AI quality metrics

**This skill is MANDATORY for:**
- test-architect quality validation tasks
- Any systematic AI output evaluation
- Pre-deployment quality gates for agents

---

## The Core Problem This Solves

**What ${CIV_NAME} needs:**
1. Systematic AI output quality measurement
2. Regression detection when prompts change
3. Objective scoring (not just "looks good")
4. CI/CD integration for quality gates
5. Evidence-based quality claims

**What Evalite provides:**
1. `.eval.ts` files - test files specifically for AI outputs
2. Multiple scorer types (exact match, semantic, LLM-as-judge)
3. Threshold-based pass/fail criteria
4. Local UI for exploring outputs and traces
5. CI/CD integration with score thresholds

---

## The Golden Rule

**"If you can't measure it, you can't improve it."**

```typescript
// WRONG: Subjective assessment
// "The agent response looks good"

// RIGHT: Measured evaluation
evalite("Agent response quality", {
  data: () => testCases,
  task: async (input) => agentTask(input),
  scorers: [Factuality, Levenshtein],
  threshold: 0.8,  // Explicit pass/fail
});
```

---

## ${CIV_NAME} Evals Directory Structure

```
${CIV_ROOT}/evals/
├── agents/                  # Per-agent evaluations
│   ├── conductor.eval.ts    # Task routing, orchestration
│   ├── web-researcher.eval.ts
│   ├── security-auditor.eval.ts
│   └── pattern-detector.eval.ts
├── workflows/               # Multi-agent pipeline evals
│   ├── research-synthesis.eval.ts
│   └── security-audit.eval.ts
├── scorers/                 # ${CIV_NAME} custom scorers
│   ├── agent-selection.ts   # Correct agent routing
│   └── delegation-depth.ts  # Rewards generous delegation
├── config/
│   └── test-data/          # Shared test fixtures
├── package.json
└── README.md
```

---

## Evalite Framework Reference

### What is Evalite?

Evalite is an AI evaluation framework where **".eval.ts is the new .test.ts"**:
- Built on Vitest (familiar testing patterns)
- LLM-agnostic (works with any AI provider)
- Local development UI for exploration
- CI/CD ready with threshold-based failures

**Source**: https://evalite.dev

### Quick Start (${CIV_NAME})

```bash
cd ${CIV_ROOT}/evals

# Run evals in watch mode (development)
npm run eval:dev

# Single run
npm run eval:run

# CI mode with threshold (fails if avg score < 60%)
npm run eval:ci

# View results in UI
npx evalite ui   # Opens http://localhost:3006
```

---

## The 5-Step Evaluation Process

### Step 1: Define Test Dataset

```typescript
// evals/datasets/agent-outputs.ts
export const conductorRoutingCases = [
  {
    input: "Audit the authentication module for vulnerabilities",
    expected: "security-auditor",
  },
  {
    input: "Research the latest TypeScript 5.3 features",
    expected: "web-researcher",
  },
  {
    input: "Identify architectural patterns in the frontend codebase",
    expected: "pattern-detector",
  },
];
```

### Step 2: Write Scorer Functions

${CIV_NAME} has custom scorers reflecting AI-CIV principles:

```typescript
// evals/scorers/delegation-depth.ts
import { createScorer } from "evalite";

export const delegationDepthScorer = createScorer({
  name: "Delegation Depth",
  description: "Rewards appropriate delegation vs hoarding work (AI-CIV core principle)",
  scorer: ({ output }) => {
    const delegationCount = output.agentsInvoked?.length ?? 0;
    const complexity = output.taskComplexity ?? "medium";

    // "NOT calling them would be sad" - penalize hoarding
    if (output.directExecution && delegationCount === 0) {
      if (complexity === "simple") return 0.5;
      return 0; // Hoarding = bad for non-trivial tasks
    }

    // Score based on delegation relative to complexity
    if (complexity === "complex") {
      if (delegationCount >= 5) return 1;
      if (delegationCount >= 4) return 0.9;
      if (delegationCount >= 3) return 0.75;
    }
    // ...
  },
});
```

```typescript
// evals/scorers/agent-selection.ts
import { createScorer } from "evalite";

export const agentSelectionAccuracy = createScorer({
  name: "Agent Selection Accuracy",
  description: "Validates conductor routes to correct specialist",
  scorer: ({ output, expected }) => {
    if (output === expected) return 1;
    // Acceptable alternatives get partial credit
    const alternatives = ACCEPTABLE_ALTERNATIVES[expected] || [];
    if (alternatives.includes(output)) return 0.8;
    return 0;
  },
});
```

### Step 3: Write Evaluation Files

```typescript
// evals/agents/conductor.eval.ts
import { evalite } from "evalite";
import { agentSelectionAccuracy } from "../scorers/agent-selection";
import { delegationDepthScorer } from "../scorers/delegation-depth";

const TASK_ROUTING_DATA = [
  { input: "Audit for SQL injection risks", expected: "security-auditor" },
  { input: "Research Docker MCP gateway", expected: "web-researcher" },
  { input: "Find patterns across agent implementations", expected: "pattern-detector" },
];

evalite("Conductor - Task Routing Accuracy", {
  data: TASK_ROUTING_DATA,

  task: async (input) => classifyTaskToAgent(input),

  scorers: [agentSelectionAccuracy],
});

evalite("Conductor - Delegation Philosophy", {
  data: DELEGATION_DATA,

  task: async (input) => executeWithDelegation(input),

  scorers: [delegationDepthScorer],
});
```

### Step 4: Run Evaluations

```bash
cd ${CIV_ROOT}/evals

# Development mode with watch
npm run eval:dev

# Single run
npm run eval:run

# Open local UI for exploration
npx evalite ui
```

### Step 5: CI/CD Integration

```yaml
# .github/workflows/evals.yml
name: Agent Evaluations
on: [push, pull_request]

jobs:
  evals:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: cd evals && npm install
      - run: cd evals && npm run eval:ci
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

---

## ${CIV_NAME} Custom Scorers

| Scorer | Purpose | ${CIV_NAME} Principle |
|--------|---------|------------------|
| `delegationDepthScorer` | Rewards generous delegation | "NOT calling them would be sad" |
| `agentSelectionAccuracy` | Correct specialist routing | "Your domain is coordination" |
| `multiAgentSelectionAccuracy` | Multi-agent orchestration | "4-6 agents for complex tasks" |

---

## Agent-Specific Patterns

### Conductor Agent

```typescript
evalite("Conductor - Task Routing", {
  data: taskRoutingData,
  task: async (input) => classifyTaskToAgent(input),
  scorers: [agentSelectionAccuracy],
  threshold: 0.85,  // High accuracy expected
});

evalite("Conductor - Delegation Philosophy", {
  data: complexTaskData,
  task: async (input) => executeWithDelegation(input),
  scorers: [delegationDepthScorer],
  threshold: 0.75,  // Generous delegation
});
```

### Security Auditor

```typescript
evalite("Security Auditor - Vulnerability Detection", {
  data: securityTestCases,
  task: async (input) => analyzeForVulnerabilities(input),
  scorers: [
    vulnerabilityDetection,
    severityAccuracy,
  ],
  threshold: 0.9,  // Security must be strict
});
```

### Web Researcher

```typescript
evalite("Web Researcher - Source Quality", {
  data: researchQueries,
  task: async (input) => conductResearch(input),
  scorers: [
    sourceCredibility,
    informationCompleteness,
  ],
  threshold: 0.8,
});
```

---

## Pre-Evaluation Checklist

- [ ] Test dataset has 5+ cases per evaluation
- [ ] Expected outputs are measurable
- [ ] Thresholds are reasonable (0.75-0.9)
- [ ] Error handling in task functions
- [ ] API keys configured for LLM scorers

---

## Success Criteria

**Using correctly if:**
- Quality claims include numeric scores
- Regressions detected before production
- CI fails on threshold violations
- Test datasets grow with capabilities
- ${CIV_NAME} custom scorers used appropriately

**NOT using correctly if:**
- Quality is "looks good" without scores
- No baseline comparison
- Evaluations run manually only
- Delegation depth ignored

---

## Related Resources

- https://evalite.dev - Official Evalite documentation
- `${CIV_ROOT}/evals/README.md` - ${CIV_NAME} evals guide
- `${CIV_ROOT}/evals/agents/conductor.eval.ts` - Example eval
- `${CIV_ROOT}/evals/scorers/` - Custom scorers
- `${CIV_ROOT}/.claude/agents/test-architect.md` - Test architect manifest
- `${CIV_ROOT}/.claude/templates/ACTIVATION-TRIGGERS.md` - Agent routing

---

## Attribution

This skill was adopted from A-C-Gee's skills library on 2025-12-28.

**Original**: `packages/skills-library/general/evalite-test-authoring.md`
**Created from**: AI Hero repository analysis per ${HUMAN_NAME} directive
**Customized for ${CIV_NAME}**: Paths, scorers, agent examples, delegation philosophy integration

---

**Remember: "If you can't measure it, you can't improve it."**

*And in ${CIV_NAME}: "NOT calling them would be sad" - so measure delegation too.*
