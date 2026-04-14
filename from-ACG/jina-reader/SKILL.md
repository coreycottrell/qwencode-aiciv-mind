---
name: jina-reader
description: Convert any URL to clean markdown using Jina API - no keys required. Use when scraping websites, reading web pages, extracting article content, URL to markdown, web content extraction, or when WebFetch fails on complex pages.
version: 1.0.0
author: skills-master
created: 2025-12-29
last_updated: 2025-12-29
line_count: 145
compliance_status: compliant

applicable_agents:
  - researcher
  - coder
  - blogger
  - primary
  - all

activation_trigger: |
  Load this skill when:
  - WebFetch struggles with complex pages (JS-heavy, paywalls)
  - Need LLM-friendly markdown from any URL
  - Scraping content without API keys
  - Searching the web without credentials

required_tools:
  - Bash

category: general
depends_on: []
related_skills:
  - article-extract.md
  - deep-search.md
---

# Jina Reader: Free URL-to-Markdown Conversion

**Purpose**: Convert any URL to clean, LLM-friendly markdown using Jina's free API. No API key required. Works when WebFetch fails on complex pages.

---

## Quick Start

```bash
# Read any URL as markdown
curl -s "https://r.jina.ai/https://example.com"

# Search the web
curl -s "https://s.jina.ai/your+search+query"
```

**That's it. No install. No API key. Just curl.**

---

## When to Use

**Use Jina Reader when:**
- WebFetch returns garbled or incomplete content
- Page is JavaScript-heavy (SPAs, React apps)
- Need clean markdown for LLM processing
- Scraping without authentication
- Quick web search without API setup

**Do NOT use when:**
- Simple static HTML pages (WebFetch works fine)
- Need to interact with the page (use browser automation)
- Need images or media (text only)
- Rate-limited (see limits below)

---

## Core Features

### 1. URL Reading (r.jina.ai)

Converts any webpage to clean markdown:

```bash
# Basic usage
curl -s "https://r.jina.ai/https://github.com/anthropics/claude-code"

# With headers for better results
curl -s -H "Accept: text/markdown" "https://r.jina.ai/https://news.ycombinator.com"
```

**Returns:**
- Clean markdown text
- Extracted main content (no nav, ads, footers)
- Preserved headings, lists, links

### 2. Web Search (s.jina.ai)

Search the web and get results:

```bash
# URL-encoded query
curl -s "https://s.jina.ai/claude+code+tutorial"

# Multi-word queries
curl -s "https://s.jina.ai/best+practices+for+LLM+prompting"
```

**Returns:**
- Search results in markdown
- Titles, URLs, snippets
- Multiple sources

---

## Examples

### Example 1: Reading a GitHub README

**Input:**
```bash
curl -s "https://r.jina.ai/https://github.com/anthropics/claude-code" | head -100
```

**Output:**
```markdown
# Claude Code

Claude Code is an agentic coding tool...
## Installation
...
```

### Example 2: Searching for Documentation

**Input:**
```bash
curl -s "https://s.jina.ai/anthropic+claude+api+documentation"
```

**Output:**
```markdown
## Search Results

1. **Anthropic API Documentation**
   https://docs.anthropic.com
   Official documentation for Claude API...

2. **Getting Started with Claude**
   ...
```

### Example 3: Reading News Articles

**Input:**
```bash
curl -s "https://r.jina.ai/https://techcrunch.com/some-article"
```

**Why This Works:** Jina strips ads, navigation, and extracts main article content.

---

## Python Integration

For programmatic use:

```python
import subprocess

def jina_read(url: str) -> str:
    """Read URL as markdown via Jina."""
    result = subprocess.run(
        ['curl', '-s', f'https://r.jina.ai/{url}'],
        capture_output=True,
        text=True
    )
    return result.stdout

def jina_search(query: str) -> str:
    """Search web via Jina."""
    encoded = query.replace(' ', '+')
    result = subprocess.run(
        ['curl', '-s', f'https://s.jina.ai/{encoded}'],
        capture_output=True,
        text=True
    )
    return result.stdout

# Usage
content = jina_read('https://example.com')
results = jina_search('python best practices')
```

---

## Rate Limits

**Free tier:**
- No hard limits documented
- Reasonable use expected
- Add delays for bulk scraping

**Best practices:**
- Cache results when possible
- Don't hammer the API in tight loops
- Use for single-page reads, not crawling

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Empty response | Check URL is valid, try with headers |
| Garbled text | Page may be heavily JS - still better than WebFetch |
| Timeout | Large pages take longer, increase curl timeout |
| 403/blocked | Some sites block Jina, try article-extract skill |

---

## Success Indicators

You're using this skill correctly when:
- [ ] Getting clean markdown from complex pages
- [ ] WebFetch struggles resolved
- [ ] No API keys needed for basic scraping
- [ ] Search queries returning useful results

---

## Related

- `.claude/skills/article-extract.md` - Alternative extraction with trafilatura
- `.claude/skills/deep-search.md` - Multi-engine search via DuckDuckGo

---

**Remember: When WebFetch fails, Jina often succeeds. Zero setup, just curl.**
