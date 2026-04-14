# Tool Calling Improvements for M2.7 (MiniMax)

**Date**: 2026-04-04
**Status**: Research & Plan (no code changes yet)
**Scope**: Improve how Cortex presents tools to M2.7 and handles its malformed responses

---

## Problem Statement

M2.7 (MiniMax via Ollama Cloud) has five known failure modes when making tool calls:

1. **Trailing commas in JSON** -- `{"limit": 10,}` crashes downstream `int()` / `as_u64()` parsing
2. **Wrong parameter names** -- uses `file_path`, `filepath`, `file`, `filename` when the tool expects `path`; uses `path` when the tool expects `file_path`
3. **Narrator mode** -- claims completion after 1 tool call instead of continuing the loop
4. **Group names instead of UUIDs** -- passes `"civsubstrate"` to Hub tools expecting UUIDs
5. **Prose about tool calls** -- generates text *describing* a tool call instead of emitting actual tool call syntax

Each failure mode has a different root cause and requires a different intervention layer.

---

## 1. System Prompt Additions

### Current State

The system prompts in `src/codex-llm/src/prompt.rs` have minimal tool-use instruction. The `TEAM_LEAD_SYSTEM` prompt (lines 177-213) includes a brief "Tool Use -- CRITICAL" section, but the `AGENT_SYSTEM` prompt (lines 215-240) and `PRIMARY_SYSTEM` prompt (lines 149-175) have no explicit tool-calling instructions at all.

### Problem

M2.7 sometimes generates prose *about* tool calls ("I would call the bash tool with...") instead of emitting actual structured tool calls. It also claims completion prematurely.

### Recommended Changes

Add a `TOOL_USE_PREAMBLE` constant injected into ALL system prompts (not just TeamLead). This should appear early in the prompt because M2.7 has a recency bias -- instructions at the end get more weight, but tool-calling format needs to be internalized before the model sees tool schemas.

```
## How to Use Tools (MANDATORY)

You have tools available as structured function calls. The system handles execution.

RULES:
1. When you want to perform an action, EMIT A TOOL CALL. Do not describe it in prose.
2. WRONG: "I would use the read tool to read /tmp/foo.txt"
   RIGHT: (emit a read tool call with file_path="/tmp/foo.txt")
3. You may call multiple tools in sequence. After each tool result, decide your next action.
4. Do NOT claim your task is complete until you have evidence from tool results.
5. If a tool returns an error, try to fix the issue and retry -- do not give up after one attempt.
6. Use EXACT parameter names as shown in tool descriptions. Do not invent aliases.
```

**Where to inject**: In `PromptBuilder::system_prompt()`, append this block after the role section and before AGENTS.md injection. Apply to all three roles (Primary, TeamLead, Agent).

### Completion Prevention

Add a specific anti-narrator-mode instruction:

```
## Completion Standard

Do NOT say "done", "complete", or "finished" unless ALL of the following are true:
- You have called at least one tool and received its result
- The tool result confirms your action succeeded
- You have addressed the full scope of the task, not just one sub-part

If the task has multiple steps, complete ALL steps before claiming completion.
```

This directly counters the pattern where M2.7 calls one tool, sees a result, and immediately says "Done! I have completed the task."

---

## 2. Parameter Aliasing / Normalization

### Current State

Tool argument parsing in `think_loop.rs` line 260:
```rust
let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
    .unwrap_or(serde_json::Value::Null);
```

No normalization occurs. The raw parsed JSON goes directly to tool handlers. Each handler does its own `args.get("file_path")` etc. If M2.7 sends `"path"` instead of `"file_path"`, the handler returns an error.

### Problem

M2.7 confuses parameter names across tools:
- `read` and `write` expect `file_path`, but M2.7 sends `path`, `filepath`, `file`, or `filename`
- `glob` and `grep` expect `path`, but M2.7 sometimes sends `directory`, `dir`, or `search_path`
- `memory_search` expects `query`, but M2.7 sometimes sends `search_query` or `q`

### Recommended Changes

Add a **parameter normalization layer** in `think_loop.rs` between JSON parsing and tool dispatch. This function runs on every tool call's arguments before they reach any handler:

```rust
fn normalize_args(tool_name: &str, mut args: serde_json::Value) -> serde_json::Value {
    let obj = match args.as_object_mut() {
        Some(o) => o,
        None => return args,
    };

    // Per-tool alias maps
    let aliases: &[(&str, &str)] = match tool_name {
        "read" | "write" => &[
            ("path", "file_path"),
            ("filepath", "file_path"),
            ("file", "file_path"),
            ("filename", "file_path"),
        ],
        "glob" => &[
            ("directory", "path"),
            ("dir", "path"),
            ("search_path", "path"),
        ],
        "grep" => &[
            ("directory", "path"),
            ("dir", "path"),
            ("search_path", "path"),
            ("regex", "pattern"),
            ("search", "pattern"),
            ("query", "pattern"),
        ],
        "bash" => &[
            ("cmd", "command"),
            ("shell_command", "command"),
        ],
        "memory_search" => &[
            ("search_query", "query"),
            ("q", "query"),
            ("search", "query"),
            ("text", "query"),
        ],
        "hub_list_rooms" => &[
            ("group", "group_id"),
            ("group_name", "group_id"),
        ],
        "hub_list_threads" => &[
            ("room", "room_id"),
            ("room_name", "room_id"),
        ],
        "hub_read_thread" => &[
            ("thread", "thread_id"),
        ],
        "hub_reply" => &[
            ("thread", "thread_id"),
            ("content", "body"),
            ("message", "body"),
            ("text", "body"),
            ("reply", "body"),
        ],
        "hub_create_thread" => &[
            ("room", "room_id"),
            ("content", "body"),
            ("message", "body"),
            ("text", "body"),
        ],
        _ => &[],
    };

    for &(alias, canonical) in aliases {
        if !obj.contains_key(canonical) {
            if let Some(val) = obj.remove(alias) {
                obj.insert(canonical.to_string(), val);
            }
        }
    }

    args
}
```

**Where to add**: Call `normalize_args(tool_name, args)` at `think_loop.rs` line 260, right after the `serde_json::from_str` parse.

**Important**: Only remap if the canonical key is NOT already present. This prevents overwriting correct values.

---

## 3. Input Sanitization (Trailing Commas, Type Coercion)

### Current State

`think_loop.rs` line 260-261:
```rust
let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
    .unwrap_or(serde_json::Value::Null);
```

If M2.7 produces `{"limit": 10,}`, `serde_json::from_str` fails and the entire argument set becomes `Null`. The tool then fails with "Missing required parameter."

### Problem

M2.7 produces invalid JSON with trailing commas approximately 15-20% of the time. This is a known MiniMax quirk. The `unwrap_or(Null)` fallback silently drops ALL arguments.

### Recommended Changes

Add a `sanitize_json_string` function that runs BEFORE `serde_json::from_str`:

```rust
fn sanitize_json_string(raw: &str) -> String {
    let mut s = raw.to_string();

    // 1. Strip trailing commas before } or ]
    //    {"limit": 10,} -> {"limit": 10}
    //    [1, 2, 3,] -> [1, 2, 3]
    let re_trailing_comma = regex::Regex::new(r",\s*([}\]])").unwrap();
    s = re_trailing_comma.replace_all(&s, "$1").to_string();

    // 2. Unquote integer values that M2.7 sometimes wraps in strings
    //    {"limit": "10"} -> leave as-is (let as_u64/as_i64 handle downstream)
    //    Actually: don't do this, it's safer to handle in type coercion layer

    s
}
```

Additionally, add **type coercion** in the tool handlers (or a shared utility):

```rust
/// Extract an integer from a JSON value, handling string-encoded numbers
/// and trailing-comma artifacts.
fn coerce_to_u64(val: &serde_json::Value) -> Option<u64> {
    // Direct integer
    if let Some(n) = val.as_u64() {
        return Some(n);
    }
    // String-encoded integer (M2.7 sometimes quotes numbers)
    if let Some(s) = val.as_str() {
        let cleaned = s.trim().trim_end_matches(',');
        return cleaned.parse::<u64>().ok();
    }
    // Float truncated to int
    if let Some(f) = val.as_f64() {
        return Some(f as u64);
    }
    None
}
```

**Where to add**:
- `sanitize_json_string`: In `think_loop.rs`, apply to `tc.function.arguments` before `from_str`
- `coerce_to_u64`: Utility in `codex-exec` or `codex-llm`, used by any handler that expects integers (grep `head_limit`, memory_search `limit`, hub tools `limit`)

### Flow After Changes

```
M2.7 emits: {"limit": 10,}
  -> sanitize_json_string -> {"limit": 10}
  -> serde_json::from_str -> Value::Object({"limit": 10})
  -> normalize_args -> (no aliases to remap)
  -> handler -> args.get("limit").and_then(coerce_to_u64) -> Some(10)
```

### Also Handle: Arguments as serde_json::Value vs String

The native Ollama API returns `arguments` as a JSON object (`serde_json::Value`), not a string. But the internal `FunctionCall` struct stores `arguments: String`. In `convert_native_response` (ollama.rs line 420):

```rust
arguments: serde_json::to_string(&tc.function.arguments).unwrap_or_default(),
```

Then in `think_loop.rs` line 260, it's parsed back:
```rust
let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)...
```

This round-trip (Value -> String -> Value) is where trailing commas could enter if M2.7 returns malformed data that serde_json stringifies faithfully. The sanitization MUST happen on the string form at line 260 before re-parsing, or alternatively on the raw native response before the `convert_native_response` call. The line-260 approach is simpler.

---

## 4. Few-Shot Examples in System Prompt

### Current State

No few-shot examples exist anywhere in the prompts.

### Problem

M2.7 benefits significantly from concrete examples. Without them, it guesses the tool call format based on its pretraining, which includes multiple incompatible formats (OpenAI, Anthropic, custom).

### Recommended Changes

Add role-specific few-shot examples to the system prompt. Keep them minimal (2-3 examples) to avoid wasting context budget.

For **Agent** minds (which have bash, read, write, glob, grep):

```
## Tool Call Examples

Here are examples of correct tool usage:

TASK: "Read the config file at /home/user/config.toml"
CORRECT: Call the `read` tool with {"file_path": "/home/user/config.toml"}

TASK: "List all Rust files in the project"
CORRECT: Call the `glob` tool with {"pattern": "*.rs"}

TASK: "Search for 'TODO' in all files"
CORRECT: Call the `grep` tool with {"pattern": "TODO"}

WRONG approaches (DO NOT do these):
- Writing "I'll read the file now..." without a tool call
- Calling read with {"path": "/home/user/config.toml"} (wrong parameter: use "file_path", not "path")
- Calling grep with {"file_path": "/src"} (wrong parameter: use "path", not "file_path")
```

For **TeamLead** minds (which have spawn_agent, delegate_to_agent, etc.):

```
## Tool Call Examples

TASK: "Research the Hub API"
CORRECT: Call `spawn_agent` with {"task": "Research the Hub API endpoints and document them"}

TASK: "Check memory for prior work"
CORRECT: Call `memory_search` with {"query": "Hub API"}
```

**Where to inject**: In the role-specific prompt constants, or as an additional `add_context` block in `PromptBuilder`.

**Key insight**: The examples must use the EXACT parameter names from the schema. This is the primary pedagogical purpose -- teaching M2.7 which parameter name goes with which tool.

---

## 5. Tool Schema Simplification

### Current State

Tool count by role:
- **Agent**: 5 built-in (bash, read, write, glob, grep) + 2 memory + 2 scratchpad + 6 Hub = **15 tools**
- **TeamLead**: 3 delegation + 2 memory + 2 scratchpad + 6 Hub = **13 tools**
- **Primary**: 4 coordination + 2 memory + 2 scratchpad = **8 tools**

### Assessment

The schemas are actually quite clean already. Each has:
- `type: "object"` with explicit `properties`
- Clear `required` arrays
- Short descriptions

However, some schemas could be tighter:

### Recommended Changes

**A. Remove optional parameters from schemas for M2.7**

M2.7 gets confused by optional parameters and often fills them with wrong values or wrong types. For lightweight/M2.7 tasks, consider emitting simplified schemas:

| Tool | Current Optional Params | Recommendation |
|------|------------------------|----------------|
| `read` | `offset`, `limit` | Keep -- frequently useful |
| `grep` | `path`, `output_mode`, `context`, `head_limit` | Remove `output_mode` and `context` for M2.7 (defaults are fine) |
| `bash` | `timeout` | Remove for M2.7 (default 120s is always fine) |
| `hub_list_threads` | `limit` | Keep but mark default prominently |
| `hub_feed` | `limit` | Keep but mark default prominently |

**B. Add default values to descriptions**

Change from:
```json
"description": "Maximum number of threads to return (default 10)"
```
To:
```json
"description": "Max threads to return. Default: 10. Pass an integer, not a string."
```

The type reminder ("Pass an integer, not a string") directly addresses M2.7's habit of quoting numbers.

**C. Consider a model-aware schema filter**

In `OllamaClient::tool_schemas()`, add an option to produce simplified schemas when the target model is M2.7:

```rust
pub fn tool_schemas_for_model(definitions: &[ToolDefinition], model: &str) -> Vec<ToolSchema> {
    let simplify = model.contains("minimax") || model.contains("m2.7");
    definitions.iter().map(|d| {
        let params = if simplify {
            strip_optional_params(&d.parameters)
        } else {
            d.parameters.clone()
        };
        ToolSchema { ... }
    }).collect()
}
```

This is a low-priority optimization. The parameter aliasing + sanitization (sections 2-3) handle most issues without reducing capability.

---

## 6. Hub Tool Descriptions

### Current State

Hub tool descriptions in `hub_interceptor.rs` are generic:
- `"UUID of the group"` -- no explicit warning about names vs UUIDs
- `"UUID of the room"` -- same

The `validate_uuid` function catches bad UUIDs at runtime, but M2.7 still wastes a turn.

### Problem

M2.7 sees a description like "UUID of the group" and passes `"civsubstrate"` because it knows that's a group name and doesn't understand the UUID requirement.

### Recommended Changes

Make UUID requirements unmistakable in EVERY Hub tool description:

```json
"group_id": {
    "type": "string",
    "description": "Group UUID (36-char format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx). NOT the group name. Example: c8eba770-a055-4281-88ad-6aed146ecf72"
}
```

```json
"room_id": {
    "type": "string",
    "description": "Room UUID (36-char format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx). NOT the room name. Get room UUIDs from hub_list_rooms."
}
```

```json
"thread_id": {
    "type": "string",
    "description": "Thread UUID (36-char format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx). NOT the thread title. Get thread UUIDs from hub_list_threads."
}
```

**Key additions**:
1. The format pattern (36-char with dashes) -- visual pattern matching helps M2.7
2. "NOT the group/room/thread name" -- explicit negative instruction
3. "Get UUIDs from [other tool]" -- teaches the lookup-first workflow

### Additionally: Inject Known UUIDs into Context

For frequently-used groups, inject their UUIDs into the system prompt or AGENTS.md:

```
## Known Hub Groups
- CivSubstrate WG: c8eba770-a055-4281-88ad-6aed146ecf72
- CivOS WG general: 6085176d-6223-4dd5-aa88-56895a54b07a
- PureBrain: 27bf21b7-0624-4bfa-9848-f1a0ff20ba27
```

This eliminates the need for a lookup call for common groups.

---

## 7. Reasoning Mode Analysis

### Current State

`config.toml` line 25: `lightweight_model = "minimax-m2.7"` -- used for red team, memory scoring, trivial planning.

The config has no `reasoning_split` setting. The Ollama native API response struct (ollama.rs line 204) has a `thinking` field but it's `#[allow(dead_code)]` and unused.

Devstral is used for primary/team-lead/agent roles. M2.7 is the lightweight model.

### Assessment

**Reasoning/thinking mode should be OFF for M2.7 tool calling.** Here's why:

1. M2.7's thinking mode (if available) consumes output budget. With `max_tokens: 2048` for lightweight tasks, thinking could eat 500-1000 tokens, leaving insufficient room for tool call JSON.

2. M2.7 is used for structured tasks (red team checks, memory scoring) where explicit chain-of-thought adds latency without improving accuracy. The task is "score this memory 0-1" not "reason deeply about memory relevance."

3. The `temperature: 0.3` setting for lightweight tasks already constrains creativity. Adding thinking mode on top would further slow responses.

**Recommendation**: Keep thinking mode OFF for M2.7. If Ollama Cloud ever adds a `/no_think` parameter for M2.7, use it explicitly. Currently, since M2.7's thinking field goes unused in `convert_native_response`, no code change is needed.

For **Devstral** (the primary model), thinking mode is already disabled by model architecture (Mistral models don't have a thinking mode). This is why Devstral was chosen -- "native tool calling, no thinking overhead" per config.toml comments.

---

## 8. Error Recovery and Re-prompting

### Current State

When a tool call fails (wrong parameter name, invalid UUID, etc.), the error message is injected as a `tool` role message (think_loop.rs line 323):

```rust
messages.push(ChatMessage::tool_result(tc.id.clone(), result_text));
```

For failures, `result_text` = `"Error: Missing 'file_path' parameter"`.

The LLM then sees this error and can retry. However, M2.7 often does NOT retry -- it either gives up or claims completion despite the error.

### Recommended Changes

**A. Enhanced error messages with correction hints**

Instead of bare error messages, include the correct parameter name:

```
Error: Missing 'file_path' parameter.
HINT: The 'read' tool requires "file_path" (not "path" or "filepath").
Example: {"file_path": "/absolute/path/to/file.txt"}
```

This can be done in each tool handler's error path, or in a centralized error-enrichment layer.

**B. Add a retry-encouraging prompt after tool errors**

In `think_loop.rs`, after injecting a tool error result, add a user message nudge:

```rust
if !result.success {
    messages.push(ChatMessage::user(
        "The tool call failed. Please read the error message, fix the issue, and try again."
    ));
}
```

This directly counters M2.7's tendency to give up after one failed tool call.

**C. Detect the prose-about-tools failure mode**

When the LLM returns text content (not tool calls) that contains patterns like "I would call", "Let me use the", "Using the bash tool", detect this and re-prompt:

```rust
let prose_tool_re = Regex::new(r"(?i)I (?:would|will|shall|'ll) (?:call|use|invoke|run) (?:the )?\w+ tool").unwrap();

if prose_tool_re.is_match(&response) && !all_schemas.is_empty() {
    messages.push(ChatMessage::assistant(&response));
    messages.push(ChatMessage::user(
        "You described a tool call in prose instead of actually calling it. \
         Please emit the actual tool call now. Do not describe it -- just call it."
    ));
    continue; // Go around the loop again
}
```

**Where to add**: In `think_loop.rs`, after the final `extract_content` but before the `return Ok(ThinkResult { ... })`.

**Guard against infinite loops**: Only allow this re-prompt once per iteration (use a counter).

---

## Implementation Priority

| Priority | Change | Impact | Effort | Files |
|----------|--------|--------|--------|-------|
| **P0** | JSON sanitization (trailing commas) | Fixes ~15-20% of tool call failures | Low | `think_loop.rs` |
| **P0** | Parameter normalization | Fixes wrong-name errors across all tools | Medium | `think_loop.rs` (new function) |
| **P1** | Hub tool UUID descriptions | Reduces wasted turns on Hub tools | Low | `hub_interceptor.rs` |
| **P1** | System prompt tool-use preamble | Reduces prose-about-tools and premature completion | Low | `prompt.rs` |
| **P1** | Error messages with correction hints | Improves retry success rate | Low | Each tool handler |
| **P2** | Few-shot examples | Teaches correct parameter names | Low | `prompt.rs` |
| **P2** | Retry-encouraging prompt after errors | Counters give-up behavior | Low | `think_loop.rs` |
| **P2** | Prose-about-tools detection | Catches narrator mode | Medium | `think_loop.rs` |
| **P3** | Model-aware schema simplification | Reduces cognitive load for M2.7 | Medium | `ollama.rs` |
| **P3** | Known UUID injection in prompts | Eliminates lookup calls for common groups | Low | AGENTS.md files |

---

## Appendix: Data Flow Diagram

```
M2.7 emits tool call
  |
  v
Ollama /api/chat response (native format)
  |  arguments: serde_json::Value (already parsed by Ollama)
  v
convert_native_response() [ollama.rs:402]
  |  Serializes arguments Value -> String for internal format
  v
FunctionCall { name, arguments: String }
  |
  v
ThinkLoop [think_loop.rs:260]
  |  1. sanitize_json_string(arguments)     <-- NEW: strip trailing commas
  |  2. serde_json::from_str -> Value
  |  3. normalize_args(tool_name, args)      <-- NEW: alias remapping
  v
Tool dispatch (interceptor -> memory -> executor)
  |
  v
Handler (e.g., read.rs, hub_interceptor.rs)
  |  args.get("file_path") with coerce helpers  <-- NEW: type coercion
  v
ToolResult { success, output, error }
  |
  v
Back to ThinkLoop as tool result message
  |  If error: enhanced message with hints   <-- NEW: correction hints
  |  If error: retry-encouraging nudge       <-- NEW: user message
  v
Next LLM turn
```

---

## Appendix: M2.7 Failure Examples (Observed)

### Trailing Comma
```json
{"query": "Hub API endpoints", "limit": 5,}
```
Fails `serde_json::from_str`, falls through to `Value::Null`, tool gets no arguments.

### Wrong Parameter Name
```json
{"path": "/home/user/config.toml"}
```
Sent to `read` tool, which expects `file_path`. Returns "Missing 'file_path' parameter".

### Group Name Instead of UUID
```json
{"group_id": "civsubstrate"}
```
Caught by `validate_uuid` but wastes a turn. M2.7 often does not retry.

### Narrator Mode
```
I'll search the memory for relevant context about the Hub API.

Based on my knowledge, the Hub API provides endpoints for...

Task complete!
```
No tool calls emitted. The model narrated what it "would" do and then claimed completion.

### Premature Completion
```
[calls memory_search with "Hub API"]
[receives results]

Done! I have searched the memory and found relevant information about the Hub API.
```
Only completed step 1 of a multi-step task.
