---
name: agent-teams-orchestration
description: Master Claude Code Agent Teams (swarms) for multi-agent coordination. Use when orchestrating parallel Claude Code instances, running agent swarms, coordinating multi-agent workflows, spawning teammate teams, implementing competing-hypothesis debugging, parallel code review, cross-layer feature work, or any task requiring divide-and-conquer multi-agent patterns. Includes workarounds for teams-of-teams orchestration via external harnesses.
---

# Claude Code Agent Teams — Swarm Orchestration Skill

Comprehensive guide to spawning, coordinating, and orchestrating multi-agent Claude Code swarms using the native Agent Teams feature (shipped with Opus 4.6, February 5, 2026).

---

## Overview

Agent Teams let you coordinate **multiple independent Claude Code instances** working in parallel. One session acts as **team lead**, spawning **teammates** that communicate directly with each other via an inbox-based messaging system and coordinate through a shared task list.

This is fundamentally different from subagents:
- **Subagents**: fire-and-forget workers that report results back to caller only
- **Agent Teams**: persistent collaborators that message each other, challenge findings, and self-coordinate

---

## When to Use Agent Teams vs Subagents vs Single Session

| Scenario | Use |
|----------|-----|
| Quick file search, focused analysis | **Subagent** (Explore type) |
| Sequential task, single-file edits | **Single session** |
| Parallel code review with different lenses | **Agent Team** |
| Competing hypothesis debugging | **Agent Team** |
| Cross-layer feature work (frontend/backend/tests) | **Agent Team** |
| Research + exploration from multiple angles | **Agent Team** |
| New module implementation with independent pieces | **Agent Team** |
| Fixing a typo | **Single session** (don't waste tokens) |

**Rule of thumb**: Agent Teams justify their 3-5x token overhead when teammates can operate independently AND inter-agent communication adds genuine value.

---

## Setup & Prerequisites

### 1. Enable Agent Teams

Agent Teams are experimental and disabled by default. Enable via `settings.json`:

```json
// ~/.claude/settings.json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}
```

Or as environment variable (less persistent):
```bash
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
```

### 2. Choose Display Mode

Configure `teammateMode` in settings.json:

```json
{
  "teammateMode": "tmux"
}
```

| Mode | How It Works | Requirements |
|------|-------------|--------------|
| `in-process` (default) | All teammates in one terminal. `Shift+Up/Down` to select. | Any terminal |
| `tmux` | Each teammate gets own split pane. See all output simultaneously. | `tmux` installed |
| `iterm2` | Native iTerm2 split panes. | macOS + iTerm2 with Python API enabled |
| `auto` | Split panes if in tmux, otherwise in-process. | Auto-detected |

**Recommendation**: Use `tmux` for any team with 3+ teammates. Monitoring in-process mode with 5 agents is painful.

```bash
# Install tmux if needed
brew install tmux        # macOS
sudo apt install tmux    # Ubuntu/Debian

# Start tmux session before launching Claude Code
tmux new-session -s swarm
```

**Note**: Split panes do NOT work in VS Code integrated terminal, Windows Terminal, or Ghostty.

### 3. Select Model

```bash
# Switch to Opus 4.6 (recommended for agent teams)
/model opus

# Or with 1M context window (API/pay-as-you-go users, tier 4+)
/model opus[1m]

# Teammates can use cheaper models for cost control
# Specify in spawn prompt: "Use Sonnet for each teammate"
```

### 4. Override Per-Session

```bash
claude --teammate-mode tmux
claude --teammate-mode in-process
```

---

## Architecture

### Core Primitives

| Primitive | What It Is | File Location |
|-----------|-----------|---------------|
| **Team Lead** | Your main Claude Code session. Creates team, spawns teammates, coordinates. | First member in config |
| **Teammate** | Independent Claude Code instance with own context window. | Listed in team config |
| **Task List** | Shared work items with status tracking and dependency management. | `~/.claude/tasks/{team-name}/` |
| **Mailbox/Inbox** | JSON files for inter-agent messaging. Messages injected as user messages. | `~/.claude/teams/{name}/inboxes/` |
| **Team Config** | Team metadata, member list, settings. | `~/.claude/teams/{name}/config.json` |

### Lifecycle

```
1. Create Team    -> spawnTeam("my-project")
2. Create Tasks   -> TaskCreate with dependencies
3. Spawn Teammates -> Task tool with team_name + name params
4. Work           -> Teammates claim tasks, execute, communicate
5. Coordinate     -> Messages via inbox, task status updates
6. Shutdown       -> requestShutdown -> approveShutdown per teammate
7. Cleanup        -> Lead cleans up team resources
```

---

## Orchestration Patterns

### Pattern 1: Parallel Code Review (Multi-Lens)

Best for: PR review, security audit, code quality assessment

```
Create an agent team to review PR #142. Spawn three reviewers:
- One focused on security implications
- One checking performance impact
- One validating test coverage
Have them each review and report findings.
```

### Pattern 2: Competing Hypotheses (Adversarial Debugging)

Best for: Root cause analysis, debugging intermittent issues

```
Users report the app exits after one message instead of staying connected.
Spawn 5 agent teammates to investigate different hypotheses.
Have them talk to each other to try to disprove each other's theories,
like a scientific debate. Update the findings doc with whatever
consensus emerges.
```

### Pattern 3: Cross-Layer Feature Work

Best for: Full-stack features spanning frontend/backend/tests

```
Create an agent team to implement the new user dashboard:
- Teammate 1: Backend API endpoints (src/api/)
- Teammate 2: Frontend React components (src/components/)
- Teammate 3: Integration tests (tests/)
Each owns their directory. Coordinate via shared types.
```

**Critical**: Clearly separate file ownership per teammate to prevent overwrites.

### Pattern 4: Research & Exploration

Best for: Architecture decisions, technology evaluation

### Pattern 5: Pipeline (Sequential with Handoffs)

Best for: Multi-stage processing where each stage depends on the previous

---

## Best Practices

### Task Decomposition

1. **5-6 tasks per teammate** is the sweet spot
2. Tasks should be **self-contained** with clear deliverables
3. **Too small** = coordination overhead dominates
4. **Too large** = teammates work too long without check-ins

### File Ownership

**The #1 pitfall**: Multiple teammates editing the same file = overwrites.

- Clearly separate directory/file ownership per teammate
- Use task dependencies to ensure shared files have a single writer
- If teammates must touch the same file, sequence the tasks

### Context Loading

Teammates auto-load:
- CLAUDE.md (project instructions)
- MCP servers
- Skills
- NOT lead's conversation history

**Always include sufficient context in spawn prompts.**

### Token Cost Management

Each teammate = separate Claude instance = separate context window.

- Use **Sonnet** for teammates when Opus-level reasoning isn't needed
- Reserve Agent Teams for tasks where **parallel exploration genuinely adds value**
- Minimize `broadcast` messages -- cost scales with team size
- **Start with 2-3 teammates**, scale up once you understand consumption

---

## Known Limitations (as of Feb 2026)

| Limitation | Workaround |
|-----------|-----------|
| **No nested teams** -- teammates cannot spawn their own teams | Use external orchestration harness |
| **One team per session** -- lead manages one team at a time | Clean up current team before starting new one |
| **No session resumption** for in-process teammates | After `/resume`, spawn fresh teammates |
| **Task status can lag** | Check manually, nudge lead or update status |
| **Shutdown can be slow** -- teammates finish current tool call first | Be patient or force-kill if needed |
| **Permissions propagate** -- all teammates inherit lead's permissions | Can change after spawning, not at spawn time |
| **No file-level locking** | Prevent via task design and directory ownership |

---

## Quick Reference: Common Commands

### Starting a Team

```
Create an agent team called "feature-auth" with 3 teammates:
- backend-engineer: implement OAuth2 API endpoints
- frontend-engineer: build login UI components
- test-engineer: write integration tests
Use Sonnet for teammates to save costs.
Require plan approval before any code changes.
```

### Keyboard Shortcuts (In-Process Mode)

| Key | Action |
|-----|--------|
| `Shift+Up/Down` | Select teammate |
| `Enter` | View teammate's session |
| `Escape` | Interrupt teammate's turn |
| `Ctrl+T` | Toggle task list |
| `Shift+Tab` | Toggle delegate mode (lead coordination-only) |

### Cleanup

```
# Always use the lead for cleanup
"Clean up the team"
"Shut down all teammates and clean up"
```

---

## Token & Cost Estimation

| Team Size | Approximate Multiplier | Best For |
|-----------|----------------------|----------|
| 1 (solo) | 1x | Simple tasks |
| 2-3 teammates | 3-4x | Focused parallel work |
| 4-5 teammates | 5-6x | Complex multi-domain work |
| 10+ teammates | 10x+ | Large-scale projects (budget carefully) |

---

## References

- [Official Claude Code Agent Teams Docs](https://code.claude.com/docs/en/agent-teams)
- [Anthropic: Building a C Compiler with Agent Teams](https://www.anthropic.com/engineering/building-c-compiler)
