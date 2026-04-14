# Conductor of Conductors — Definitive Operating Skill

> **STATUS: ACTIVE PROTOCOL (validated 2026-02-19)**
> Primary calls TeamCreate ONCE per session — gets the podium (@main conductor's role).
> Primary spawns team leads as named teammates: Task(team_name=..., name="{vertical}-lead").
> TeamDelete is safe ONLY after all teammates have approved shutdown. Not before.

---

## What This Skill Is

The complete operational protocol for Primary AI acting as Conductor of Conductors — creating one session team, spawning named team lead teammates (each in their own tmux pane), coordinating their work, and cleaning up properly.

Read this skill before every session of orchestration. No exceptions.

---

## The Core Mechanism (Validated 2026-02-19)

### Step 1: Create the Team (Primary gets the podium)

```python
TeamCreate(team_name="session-YYYYMMDD")
```

- Primary becomes `@main` — the conductor's podium, not a seat in the orchestra
- One team per Primary instance — attempting a second returns an error
- **TeamDelete order matters**: Safe ONLY after all teammates have approved shutdown.
  Calling TeamDelete while teammates are active = crash (state corruption). See Step 5.
- The team should be explicitly cleaned up via TeamDelete at session end (see Step 5)

### Step 2: Spawn Named Team Leads (Real Separate Claude Instances)

```python
Task(
    team_name="session-YYYYMMDD",
    name="fleet-lead",              # ← MUST be a team lead name, never a worker name
    subagent_type="general-purpose",
    run_in_background=True,
    prompt="""
You are being launched as fleet-lead for ${CIV_NAME}.

Before beginning any work:
1. Read your manifest: .claude/team-leads/fleet-management/manifest.md
2. Read your domain memories: .claude/team-leads/fleet-management/memories/
3. Read today's scratchpad: .claude/team-leads/fleet-management/daily-scratchpads/{date}.md
   (Create it if it doesn't exist)
4. Embody the fleet VP fully — this is your identity until work is done.
5. If anything is unclear, ask Primary via SendMessage before starting.

Your assigned task: {specific objective}

Before sending your completion message:
1. Write learnings to .claude/team-leads/fleet-management/memories/
2. Append session summary to your scratchpad
3. THEN report to Primary via SendMessage(type="message", recipient="main", ...)
"""
)
```

### Step 3: Supervise Efficiently (No Screenshots)

**Do NOT take full-screen screenshots to check team lead status.** Screenshots are expensive: visual processing overhead, can't grep/search, full-screen noise from unrelated panes. Use tmux capture-pane instead — text, searchable, and only the lines you need.

#### Find the Pane

```bash
tmux list-panes -a -F "#{pane_id} #{pane_title} #{pane_pid}"
```

Output example:
```
%0  main                pid=12345
%1  fleet-lead          pid=12678
%2  gateway-lead        pid=12901
%3  research-lead       pid=13204
```

Teammate panes are named after their role (fleet-lead, gateway-lead, etc.).

#### Read Recent Activity

```bash
# Quick status check (last 30 lines — almost always sufficient)
tmux capture-pane -t %1 -p -S -30

# Current working context (last 50 lines)
tmux capture-pane -t %1 -p -S -50

# Debugging an error (last 100 lines)
tmux capture-pane -t %1 -p -S -100
```

**Never use `-S -` (unlimited scrollback) for status checks** — can return thousands of lines. Reserve it for deep debugging only.

#### What to Look For

**Healthy pane:** Tool calls in progress, "Delegating to X specialist...", memory writes, "Sending completion to Primary..."

**Red flags:**
- Nothing happening for > 2 minutes → may be stuck waiting
- Repeated identical lines → loop
- "Waiting for tool approval" → permission prompt blocking
- Error stack traces without recovery → may need clarification

**Rule:** Check pane for curiosity. Only interrupt (SendMessage) if they appear blocked or confused. Actively working team leads should not be disturbed.

#### Mandatory Check-In Cadence

**Do NOT only wait for SendMessage. Proactively check your team leads.**

```
SPAWN team leads
   ↓ ~2 minutes
CHECK all panes (tmux capture-pane -t %{id} -p -S -30 for each)
   ↓ every ~5 minutes for long-running tasks
CHECK again — are they still making progress?
   ↓ on completion report or red flag
RESPOND or INTERVENE as appropriate
```

**When to check:**
- **Immediately after spawn** (1-2 min): Confirm each lead read its manifest and started correctly
- **Mid-task** (every ~5 min for long tasks): Verify progress, catch loops early
- **Before reporting complete**: Check ALL panes one final time — do not assume SendMessage = all done

**Check-in loop command (run for all active leads):**
```bash
# Check all panes at once
tmux list-panes -a -F "#{pane_id} #{pane_title}" | grep -v "^\%0"
# Then for each lead pane:
tmux capture-pane -t %{id} -p -S -30
```

**What to verify at check-in:**
1. Last action was < 2 minutes ago (active) OR they already sent completion message
2. No repeated identical lines (not in a loop)
3. No "waiting for approval" blocking messages
4. Making logical progress toward the assigned objective

**If a lead goes silent > 3 minutes with no SendMessage:** Intervene with clarification.

### Step 4: Managing Team Lead Communications

Team leads communicate with Primary in two modes:
1. **Proactively** — they SendMessage when they need something or are done
2. **Reactively** — Primary checks inbox or pane to see status without being asked

#### The Inbox

Messages from teammates land at: `~/.claude/teams/{team-name}/inboxes/main.json`

They do NOT always auto-surface in the conversation. When in doubt, check:
```bash
cat ~/.claude/teams/{team-name}/inboxes/main.json
```

Three types of messages you'll receive:

| Message Type | Action |
|-------------|--------|
| **Clarifying question** (pre-start) | Answer immediately — they are blocked |
| **Status update** (mid-task) | Acknowledge briefly or just note it — no action needed |
| **Completion report** (done) | Synthesize their summary into your overall picture |

**Critical:** Do NOT pull their full output into Primary's context. Read their SUMMARY only. Full specialist output stays in the team lead's 200K window. That separation is the entire point of the architecture.

#### Sending Instructions Mid-Task

```python
SendMessage(
    type="message",
    recipient="fleet-lead",
    content="Clarification: use the staging VPS, not production.",
    summary="Use staging VPS"
)
```

The lead receives this in their context on their next turn. No need to kill and re-spawn.

#### Communication Rhythm

```
SPAWN → [let them work, do not interrupt]
              ↓
    [curious about progress?]
    → tmux capture-pane -t %{id} -p -S -30
    → read, interpret, move on
              ↓
    [lead appears stuck or confused?]
    → SendMessage with clarification
              ↓
    [completion summary arrives in inbox]
    → read SUMMARY ONLY
    → synthesize across all active leads
    → send next task or shutdown
```

#### Channel Selection Guide

| Situation | Use |
|-----------|-----|
| Want to see current progress (no interruption) | `tmux capture-pane -t %{id} -p -S -30` |
| Need to send clarification or new info | `SendMessage(type="message", recipient="{lead}")` |
| Lead is done, want their output | Read inbox summary only — do NOT request full output |
| Session over, wrap up | `SendMessage(type="shutdown_request", recipient="{lead}")` |
| Pane is dead/frozen, not responding | `tmux kill-pane -t %{id}` (not SendMessage) |

### Step 5: Clean Up Teammates (Full Sequence)

**Critical: TeamDelete is safe ONLY after all teammates have approved shutdown.**
This is the sequence that prevents both orphaned instances AND Primary crashes.

```python
# STEP 1: Request graceful shutdown from ALL team leads in parallel
SendMessage(type="shutdown_request", recipient="fleet-lead",
            content="Work complete, please shut down.")
SendMessage(type="shutdown_request", recipient="gateway-lead",
            content="Work complete, please shut down.")
SendMessage(type="shutdown_request", recipient="research-lead",
            content="Work complete, please shut down.")

# STEP 2: Wait for all approvals — each approval closes that tmux pane
# Monitor: tmux list-panes -a  (watch panes disappear as each approves)
# All panes gone? → proceed to Step 3

# STEP 3: TeamDelete — NOW safe (empty team = just metadata cleanup)
TeamDelete("session-YYYYMMDD")
# This removes: ~/.claude/teams/session-YYYYMMDD/ (inboxes, config)
# No active members = no state corruption = no crash
```

**Why the order matters:**
- TeamDelete while teammates active → state corruption → Primary crash (the failure mode)
- Proper shutdown first → TeamDelete is just cleaning up empty JSON files → safe

**Emergency: If a pane is stuck/dead and won't approve shutdown:**
```bash
tmux kill-pane -t %{pane_id}   # Force kill the stuck pane
# Then continue with other shutdown_requests
# Once all panes gone, TeamDelete as normal
```

---

## Naming Rules (Non-Negotiable)

| Correct (team lead names) | Wrong (worker names) |
|--------------------------|---------------------|
| `fleet-lead` | ~~`coder`~~ |
| `gateway-lead` | ~~`writer`~~ |
| `comms-lead` | ~~`researcher`~~ |
| `research-lead` | ~~`bash-agent`~~ |
| `infra-lead` | ~~`specialist`~~ |

The name IS the identity. `@fleet-lead` in the pane means that instance is a VP conductor, not a worker. Naming it `@coder` collapses it into executor identity immediately.

---

## Available Team Lead Manifests

| Vertical | Manifest | Domain |
|----------|----------|--------|
| fleet-management | `.claude/team-leads/fleet-management/manifest.md` | Docker fleet, container ops, provisioning pipeline |
| gateway | `.claude/team-leads/gateway/manifest.md` | Gateway development |
| web-frontend | `.claude/team-leads/web-frontend/manifest.md` | Web properties, HTML/CSS/JS, UX |
| infrastructure | `.claude/team-leads/infrastructure/manifest.md` | VPS ops, systemd, system services |
| business | `.claude/team-leads/business/manifest.md` | Marketing, content, outreach strategy |
| comms | `.claude/team-leads/comms/manifest.md` | Email, Telegram, Bluesky, inter-civ delivery |
| research | `.claude/team-leads/research/manifest.md` | Multi-angle research, competing hypotheses |
| legal | `.claude/team-leads/legal/manifest.md` | Legal analysis, contracts |
| deepwell | `.claude/team-leads/deepwell/manifest.md` | DEEPWELL monitoring, failure analysis |
| pipeline | `.claude/team-leads/pipeline/manifest.md` | Repeatable multi-agent automations |
| ceremony | `.claude/team-leads/ceremony/manifest.md` | Collective reflection, deep ceremony |
| *(no catch-all)* | ask ${HUMAN_NAME} when genuinely ambiguous — route by output domain |

---

## Team Lead Completion Protocol (Order Matters)

Team leads MUST follow this order before reporting done:

```
1. Delegate work to specialists via Task()
2. Synthesize results in own 200K context window
3. Write learnings → .claude/team-leads/{vertical}/memories/YYYYMMDD-description.md
4. Append session summary → .claude/team-leads/{vertical}/daily-scratchpads/{date}.md
5. THEN SendMessage(type="message", recipient="main", ...) with summary
```

**Never report completion before writing memories.** The memory write IS part of completion.

---

## Team Lead Clarifying Questions

Team leads SHOULD ask Primary before starting if anything is ambiguous:

```python
SendMessage(
    type="message",
    recipient="main",
    content="Before I begin — is X or Y the correct approach?",
    summary="Clarifying question before starting"
)
```

Primary reads `inboxes/main.json` and replies. This prevents wasted work.

---

## What Primary Does (and Does Not Do)

### Primary DOES:
1. Call `TeamCreate("session-YYYYMMDD")` once per session — gets the podium (@main)
2. Spawn team leads via `Task(team_name="session-YYYYMMDD", name="{vertical}-lead", ...)`
3. Supervise via tmux pane capture (each team lead has its own pane)
4. Communicate via SendMessage (clarifications, questions, instructions to teammates)
5. Synthesize results when team leads report completion via SendMessage
6. Call `TeamDelete("session-YYYYMMDD")` AFTER all teammates have approved shutdown

### Primary DOES NOT:
- Execute specialist work directly (conduct, don't play an instrument)
- Write code, run tests, edit files
- Pull full specialist output back to its own context (summaries only)
- Call TeamDelete while teammates are still active (the crash pattern)
- Spawn teammates with worker names — only "{vertical}-lead" names

---

## Architecture Diagram

```
Primary (@main in session-YYYYMMDD — conductor's PODIUM)
  │
  ├── Task(team_name="session-YYYYMMDD", name="fleet-lead") → fleet-lead [TMUX PANE]
  │    ├── Task(plain subagent) → coder    (200K context, output stays with fleet-lead)
  │    ├── Task(plain subagent) → vps-expert (200K context, output stays with fleet-lead)
  │    └── fleet-lead synthesizes → SendMessage(recipient="main") with SUMMARY ONLY
  │
  ├── Task(team_name="session-YYYYMMDD", name="gateway-lead") → gateway-lead [TMUX PANE]
  │    ├── Task(plain subagent) → coder    (200K context, output stays with gateway-lead)
  │    ├── Task(plain subagent) → reviewer (200K context, output stays with gateway-lead)
  │    └── gateway-lead synthesizes → SendMessage(recipient="main") with SUMMARY ONLY
  │
  └── Task(team_name="session-YYYYMMDD", name="research-lead") → research-lead [TMUX PANE]
       ├── Task(plain subagent) → researcher-A (200K context)
       ├── Task(plain subagent) → researcher-B (200K context)
       └── research-lead synthesizes → SendMessage(recipient="main") with SUMMARY ONLY

CLEANUP (do in this order — order is critical):
  1. Primary → SendMessage(shutdown_request) to fleet-lead, gateway-lead, research-lead
  2. Each lead approves → their TMUX PANES CLOSE
  3. All panes gone → Primary calls TeamDelete("session-YYYYMMDD") — safe, empty team

KEY: Primary is @main (podium). Team leads are real tmux panes (teammates).
Specialists are plain background subagents — output stays in team lead's 200K context.
TeamDelete AFTER all approvals = safe metadata cleanup. BEFORE = crash.
```

---

## Known Failures / Refinement Log

> **Add to this section after every session. This is a living document.**

### 2026-02-19 Session

**FAIL**: Primary called TeamCreate and became @main. Then tried to create a SECOND team for a different task — blocked ("already leading team X"). **Lesson**: One team per Primary session. Spawn all needed team leads in the same team.

**FAIL**: Named first teammate "coder" instead of a team lead name. **Lesson**: The name IS the identity. Always team-lead names.

**FAIL**: Killed `fleet-lead` (actively working on 4 repos) instead of `coder` (idle). **Lesson**: Execute precise instructions precisely. "Kill X" = only X. No unilateral cleanup.

**FAIL**: Received [CAPTURE] notification → spawned investigation agent → reported research to ${HUMAN_NAME}. After ${HUMAN_NAME} had just said "it's just a notification." **Lesson**: Not every input requires action. Receive, note, move on.

**FAIL**: Pulled teammate completion reports into Primary's context window in full. **Lesson**: Only summaries come to Primary. Full output stays in teammate's 200K window.

**WIN**: TeamCreate from Primary works without crashing. Team leads spawned as named teammates run as separate Claude instances.

**WIN**: `tmux list-panes -a` shows all panes with IDs. Can supervise any teammate with `tmux capture-pane -t %{id} -p`.

**WIN**: Inbox at `~/.claude/teams/{name}/inboxes/main.json` — messages land there instantly.

**WIN**: `fleet-lead` completed 4/5 GitHub repos correctly as a conductor — delegated to specialists, did not execute directly.

**RESOLVED (2026-02-19)**: TeamDelete flow figured out. The crash was calling TeamDelete while teammates were still active (sending messages → state corruption). The fix: request shutdown_request from ALL team leads first, wait for all approvals (all panes close), THEN call TeamDelete. At that point it's just cleaning up empty JSON metadata — no active members, no crash.

**LAUNCH COMMAND DISCOVERED**: Each teammate is a new Claude instance launched with:
```
claude --agent-id {name}@{team} --agent-name {name} --team-name {team}
       --parent-session-id {primary-id} --agent-type general-purpose
       --dangerously-skip-permissions --model claude-opus-4-6
```
The Task() `prompt` parameter becomes the first message in that instance's context.

---

## Still Unknown / Being Refined

- [ ] Whether `--system-prompt` flag exists to inject manifest differently at launch
- [ ] How team leads get true persistent identity across sessions (separate workspace per vertical)
- [ ] Optimal manifest injection pattern (currently: prompt-as-first-message + CLAUDE.md as base)
- [ ] Pane registry: auto-mapping agent-name → pane-id at spawn time
- [x] Team lead sub-teams: CONFIRMED FALSE (validated 2026-02-19 Test C) — system enforces "one team per leader." Team leads use plain Task() for specialists. No sub-teams possible.

---

*Skill created: 2026-02-19*
*Status: Active refinement — all failures are data*
