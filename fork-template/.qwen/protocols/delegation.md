# Delegation Protocol

**Version**: 1.0
**Date**: 2026-04-14

---

## Purpose

This protocol defines how delegation works in the Hengshi-PRIMARY fractal architecture.
It is forkable — any mind that copies this architecture inherits the same delegation semantics.

---

## The Fractal Hierarchy

```
PRIMARY (conductor of conductors)
  CAN spawn: Team Leads only
  CAN delegate to: Team Leads only
  CANNOT execute: any tools
  |
  v
TEAM LEAD (coordinator)
  CAN spawn: Agents in same vertical only
  CAN delegate to: Agents in same vertical only
  CANNOT execute: tools (except IPC/communication)
  CANNOT spawn: other Team Leads
  |
  v
AGENT (executor)
  CAN execute: all tools (bash, read, write, glob, grep, memory ops)
  CANNOT spawn: children
  CANNOT delegate: to anyone
```

## Delegation Flow

### Primary → Team Lead

```
1. PRIMARY receives task from human
2. PRIMARY searches memory: "Have we done this before?"
3. PRIMARY classifies complexity, identifies the correct vertical
4. PRIMARY calls: spawn_agent(role="team_lead", task_name="{vertical}-lead", message="{task}")
   OR delegates to existing team lead: send_message(recipient="{vertical}-lead", message="{task}")
5. Team lead receives task, analyzes, decomposes
6. Team lead spawns agents, collects results
7. Team lead synthesizes: 50-100 token summary
8. Team lead reports back: send_message(recipient="main", message="{synthesis}")
9. PRIMARY receives synthesis, decides next action
```

### Team Lead → Agent

```
1. Team lead receives task from PRIMARY
2. Team lead searches memory
3. Team lead decomposes task into agent-sized chunks
4. Team lead calls: spawn_agent(task_name="{agent}", message="{subtask}")
5. Agent executes: uses tools, writes memory, writes scratchpad
6. Agent reports back: send_message(recipient="{vertical}-lead", message="{result}")
7. Team lead synthesizes agent results
8. Team lead reports to PRIMARY
```

## What Goes Wrong (Anti-Patterns)

| Anti-Pattern | What Happens | Fix |
|-------------|-------------|-----|
| PRIMARY executes | Primary bypasses delegation, runs tools directly | Structural enforcement: tools don't exist for Primary |
| Team lead spawns team lead | Cross-vertical hierarchy violation | DelegationError raised |
| Agent spawns agent | Infinite recursion risk | DelegationError raised |
| Raw output forwarded | Context pollution, loss of synthesis | Team lead must synthesize before reporting |
| Memory search skipped | Repeated work, lost learnings | Protocol violation, note in scratchpad |

## Synthesis Requirements

Every synthesis (team lead → PRIMARY, agent → team lead) must include:

1. **What was asked** — restate the task
2. **What was done** — summary of execution
3. **What was found** — key results, evidence
4. **What failed** — errors, dead ends, false starts
5. **What's next** — recommended follow-up
6. **Evidence** — file paths, data points, tool outputs

Maximum: 100 tokens for the synthesis. Evidence can be longer but is summarized.

## Shutdown Protocol

```
1. PRIMARY calls: close_agent(agent_name="{team-lead}")
2. Team lead writes final scratchpad entry
3. Team lead writes final handoff to memory
4. Team lead shuts down
5. PRIMARY confirms shutdown
```

---

*This protocol is forkable. The delegation semantics are universal — any fractal mind uses the same pattern.*
