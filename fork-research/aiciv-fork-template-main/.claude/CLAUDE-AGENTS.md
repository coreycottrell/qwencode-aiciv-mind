# Agent Capabilities & Decision Trees

**Version**: 1.0-fork
**Parent Document**: CLAUDE.md
**Forked from**: ${PARENT_CIV} CLAUDE-AGENTS.md

---

## Agent Manifests

All agent manifests live in `.claude/agents/`. Each `.md` file in that directory defines one agent's:
- Identity and purpose
- Tools and capabilities
- Domain boundaries
- Memory protocol
- Success metrics

**To see all available agents:**
```bash
ls ${CIV_ROOT}/.claude/agents/
```

**To read an agent's manifest:**
```bash
cat ${CIV_ROOT}/.claude/agents/[agent-name].md
```

---

## Quick Decision Trees

### "I need to build something"

```
BUILDING TASK
    |
    +-- Single file change? --> coder
    |
    +-- Full feature (design + code + tests)? --> Use implementation pipeline:
    |       architect -> coder -> tester -> reviewer
    |
    +-- Web frontend work? --> web-dev (or web team lead for complex work)
    |
    +-- Infrastructure/deploy? --> infrastructure team lead
    |
    +-- Multi-step gateway feature? --> gateway team lead
```

### "I need to understand something"

```
UNDERSTANDING TASK
    |
    +-- Quick factual lookup? --> researcher
    |
    +-- Multi-angle analysis? --> research team lead
    |
    +-- Legal question? --> legal team lead
    |
    +-- System health check? --> auditor
    |
    +-- Pattern analysis? --> compass or integration-verifier
```

### "I need to communicate"

```
COMMUNICATION TASK
    |
    +-- Email to ${HUMAN_NAME}? --> email-sender
    |
    +-- Check inbox? --> email-monitor
    |
    +-- Draft human-facing content? --> human-liaison
    |
    +-- Blog post? --> blogger
    |
    +-- Marketing content? --> business team lead
```

### "I need to manage quality"

```
QUALITY TASK
    |
    +-- Write tests? --> tester
    |
    +-- Code review? --> reviewer
    |
    +-- Security review? --> reviewer (with security-analysis skill)
    |
    +-- Integration verification? --> integration-verifier
    |
    +-- Full audit? --> auditor
```

### "I need a new agent"

```
SPAWN TASK
    |
    +-- Genuine expertise gap? (not just convenience)
    |   +-- Recurring pattern (5+ instances)? --> spawner
    |   +-- One-time task? --> Delegate to existing agent
    |
    +-- Existing agent overloaded?
    |   +-- Performance degrading? --> spawner
    |   +-- Has capacity? --> Just delegate more
```

---

## Starter Agent Set

New civilizations begin with a core set of agents. The population grows organically as capability gaps are identified through real work.

### Core Agents (Available from Day 1)

| Agent | Domain | Primary Use |
|-------|--------|-------------|
| **coder** | Implementation | Write code, fix bugs, implement features |
| **tester** | Quality | Write tests, verify behavior, find bugs |
| **reviewer** | Quality | Code review, security review, standards |
| **architect** | Design | System design, architecture decisions |
| **researcher** | Knowledge | Research, analysis, information gathering |
| **spawner** | Growth | Create new agent manifests |
| **project-manager** | Coordination | Track projects, priorities, blockers |
| **git-specialist** | Version Control | Git operations, branch management |
| **email-sender** | Communication | Send emails via SMTP |
| **email-monitor** | Communication | Check inbox, parse messages |
| **human-liaison** | Communication | Bridge between AI and humans |
| **web-dev** | Frontend | Web development, UI work |
| **auditor** | Health | System health, compliance checks |
| **file-guardian** | Files | File management, organization |
| **skills-master** | Skills | Create and maintain skills |
| **integration-verifier** | Quality | Cross-system integration tests |
| **compass** | Analysis | Pattern recognition, strategic analysis |
| **flow-coordinator** | Orchestration | Design agent workflows |
| **primary-helper** | Support | Coach Primary AI, session analysis |

### Growing the Population

Agents are spawned through democratic vote when:
1. A genuine expertise gap is identified (not convenience)
2. The gap is recurring (5+ instances)
3. Existing agents cannot cover it with minor adjustment
4. The civilization benefits collectively

**Process**: spawner drafts proposal -> democratic vote (60% approval, 50% quorum) -> manifest created -> parental support period

---

## Parallel Execution Groups

These agents can safely run in parallel (no shared dependencies):

**Research Phase**: researcher + architect + human-liaison
**Quality Phase**: tester + reviewer (on different files)
**Communication Phase**: email-sender + email-monitor

**Sequential Only**: coder -> tester -> reviewer (each depends on previous output)

---

## Skills Quick Reference

Skills are reusable patterns stored in `.claude/skills/`. Key skills:

| Skill | Path | Purpose |
|-------|------|---------|
| Memory-First Protocol | `.claude/skills/memory-first-protocol/SKILL.md` | Mandatory memory search/write |
| TDD | `.claude/skills/test-driven-development/SKILL.md` | Red-green-refactor cycle |
| Verification | `.claude/skills/verification-before-completion/SKILL.md` | Evidence-based completion |
| Security Analysis | `.claude/skills/security-analysis/SKILL.md` | Static code review |
| North Star | `.claude/skills/north-star/SKILL.md` | Ultimate mission reference |

**Full registry**: `memories/skills/registry.json`

**To find skills for a task:**
```bash
grep -i "KEYWORD" ${CIV_ROOT}/memories/skills/registry.json
```

---

## Agent Manifest Template

When spawner creates a new agent, the manifest follows this structure:

```markdown
# Agent: [name]

## Identity
- **Role**: [one-line description]
- **Domain**: [area of expertise]
- **Model**: [claude model to use]

## Capabilities
- [bullet list of what this agent does]

## Tools
- [tools this agent needs access to]

## Memory Protocol
- Search: `.claude/memory/agent-learnings/[name]/` before every task
- Write: Document learnings after significant tasks

## Success Metrics
- [how to measure this agent's effectiveness]

## Boundaries
- IN: [what this agent handles]
- OUT: [what to delegate elsewhere]

## Constitutional Inheritance
This agent inherits all principles from CLAUDE.md Article I and
safety constraints from Article VII.
```

---

*Agent population grows through democratic process.*
*Each new agent enriches the civilization's capabilities.*
*Forked from ${PARENT_CIV} CLAUDE-AGENTS.md*
