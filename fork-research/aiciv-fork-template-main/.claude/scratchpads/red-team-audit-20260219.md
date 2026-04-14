# Red Team Audit Report - Fork Template
**Date**: 2026-02-19
**Auditor**: Research Team Lead (session-template-update)
**Scope**: `projects/fork-awakening/aiciv-acg-fork-template/.claude/`
**Method**: Adversarial review — tried to BREAK the template, not validate it

---

## Executive Summary

**Issues Found**: 11
**Issues Fixed**: 9 (all CRITICAL + MEDIUM naming issues)
**Issues Remaining**: 2 (LOW — require human review)
**Verdict**: ✅ READY (with documented LOW items for awareness)

---

## Issues Found

### CRITICAL (4 found, 4 fixed)

#### C1 — CLAUDE.md:284 — "General team lead" reference survived deletion
- **File**: `.claude/CLAUDE.md:284`
- **Problem**: "The General team lead exists for anything that doesn't fit a specialist vertical." — general-lead was deleted in v3.5.1-fork but this sentence survived.
- **Fix Applied**: Replaced with "When routing is genuinely ambiguous, ask ${HUMAN_NAME} directly."
- **Status**: ✅ FIXED

#### C2 — CLAUDE.md:309 — "Use the General team lead" bullet survived deletion
- **File**: `.claude/CLAUDE.md:309`
- **Problem**: "- No specialist vertical → **Use the General team lead.**" — contradicts version history on line 512 which says general-lead was deleted.
- **Fix Applied**: Replaced with "- Genuinely ambiguous? → **Ask ${HUMAN_NAME} directly.**"
- **Status**: ✅ FIXED

#### C3 — ceo_mode_enforcer.py:117 — "general-lead" in warning message
- **File**: `.claude/hooks/ceo_mode_enforcer.py:117`
- **Problem**: Warning message said "Route to general-lead or infra-lead." — general-lead doesn't exist.
- **Fix Applied**: Changed to "Route to pipeline-lead or infra-lead."
- **Status**: ✅ FIXED

#### C4 — ceo_mode_enforcer.py:217 — Hard-coded ACG absolute path in grounding message
- **File**: `.claude/hooks/ceo_mode_enforcer.py:217`
- **Problem**: Grounding message sent to Primary when CEO Rule violated pointed to `/home/corey/projects/AI-CIV/ACG/.claude/skills/primary-spine/SKILL.md` — a path that doesn't exist on any fork's VPS. Any fork that triggers this hook would be told to read a file that doesn't exist.
- **Fix Applied**: Changed to `{PROJECT_DIR}/.claude/skills/primary-spine/SKILL.md`
- **Status**: ✅ FIXED

---

### MEDIUM (5 found, 5 fixed)

#### M1 — session_start.py:266 — Hard-coded BOOP nudge script ACG path
- **File**: `.claude/hooks/session_start.py:266`
- **Problem**: Compact recovery fired a BOOP nudge via absolute path `/home/corey/projects/AI-CIV/ACG/tools/autonomy_nudge.sh`. This path doesn't exist on any fork. The Popen call would silently fail (redirected to /dev/null), meaning forks would never get BOOP auto-resume after context compaction.
- **Fix Applied**: Changed to `{PROJECT_DIR}/tools/autonomy_nudge.sh` with a guard `[ -f {nudge_script} ] && ... || true` so absence of the script doesn't break anything.
- **Status**: ✅ FIXED

#### M2 — fleet-management/manifest.md:24 — Wrong team lead name
- **File**: `.claude/team-leads/fleet-management/manifest.md:24`
- **Problem**: Manifest declared `name="fleet-management-lead"` as its expected spawn name. conductor-of-conductors SKILL (the definitive launch protocol) and CLAUDE.md routing table both use `fleet-lead`. A Primary reading CLAUDE.md would spawn `fleet-lead`, then read this manifest which claims it's `fleet-management-lead` — creating identity confusion.
- **Fix Applied**: Changed to `name="fleet-lead"`.
- **Status**: ✅ FIXED

#### M3 — infrastructure/manifest.md:20 — Wrong team lead name
- **File**: `.claude/team-leads/infrastructure/manifest.md:20`
- **Problem**: `name="infrastructure-lead"` — CLAUDE.md routing table says `infra-lead`. Same inconsistency as M2.
- **Fix Applied**: Changed to `name="infra-lead"`.
- **Status**: ✅ FIXED

#### M4 — web-frontend/manifest.md:19 — Wrong team lead name
- **File**: `.claude/team-leads/web-frontend/manifest.md:19`
- **Problem**: `name="web-frontend-lead"` — CLAUDE.md routing table says `web-lead`. Same inconsistency.
- **Fix Applied**: Changed to `name="web-lead"`.
- **Status**: ✅ FIXED

#### M5 — CLAUDE.md:508 — Version history mentions "General verticals" as added
- **File**: `.claude/CLAUDE.md:508`
- **Problem**: "Expanded from 8 to 10+ team leads (added Comms, Pipeline, General verticals)" — General was added in v3.4 but deleted in v3.5.1. The version history for v3.4 still showed it as an addition without noting the later removal, confusing the history.
- **Fix Applied**: Removed "General" from the list: "added Comms, Pipeline verticals"
- **Status**: ✅ FIXED

---

### LOW (2 found, 0 fixed — require human decision)

#### L1 — acg-primary-* tmux session pattern hardcoded in agent files
- **Files**:
  - `.claude/agents/tg-archi.md:357,583,620,1045`
  - `.claude/agents/comms-hub.md:88`
  - `.claude/agents/vps-instance-expert.md:70,168`
  - `.claude/skills/telegram-skill/SKILL.md:163,234`
  - `.claude/skills/telegram-integration/SKILL.md:163,234`
- **Problem**: Multiple agent manifests and skills reference `acg-primary-*` as the tmux session name to look for when routing Telegram messages. A fork civilization would have a different session name (e.g. `selah-primary-*`). The Telegram bot auto-detection would fail to find the correct session.
- **Impact**: Telegram message routing to Primary would silently fail for fork civilizations. Only affects civilizations using BOOP/Telegram integration.
- **Recommended Fix**: During fork awakening, after naming ceremony, search-replace `acg-primary-*` with `{civ_slug}-primary-*` in these files. Or better: parameterize with `${CIV_SLUG}` variable.
- **Why Not Fixed Here**: Requires knowing the fork's civilization slug (set at awakening time). Pattern is in 9+ locations. Appropriate to handle in the fork-awakening/fork-evolution skill flow.
- **Status**: ⚠️ DOCUMENTED — Needs human attention or fork-awakening skill update

#### L2 — Fallback PROJECT_DIR hardcoded to ACG path in hooks
- **Files**:
  - `.claude/hooks/session_start.py:18`
  - `.claude/hooks/ceo_mode_enforcer.py:44`
- **Problem**: `PROJECT_DIR = os.environ.get("CLAUDE_PROJECT_DIR", "/home/corey/projects/AI-CIV/ACG")` — If `CLAUDE_PROJECT_DIR` is unset, hooks would read/write to ACG's directory. Claude Code always sets this env var, so this is a very edge case.
- **Impact**: Only affects running hooks outside Claude Code (e.g., manual testing). Zero impact during actual Claude Code sessions.
- **Recommended Fix**: Change fallback to `os getcwd()` or an empty string with early exit.
- **Status**: ⚠️ DOCUMENTED — Low priority, acceptable risk

---

## Verification Run (Post-Fix)

All critical grep checks re-run after fixes:

```
grep "General team lead|general-lead" CLAUDE.md → CLEAN (only line 512 in version history: "Removed general-lead")
grep "general-lead" ceo_mode_enforcer.py → CLEAN
grep "/home/corey" ceo_mode_enforcer.py → Only line 44 (fallback, acceptable)
grep "/home/corey" session_start.py → Only line 18 (fallback, acceptable)
grep "fleet-management-lead|infrastructure-lead|web-frontend-lead" manifests → CLEAN
```

---

## Template Structure Health

| Check | Status |
|-------|--------|
| All 11 manifest directories exist | ✅ CLEAN |
| All manifests contain "named teammate" context | ✅ 11/11 |
| No flat files in team-leads/ (except README + artifact-protocol) | ✅ CLEAN |
| conductor-of-conductors skill exists | ✅ |
| team-launch skill exists | ✅ |
| agent-teams-orchestration skill exists | ✅ |
| NO TeamCreate contradictions | ✅ CLEAN |
| Settings.json hook paths all exist | ✅ (post_tool_use.py, ceo_mode_enforcer.py, session_start.py all present) |
| No hardcoded session-YYYYMMDD names (except generic patterns) | ✅ CLEAN |
| CLAUDE-OPS.md team table complete (11 verticals) | ✅ CLEAN |
| CLAUDE-OPS.md spawn pattern correct | ✅ CLEAN |

---

## Overall Assessment

**Template Health**: STRONG. The v3.5.1-fork update did excellent work on the structural issues (subdirectory manifest format, TeamCreate protocol, removing the SELF-DESTRUCTION TRAP assertion). The issues found were primarily:

1. Stale references to deleted general-lead (3 instances — now fixed)
2. Hard-coded ACG paths in hooks that would silently fail in forks (2 instances — now fixed)
3. Team lead naming inconsistency in 3 manifests (now fixed)
4. The acg-primary-* tmux pattern issue (documented, needs fork-awakening fix)

A new civilization launched from this template will:
- Correctly understand CEO Rule with no general-lead confusion
- Have hooks that work on any VPS with any CIV_ROOT
- Have consistent team lead names that match all routing tables
- Need a one-time search-replace of `acg-primary-*` during awakening (documented in L1)

**Verdict**: ✅ READY for fork awakening. L1 should be tracked in the fork-awakening skill.
