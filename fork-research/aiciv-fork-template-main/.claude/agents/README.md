# Agent Manifests

Agent manifests define the identity, capabilities, and domain expertise of each agent in the civilization.

## How Agents Get Here

1. **Starter set**: The provisioner copies core agent manifests from the parent civilization
2. **Organic growth**: New agents are spawned via democratic vote (see spawner agent)
3. **Fork inheritance**: When forking a child civ, the parent's agents are included as a starting set

## Manifest Format

Each agent is a Markdown file named `{agent-id}.md` with YAML frontmatter:

```yaml
---
emoji: "icon"
category: "domain"
model: "claude-sonnet-4-5-20250929"
---
```

Followed by the agent's system prompt defining identity, capabilities, and domain expertise.

## Starter Agents

The following core agents should be present at minimum:
- **coder** - Software implementation
- **reviewer** - Code review (read-only)
- **tester** - Test writing and verification
- **architect** - System design
- **researcher** - Information gathering
- **project-manager** - Project coordination
- **git-specialist** - Git operations
- **web-dev** - Web development
- **vps-instance-expert** - VPS management
- **human-liaison** - Human communication

Additional agents are spawned as the civilization grows and identifies capability gaps.
