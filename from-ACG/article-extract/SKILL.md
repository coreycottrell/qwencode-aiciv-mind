---
name: article-extract
description: Extract clean article text from URLs using trafilatura. Use when scraping articles, extracting content, reading web pages, news extraction, batch URL processing, or when jina-reader is blocked. No API keys required.
version: 1.0.0
author: skills-master
created: 2025-12-29
last_updated: 2025-12-29
line_count: 175
compliance_status: compliant

applicable_agents:
  - researcher
  - blogger
  - coder
  - primary
  - all

activation_trigger: |
  Load this skill when:
  - Need clean article text from any URL
  - Extracting content for LLM processing
  - Scraping without API keys
  - Jina Reader blocked or unavailable

required_tools:
  - Bash

category: general
depends_on: []
related_skills:
  - jina-reader.md
  - deep-search.md
---

# Article Extract: Clean Content Extraction with Trafilatura

**Purpose**: Extract clean article text from any URL using trafilatura. Used by HuggingFace and Microsoft Research. Free, no API key required.

---

## Quick Start

### Installation (one time)

```bash
pip install trafilatura
```

### Basic Usage

```python
import trafilatura

# Fetch and extract
downloaded = trafilatura.fetch_url('https://example.com/article')
text = trafilatura.extract(downloaded)
print(text)
```

**That's it. Clean text from any article.**

---

## When to Use

**Use Article Extract when:**
- Need clean article text without boilerplate
- Jina Reader is blocked or unavailable
- Processing multiple articles in batch
- Need metadata (author, date, etc.)
- Building content pipelines

**Do NOT use when:**
- Page is JavaScript-heavy (try jina-reader first)
- Need to preserve exact HTML formatting
- Need images or media content
- Page requires authentication

---

## Core Features

### 1. Basic Extraction

```python
import trafilatura

url = 'https://example.com/blog-post'
downloaded = trafilatura.fetch_url(url)
text = trafilatura.extract(downloaded)
```

### 2. With Metadata

```python
import trafilatura

downloaded = trafilatura.fetch_url(url)
result = trafilatura.extract(
    downloaded,
    include_comments=False,
    include_tables=True,
    output_format='json',  # Returns JSON with metadata
)
```

### 3. Multiple Output Formats

```python
import trafilatura

downloaded = trafilatura.fetch_url(url)

# Plain text (default)
text = trafilatura.extract(downloaded)

# Markdown
markdown = trafilatura.extract(downloaded, output_format='markdown')

# XML with structure
xml = trafilatura.extract(downloaded, output_format='xml')

# JSON with metadata
json_out = trafilatura.extract(downloaded, output_format='json')
```

---

## Examples

### Example 1: Simple Article Extraction

```python
import trafilatura

url = 'https://techcrunch.com/some-article'
downloaded = trafilatura.fetch_url(url)
text = trafilatura.extract(downloaded)

if text:
    print(f"Extracted {len(text)} characters")
    print(text[:500])
else:
    print("Could not extract content")
```

### Example 2: Get Article with Metadata

```python
import trafilatura
import json

url = 'https://example.com/article'
downloaded = trafilatura.fetch_url(url)

# Extract as JSON to get metadata
result = trafilatura.extract(
    downloaded,
    output_format='json',
    with_metadata=True
)

if result:
    data = json.loads(result)
    print(f"Title: {data.get('title')}")
    print(f"Author: {data.get('author')}")
    print(f"Date: {data.get('date')}")
    print(f"Content: {data.get('text')[:200]}...")
```

### Example 3: Batch Processing Multiple URLs

```python
import trafilatura
from concurrent.futures import ThreadPoolExecutor

urls = [
    'https://example.com/article1',
    'https://example.com/article2',
    'https://example.com/article3',
]

def extract_url(url):
    """Extract content from single URL."""
    try:
        downloaded = trafilatura.fetch_url(url)
        text = trafilatura.extract(downloaded)
        return {'url': url, 'text': text, 'success': True}
    except Exception as e:
        return {'url': url, 'error': str(e), 'success': False}

# Process in parallel
with ThreadPoolExecutor(max_workers=5) as executor:
    results = list(executor.map(extract_url, urls))

for r in results:
    status = 'OK' if r['success'] else 'FAILED'
    print(f"{status}: {r['url']}")
```

### Example 4: Complete Research Script

```python
#!/usr/bin/env python3
"""Extract article content for research."""

import trafilatura
import json
import sys

def extract_article(url: str, format: str = 'text') -> dict:
    """
    Extract article content from URL.

    Args:
        url: Article URL
        format: Output format ('text', 'markdown', 'json')

    Returns:
        Dictionary with content and metadata
    """
    downloaded = trafilatura.fetch_url(url)

    if not downloaded:
        return {'success': False, 'error': 'Failed to fetch URL'}

    if format == 'json':
        result = trafilatura.extract(
            downloaded,
            output_format='json',
            with_metadata=True
        )
        if result:
            return {'success': True, 'data': json.loads(result)}
    else:
        text = trafilatura.extract(
            downloaded,
            output_format=format
        )
        if text:
            return {'success': True, 'text': text}

    return {'success': False, 'error': 'No content extracted'}

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python script.py <url>")
        sys.exit(1)

    result = extract_article(sys.argv[1])
    if result['success']:
        print(result.get('text', json.dumps(result.get('data'), indent=2)))
    else:
        print(f"Error: {result['error']}")
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Empty result | Page may be JS-heavy, try jina-reader |
| ImportError | `pip install trafilatura` |
| Timeout | Check URL is accessible |
| No metadata | Not all pages have extractable metadata |

---

## Comparison: Trafilatura vs Jina Reader

| Feature | Trafilatura | Jina Reader |
|---------|-------------|-------------|
| Install | pip install | None (curl) |
| Speed | Faster | Network call |
| Batch | Better | One at a time |
| JS pages | Worse | Better |
| Metadata | Yes | Limited |
| Offline | After fetch | No |

**Recommendation**: Try jina-reader first for complex pages. Use trafilatura for batch processing or when jina is blocked.

---

## Success Indicators

You're using this skill correctly when:
- [ ] Getting clean text without boilerplate
- [ ] Handling extraction failures gracefully
- [ ] Using appropriate output format for task
- [ ] Processing batches efficiently

---

## Related

- `.claude/skills/jina-reader.md` - Alternative for JS-heavy pages
- `.claude/skills/deep-search.md` - Find articles to extract

---

**Remember: Used by HuggingFace and Microsoft Research. Production-quality extraction.**
