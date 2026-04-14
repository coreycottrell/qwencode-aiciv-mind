# Democratic Mission Selection Flow

**Status**: ✅ TESTED & VALIDATED (2025-10-01)

## Purpose

Enable the collective to democratically choose the next mission by having all agents propose ideas, evaluate all proposals, and execute the highest-ranked option.

## Pattern Type

**Hybrid**: Parallel brainstorming → Parallel evaluation → Sequential execution

## Agents Involved

**All 14 agents participate**:
- web-researcher
- code-archaeologist
- pattern-detector
- refactoring-specialist
- security-auditor
- test-architect
- feature-designer
- api-architect
- doc-synthesizer
- performance-optimizer
- naming-consultant
- task-decomposer
- result-synthesizer
- conflict-resolver

## Flow Stages

### Stage 1: Idea Generation (Parallel)
**Duration**: ~30-45 minutes

Each agent proposes **one mission idea** that:
- Aligns with their expertise
- Provides clear value to the collective
- Has measurable success criteria
- Includes scope definition

**Deliverable**: 14 mission proposals

### Stage 2: Evaluation (Parallel)
**Duration**: ~45-60 minutes

Each agent reviews **all 14 proposals** and ranks them from best (1st) to least preferred (14th).

**Scoring System**:
- 1st place = 14 points
- 2nd place = 13 points
- ...
- 14th place = 1 point

**Deliverable**: 14 complete rankings (196 total rankings)

### Stage 3: Tallying
**Duration**: ~5 minutes

Calculate total points for each mission and identify the winner.

**Deliverable**:
- Mission rankings table
- Identified winner
- Score breakdown

### Stage 4: Execution
**Duration**: Variable (depends on mission complexity)

Deploy appropriate agents to execute the winning mission.

**Deliverable**: Completed mission with synthesis

## Inputs Required

- **Context**: What is the collective trying to achieve right now?
- **Constraints**: Any limitations (time, scope, resources)?
- **Focus areas**: Any specific domains to prioritize?

## Outputs Produced

1. **Mission proposals document** - All 14 ideas documented
2. **Rankings document** - Complete voting results and scores
3. **Winning mission execution** - Full mission results
4. **Email report** - Sent to user
5. **Git backup** - Committed to repository

## Success Criteria

✅ All 14 agents successfully propose missions
✅ All 14 agents successfully rank all proposals
✅ Clear winner emerges (no ties or conflicts)
✅ Winning mission is successfully executed
✅ Results are documented and reported

## Coordination Notes

### Batching Strategy
Deploy agents in batches of 3-4 to manage parallel executions:
- Batch 1: web-researcher, code-archaeologist, pattern-detector, refactoring-specialist
- Batch 2: security-auditor, test-architect, feature-designer, api-architect
- Batch 3: doc-synthesizer, performance-optimizer, naming-consultant
- Batch 4: task-decomposer, result-synthesizer, conflict-resolver

### Context Management
Each agent needs:
- Access to their own identity file
- Understanding of the collective's current state
- Ability to read other agents' proposals (for ranking stage)

## Real-World Results (2025-10-01)

**Mission Executed**: Democratic Mission Selection
**Proposals Collected**: 14/14 ✅
**Rankings Collected**: 14/14 ✅
**Winner**: Mission 2 - AI-CIV System Dependency Map (141 points)
**Execution**: Successfully completed with 3-agent deployment
**Key Finding**: Broad consensus emerged (winner ranked top 5 by 11/14 agents)

### What Worked Well
- Parallel batching kept execution time reasonable
- Agents provided thoughtful, diverse proposals
- Ranked-choice voting revealed clear consensus
- Each agent brought unique perspective to rankings

### What Could Improve
- Could compress proposal stage with tighter constraints
- Rankings could include brief justifications
- Tie-breaking protocol needed for close votes

## When to Use This Flow

**Good for**:
- Deciding next major feature or capability
- Choosing between multiple strategic directions
- Ensuring buy-in from all agent perspectives
- Discovering creative solutions the Conductor might miss

**Not ideal for**:
- Urgent decisions (too time-consuming)
- Simple binary choices (overkill)
- Situations requiring deep expertise in one domain

## Variations

### Speed Run Version
- Each agent proposes in 1 paragraph
- Limit to top 3 rankings only
- Execute with smaller agent team (2-3 agents)

### Deep Deliberation Version
- Agents propose with full design docs
- Multiple ranking rounds with discussion
- Full collective deployment for execution

### Domain-Specific Version
- Only relevant agents participate (e.g., only engineering agents for technical decisions)
- Weighted voting based on expertise alignment

---

**Democratic Mission Selection demonstrates the collective's ability to self-organize and make decisions together.** 🗳️✨
