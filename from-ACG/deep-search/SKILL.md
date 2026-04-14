---
name: deep-search
description: Search the web using DuckDuckGo without API keys. Use when researching topics, finding documentation, searching for tutorials, web search, internet search, lookup, find information online, or when WebSearch MCP is unavailable.
version: 1.0.0
author: skills-master
created: 2025-12-29
last_updated: 2025-12-29
line_count: 185
compliance_status: compliant

applicable_agents:
  - researcher
  - blogger
  - coder
  - primary
  - all

activation_trigger: |
  Load this skill when:
  - Need to search the web without API keys
  - Research requiring multiple sources
  - Finding documentation, tutorials, or articles
  - WebSearch MCP not available or limited

required_tools:
  - Bash

category: general
depends_on: []
related_skills:
  - jina-reader.md
  - article-extract.md
---

# Deep Search: Multi-Engine Web Search via DuckDuckGo

**Purpose**: Search the web using DuckDuckGo's multi-engine search. Free, no API key required. Great for research when WebSearch MCP is unavailable.

---

## Quick Start

### Installation (one time)

```bash
pip install ddgs
# Note: Old package name was 'duckduckgo-search' - use 'ddgs' now
```

### Basic Usage

```python
from ddgs import DDGS

results = list(DDGS().text('your search query', max_results=10))

for r in results:
    print(f"{r['title']}")
    print(f"  {r['href']}")
    print(f"  {r['body'][:100]}...")
    print()
```

---

## When to Use

**Use Deep Search when:**
- WebSearch MCP not available
- Need multiple search results programmatically
- Research requiring source URLs
- Finding documentation, tutorials, repos
- Building search into scripts

**Do NOT use when:**
- WebSearch MCP is working fine (use that instead)
- Need real-time/instant results (slight delay)
- Searching sensitive/personalized content
- Need Google-specific features

---

## Core Features

### 1. Text Search

```python
from ddgs import DDGS

# Basic search
results = list(DDGS().text('claude code tutorial', max_results=10))

# With region filter
results = list(DDGS().text('python best practices', region='us-en', max_results=5))
```

### 2. News Search

```python
from ddgs import DDGS

# Recent news
news = list(DDGS().news('anthropic claude', max_results=10))

for article in news:
    print(f"{article['title']}")
    print(f"  {article['url']}")
    print(f"  {article['date']}")
```

### 3. Image Search

```python
from ddgs import DDGS

# Find images
images = list(DDGS().images('claude ai logo', max_results=5))

for img in images:
    print(f"{img['title']}: {img['image']}")
```

---

## Examples

### Example 1: Basic Research Query

```python
from ddgs import DDGS

query = 'MCP model context protocol anthropic'
results = list(DDGS().text(query, max_results=10))

print(f"Found {len(results)} results for: {query}\n")

for i, r in enumerate(results, 1):
    print(f"{i}. {r['title']}")
    print(f"   URL: {r['href']}")
    print(f"   {r['body'][:150]}...")
    print()
```

### Example 2: Find GitHub Repos

```python
from ddgs import DDGS

# Search for GitHub repos
query = 'site:github.com claude code extensions'
results = list(DDGS().text(query, max_results=10))

repos = [r for r in results if 'github.com' in r['href']]
for repo in repos:
    print(f"Repo: {repo['href']}")
```

### Example 3: Complete Research Script

```python
#!/usr/bin/env python3
"""Deep search for research tasks."""

from ddgs import DDGS
from typing import List, Dict
import json

def deep_search(
    query: str,
    max_results: int = 10,
    search_type: str = 'text'
) -> List[Dict]:
    """
    Perform deep search using DuckDuckGo.

    Args:
        query: Search query
        max_results: Maximum results to return
        search_type: 'text', 'news', or 'images'

    Returns:
        List of result dictionaries
    """
    ddgs = DDGS()

    if search_type == 'text':
        results = list(ddgs.text(query, max_results=max_results))
    elif search_type == 'news':
        results = list(ddgs.news(query, max_results=max_results))
    elif search_type == 'images':
        results = list(ddgs.images(query, max_results=max_results))
    else:
        raise ValueError(f"Unknown search type: {search_type}")

    return results

def format_results(results: List[Dict], search_type: str = 'text') -> str:
    """Format results as markdown."""
    lines = []

    for i, r in enumerate(results, 1):
        if search_type == 'text':
            lines.append(f"### {i}. {r.get('title', 'No title')}")
            lines.append(f"**URL**: {r.get('href', 'N/A')}")
            lines.append(f"\n{r.get('body', 'No description')}\n")
        elif search_type == 'news':
            lines.append(f"### {i}. {r.get('title', 'No title')}")
            lines.append(f"**URL**: {r.get('url', 'N/A')}")
            lines.append(f"**Date**: {r.get('date', 'N/A')}")
            lines.append(f"\n{r.get('body', 'No description')}\n")

    return '\n'.join(lines)

# Usage
if __name__ == '__main__':
    results = deep_search('anthropic claude code', max_results=5)
    print(format_results(results))
```

### Example 4: Quick One-Liner

```python
# Quick search in Python REPL
from ddgs import DDGS
print('\n'.join(f"{r['title']}: {r['href']}" for r in DDGS().text('query', max_results=5)))
```

---

## Advanced Options

```python
from ddgs import DDGS

# All text search options
results = list(DDGS().text(
    keywords='search query',
    region='us-en',           # Region (wt-wt for no region)
    safesearch='moderate',    # off, moderate, strict
    timelimit='m',            # d=day, w=week, m=month, y=year
    max_results=20
))
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| ImportError | `pip install ddgs` |
| Empty results | Try different query, check internet |
| Rate limited | Add delays between searches |
| Timeout | Increase timeout or retry |

---

## Integration Tips

**For Research Agent:**
```python
# Search, then use jina-reader or article-extract on top results
results = list(DDGS().text(query, max_results=5))
urls = [r['href'] for r in results]
# Then scrape each URL with jina or trafilatura
```

**For Batch Research:**
```python
queries = ['topic 1', 'topic 2', 'topic 3']
all_results = {}
for q in queries:
    all_results[q] = list(DDGS().text(q, max_results=5))
    time.sleep(1)  # Be polite
```

---

## Success Indicators

You're using this skill correctly when:
- [ ] Getting multiple search results programmatically
- [ ] Finding relevant URLs for research
- [ ] Using appropriate search type (text/news/images)
- [ ] Respecting rate limits with delays

---

## Related

- `.claude/skills/jina-reader.md` - Convert found URLs to markdown
- `.claude/skills/article-extract.md` - Extract article content from URLs

---

**Remember: Free search without API keys. Perfect for research pipelines.**
