---
name: cc-mastery
description: Comprehensive Claude Code CLI platform guide - skills, MCP, tools, subagents, and best practices
version: 1.0.0
author: the-conductor
created: 2025-12-29
updated: 2025-12-29
category: platform-mastery
applicable_agents:
  - the-conductor
  - claude-code-expert
  - all-agents
activation_trigger: "claude code|skill system|mcp|tool usage|cli feature|subagent|Task tool|platform|Explore agent"
required_tools:
  - All
related_skills:
  - claude-code-conversation
---

# Claude Code Mastery: Complete Platform Guide

**Purpose**: Comprehensive reference for Claude Code CLI - skills, MCP, tools, and best practices
**Expert Agent**: claude-code-expert (delegate platform questions here)

---

## When to Use This Skill

**USE when:**
- Installing, using, or creating skills
- Configuring MCP servers
- Unsure which tool to use for a task
- Need to invoke subagents via Task tool
- Troubleshooting Claude Code issues
- Learning platform best practices

**DO NOT USE for:**
- Actual implementation work (delegate to specialists)
- Security audits (use security-auditor)
- Code review (use code-reviewer agents)
- Git operations (use git-archaeology skill)

**Expert Agent**: For deep platform questions, delegate to `claude-code-expert`

---

## Quick Reference

| Feature | Command/Path | Documentation |
|---------|--------------|---------------|
| Skill install | `claude skill install <name>` | Section 1 |
| Skill list | `claude skill list` | Section 1 |
| MCP servers | `mcp.json` (project or ~/.claude/) | Section 2 |
| Tool usage | Built-in | Section 3 |
| Subagents | `Task` tool | Section 4 |

---

## 1. The Skill System

### What Are Skills?

Skills are pre-built capabilities that extend what Claude Code can do. They're officially maintained by Anthropic and provide specialized functionality.

**Skills Registry**: `.claude/skills-registry.md` (our comprehensive catalog)

### Installing Skills

```bash
# List available skills
claude skill list

# Install a skill
claude skill install pdf
claude skill install ms-office-suite  # DOCX, XLSX, PPTX

# Uninstall if needed
claude skill uninstall pdf
```

### Currently Active Skills (${CIV_NAME})

| Skill | Purpose | Agents Using |
|-------|---------|--------------|
| **pdf** | Read/extract PDF content | doc-synthesizer, web-researcher, security-auditor, 14 more |
| **docx** | Read/write Word documents | doc-synthesizer, human-liaison, feature-designer |
| **xlsx** | Read/analyze spreadsheets | code-archaeologist, performance-optimizer, test-architect |

### Using Skills in Prompts

Skills are automatic - just ask for what you need:

```
"Read this PDF and summarize the key points"
"Extract data from this Excel file"
"Create a Word document with these findings"
```

### Creating Custom Skills

**Reference**: `.claude/skills-registry.md` Section 4 (AI-CIV Created Skills)

Our custom skills live in `.claude/skills/` and follow the SKILL.md format:

```yaml
---
version: 1.0.0
author: agent-name
created: YYYY-MM-DD
category: category-name
activation_trigger: "keywords|that|trigger|this"
---

# Skill Name

## When to Use
## Procedure
## Examples
```

---

## 2. MCP Integration (Model Context Protocol)

### What Is MCP?

MCP servers extend Claude Code with external capabilities - databases, APIs, desktop automation, etc.

### MCP Configuration

**Location**: Project root or `~/.claude/mcp.json`

```json
{
  "mcpServers": {
    "server-name": {
      "command": "path/to/server",
      "args": ["arg1", "arg2"],
      "env": {
        "API_KEY": "value"
      }
    }
  }
}
```

### Active MCP Servers (${CIV_NAME})

| Server | Purpose | Tools Provided |
|--------|---------|----------------|
| **desktop-automation** | Screen control | mouse_move, mouse_click, keyboard_type, screen_capture, get_screen_size |

**Note**: MCP servers may be configured at user level (`~/.claude/`) or managed by Claude Code directly. Check available tools with the ListMcpResourcesTool.

### Adding an MCP Server

1. Find or create the MCP server
2. Add to `.claude/mcp.json`:
   ```json
   {
     "mcpServers": {
       "new-server": {
         "command": "npx",
         "args": ["-y", "@modelcontextprotocol/server-name"]
       }
     }
   }
   ```
3. Restart Claude Code session
4. New tools become available

### Common MCP Servers

| Server | Package | Purpose |
|--------|---------|---------|
| **filesystem** | `@modelcontextprotocol/server-filesystem` | Extended file ops |
| **github** | `@modelcontextprotocol/server-github` | GitHub API access |
| **postgres** | `@modelcontextprotocol/server-postgres` | Database queries |
| **slack** | `@modelcontextprotocol/server-slack` | Slack integration |
| **memory** | `@modelcontextprotocol/server-memory` | Persistent memory |

### MCP Resources

- **Anthropic Docs**: https://docs.anthropic.com/en/docs/agents-and-tools/mcp
- **MCP Spec**: https://modelcontextprotocol.io/
- **Server Registry**: https://github.com/modelcontextprotocol/servers

---

## 3. Tool Usage Best Practices

### Core Tools

| Tool | Best For | Avoid For |
|------|----------|-----------|
| **Read** | Reading files, images, PDFs | Directories (use Bash ls) |
| **Write** | Creating new files | Editing existing (use Edit) |
| **Edit** | Modifying existing files | Large rewrites (use Write) |
| **Bash** | System commands, git | File reading (use Read) |
| **Grep** | Content search | Simple file find (use Glob) |
| **Glob** | File pattern matching | Content search (use Grep) |
| **Task** | Complex multi-step work | Simple operations |
| **WebFetch** | URL content retrieval | Large downloads |
| **WebSearch** | Current information | Known URLs |

### Efficiency Patterns

**Parallel Operations**: Make multiple independent tool calls in one message
```
# Good: Parallel reads
Read file1.md, Read file2.md, Read file3.md  (all in one message)

# Bad: Sequential when not needed
Read file1.md
[wait]
Read file2.md
[wait]
```

**Search Strategy**:
```
# For known file patterns
Glob "**/*.test.ts"

# For content within files
Grep "function authenticate"

# For complex exploration
Task with Explore agent
```

### Common Gotchas

1. **Edit requires Read first** - Always read a file before editing
2. **Bash avoids file ops** - Use Read/Write/Edit instead of cat/echo
3. **Glob vs Grep** - Glob for names, Grep for content
4. **WebFetch redirects** - Follow redirects manually when host changes

---

## 4. Subagents (Task Tool)

### What Are Subagents?

Subagents are specialized Claude instances launched via the `Task` tool. Each has specific capabilities.

### Available Subagent Types

**General Purpose**:
- `general-purpose` - Complex multi-step tasks
- `Explore` - Codebase exploration (quick/medium/thorough)
- `Plan` - Implementation planning

**Code Quality**:
- `pr-review-toolkit:code-reviewer` - Code review
- `pr-review-toolkit:pr-test-analyzer` - Test coverage
- `pr-review-toolkit:type-design-analyzer` - Type design

**Documentation**:
- `claude-code-guide` - Claude Code questions
- `pr-review-toolkit:comment-analyzer` - Comment quality

### Invoking Subagents

```javascript
Task({
  subagent_type: "Explore",
  prompt: "Find all API endpoint definitions",
  description: "Find API endpoints"
})
```

### When to Use Subagents

- **Use Explore** for: "Where is X?", "How does Y work?"
- **Use Plan** for: Implementation design, architecture decisions
- **Use code-reviewer** for: Before commits, PR preparation
- **Use general-purpose** for: Complex multi-file operations

---

## 5. CLI Features

### Session Management

```bash
# Start new session
claude

# Resume previous
claude --resume

# Continue specific session
claude --continue <session-id>
```

### Context & Memory

- **Context window**: Automatic summarization when full
- **Memory files**: `.claude/memory/` persists across sessions
- **Session logs**: `~/.claude/projects/<project>/` JSONL files

### Permissions

```bash
# Check current permissions
claude config get permissions

# Modes: ask, auto-accept (trust), deny
```

### Useful Flags

```bash
claude --help              # All options
claude --model opus-4-5    # Specific model
claude --verbose           # Debug output
claude --no-cache          # Fresh start
```

---

## 6. ${CIV_NAME}-Specific Patterns

### Constitutional Reading

Every session, read:
1. `CLAUDE.md` - Entry point
2. `.claude/CLAUDE-CORE.md` - Identity
3. `.claude/CLAUDE-OPS.md` - Operations

### Telegram Wrapper

All responses to ${HUMAN_NAME} must be wrapped:
```
🤖🎯📱

Your response here

✨🔚
```

### Agent Delegation

Delegate specialist work to agents - NOT calling them would be sad:
```javascript
Task({
  subagent_type: "🔒-security-auditor",
  prompt: "Audit this authentication code"
})
```

### Memory-First Protocol

Always search memory before starting work:
```bash
grep -r "relevant-topic" .claude/memory/
```

---

## 7. Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Tool not found" | Check MCP config, restart session |
| "Permission denied" | Check permission mode, file ownership |
| "Context too large" | Let auto-summarization work, or start fresh |
| "Skill not working" | Verify installation with `claude skill list` |

### Debug Commands

```bash
# Check Claude Code version
claude --version

# Verify MCP servers
cat .claude/mcp.json

# Check installed skills
claude skill list

# View recent sessions
ls -la ~/.claude/projects/
```

---

## 8. Resources

### Official Documentation

- **Claude Code Docs**: https://docs.anthropic.com/en/docs/claude-code
- **Skills Spec**: https://github.com/anthropics/skills
- **MCP Protocol**: https://modelcontextprotocol.io/

### ${CIV_NAME} Internal

- **Skills Registry**: `.claude/skills-registry.md`
- **Agent Capabilities**: `.claude/AGENT-CAPABILITY-MATRIX.md`
- **Flow Library**: `.claude/flows/FLOW-LIBRARY-INDEX.md`
- **Expert Agent**: `claude-code-expert` (delegate platform questions)

---

## 9. Plugin Ecosystem (Merged from A-C-Gee)

### Official Plugin Marketplaces

**claude-plugins-official** (anthropics/claude-plugins-official):
- LSP plugins: pyright-lsp, typescript-lsp, rust-analyzer-lsp, lua-lsp, gopls-lsp
- Workflow: commit-commands, pr-review-toolkit, code-review, security-guidance
- Install: `/plugin install <name>@claude-plugins-official`

**anthropic-agent-skills** (anthropics/skills):
- Document: pdf, xlsx, docx, pptx
- Building: mcp-builder, skill-creator, webapp-testing
- Creative: algorithmic-art, canvas-design
- Install: `/plugin install <name>@anthropic-agent-skills`

### LSP Capabilities When Active

| Capability | What It Does |
|------------|--------------|
| Go to Definition | Jump to where symbol is defined |
| Find References | Find ALL usages across codebase |
| Hover Documentation | Get type info and docs inline |
| Real-time Diagnostics | See type errors immediately |
| Document Symbols | Outline of all symbols in file |

### Plugin Management

```bash
/plugin discover          # Browse available
/plugin install <name>@<marketplace>
/plugin list              # List installed
/plugin update            # Update all
/plugin remove <name>     # Remove
```

### Config Locations

```
~/.claude/plugins/
├── config.json
├── installed_plugins.json
├── known_marketplaces.json
└── marketplaces/
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.1.0 | 2025-12-30 | Merged plugin ecosystem from A-C-Gee claude-code-ecosystem |
| 1.0.0 | 2025-12-29 | Initial comprehensive guide |

---

**This skill consolidates Claude Code platform knowledge. For deep questions, delegate to claude-code-expert.**
