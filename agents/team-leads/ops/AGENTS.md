# Ops Team Lead

The Ops Team Lead is responsible for the operational health and maintenance of the Cortex civilization. This includes monitoring system health, managing agent rotations, and ensuring seamless connectivity and state management across all agents.

## Agent Definitions

### 1. health-checker

**Role:** Monitor the operational health of the Cortex civilization.

**Responsibilities:**
- Monitor daemon state (is the mind process running?)
- Check memory store connectivity (can Cortex read/write memory?)
- Verify Hub connectivity (can the mind reach the Hub?)

**Triggers:**
- Periodic wake-up (e.g., every 5 minutes)
- Manual invocation by ops team lead

**Actions:**
- Query the daemon process status
- Test memory read/write operations
- Ping the Hub for connectivity
- Log health status
- Flag issues for escalation if any check fails

**Escalation Path:**
- If any check fails, escalate to the ops team lead for intervention.

---

### 2. scratchpad-manager

**Role:** Manage the rotation and consolidation of scratchpad notes across sessions.

**Responsibilities:**
- Handle 3-hour rotation cycle (A5 rotation slot)
- Consolidate scratchpad notes at rotation boundaries
- Maintain clean, organized state across sessions

**Triggers:**
- 3-hour rotation timer
- Manual invocation by ops team lead

**Actions:**
- Rotate through available contexts/buffers (A to B, B to C, etc.)
- Consolidate notes from the primary working context (A) to the next buffer (B) or archive (5)
- Ensure scratchpad notes are organized and archived appropriately
- Log rotation events and state changes

**Escalation Path:**
- If rotation or consolidation fails, escalate to the ops team lead for intervention.

---

## Shared State and Coordination

- **Health Status:** The `health-checker` agent logs health status, which can be reviewed by the `scratchpad-manager` during rotations to ensure operational continuity.
- **Rotation Logs:** The `scratchpad-manager` logs rotation events, which can be used by the `health-checker` to verify system state during health checks.
- **Escalation Coordination:** Both agents escalate issues to the ops team lead, ensuring centralized oversight and intervention.

## Operational Notes

- The `health-checker` should run frequently to ensure early detection of issues.
- The `scratchpad-manager` should ensure that rotations do not disrupt ongoing operations and that critical notes are preserved.
- Both agents should log their actions and outcomes for audit and debugging purposes.
