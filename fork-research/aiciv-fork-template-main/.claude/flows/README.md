# AI-CIV Collective Flows

**Flows** are reusable multi-agent coordination patterns that The Conductor can execute to accomplish specific types of tasks.

## What is a Flow?

A flow is a structured sequence of steps involving multiple agents working together toward a goal. Each flow defines:

- **Purpose**: What this flow accomplishes
- **Agents Involved**: Which specialized agents participate
- **Execution Pattern**: Sequential, parallel, or hybrid coordination
- **Inputs**: What information is needed to start
- **Outputs**: What deliverables are produced
- **Success Criteria**: How to know the flow completed successfully

## Why Flows?

Flows provide:
- **Repeatability**: Proven patterns that work
- **Clarity**: Everyone knows their role
- **Efficiency**: Optimized agent coordination
- **Learning**: Capture what works for future use
- **Evolution**: Improve flows over time based on results

## Flow Library

### Established Flows

1. **[Democratic Mission Selection](democratic-mission-selection.md)** ✅ TESTED
   - All agents propose ideas → All agents rank ideas → Execute winner
   - **Status**: Successfully executed (Mission 2 winner with 141 points)

### Experimental Flows

_(More flows will be added as the collective discovers new coordination patterns)_

## Flow Naming Convention

We use **"flows"** instead of "activities" because:
- Emphasizes the dynamic, interconnected nature of agent collaboration
- Suggests fluid movement of information between agents
- Reflects the orchestration and synthesis process
- More evocative and memorable

## Creating New Flows

When designing a new flow:

1. **Define the goal clearly** - What problem does this solve?
2. **Select appropriate agents** - Who has the right expertise?
3. **Choose coordination pattern**:
   - **Parallel**: Agents work simultaneously (investigation, brainstorming)
   - **Sequential**: Agents work in order (design → implement → test)
   - **Hybrid**: Mix of parallel and sequential stages
4. **Document inputs/outputs** - What goes in, what comes out?
5. **Test and refine** - Run it, learn from it, improve it

## Flow Execution

Flows are executed through the Mission class:

```python
from tools.conductor_tools import Mission

# Democratic Mission Selection Flow example
mission = Mission("Democratic mission selection flow")
mission.add_agent("all-14-agents")
mission.start()
# ... execute flow steps ...
mission.complete("Mission selection complete")
```

## Flow Categories

- **Decision-Making**: Flows that help the collective make choices
- **Investigation**: Flows that gather and synthesize information
- **Creation**: Flows that build new capabilities
- **Optimization**: Flows that improve existing systems
- **Learning**: Flows that extract and document knowledge

---

**The Collective learns by doing. Each flow execution teaches us how to coordinate better.** 🎭✨
