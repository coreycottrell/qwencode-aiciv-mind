# OpenCLAW Adoption Recommendations
**Research Date:** 2026-05-03
**Source:** github.com/openclaw/openclaw (368k stars, TypeScript/Node.js)
**Method:** Shallow clone + AGENTS.md + VISION.md + CONTRIBUTING.md analysis

---

## Executive Summary

OpenCLAW is a personal AI assistant that runs on your devices, in your channels, with your rules. It supports 20+ messaging channels and has a sophisticated plugin/skills architecture. This report identifies 3 specific patterns we should adopt that we currently lack.

---

## Pattern 1: Scoped AGENTS.md Subdirectory Guidance

### What It Does
OpenCLAW uses a "telegraph style" AGENTS.md at root with "root rules only." Critical: **each subdirectory has its own scoped AGENTS.md** that agents read before working in that subtree.

From AGENTS.md line 3:
> "Telegraph style. Root rules only. Read scoped `AGENTS.md` before subtree work."

The "Map" section (line 19-23) lists all scoped guide locations:
```
extensions/, src/{plugin-sdk,channels,plugins,gateway,gateway/protocol,agents}/,
test/helpers*/, docs/, ui/, scripts/
```

### Where In Their Repo
- Root: `/AGENTS.md` (main agent guidance)
- Subdirectories: Each of the above has its own `AGENTS.md`
- Symlink: Line 17 notes "New `AGENTS.md`: add sibling `CLAUDE.md` symlink"

### Why Ours Doesn't Have It
Our `skills/` directory is flat — each skill has a `SKILL.md` but:
- No subdirectory-scoped guidance (e.g., no `skills/hub-triad/AGENTS.md`)
- No hierarchical guidance inheritance
- Skills don't know context about parent directories

### Effort to Adopt: **Medium**
- Would need convention: `skills/*/AGENTS.md` for skill-specific agent guidance
- Could add to skill template: optional `AGENTS.md` alongside `SKILL.md`
- Example use: hub-triad could have `skills/hub-triad/AGENTS.md` with specific Hub API version guidance, JWT auth gotchas, etc.

---

## Pattern 2: Bundle-Style Plugin Distinction

### What It Does
OpenCLAW distinguishes two plugin types (VISION.md lines 59-66):

1. **Code plugins** — run OpenClaw plugin code, appropriate for deeper runtime extension (runtime hooks, providers, channels, tools)

2. **Bundle-style plugins** — package stable external surfaces: skills, MCP servers, related configuration. Smaller interface, better security boundaries.

Key quote:
> "Prefer bundle-style plugins when they can express the capability. They have a smaller, more stable interface and better security boundaries."

### Where In Their Repo
- VISION.md lines 59-66 (full plugin architecture philosophy)
- `/Dockerfile:173` — `COPY --from=runtime-assets --chown=node:node /app/skills ./skills`
- ClawHub at https://clawhub.ai/ — skills registry and distribution

### Why Ours Doesn't Have It
We have skills but no equivalent distinction:
- No formal "bundle-style" vs "code plugin" separation
- Skills mix implementation and configuration arbitrarily
- No equivalent of ClawHub for skill distribution/discovery

### Effort to Adopt: **Large**
- Requires architectural decision: what is "bundle-style" vs "code" in our context?
- Could start small: add `bundle: true/false` metadata to SKILL.md frontmatter
- ClawHub-equivalent is a separate system (beyond scope)
- Lower effort: formalize skill packaging convention

---

## Pattern 3: Testbox/Blacksmith Remote Test Infrastructure

### What It Does
OpenCLAW has sophisticated test infrastructure:

**Changed Lanes System** (AGENTS.md lines 91-98):
- `core prod`: core prod typecheck + core tests
- `core tests`: core test typecheck/tests
- `extension prod`: extension prod typecheck + extension tests
- `extension tests`: extension test typecheck/tests
- `public SDK/plugin contract`: extension prod/test too
- Unknown root/config: all lanes

**Sparse-Safe Commands** (AGENTS.md line 49):
> "`pnpm check:changed` is sparse-safe and may skip sparse-missing typecheck projects"

**Testbox + Blacksmith** (AGENTS.md lines 58-61):
> "Broad/shared validation defaults to Testbox. This includes `pnpm check`, `pnpm check:changed`, `pnpm test`..."
> "Testbox full-suite profile: blacksmith testbox run..."

**Commit Fast Lane** (CONTRIBUTING.md line 104):
> "`scripts/committer --fast 'message'` passes `FAST_COMMIT=1` through to pre-commit hook so it skips repo-wide `pnpm check`"

### Where In Their Repo
- AGENTS.md lines 44-62 (full command reference)
- AGENTS.md lines 88-111 (gates and changed lanes)
- CONTRIBUTING.md lines 101-127 (before-you-PR validation)
- Scripts: `scripts/committer`, `scripts/check-*-boundary.mjs`

### Why Ours Doesn't Have It
Our `skill-test-runner` is a basic Python script:
- No changed lanes (we run all checks regardless of what changed)
- No sparse-safe checks (we don't use sparse checkouts)
- No remote testbox infrastructure
- No fast-commit lane for small changes

### Effort to Adopt: **Medium-Large**
- Changed lanes: medium effort — categorize skills by type (core/test/extension), run only relevant checks
- Sparse-safe: not applicable (we don't use sparse checkouts)
- Testbox/Blacksmith: large effort — requires Blacksmith account + remote execution infrastructure
- Fast-commit lane: low effort — add `scripts/committer --fast` equivalent

---

## Summary Table

| Pattern | What It Adds | Where | Why We Lack It | Effort |
|---------|--------------|-------|----------------|--------|
| Scoped AGENTS.md | Subdirectory-specific agent guidance | `/AGENTS.md` + `extensions/`, etc. | Flat skills/ directory | M |
| Bundle-style plugins | Skills vs code plugin distinction | `VISION.md` lines 59-66 | No formal packaging distinction | L |
| Testbox/Blacksmith | Remote test execution, changed lanes | `AGENTS.md` lines 44-111 | Basic Python runner | M-L |

## Recommended Priority

1. **Scoped AGENTS.md (M)** — Low risk, high value for skill-specific guidance
2. **Bundle metadata (L)** — Add `bundle: true` to skill frontmatter, formalize packaging
3. **Changed lanes (M)** — Categorize skills, run only relevant tests
4. **Testbox/Blacksmith (L)** — Long-term infrastructure goal

---

## Not Applicable to Our Architecture

- **Multi-channel inbox** — We are single-channel (Hub API)
- **Sandboxing per session** — We are single-user civ
- **20+ platform support** — Out of scope for AiCIV coordination
- **Bundle-style vs code plugin** — We only have skills, no runtime plugin hooks

---

## Files Analyzed

- `/AGENTS.md` — 203 lines, main agent guidance (telegraph style)
- `/VISION.md` — 118 lines, architecture philosophy
- `/CONTRIBUTING.md` — 150+ lines, PR and test guidance
- `/README.md` — 200+ lines, product overview
- `/SECURITY.md` — referenced for security model
