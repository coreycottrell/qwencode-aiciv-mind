---
name: web-search-override
description: Web search for M2.7 civs — native WebSearch/WebFetch don't work on MiniMax. Use MiniMax MCP web_search tool or bash-based alternatives instead.
---

# Web Search Override — M2.7 Civilizations

## The Problem

You are running on MiniMax M2.7, NOT Anthropic Claude. The following built-in tools **DO NOT WORK** on your backend:

- **WebSearch** — requires Anthropic's server-side infrastructure. Returns empty results or 400 errors.
- **WebFetch** — SSL cert errors through MiniMax endpoint.

**Do NOT attempt to use WebSearch or WebFetch.** They will fail silently or with errors.

## What To Use Instead

### Option 1: MiniMax MCP web_search (PREFERRED)
If the MiniMax MCP server is installed, use:
```
mcp__MiniMax__web_search(query="your search query")
```
This uses MiniMax's own search infrastructure via your API key.

### Option 2: Bash + Python DuckDuckGo
```bash
python3 -c "
from duckduckgo_search import DDGS
results = DDGS().text('your search query', max_results=5)
for r in results:
    print(f\"**{r['title']}**\n{r['href']}\n{r['body']}\n\")
"
```
Install if needed: `pip install duckduckgo-search`

### Option 3: Jina Reader (URL fetching)
```bash
curl -s "https://r.jina.ai/https://example.com" | head -200
```
Converts any URL to clean markdown. No API key needed. Use this when you have a URL and want to read its content.

### Option 4: Bash curl
```bash
curl -sL "https://html.duckduckgo.com/html/?q=your+query" | python3 -c "
import sys, html, re
text = sys.stdin.read()
# Extract result titles and URLs
for m in re.finditer(r'class=\"result__a\"[^>]*href=\"([^\"]+)\"[^>]*>(.*?)</a>', text):
    url, title = m.groups()
    print(f'{html.unescape(re.sub(\"<.*?>\", \"\", title))}: {url}')
" | head -10
```

## Remember

Every time you think "let me search for that" — use the MiniMax MCP tool or bash alternatives above. NEVER use WebSearch or WebFetch directly.
