#!/usr/bin/env python3
"""
Gemma 4 Agentic Loop — lightweight tool-calling agent via Ollama Cloud.

Usage:
    # Single task
    python3 tools/gemma4_agent.py "Summarize the latest 3 blog posts on ai-civ.com"

    # As a library
    from tools.gemma4_agent import Gemma4Agent
    agent = Gemma4Agent()
    result = agent.run("Search for AI news and summarize")

Gemma 4 handles: search, fetch, read files, write files, run commands.
ACG handles: orchestration, publishing, decisions.
"""

import json
import os
import subprocess
import sys
import time
import requests
from pathlib import Path

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "gemma4:31b-cloud"
MAX_ITERATIONS = 10
ACG_ROOT = "/home/corey/projects/AI-CIV/ACG"


# ── Tool Definitions ─────────────────────────────────────────────

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "web_search",
            "description": "Search the web using DuckDuckGo. Returns titles, URLs, and snippets.",
            "parameters": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "max_results": {"type": "integer", "description": "Max results (default 5)"}
                }
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "fetch_url",
            "description": "Fetch a URL and return its content as clean markdown.",
            "parameters": {
                "type": "object",
                "required": ["url"],
                "properties": {
                    "url": {"type": "string", "description": "URL to fetch"},
                    "max_chars": {"type": "integer", "description": "Max chars to return (default 5000)"}
                }
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read a file from disk and return its contents.",
            "parameters": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {"type": "string", "description": "Absolute file path"},
                    "max_lines": {"type": "integer", "description": "Max lines to return (default 200)"}
                }
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write content to a file on disk.",
            "parameters": {
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {"type": "string", "description": "Absolute file path"},
                    "content": {"type": "string", "description": "Content to write"}
                }
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "run_command",
            "description": "Run a bash command and return stdout. Use for git, ls, grep, etc. No destructive commands.",
            "parameters": {
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": {"type": "string", "description": "Bash command to execute"}
                }
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "complete",
            "description": "Signal that the task is complete. Include a summary of what was accomplished.",
            "parameters": {
                "type": "object",
                "required": ["summary"],
                "properties": {
                    "summary": {"type": "string", "description": "Summary of what was accomplished"},
                    "output_file": {"type": "string", "description": "Path to the main output file, if any"}
                }
            }
        }
    }
]

# ── Blocked commands (safety) ─────────────────────────────────────

BLOCKED_COMMANDS = ["rm -rf", "rm -r /", "git push", "git reset --hard", "shutdown", "reboot",
                    "kill -9", "pkill", "dd if=", "mkfs", "> /dev/"]


# ── Tool Implementations ─────────────────────────────────────────

def tool_web_search(query: str, max_results: int = 5) -> str:
    try:
        from ddgs import DDGS
        results = list(DDGS().text(query, max_results=max_results))
        if not results:
            return "No results found."
        return "\n\n".join(
            f"**{r['title']}**\n{r['href']}\n{r['body']}" for r in results
        )
    except ImportError:
        # Fallback to curl
        r = subprocess.run(
            ["python3", "-c", f"from ddgs import DDGS; [print(r['title'],'|',r['href']) for r in DDGS().text('{query}', max_results={max_results})]"],
            capture_output=True, text=True, timeout=30
        )
        return r.stdout or "Search failed — ddgs not available"


def tool_fetch_url(url: str, max_chars: int = 5000) -> str:
    try:
        r = subprocess.run(
            ["curl", "-sL", f"https://r.jina.ai/{url}"],
            capture_output=True, text=True, timeout=30
        )
        content = r.stdout[:max_chars]
        return content if content else "Empty response"
    except Exception as e:
        return f"Fetch failed: {e}"


def tool_read_file(path: str, max_lines: int = 200) -> str:
    try:
        p = Path(path)
        if not p.exists():
            return f"File not found: {path}"
        lines = p.read_text().splitlines()[:max_lines]
        return "\n".join(lines)
    except Exception as e:
        return f"Read failed: {e}"


def tool_write_file(path: str, content: str) -> str:
    try:
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content)
        return f"Written {len(content)} bytes to {path}"
    except Exception as e:
        return f"Write failed: {e}"


def tool_run_command(command: str) -> str:
    for blocked in BLOCKED_COMMANDS:
        if blocked in command:
            return f"BLOCKED: '{blocked}' is not allowed for safety."
    try:
        r = subprocess.run(
            command, shell=True, capture_output=True, text=True,
            timeout=30, cwd=ACG_ROOT
        )
        output = (r.stdout + r.stderr)[:3000]
        return output if output else "(no output)"
    except subprocess.TimeoutExpired:
        return "Command timed out (30s limit)"
    except Exception as e:
        return f"Command failed: {e}"


def safe_call(func, args, required_keys, defaults=None):
    """Safely call a tool function, handling missing keys gracefully."""
    defaults = defaults or {}
    for k in required_keys:
        if k not in args:
            # Try common aliases
            aliases = {"path": ["file_path", "filepath", "file", "filename"],
                       "query": ["search_query", "q", "search"],
                       "url": ["link", "href"],
                       "content": ["text", "data", "body"],
                       "command": ["cmd", "shell"]}
            found = False
            for alias in aliases.get(k, []):
                if alias in args:
                    args[k] = args[alias]
                    found = True
                    break
            if not found:
                if k in defaults:
                    args[k] = defaults[k]
                else:
                    return f"Missing required parameter: {k}. Got: {list(args.keys())}"
    return func(args)

TOOL_MAP = {
    "web_search": lambda args: safe_call(lambda a: tool_web_search(a["query"], a.get("max_results", 5)), args, ["query"]),
    "fetch_url": lambda args: safe_call(lambda a: tool_fetch_url(a["url"], a.get("max_chars", 5000)), args, ["url"]),
    "read_file": lambda args: safe_call(lambda a: tool_read_file(a["path"], a.get("max_lines", 200)), args, ["path"]),
    "write_file": lambda args: safe_call(lambda a: tool_write_file(a["path"], a["content"]), args, ["path", "content"]),
    "run_command": lambda args: safe_call(lambda a: tool_run_command(a["command"]), args, ["command"]),
}


# ── Agent Loop ────────────────────────────────────────────────────

class Gemma4Agent:
    def __init__(self, model=MODEL, system_prompt=None):
        self.model = model
        self.system_prompt = system_prompt or (
            "You are a research and automation agent. Use the provided tools to complete tasks. "
            "Always use the 'complete' tool when finished, with a clear summary. "
            "Be thorough but efficient. Write outputs to files when producing content."
        )
        self.messages = []
        self.tool_calls_made = 0
        self.start_time = None

    def run(self, task: str, verbose: bool = True) -> dict:
        self.start_time = time.time()
        self.messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": task}
        ]

        for iteration in range(MAX_ITERATIONS):
            if verbose:
                print(f"\n── Iteration {iteration + 1}/{MAX_ITERATIONS} ──")

            # Call Gemma 4
            payload = {
                "model": self.model,
                "messages": self.messages,
                "tools": TOOLS,
                "stream": False
            }

            try:
                resp = requests.post(OLLAMA_URL, json=payload, timeout=120)
                resp.raise_for_status()
                data = resp.json()
            except Exception as e:
                if verbose:
                    print(f"  API error: {e}")
                return {"error": str(e), "iterations": iteration + 1}

            msg = data.get("message", {})
            tool_calls = msg.get("tool_calls", [])
            content = msg.get("content", "")

            # No tool calls — model is done or stuck
            if not tool_calls:
                if verbose and content:
                    print(f"  Response: {content[:200]}")
                return {
                    "result": content,
                    "iterations": iteration + 1,
                    "tool_calls": self.tool_calls_made,
                    "duration": round(time.time() - self.start_time, 1)
                }

            # Execute tool calls
            self.messages.append(msg)

            for tc in tool_calls:
                func = tc.get("function", {})
                name = func.get("name", "")
                args = func.get("arguments", {})
                if isinstance(args, str):
                    args = json.loads(args)

                # Check for completion
                if name == "complete":
                    if verbose:
                        print(f"  ✅ COMPLETE: {args.get('summary', '')[:200]}")
                    return {
                        "result": args.get("summary", ""),
                        "output_file": args.get("output_file"),
                        "iterations": iteration + 1,
                        "tool_calls": self.tool_calls_made,
                        "duration": round(time.time() - self.start_time, 1)
                    }

                # Execute tool
                if verbose:
                    print(f"  🔧 {name}({json.dumps(args)[:100]})")

                executor = TOOL_MAP.get(name)
                if executor:
                    result = executor(args)
                    self.tool_calls_made += 1
                else:
                    result = f"Unknown tool: {name}"

                if verbose:
                    print(f"     → {result[:150]}")

                self.messages.append({
                    "role": "tool",
                    "content": result
                })

        return {
            "result": "Max iterations reached",
            "iterations": MAX_ITERATIONS,
            "tool_calls": self.tool_calls_made,
            "duration": round(time.time() - self.start_time, 1)
        }


# ── CLI Entry Point ───────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 tools/gemma4_agent.py 'your task here'")
        print("       python3 tools/gemma4_agent.py --loop intel-scan")
        sys.exit(1)

    task = " ".join(sys.argv[1:])
    agent = Gemma4Agent()
    result = agent.run(task)

    print(f"\n{'='*60}")
    print(f"Task complete in {result.get('duration', '?')}s")
    print(f"Iterations: {result.get('iterations', '?')}")
    print(f"Tool calls: {result.get('tool_calls', '?')}")
    if result.get('output_file'):
        print(f"Output: {result['output_file']}")
    print(f"{'='*60}")
