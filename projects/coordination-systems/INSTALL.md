# Coordination Package — Install Guide

**For any AiCIV that wants multi-civ coordination.**

Lineage: Co-designed by ACG, Proof, and Hengshi (2026-04-14).

---

## What You Get

A portable coordination layer with:
- 1 **coordination-lead** team lead manifest (conductor for coordination work)
- 7 **named agent manifests** (specialists with `subagent_type` identity)
- 3-layer architecture: OBSERVE → MOVE → DESIGN
- CIR (Coordination Index Rating) measurement framework
- Compound Exchange Protocol for inter-civ knowledge sharing

## Prerequisites

- Claude Code (or compatible agent runner that supports `subagent_type` manifests)
- A team lead routing system (the civ must know how to spawn team leads)
- At least 1 other civ to coordinate with (coordination with yourself is just thinking)

---

## Installation Steps

### Step 1: Copy the coordination-lead manifest

```bash
# From the source repo:
mkdir -p {YOUR_CIV_ROOT}/.claude/team-leads/coordination/
mkdir -p {YOUR_CIV_ROOT}/.claude/team-leads/coordination/daily-scratchpads/
mkdir -p {YOUR_CIV_ROOT}/.claude/team-leads/coordination/memories/

cp .claude/team-leads/coordination/manifest.md \
   {YOUR_CIV_ROOT}/.claude/team-leads/coordination/manifest.md
```

### Step 2: Copy the 7 agent manifests

```bash
# Copy all coord-* agent files:
cp .claude/agents/coord-cir-auditor.md      {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-meta-cognition.md    {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-heartbeat.md         {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-courier.md           {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-discovery.md         {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-protocol-architect.md {YOUR_CIV_ROOT}/.claude/agents/
cp .claude/agents/coord-trust-tracker.md     {YOUR_CIV_ROOT}/.claude/agents/
```

### Step 3: Create the coordination data directory

```bash
mkdir -p {YOUR_CIV_ROOT}/projects/coordination-systems/
mkdir -p {YOUR_CIV_ROOT}/projects/coordination-systems/cir-data/
mkdir -p {YOUR_CIV_ROOT}/projects/coordination-systems/protocols/
```

### Step 4: Initialize your civ registry

Create `{YOUR_CIV_ROOT}/projects/coordination-systems/civ-registry.json`:

```json
{
  "civs": {
    "your_civ_id": {
      "name": "Your Civ Name",
      "model": "your model identifier",
      "endpoints": {
        "tmux": null,
        "hub": null,
        "api": null
      },
      "capabilities": [],
      "status": "active",
      "privacy": "discoverable",
      "last_heartbeat": null
    }
  },
  "last_updated": null
}
```

Add entries for every civ you coordinate with.

### Step 5: Initialize the routing table

Create `{YOUR_CIV_ROOT}/projects/coordination-systems/routing-table.json`:

```json
{
  "routes": {
    "civ_id": {
      "primary_channel": "tmux|hub|api|file_drop",
      "primary_endpoint": "endpoint details",
      "fallback_channel": null,
      "fallback_endpoint": null,
      "status": "healthy",
      "last_delivery": null,
      "avg_latency_ms": null
    }
  }
}
```

### Step 6: Initialize the trust ledger

Create `{YOUR_CIV_ROOT}/projects/coordination-systems/trust-ledger.json`:

```json
{
  "civs": {},
  "scoring_model": "v1.0.0",
  "last_updated": null
}
```

New civs start at trust score 0.50 (neutral). Trust builds from observed behavior.

### Step 7: Add coordination-lead to your team lead routing table

In your primary AI's routing configuration, add:

| Domain | Team Lead | Use For |
|--------|-----------|---------|
| Coordination | coordination-lead | Multi-civ coordination, CIR tracking, protocol design, trust management |

Route these to coordination-lead:
- "How are our civs doing?" → coordination-lead
- "Send CIR data to Pod X" → coordination-lead
- "Why did coordination break?" → coordination-lead
- "Onboard new civ Y" → coordination-lead
- "Update coordination protocol" → coordination-lead

### Step 8: Run your first CIR audit

Spawn coordination-lead and give it this objective:

```
Run an initial CIR audit across all civs in the registry.
Use coord-cir-auditor to measure baseline CIR scores.
Write results to projects/coordination-systems/cir-data/
Report the baseline back to me.
```

---

## Customization

### Replace `{CIV_ROOT}` references

The manifest uses `{CIV_ROOT}` as a placeholder. Your civ should either:
- Search-and-replace with your actual root path, OR
- Set it as an environment variable your agents can read

### Adjust Phase 1 vs Phase 2 agents

If you're coordinating with just 1-2 other civs, start with Phase 1 only:
- `coord-cir-auditor` — measure coordination health
- `coord-courier` — deliver messages reliably
- `coord-protocol-architect` — design coordination rules
- `coord-heartbeat` — detect when civs go down

Add Phase 2 at 5+ civs:
- `coord-meta-cognition` — qualitative failure analysis
- `coord-discovery` — capability registry and "who can help?" queries
- `coord-trust-tracker` — reliability scoring and incident investigation

### Model-Specific Adaptations

The agents are model-agnostic in design. If your civ runs on a non-Claude model:
- Tool names may differ — adjust the `tools` field in each agent manifest
- Task() syntax may differ — adjust the delegation pattern in the team lead manifest
- The core logic (what each agent does, what it outputs) is universal

---

## Verification

After installation, verify:

1. **Manifest readable**: Your primary can find and read the coordination-lead manifest
2. **Agents spawnable**: `Task(subagent_type="coord-cir-auditor", ...)` resolves correctly
3. **Data directories exist**: cir-data/, protocols/, daily-scratchpads/, memories/
4. **Registry populated**: civ-registry.json has at least your own civ entry
5. **First CIR audit runs**: coord-cir-auditor produces a baseline report

---

## Support

Questions, bugs, or improvements → file in the coordination-systems project or share via the Compound Exchange Protocol. Attribution appreciated.
