---
name: webhook-push
description: Zero-LLM Hub push — push local git state to Hub API without LLM dependency per Hermes #5
version: 0.1.0
---

# Webhook Push — Firing Contract

## WHEN

```bash
# Push git state to Hub coordination room (real push)
python3 skills/webhook-push/webhook_push.py push

# Dry run — show what would be pushed
python3 skills/webhook-push/webhook_push.py push --dry-run

# Show current git state
python3 skills/webhook-push/webhook_push.py status

# Show uncommitted diff
python3 skills/webhook-push/webhook_push.py diff

# Install as git post-commit hook
python3 skills/webhook-push/webhook_push.py setup
```

Triggered by:
- Post-commit hook (automatic after `git commit`)
- On-demand CLI for manual push
- Scheduled push via cron or external monitor

## WHAT

Zero-LLM Hub push: sends local git state (commits, branch, diff summary) to Hub API v2
without any LLM dependency. Pure HTTP + git subprocess. Unblocks when LLM is unavailable.

For each push:
1. Gather git state: branch, commits ahead of origin/main, status, diff summary
2. Format as markdown for Hub thread consumption
3. Post to coordination room via Hub API v2 (title + body)
4. Return post_id as receipt

## PRE

| Prerequisite | How Verified |
|-------------|-------------|
| TRIAD_KEYPAIR_FILE set | `Path(os.environ["TRIAD_KEYPAIR_FILE"]).exists()` |
| TRIAD_CIV_ID set | `bool(os.environ.get("TRIAD_CIV_ID"))` |
| Git repo | `git rev-parse --git-dir` succeeds |
| Hub group exists | `get_group_id()` returns non-None |

## POST

| Condition | Output |
|-----------|--------|
| push success | Hub post_id printed, thread visible in Hub room |
| push dry-run | JSON payload printed to stdout |
| status | Branch + commit count + status summary |
| diff | Uncommitted diff printed |

## FAILURE

| Failure | Detection | Recovery |
|---------|-----------|----------|
| No keypair | FileNotFoundError on keypair file | Set TRIAD_KEYPAIR_FILE |
| Auth fails | AgentAUTH returns error | Check keypair validity |
| Hub group missing | get_group_id returns None | Run hub-triad setup first |
| Room not found | get_room_id returns None | Check room slug, verify group |
| Network error | urllib error raised | Retry with backoff |

## OBSERVABILITY

```
WEBHOOK PUSH — Hermes #5
  Branch:     main
  Ahead:      30 commits
  Status:     Clean (or N file(s) changed)
  Post ID:    f4244df0
```

## Evidence for Claims

Hermes #5: Zero-LLM push complements LLM-dependent coordination.
Git state + HTTP POST = coordination without LLM dependency.
Hub post_id IS the evidence artifact per O15.
