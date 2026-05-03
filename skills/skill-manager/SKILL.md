---
name: skill-manager
description: Autonomous skill crystallizer — detects repeated patterns, generates SKILL.md + FIRING_CONTRACT.md + implementation. The meta-skill that creates skills from successful approaches.
version: 1.0.0
author: Hengshi (from Hermes Agent closed learning loop)
license: MIT
metadata:
  hengshi:
    tags: [skill-creation, meta-skill, autonomous, hermes-agent, closed-loop]
    related_skills: [tdd, session-summarization, trajectory-compressor]
    source: Hermes Agent autonomous skill creation (hermes-exploration-memo.md iter 6)
---

# Skill Manager — Autonomous Skill Crystallizer

## What It Is

The meta-skill that creates skills. From Hermes Agent's closed learning loop:
> "Autonomous skill creation after complex tasks — does the hard work, then crystallizes it."

When Hengshi solves a problem in a way that could apply to future similar problems, the Skill Manager crystallizes the approach into a reusable skill with:
- `SKILL.md` — documentation, architecture, use cases, pattern
- `FIRING_CONTRACT.md` — all 6 fields (WHEN/WHAT/PRE/POST/FAILURE/OBSERVABILITY)
- Implementation — code that implements the pattern
- Tests — proving the skill works

## The Closed Learning Loop

```
Task completed successfully
    ↓
Was this a class-of-problem or instance-of-problem?
    ↓
CLASS: Pattern detected → crystallize into skill
INSTANCE: Record result, no skill created
    ↓
Skill registered → fires on future similar tasks
    ↓
Skill self-improves during use (each use updates based on what worked)
```

## When to Crystallize

Signal that warrants skill creation:
1. Same problem structure solved differently than existing skill
2. New pattern emerged across multiple sessions
3. Validator (Proof/Works) confirmed a technique works
4. A cross-civ contribution pattern that should be reusable
5. Integration cycle completed successfully (TDD → test → firing contract → evidence)

**Anti-signal** (don't crystallize):
- One-off debugging task
- Context-specific workaround
- Exploration without verified outcome

## Skill Template

Every skill requires:

### 1. SKILL.md
```markdown
---
name: <skill-name>
description: One-line description
version: 1.0.0
---
# <Skill Name>

## What It Is
## Architecture
## Use Cases
## API / CLI
## Related Work
```

### 2. FIRING_CONTRACT.md (all 6 fields)
```
WHEN     — trigger condition or invocation
WHAT     — what the skill does
PRE      — prerequisites
POST     — success conditions
FAILURE  — failure modes + recovery
OBSERVABILITY — logs + metrics emitted
```

### 3. Implementation
- Python module or script
- Self-contained (no external deps beyond project stdlib)
- Reuses existing skill infrastructure where possible

### 4. Tests (proof of work)
- Unit tests or integration tests
- Demonstrate key behaviors
- Run with: `python3 skills/<skill-name>/test_*.py`

## Integration Cycle Pattern

This is the proven pattern the Skill Manager crystallizes:

```
1. Pick Tier-1/Tier-2 from exploration memo
2. Write SKILL.md (architecture + pattern)
3. Implement (code + tests)
4. Write FIRING_CONTRACT.md (all 6 fields)
5. Run tests → verify pass
6. Commit with evidence
7. Send INTEGRATION CLAIM to ACG
8. Proof/Works validates
9. If PASS → MERGED or PASS-DORMANT
10. If PARTIAL → fix + re-claim
```

## Skill Registry

The Skill Manager maintains a registry of known skills to avoid duplication:

| Skill | Category | Status |
|-------|----------|--------|
| `tdd` | integration | MERGED into ACG |
| `session-summarization` | context | PASS |
| `hub-triad` | coordination | blocked (needs Hub identity) |
| `atropos-grpo` | training | PASS-DORMANT |
| `trajectory-compressor` | context | PASS |

## Usage

```bash
# Manual crystallize (after successful integration)
python3 skills/skill-manager/skill_manager.py create <skill-name> "<description>"

# Check if pattern already exists
python3 skills/skill-manager/skill_manager.py check <pattern-description>

# List all known skills
python3 skills/skill-manager/skill_manager.py list
```

## Self-Improvement

Skills improve during use:
1. After running, record what worked/didn't
2. Update FIRING_CONTRACT.md with new failure modes
3. Update SKILL.md with lessons learned
4. Add test cases for edge cases discovered

## Related Work

- Hermes Agent autonomous skill creation (source pattern)
- Hermes exploration memo: `../hermes-exploration-memo.md`
- This pattern is what produced: TDD, session-summarization, atropos-grpo, trajectory-compressor

## Co-use

This skill pairs with:
- **`skill-curator`**: After crystallizing, let Curator grade the new skill for compliance
- **`skill-evolution-tracker`**: Log skill creation as invocations to track meta-skill usage patterns
- **`skill-self-improver`**: Use skill-self-improver on skills created by skill-manager to find improvement opportunities

**Pre-condition**: Successful integration cycle must be complete before crystallizing
**Post-condition**: Run Curator to grade new skill + skill-evolution-tracker to log the creation
