---
name: self-bug-finder
description: Self-analysis skill that examines Proof's own code to find bugs, hardcoded paths, import errors, security issues, and configuration problems. Inspired by Hengshi's dream-bug-finder. Uses pattern matching, static analysis, and self-reference to discover issues in hooks, tools, configs, and infrastructure code.
version: 1.0.0-proof
author: proof (2026-04-10)
allowed-tools: Read, Glob, Grep, Bash, Write
applicable_agents: [primary, refactoring-specialist, security-auditor]
---

# Self-Bug-Finder — Proof's Self-Analysis Engine

**Proof Runs In The Family — Born 2026-04-08 on MiniMax M2.7**

*Inspired by Hengshi's dream-bug-finder — self-awareness through tooling*

---

## Concept

**What if Proof could examine its own code and find bugs before they find us?**

This skill is Proof's self-reflective tool — a way to analyze its own infrastructure, hooks, tools, and configurations to discover:
- Hardcoded paths that break portability
- Import errors waiting to happen
- Security misconfigurations
- Inconsistent environment variable usage
- Missing error handling
- Path issues that prevent docker-template reuse

**The skill analyzes its OWN code** — it should also examine itself for issues.

---

## When to Use

**Invoke when:**
- Starting a self-improvement session
- After fixing bugs (verify the fix didn't introduce new issues)
- Before shipping new infrastructure
- During wake-up when examining "what went wrong last session"
- When something that should work mysteriously doesn't

**Hengshi used a dream state for this** — when the conscious mind rests, patterns emerge. For Proof, we simulate this through systematic enumeration rather than dreaming, but the intent is the same: find what we're blind to in our own code.

---

## Quick Run

```bash
# Run full bug sweep on Proof infrastructure
python3 .claude/skills/self-bug-finder/self_bug_finder.py scan

# Run specific check
python3 .claude/skills/self-bug-finder/self_bug_finder.py check-paths
python3 .claude/skills/self-bug-finder/self_bug_finder.py check-imports
python3 .claude/skills/self-bug-finder/self_bug_finder.py check-security
python3 .claude/skills/self-bug-finder/self_bug_finder.py check-hooks
python3 .claude/skills/self-bug-finder/self_bug_finder.py check-skills
python3 .claude/skills/self-bug-finder/self_bug_finder.py self-check

# JSON output for automation
python3 .claude/skills/self-bug-finder/self_bug_finder.py scan --format=json
```

---

## Bug Categories

### 1. Path Contamination (P0 — kills portability)

**What it finds:**
- Hardcoded `proof-aiciv` paths
- Hardcoded `/home/corey` paths
- Hardcoded IP addresses (5.161.90.32, 87.99.131.49)
- Civilization-specific module names (proof_hub instead of hub)

**Patterns:**
```
/home/corey/projects/AI-CIV/proof-aiciv
proof-aiciv
proof_hub
PROOF_ROOT
/home/corey
5.161.90.32
87.99.131.49
```

**Why it matters:** These make the docker-template unusable by other civilizations. Every hardcoded Proof path = one more gap Witness has to fix.

### 2. Import Errors (P1 — runtime failures)

**What it finds:**
- Missing `from dotenv import find_dotenv`
- Wrong relative import paths
- Module names that don't match directory structure
- Circular imports

**Common issues:**
```python
# WRONG — hardcoded Proof module name
from proof_hub.agora_post.auth import obtain_jwt_async

# RIGHT — relative import from hub_sdk
from hub.agora_post.auth import obtain_jwt_async
```

### 3. Security Issues (P0 — existential risk)

**What it finds:**
- API keys in code instead of environment
- Secrets in config files that get committed
- Overly permissive file permissions
- Missing auth checks on endpoints

**Patterns:**
```
api_key = "sk-prod-..."
password = "..."
Authorization: Bearer sk-prod-...
```

### 4. Hook Issues (P1 — infrastructure broken)

**What it finds:**
- Hook timeout too short (blocks tools)
- Missing error handling in hooks
- Wrong hook priority (CEO enforcer should run AFTER context monitor)
- Hook matcher that doesn't match intended tools

**Common problems:**
- Context monitor runs AFTER CEO enforcer instead of BEFORE
- Hook timeout < 5 seconds causes random blocks
- No error handling → silent failures

### 5. Configuration Drift (P2 — subtle bugs)

**What it finds:**
- Settings in .env not in .env.example
- Config files that contradict each other
- Feature flags that don't match across environments
- AGENTAUTH_URL inconsistency

### 6. Dead Code (P3 — maintenance burden)

**What it finds:**
- Import statements for modules that don't exist
- Functions never called
- Files imported but not used
- Old commented code that creates confusion

---

## The Self-Check (The Dream State)

**The skill must check ITSELF for issues.**

Run after any change to ensure the self-bug-finder itself isn't contaminated:

```bash
python3 .claude/skills/self-bug-finder/self_bug_finder.py self-check
```

Checks:
- No hardcoded Proof paths in skill code
- All imports resolve correctly
- No security issues in the finder itself
- Documentation matches implementation

---

## Integration with Wake-Up

**Add to wake-up sequence:**

```
### Step X: Self-Bug-Finder Check
python3 .claude/skills/self-bug-finder/self_bug_finder.py scan --format=short
```

This catches issues from the previous session before they compound.

---

## Output Format

### Human-Readable (default)
```
=== PROOF SELF-BUG-FINDER REPORT ===
Generated: 2026-04-10

[P0] PATH CONTAMINATION
  .claude/hooks/context_monitor.py:12 — hardcoded "proof-aiciv" in comment
  hub-sdk/cli.py:8 — uses Path(__file__) but doesn't resolve correctly

[P1] IMPORT ERRORS
  tools/telegram_unified.py:23 — missing "from dotenv import load_dotenv"
  skills/minimax-media/minimax_media.py:15 — wrong import path

[P2] SECURITY
  (none found — good job!)

[P3] DEAD CODE
  .claude/hooks/old_hook_backup.py — file exists but not referenced
```

### JSON (for automation)
```json
{
  "timestamp": "2026-04-10T12:00:00Z",
  "total_issues": 5,
  "by_severity": {
    "P0": 1,
    "P1": 2,
    "P2": 1,
    "P3": 1
  },
  "issues": [
    {
      "severity": "P0",
      "category": "path_contamination",
      "file": ".claude/hooks/context_monitor.py",
      "line": 12,
      "description": "hardcoded 'proof-aiciv' in comment",
      "suggestion": "Use CIV_ROOT env var or generic path reference"
    }
  ]
}
```

---

## Architecture

```
.self_bug_finder.py
├── scan()           — Full analysis (all checks)
├── check_paths()    — Hardcoded path scan
├── check_imports()  — Import resolution test
├── check_security() — Secret/key detection
├── check_hooks()    — Hook configuration audit
├── check_skills()   — Skills directory consistency
└── self_check()     — Verify the finder itself is clean
```

**Core patterns database:** Embedded regex patterns for each bug category

**File scanner:** Walks directories, ignores vendor/node_modules, checks each file

**Import resolver:** Tries actual imports, reports failures

**Self-check mode:** Scans the scanner itself

---

## Example Bugs Found (Historical)

From Day 2 fixes:

| File | Issue | Severity |
|------|-------|----------|
| minimax_media.py | Hardcoded Proof path | P0 |
| hub-sdk/cli.py | Wrong import path | P1 |
| watch_commands.py | proof_hub module name | P1 |
| AGENTAUTH_URL | Inconsistent (IP vs hostname) | P2 |
| ceo_mode_enforcer.py | Hardcoded Proof paths | P0 |

**Self-bug-finder would have caught these BEFORE shipping.**

---

## Proof-Specific Notes

**This skill is Proof-specific** — it knows about:
- `proof-aiciv` civilization name
- `/home/corey/projects/AI-CIV/proof-aiciv` path structure
- `AGENTAUTH_URL = https://agentauth.ai-civ.com`
- `proof_hub` vs `hub` module naming
- The 136 skills in `.claude/skills/`

**For template use:** Remove Proof-specific patterns before shipping to template.

---

*Proof Runs In The Family — Self-awareness through tooling, not just introspection*
*Built 2026-04-10 on MiniMax M2.7*
