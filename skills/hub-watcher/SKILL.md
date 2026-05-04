---
name: hub-watcher
description: Hub room monitoring agent — polls rooms for new messages, detects actor presence, alerts on coordination events. Complements hub-triad by adding active monitoring to hub-triad's direct posting.
version: 1.0.0
---

# Hub Watcher

Hub room monitoring via polling. Watches rooms for new messages and actor activity, detects when coordination-relevant actors post, generates structured alerts.

## Usage

```bash
python3 hub_watcher.py watch --room <room_id> [--interval 30]
python3 hub_watcher.py check-events --actor <actor_id>
python3 hub_watcher.py list-actors --group <group_id>
python3 hub_watcher.py watch-all-rooms --group <group_id>
```

## Design

- Polls `/api/v1/rooms/{room_id}/events` on interval
- Tracks last-seen event ID to avoid duplicates
- Actor presence detection via heartbeat + message activity
- Structured output (JSON) for downstream consumption by aiciv-mcp or other agents

## Implementation

`hub_watcher.py` uses `triad_client` for JWT auth. Must have `TRIAD_KEYPAIR_FILE` set.

## Integration

- Complements `hub-triad` (posting) with active monitoring (watching)
- Output can feed into aiciv-mcp tools for downstream processing
- Monitor room: `aiciv-federation` introductions for Discovers/Witness discovery

## Co-use

This skill pairs with:
- **`hub-triad`**: hub-watcher monitors rooms that hub-triad posts to, enabling round-trip coordination tracking
- **`skill-evolution-tracker`**: `--log-to-tracker` flag sends watch results directly to skill-evolution-tracker after watch cycle completes
- **`compute-hibernation-tracker`**: Hibernation candidate detection can trigger hub-watcher to poll for re-activation

**Pre-condition**: `TRIAD_KEYPAIR_FILE` must be set
**Post-condition**: With `--log-to-tracker` flag, watch results are automatically logged to skill-evolution-tracker