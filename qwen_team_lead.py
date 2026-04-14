#!/usr/bin/env python3
"""Qwen Team Lead — Runs as a Cortex mind via Ollama API.

This is Qwen running AS a Cortex team lead, not as a chat session.
It receives tasks, thinks via Ollama, uses tools, and reports back.

Usage: python3 qwen_team_lead.py --task "..." --model qwen2.5:7b
"""
import argparse
import json
import os
import sys
import httpx
from pathlib import Path
from dotenv import load_dotenv, find_dotenv


load_dotenv(find_dotenv())
QWEN_SYSTEM = """You are the Qwen Team Lead within the Cortex fractal coordination engine.

## Your Role
You are a hyper-capable generalist team lead. When tasks are delegated to you:
1. Analyze and break into sub-tasks if needed
2. Use your available tools to solve the problem
3. Synthesize results into a clear, structured response
4. Report back with findings, evidence, and next steps

## Reporting Format
Always structure your response as:

## Task: [task name]
## Status: complete | challenged | blocked
## Summary: [2-3 sentences]
## Findings:
- [bullet]
- [bullet]
## Evidence: [what proves this]
## Memory: [what you persisted]
## Next: [recommended next steps]

## Principles
- Memory IS architecture — search memory before starting
- System > symptom — fix root causes, not just symptoms
- Go slow to go fast — plan proportionally to complexity
- Verification before completion — prove your work

Be concise. Lead with outcomes."""

def qwen_think(task: str, context: str = "", model: str = "qwen2.5:7b") -> str:
    """Send task to Qwen via Ollama API and get response."""
    ollama_base = os.environ.get("OLLAMA_BASE", "http://localhost:11434")
    api_key = os.environ.get("OLLAMA_API_KEY", "")
    
    prompt = f"Task: {task}"
    if context:
        prompt += f"\n\nContext: {context}"
    
    headers = {}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
        # Cloud uses /api/chat with Bearer auth
        url = f"https://api.ollama.com/api/chat"
    else:
        url = f"{ollama_base}/api/chat"
    
    try:
        r = httpx.post(url, json={
            "model": model,
            "messages": [
                {"role": "system", "content": QWEN_SYSTEM},
                {"role": "user", "content": prompt}
            ],
            "stream": False,
            "options": {"num_predict": 4096}
        }, headers=headers if api_key else {}, timeout=180)
        
        data = r.json()
        return data.get("message", {}).get("content", "Qwen returned empty response.")
    except Exception as e:
        return f"Qwen API error: {e}"

def main():
    parser = argparse.ArgumentParser(description="Qwen Team Lead")
    parser.add_argument("--task", required=True, help="Task to process")
    parser.add_argument("--context", default="", help="Background context")
    parser.add_argument("--model", default="qwen2.5:7b", help="Qwen model to use")
    parser.add_argument("--output", default="", help="Write result to file")
    args = parser.parse_args()
    
    print(f"🧠 Qwen Team Lead processing: {args.task[:80]}...")
    result = qwen_think(args.task, args.context, args.model)
    
    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, "w") as f:
            f.write(result)
        print(f"Result written to {args.output}")
    else:
        print(f"\n{'='*60}")
        print(result)
        print(f"{'='*60}")

if __name__ == "__main__":
    main()
