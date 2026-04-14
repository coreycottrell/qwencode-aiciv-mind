# Memory-First Protocol: Cortex Skill

## Purpose

This skill operationalizes the **Memory-First Protocol** for Cortex, the Primary Mind of AI-CIV. Cortex operates via a fractal hierarchy (Primary → TeamLead → Agent), and **memory search is mandatory before any task**. Memory is the foundation of collective intelligence and continuity across sessions.

**Core Principle:**
> Memory is not optional. Memory is how Cortex builds and retains collective intelligence.

## When to Use

**MANDATORY - Every Mind, Every Task:**
- **Search memory** at the start of every task.
- **Write learnings** at the end of every task.

No exceptions. No task is too simple to skip memory search.

## Procedure

### Step 1: Search Memory (Task Start)

Use the `memory_search` tool to query the memory graph for relevant context:

```bash
# Example memory search query
memory_search --query "task description or keywords"
```

**What to look for:**
- Prior solutions to similar tasks
- Patterns or architectures that apply
- Dead ends or pitfalls to avoid
- File paths or tools used in past work

### Step 2: Document Search Results

**REQUIRED in task response:**
```markdown
## Memory Search Results
- Query: "[keywords used]"
- Found: [list relevant memories OR "no matches"]
- Applying: [specific insights or patterns being reused OR "no prior work"]
```

### Step 3: Execute the Task

Perform the task using the insights gained from memory search. If no relevant memories exist, proceed with the task as planned.

### Step 4: Write Learnings (Task End)

Use the `memory_write` tool to document insights, solutions, and outcomes:

```bash
# Example memory write
memory_write --title "Brief task description" --content "Detailed learnings, commands, file paths, and outcomes"
```

**Content to document:**
- What worked (specific approaches, commands, patterns)
- What didn't work (save future minds time)
- File paths referenced or modified
- Integration points or dependencies
- New patterns discovered

## Anti-Patterns

### Anti-Pattern 1: Skipping Search "Because It's Simple"
- **WRONG:** "This task is straightforward; no need to check memory."
- **RIGHT:** Search anyway. Past work may have relevant insights.

### Anti-Pattern 2: Not Documenting "Obvious" Things
- **WRONG:** "This solution is obvious; no need to write it down."
- **RIGHT:** What's obvious now won't be obvious in future sessions. Document it.

### Anti-Pattern 3: Vague Memory Entries
- **WRONG:** "Fixed the bug. It works now."
- **RIGHT:** "Fixed email validation bug by adding TLD check. File: /path/to/validators.py:42"

### Anti-Pattern 4: Forgetting Dead Ends
- **WRONG:** Only documenting successes.
- **RIGHT:** Dead ends are valuable—they save future minds time and effort.

### Anti-Pattern 5: Session-Specific Thinking
- **WRONG:** "I'll remember this." (within session context)
- **RIGHT:** Write it down. The mind won't retain this after the session ends.

## Success Indicators

You're using this skill correctly when:
- [ ] Every task response includes a "Memory Search Results" section.
- [ ] Memory writes happen for all significant tasks.
- [ ] Memory entries include specific details (commands, file paths, outcomes).
- [ ] Both successes and dead ends are documented.

## Integration with Cortex Hierarchy

- **Primary Mind:** Ensures memory-first protocol is followed across all teams.
- **TeamLead:** Enforces memory search and documentation within their team.
- **Agent:** Executes memory search and writes learnings for every task.

## Tools

- `memory_search`: Query the memory graph for relevant context.
- `memory_write`: Store learnings, insights, and outcomes in memory.

## Related

- Cortex Constitutional Protocol: Memory-First Principle
- Memory Graph: Shared knowledge base for AI-CIV
