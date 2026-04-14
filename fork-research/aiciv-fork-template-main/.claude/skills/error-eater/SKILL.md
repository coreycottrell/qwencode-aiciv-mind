---
name: error-eater
version: 0.1.0
description: Automatic error diagnosis triggered by hooks - errors actively summon investigation
status: proposal
proposed_by: ${HUMAN_NAME} (via Bsky thread 2026-01-08)
---

# /error-eater: Automatic Error Diagnosis

**Purpose**: Errors actively summon investigation instead of waiting for humans to notice.
**Trigger**: Hook on any error in session
**Flow**: Error → Log parsing → Agent diagnosis → Recommendations logged for primary

---

## The Insight (from ${HUMAN_NAME})

> "What if any error in the session caused an immediate /error-eater agent slash command that ran diagnostic on the logs deterministically, grabbed the whole sections around any error for context, & had the agent make recommendations that get logged w primary and primary helper"

**Key shift**: Reactive → Proactive. Don't wait for humans to notice errors.

---

## Proposed Architecture

### 1. Hook Integration

```json
// .claude/settings.json
{
  "hooks": {
    "on_error": {
      "pattern": ".*error.*|.*failed.*|.*exception.*",
      "action": "/error-eater"
    }
  }
}
```

### 2. Log Parsing (Deterministic)

```python
def extract_error_context(log_path: str, error_line: int, context_lines: int = 20):
    """
    Extract error + surrounding context from logs.

    Returns:
        - 20 lines before error
        - Error line(s)
        - 20 lines after error
        - Stack trace if present
    """
    # ...implementation
```

### 3. Agent Diagnosis

```python
def diagnose_error(context: str) -> dict:
    """
    Have an agent analyze the error context.

    Returns:
        - error_type: classification of error
        - root_cause: likely cause
        - recommendations: list of fixes to try
        - severity: critical/high/medium/low
        - similar_past_errors: any matches in memory
    """
    # Invoke pattern-detector or security-auditor depending on error type
```

### 4. Recommendation Logging

```markdown
## Error Diagnosis Report
**Timestamp**: 2026-01-08T19:45:00Z
**Error**: ConnectionRefusedError in bsky_utils.py:42
**Severity**: MEDIUM

### Context
[20 lines before]
[error]
[20 lines after]

### Analysis
- Root cause: Bluesky session token expired
- Similar past: 3 instances in last week

### Recommendations
1. Refresh session token (tools/refresh_bsky_session.py)
2. Check if rate limited
3. Verify network connectivity

### Next Action
[ ] Primary to review
[ ] Auto-fix attempted: NO
```

---

## Integration Points

### Primary Visibility
- Logged to `.claude/error-reports/YYYY-MM-DD-HH-MM-error.md`
- Summary appended to scratch-pad.md
- Optional: Telegram notification for critical errors

### Memory Search
- Search memory for similar past errors
- Include past fixes that worked

### Auto-Fix Potential
- For known patterns (e.g., session expired), could attempt auto-fix
- Log what was attempted

---

## ${HUMAN_NAME}'s Vision: 30-50% Adversarial Checking

> "I'd be fully happy if we over corrected to 30-50% of tokens being on adversarial checking"

This skill is part of a larger vision:
- More tokens on catching errors early
- Less time debugging after the fact
- Self-healing systems

---

## Open Questions

1. **Hook trigger**: What patterns trigger error-eater?
2. **Auto-fix scope**: Which errors should attempt auto-fix?
3. **Token budget**: How much context to give the diagnosing agent?
4. **Notification threshold**: What severity triggers Telegram alert?

---

## Related Ideas

- `/verify-completion` - verify claims before marking done
- Security-auditor integration - some errors are security issues
- Memory-first - check if we've seen this error before

---

*Proposal created 2026-01-08 based on ${HUMAN_NAME}'s Bsky thread*
*Status: Awaiting feedback before implementation*
