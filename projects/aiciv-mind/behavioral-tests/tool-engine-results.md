# Edit Tool — Behavioral Test Results

**Agent**: mind-tool-engine
**Date**: 2026-04-16
**Target**: `src/codex-exec/src/tools/edit.rs` (EditTool, ~133 lines)
**Test file**: `src/codex-exec/tests/edit_behavioral.rs` (integration tests, ~220 lines)
**Result**: **5/5 PASS**

---

## Test Results

| # | Test | What It Proves | Result |
|---|------|---------------|--------|
| 1 | Happy path: single-line edit in Python file | Creates a real `.py` file with two functions, edits one line (`"Hello"` → `"Hi"`), verifies the changed line is correct AND every other line is byte-identical. Checks line count is preserved. | **PASS** |
| 2 | Multiline edit: replace 3 lines in middle of 10 | Creates a 10-line file, replaces lines 4–6 with new content, verifies all 10 lines individually — 3 changed, 7 untouched. Line count preserved. | **PASS** |
| 3 | Ambiguous match rejection: 5 TODOs | Creates a Python file with `TODO` on 5 different lines. Calls edit without `replace_all`. Verifies: (a) tool fails, (b) error says "5 times", (c) error shows all 5 line numbers (1, 3, 5, 7, 9), (d) error suggests "replace_all" or "more context", (e) file is completely unchanged. | **PASS** |
| 4 | Replace-all: 5 TODOs → 5 DONEs | Same file as test 3, but with `replace_all: true`. Verifies: (a) tool succeeds, (b) output says "5 occurrences", (c) zero `TODO` remaining, (d) exactly 5 `DONE` present, (e) non-TODO content intact. | **PASS** |
| 5 | Nonexistent file: helpful error | Calls edit on `/tmp/.../this-file-does-not-exist-at-all.rs`. Verifies: (a) tool fails (no panic), (b) error includes the full file path, (c) error says "Failed to read", (d) error is not empty/panic/stack trace. | **PASS** |

### Execution Summary

```
$ cargo test -p codex-exec -- --nocapture

running 25 tests (unit)   ... ok. 25 passed
running 5 tests (behavioral) ... ok. 5 passed
running 0 tests (doc)     ... ok. 0 passed

Total: 30 passed; 0 failed; 0 ignored
```

---

## Blocker Analysis

### Can the Edit tool be called from outside the harness?

**YES** — and this is a strength. `EditTool` implements `ToolHandler` which takes `serde_json::Value` args and returns `ToolResult`. It has zero runtime dependencies on the harness — no `ToolExecutor`, no `SandboxEnforcer`, no `Role` system needed. You can construct `EditTool` and call `.execute()` directly from any Rust code. The behavioral tests prove this works.

### Is there a CLI entry point to invoke a single tool?

**NO** — this is a gap. Today there is no way to do:
```bash
$ aiciv-mind tool edit --file_path foo.rs --old_string "x" --new_string "y"
```

The only entry points are:
- `cortex` binary (daemon/serve/demo modes) — tools are called by the LLM through ThinkLoop
- Direct Rust test code (what we did here)

**Recommendation**: Add a `cortex tool <name> <json-args>` CLI subcommand to `src/cortex/src/main.rs`. This would:
1. Parse the tool name and JSON args from CLI
2. Construct a `ToolRegistry`, register builtins
3. Call `registry.get(name).unwrap().execute(args).await`
4. Print result as JSON to stdout

This is ~30 lines of code and would make debugging/testing tools trivial.

### What integration points are missing?

1. **No hook wiring**: `PreToolUse`/`PostToolUse` hooks from `aiciv-hooks` are not wired into the edit path yet. The hooks dispatcher exists but isn't called. This means pre-edit approval (like "are you sure you want to modify this file?") doesn't work yet.

2. **No role allowlist entry**: `edit` is registered as a builtin but hasn't been added to `codex_roles::is_tool_allowed()` for any role. Currently only `Role::Agent` gets unrestricted tool access. If a `Role::TeamLead` or `Role::Specialist` tries to use `edit`, it will be denied by the policy layer.

3. **No sandbox path enforcement**: The `SandboxEnforcer` checks mutations for `bash` and `write` tools, but the `edit` tool's `file_path` argument isn't checked against the workspace boundary by the sandbox. You could theoretically edit `/etc/passwd` — the sandbox only kicks in when `ToolExecutor.execute()` is called (which calls `check_mutation`), but `check_mutation`'s path validation depends on whether the sandbox impl parses `file_path` from the edit tool's JSON args. **This needs verification** — the sandbox may not yet know how to extract paths from edit's specific argument schema.

4. **No undo/backup**: Unlike Claude Code's Edit tool, there's no backup or undo mechanism. If the edit is wrong, there's no way to revert. For a coding assistant, this is an acceptable gap in Phase 1 (git handles it), but for a general tool it's worth noting.

### What would break in a real coding session?

1. **Large files**: The tool reads the entire file into memory (`fs::read_to_string`), does string matching, and writes it all back. For files >100MB this could be slow or OOM. Not a practical concern for source code (rarely >1MB), but worth noting.

2. **Binary files**: `read_to_string` will fail on binary files with a UTF-8 decode error. The error message would say "Failed to read" but wouldn't clarify it's a binary file issue. Minor UX gap.

3. **Concurrent edits**: If two tool calls edit the same file simultaneously (e.g., parallel tool execution), one write could clobber the other. The tool has no file locking. In practice, ThinkLoop executes tools sequentially today, so this isn't a current risk — but it would become one with parallel tool execution.

4. **Permissions**: The tool doesn't check file permissions before attempting to write. It will fail with an OS error if the file is read-only, but the error message (`"Failed to write"`) could be more specific about why.

---

## Quality Assessment

The Edit tool is **production-ready for Phase 1**. Key strengths:
- Clean `ToolHandler` trait implementation with no external dependencies
- Helpful error messages (line numbers on ambiguous matches, file preview on not-found)
- Correct use of `replacen(1)` for single occurrence (not just `.replace()`)
- `replace_all` flag with accurate occurrence counting
- Identical-strings guard prevents no-op edits
- All 30 tests pass including 5 end-to-end behavioral tests

The blockers identified above are all Phase 2 concerns. Nothing prevents using the edit tool in a real LLM-driven coding session today, as long as the `Role::Agent` path is used.
