# mind-tool-engine — Domain Mind-Map

**Owner**: mind-tool-engine
**Crate**: `src/codex-exec/`
**Lines**: ~700 (Hengshi foundation)
**Status**: Foundation built, Phase 2 cherry-pick pending

---

## 1. What Exists Today

### File Inventory

```
src/codex-exec/
├── Cargo.toml              # Dependencies: codex-roles, tokio, serde, serde_json, tracing, thiserror, async-trait
├── src/
│   ├── lib.rs              # 19 lines — module root, re-exports ToolRegistry, ToolExecutor, ToolHandler, etc.
│   ├── registry.rs         # 383 lines — ToolDefinition, ToolCall, ToolResult, ToolHandler trait, ToolRegistry, ToolExecutor
│   ├── sandbox.rs          # 306 lines — SandboxEnforcer with workspace containment + publishing gates
│   └── tools/
│       ├── mod.rs           # 27 lines — register_builtins() wiring function
│       ├── bash.rs          # 88 lines — BashTool: shell execution with timeout, cwd, stdout/stderr capture
│       ├── read.rs          # 66 lines — ReadTool: file read with offset/limit, line-numbered output
│       ├── write.rs         # 58 lines — WriteTool: file write with auto-mkdir
│       ├── grep.rs          # 126 lines — GrepTool: regex search via system grep, output modes, context lines
│       ├── glob.rs          # 85 lines — GlobTool: file finder via system find, excludes target/.git
│       ├── web_fetch.rs     # 145 lines — WebFetchTool: Ollama Cloud primary + Jina Reader fallback
│       └── web_search.rs    # 153 lines — WebSearchTool: Ollama Cloud primary + DDG fallback
```

### Architecture Overview

```
LLM tool_use response
       │
       ▼
  ToolExecutor.execute(call, role)
       │
       ├── 1. Registry lookup (ToolRegistry.get)
       │       └── ToolHandler trait: execute() + definition()
       │
       ├── 2. Role filtering (codex_roles::is_tool_allowed)
       │       ├── Primary → coordination tools only
       │       ├── TeamLead → delegation tools only
       │       └── Agent → wildcard (all tools)
       │
       ├── 3. Exec policy check (codex_roles::exec_policy_for_role)
       │       ├── DenyAll → block everything (Primary)
       │       ├── DenyExceptIpc → IPC tools only (TeamLead)
       │       └── Sandboxed → proceed to sandbox (Agent)
       │
       ├── 4. Sandbox check (SandboxEnforcer.check_mutation)
       │       ├── ReadOnlyCoordination → deny all mutations
       │       ├── TeamScoped → scratchpad/memory writes only
       │       ├── WorkspaceWrite → workspace containment + forbidden paths
       │       └── ReadOnly → deny all mutations (red team)
       │
       └── 5. Execute (ToolHandler.execute)
               └── Returns ToolResult { success, output, error }
```

### Core Types

| Type | Location | Purpose |
|------|----------|---------|
| `ToolDefinition` | registry.rs:19 | Tool metadata: name, description, JSON schema, mutates flag |
| `ToolCall` | registry.rs:31 | Inbound tool invocation: name + JSON arguments |
| `ToolResult` | registry.rs:40 | Execution result: success, output, error |
| `ToolHandler` | registry.rs:91 | Trait: `execute(args) -> ToolResult` + `definition() -> ToolDefinition` |
| `ToolRegistry` | registry.rs:100 | HashMap<String, Arc<dyn ToolHandler>> — name→handler mapping |
| `ToolExecutor` | registry.rs:158 | Registry + SandboxEnforcer — full pipeline execution |
| `ExecError` | registry.rs:70 | Error enum: ToolNotFound, PolicyDenied, SandboxViolation, ExecutionFailed |
| `SandboxEnforcer` | sandbox.rs:26 | Workspace-scoped mutation checker with publishing gates |

### Built-in Tools (7 total)

| Tool | File | mutates | How It Works |
|------|------|---------|-------------|
| `bash` | bash.rs | yes | `tokio::process::Command` with timeout, cwd = workspace_root |
| `read` | read.rs | no | `tokio::fs::read_to_string` with offset/limit, line-numbered output |
| `write` | write.rs | yes | `tokio::fs::write` with auto parent dir creation |
| `grep` | grep.rs | no | Shells out to system `grep -rnE` with include filters + output modes |
| `glob` | glob.rs | no | Shells out to system `find -name -type f` excluding target/.git |
| `web_fetch` | web_fetch.rs | no | Ollama Cloud API → Jina Reader fallback, 8KB truncation |
| `web_search` | web_search.rs | no | Ollama Cloud API → DDG Python fallback, 8-result default |

### Sandbox Enforcement Layers

```
Layer 1: FORBIDDEN_WRITE_PATHS — publishing gate (hardcoded)
         - /home/corey/projects/AI-CIV/ACG/projects/aiciv-inc
         - projects/aiciv-inc

Layer 2: FORBIDDEN_COMMANDS — deploy gate (hardcoded)
         - "netlify deploy"
         - "netlify-deploy"

Layer 3: Dangerous command patterns — safety net (hardcoded)
         - rm -rf /, fork bomb, mkfs, dd if=/dev/zero, chmod 777 /, curl|bash

Layer 4: Workspace containment — path prefix check
         - Absolute paths must start_with(workspace_root)
         - Relative paths joined to workspace_root then checked

Layer 5: Role-based sandbox levels (from codex-roles)
         - ReadOnlyCoordination, TeamScoped, WorkspaceWrite, ReadOnly
```

### Test Coverage

- **registry.rs**: 7 tests — register/lookup, role filtering, executor policy enforcement, tool-not-found
- **sandbox.rs**: 11 tests — workspace containment, role-level enforcement, publishing gates, dangerous commands
- Total: **18 tests** covering the core execution pipeline

### Dependencies

| Dependency | What We Use |
|-----------|------------|
| `codex-roles` | `Role`, `ExecPolicyLevel`, `SandboxLevel`, `is_tool_allowed`, `exec_policy_for_role`, `sandbox_for_role` |
| `tokio` | Async runtime, `process::Command`, `fs`, `time::timeout` |
| `serde`/`serde_json` | JSON serialization for tool definitions and arguments |
| `async-trait` | `ToolHandler` trait uses async fn in trait |
| `tracing` | Logging for tool execution events |
| `thiserror` | `ExecError` derive |

---

## 2. What's Missing (Gaps vs Full Tool Engine)

### Critical Gaps

| Gap | Severity | Description |
|-----|----------|-------------|
| **No process isolation** | HIGH | Bash tool runs commands as the same user — no bwrap, no Landlock, no seccomp. A compromised agent can escape workspace via bash. |
| **No Edit tool** | HIGH | Only read/write exist. No surgical string-replacement tool (Claude Code's Edit). Agents must overwrite entire files. |
| **No NotebookEdit tool** | LOW | Missing Jupyter notebook cell editing. Not critical for CLI harness. |
| **No timeout on read/write** | MEDIUM | Read/write tools have no timeout. A huge file or slow disk could block the executor indefinitely. |
| **No output truncation on bash** | MEDIUM | Bash tool returns full stdout — no cap. Large command outputs could blow context. |
| **Grep/Glob shell out to system commands** | MEDIUM | Depends on system `grep`/`find` being installed. Should use Rust-native libraries (walkdir + regex) for portability. |
| **No tool deregistration** | LOW | Can register tools but never remove them. No dynamic tool loading/unloading at runtime. |
| **No tool versioning** | LOW | No version metadata on tools. Can't evolve tool schemas without breaking callers. |
| **No execution hooks** | HIGH | No pre/post tool-use events emitted. mind-hooks needs these to implement lifecycle hooks (approval, logging, blocking). |
| **No execution metrics** | MEDIUM | No timing, success rate, or error tracking per tool. mind-testing needs this for fitness scoring. |
| **No parallel tool execution** | MEDIUM | ToolExecutor processes one call at a time. No batch execution or parallel dispatch. |
| **No MCP tool bridge** | MEDIUM | Can only run built-in tools. No way to proxy tool calls to external MCP servers. mind-mcp needs this. |
| **No apply_patch tool** | MEDIUM | Codex has a sophisticated apply_patch for surgical diffs. We rely on full-file write. |
| **Sandbox doesn't resolve symlinks** | MEDIUM | Path containment check uses `starts_with` without resolving symlinks. Agent could symlink-escape. |
| **No env var isolation** | MEDIUM | Bash tool inherits full parent environment. Should whitelist env vars for sandboxed processes. |
| **No resource limits** | MEDIUM | No cgroup, no ulimit. Bash commands can consume unlimited CPU/RAM/disk. |

### Architectural Gaps

| Gap | Description |
|-----|-------------|
| **No ToolEvent emission** | Execute should emit events (ToolStarted, ToolCompleted, ToolFailed) for EventBus integration with codex-drive |
| **No async tool registration** | Tools are registered synchronously at startup. No support for discovering new tools at runtime (MCP discovery). |
| **No tool-use approval flow** | No mechanism for "ask before executing" (auto/suggest/always-ask permission modes). |
| **No command allowlists** | Sandbox checks blacklist patterns but has no positive allowlist. Codex uses Starlark exec policies for precise control. |

---

## 3. What to Cherry-Pick from Codex

### 3a. Sandboxing (~9,500 lines) — Primary Cherry-Pick

**Source**: `codex-upstream/codex-rs/sandboxing/` (3,468 lines) + `codex-upstream/codex-rs/linux-sandbox/` (4,009 lines)

#### TAKE (Linux-native isolation stack)

| File | Lines | What It Does | Priority |
|------|-------|-------------|----------|
| `linux-sandbox/src/landlock.rs` | 343 | **Landlock LSM**: `apply_sandbox_policy_to_current_thread()` — Landlock v5 filesystem rules (read-everywhere, write-only-writable-roots), seccomp BPF for network (deny connect/accept/bind/listen/sendto), `PR_SET_NO_NEW_PRIVS`. This is the kernel-level enforcement our sandbox.rs is faking with string matching. | P0 |
| `sandboxing/src/manager.rs` | 284 | **SandboxManager**: Orchestrates sandbox type selection (Auto/Require/Forbid), transforms commands into sandboxed commands. Entry point for the entire sandboxing pipeline. | P0 |
| `sandboxing/src/policy_transforms.rs` | 446 | **EffectiveSandboxPermissions**: Merges base policy with per-tool additional permissions. Computes effective file-system and network policies. | P1 |
| `sandboxing/src/landlock.rs` | 117 | **Landlock CLI builder**: `create_linux_sandbox_command_args_for_policies()` — builds the CLI invocation for the sandbox helper binary. | P1 |
| `sandboxing/src/bwrap.rs` | 46 | **Bubblewrap detection**: `find_system_bwrap_in_path()` — locates system bwrap binary. | P2 |
| `linux-sandbox/src/launcher.rs` | 226 | **Bwrap launcher**: System bwrap vs vendored bwrap selection, `exec_bwrap()` with fd inheritance. | P2 |
| `linux-sandbox/src/bwrap.rs` | 1,256 | **Bwrap command builder**: Full bubblewrap command-line construction — mount binds, proc/dev/tmp, overlays, network namespace, `--die-with-parent`. | P2 |
| `linux-sandbox/src/linux_run_main.rs` | 742 | **Main entry point**: CLI arg parsing, sandbox policy deserialization, calls landlock + bwrap. | P2 |

#### SKIP

| File | Lines | Why Skip |
|------|-------|----------|
| `sandboxing/src/seatbelt.rs` | 529 | macOS only — we're Linux-only for now |
| `sandboxing/src/seatbelt_tests.rs` | 1,069 | macOS test code |
| `linux-sandbox/src/proxy_routing.rs` | 797 | Network proxy routing — complex, not needed yet |
| `linux-sandbox/src/vendored_bwrap.rs` | 78 | Vendored bwrap binary embedding — we can use system bwrap |

#### Integration Strategy

```
Current sandbox.rs (306 lines)        Codex upstream (~3,200 lines relevant)
────────────────────────────           ──────────────────────────────────────
SandboxEnforcer                        SandboxManager
  .check_mutation()                      .select_initial() → SandboxType
  - string-based path check              .transform() → SandboxExecRequest
  - dangerous command blacklist
  - publishing gates                   apply_sandbox_policy_to_current_thread()
                                         - PR_SET_NO_NEW_PRIVS
                                         - Landlock filesystem rules
                                         - seccomp BPF network filter

                                       Bwrap launcher
                                         - Filesystem namespace isolation
                                         - --die-with-parent
                                         - Mount bind whitelist

Plan: Layer kernel enforcement UNDER our existing application-level checks.
  1. Keep SandboxEnforcer (publishing gates, command blacklist) as Layer 1
  2. Add Landlock + seccomp as Layer 2 (kernel enforcement)
  3. Add bwrap as Layer 3 (full namespace isolation for bash)
```

### 3b. Tool System (~9,489 lines) — Study & Adapt

**Source**: `codex-upstream/codex-rs/tools/` (9,489 lines)

#### STUDY (patterns to adopt)

| File | Lines | Pattern to Learn |
|------|-------|-----------------|
| `tool_definition.rs` | 31 | `ToolDefinition` with `input_schema`, `output_schema`, `defer_loading` — richer than ours |
| `tool_spec.rs` | 195 | `ToolSpec` enum: Function, ToolSearch, LocalShell, ImageGeneration, WebSearch, Custom — tagged union pattern for heterogeneous tools |
| `tool_config.rs` | ~300 | `ToolsConfig` with shell type, backend config, exec mode — configurable tool behavior |
| `tool_discovery.rs` | ~400 | `DiscoverableTool` with search/suggest — deferred tool loading for context optimization |
| `tool_registry_plan.rs` | ~300 | `ToolRegistryPlan` — declarative tool registration plan (which tools to load, their handlers, their configs) |
| `apply_patch_tool.rs` | ~300 | `ApplyPatchTool` — surgical diff application (we need this) |
| `local_tool.rs` | ~400 | `ShellTool` + `CommandTool` + `RequestPermissions` — configurable shell execution with approval |
| `agent_tool.rs` | ~500 | `SpawnAgent`, `CloseAgent`, `SendMessage`, `WaitAgent` — agent lifecycle tools |
| `mcp_tool.rs` | ~200 | MCP tool proxying — bridge to external tool servers |

#### SKIP

| File | Lines | Why Skip |
|------|-------|----------|
| `js_repl_tool.rs` | ~200 | JavaScript REPL — not needed |
| `view_image.rs` | ~150 | Image viewing — not needed yet |
| `code_mode.rs` | ~300 | Code mode — Codex-specific feature |
| `responses_api.rs` | ~300 | OpenAI Responses API format — we use OpenAI chat format via codex-llm |
| All `*_tests.rs` files | ~4,000 | Study for patterns, but tests are Codex-specific |

### 3c. Exec Policy (~1,500 lines) — Study

**Source**: `codex-upstream/codex-rs/execpolicy/`

| Pattern | What to Learn |
|---------|--------------|
| Starlark policy language | Declarative allow/deny rules for commands — more principled than our string blacklist |
| `Policy::evaluate()` | Match commands against rules, return Allow/Deny decisions |
| `Rule` types | Glob patterns, regex, network protocols — structured command filtering |

**Recommendation**: Study but don't integrate yet. Our codex-roles + SandboxEnforcer approach is simpler and sufficient for Phase 2. Exec policies become valuable when we need per-agent or per-project command allowlists.

---

## 4. Dependencies on Other Agents' Modules

### What I CONSUME

| Agent | Module | What I Need | Status |
|-------|--------|------------|--------|
| **mind-coordination** | `codex-types` | Shared types: if tool types migrate there (ToolName, TaskId) | Not needed yet — our types are self-contained |
| **mind-coordination** | `codex-roles` | `Role`, `SandboxLevel`, `ExecPolicyLevel`, filtering functions | **Active dependency** — compiles, well-defined |
| **mind-model-router** | `codex-llm` | Tool call format in LLM responses (how tool_use blocks are parsed into ToolCall) | **Interface needed** — who converts LLM response → ToolCall? |
| **mind-hooks** | `codex-patcher` | Pre/post tool-use hook events (should I emit events? what format?) | **Interface needed** — no hook protocol defined yet |
| **mind-coordination** | `codex-drive` | EventBus integration for ToolStarted/ToolCompleted events | **Interface needed** — no event emission yet |

### What I PROVIDE

| Consumer | What I Provide | Interface |
|----------|---------------|-----------|
| **mind-coordination** (cortex main binary) | `ToolExecutor` — the execution engine | `ToolExecutor::new(registry, sandbox)` + `execute(call, role)` |
| **mind-model-router** (codex-drive) | `ToolRegistry::definitions()` — tool list for LLM | `Vec<ToolDefinition>` serialized as JSON schemas |
| **mind-hooks** (codex-patcher) | Tool execution events (PLANNED) | Not yet implemented — need event protocol |
| **mind-mcp** (codex-suite-client) | Tool registration interface for MCP-discovered tools | `ToolRegistry::register(Arc<dyn ToolHandler>)` — already works |
| **mind-testing** | Execution metrics (PLANNED) | Not yet implemented — need metrics protocol |
| **All agents** | `register_builtins()` — standard tool set | `tools::register_builtins(&mut registry, workspace_root)` |

---

## 5. Recommended Build Order

### Phase 2A: Critical Missing Tools (1-2 sessions)

1. **Edit tool** (`tools/edit.rs`)
   - Exact string replacement (old_string → new_string)
   - Unique match enforcement
   - `replace_all` flag
   - This unblocks efficient code editing without full-file rewrites

2. **Apply Patch tool** (`tools/apply_patch.rs`)
   - Study Codex's `apply_patch_tool.rs`
   - Unified diff application
   - Error recovery on failed hunks

3. **Output truncation for bash**
   - Cap stdout at configurable limit (default 8KB)
   - Tail mode for long outputs

### Phase 2B: Kernel-Level Sandboxing (2-3 sessions)

4. **Landlock integration** (`sandbox/landlock.rs`)
   - Cherry-pick `linux-sandbox/src/landlock.rs`
   - `apply_sandbox_policy_to_current_thread()` — Landlock v5 filesystem rules + seccomp BPF
   - Adapt to our `SandboxLevel` enum (map WorkspaceWrite → Landlock writable roots)

5. **Seccomp network filter** (part of landlock.rs)
   - Block network syscalls (connect, accept, bind) for sandboxed agents
   - Allow AF_UNIX for IPC

6. **Bwrap integration** (`sandbox/bwrap.rs`)
   - Cherry-pick `linux-sandbox/src/bwrap.rs` + `launcher.rs`
   - Wrap bash tool execution in bubblewrap namespace
   - `--die-with-parent` for process cleanup
   - Mount bind whitelist from workspace root

7. **SandboxManager** (`sandbox/manager.rs`)
   - Cherry-pick `sandboxing/src/manager.rs` pattern
   - Auto/Require/Forbid preference
   - Command transformation pipeline

### Phase 2C: Tool Engine Maturity (2-3 sessions)

8. **ToolEvent emission**
   - Define `ToolEvent` enum: Started, Completed, Failed, Blocked
   - Emit via EventBus (coordinate with mind-model-router for codex-drive)
   - Interface Note to mind-hooks for hook integration

9. **Execution metrics**
   - Per-tool timing, success rate, error counts
   - Interface Note to mind-testing for fitness scoring

10. **Symlink resolution**
    - Resolve symlinks before path containment check
    - Use `tokio::fs::canonicalize` for real path resolution

11. **Environment variable isolation**
    - Whitelist env vars for bash tool
    - Inherit: PATH, HOME, USER, LANG
    - Block: API keys, tokens, secrets (unless explicitly allowed)

12. **Resource limits**
    - Set ulimits for bash child processes (CPU time, memory, file size)
    - Optional cgroup integration for hard limits

### Phase 3: Advanced Capabilities

13. **Dynamic tool registration** — runtime tool discovery from MCP servers
14. **Tool-use approval flow** — auto/suggest/always-ask permission modes
15. **Parallel tool execution** — batch dispatch with concurrent limits
16. **Rust-native grep/glob** — replace system command dependencies with walkdir + regex

---

## 6. Dependency Graph (Visual)

```
codex-roles (EXTERNAL — mind-coordination owns)
    │
    ├── Role, SandboxLevel, ExecPolicyLevel
    │
    ▼
codex-exec (THIS CRATE — mind-tool-engine owns)
    │
    ├── registry.rs ──── ToolRegistry, ToolExecutor, ToolHandler trait
    │       │
    │       ├── tools/mod.rs ──── register_builtins()
    │       │       ├── bash.rs ──── tokio::process::Command
    │       │       ├── read.rs ──── tokio::fs::read_to_string
    │       │       ├── write.rs ──── tokio::fs::write
    │       │       ├── grep.rs ──── system grep
    │       │       ├── glob.rs ──── system find
    │       │       ├── web_fetch.rs ──── curl → Ollama/Jina
    │       │       └── web_search.rs ──── curl → Ollama/DDG
    │       │
    │       └── sandbox.rs ──── SandboxEnforcer
    │               ├── Workspace containment (path prefix)
    │               ├── Publishing gates (forbidden paths)
    │               ├── Command safety (pattern blacklist)
    │               └── [PLANNED] Landlock + seccomp + bwrap
    │
    ▼
CONSUMERS:
    codex-drive (mind-model-router) ──── calls ToolExecutor.execute()
    codex-coordination (mind-coordination) ──── uses ToolRegistry.definitions()
    codex-patcher (mind-hooks) ──── [PLANNED] receives ToolEvents
    codex-suite-client (mind-mcp) ──── registers external tools into ToolRegistry
```

---

## 7. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Bash tool is unsandboxed | HIGH — agent can escape workspace, read secrets, modify system | Phase 2B: Landlock + bwrap integration |
| No Edit tool | MEDIUM — agents must overwrite entire files | Phase 2A: Build Edit tool first |
| Symlink escape | MEDIUM — attacker can symlink to outside workspace | Phase 2C: Add canonicalize before containment check |
| Env var leakage | MEDIUM — bash inherits API keys | Phase 2C: Environment whitelist |
| System command dependencies | LOW — grep/glob need system tools | Phase 3: Rust-native replacements |

---

*Generated by mind-tool-engine | 2026-04-16*
*Next: Build Edit tool (Phase 2A, item 1), then begin Landlock cherry-pick (Phase 2B, item 4)*
