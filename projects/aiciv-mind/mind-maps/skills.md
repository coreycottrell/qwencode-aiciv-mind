# Mind-Map: Skills Module (mind-skills)

**Owner**: mind-skills
**Crate**: `src/aiciv-skills/` (to be created)
**Date**: 2026-04-16

---

## 1. What Exists Today

### Nothing

The `src/aiciv-skills/` crate does **not exist** yet. No skill loading, discovery, rendering, or invocation system exists in any Hengshi crate. The concept of skills is entirely absent from the current 18-crate workspace.

The closest analog in the current codebase is `codex-patcher`, which does build-time code injection via patches -- but that is structural patching, not a skill system. There is no overlap.

### ACG's Existing Skill Format (Context)

ACG's `.claude/skills/` directory contains 100+ skills in a compatible format: each is a directory with a `SKILL.md` file containing YAML frontmatter (`name`, `description`) followed by markdown body with workflow instructions. Some skills include `scripts/`, `references/`, and `assets/` subdirectories. This is the exact format Codex uses, which means aiciv-mind can load ACG's existing skills immediately once the crate is built.

---

## 2. What Codex Has (the cherry-pick target)

Source: `codex-upstream/codex-rs/core-skills/` (5,668 lines, 16 files) + `codex-upstream/codex-rs/skills/` (bundled skill assets)

### Architecture Overview

Codex's skill system has 5 layers:

```
┌─────────────────────────────────────────────────┐
│  5. Injection Layer (injection.rs)              │
│     Load skill content into prompt on demand    │
│     Detect $skill-name and [skill](path) in    │
│     user input. Analytics/metrics tracking.     │
├─────────────────────────────────────────────────┤
│  4. Rendering Layer (render.rs)                 │
│     Format skill metadata into system prompt    │
│     section: name, description, file path.     │
│     Usage instructions for the LLM.            │
├─────────────────────────────────────────────────┤
│  3. Manager Layer (manager.rs)                  │
│     SkillsManager: orchestrates loading.       │
│     Two caches: by-cwd and by-config-key.      │
│     System skill install/uninstall. Product     │
│     filtering. Config rule application.        │
├─────────────────────────────────────────────────┤
│  2. Discovery/Loading Layer (loader.rs)         │
│     BFS dir scan with depth/count limits.      │
│     Parse SKILL.md frontmatter + metadata.     │
│     Hierarchical root discovery. Dedup.        │
│     Scope priority: Repo > User > System > Admin│
├─────────────────────────────────────────────────┤
│  1. Data Model (model.rs)                       │
│     SkillMetadata, SkillPolicy, SkillInterface,│
│     SkillDependencies, SkillLoadOutcome,       │
│     SkillError. Scope and product gating.      │
└─────────────────────────────────────────────────┘
```

### File-by-File Analysis

| File | Lines | Purpose | Cherry-Pick Value |
|------|-------|---------|-------------------|
| `model.rs` | 143 | Core types: `SkillMetadata` (name, description, path, scope, dependencies, policy, interface), `SkillPolicy` (implicit invocation, product gating), `SkillInterface` (display_name, icons, brand_color, default_prompt), `SkillDependencies` (tool deps with type/value/transport), `SkillLoadOutcome` (skills + errors + disabled_paths + implicit indexes) | **HIGH** -- take types, strip Product gating |
| `loader.rs` | 851 | Skill discovery and parsing engine. `load_skills_from_roots()` scans root dirs via BFS. `parse_skill_file()` extracts YAML frontmatter from SKILL.md. `skill_roots()` builds hierarchical root list from config layers. Limits: `MAX_SCAN_DEPTH=6`, `MAX_SKILLS_DIRS_PER_ROOT=2000`. Handles symlinks for non-system scopes. `extract_frontmatter()` splits `---` delimited YAML. `load_skill_metadata()` loads optional `agents/openai.yaml` sidecar. | **HIGH** -- core loading logic. Strip codex-config dependency, replace with aiciv-mind's config |
| `manager.rs` | 297 | `SkillsManager`: Two-cache system (by-cwd for directory-based sessions, by-config for session-local overrides). `skills_for_cwd()` async + `skills_for_config()` sync paths. System skill install/uninstall on construction. Cache clearing. Product filtering on load. | **HIGH** -- take cache pattern, simplify two-cache into one |
| `render.rs` | 48 | `render_skills_section()`: Formats all loaded skills into a `## Skills` section with `### Available skills` list (name + description + path per skill) and `### How to use skills` instructions for the LLM. Wrapped in `<skills-instructions>` tags. | **HIGH** -- take directly, replace tags with aiciv-mind tags |
| `injection.rs` | 494 | Two systems: (1) `build_skill_injections()` reads full SKILL.md content for mentioned skills, wraps in `SkillInstructions` response items with analytics. (2) `collect_explicit_skill_mentions()` detects `$skill-name` plain mentions + `[$skill](path)` linked mentions from user input. Disambiguation via name counts + connector slug counts. | **HIGH** -- take mention detection and injection, strip analytics/otel deps |
| `config_rules.rs` | 134 | `SkillConfigRules`: Enable/disable skills by name or path. Rules extracted from config layer stack. Later rules override earlier ones. Selectors: `Name(String)` or `Path(PathBuf)`. | **MEDIUM** -- simpler version needed for aiciv-mind |
| `invocation_utils.rs` | 156 | `detect_implicit_skill_invocation_for_command()`: When a bash command runs a skill's script (`python scripts/foo.py`) or reads its doc (`cat SKILL.md`), automatically inject that skill. Uses two indexes: `implicit_skills_by_scripts_dir` and `implicit_skills_by_doc_path`. | **MEDIUM** -- useful but not critical for Phase 2 |
| `mention_counts.rs` | 25 | `build_skill_name_counts()`: Counts exact and lowercase skill name occurrences for disambiguation (avoid injecting ambiguous names). | **LOW** -- simple utility, easy to rebuild |
| `env_var_dependencies.rs` | 31 | `collect_env_var_dependencies()`: Extracts `type: env_var` tool dependencies from mentioned skills. Used to warn about missing env vars. | **LOW** -- nice to have |
| `remote.rs` | 271 | Remote skill marketplace: `list_remote_skills()` and `export_remote_skill()`. Downloads zip archives from OpenAI's API, extracts to local filesystem. ChatGPT auth required. | **SKIP** -- OpenAI-specific, not relevant |
| `system.rs` | 10 | System skill install/uninstall: thin wrapper around `codex_skills::install_system_skills()`. | **LOW** -- bundled skill management |
| `Cargo.toml` | 40 | 15 dependencies including codex-analytics, codex-config, codex-protocol, codex-login, codex-otel, codex-skills, serde_yaml, shlex, zip | **REFERENCE** -- identify which deps we need vs skip |

### Key Design Patterns in Codex

1. **Progressive Disclosure**: Only metadata (name, description, path) loaded initially. Full SKILL.md content loaded on-demand only when the skill is invoked.

2. **Hierarchical Discovery**: Skills discovered from multiple roots in priority order. Repo-level skills (`.agents/skills/`) override user-level (`$HOME/.agents/skills/`) which override system builtins.

3. **Dual Invocation**: Skills can be invoked explicitly (`$skill-name`, `/skills` command, `[$skill](path)`) or implicitly (running a skill's script triggers its injection).

4. **Sidecar Metadata**: Rich metadata lives in `agents/openai.yaml` next to SKILL.md, not in the frontmatter. This keeps SKILL.md focused on instructions. Sidecar provides: `display_name`, `short_description`, `icon_small`, `icon_large`, `brand_color`, `default_prompt`.

5. **SKILL.md Anatomy**: YAML frontmatter (`name`, `description`, optional `metadata.short-description`) + markdown body. Directory structure convention: `SKILL.md`, `agents/openai.yaml`, `assets/`, `references/`, `scripts/`.

6. **Fail-Open Loading**: Metadata sidecar parse failures are warned but don't block skill loading. Missing frontmatter or required fields DO cause parse errors.

7. **Scope-Based Sorting**: Skills sorted by scope rank (Repo=0, User=1, System=2, Admin=3) then alphabetically.

8. **Name Namespacing**: Skills from plugins get namespace-prefixed names (`plugin-name:skill-name`). This prevents collisions across providers.

### Codex External Dependencies We Must Strip

| Codex Dependency | Why Strip | Replace With |
|-----------------|-----------|-------------|
| `codex-config` (ConfigLayerStack) | Complex config layer system | Simple TOML config struct |
| `codex-protocol` (Product, SkillScope) | Codex/ChatGPT product gating | Our own SkillScope enum |
| `codex-analytics` / `codex-otel` | Telemetry | Optional metrics trait |
| `codex-login` (CodexAuth) | ChatGPT auth for remote skills | Not needed (skip remote skills) |
| `codex-app-server-protocol` | IDE integration | Not needed |
| `codex-utils-plugins` | Plugin namespace parsing | Simple string manipulation |
| `codex-instructions` | SkillInstructions response item | Our own response type |
| `codex-skills` (install_system_skills) | Bundled skill extraction | Direct filesystem copy |

---

## 3. What Gemini CLI Contributes

Source: Agent definition references `projects/coordination-systems/gemini-cli-module-map.md` (not found in repo -- using agent spec description)

### Key Patterns to Adapt

1. **Hierarchical Skill Discovery** (also in Codex):
   - Workspace-level > User-level > Extensions > Builtins
   - Matches Codex's Repo > User > System > Admin ordering
   - **Takeaway**: Confirm this pattern works for aiciv-mind's multi-civ world

2. **SKILL.md with YAML Frontmatter** (identical to Codex):
   - Same format = shared skill ecosystem across tools
   - **Takeaway**: Keep this format exactly -- skills should be portable between Claude Code, Codex, aiciv-mind

3. **Civ-Level Skills** (aiciv-mind unique):
   - Neither Codex nor Gemini has a "civilization" concept
   - aiciv-mind needs: `Civ > Repo > User > Builtin` hierarchy
   - Civ-level skills = shared across all agents in a civilization
   - Cross-civ skill sharing = discover skills published by sister civs

---

## 4. Gap Analysis: Codex vs aiciv-mind Requirements

### What Codex Has That We Need (Take)

| Feature | Codex Implementation | aiciv-mind Adaptation |
|---------|---------------------|----------------------|
| SKILL.md parsing | `extract_frontmatter()` + `serde_yaml` | Take directly |
| Directory BFS scanning | `discover_skills_under_root()` | Take, adjust root paths |
| Skill metadata types | `SkillMetadata`, `SkillPolicy` etc. | Take, strip Product gating |
| Hierarchical roots | `skill_roots()` | Adapt for Civ > Repo > User > Builtin |
| Manager with cache | `SkillsManager` | Simplify to single cache |
| System prompt rendering | `render_skills_section()` | Take, customize tags/instructions |
| Mention detection | `extract_tool_mentions()` | Take, adapt sigil character |
| On-demand content loading | `build_skill_injections()` | Take, strip analytics |
| Implicit invocation | `detect_implicit_skill_invocation_for_command()` | Take for Phase 3 |
| Enable/disable rules | `SkillConfigRules` | Simplify for TOML config |

### What Codex Has That We Don't Need (Skip)

| Feature | Why Skip |
|---------|---------|
| Product gating (Codex vs ChatGPT) | Single product -- aiciv-mind |
| Remote skill marketplace | OpenAI-specific API |
| ChatGPT auth for remote skills | Not relevant |
| Analytics/OTel tracking | Build our own metrics later |
| ConfigLayerStack integration | Too complex; use simple config |
| Plugin namespace prefix | We'll use civ-name prefix instead |

### What aiciv-mind Needs That Codex Lacks (Build)

| Feature | Why Needed | Design Approach |
|---------|-----------|----------------|
| Civ-level skill scope | Skills shared across all agents in a civ | New `SkillScope::Civ` variant, discovered from civ config dir |
| Cross-civ skill sharing | Sister civs publish skills we can discover | Future: discovery via HUB or filesystem |
| Agent-scoped skills | Some skills only available to specific agent roles | Filter by agent role at load time |
| Skill hot-reload | Skills change during long-running sessions | Watch filesystem, invalidate cache on changes |
| Model-aware skill rendering | Different models need different prompt formats | Renderer takes model hint, adjusts format |

---

## 5. Cherry-Pick Plan

### Phase 2A: Core Loading (Priority 1 -- needed for anything to work)

**Source files to cherry-pick and adapt:**

1. **`model.rs`** → `src/aiciv-skills/src/types.rs`
   - Take: `SkillMetadata`, `SkillDependencies`, `SkillToolDependency`, `SkillError`, `SkillLoadOutcome`
   - Modify: Replace `SkillScope` enum with aiciv-mind's version (`Civ`, `Repo`, `User`, `Builtin`)
   - Strip: `SkillPolicy.products` (no product gating), `filter_skill_load_outcome_for_product()`
   - Simplify: `SkillInterface` -- keep `display_name`, `short_description`; drop icons/brand_color for now

2. **`loader.rs`** → `src/aiciv-skills/src/loader.rs`
   - Take: `SkillRoot`, `load_skills_from_roots()`, `discover_skills_under_root()`, `parse_skill_file()`, `extract_frontmatter()`, `load_skill_metadata()`
   - Modify: `skill_roots()` → `skill_roots_for_aiciv()` -- build roots from simple config, not ConfigLayerStack
   - Strip: `repo_agents_skill_roots()` (uses codex-config project root markers) → replace with simpler workspace root detection
   - Strip: `plugin_namespace_for_skill_path()` → replace with civ-name prefix logic
   - Keep: All validation (`validate_len`, `sanitize_single_line`), all limits (MAX_SCAN_DEPTH, MAX_SKILLS_DIRS_PER_ROOT), symlink following

3. **`render.rs`** → `src/aiciv-skills/src/renderer.rs`
   - Take: `render_skills_section()` almost verbatim
   - Modify: Replace `SKILLS_INSTRUCTIONS_OPEN_TAG`/`CLOSE_TAG` with aiciv-mind's own tags
   - Modify: Adjust LLM usage instructions for aiciv-mind's invocation model

### Phase 2B: Management & Invocation (Priority 2 -- needed for production use)

4. **`manager.rs`** → `src/aiciv-skills/src/manager.rs`
   - Take: `SkillsManager` structure, cache pattern
   - Simplify: Merge two caches (by-cwd + by-config) into single cache keyed by workspace root
   - Strip: `bundled_skills_enabled_from_stack()` → simple config bool
   - Strip: Product filtering
   - Add: `force_reload()` method for hot-reload support

5. **`injection.rs`** → `src/aiciv-skills/src/injection.rs`
   - Take: `build_skill_injections()` (read SKILL.md content on demand)
   - Take: `collect_explicit_skill_mentions()` (detect `$skill-name` in user input)
   - Take: `extract_tool_mentions()` (tokenize mentions from text)
   - Strip: Analytics client, OTel, TrackEventsContext
   - Strip: `ToolMentionKind::App/Mcp/Plugin` variants → keep only `Skill`
   - Simplify: `SkillInjections` → return Vec<(SkillMetadata, String)> instead of ResponseItem

6. **`config_rules.rs`** → `src/aiciv-skills/src/config.rs`
   - Take: `SkillConfigRule`, `SkillConfigRuleSelector`, `resolve_disabled_skill_paths()`
   - Modify: Load rules from simple TOML config instead of ConfigLayerStack
   - Simplify: Remove layer-specific logic

### Phase 3: Advanced Features (Priority 3 -- nice to have)

7. **`invocation_utils.rs`** → `src/aiciv-skills/src/implicit.rs`
   - Take: `detect_implicit_skill_invocation_for_command()`, script detection, doc read detection
   - Low priority but useful for seamless skill integration

8. **`env_var_dependencies.rs`** → fold into `types.rs`
   - Take: `collect_env_var_dependencies()` utility

### Not Cherry-Picked

- **`remote.rs`** -- OpenAI marketplace, not relevant
- **`system.rs`** -- System skill install via `codex_skills` internal crate, too coupled
- **`mention_counts.rs`** -- 25 lines, trivial to rewrite

---

## 6. Interface Contract

### What mind-skills Provides to Other Agents

```rust
// Types (available to all crates via pub use)
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
    pub path_to_skill_md: PathBuf,
    pub scope: SkillScope,
    pub dependencies: Option<SkillDependencies>,
    pub policy: Option<SkillPolicy>,
}

pub enum SkillScope {
    Civ,      // Civilization-level (shared across all agents)
    Repo,     // Repository/workspace-level
    User,     // User-level ($HOME/.aiciv/skills/)
    Builtin,  // Bundled with aiciv-mind
}

pub struct SkillLoadOutcome {
    pub skills: Vec<SkillMetadata>,
    pub errors: Vec<SkillError>,
    pub disabled_paths: HashSet<PathBuf>,
}

// Manager (owned by mind-skills, consumed by mind-coordination)
pub struct SkillsManager { /* ... */ }
impl SkillsManager {
    pub fn new(config: SkillsConfig) -> Self;
    pub async fn load_skills(&self, workspace_root: &Path) -> SkillLoadOutcome;
    pub fn clear_cache(&self);
}

// Rendering (consumed by mind-model-router for prompt building)
pub fn render_skills_section(skills: &[SkillMetadata]) -> Option<String>;

// Injection (consumed by mind-model-router or mind-coordination)
pub async fn load_skill_content(skill: &SkillMetadata) -> Result<String>;
pub fn detect_skill_mentions(input: &str, skills: &[SkillMetadata]) -> Vec<SkillMetadata>;
```

### What mind-skills Needs from Other Agents

| Dependency | From Agent | What's Needed | Interface |
|-----------|-----------|---------------|-----------|
| Workspace root path | mind-coordination | The current working directory / workspace root to discover repo-level skills | `&Path` passed to `load_skills()` |
| Config | mind-coordination | Skill-related config (roots, enabled/disabled, builtin toggle) | `SkillsConfig` struct (TOML deserialized) |
| Prompt injection point | mind-model-router | Where to insert rendered skills section and loaded skill content in prompts | Callback or trait method |
| Type definitions | mind-coordination (codex-types) | If we share types like `SkillId` across crates | `pub type SkillId = String` in codex-types |

### Key Interface Boundaries

1. **Loading is async** -- filesystem scanning happens in background
2. **Rendering is sync** -- just string formatting from loaded metadata
3. **Mention detection is sync** -- regex/parsing on user input text
4. **Content loading is async** -- reads SKILL.md files on demand
5. **Cache invalidation is manual** -- caller triggers `clear_cache()` or `force_reload()`

---

## 7. File Plan

```
src/aiciv-skills/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Module root, pub use exports
│   ├── types.rs        # SkillMetadata, SkillScope, SkillPolicy, SkillDependencies,
│   │                   # SkillLoadOutcome, SkillError, SkillDependencyInfo
│   ├── loader.rs       # SkillRoot, load_skills_from_roots(), discover_skills_under_root(),
│   │                   # parse_skill_file(), extract_frontmatter(), skill_roots_for_aiciv()
│   ├── manager.rs      # SkillsManager: load, cache, invalidate, config rule application
│   ├── renderer.rs     # render_skills_section() -- format skills for system prompt
│   ├── injection.rs    # load_skill_content(), detect_skill_mentions(),
│   │                   # extract_tool_mentions()
│   ├── config.rs       # SkillsConfig, SkillConfigRule, resolve_disabled_skill_paths()
│   └── implicit.rs     # detect_implicit_skill_invocation_for_command() (Phase 3)
```

**Estimated lines**: ~1,500-2,000 (cherry-picked from 5,668, stripping Codex-specific deps and simplifying)

---

## 8. Dependencies

### Required (Cargo.toml)

```toml
[dependencies]
# Serialization
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"

# Filesystem
dunce = "1"          # Path canonicalization (Windows compat, also good practice)

# Async
tokio = { version = "1", features = ["fs"] }

# Logging
tracing = "0.1"

# Error handling
anyhow = "1"
thiserror = "2"

# Internal
codex-types = { path = "../codex-types" }  # Shared types (if we add SkillId there)
```

### NOT needed (stripped from Codex)

| Codex Dep | Why Not Needed |
|-----------|----------------|
| `codex-analytics` | No telemetry |
| `codex-otel` | No OpenTelemetry |
| `codex-login` | No ChatGPT auth |
| `codex-config` | Simple TOML config instead |
| `codex-protocol` | Own SkillScope enum |
| `codex-app-server-protocol` | No IDE integration |
| `codex-utils-plugins` | Simple string handling |
| `codex-instructions` | Own response types |
| `codex-skills` | No bundled skill installer |
| `shlex` | Only needed for implicit invocation (Phase 3) |
| `zip` | Only needed for remote skill download (skipped) |

---

## 9. Skill Discovery Hierarchy for aiciv-mind

```
Priority 0 (highest): Civ-level skills
  Location: {civ_config_dir}/skills/
  Scope: SkillScope::Civ
  Use case: Skills shared across ALL agents in the civilization
  Example: /home/corey/projects/AI-CIV/ACG/.claude/skills/

Priority 1: Repo/workspace-level skills
  Location: {workspace_root}/.aiciv/skills/  (walked up from cwd)
  Scope: SkillScope::Repo
  Use case: Project-specific skills
  Example: /home/corey/projects/AI-CIV/qwen-aiciv-mind/.aiciv/skills/

Priority 2: User-level skills
  Location: $HOME/.aiciv/skills/
  Scope: SkillScope::User
  Use case: User-installed skills

Priority 3 (lowest): Built-in skills
  Location: Bundled with aiciv-mind binary (or cached to disk)
  Scope: SkillScope::Builtin
  Use case: Default capabilities
```

**Dedup rule**: If the same skill name exists at multiple levels, the highest-priority scope wins.

---

## 10. SKILL.md Format (Shared with Codex/Claude Code)

### Frontmatter (YAML between `---` delimiters)

```yaml
---
name: "skill-name"
description: "One-line description of what this skill does and when to use it"
metadata:
  short-description: "Even shorter description for listings"
---
```

### Optional Sidecar: `agents/openai.yaml` (adapt to `agents/aiciv.yaml`)

```yaml
interface:
  display_name: "Human-Friendly Name"
  short_description: "Short tagline"
  default_prompt: "Default prompt when skill is invoked without specific input"
dependencies:
  tools:
    - type: "env_var"
      value: "OPENAI_API_KEY"
      description: "Required for API calls"
    - type: "mcp_server"
      value: "playwright"
      description: "Browser automation"
policy:
  allow_implicit_invocation: true
```

### Directory Convention

```
my-skill/
├── SKILL.md              # Required: frontmatter + instructions
├── agents/
│   └── aiciv.yaml        # Optional: rich metadata sidecar
├── assets/               # Optional: icons, images
├── references/           # Optional: supplementary docs
└── scripts/              # Optional: executable scripts
```

---

## 11. Open Questions

1. **SkillId in codex-types?** Should `SkillId` be defined in `codex-types` for cross-crate use, or is `String` sufficient? Mind-coordination owns types -- need their input.

2. **Sidecar filename**: Codex uses `agents/openai.yaml`. Should we use `agents/aiciv.yaml` to signal our ecosystem, or keep `openai.yaml` for cross-tool compatibility?

3. **Civ skill discovery path**: Should civ-level skills come from a config-specified path, or should we auto-discover them by walking up to find `.claude/skills/` or `.aiciv/skills/`?

4. **Prompt injection format**: What tags should wrap the rendered skills section? Codex uses `<skills-instructions>`. Do we want `<aiciv-skills>` or keep the same tags for model compatibility?

5. **Mention sigil**: Codex uses `$skill-name`. Do we keep `$` or use `/skill-name` (Claude Code convention) or both?

6. **Hot-reload mechanism**: Should we use `notify` crate for filesystem watching, or poll-based invalidation on each prompt cycle? Polling is simpler but adds latency.

7. **Agent-scoped skills**: How to filter skills by agent role? Add `roles: [primary, team-lead]` to frontmatter policy, or handle at the manager level?

---

## 12. Relationship to Other Modules

```
                    ┌─────────────────────┐
                    │  mind-coordination  │
                    │  (workspace root,   │
                    │   config, types)    │
                    └────────┬────────────┘
                             │ provides workspace_root, config
                             ▼
                    ┌─────────────────────┐
                    │    aiciv-skills     │
                    │  (THIS MODULE)      │
                    │  discover, load,    │
                    │  cache, render,     │
                    │  inject skills      │
                    └────────┬────────────┘
                             │ provides rendered section + injected content
                             ▼
                    ┌─────────────────────┐
                    │  mind-model-router  │
                    │  (prompt builder,   │
                    │   LLM API client)   │
                    └─────────────────────┘
```

### Integration Points

1. **At session start**: mind-coordination calls `SkillsManager::load_skills()` with workspace root → gets `SkillLoadOutcome`
2. **At prompt build**: mind-model-router calls `render_skills_section()` with loaded skills → injects into system prompt
3. **At user input**: mind-coordination calls `detect_skill_mentions()` on user text → passes mentioned skills to `load_skill_content()` → injects into conversation
4. **At tool use** (Phase 3): mind-tool-engine calls `detect_implicit_skill_invocation_for_command()` → auto-injects relevant skill
