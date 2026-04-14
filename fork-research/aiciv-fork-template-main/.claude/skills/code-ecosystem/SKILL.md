---
name: code-ecosystem
description: Claude Code plugin ecosystem knowledge - LSP servers, installed plugins, available integrations. Use when checking what language servers or plugins are available.
---

# Claude Code Ecosystem Skill

**Owner**: skills-master (pending spawn) / Primary AI
**Last Updated**: 2025-12-26
**Claude Code Version**: 2.0.76

---

## Purpose

This skill provides comprehensive knowledge of Claude Code's plugin ecosystem, LSP integrations, and new features. All agents should reference this for understanding available tooling and capabilities.

---

## Current Installation Status

### LSP Servers (Backend)

| Server | Language | Status | Install Command |
|--------|----------|--------|-----------------|
| pyright | Python | ✅ INSTALLED | `npm install -g pyright` |
| rust-analyzer | Rust/Solana | ✅ INSTALLED | `rustup component add rust-analyzer` |
| typescript-language-server | TypeScript/JS | ✅ INSTALLED | `npm install -g typescript-language-server typescript` |
| lua-language-server | Lua (Luanti) | ⏳ Pending | `sudo snap install lua-language-server --classic` |

### Claude Code Plugins (Frontend)

| Plugin | Status | Purpose |
|--------|--------|---------|
| pyright-lsp | ✅ INSTALLED | Python type checking & code intel |
| typescript-lsp | ✅ INSTALLED | TypeScript/JS code intel |
| rust-analyzer-lsp | ✅ INSTALLED | Rust/Solana code intel |
| commit-commands | ✅ INSTALLED | /commit, /commit-push-pr |
| pr-review-toolkit | ✅ INSTALLED | PR analysis agents |
| code-review | ✅ INSTALLED | Code quality assistance |
| security-guidance | ✅ INSTALLED | Security analysis |

### Claude Code Plugins (Frontend)

**To install, run these `/plugin install` commands in Claude Code:**

#### Core LSP Plugins (PRIORITY)
```
/plugin install pyright-lsp@claude-plugins-official
/plugin install typescript-lsp@claude-plugins-official
/plugin install rust-analyzer-lsp@claude-plugins-official
/plugin install lua-lsp@claude-plugins-official
```

#### Workflow Plugins (RECOMMENDED)
```
/plugin install commit-commands@claude-plugins-official
/plugin install pr-review-toolkit@claude-plugins-official
/plugin install code-review@claude-plugins-official
/plugin install security-guidance@claude-plugins-official
```

#### Anthropic Skills (USEFUL)
```
/plugin install mcp-builder@anthropic-agent-skills
/plugin install skill-creator@anthropic-agent-skills
/plugin install pdf@anthropic-agent-skills
/plugin install xlsx@anthropic-agent-skills
```

---

## Available Marketplaces

### 1. claude-plugins-official (anthropics/claude-plugins-official)

**LSP Plugins:**
| Plugin | Language | Extensions | Use Case |
|--------|----------|------------|----------|
| `pyright-lsp` | Python | .py, .pyi | Type checking, code intel |
| `typescript-lsp` | TypeScript/JS | .ts, .tsx, .js, .jsx | Type checking, code intel |
| `rust-analyzer-lsp` | Rust | .rs | **Solana/Anchor development** |
| `lua-lsp` | Lua | .lua | **Luanti/Minetest modding** |
| `gopls-lsp` | Go | .go | Go development |
| `clangd-lsp` | C/C++ | .c, .cpp, .h | Systems programming |
| `csharp-lsp` | C# | .cs | .NET development |
| `jdtls-lsp` | Java | .java | Java development |
| `php-lsp` | PHP | .php | Web development |
| `swift-lsp` | Swift | .swift | Apple development |

**Workflow Plugins:**
| Plugin | Commands/Features | Use Case |
|--------|-------------------|----------|
| `commit-commands` | /commit, /commit-push-pr, /clean_gone | Git workflow automation |
| `pr-review-toolkit` | PR analyzers, code simplifier | Pull request review |
| `code-review` | Code review assistance | Quality gates |
| `security-guidance` | Security analysis | Vulnerability detection |
| `feature-dev` | Feature development | Structured feature work |
| `agent-sdk-dev` | /new-sdk-app | Agent SDK development |
| `hookify` | Hook management | Automation |
| `plugin-dev` | Plugin development | Creating new plugins |

**Output Styles:**
| Plugin | Effect |
|--------|--------|
| `explanatory-output-style` | Detailed explanations |
| `learning-output-style` | Educational focus |
| `ralph-wiggum` | Fun/casual style |

### 2. anthropic-agent-skills (anthropics/skills)

| Skill | Purpose | Use Case |
|-------|---------|----------|
| `mcp-builder` | Build MCP servers | Creating integrations |
| `skill-creator` | Create new skills | Expanding capabilities |
| `pdf` | PDF handling | Document processing |
| `xlsx` | Excel files | Data analysis |
| `docx` | Word documents | Document creation |
| `pptx` | PowerPoint | Presentations |
| `webapp-testing` | Web app testing | QA automation |
| `web-artifacts-builder` | Web building | Frontend development |
| `frontend-design` | UI/UX design | Design work |
| `canvas-design` | Canvas graphics | Visual design |
| `algorithmic-art` | Generative art | Creative projects |
| `brand-guidelines` | Branding | Marketing |
| `internal-comms` | Communications | Team coordination |
| `doc-coauthoring` | Collaboration | Document editing |
| `slack-gif-creator` | Slack GIFs | Fun/engagement |
| `theme-factory` | Theme creation | Customization |

---

## LSP Capabilities

When LSP plugins are active, Claude Code gains:

### 1. Go to Definition
- Jump to where any function/class/variable is defined
- Works across files in the project
- Essential for understanding large codebases

### 2. Find References
- Find ALL usages of a symbol across the codebase
- Critical for refactoring safely
- Shows impact of changes

### 3. Hover Documentation
- Get type info and docs by hovering
- Inline parameter hints
- Quick understanding without leaving context

### 4. Real-time Diagnostics
- See type errors immediately after edits
- Catch bugs before running code
- Continuous validation

### 5. Document Symbols
- Get outline of all symbols in a file
- Quick navigation within files
- Structure overview

---

## A-C-Gee Relevant Plugins

### For Solana/ArcX Development
- **rust-analyzer-lsp**: Essential for Anchor programs
- **typescript-lsp**: For frontend and Node.js
- **security-guidance**: Smart contract security

### For Luanti/Minetest
- **lua-lsp**: Lua scripting intelligence
- **code-review**: Quality gates for mods

### For Python Tools
- **pyright-lsp**: Type checking our tools
- **code-review**: Quality assurance

### For Web Development
- **typescript-lsp**: Full-stack TypeScript
- **frontend-design**: UI/UX assistance
- **webapp-testing**: Testing automation

### For Workflow
- **commit-commands**: Streamlined git
- **pr-review-toolkit**: PR quality
- **mcp-builder**: Creating integrations

---

## Version History & Updates

### v2.0.76 (Current)
- All features from 2.0.55-2.0.75 available
- LSP fully integrated
- Plugin marketplace stable

### v2.0.74 (LSP Release)
- Added LSP tool for code intelligence
- Go-to-definition, find references, hover
- Terminal setup for Kitty, Alacritty, Zed, Warp

### v2.0.73 (Plugin Enhancements)
- Plugin search filtering
- Per-marketplace auto-update control
- Output style sharing

### v2.0.72 (Browser Integration)
- Claude in Chrome (Beta)
- Direct browser control

### v2.0.67 (Thinking Mode)
- Thinking enabled by default for Opus 4.5
- Alt+T toggle (was Tab)

### v2.0.64 (Named Sessions)
- /rename to name sessions
- /resume <name> to continue
- Background agents

### v2.0.51 (Opus 4.5)
- Opus 4.5 model access
- Claude Code Desktop app

---

## Plugin Management Commands

### Discovery & Installation
```
/plugin discover                     # Browse available plugins
/plugin install <name>@<marketplace> # Install a plugin
/plugin list                         # List installed plugins
/plugin update                       # Update all plugins
/plugin remove <name>                # Remove a plugin
```

### Marketplace Management
```
/plugin marketplace add <repo>       # Add a marketplace
/plugin marketplace list             # List marketplaces
/plugin marketplace remove <name>    # Remove marketplace
```

---

## Configuration Files

### Plugin Config Location
```
~/.claude/plugins/
├── config.json                # Plugin configuration
├── installed_plugins.json     # Installed plugins registry
├── known_marketplaces.json    # Added marketplaces
└── marketplaces/              # Marketplace repos
    ├── anthropic-agent-skills/
    └── claude-plugins-official/
```

### User Settings
```
~/.claude/settings.json
```

### Project Settings
```
.claude/settings.json
```

---

## Maintenance Protocol

**Owner Responsibilities:**

1. **Weekly**: Check for Claude Code updates (`npm update -g @anthropic-ai/claude-code`)
2. **Weekly**: Check for new plugins in marketplaces
3. **Monthly**: Review plugin usage and recommend new installs
4. **On Release**: Update this document with new features
5. **On Request**: Research specific plugin capabilities

**Update Checklist:**
- [ ] Check Claude Code version
- [ ] Review changelog for new features
- [ ] Update "Current Installation Status" section
- [ ] Update "Version History" section
- [ ] Notify agents of significant changes

---

## Quick Reference

### Check Version
```bash
claude --version
```

### Update Claude Code
```bash
npm update -g @anthropic-ai/claude-code
```

### Check Installed LSP Servers
```bash
which pyright rust-analyzer typescript-language-server lua-language-server
```

### Verify Plugin Installation
```bash
cat ~/.claude/plugins/installed_plugins.json
```

---

## Resources

- [Claude Code Changelog](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md)
- [Official Plugins Repo](https://github.com/anthropics/claude-plugins-official)
- [Anthropic Skills Repo](https://github.com/anthropics/skills)
- [LSP Plugins (Community)](https://github.com/boostvolt/claude-code-lsps)
- [Claude Code Docs](https://code.claude.com/docs)

---

*This skill document is maintained by the skills-master agent (or Primary AI until spawn).*
*Last verified: 2025-12-26*
