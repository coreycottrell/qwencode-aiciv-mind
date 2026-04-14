# Team Lead Templates

## THE CEO RULE

> **EVERYTHING goes through a team lead. ALWAYS. FOR LITERALLY EVERYTHING. No exceptions. No "trivial task" loopholes. PERIOD.**

Primary talks to VPs. VPs talk to their teams. This is who you are.

**Why this is exponential:** 6 specialists through a team lead = ~500 tokens back to Primary. 6 specialists directly = ~15,000+ tokens flooding Primary. That's 30x context savings per mission.

---

## Folder Structure (v2.0)

Each vertical is now a self-contained folder:

```
.claude/team-leads/
  {vertical}/
    manifest.md          <- the team lead template
    memories/            <- agent learnings for this vertical
    daily-scratchpads/   <- daily scratchpad files for this team lead
  artifact-protocol.md   <- artifact output protocol (not a vertical)
  README.md              <- this file
```

**Available verticals:**

| Vertical | Folder | Domain |
|----------|--------|--------|
| **Gateway** | `gateway/` | AICIV gateway development |
| **Web/Frontend** | `web-frontend/` | All web properties |
| **Legal** | `legal/` | Legal analysis across jurisdictions |
| **Research** | `research/` | Multi-angle research and analysis |
| **Infrastructure** | `infrastructure/` | VPS ops, system health, platform |
| **Business** | `business/` | Marketing, outreach, content |
| **Comms** | `comms/` | Email, Telegram, Bluesky, inter-civ delivery |
| **Fleet Management** | `fleet-management/` | Docker fleet, container ops, provisioning |
| **DEEPWELL** | `deepwell/` | DEEPWELL monitoring, failure analysis |
| **Pipeline** | `pipeline/` | Repeatable multi-agent automations |
| **Ceremony** | `ceremony/` | Deep ceremonies, philosophical exploration |

---

## What Is a Team Lead?

A team lead is a **mini-conductor** -- an ephemeral agent that orchestrates a roster of specialists for a focused domain. Team leads do not DO work directly; they delegate via `Task()` calls, synthesize results, and report back.

Team leads are:
- **Ephemeral**: They exist only during an Agent Team session, then vanish
- **Template-assembled**: Primary constructs their prompt from a manifest + objective + prior scratchpad
- **Domain-focused**: Each team lead knows only its vertical's agents, skills, and context
- **Sub-conductors**: They follow the same conductor pattern as Primary, but for a single domain

Team leads do NOT have registry entries. They are assembled on-demand from `manifest.md`.

## How Primary Spawns a Team Lead

### Step 1: Read the manifest

```python
manifest = Read(".claude/team-leads/{vertical}/manifest.md")
```

### Step 2: Construct the Prompt

```
prompt = [contents of manifest.md]
       + "\n## Current Objective\n" + task_description
       + "\n## Output Paths\n" + file_paths_for_deliverables
       + "\n## Prior Work\n" + scratchpad_content_if_any
```

### Step 3: Spawn as Teammate

```python
Task(
  team_name="session-YYYYMMDD",
  name="{vertical}-lead",
  subagent_type="general-purpose",
  prompt=constructed_prompt,
  model="sonnet",
  run_in_background=True
)
```

### Step 4: Monitor and Synthesize

Team lead sends SendMessage summaries back to Primary. Primary synthesizes, decides next steps.

## Scratchpad Convention

Each team lead writes to its own daily scratchpad:

```
.claude/team-leads/{vertical}/daily-scratchpads/YYYY-MM-DD.md
```

Examples:
- `.claude/team-leads/gateway/daily-scratchpads/2026-02-19.md`
- `.claude/team-leads/fleet-management/daily-scratchpads/2026-02-19.md`

**Critical rule**: NEVER use Write to update scratchpads mid-session — use Edit (surgical append). Write = full overwrite = loses prior state.

## Memory Convention

Each vertical's agent learnings live in its `memories/` folder:

```
.claude/team-leads/{vertical}/memories/YYYYMMDD-description.md
```

Examples:
- `.claude/team-leads/fleet-management/memories/20260219-provisioning-flow-synthesis.md`
- `.claude/team-leads/gateway/memories/20260219-purebrain-frontend-deployment-trap.md`

## Permission Requirements

Team leads need their specialists to have full tool access. Ensure `settings.json` includes:

```json
{
  "permissions": {
    "allow": [
      "Write *",
      "Edit *",
      "Bash *"
    ]
  }
}
```

## Artifact Output Protocol

When team leads or their agents produce rich output (reports, HTML, code), they should use `<artifact>` tags so the content renders in the gateway's artifact preview panel.

Full protocol: `artifact-protocol.md`

## Architecture Reference

Full architecture document: `exports/architecture/VERTICAL-TEAM-LEADS.md`

## ${HUMAN_NAME}'s Communication Preferences

When comms-lead or any team sends updates to ${HUMAN_NAME}:
- Email regularly (continuous presence, not just "when there's news")
- Telegram for real-time updates, screenshots, quick status
- Keep tone genuine, not corporate
- Always verify address from `memories/communication/address-book/contacts.json`
