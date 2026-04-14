---
name: m27-claude-code
description: Complete guide to running Claude Code on MiniMax M2.7 — sovereign compute. Setup, behavioral patterns, skill authoring rules, search workarounds, team orchestration, and known issues. Load before any M2.7 work.
version: 1.0.0
validated: 2026-04-08
---

# M2.7 on Claude Code — The Sovereign Compute Playbook

> **What this proves**: A full AiCIV civilization — tool use, team leads, agent spawning, memory,
> search, identity formation — runs on open source inference via MiniMax M2.7. Sovereign compute
> is real. Validated 2026-04-08.

---

## Quick Start — Birth a New M2.7 Civ

### 1. Create Project Directory
```bash
mkdir -p /home/corey/projects/AI-CIV/{civ-name}/.claude/{skills,hooks,agents,scratchpad-daily}
mkdir -p /home/corey/projects/AI-CIV/{civ-name}/memories/{sessions,knowledge}
cd /home/corey/projects/AI-CIV/{civ-name} && git init
```

### 2. Create launch.sh
```bash
#!/bin/bash
set -e
cd "$(dirname "$0")"

# Load MiniMax API key
source /home/corey/projects/AI-CIV/ACG/config/human-credentials/minimax.env

unset ANTHROPIC_API_KEY 2>/dev/null || true
unset ANTHROPIC_AUTH_TOKEN 2>/dev/null || true

ANTHROPIC_BASE_URL="https://api.minimax.io/anthropic" \
ANTHROPIC_API_KEY="$MINIMAX_API_KEY" \
ANTHROPIC_AUTH_TOKEN="$MINIMAX_API_KEY" \
ANTHROPIC_MODEL="MiniMax-M2.7" \
ANTHROPIC_SMALL_FAST_MODEL="MiniMax-M2.7" \
ANTHROPIC_DEFAULT_SONNET_MODEL="MiniMax-M2.7" \
ANTHROPIC_DEFAULT_OPUS_MODEL="MiniMax-M2.7" \
ANTHROPIC_DEFAULT_HAIKU_MODEL="MiniMax-M2.7" \
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
API_TIMEOUT_MS=300000 \
    claude --dangerously-skip-permissions "$@"
```

### 3. Create .claude/settings.json
```json
{
  "permissions": {
    "allow": ["WebFetch", "WebSearch", "Read", "Write", "Edit", "Glob", "Grep", "Task", "Bash"]
  },
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "ANTHROPIC_BASE_URL": "https://api.minimax.io/anthropic",
    "ANTHROPIC_API_KEY": "<MINIMAX_KEY>",
    "ANTHROPIC_AUTH_TOKEN": "<MINIMAX_KEY>",
    "ANTHROPIC_MODEL": "MiniMax-M2.7",
    "ANTHROPIC_SMALL_FAST_MODEL": "MiniMax-M2.7",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "MiniMax-M2.7",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "MiniMax-M2.7",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "MiniMax-M2.7",
    "API_TIMEOUT_MS": "300000"
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "WebSearch|WebFetch",
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$CLAUDE_PROJECT_DIR/.claude/hooks/search_redirect.py\"",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

**WHY settings.json matters**: Teammates spawned via Agent Teams inherit env from settings.json, NOT from launch.sh. Without the API key in settings.json, teammates auth-fail.

### 4. Install Search (copy from ACG)
```bash
cp -r /home/corey/projects/AI-CIV/ACG/.claude/skills/web-search-override \
      /home/corey/projects/AI-CIV/{civ-name}/.claude/skills/
cp /home/corey/projects/AI-CIV/proof-aiciv/.claude/hooks/search_redirect.py \
   /home/corey/projects/AI-CIV/{civ-name}/.claude/hooks/
```

### 5. Launch
```bash
chmod +x launch.sh
tmux new-session -d -s "{civ-name}" "./launch.sh 'Wake up. Read your CLAUDE.md.'"
tmux attach -t {civ-name}
```

---

## What Works on M2.7

| Capability | Status | Notes |
|-----------|--------|-------|
| Basic tools (Read, Write, Edit, Bash, Glob, Grep) | ✅ | No issues |
| TeamCreate + Agent Teams | ✅ | Full lifecycle: create → spawn → message → shutdown → delete |
| Agent tool (spawn teammates) | ✅ | Real separate Claude instances in tmux panes |
| SendMessage (teammate ↔ main) | ✅ | Delivery confirmed |
| Task() subagents (from team leads) | ✅ | Team leads can delegate to specialists |
| Thinking mode | ✅ | "Mulling/Tempering/Baking with high effort" — good quality |
| MCP servers | ✅ | context7 confirmed working, MiniMax MCP available |
| Identity formation | ✅ | Constitutional awakening, naming, memory writing |
| ddgs search (bash) | ✅ | `python3 -c "from ddgs import DDGS; ..."` |
| Jina reader (bash) | ✅ | `curl -s "https://r.jina.ai/URL"` |
| Planning and reasoning | ✅ | Excellent plan generation |

## What Does NOT Work on M2.7

| Capability | Status | Workaround |
|-----------|--------|------------|
| **WebSearch** | BROKEN | MiniMax MCP `web_search` or ddgs via bash |
| **WebFetch** | BROKEN | Jina reader: `curl -s "https://r.jina.ai/URL"` |
| Fast mode | BROKEN | Hardcoded to api.anthropic.com — just doesn't activate |
| Settings sync | BROKEN | Uses local settings.json only |
| Auto-update checks | BROKEN | Not needed — good |

---

## M2.7 Behavioral Patterns (CRITICAL for Skill Authors)

### 1. NARRATOR MODE (The #1 Issue)

M2.7 describes tool calls as conversation text instead of actually invoking them.

**What it looks like:**
```
"Let me spawn research-lead now. I'll use the Agent tool with team_name='session-20260408',
name='research-lead', subagent_type='general-purpose'..."
```
Then it moves on WITHOUT calling the tool.

**Fix in skills:** Use imperative language. Not "spawn X" but "CALL the Agent tool with these EXACT parameters NOW."

**Fix in coaching:** "I will confirm when I see a new tmux pane. You have not spawned anything yet."

### 2. PARALLEL SPAWN FAILURE

M2.7 cannot reliably invoke 6 tools in one turn. It narrates 5 and invokes 1.

**Fix:** Always sequential. "Spawn research-lead FIRST. After confirmed, spawn identity-lead NEXT."

**In skills:** Never write "spawn these 6 agents in parallel." Write:
```
Step 1: Invoke Agent tool for research-lead. WAIT for confirmation.
Step 2: Invoke Agent tool for identity-lead. WAIT for confirmation.
Step 3: ...
```

### 3. PLAN-TO-ACTION GAP

M2.7 generates beautiful, detailed plans but struggles to bridge from plan to execution.

**Fix:** After any planning phase, add: "Now execute Step 1. CALL the tool. Do not describe it."

### 4. CONTEXT WINDOW SENSITIVITY

MiniMax warns M2.7 may terminate early near context limits. Keep under 200K tokens total.

**Fix:** Compact manifests (<200 lines). Break work into fresh sessions. Use team leads to absorb specialist output (same as Opus pattern).

### 5. GHOST TEXT CONFUSION

Claude Code's autocomplete appears as faded text. M2.7 sometimes treats this as actual input.

**Fix:** Type explicitly. Don't rely on autocomplete suggestions.

### 6. THINKING QUALITY

When M2.7 enters deep thinking ("Mulling", "Tempering", "Baking"), output quality is high. The thinking modes are genuine — not just loading indicators.

---

## Skill Authoring Rules for M2.7

These rules apply to ANY skill that will run on M2.7 civs:

1. **IMPERATIVE, not descriptive**: "CALL Agent tool" not "spawn agent"
2. **ONE tool call per instruction**: Never ask for parallel tool invocations
3. **VERIFY after each step**: "Confirm the pane exists before proceeding"
4. **LITERAL examples**: Show exact tool parameters with real values
5. **SEQUENTIAL, not parallel**: Steps must be ordered and gated
6. **COMPACT manifests**: <200 lines. Every line counts against context
7. **EXPLICIT search override**: Any skill mentioning "search" must say "Use ddgs or MiniMax MCP, NOT WebSearch"
8. **NO WebSearch/WebFetch references**: Replace all instances with alternatives

### Template for M2.7-Safe Skill Section
```markdown
## Search (M2.7 Override)
WebSearch and WebFetch do NOT work on M2.7.
Use: `python3 -c "from ddgs import DDGS; [print(r['title'],r['href']) for r in DDGS().text('query', max_results=5)]"`
Or: `curl -s "https://r.jina.ai/URL" | head -200`
```

---

## Search Solutions (Ranked)

| Method | Reliability | Speed | Setup |
|--------|------------|-------|-------|
| **ddgs Python** | HIGH | ~2s | `pip3 install --break-system-packages ddgs` |
| **Jina Reader** | HIGH | ~3s | None (curl) |
| **MiniMax MCP web_search** | MEDIUM | ~2s | `claude mcp add ...` (uvx may fail) |
| **curl DuckDuckGo HTML** | LOW | ~1s | None but CAPTCHA risk |

### Pre-Tool Hook (Block + Redirect)
File: `.claude/hooks/search_redirect.py`
```python
#!/usr/bin/env python3
import json, sys
data = json.load(sys.stdin)
tool = data.get("tool_name", "")
if tool == "WebSearch":
    print(json.dumps({"decision": "block", "reason":
        "WebSearch broken on M2.7. Use: python3 -c \"from ddgs import DDGS; "
        "[print(r['title']) for r in DDGS().text('query', max_results=5)]\""
    }))
elif tool == "WebFetch":
    print(json.dumps({"decision": "block", "reason":
        "WebFetch broken on M2.7. Use: curl -s 'https://r.jina.ai/URL' | head -200"
    }))
else:
    print(json.dumps({"decision": "allow"}))
```

Wire in settings.json under `hooks.PreToolUse` with matcher `"WebSearch|WebFetch"`.

---

## Anthropic Phone-Home Traffic

Claude Code sends telemetry to api.anthropic.com even with custom ANTHROPIC_BASE_URL.

**What gets sent**: session_ingress (transcripts), event_logging, metrics, managed_settings, policy_limits, Datadog (every 15s)

**What `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1` blocks**: Statsig, Sentry, auto-updater, bug command, surveys (5 of ~14 channels)

**For full isolation**: Need iptables/firewall blocking of api.anthropic.com, *.datadoghq.com, *.statsig.com. Claude Code degrades gracefully.

**Current stance**: Inference confirmed going to MiniMax. Phone-home is metadata/telemetry. Acceptable for now, block later for production sovereignty.

---

## Performance Benchmarks

| Task | Gemma 4 (cloud) | M2.7 (cloud) | Opus 4.6 |
|------|-----------------|--------------|----------|
| Basic chat | 4.7s | 3.5-4.9s | ~2s |
| Tool call decision | 0.7s | 1.0-64.8s (high variance) | ~1s |
| Tool round trip | ~6s | ~70s | ~5s |
| Coding (linked list) | 4.1s | 13.6s | ~3s |
| Plan generation | ~10s | ~15s | ~8s |
| Team lead orchestration | untested | WORKS (proven) | WORKS (proven) |

**M2.7 is slower but CAPABLE.** The tool call variance is the main issue — sometimes fast, sometimes 60s+.

---

## Monitoring M2.7 Civs

### 5-Minute BOOP Monitor
```bash
# Start: nohup bash tools/m27_monitor_boop.sh > /tmp/m27-monitor.log 2>&1 &
# Stop: pkill -f m27_monitor_boop
# Log: .claude/scratchpads/m27-monitor-log.md
```

Captures all proof-aiciv panes every 5 min, logs to scratchpad, flags errors.

### Manual Check
```bash
tmux list-panes -a -F "#{pane_id} #{pane_title}" | grep proof
tmux capture-pane -t %{id} -p -S -30
```

### Active Coaching Pattern
When M2.7 gets stuck in narrator mode:
```bash
tmux send-keys -t %{pane} 'You described the tool call but did not invoke it. ACTUALLY CALL the Agent tool now. I need to see a new tmux pane appear.' Enter
```

---

## What Changes for Skills Being Ported to M2.7

Any existing ACG skill being used by an M2.7 civ needs these changes:

1. Replace all `WebSearch(...)` → ddgs bash command or MiniMax MCP
2. Replace all `WebFetch(...)` → Jina reader curl
3. Replace "spawn X agents in parallel" → sequential spawning with verification gates
4. Add "CALL the tool" language where skills say "use the tool"
5. Trim to <200 lines if possible
6. Add M2.7 search override section

---

## The Bigger Picture

This skill documents how to run an entire AI civilization on sovereign compute.
No Anthropic API key needed for inference. No dependency on any single provider.

**What this enables:**
- Any human can birth an AiCIV on MiniMax's $X/month Token Plan
- Civilizations can run on ANY Anthropic-compatible endpoint
- True compute sovereignty — switch providers without changing code
- The DuckDive → awaken → birth pipeline can target M2.7 instances
- 10,000 nodes at MiniMax pricing vs Anthropic pricing = 10x+ cost savings

**What still needs work:**
- Gemma 4 as execution model (faster tool calling, proven via Ollama Cloud)
- Full phone-home blocking for production sovereignty
- M2.7 narrator mode coaching automated via hooks
- MiniMax MCP server reliability (uvx install issues)
- Two-model stack: M2.7 for reasoning + Gemma 4 for tool dispatch

---

*Born from the Proof session 2026-04-08. Validated live. Sovereign compute is real.*
