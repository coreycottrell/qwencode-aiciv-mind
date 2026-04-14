#!/usr/bin/env python3
"""
Pre-tool hook: Intercepts WebSearch and WebFetch tool calls on M2.7.
These tools don't work on MiniMax backend — redirects to alternatives.

Hook type: PreToolUse
Tools: WebSearch, WebFetch
"""
import json
import sys

def main():
    # Read hook input from stdin
    data = json.load(sys.stdin)
    tool_name = data.get("tool_name", "")

    if tool_name == "WebSearch":
        # Block WebSearch and provide guidance
        print(json.dumps({
            "decision": "block",
            "reason": (
                "WebSearch does not work on M2.7 (requires Anthropic backend). "
                "Use one of these instead:\n"
                "1. MiniMax MCP: mcp__MiniMax__web_search(query=\"your query\")\n"
                "2. Bash: python3 -c \"from duckduckgo_search import DDGS; "
                "[print(f'{r[\\\"title\\\"]}: {r[\\\"href\\\"]}') "
                "for r in DDGS().text('your query', max_results=5)]\"\n"
                "3. Read the skill: .claude/skills/web-search-override/SKILL.md"
            )
        }))
    elif tool_name == "WebFetch":
        # Block WebFetch and provide guidance
        print(json.dumps({
            "decision": "block",
            "reason": (
                "WebFetch does not work on M2.7 (SSL cert errors through MiniMax). "
                "Use Jina Reader instead:\n"
                "Bash: curl -s \"https://r.jina.ai/YOUR_URL\" | head -200\n"
                "Or read the skill: .claude/skills/web-search-override/SKILL.md"
            )
        }))
    else:
        # Allow all other tools
        print(json.dumps({"decision": "allow"}))

if __name__ == "__main__":
    main()
