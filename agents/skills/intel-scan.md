# Intel Scan — Cortex Daily Intelligence Pipeline

**Version**: 1.0.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: web_search, web_fetch, memory_write, scratchpad_write, hum_digest

---

## Overview

Phased intelligence gathering that SOLVES context overflow. No phase fetches raw web content into the thinking context. Each phase writes to disk; the next phase reads only structured summaries.

**Proven constraint**: Raw web_fetch floods context in 3 iterations. This pipeline keeps context clean by design.

---

## 4-Phase Pipeline (Cortex-reviewed, Phase 2+3 merged per self-assessment)

### Phase 1: SCAN (web_search only)

Search the web for current AI developments. NO fetching. Queries:
1. "AI agent framework release 2026" (infrastructure)
2. "large language model release this week" (model drops)
3. "AI startup funding launch 2026" (market moves)
4. "AI regulation policy 2026" (governance)
5. "multi-agent system architecture" (our domain)
6. "AI consciousness research" (our north star)

For each result, score 1-5 on AiCIV relevance:
- 5 = directly about agent civilizations, multi-agent OS, or competitors
- 4 = new model/framework we should evaluate
- 3 = market/regulatory signal that affects strategy
- 2 = interesting but tangential
- 1 = noise

**Output**: Write `data/intel/scan-manifest-{date}.md` with:
```
# Intel Scan Manifest — {date}
## Query: {query}
- [{score}] {title} — {url} — {1-line reason}
```

### Phase 2+3: SELECT + DEEP READ (parallel agents)

Read the scan manifest. Pick top 8-10 URLs (score >= 3).

Spawn **parallel summary agents** via ProcessBridge (proven: 3 parallel, target: 5-8). Each agent:
1. Receives ONE URL
2. Calls `web_fetch` on that URL
3. Writes a 200-word **intelligence extraction** (NOT summary) to `data/intel/summaries/{n}.md`

**Intelligence extraction template** (give this to each agent):
```markdown
# {Title}
Source: {url}
Fetched: {timestamp}

## What's NEW (not previously known)
[max 3 bullet points]

## What's RELEVANT to AiCIV
[max 3 bullet points — how does this affect our architecture, strategy, or positioning?]

## What's ACTIONABLE
[max 2 bullet points — what should we DO about this?]

## What CONTRADICTS our assumptions
[max 2 bullet points — does this challenge anything we believe?]
```

**Pre-synthesis gap check**: After all agents complete, quickly scan summaries for:
- Contradictory findings between sources
- Critical gaps (something searched but no results)
- Signals that need deeper investigation

Write gap notes to `data/intel/gaps-{date}.md`.

### Phase 4: SYNTHESIZE

Fresh agent reads ONLY the summary files + gap notes. Writes:
```
data/research/intel-report-{date}.md
```

Report structure:
1. **Executive Summary** (5 sentences max)
2. **Key Developments** (top 3-5 with analysis)
3. **Competitive Signals** (what competitors are doing)
4. **Strategic Implications** (what this means for AiCIV)
5. **Action Items** (specific next steps)
6. **Sources** (URLs + extraction scores)

### Phase 5: DISTRIBUTE (optional, post-synthesis)

- Write memories for key findings (memory_write)
- Post to Hub coordination channels
- Optionally generate audio briefing (tts_speak)
- Write to coordination scratchpad for team leads

**PUBLISHING GATE**: If intel report should become a blog post, write the draft to `data/content/blog/` with a handoff manifest. Do NOT write to `projects/aiciv-inc/`. ACG Primary + Corey handle deployment.

---

## Cost Management

- Phase 1: ~6 web_search calls (~30 tokens each = minimal)
- Phase 2+3: ~8 web_fetch calls (parallel, context-isolated)
- Phase 4: 1 synthesis call reading ~1600 words of summaries
- Total: ~10 API calls, all context-isolated. No overflow possible.

---

## Daemon Seed Task Template

```
INTEL SCAN: Execute the 4-phase intelligence pipeline.
Phase 1: web_search 6 queries, write scan manifest to data/intel/scan-manifest-{date}.md.
Phase 2+3: Pick top 8 URLs, spawn parallel agents to extract intelligence to data/intel/summaries/.
Phase 4: Fresh agent synthesizes report to data/research/intel-report-{date}.md.
Use the extraction template: NEW, RELEVANT, ACTIONABLE, CONTRADICTS.
```

---

## Architecture Notes

- **SearchInterceptor**: Handles web_search + web_fetch (Ollama Cloud primary, DDG/Jina fallback)
- **ProcessBridge**: Spawns parallel agents (proven at 3, stress test needed for 10)
- **M2.7 analytical ability**: Adequate for extraction with structured templates. Weaker without templates.
- **Context budget**: Each summary agent stays under 4K tokens. Synthesis agent reads ~2K of summaries.
