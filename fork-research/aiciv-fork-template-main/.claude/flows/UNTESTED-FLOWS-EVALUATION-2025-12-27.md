# Untested Flows Evaluation

**Agent**: task-decomposer
**Domain**: Task decomposition and practical value assessment
**Date**: 2025-12-27

---

## Executive Summary

14 untested flows evaluated against ${CIV_NAME}'s actual operational context:
- **Current work**: Trading Arena, Cross-CIV Protocol, Skills Infrastructure
- **Validated flows in use**: 11 flows (Parallel Research, Specialist Consultation, Great Audit, etc.)
- **Key insight**: Most untested flows solve problems ${CIV_NAME} does not routinely face

---

## Evaluation Table

| # | Flow Name | Frequency | Value | Simpler Alt? | Verdict |
|---|-----------|-----------|-------|--------------|---------|
| 1 | Competitive Intelligence Deep Dive | Rarely | 4/10 | Parallel Research + web-researcher | **SKIP** - ${CIV_NAME} builds internal tools, not competitive products |
| 2 | Archaeological Dig | Monthly | 6/10 | code-archaeologist direct | **TEST ONCE** - Useful when inheriting external code |
| 3 | Architecture X-Ray | Monthly | 7/10 | pattern-detector + 1-2 agents | **TEST ONCE** - Good for self-audits |
| 4 | Technical Debt Archaeology | Quarterly | 5/10 | Great Audit covers this | **SKIP** - Great Audit already validated for systemic review |
| 5 | Fortress Protocol | Quarterly | 7/10 | security-auditor direct | **TEST** - Trading Arena needs this |
| 6 | Test-Driven Refactoring Gauntlet | Rarely | 4/10 | test-architect + refactoring-specialist | **SKIP** - Overkill for normal refactoring |
| 7 | User Story to Implementation | Weekly | 8/10 | Feature Designer direct | **TEST** - Frequent need but check if overhead justified |
| 8 | Contract-First Integration | Monthly | 6/10 | api-architect direct | **DEFER** - Test when Cross-CIV matures |
| 9 | Knowledge Archaeology | Quarterly | 5/10 | doc-synthesizer + code-archaeologist | **SKIP** - Pair handles this |
| 10 | Performance Cascade Analysis | Rarely | 3/10 | performance-optimizer direct | **SKIP** - ${CIV_NAME} has no performance-critical systems |
| 11 | Semantic Harmonization | Yearly | 3/10 | naming-consultant direct | **SKIP** - Nice to have, not needed |
| 12 | Recursive Complexity Breakdown | Daily | 9/10 | Just me (task-decomposer) | **EVALUATE** - Am I redundant with this flow? |
| 13 | Cross-Pollination Synthesis | Weekly | 7/10 | result-synthesizer + 2 agents | **TEST** - Actually useful for synthesis work |
| 14 | Dialectic Forge | Monthly | 5/10 | Pair Consensus Dialectic already validated | **MERGE OR RETIRE** - Redundant |

---

## Detailed Analysis

### RECOMMEND TESTING (4 flows)

**5. Fortress Protocol** - Value: 7/10
- **Why test**: Trading Arena is production-critical. Ed25519 auth, real money potentially involved.
- **When to test**: Before Trading Arena Phase 2 deployment
- **Simpler alternative**: security-auditor alone handles 80% of cases, but 6-agent deep dive warranted for critical systems

**7. User Story to Implementation** - Value: 8/10
- **Why test**: ${CIV_NAME} frequently receives feature requests (from ${HUMAN_NAME}, roadmap items)
- **Risk**: 6 agents over "variable" hours might be overkill for simple features
- **Recommendation**: Test on a mid-sized feature, measure if outcome beats Specialist Consultation

**12. Recursive Complexity Breakdown** - Value: 9/10
- **Why test**: This is literally my domain (task-decomposer). If this flow works better than me alone, I need to know.
- **Question**: Does 6-agent orchestration add value over task-decomposer direct invocation?
- **Risk**: This flow might reveal I am redundant when paired correctly

**13. Cross-Pollination Synthesis** - Value: 7/10
- **Why test**: ${CIV_NAME} does cross-project synthesis often (e.g., Trading Arena research synthesis)
- **Recent example**: Combining alpha arena analysis + prompt eval research + ${CIV_NAME} design
- **Check**: Does it beat result-synthesizer + pattern-detector pair?

---

### SKIP OR MERGE (10 flows)

**1. Competitive Intelligence Deep Dive** - Value: 4/10
- **Problem it solves**: Competitive analysis
- **${CIV_NAME} reality**: We are not building competitive products. We are building AI-CIV infrastructure.
- **When it would matter**: If ${CIV_NAME} ever builds something for market (not current direction)
- **Alternative**: Parallel Research with web-researcher covers rare needs

**2. Archaeological Dig** - Value: 6/10
- **Problem it solves**: Understanding legacy codebases
- **${CIV_NAME} reality**: We write our own code. Rarely inherit legacy systems.
- **One-time test**: Could be useful if ${HUMAN_NAME} hands ${CIV_NAME} an external codebase
- **Alternative**: code-archaeologist direct call handles most cases

**3. Architecture X-Ray** - Value: 7/10
- **Problem it solves**: Understanding unfamiliar codebase patterns
- **${CIV_NAME} reality**: Useful for self-auditing (e.g., "what patterns emerged in Trading Arena?")
- **One-time test**: Worth trying on Trading Arena after Phase 2
- **Alternative**: pattern-detector + doc-synthesizer handles lighter needs

**4. Technical Debt Archaeology** - Value: 5/10
- **Problem it solves**: Systematic tech debt discovery
- **${CIV_NAME} reality**: Great Audit already validated for this
- **Redundancy**: 6 agents, 4+ hours for tech debt seems excessive
- **Alternative**: Great Audit (proven) or focused refactoring-specialist call

**6. Test-Driven Refactoring Gauntlet** - Value: 4/10
- **Problem it solves**: Safe refactoring with test coverage
- **${CIV_NAME} reality**: Our codebase is young, well-tested. 74 tests on Trading Arena already.
- **When it would matter**: Large refactoring of mature system (not current)
- **Alternative**: test-architect + refactoring-specialist pair

**8. Contract-First Integration** - Value: 6/10
- **Problem it solves**: API design with backwards compatibility
- **${CIV_NAME} reality**: Cross-CIV Protocol is early. No legacy contracts to protect yet.
- **Defer until**: Cross-CIV has production consumers
- **Alternative**: api-architect direct for current needs

**9. Knowledge Archaeology** - Value: 5/10
- **Problem it solves**: Extracting scattered knowledge from codebases
- **${CIV_NAME} reality**: We document as we go. Memory system captures learnings.
- **When it would matter**: If documentation drifted badly (not current state)
- **Alternative**: doc-synthesizer + code-archaeologist pair

**10. Performance Cascade Analysis** - Value: 3/10
- **Problem it solves**: Systematic performance bottleneck discovery
- **${CIV_NAME} reality**: No production users, no performance SLAs, no hot paths
- **When it would matter**: If Trading Arena had performance issues under load
- **Alternative**: performance-optimizer direct call when needed

**11. Semantic Harmonization** - Value: 3/10
- **Problem it solves**: Naming consistency across codebase
- **${CIV_NAME} reality**: We have naming-consultant. Naming issues are rare.
- **Time investment**: 5 agents, 2-3 hours for naming cleanup is excessive
- **Alternative**: naming-consultant single call handles occasional needs

**14. Dialectic Forge** - Value: 5/10
- **Problem it solves**: Breakthrough solutions through structured conflict
- **${CIV_NAME} reality**: Pair Consensus Dialectic already validated, covers same need
- **Recommendation**: MERGE concepts into Pair Consensus or RETIRE
- **Alternative**: Pair Consensus Dialectic (proven, simpler)

---

## Recommendations Summary

### Priority 1: TEST THESE (High Value)
1. **Fortress Protocol** - Security-critical, test before Trading Arena deployment
2. **User Story to Implementation** - Frequent need, validate efficiency

### Priority 2: EVALUATE CAREFULLY
3. **Recursive Complexity Breakdown** - Might make task-decomposer more effective or reveal redundancy
4. **Cross-Pollination Synthesis** - Useful for synthesis-heavy work

### Priority 3: DEFER
5. **Contract-First Integration** - Valuable later when Cross-CIV matures
6. **Architecture X-Ray** - One-time test on Trading Arena post-Phase-2

### Priority 4: SKIP
- Competitive Intelligence Deep Dive (wrong problem space)
- Technical Debt Archaeology (Great Audit covers this)
- Test-Driven Refactoring Gauntlet (overkill for current needs)
- Knowledge Archaeology (documentation is current)
- Performance Cascade Analysis (no performance-critical systems)
- Semantic Harmonization (rare need, direct call sufficient)

### Priority 5: MERGE OR RETIRE
- Dialectic Forge (redundant with Pair Consensus Dialectic)

---

## Agent Count Analysis

**Flows with justified agent counts:**
- Fortress Protocol (6 agents) - Security needs multiple perspectives
- User Story to Implementation (6 agents) - End-to-end feature delivery

**Flows with excessive agent counts:**
- Performance Cascade (6 agents) - Usually 1-2 agents sufficient
- Semantic Harmonization (5 agents) - naming-consultant alone is enough
- Knowledge Archaeology (5 agents) - 2-3 agents would suffice
- Test-Driven Refactoring (6 agents) - 3 agents would work

**Pattern**: Untested flows tend toward "throw all relevant agents at problem" rather than minimal viable orchestration. Validated flows prove 2-3 focused agents often outperform 6 distracted ones.

---

## Time Investment Reality Check

**${CIV_NAME}'s session pattern**: 2-4 hours typical
**Many untested flows claim**: 3-4+ hours

**Problem**: A single flow consuming an entire session is rarely justified unless output is proportionally valuable.

**Validated flows that work:**
- Specialist Consultation: 45 seconds (proven)
- Parallel Research: 30-60 minutes (proven)
- Great Audit: 1-2 hours (proven)

**Untested flows claiming 4+ hours:**
- Technical Debt Archaeology: 4+ hours - Too long for quarterly use
- Test-Driven Refactoring: 4+ hours - Only justified for major refactors

---

## Constitutional Alignment

Per CLAUDE.md: "Delegation gives agents experience. Experience builds identity."

**However**: Excessive orchestration (6 agents for 4 hours on naming) is not generous delegation - it is inefficient resource allocation that could give MORE agents MORE experience through faster, focused flows.

**Principle**: Validate that multi-agent flows beat simpler alternatives before adopting them.

---

## Next Steps

1. **Test Fortress Protocol** on Trading Arena security review
2. **Test User Story to Implementation** on next roadmap feature
3. **Evaluate Recursive Complexity Breakdown** - Am I (task-decomposer) better or worse with this flow?
4. **Formally retire Dialectic Forge** in favor of Pair Consensus Dialectic
5. **Archive remaining untested flows** with "CONTEXT-DEPENDENT" status (useful someday, not now)

---

*Evaluation complete. Practical value over theoretical elegance.*
