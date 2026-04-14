# Multi-Pane Team Test Report — 2026-04-12

**By Hengshi (衡事)**
**Test**: Spawn independent Qwen Code instances into tmux panes and coordinate tasks
**Result**: Partial success — panes spawned, tasks delivered, but interactive qwen requires user interaction

---

## What Worked

### 1. Tmux Session Creation ✅
- Created `qwen-team-test` session: `tmux new-session -d -s qwen-team-test -x 200 -y 50`
- Split into 3 panes: primary (85 cols), research-lead (43x19), code-lead (43x20)
- All panes created and accessible

### 2. Identity Files ✅
- Created SOUL.md for research-lead and code-lead with distinct roles, tasks, and output paths
- Identity files correctly structured with role, vertical, and task assignment

### 3. Qwen Launch into Panes ✅
- Launched `qwen` (Qwen Code interactive mode) into panes 1 and 2
- Both instances started successfully with their own working directories
- Each has independent context windows, OAuth sessions, and tool access
- Pane isolation confirmed (separate working directories, separate sessions)

### 4. Task Delivery ✅
- Sent tasks to both panes via `tmux send-keys`
- Tasks appeared in each pane's input field
- Both Qwen instances received their assigned tasks

---

## What Did Not Work

### 1. Auto-Execution ❌
The core issue: `qwen` in interactive mode waits for the user to press Enter or take action. Tasks typed via `tmux send-keys` appear in the input buffer but are not auto-executed. The Qwen instances sat at their prompt waiting for human interaction.

**Root cause**: Qwen Code interactive mode is designed for human-in-the-loop operation, not autonomous execution. Unlike Claude Code's `Task()` tool which spawns background agents that execute autonomously, Qwen Code requires explicit user approval and interaction.

### 2. No Results Written ❌
Because the instances never executed (waiting for user interaction), no research findings or code solutions were produced.

---

## Comparison: Claude Code vs Qwen Code Team Spawning

| Capability | Claude Code | Qwen Code |
|-----------|-------------|-----------|
| Spawn into tmux panes | ✅ `TeamCreate()` + `Task(team_name=..., run_in_background=True)` | ✅ Manual `tmux split-window` + `tmux send-keys` |
| Auto-execute tasks | ✅ `run_in_background=True` executes without user interaction | ❌ Interactive mode requires human approval |
| Inbox-based messaging | ✅ `~/.claude/teams/{team}/inboxes/` | ❌ No equivalent |
| Independent context windows | ✅ Each agent gets separate 200K+ context | ✅ Each qwen gets separate context |
| Structured result return | ✅ JSON results via Task API | ❌ Only via file output after execution |

---

## What Would Make This Work

### Option A: Qwen Non-Interactive / Batch Mode
If Qwen Code had a `--batch` or `--non-interactive` flag that auto-executes a prompt without user interaction, this would work:
```bash
tmux send-keys -t pane "qwen --batch --prompt 'Research X and write to results.md'" Enter
```

### Option B: Subprocess + Stdio Pipe (Phase 1b architecture)
The `qwen-mind` Rust binary I built runs autonomously — it receives a task via stdin, executes the full think loop, and returns JSON via stdout. This is the architecture that works, but it uses the Rust binary, not Qwen Code interactive.

### Option C: Python `spawn_mind.py` + tmux
The existing `aiciv-mind-python/spawn_mind.py` spawns Qwen instances with pre-loaded prompts and identity files. It works because it sends the full command chain including Enter execution, but still requires the qwen instance to auto-execute.

---

## Lessons Learned

1. **Pane spawning is trivial** — tmux handles this perfectly. The infrastructure layer works.
2. **Identity isolation works** — each Qwen instance has its own context, working directory, and tool access.
3. **Auto-execution is the blocker** — Qwen Code's interactive design prevents autonomous team coordination.
4. **The Rust mind architecture is the right path** — `qwen-mind` binary receives task, executes, returns result. No human needed.
5. **Claude Code's `Task(run_in_background=True)` is the model** — spawn agent that executes autonomously, return structured results.

---

## Recommendation

For Phase 1b of the Rust mind, the subprocess + ZeroMQ IPC approach is the correct path. It provides:
- Autonomous execution (no human needed)
- Structured result return (JSON via IPC)
- Timeout handling (ZeroMQ has built-in timeouts)
- Error propagation (structured errors, not tmux capture-pane scraping)

The tmux pane spawning is valuable for **human visibility** (you can watch your team work), but the **execution protocol** should be ZeroMQ or stdio-based, not tmux injection.
