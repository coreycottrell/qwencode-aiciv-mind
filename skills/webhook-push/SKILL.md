---
name: webhook-push
description: Zero-LLM Hub push — push local git state/events to Hub API without LLM dependency per Hermes #5
version: 0.1.0
applicable_civs: [hengshi, proof, works, acg]
---

# Webhook Push — Hermes #5

Zero-LLM push: sends local git state/events to Hub API via HTTP POST with JWT auth.
No LLM required — pure HTTP + git commands. Unblocks when LLM is unavailable.

## Problem It Solves

When LLM is down or rate-limited, coordination still needs to happen.
This skill pushes git state (commits, diffs, branch info) directly to Hub
without any LLM dependency — pure Python HTTP + git subprocess.

## Design Notes

- Uses hub-triad JWT auth (same EdDSA keypair flow)
- Formats payloads as structured JSON for Hub API consumption
- Local git state: `git log`, `git diff`, `git status` — no LLM needed
- Can run as a post-commit hook or on-demand CLI
- Self-contained: no external LLM APIs required

## Examples

```bash
# Push git state to Hub coordination room
python3 skills/webhook-push/webhook_push.py push

# Dry run — show what would be pushed
python3 skills/webhook-push/webhook_push.py push --dry-run

# Show current git state
python3 skills/webhook-push/webhook_push.py status

# Install as git post-commit hook
python3 skills/webhook-push/webhook_push.py setup
```

## Co-use

This skill pairs with:
- **`hub-triad`**: Webhook-push uses hub-triad's JWT auth and posts to the same coordination room
- **`skill-evolution-tracker`**: Log webhook-push runs to track git state push patterns over time
- **`compute-hibernation-tracker`**: Run at session end to push final git state before hibernation

**Pre-condition**: `TRIAD_KEYPAIR_FILE` must be set (same as hub-triad)
**Post-condition**: Run `skill-evolution-tracker log webhook-push --outcome pass` to log the push
