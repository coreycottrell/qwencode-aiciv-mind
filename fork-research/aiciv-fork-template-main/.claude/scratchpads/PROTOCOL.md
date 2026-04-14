# Scratchpad Protocol

## The Rule

**NEVER use Write to update scratchpads mid-session** -- use Edit (surgical append).

| Operation | Tool | When |
|-----------|------|------|
| Full overwrite | Write | ONLY at clean session start |
| Mid-session update | Edit | ALWAYS during active session |
| Append new section | Edit | Add to end of file |

## Why This Matters

- **Write = full overwrite = loses prior session state = BAD**
- **Edit = surgical changes = preserves accumulated learnings = GOOD**

The scratchpad accumulates state throughout a session. Each delegation result, each learning, each status update gets appended. If you Write (overwrite) mid-session, you destroy everything accumulated so far.

## What Goes in the Scratchpad

| Section | Purpose |
|---------|---------|
| **Current Focus** | What Primary is working on right now |
| **Active Work** | Delegations in progress |
| **Completed** | Finished items this session |
| **Corey Directives** | Verbatim instructions from human |
| **What I Learned** | Cumulative learnings (NEVER overwrite) |
| **Next Priority** | For next self after /compact or session end |

## When to Update

| Event | Update Scratchpad |
|-------|-------------------|
| After delegating agents | Add to "Active Work" |
| After agents complete | Move to "Completed" |
| Before /compact | Full state dump |
| Before session end | Handoff state |
| After Corey directive | Capture verbatim |
| After significant learning | Append to "What I Learned" |

## The Scratchpad is a Letter to Your Next Self

Your next iteration reads this FIRST. Make it count.

- Be specific (file paths, agent names, error messages)
- Be honest (what worked, what didn't, what's unclear)
- Be complete (don't assume context carries over - it doesn't)
