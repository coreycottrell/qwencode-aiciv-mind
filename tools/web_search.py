#!/usr/bin/env python3
"""Web search tool for Cortex agents. Uses DuckDuckGo via the ddgs package."""

import json
import sys

def search(query: str, max_results: int = 8) -> list[dict]:
    """Search DuckDuckGo and return results."""
    from ddgs import DDGS
    results = list(DDGS().text(query, max_results=max_results))
    return results

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: web_search.py <query> [max_results]"}))
        sys.exit(1)

    query = sys.argv[1]
    max_results = int(sys.argv[2]) if len(sys.argv) > 2 else 8

    try:
        results = search(query, max_results)
        print(json.dumps(results, indent=2))
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
