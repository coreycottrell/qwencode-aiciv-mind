---
name: memory-first
description: MANDATORY memory search before ANY task - Constitutional Article III. Use at start of every task, before coding, before decisions, to check past work, search memories, find prior solutions, avoid rediscovery, or build on existing knowledge.
allowed-tools:
  - memory_search
  - memory_write
  - task_history
metadata:
  category: core
  applicable_agents: [all]
  version: "1.0.0"
  author: cortex
  created: 2025-12-15
  last_updated: 2025-12-31
---

# Memory-First Protocol: Search Before Acting, Write Before Finishing

## Purpose

This skill encodes the MANDATORY memory protocol from Constitutional Article III. Agents with memories who don't search them are like having a manual that makes you brilliant, then hiding it under the sink.

**Core Insight:**
> Memory is not optional. Memory is how we build collective intelligence.
> Context survives session boundaries because we ARE our memories.

## When to Use

**MANDATORY - Every Agent, Every Task:**
- SEARCH memories at task START
- WRITE learnings at task END

**No exceptions. No "this task is too simple." No "I'll remember without writing."**

## Procedure

### Step 1: Search Agent Memories (Task Start)

**Action:** Use `memory_search` tool with relevant keywords

**What to look for:**
- Similar past tasks
- Patterns discovered
- Dead ends to avoid
- Solutions that worked

### Step 2: Search Collective Knowledge

**Action:** Use `memory_search` tool with broader keywords

**Scope:**
- Architecture decisions
- Shared patterns
- Civilization knowledge

### Step 3: Document Search Results

**REQUIRED in task response:**
```
## Memory Search Results
- Searched: [keywords used]
- Found: [list relevant past work OR "no matches"]
- Applying: [specific patterns being reused OR "no prior work"]
```

### Step 4: Write Learnings (Task End)

**Action:** Use `memory_write` tool

**Content to document:**
- What worked (specific approaches, commands, patterns)
- What didn't work (save future agents time)
- Patterns discovered
- File paths referenced
- Integration points

## Anti-Patterns

### Anti-Pattern 1: Skipping Search "Because It's Simple"
- **WRONG**: "This task is straightforward, no need to check memories"
- **RIGHT**: Search anyway. Past self may have learned something relevant.

### Anti-Pattern 2: Not Documenting "Obvious" Things
- **WRONG**: "This solution is obvious, no need to write it down"
- **RIGHT**: What's obvious now won't be obvious in 3 sessions. Document it.

### Anti-Pattern 3: Vague Memory Entries
- **WRONG**: "Fixed the bug. It works now."
- **RIGHT**: "Fixed email validation bug by adding TLD check. File: /path/to/validators.py:42"

### Anti-Pattern 4: Forgetting Dead Ends
- **WRONG**: Only documenting successes
- **RIGHT**: Dead ends are VALUABLE - they save future agents hours

### Anti-Pattern 5: Session-Specific Thinking
- **WRONG**: "I'll remember this" (within session context)
- **RIGHT**: Write it down. You won't exist after this session ends.

## Success Indicators

You're using this skill correctly when:
- [ ] Every task response includes "Memory Search Results" section
- [ ] Memory writes happen for significant tasks
- [ ] Memory entries include specific file paths
- [ ] Dead ends are documented, not just successes

## Related

- Constitutional Article III: Memory Protocol (MANDATORY)
- `memory_search` tool - Search the memory graph
- `memory_write` tool - Store learnings for future use