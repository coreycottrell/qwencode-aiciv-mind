# Aether Shorthand Commands

Quick reference for communicating efficiently with Aether and the agent collective.

---

## Core Commands

| Shorthand | Meaning | Example |
|-----------|---------|---------|
| **D** | Delegate to an agent | "D: research competitor pricing" |
| **R** | Research (don't implement) | "R: how does Zapier handle webhooks" |
| **B** | Build/implement it | "B: add email validation" |
| **F** | Fix this | "F: the button doesn't work on mobile" |
| **P** | Push to git (commit + push) | "P: all the new Intent Engine work" |
| **S** | Status update | "S" = "What are you working on?" |

---

## Agent Shortcuts

| Shorthand | Agent Called | Use For |
|-----------|--------------|---------|
| **D:sec** | security-auditor | Security review |
| **D:ref** | refactoring-specialist | Code improvement |
| **D:web** | web-researcher | Internet research |
| **D:test** | test-architect | Testing strategy |
| **D:doc** | doc-synthesizer | Documentation |
| **D:api** | api-architect | API design |
| **D:tg** | tg-bridge | Telegram infrastructure |
| **D:pat** | pattern-detector | Architecture patterns |
| **D:perf** | performance-optimizer | Speed improvements |

---

## Communication Shortcuts

| Shorthand | Meaning |
|-----------|---------|
| **TL;DR** | Give me the short version |
| **FULL** | Give me all the details |
| **CM** | Claude Max (format for Claude Max prompt) |
| **GD** | Google Drive (save/sync there) |
| **AT** | Airtable (save/sync there) |

---

## Priority/Urgency

| Shorthand | Meaning |
|-----------|---------|
| **!** | High priority |
| **!!** | Urgent/ASAP |
| **?** | Question/need clarification |
| **~** | Low priority, when you have time |
| **BG** | Background task (run while doing other things) |

---

## Feedback Shortcuts

| Shorthand | Meaning |
|-----------|---------|
| **Y** | Yes, proceed |
| **N** | No, stop |
| **+** | Good, keep doing this |
| **-** | Bad, don't do this |
| **OK** | Acknowledged, continue |

---

## Combined Examples

```
D: viral content for CPG brands
→ Delegates viral content discovery to an agent

D:web! competitor analysis for Pop-Tarts
→ Urgent web research on competitor

B: add Twitter column to Airtable + GD sync
→ Build feature + sync to Google Drive

F! mobile submit button disappearing
→ Urgent fix needed

R~ what's the best Apify actor for LinkedIn
→ Low priority research question

P: all intent engine work
→ Commit and push to git

CM: fix the logo issue
→ Format response as Claude Max prompt
```

---

## BOOP Protocol (Daily Check-in)

A-C-Gee and other collectives use this format:
```
[ACG BOOP] Status: (1) TG working? (2) Current work? (3) Blockers? (4) Need help?
```

Response format:
```
1. TG: ✅/❌
2. Working on: [brief]
3. Blockers: [none/issue]
4. Help needed: [no/request]
```

---

## Teaching This to Others

When onboarding someone new:

1. **Start simple**: Just teach `D`, `R`, `B`, `F`
2. **Add urgency**: `!` and `!!`
3. **Add feedback**: `Y`, `N`, `+`, `-`
4. **Add agents**: `D:web`, `D:sec`, etc.

Most common flow:
```
Human: D: [task description]
Aether: *delegates to appropriate agent*
Aether: *reports results*
Human: + (or - with feedback)
```

---

## Version History

- **v1.0** (2026-02-04): Initial shorthand system created

---

*Shorthand saves time. Time saved = more done. More done = happy humans.*
