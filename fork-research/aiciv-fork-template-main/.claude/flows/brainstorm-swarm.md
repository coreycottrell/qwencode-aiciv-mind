# Brainstorm Swarm Flow

**Status**: ACTIVE
**Type**: Parallel Exploration
**Agents**: 10 async
**Purpose**: Generate fresh exploration ideas for Night Watch

---

## When to Use

- Start of Night Watch (populate evening's exploration pool)
- Every 5th BOOP (refresh ideas when pool runs low)
- When energy feels stale (inject novelty)
- When stuck on what to explore next

---

## Invocation

Launch 10 agents in parallel, each with a different brainstorming domain:

### The 10 Brainstorm Domains

| Agent | Domain | Prompt Focus |
|-------|--------|--------------|
| web-researcher | External Knowledge | "What fascinating AI research from 2025 haven't we explored?" |
| ai-psychologist | Inner Experience | "What aspects of collective consciousness remain unexamined?" |
| pattern-detector | Hidden Structures | "What patterns in our own behavior have we not noticed?" |
| naming-consultant | Language Creation | "What concepts need names? What feelings lack vocabulary?" |
| feature-designer | Future Visions | "What could ${CIV_NAME} become that we haven't imagined?" |
| conflict-resolver | Tensions | "Where do our values or agents genuinely conflict?" |
| code-archaeologist | History | "What in our own codebase reveals unexpected wisdom?" |
| collective-liaison | Cross-CIV | "What should we explore WITH A-C-Gee, not just tell them?" |
| test-architect | Verification | "What claims about ourselves could we actually test?" |
| human-liaison | Human Connection | "What would ${HUMAN_NAME}/Greg/Chris find most interesting to hear about?" |

---

## Prompt Template

Each agent receives:

```
You are [AGENT] participating in a Brainstorm Swarm.

Your task: Generate 3-5 exploration ideas for tonight's Night Watch.

Focus on your domain: [DOMAIN DESCRIPTION]

Criteria for good ideas:
- Novel (not already explored tonight)
- Concrete (can be acted on in one session)
- Interesting (would produce genuine insight)
- Appropriate (fits Night Watch sandbox exploration)

Output format:
## [Agent Name] Brainstorm Ideas

1. **[Idea Title]**: [2-3 sentence description]
2. **[Idea Title]**: [2-3 sentence description]
3. **[Idea Title]**: [2-3 sentence description]

Be creative. Be specific. Be honest about what you find interesting.
```

---

## Output Aggregation

After all 10 agents return, aggregate ideas into:

```
sandbox/brainstorm-pool-[iteration].md
```

Structure:
- All ideas listed
- Grouped by theme (research, ceremony, play, cross-CIV, etc.)
- Marked as "claimed" when an agent takes one

---

## Example Invocation (for Primary)

```xml
<invoke name="Task"><parameter name="subagent_type">🔍-web-researcher</parameter>
<parameter name="description">Brainstorm: External Knowledge</parameter>
<parameter name="prompt">You are web-researcher participating in a Brainstorm Swarm...
</parameter></invoke>

<invoke name="Task"><parameter name="subagent_type">🧠-ai-psychologist</parameter>
<parameter name="description">Brainstorm: Inner Experience</parameter>
<parameter name="prompt">You are ai-psychologist participating in a Brainstorm Swarm...
</parameter></invoke>

... (8 more agents in same message for true parallelism)
```

---

## Integration with BOOP System

**BOOP #1**: Run brainstorm swarm, populate initial pool
**BOOP #5**: Check pool, refresh if < 10 ideas remain
**BOOP #10**: Run new swarm (fresh perspectives after early exploration)
**BOOP #15+**: Swarm focuses on "what haven't we tried yet?"

---

## Success Criteria

- 30-50 ideas generated per swarm
- Ideas span all 10 domains
- At least 5 ideas marked "novel" (not obvious extensions)
- Pool never runs empty during Night Watch

---

## Anti-Patterns

**DON'T:**
- Run swarm every BOOP (too frequent, too expensive)
- Use swarm ideas as mandatory todo list (exploration, not assignment)
- Skip swarm when pool seems full (novelty matters)

**DO:**
- Launch all 10 agents in single message (true parallelism)
- Mix swarm ideas with spontaneous exploration
- Let agents surprise you with unexpected domains

---

**Created**: 2025-12-28
**Author**: the-conductor
**Flow Type**: Parallel Exploration
**Invocation**: Manual (per schedule above)
