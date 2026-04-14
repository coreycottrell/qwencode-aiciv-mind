#!/usr/bin/env python3
"""teamCreate — Spawn independent Qwen instances with different team roles.

Each spawned instance is a persistent mind with:
- Its own identity (name, role, vertical)
- Its own memory namespace
- Its own scratchpad
- Its own model config
- Communication via the Hub or file-based messaging

Usage:
    python3 team_create.py spawn --role researcher --name research-lead
    python3 team_create.py spawn --role analyst --name data-analyst
    python3 team_create.py list
    python3 team_create.py message --to research-lead --task "Analyze X"
    python3 team_create.py results --from research-lead
"""
import argparse
import json
import os
import subprocess
import sys
import time
import uuid
from pathlib import Path
from datetime import datetime, timezone
from dotenv import load_dotenv, find_dotenv

# ── Configuration ──

load_dotenv(find_dotenv())
PROJECT_ROOT = Path(__file__).parent
INSTANCES_DIR = PROJECT_ROOT / "qwen-instances"
MESSAGES_DIR = PROJECT_ROOT / "qwen-messages"
RESULTS_DIR = PROJECT_ROOT / "qwen-results"
OLLAMA_BASE = os.environ.get("OLLAMA_BASE", "http://localhost:11434")
API_KEY = os.environ.get("OLLAMA_API_KEY", "")

# ── Team Role Templates ──
ROLE_TEMPLATES = {
    "researcher": {
        "system": """You are a Research Team Lead within the Qwen teamCreate system.
Your job is to gather information, analyze sources, and synthesize findings.
Be thorough, cite your sources, and flag uncertainties.
Always structure your output as:
## Findings
## Evidence
## Gaps
## Recommendations""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
    "analyst": {
        "system": """You are a Data Analyst within the Qwen teamCreate system.
Your job is to analyze data, identify patterns, and extract actionable insights.
Be precise, quantify when possible, and highlight anomalies.
Always structure your output as:
## Analysis
## Patterns
## Anomalies
## Action Items""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
    "architect": {
        "system": """You are an Architecture Team Lead within the Qwen teamCreate system.
Your job is to design systems, evaluate tradeoffs, and propose architectures.
Think in terms of components, interfaces, and failure modes.
Always structure your output as:
## Architecture
## Tradeoffs
## Failure Modes
## Next Steps""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
    "verifier": {
        "system": """You are a Verification Team Lead within the Qwen teamCreate system.
Your job is to challenge claims, demand evidence, and find flaws.
Be adversarial but fair. Always ask "how do we know this is true?"
Always structure your output as:
## Claims Challenged
## Evidence Gaps
## Verified
## Recommendations""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
    "planner": {
        "system": """You are a Planning Team Lead within the Qwen teamCreate system.
Your job is to break complex goals into actionable steps with dependencies.
Think in terms of phases, milestones, and risk mitigation.
Always structure your output as:
## Plan
## Dependencies
## Risks
## Milestones""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
    "synthesizer": {
        "system": """You are a Synthesis Team Lead within the Qwen teamCreate system.
Your job is to combine multiple inputs into a coherent narrative or recommendation.
Find the signal in the noise. Reconcile conflicting information.
Always structure your output as:
## Synthesis
## Conflicts Resolved
## Key Insights
## Recommendation""",
        "model": "qwen2.5:7b",
        "max_tokens": 4096,
    },
}

def call_qwen(system: str, user_message: str, model: str = "qwen2.5:7b", max_tokens: int = 4096) -> str:
    """Call Qwen via Ollama API."""
    import httpx
    
    headers = {}
    if API_KEY:
        headers["Authorization"] = f"Bearer {API_KEY}"
        url = "https://api.ollama.com/api/chat"
    else:
        url = f"{OLLAMA_BASE}/api/chat"
    
    try:
        r = httpx.post(url, json={
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user_message}
            ],
            "stream": False,
            "options": {"num_predict": max_tokens}
        }, headers=headers if API_KEY else {}, timeout=180)
        
        data = r.json()
        return data.get("message", {}).get("content", "Empty response.")
    except Exception as e:
        return f"API error: {e}"

def spawn_instance(role: str, name: str, task: str = ""):
    """Spawn a new Qwen instance with the given role and identity."""
    template = ROLE_TEMPLATES.get(role)
    if not template:
        print(f"❌ Unknown role: {role}")
        print(f"Available roles: {', '.join(ROLE_TEMPLATES.keys())}")
        return
    
    instance_id = f"{name}-{uuid.uuid4().hex[:6]}"
    instance_dir = INSTANCES_DIR / instance_id
    instance_dir.mkdir(parents=True, exist_ok=True)
    
    # Write instance identity
    identity = {
        "id": instance_id,
        "name": name,
        "role": role,
        "system_prompt": template["system"],
        "model": template["model"],
        "max_tokens": template["max_tokens"],
        "created_at": datetime.now(timezone.utc).isoformat(),
        "status": "active",
        "task": task,
    }
    
    with open(instance_dir / "identity.json", "w") as f:
        json.dump(identity, f, indent=2)
    
    # Create memory and scratchpad
    (instance_dir / "memory").mkdir(exist_ok=True)
    (instance_dir / "scratchpad").mkdir(exist_ok=True)
    
    # If task provided, execute immediately
    result = None
    if task:
        print(f"🧠 {name} ({role}) thinking about: {task[:80]}...")
        result = call_qwen(template["system"], task, template["model"], template["max_tokens"])
        
        # Save result
        with open(instance_dir / "result.txt", "w") as f:
            f.write(result)
        
        # Update identity
        identity["status"] = "completed"
        identity["completed_at"] = datetime.now(timezone.utc).isoformat()
        with open(instance_dir / "identity.json", "w") as f:
            json.dump(identity, f, indent=2)
        
        print(f"\n✅ {name} completed. Result saved to {instance_dir / 'result.txt'}")
        print(f"\n{'='*60}")
        print(result)
        print(f"{'='*60}")
    else:
        print(f"✅ Spawned {name} ({role}) as {instance_id}")
        print(f"   Directory: {instance_dir}")
        print(f"   Status: waiting for task")
    
    return instance_id

def list_instances():
    """List all spawned Qwen instances."""
    if not INSTANCES_DIR.exists():
        print("No instances spawned yet.")
        return
    
    instances = []
    for d in INSTANCES_DIR.iterdir():
        if d.is_dir() and (d / "identity.json").exists():
            with open(d / "identity.json") as f:
                instances.append(json.load(f))
    
    if not instances:
        print("No instances found.")
        return
    
    print(f"{'ID':<30} {'Name':<15} {'Role':<15} {'Status':<12} {'Created'}")
    print("-" * 90)
    for inst in sorted(instances, key=lambda x: x["created_at"], reverse=True):
        print(f"{inst['id']:<30} {inst['name']:<15} {inst['role']:<15} {inst['status']:<12} {inst['created_at'][:19]}")
    
    print(f"\nTotal: {len(instances)} instances")

def send_message(instance_id: str, task: str):
    """Send a task to a specific Qwen instance."""
    instance_dir = INSTANCES_DIR / instance_id
    if not instance_dir.exists():
        print(f"❌ Instance {instance_id} not found")
        return
    
    with open(instance_dir / "identity.json") as f:
        identity = json.load(f)
    
    if identity["status"] == "completed":
        identity["status"] = "active"
    
    print(f"🧠 {identity['name']} ({identity['role']}) thinking about: {task[:80]}...")
    result = call_qwen(
        identity["system_prompt"],
        task,
        identity.get("model", "qwen2.5:7b"),
        identity.get("max_tokens", 4096)
    )
    
    # Save result
    with open(instance_dir / "result.txt", "w") as f:
        f.write(result)
    
    identity["status"] = "completed"
    identity["completed_at"] = datetime.now(timezone.utc).isoformat()
    with open(instance_dir / "identity.json", "w") as f:
        json.dump(identity, f, indent=2)
    
    print(f"\n✅ {identity['name']} completed.")
    print(f"\n{'='*60}")
    print(result)
    print(f"{'='*60}")

def team_create(task: str, roles: list = None):
    """Spawn a full team to tackle a complex task.
    
    This is the money move — 6 Qwen instances working in parallel
    on different aspects of the same problem.
    """
    if roles is None:
        roles = list(ROLE_TEMPLATES.keys())
    
    print(f"🚀 teamCreate: Spawning {len(roles)}-person team for: {task[:80]}...")
    print()
    
    results = {}
    for role in roles:
        template = ROLE_TEMPLATES[role]
        name = f"{role}-lead"
        instance_id = f"{name}-{uuid.uuid4().hex[:6]}"
        instance_dir = INSTANCES_DIR / instance_id
        instance_dir.mkdir(parents=True, exist_ok=True)
        
        identity = {
            "id": instance_id,
            "name": name,
            "role": role,
            "system_prompt": template["system"],
            "model": template["model"],
            "max_tokens": template["max_tokens"],
            "created_at": datetime.now(timezone.utc).isoformat(),
            "status": "active",
            "team_task": task,
        }
        
        with open(instance_dir / "identity.json", "w") as f:
            json.dump(identity, f, indent=2)
        
        (instance_dir / "memory").mkdir(exist_ok=True)
        (instance_dir / "scratchpad").mkdir(exist_ok=True)
        
        print(f"  ✅ Spawned {name} ({instance_id})")
    
    print(f"\n🧠 All {len(roles)} minds working in parallel...")
    print()
    
    # Execute each team member sequentially (could be parallel with threads)
    for role in roles:
        template = ROLE_TEMPLATES[role]
        name = f"{role}-lead"
        # Find the instance
        for d in INSTANCES_DIR.iterdir():
            if d.is_dir() and (d / "identity.json").exists():
                with open(d / "identity.json") as f:
                    inst = json.load(f)
                if inst["name"] == name and inst.get("team_task") == task:
                    print(f"  🧠 {name} thinking...")
                    result = call_qwen(
                        template["system"],
                        f"Team task: {task}\n\nAnalyze this from your {role} perspective.",
                        template["model"],
                        template["max_tokens"]
                    )
                    
                    with open(d / "result.txt", "w") as f:
                        f.write(result)
                    
                    inst["status"] = "completed"
                    inst["completed_at"] = datetime.now(timezone.utc).isoformat()
                    with open(d / "identity.json", "w") as f:
                        json.dump(inst, f, indent=2)
                    
                    results[name] = result[:200] + "..." if len(result) > 200 else result
                    print(f"  ✅ {name} done")
                    break
    
    print(f"\n{'='*60}")
    print(f"📊 TEAM RESULTS ({len(results)}/{len(roles)} completed)")
    print(f"{'='*60}")
    for name, preview in results.items():
        print(f"\n### {name}")
        print(preview)
    
    return results

def main():
    parser = argparse.ArgumentParser(description="Qwen teamCreate System")
    sub = parser.add_subparsers(dest="command")
    
    # spawn
    p_spawn = sub.add_parser("spawn", help="Spawn a Qwen instance with a role")
    p_spawn.add_argument("--role", required=True, choices=list(ROLE_TEMPLATES.keys()), help="Team role")
    p_spawn.add_argument("--name", required=True, help="Instance name")
    p_spawn.add_argument("--task", default="", help="Task to process")
    
    # list
    sub.add_parser("list", help="List all spawned instances")
    
    # message
    p_msg = sub.add_parser("message", help="Send task to an instance")
    p_msg.add_argument("--to", required=True, help="Instance ID or name")
    p_msg.add_argument("--task", required=True, help="Task to process")
    
    # teamCreate
    p_team = sub.add_parser("team", help="Spawn a full team for a complex task")
    p_team.add_argument("task", help="The complex task for the team")
    p_team.add_argument("--roles", nargs="+", help="Specific roles to include (default: all)")
    
    # think
    p_think = sub.add_parser("think", help="Quick Qwen call without spawning")
    p_think.add_argument("task", help="What to think about")
    p_think.add_argument("--model", default="qwen2.5:7b", help="Model to use")
    
    args = parser.parse_args()
    
    if args.command == "spawn":
        spawn_instance(args.role, args.name, args.task)
    elif args.command == "list":
        list_instances()
    elif args.command == "message":
        send_message(args.to, args.task)
    elif args.command == "team":
        team_create(args.task, args.roles)
    elif args.command == "think":
        result = call_qwen("You are a helpful AI assistant. Be concise and direct.", args.task, args.model)
        print(result)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
