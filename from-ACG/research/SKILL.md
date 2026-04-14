---
description: "Parallel research orchestration - breaks queries into focused sub-searches across multiple researcher agents. Use when the user types /research [query], asks to research something, look into a topic, or needs comprehensive web-based information gathering."
triggers:
  - /research
  - research this
  - look into
  - find out about
allowed-tools:
  - Task
  - Read
  - Write
---

# Research Skill: Parallel Multi-Agent Search

**Invocation**: `/research [query]`

**Core Principle**: Launch 3-5 researcher agents IN PARALLEL with focused, narrow queries. Never send one monolithic agent on a sprawling 10+ search odyssey.

---

## Why Parallel Research

| Old Pattern (Monolithic) | New Pattern (Parallel) |
|--------------------------|------------------------|
| 1 agent, 10+ searches | 3-5 agents, 2-3 searches each |
| Sequential, slow | Parallel, fast |
| Timeout risk (agent runs too long) | Each agent finishes in under 30 seconds |
| One failure kills everything | Partial results still useful |
| Context window bloat | Each agent stays focused |

---

## The Protocol (Step by Step)

### Step 1: Decompose the Query

When a user asks `/research [query]`, the Primary AI breaks it into 3-5 focused sub-queries.

**Rules for decomposition:**
- Each sub-query targets a SPECIFIC facet of the topic
- Sub-queries should be non-overlapping (minimize redundancy)
- Each sub-query should be answerable with 2-3 web searches
- Include specific names, products, companies, or terms (not vague)
- Frame as search engine queries, not essay prompts

**Example decomposition:**

User asks: `/research AI agent frameworks 2026`

| Agent | Focused Query |
|-------|--------------|
| 1 | "OpenAI Agents SDK and Responses API developments in 2026" |
| 2 | "Anthropic Claude Agent SDK and Model Context Protocol updates 2026" |
| 3 | "Google ADK and open source agent frameworks LangGraph CrewAI AutoGen 2026" |
| 4 | "Enterprise AI agent deployment trends and production patterns 2026" |

User asks: `/research best practices for RAG pipelines`

| Agent | Focused Query |
|-------|--------------|
| 1 | "RAG chunking strategies and embedding model selection best practices 2025-2026" |
| 2 | "RAG retrieval optimization: hybrid search, reranking, and query expansion" |
| 3 | "RAG evaluation metrics and common failure modes (hallucination, context poisoning)" |
| 4 | "Production RAG architectures: vector databases, caching, and scaling patterns" |

User asks: `/research Florida LLC formation for AI company`

| Agent | Focused Query |
|-------|--------------|
| 1 | "Florida LLC formation process steps filing requirements 2026" |
| 2 | "Florida LLC operating agreement requirements for AI technology companies" |
| 3 | "Florida LLC tax implications: state tax, federal pass-through, annual report fees" |
| 4 | "AI company legal considerations: IP assignment, liability, terms of service" |

### Step 2: Launch Parallel Researcher Agents

**CRITICAL: All Task calls go in ONE message block so they execute in parallel.**

```
Task(researcher): "FOCUSED SEARCH: [sub-query 1]. Do 2-3 web searches MAX. Return bullet-point findings with source URLs. Be fast and focused - do NOT go deep on tangents."

Task(researcher): "FOCUSED SEARCH: [sub-query 2]. Do 2-3 web searches MAX. Return bullet-point findings with source URLs. Be fast and focused - do NOT go deep on tangents."

Task(researcher): "FOCUSED SEARCH: [sub-query 3]. Do 2-3 web searches MAX. Return bullet-point findings with source URLs. Be fast and focused - do NOT go deep on tangents."

Task(researcher): "FOCUSED SEARCH: [sub-query 4]. Do 2-3 web searches MAX. Return bullet-point findings with source URLs. Be fast and focused - do NOT go deep on tangents."
```

**Key instruction to each agent:**
- "FOCUSED SEARCH:" prefix signals narrow scope
- "2-3 web searches MAX" prevents scope creep
- "bullet-point findings with source URLs" ensures structured output
- "Be fast and focused" prevents tangent-chasing

### Step 3: Synthesize Results

After all agents return, Primary synthesizes:

1. **Merge findings** - Combine all bullet points by theme
2. **Deduplicate** - Remove overlapping information
3. **Identify patterns** - What do multiple agents agree on?
4. **Note gaps** - What was NOT found? What needs follow-up?
5. **Present coherently** - Structure as a unified research report

**Synthesis output format:**

```markdown
## Research: [Original Query]

### Key Findings
- [Most important finding, with source]
- [Second finding, with source]
- [Third finding, with source]

### Detailed Breakdown

#### [Theme 1]
- Finding (Source: URL)
- Finding (Source: URL)

#### [Theme 2]
- Finding (Source: URL)
- Finding (Source: URL)

### Gaps & Follow-Up
- [What was not covered or needs deeper investigation]

### Sources
1. [URL list from all agents]
```

---

## Agent Instruction Template

Each researcher agent receives this structure:

```
FOCUSED SEARCH: [specific sub-query]

Instructions:
- Do 2-3 web searches using DuckDuckGo (ddgs library)
- For each search, extract the top 3-5 most relevant results
- If a result looks particularly valuable, use WebFetch to read the full page
- Return your findings as bullet points with source URLs
- Do NOT write files - just return your findings directly
- Stay focused on your specific query - do not expand scope
- Target completion: under 30 seconds
```

---

## When to Use Fewer or More Agents

| Situation | Agents | Rationale |
|-----------|--------|-----------|
| Simple factual lookup | 2-3 | Topic is narrow |
| Broad multi-faceted topic | 4-5 | Need coverage across dimensions |
| Comparison query (A vs B vs C) | 3 | One agent per thing being compared |
| Current events / breaking news | 3-4 | News + analysis + reaction |
| Technical how-to | 3 | Concept + implementation + gotchas |

---

## Gateway Integration

This slash command is registered in the AiCIV Gateway's `SLASH_COMMAND_ROUTES`:
- Route: `/research` -> triggers this parallel orchestration skill
- Default prompt (bare command): "What topic?"
- Frontend button: Research (pre-fills `/research ` for user to type query)

---

## Anti-Patterns

| Anti-Pattern | Why It Fails | Do This Instead |
|--------------|-------------|-----------------|
| One agent with the full query | Timeout, context bloat, slow | Decompose into 3-5 focused agents |
| Vague sub-queries ("learn about AI") | Agent doesn't know what to search | Specific terms, names, years |
| 5+ searches per agent | Slow, loses focus, risks timeout | 2-3 searches MAX per agent |
| Agents writing full reports | Wastes time on formatting | Bullet points only; Primary formats |
| Sequential agent launches | Wastes time waiting | ALL Task calls in ONE message block |
| Not synthesizing | User gets 4 disconnected reports | Primary merges into one coherent answer |

---

## Researcher Agent Capabilities

The researcher agent (`.claude/agents/researcher.md`) has access to:
- **WebSearch**: DuckDuckGo via `ddgs` library (no API key needed)
- **WebFetch**: Direct URL content retrieval
- **Read/Grep/Glob**: Local codebase and file exploration
- **Skills**: `deep-search`, `jina-reader`, `article-extract`, `youtube-transcript`

---

## Performance Expectations

| Metric | Target |
|--------|--------|
| Time per agent | Under 30 seconds |
| Searches per agent | 2-3 |
| Total research time | Under 60 seconds (parallel) |
| Coverage | 80%+ of query facets addressed |
| Source quality | Official docs > blogs > forums |

---

## Related Skills

- `.claude/skills/deep-search/SKILL.md` - Web search mechanics (ddgs library)
- `.claude/skills/jina-reader/SKILL.md` - URL content extraction
- `.claude/skills/article-extract/SKILL.md` - Article content parsing
- `.claude/skills/youtube-transcript/SKILL.md` - YouTube transcript extraction
