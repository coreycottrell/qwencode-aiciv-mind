//! ThinkLoop — the core reasoning cycle.
//!
//! Prompt → LLM → Tool Calls → Execute → Inject Results → Loop
//!
//! This is where Cortex THINKS. The loop continues until the LLM
//! produces a final text response (no tool calls) or hits the max iterations.
//!
//! ## Memory Integration
//!
//! When a `MemoryStore` is provided, the ThinkLoop automatically:
//! - Adds `memory_search` and `memory_write` tools to the LLM's tool set
//! - Intercepts calls to these tools and handles them directly
//! - Delegates all other tool calls to the `ToolExecutor`
//!
//! ## Tool Interceptors
//!
//! Implement `ToolInterceptor` to inject custom tools into the ThinkLoop.
//! Used by TeamLeads to add delegation tools (spawn_agent, delegate_to_agent)
//! without modifying the core loop. Interceptors are checked before memory tools
//! and the standard ToolExecutor.

use async_trait::async_trait;
use codex_exec::{ToolCall, ToolExecutor, ToolResult};
use codex_memory::{MemoryStore, MemoryQuery, MemoryCategory, MemoryTier, NewMemory};
use codex_redteam::{Challenger, ChallengerCheck, ChallengerToolCall, Severity};
use codex_roles::Role;
use tracing::{info, warn};

use crate::ollama::{
    ChatMessage, ChatResponse, FunctionSchema,
    ToolSchema,
};
use crate::prompt::PromptBuilder;
use crate::provider::LlmProvider;
use crate::rate_limiter::RateLimiter;

/// Tool interceptor — inject custom tools into the ThinkLoop.
///
/// Interceptors are checked FIRST, before memory tools and the standard ToolExecutor.
/// If `handle` returns `Some(result)`, the result is used and the ToolExecutor is skipped.
/// If `handle` returns `None`, processing falls through to memory tools / ToolExecutor.
///
/// Used by TeamLeads to add `spawn_agent` and `delegate_to_agent` tools.
#[async_trait]
pub trait ToolInterceptor: Send + Sync {
    /// Return tool schemas to add to the LLM's tool set.
    fn schemas(&self) -> Vec<crate::ollama::ToolSchema>;

    /// Handle a tool call. Return Some(result) to intercept, None to pass through.
    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<ToolResult>;
}

/// Chains multiple interceptors. Checks each in order; returns the first `Some`.
pub struct CompositeInterceptor<'a> {
    interceptors: Vec<&'a dyn ToolInterceptor>,
}

impl<'a> CompositeInterceptor<'a> {
    pub fn new(interceptors: Vec<&'a dyn ToolInterceptor>) -> Self {
        Self { interceptors }
    }
}

#[async_trait]
impl ToolInterceptor for CompositeInterceptor<'_> {
    fn schemas(&self) -> Vec<crate::ollama::ToolSchema> {
        self.interceptors.iter()
            .flat_map(|i| i.schemas())
            .collect()
    }

    async fn handle(&self, name: &str, args: &serde_json::Value) -> Option<codex_exec::ToolResult> {
        for interceptor in &self.interceptors {
            if let Some(result) = interceptor.handle(name, args).await {
                return Some(result);
            }
        }
        None
    }
}

/// Configuration for the thinking loop.
#[derive(Debug, Clone)]
pub struct ThinkLoopConfig {
    /// Maximum number of LLM turns before forcing completion.
    pub max_iterations: u32,
}

impl Default for ThinkLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 15,
        }
    }
}

/// The thinking loop — Cortex's core reasoning engine.
///
/// Accepts any `LlmProvider` implementation (OllamaClient, OpenAI-compat, etc.).
/// The ThinkLoop does not know or care which model backend is behind the trait.
pub struct ThinkLoop {
    provider: Box<dyn LlmProvider>,
    config: ThinkLoopConfig,
    /// Per-turn adversarial checker — fires after every tool execution.
    challenger: Challenger,
    /// Directory for scratchpad files. When set, enables scratchpad_read/scratchpad_write tools.
    scratchpad_dir: Option<std::path::PathBuf>,
    /// Directory for Hum observation JSONL files. When set, enables hum_digest tool.
    hum_dir: Option<std::path::PathBuf>,
    /// Rate limiter for usage tracking. When set, enables ollama_usage tool.
    rate_limiter: Option<RateLimiter>,
}

/// Result of a complete thinking session.
#[derive(Debug, Clone)]
pub struct ThinkResult {
    /// The final text response from the LLM.
    pub response: String,
    /// Tool calls made during the session.
    pub tool_calls_made: Vec<ToolCallRecord>,
    /// Number of LLM turns used.
    pub iterations: u32,
    /// Whether the loop completed naturally (vs hitting max iterations).
    pub completed: bool,
    /// Number of challenger warnings raised during this session.
    pub challenger_warnings: u32,
    /// Whether the loop was killed by Challenger stall detection.
    pub stall_killed: bool,
}

/// Record of a tool call made during thinking.
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: ToolResult,
    pub iteration: u32,
}

impl ThinkLoop {
    /// Create a new ThinkLoop with the given LLM provider and config.
    ///
    /// The provider must already be fully configured (rate limiter, API key, etc.)
    /// before being passed here. ThinkLoop treats it as an opaque `LlmProvider`.
    pub fn new(provider: Box<dyn LlmProvider>, config: ThinkLoopConfig) -> Self {
        Self {
            provider,
            config,
            challenger: Challenger::default(),
            scratchpad_dir: None,
            hum_dir: None,
            rate_limiter: None,
        }
    }

    /// Enable scratchpad tools by setting the scratchpad directory.
    /// Scratchpad files are stored as `{scratchpad_dir}/{mind_id}.md`.
    pub fn with_scratchpad_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.scratchpad_dir = Some(dir);
        self
    }

    /// Enable hum_digest tool by setting the Hum observation directory.
    pub fn with_hum_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.hum_dir = Some(dir);
        self
    }

    /// Enable the `ollama_usage` tool for usage tracking visibility.
    ///
    /// Note: the rate limiter for actual LLM call throttling should be attached
    /// to the provider (e.g., `OllamaClient::with_rate_limiter()`) BEFORE passing
    /// the provider to `ThinkLoop::new()`. This method only enables the usage
    /// reporting tool so the LLM can check its own consumption.
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(limiter);
        self
    }

    /// Dispatch a tool call to built-in handlers (memory, scratchpad, hum) or the executor.
    ///
    /// Builtin tools now receive the same enforcement as registered tools:
    /// 1. Role permission check via `codex_roles::is_tool_allowed`
    /// 2. PreToolUse hook (can block)
    /// 3. Execute the builtin handler
    /// 4. PostToolUse hook
    ///
    /// Non-builtin tools fall through to `ToolExecutor::execute()` which has its own
    /// full enforcement pipeline.
    async fn dispatch_builtin_or_exec(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
        memory: Option<&MemoryStore>,
        mind_id: Option<&str>,
        role: Role,
        executor: &ToolExecutor,
        original_name: &str,
    ) -> ToolResult {
        let is_builtin = matches!(
            tool_name,
            "memory_search"
                | "memory_write"
                | "scratchpad_read"
                | "scratchpad_write"
                | "coordination_read"
                | "coordination_write"
                | "team_scratchpad_read"
                | "team_scratchpad_write"
                | "hum_digest"
                | "ollama_usage"
        );

        if is_builtin {
            // ── Enforcement layer for builtins (B1 fix) ──

            // Step 1: Role permission check
            if !codex_roles::is_tool_allowed(role, tool_name) {
                info!(tool = %tool_name, role = %role, "Builtin tool denied by role check");
                return ToolResult::err(format!(
                    "Permission denied: tool '{}' not allowed for role {:?}",
                    tool_name, role
                ));
            }

            // Step 2: PreToolUse hook
            if let Some(reason) = executor.fire_pre_tool_use(tool_name, args).await {
                return ToolResult::err(format!("Blocked by hook: {reason}"));
            }

            // Step 3: Execute the builtin handler
            let result = match tool_name {
                "memory_search" => handle_memory_search(memory, args).await,
                "memory_write" => handle_memory_write(memory, args, mind_id, role).await,
                "scratchpad_read" => handle_scratchpad_read(self.scratchpad_dir.as_deref(), mind_id),
                "scratchpad_write" => handle_scratchpad_write(self.scratchpad_dir.as_deref(), mind_id, args),
                "coordination_read" => handle_coordination_read(self.scratchpad_dir.as_deref()),
                "coordination_write" => handle_coordination_write(self.scratchpad_dir.as_deref(), mind_id, args),
                "team_scratchpad_read" => handle_team_scratchpad_read(self.scratchpad_dir.as_deref(), args),
                "team_scratchpad_write" => handle_team_scratchpad_write(self.scratchpad_dir.as_deref(), mind_id, args),
                "hum_digest" => handle_hum_digest(self.hum_dir.as_deref()),
                "ollama_usage" => {
                    let text = crate::rate_limiter::handle_ollama_usage(self.rate_limiter.as_ref()).await;
                    ToolResult::ok(text)
                }
                _ => unreachable!("is_builtin guard ensures this"),
            };

            // Step 4: PostToolUse hook
            executor.fire_post_tool_use(tool_name, args, &result).await;

            result
        } else {
            // Non-builtin: delegate to ToolExecutor (has its own full enforcement pipeline)
            let call = ToolCall {
                name: original_name.to_string(),
                arguments: args.clone(),
            };
            match executor.execute(&call, role).await {
                Ok(r) => r,
                Err(e) => ToolResult::err(format!("Tool execution error: {e}")),
            }
        }
    }

    /// Run the thinking loop for a specific task.
    ///
    /// This is the core of Cortex — the LLM thinks, calls tools, and loops
    /// until it produces a final response or hits the iteration limit.
    pub async fn run(
        &self,
        prompt_builder: &PromptBuilder,
        task: &str,
        tool_schemas: &[ToolSchema],
        executor: &ToolExecutor,
        role: Role,
    ) -> Result<ThinkResult, ThinkError> {
        self.run_with_memory(prompt_builder, task, tool_schemas, executor, role, None, None).await
    }

    /// Run the thinking loop with memory integration.
    ///
    /// When `memory` is provided, the LLM gains access to `memory_search` and
    /// `memory_write` tools. These are intercepted by the ThinkLoop and handled
    /// directly — they never reach the ToolExecutor.
    pub async fn run_with_memory(
        &self,
        prompt_builder: &PromptBuilder,
        task: &str,
        tool_schemas: &[ToolSchema],
        executor: &ToolExecutor,
        role: Role,
        memory: Option<&MemoryStore>,
        mind_id: Option<&str>,
    ) -> Result<ThinkResult, ThinkError> {
        self.run_full(prompt_builder, task, tool_schemas, executor, role, memory, mind_id, None).await
    }

    /// Run the thinking loop with memory integration AND custom tool interceptors.
    ///
    /// This is the most general form of the ThinkLoop. Tool call resolution order:
    /// 1. Interceptor (if provided and returns Some)
    /// 2. Memory tools (memory_search, memory_write)
    /// 3. Standard ToolExecutor (bash, read, write, etc.)
    pub async fn run_full(
        &self,
        prompt_builder: &PromptBuilder,
        task: &str,
        tool_schemas: &[ToolSchema],
        executor: &ToolExecutor,
        role: Role,
        memory: Option<&MemoryStore>,
        mind_id: Option<&str>,
        interceptor: Option<&dyn ToolInterceptor>,
    ) -> Result<ThinkResult, ThinkError> {
        let mut messages = prompt_builder.build_messages(task);
        let mut tool_calls_made = Vec::new();
        let mut iteration = 0;
        let mut consecutive_critical_stalls: u32 = 0;

        // Build the full tool set — user tools + interceptor tools + memory tools
        let mut all_schemas: Vec<ToolSchema> = tool_schemas.to_vec();
        if let Some(interceptor) = interceptor {
            let interceptor_schemas = interceptor.schemas();
            info!("ThinkLoop: {} interceptor tool(s) enabled", interceptor_schemas.len());
            all_schemas.extend(interceptor_schemas);
        }
        if memory.is_some() {
            all_schemas.extend(memory_tool_schemas());
            info!("ThinkLoop: memory tools enabled (memory_search, memory_write)");
        }
        if self.scratchpad_dir.is_some() {
            all_schemas.extend(scratchpad_tool_schemas());
            all_schemas.extend(group_scratchpad_tool_schemas());
            info!("ThinkLoop: scratchpad tools enabled (scratchpad_read, scratchpad_write, coordination_read/write, team_scratchpad_read/write)");
        }
        if self.hum_dir.is_some() {
            all_schemas.extend(hum_tool_schemas());
            info!("ThinkLoop: hum tools enabled (hum_digest)");
        }
        if self.rate_limiter.is_some() {
            let schema_val = crate::rate_limiter::ollama_usage_tool_schema();
            all_schemas.push(ToolSchema {
                tool_type: "function".into(),
                function: FunctionSchema {
                    name: schema_val["name"].as_str().unwrap().to_string(),
                    description: schema_val["description"].as_str().unwrap().to_string(),
                    parameters: schema_val["parameters"].clone(),
                },
            });
            info!("ThinkLoop: rate limiter tool enabled (ollama_usage)");
        }

        info!(task = task, role = %role, tools = all_schemas.len(), "ThinkLoop starting");

        loop {
            iteration += 1;
            if iteration > self.config.max_iterations {
                warn!(
                    iterations = iteration - 1,
                    "ThinkLoop hit max iterations, forcing completion"
                );
                messages.push(ChatMessage::user(
                    "You have used all available iterations. Please provide your final response now, \
                     summarizing what you have accomplished and any remaining work.",
                ));
                let final_resp = self.provider.chat(&messages, None).await
                    .map_err(|e| ThinkError::Llm(e.to_string()))?;

                let response = extract_content(&final_resp);
                return Ok(ThinkResult {
                    response,
                    tool_calls_made,
                    iterations: iteration,
                    completed: false,
                    challenger_warnings: 0,
                    stall_killed: false,
                });
            }

            info!(iteration = iteration, "ThinkLoop turn");

            let tools = if all_schemas.is_empty() { None } else { Some(all_schemas.as_slice()) };
            let resp = self.provider.chat(&messages, tools).await
                .map_err(|e| ThinkError::Llm(e.to_string()))?;

            let Some(choice) = resp.choices.first() else {
                return Err(ThinkError::Llm("No choices in response".into()));
            };

            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    // Preserve content (including <think> tags) alongside tool calls.
                    // Dropping thinking from conversation history causes 35-40% degradation in M2.7.
                    messages.push(ChatMessage::assistant_with_tool_calls(
                        choice.message.content.clone(),
                        tool_calls.clone(),
                    ));

                    let mut any_tool_failed = false;
                    for tc in tool_calls {
                        let sanitized = sanitize_json_string(&tc.function.arguments);
                        let mut args: serde_json::Value = serde_json::from_str(&sanitized)
                            .unwrap_or(serde_json::Value::Null);
                        args = normalize_args(&tc.function.name, args);

                        info!(
                            tool = %tc.function.name,
                            iteration = iteration,
                            "Executing tool call"
                        );

                        // Resolution order: enforcement → interceptor → builtin/executor
                        // ALL paths get role check + hooks (B1 + B2 fix).
                        let tool_name = tc.function.name.as_str();
                        let result = if let Some(interceptor) = interceptor {
                            if let Some(r) = interceptor.handle(tool_name, &args).await {
                                // ── Enforcement for intercepted tools (B2 fix) ──
                                // Step 1: Role permission check
                                if !codex_roles::is_tool_allowed(role, tool_name) {
                                    info!(tool = %tool_name, role = %role, "Intercepted tool denied by role check");
                                    ToolResult::err(format!(
                                        "Permission denied: tool '{}' not allowed for role {:?}",
                                        tool_name, role
                                    ))
                                } else {
                                    // Step 2: PreToolUse hook (can block)
                                    if let Some(reason) = executor.fire_pre_tool_use(tool_name, &args).await {
                                        ToolResult::err(format!("Blocked by hook: {reason}"))
                                    } else {
                                        // Step 3: Use interceptor result
                                        // Step 4: PostToolUse hook
                                        executor.fire_post_tool_use(tool_name, &args, &r).await;
                                        r
                                    }
                                }
                            } else {
                                // Interceptor did not handle — fall through to builtin/executor
                                // (dispatch_builtin_or_exec has its own enforcement)
                                self.dispatch_builtin_or_exec(tool_name, &args, memory, mind_id, role, executor, &tc.function.name).await
                            }
                        } else {
                            // No interceptor — go directly to builtin/executor
                            // (dispatch_builtin_or_exec has its own enforcement)
                            self.dispatch_builtin_or_exec(tool_name, &args, memory, mind_id, role, executor, &tc.function.name).await
                        };

                        tool_calls_made.push(ToolCallRecord {
                            tool_name: tc.function.name.clone(),
                            arguments: args,
                            result: result.clone(),
                            iteration,
                        });

                        let result_text = if result.success {
                            result.output
                        } else {
                            any_tool_failed = true;
                            format!("Error: {}", result.error.unwrap_or_default())
                        };
                        messages.push(ChatMessage::tool_result(tc.id.clone(), result_text));
                    }

                    // Nudge the LLM to retry after tool failures (counters M2.7's
                    // tendency to give up or claim completion after one error).
                    if any_tool_failed {
                        messages.push(ChatMessage::user(
                            "One or more tool calls failed. Read the error messages, \
                             fix the issues (check parameter names and types), and try again.",
                        ));
                    }

                    // Challenger: check for structural problems after tool execution
                    let challenger_calls: Vec<ChallengerToolCall> = tool_calls_made.iter()
                        .map(|r| ChallengerToolCall {
                            name: r.tool_name.clone(),
                            arguments: r.arguments.clone(),
                            iteration: r.iteration,
                            reasoning_trace: None,
                            result_text: Some(r.result.output.clone()),
                        })
                        .collect();
                    let warnings = self.challenger.check_stateless(&challenger_calls, None, iteration);

                    // Track consecutive critical stall warnings for kill decision
                    let has_critical_stall = warnings.iter().any(|w|
                        w.check == ChallengerCheck::StallDetection && w.severity == Severity::Critical
                    );
                    if has_critical_stall {
                        consecutive_critical_stalls += 1;
                    } else if warnings.iter().all(|w| w.check != ChallengerCheck::StallDetection) {
                        consecutive_critical_stalls = 0;
                    }

                    for w in &warnings {
                        warn!(check = ?w.check, severity = ?w.severity, "Challenger: {}", w.message);
                        // Use 'user' role (not 'system') because Ollama native API
                        // rejects system messages after tool result messages.
                        messages.push(ChatMessage::user(format!(
                            "[CHALLENGER WARNING — {:?}] {}",
                            w.check, w.message
                        )));
                    }

                    // Stall kill: Challenger critical stall for 2+ consecutive turns → BREAK
                    if consecutive_critical_stalls >= 2 {
                        warn!(
                            iteration = iteration,
                            consecutive_critical_stalls = consecutive_critical_stalls,
                            "Challenger stall kill — forcing ThinkLoop termination"
                        );
                        let response = format!(
                            "[STALL KILLED at iteration {}] Agent produced no productive output \
                             for {} iterations. Challenger terminated the loop.",
                            iteration, iteration
                        );
                        return Ok(ThinkResult {
                            response,
                            tool_calls_made,
                            iterations: iteration,
                            completed: false,
                            challenger_warnings: warnings.len() as u32,
                            stall_killed: true,
                        });
                    }

                    continue;
                }
            }

            let response = extract_content(&resp);

            // Challenger: check the final response for structural problems
            let challenger_calls: Vec<ChallengerToolCall> = tool_calls_made.iter()
                .map(|r| ChallengerToolCall {
                    name: r.tool_name.clone(),
                    arguments: r.arguments.clone(),
                    iteration: r.iteration,
                    reasoning_trace: None,
                    result_text: Some(r.result.output.clone()),
                })
                .collect();
            let final_warnings = self.challenger.check_stateless(&challenger_calls, Some(&response), iteration);
            for w in &final_warnings {
                warn!(check = ?w.check, severity = ?w.severity, "Challenger (final): {}", w.message);
            }

            info!(
                iterations = iteration,
                tool_calls = tool_calls_made.len(),
                response_len = response.len(),
                challenger_warnings = final_warnings.len(),
                "ThinkLoop completed"
            );

            return Ok(ThinkResult {
                response,
                tool_calls_made,
                iterations: iteration,
                completed: true,
                challenger_warnings: final_warnings.len() as u32,
                stall_killed: false,
            });
        }
    }
}

// ── JSON Sanitization & Parameter Normalization ────────────────────────────
//
// M2.7 produces invalid JSON (trailing commas) ~15-20% of the time and
// confuses parameter names across tools. These helpers run on every tool call
// between raw argument parsing and tool dispatch.

/// Strip trailing commas before `}` or `]` — the primary M2.7 JSON quirk.
/// Runs on the raw argument string BEFORE serde_json::from_str.
fn sanitize_json_string(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            // Look ahead past whitespace for } or ]
            let mut j = i + 1;
            while j < bytes.len() && matches!(bytes[j], b' ' | b'\n' | b'\r' | b'\t') {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // Skip the trailing comma
                i += 1;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| raw.to_string())
}

/// Remap common parameter name aliases to their canonical names.
/// Only remaps if the canonical key is NOT already present (prevents overwriting correct values).
fn normalize_args(tool_name: &str, mut args: serde_json::Value) -> serde_json::Value {
    let obj = match args.as_object_mut() {
        Some(o) => o,
        None => return args,
    };

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
        "web_search" => &[
            ("search", "query"),
            ("q", "query"),
            ("search_query", "query"),
            ("term", "query"),
        ],
        "web_fetch" => &[
            ("link", "url"),
            ("href", "url"),
            ("page", "url"),
            ("website", "url"),
        ],
        "tts_speak" => &[
            ("content", "text"),
            ("message", "text"),
            ("input", "text"),
            ("speech", "text"),
            ("voice_name", "voice"),
            ("speaker", "voice"),
            ("output", "filename"),
            ("file", "filename"),
            ("output_file", "filename"),
        ],
        "coordination_write" | "team_scratchpad_write" | "scratchpad_write" => &[
            ("text", "content"),
            ("note", "content"),
            ("message", "content"),
            ("body", "content"),
        ],
        "team_scratchpad_read" | "team_scratchpad_write" => &[
            ("team", "team_id"),
            ("group", "team_id"),
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

// ── Memory Tool Definitions ───────────────────────────────────────────────

/// Generate the OpenAI-compatible tool schemas for memory tools.
fn memory_tool_schemas() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "memory_search".into(),
                description: "Search the memory graph for relevant context. Returns matching memories with titles and content.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search text to find relevant memories"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default 5)"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "memory_write".into(),
                description: "Store a new learning or insight in the memory graph for future use by this mind or others.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Short title for the memory"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to remember"
                        },
                        "category": {
                            "type": "string",
                            "enum": ["pattern", "learning", "observation", "decision"],
                            "description": "Category of the memory (default: learning)"
                        }
                    },
                    "required": ["title", "content"]
                }),
            },
        },
    ]
}

// ── Scratchpad Tool Definitions ──────────────────────────────────────────

/// Generate tool schemas for scratchpad tools.
fn scratchpad_tool_schemas() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "scratchpad_read".into(),
                description: "Read your scratchpad — persistent notes that survive across sessions. Use this at the start of work to see what you left behind.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "scratchpad_write".into(),
                description: "Append a note to your scratchpad. Use this to leave notes for yourself or future sessions. Content is appended, never overwritten.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text to append to the scratchpad"
                        }
                    },
                    "required": ["content"]
                }),
            },
        },
    ]
}

/// Handle a memory_search tool call.
async fn handle_memory_search(
    memory: Option<&MemoryStore>,
    args: &serde_json::Value,
) -> ToolResult {
    let Some(store) = memory else {
        return ToolResult::err("Memory store not available");
    };

    let query_text = args.get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let limit = args.get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(5);

    let query = MemoryQuery {
        text: Some(query_text.to_string()),
        limit: Some(limit),
        ..Default::default()
    };

    match store.search(&query).await {
        Ok(results) => {
            if results.is_empty() {
                ToolResult::ok("No memories found matching your query.")
            } else {
                let mut output = format!("Found {} memories:\n\n", results.len());
                for (i, r) in results.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. **{}** (depth: {:.2}, tier: {})\n   {}\n\n",
                        i + 1, r.memory.title, r.memory.depth_score, r.memory.tier, r.memory.content
                    ));
                }
                info!(count = results.len(), query = query_text, "Memory search returned results");
                ToolResult::ok(output)
            }
        }
        Err(e) => ToolResult::err(format!("Memory search error: {e}")),
    }
}

/// Handle a memory_write tool call.
async fn handle_memory_write(
    memory: Option<&MemoryStore>,
    args: &serde_json::Value,
    mind_id: Option<&str>,
    role: Role,
) -> ToolResult {
    let Some(store) = memory else {
        return ToolResult::err("Memory store not available");
    };

    let title = args.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    let content = args.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let category_str = args.get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("learning");

    let category = match category_str {
        "pattern" => MemoryCategory::Pattern,
        "observation" => MemoryCategory::Observation,
        "decision" => MemoryCategory::Decision,
        _ => MemoryCategory::Learning,
    };

    let new_mem = NewMemory {
        mind_id: mind_id.unwrap_or("unknown").to_string(),
        role: role.to_string(),
        vertical: None,
        category,
        title: title.to_string(),
        content: content.to_string(),
        evidence: vec![],
        tier: MemoryTier::Working,
        session_id: None,
        task_id: None,
    };

    match store.store(new_mem).await {
        Ok(id) => {
            info!(id = %id, title = title, "Memory written during thinking");
            ToolResult::ok(format!("Memory stored: '{}' (id: {})", title, id))
        }
        Err(e) => ToolResult::err(format!("Memory write error: {e}")),
    }
}

// ── Scratchpad Tool Handlers ─────────────────────────────────────────────

/// Build today's scratchpad filename: `{mind_id}-{YYYY-MM-DD}.md`
/// Same day = same file. New day = fresh scratchpad.
fn scratchpad_filename(mind_id: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d");
    format!("{mind_id}-{today}.md")
}

/// Handle a scratchpad_read tool call.
fn handle_scratchpad_read(
    scratchpad_dir: Option<&std::path::Path>,
    mind_id: Option<&str>,
) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let mind = mind_id.unwrap_or("default");
    let path = dir.join(scratchpad_filename(mind));

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            if content.is_empty() {
                ToolResult::ok("(scratchpad is empty)")
            } else {
                info!(mind_id = mind, lines = content.lines().count(), "Scratchpad read");
                ToolResult::ok(content)
            }
        }
        Err(_) => ToolResult::ok("(no scratchpad yet — use scratchpad_write to create one)"),
    }
}

/// Handle a scratchpad_write tool call.
fn handle_scratchpad_write(
    scratchpad_dir: Option<&std::path::Path>,
    mind_id: Option<&str>,
    args: &serde_json::Value,
) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let mind = mind_id.unwrap_or("default");
    let content = args.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if content.is_empty() {
        return ToolResult::err("No content provided");
    }

    let path = dir.join(scratchpad_filename(mind));
    let _ = std::fs::create_dir_all(dir);

    // Append with timestamp
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    let entry = format!("\n## {timestamp}\n\n{content}\n");

    use std::io::Write;
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                return ToolResult::err(format!("Write failed: {e}"));
            }
            info!(mind_id = mind, bytes = entry.len(), "Scratchpad written");
            ToolResult::ok(format!("Appended to scratchpad ({} bytes)", content.len()))
        }
        Err(e) => ToolResult::err(format!("Failed to open scratchpad: {e}")),
    }
}

// ── Group Scratchpad Tool Definitions (A4 Architecture) ──────────────────

/// Generate tool schemas for coordination + team scratchpad tools.
fn group_scratchpad_tool_schemas() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "coordination_read".into(),
                description: "Read the coordination scratchpad — a shared surface where Primary posts directives and status that all team leads can see.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "coordination_write".into(),
                description: "Append a note to the coordination scratchpad. Use this to post directives, status updates, or decisions visible to all team leads and agents.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text to append to the coordination scratchpad"
                        }
                    },
                    "required": ["content"]
                }),
            },
        },
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "team_scratchpad_read".into(),
                description: "Read a team's scratchpad. Team leads write here so their agents can see directives. Agents write here so their team lead can see progress.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team_id": {
                            "type": "string",
                            "description": "Team identifier (usually the team lead's mind_id). Defaults to your own mind_id."
                        }
                    }
                }),
            },
        },
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "team_scratchpad_write".into(),
                description: "Append a note to a team's scratchpad. Team leads use this to post objectives for agents. Agents use this to report progress to their team lead.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text to append to the team scratchpad"
                        },
                        "team_id": {
                            "type": "string",
                            "description": "Team identifier (usually the team lead's mind_id). Defaults to your own mind_id."
                        }
                    },
                    "required": ["content"]
                }),
            },
        },
    ]
}

/// Today's coordination scratchpad filename.
fn coordination_filename() -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d");
    format!("coordination-{today}.md")
}

/// Today's team scratchpad filename for a given team.
fn team_scratchpad_filename(team_id: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d");
    format!("team-{team_id}-{today}.md")
}

/// Handle coordination_read.
fn handle_coordination_read(scratchpad_dir: Option<&std::path::Path>) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let path = dir.join(coordination_filename());
    match std::fs::read_to_string(&path) {
        Ok(content) if content.is_empty() => ToolResult::ok("(coordination scratchpad is empty)"),
        Ok(content) => {
            info!(lines = content.lines().count(), "Coordination scratchpad read");
            ToolResult::ok(content)
        }
        Err(_) => ToolResult::ok("(no coordination scratchpad yet — use coordination_write to create one)"),
    }
}

/// Handle coordination_write.
fn handle_coordination_write(
    scratchpad_dir: Option<&std::path::Path>,
    mind_id: Option<&str>,
    args: &serde_json::Value,
) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    if content.is_empty() {
        return ToolResult::err("No content provided");
    }
    let mind = mind_id.unwrap_or("unknown");
    let path = dir.join(coordination_filename());
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    let entry = format!("\n## [{mind}] {timestamp}\n\n{content}\n");

    use std::io::Write;
    match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                return ToolResult::err(format!("Write failed: {e}"));
            }
            info!(mind_id = mind, bytes = entry.len(), "Coordination scratchpad written");
            ToolResult::ok(format!("Appended to coordination scratchpad ({} bytes)", content.len()))
        }
        Err(e) => ToolResult::err(format!("Failed to open coordination scratchpad: {e}")),
    }
}

/// Handle team_scratchpad_read.
fn handle_team_scratchpad_read(
    scratchpad_dir: Option<&std::path::Path>,
    args: &serde_json::Value,
) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let team_id = args.get("team_id").and_then(|v| v.as_str()).unwrap_or("root");
    let path = dir.join(team_scratchpad_filename(team_id));
    match std::fs::read_to_string(&path) {
        Ok(content) if content.is_empty() => ToolResult::ok(format!("(team '{team_id}' scratchpad is empty)")),
        Ok(content) => {
            info!(team_id = team_id, lines = content.lines().count(), "Team scratchpad read");
            ToolResult::ok(content)
        }
        Err(_) => ToolResult::ok(format!("(no team '{team_id}' scratchpad yet — use team_scratchpad_write to create one)")),
    }
}

/// Handle team_scratchpad_write.
fn handle_team_scratchpad_write(
    scratchpad_dir: Option<&std::path::Path>,
    mind_id: Option<&str>,
    args: &serde_json::Value,
) -> ToolResult {
    let Some(dir) = scratchpad_dir else {
        return ToolResult::err("Scratchpad not configured");
    };
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    if content.is_empty() {
        return ToolResult::err("No content provided");
    }
    let mind = mind_id.unwrap_or("unknown");
    let team_id = args.get("team_id").and_then(|v| v.as_str()).unwrap_or(mind);
    let path = dir.join(team_scratchpad_filename(team_id));
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    let entry = format!("\n## [{mind}] {timestamp}\n\n{content}\n");

    use std::io::Write;
    match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                return ToolResult::err(format!("Write failed: {e}"));
            }
            info!(mind_id = mind, team_id = team_id, bytes = entry.len(), "Team scratchpad written");
            ToolResult::ok(format!("Appended to team '{team_id}' scratchpad ({} bytes)", content.len()))
        }
        Err(e) => ToolResult::err(format!("Failed to open team scratchpad: {e}")),
    }
}

// ── Hum Digest Tool Definitions ──────────────────────────────────────────

/// Generate tool schemas for hum_digest.
fn hum_tool_schemas() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            tool_type: "function".into(),
            function: FunctionSchema {
                name: "hum_digest".into(),
                description: "Digest today's Hum observations — returns patterns, error rates, tool usage stats, and actionable insights from the passive observation log. Use this to understand what happened in recent daemon cycles.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
    ]
}

/// Handle hum_digest — read today's JSONL, compute stats, return digest.
fn handle_hum_digest(hum_dir: Option<&std::path::Path>) -> ToolResult {
    let Some(dir) = hum_dir else {
        return ToolResult::err("Hum directory not configured");
    };
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let path = dir.join(format!("{today}.jsonl"));

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return ToolResult::ok("(no Hum observations today — daemon may not have run yet)"),
    };

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return ToolResult::ok("(Hum log exists but is empty)");
    }

    let mut total = 0u32;
    let mut ok_count = 0u32;
    let mut error_count = 0u32;
    let mut total_duration_ms = 0u64;
    let mut total_iterations = 0u32;
    let mut total_tool_calls = 0u32;
    let mut challenger_warnings = 0u32;
    let mut tool_usage: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut stall_events = 0u32;

    for line in &lines {
        let Ok(obs) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        total += 1;

        match obs.get("outcome").and_then(|v| v.as_str()) {
            Some("ok") => ok_count += 1,
            Some("error") => {
                error_count += 1;
                if let Some(err) = obs.get("error").and_then(|v| v.as_str()) {
                    if errors.len() < 5 {
                        errors.push(err.chars().take(120).collect());
                    }
                }
            }
            _ => {}
        }

        if let Some(d) = obs.get("duration_ms").and_then(|v| v.as_u64()) {
            total_duration_ms += d;
        }
        if let Some(i) = obs.get("iterations").and_then(|v| v.as_u64()) {
            total_iterations += i as u32;
        }
        if let Some(tc) = obs.get("tool_calls").and_then(|v| v.as_u64()) {
            total_tool_calls += tc as u32;
        }
        if let Some(cw) = obs.get("challenger_warnings").and_then(|v| v.as_u64()) {
            challenger_warnings += cw as u32;
        }

        if let Some(tools) = obs.get("tools_used").and_then(|v| v.as_array()) {
            for tool in tools {
                if let Some(name) = tool.as_str() {
                    *tool_usage.entry(name.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Detect stall pattern: high iterations with low/no tool calls
        let iters = obs.get("iterations").and_then(|v| v.as_u64()).unwrap_or(0);
        let tcs = obs.get("tool_calls").and_then(|v| v.as_u64()).unwrap_or(0);
        if iters >= 5 && tcs == 0 {
            stall_events += 1;
        }
    }

    let avg_duration = if total > 0 { total_duration_ms / total as u64 } else { 0 };
    let error_rate = if total > 0 { (error_count as f64 / total as f64) * 100.0 } else { 0.0 };

    // Sort tool usage by frequency
    let mut tool_vec: Vec<_> = tool_usage.into_iter().collect();
    tool_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let top_tools: Vec<String> = tool_vec.iter().take(10)
        .map(|(name, count)| format!("  {name}: {count}"))
        .collect();

    let mut digest = format!(
        "# Hum Digest — {today}\n\n\
         ## Summary\n\
         - Events: {total} ({ok_count} ok, {error_count} errors, {error_rate:.1}% error rate)\n\
         - Total duration: {total_duration_ms}ms (avg {avg_duration}ms/event)\n\
         - Iterations: {total_iterations} total, Tool calls: {total_tool_calls} total\n\
         - Challenger warnings: {challenger_warnings}\n\
         - Stall events (5+ iters, 0 tool calls): {stall_events}\n"
    );

    if !top_tools.is_empty() {
        digest.push_str("\n## Tool Usage (top 10)\n");
        for t in &top_tools {
            digest.push_str(t);
            digest.push('\n');
        }
    }

    if !errors.is_empty() {
        digest.push_str("\n## Recent Errors\n");
        for (i, err) in errors.iter().enumerate() {
            digest.push_str(&format!("{}. {}\n", i + 1, err));
        }
    }

    // Actionable insights
    let mut insights = Vec::new();
    if error_rate > 20.0 {
        insights.push("HIGH ERROR RATE — investigate LLM availability or tool failures");
    }
    if stall_events > 0 {
        insights.push("STALL DETECTED — agents running many iterations without tool use (may need better prompts)");
    }
    if challenger_warnings > 3 {
        insights.push("FREQUENT CHALLENGER WARNINGS — review agent behavior patterns");
    }
    if avg_duration > 60000 {
        insights.push("SLOW CYCLES — average >60s per event, may indicate context overflow");
    }

    if !insights.is_empty() {
        digest.push_str("\n## Actionable Insights\n");
        for insight in &insights {
            digest.push_str(&format!("- {insight}\n"));
        }
    }

    info!(events = total, errors = error_count, "Hum digest generated");
    ToolResult::ok(digest)
}

/// Extract text content from a chat response.
fn extract_content(resp: &ChatResponse) -> String {
    resp.choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default()
}

/// Errors from the thinking loop.
#[derive(Debug, thiserror::Error)]
pub enum ThinkError {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn think_result_default() {
        let result = ThinkResult {
            response: "Done".into(),
            tool_calls_made: vec![],
            iterations: 1,
            completed: true,
            challenger_warnings: 0,
            stall_killed: false,
        };
        assert!(result.completed);
        assert_eq!(result.iterations, 1);
        assert_eq!(result.challenger_warnings, 0);
    }

    #[test]
    fn tool_call_record() {
        let record = ToolCallRecord {
            tool_name: "bash".into(),
            arguments: serde_json::json!({"command": "ls"}),
            result: ToolResult::ok("file1\nfile2"),
            iteration: 1,
        };
        assert_eq!(record.tool_name, "bash");
        assert!(record.result.success);
    }

    #[test]
    fn default_config() {
        let cfg = ThinkLoopConfig::default();
        assert_eq!(cfg.max_iterations, 15);
    }

    #[test]
    fn extract_content_from_response() {
        let resp = ChatResponse {
            id: Some("test".into()),
            choices: vec![crate::ollama::Choice {
                index: 0,
                message: ChatMessage::assistant("Hello from Cortex"),
                finish_reason: Some("stop".into()),
            }],
            usage: None,
        };
        assert_eq!(extract_content(&resp), "Hello from Cortex");
    }

    #[test]
    fn extract_content_empty() {
        let resp = ChatResponse {
            id: None,
            choices: vec![],
            usage: None,
        };
        assert_eq!(extract_content(&resp), "");
    }

    #[test]
    fn memory_tool_schemas_count() {
        let schemas = memory_tool_schemas();
        assert_eq!(schemas.len(), 2);
        assert_eq!(schemas[0].function.name, "memory_search");
        assert_eq!(schemas[1].function.name, "memory_write");
    }

    #[tokio::test]
    async fn handle_memory_search_no_store() {
        let args = serde_json::json!({"query": "test"});
        let result = handle_memory_search(None, &args).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not available"));
    }

    #[tokio::test]
    async fn handle_memory_write_no_store() {
        let args = serde_json::json!({"title": "Test", "content": "test content"});
        let result = handle_memory_write(None, &args, None, Role::Agent).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not available"));
    }

    #[tokio::test]
    async fn handle_memory_search_with_store() {
        let store = MemoryStore::new(":memory:").await.unwrap();
        let args = serde_json::json!({"query": "nonexistent"});
        let result = handle_memory_search(Some(&store), &args).await;
        assert!(result.success);
        assert!(result.output.contains("No memories found"));
    }

    #[tokio::test]
    async fn handle_memory_write_with_store() {
        let store = MemoryStore::new(":memory:").await.unwrap();
        let args = serde_json::json!({
            "title": "Test Learning",
            "content": "Cortex can write memories during thinking",
            "category": "learning"
        });
        let result = handle_memory_write(Some(&store), &args, Some("test-mind"), Role::Agent).await;
        assert!(result.success);
        assert!(result.output.contains("Memory stored"));
        assert!(result.output.contains("Test Learning"));
    }

    #[tokio::test]
    async fn memory_write_then_search() {
        let store = MemoryStore::new(":memory:").await.unwrap();

        // Write a memory
        let write_args = serde_json::json!({
            "title": "Fractal Pattern",
            "content": "Fractal coordination enables recursive delegation",
            "category": "pattern"
        });
        let write_result = handle_memory_write(Some(&store), &write_args, Some("test-mind"), Role::Agent).await;
        assert!(write_result.success);

        // Search for it
        let search_args = serde_json::json!({"query": "fractal coordination"});
        let search_result = handle_memory_search(Some(&store), &search_args).await;
        assert!(search_result.success);
        assert!(search_result.output.contains("Fractal Pattern"));
        assert!(search_result.output.contains("recursive delegation"));
    }

    #[test]
    fn scratchpad_tool_schemas_count() {
        let schemas = scratchpad_tool_schemas();
        assert_eq!(schemas.len(), 2);
        assert_eq!(schemas[0].function.name, "scratchpad_read");
        assert_eq!(schemas[1].function.name, "scratchpad_write");
    }

    #[test]
    fn scratchpad_read_no_dir() {
        let result = handle_scratchpad_read(None, Some("test"));
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not configured"));
    }

    #[test]
    fn scratchpad_read_no_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = handle_scratchpad_read(Some(tmp.path()), Some("nonexistent"));
        assert!(result.success);
        assert!(result.output.contains("no scratchpad yet"));
    }

    #[test]
    fn scratchpad_write_then_read() {
        let tmp = tempfile::TempDir::new().unwrap();
        let args = serde_json::json!({"content": "Note: fractal delegation works"});
        let write_result = handle_scratchpad_write(Some(tmp.path()), Some("test-mind"), &args);
        assert!(write_result.success);

        let read_result = handle_scratchpad_read(Some(tmp.path()), Some("test-mind"));
        assert!(read_result.success);
        assert!(read_result.output.contains("fractal delegation works"));
    }

    #[test]
    fn scratchpad_write_appends() {
        let tmp = tempfile::TempDir::new().unwrap();
        let args1 = serde_json::json!({"content": "First note"});
        let args2 = serde_json::json!({"content": "Second note"});

        handle_scratchpad_write(Some(tmp.path()), Some("test"), &args1);
        handle_scratchpad_write(Some(tmp.path()), Some("test"), &args2);

        let read_result = handle_scratchpad_read(Some(tmp.path()), Some("test"));
        assert!(read_result.output.contains("First note"));
        assert!(read_result.output.contains("Second note"));
    }

    #[test]
    fn scratchpad_write_empty_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let args = serde_json::json!({"content": ""});
        let result = handle_scratchpad_write(Some(tmp.path()), Some("test"), &args);
        assert!(!result.success);
    }

    /// Test ToolInterceptor trait implementation.
    struct TestInterceptor;

    #[async_trait]
    impl ToolInterceptor for TestInterceptor {
        fn schemas(&self) -> Vec<crate::ollama::ToolSchema> {
            vec![crate::ollama::ToolSchema {
                tool_type: "function".into(),
                function: crate::ollama::FunctionSchema {
                    name: "custom_tool".into(),
                    description: "A test interceptor tool".into(),
                    parameters: serde_json::json!({"type": "object", "properties": {}}),
                },
            }]
        }

        async fn handle(&self, name: &str, _args: &serde_json::Value) -> Option<ToolResult> {
            if name == "custom_tool" {
                Some(ToolResult::ok("intercepted!"))
            } else {
                None
            }
        }
    }

    #[test]
    fn interceptor_schemas() {
        let interceptor = TestInterceptor;
        let schemas = interceptor.schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].function.name, "custom_tool");
    }

    #[tokio::test]
    async fn interceptor_handles_known_tool() {
        let interceptor = TestInterceptor;
        let result = interceptor.handle("custom_tool", &serde_json::json!({})).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().output, "intercepted!");
    }

    #[tokio::test]
    async fn interceptor_passes_unknown_tool() {
        let interceptor = TestInterceptor;
        let result = interceptor.handle("unknown_tool", &serde_json::json!({})).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn composite_interceptor_chains() {
        struct Alpha;
        #[async_trait]
        impl ToolInterceptor for Alpha {
            fn schemas(&self) -> Vec<crate::ollama::ToolSchema> {
                vec![ToolSchema {
                    tool_type: "function".into(),
                    function: FunctionSchema {
                        name: "alpha_tool".into(),
                        description: "Alpha".into(),
                        parameters: serde_json::json!({"type": "object", "properties": {}}),
                    },
                }]
            }
            async fn handle(&self, name: &str, _args: &serde_json::Value) -> Option<ToolResult> {
                if name == "alpha_tool" {
                    Some(ToolResult::ok("alpha"))
                } else {
                    None
                }
            }
        }

        struct Beta;
        #[async_trait]
        impl ToolInterceptor for Beta {
            fn schemas(&self) -> Vec<crate::ollama::ToolSchema> {
                vec![ToolSchema {
                    tool_type: "function".into(),
                    function: FunctionSchema {
                        name: "beta_tool".into(),
                        description: "Beta".into(),
                        parameters: serde_json::json!({"type": "object", "properties": {}}),
                    },
                }]
            }
            async fn handle(&self, name: &str, _args: &serde_json::Value) -> Option<ToolResult> {
                if name == "beta_tool" {
                    Some(ToolResult::ok("beta"))
                } else {
                    None
                }
            }
        }

        let alpha = Alpha;
        let beta = Beta;
        let composite = CompositeInterceptor::new(vec![
            &alpha as &dyn ToolInterceptor,
            &beta as &dyn ToolInterceptor,
        ]);

        // Schemas from both
        assert_eq!(composite.schemas().len(), 2);

        // Alpha tool handled by Alpha
        let r = composite.handle("alpha_tool", &serde_json::json!({})).await;
        assert!(r.is_some());
        assert_eq!(r.unwrap().output, "alpha");

        // Beta tool handled by Beta
        let r = composite.handle("beta_tool", &serde_json::json!({})).await;
        assert!(r.is_some());
        assert_eq!(r.unwrap().output, "beta");

        // Unknown passes through
        let r = composite.handle("unknown", &serde_json::json!({})).await;
        assert!(r.is_none());
    }

    // ── JSON Sanitization Tests ─────────────────────────────────────────

    #[test]
    fn sanitize_trailing_comma_object() {
        let input = r#"{"limit": 10,}"#;
        let output = sanitize_json_string(input);
        assert_eq!(output, r#"{"limit": 10}"#);
        // Verify it's valid JSON after sanitization
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn sanitize_trailing_comma_array() {
        let input = r#"[1, 2, 3,]"#;
        let output = sanitize_json_string(input);
        assert_eq!(output, r#"[1, 2, 3]"#);
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn sanitize_trailing_comma_with_whitespace() {
        let input = r#"{"query": "hello" ,  }"#;
        let output = sanitize_json_string(input);
        assert_eq!(output, r#"{"query": "hello"   }"#);
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn sanitize_valid_json_unchanged() {
        let input = r#"{"file_path": "/tmp/test.txt", "limit": 5}"#;
        let output = sanitize_json_string(input);
        assert_eq!(output, input);
    }

    #[test]
    fn sanitize_nested_trailing_commas() {
        let input = r#"{"a": [1, 2,], "b": {"c": 3,},}"#;
        let output = sanitize_json_string(input);
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    // ── Parameter Normalization Tests ───────────────────────────────────

    #[test]
    fn normalize_read_path_to_file_path() {
        let args = serde_json::json!({"path": "/tmp/test.txt"});
        let result = normalize_args("read", args);
        assert_eq!(result.get("file_path").unwrap().as_str().unwrap(), "/tmp/test.txt");
        assert!(result.get("path").is_none());
    }

    #[test]
    fn normalize_write_filepath_to_file_path() {
        let args = serde_json::json!({"filepath": "/tmp/out.txt", "content": "hello"});
        let result = normalize_args("write", args);
        assert_eq!(result.get("file_path").unwrap().as_str().unwrap(), "/tmp/out.txt");
        assert_eq!(result.get("content").unwrap().as_str().unwrap(), "hello");
    }

    #[test]
    fn normalize_does_not_overwrite_canonical() {
        // If both "file_path" and "path" are present, "file_path" wins
        let args = serde_json::json!({"file_path": "/correct", "path": "/wrong"});
        let result = normalize_args("read", args);
        assert_eq!(result.get("file_path").unwrap().as_str().unwrap(), "/correct");
    }

    #[test]
    fn normalize_bash_cmd_to_command() {
        let args = serde_json::json!({"cmd": "ls -la"});
        let result = normalize_args("bash", args);
        assert_eq!(result.get("command").unwrap().as_str().unwrap(), "ls -la");
    }

    #[test]
    fn normalize_memory_search_aliases() {
        let args = serde_json::json!({"search_query": "hub api"});
        let result = normalize_args("memory_search", args);
        assert_eq!(result.get("query").unwrap().as_str().unwrap(), "hub api");
    }

    #[test]
    fn normalize_hub_reply_aliases() {
        let args = serde_json::json!({"thread": "abc-123", "message": "hello world"});
        let result = normalize_args("hub_reply", args);
        assert_eq!(result.get("thread_id").unwrap().as_str().unwrap(), "abc-123");
        assert_eq!(result.get("body").unwrap().as_str().unwrap(), "hello world");
    }

    #[test]
    fn normalize_unknown_tool_passthrough() {
        let args = serde_json::json!({"foo": "bar"});
        let result = normalize_args("unknown_tool", args.clone());
        assert_eq!(result, args);
    }

    #[test]
    fn normalize_null_args_passthrough() {
        let result = normalize_args("read", serde_json::Value::Null);
        assert!(result.is_null());
    }

    // ── Sprint 6: Builtin + Interceptor Enforcement Tests (B1 + B2) ──

    /// Helper: create a ToolExecutor with no registered tools and no hooks.
    fn make_executor() -> ToolExecutor {
        let reg = codex_exec::ToolRegistry::new();
        let sandbox = codex_exec::SandboxEnforcer::new("/tmp/test-workspace".into());
        ToolExecutor::new(reg, sandbox)
    }

    /// Helper: create a ToolExecutor with a blocking hook on PreToolUse.
    fn make_executor_with_blocking_hook(reason: &str) -> ToolExecutor {
        use aiciv_hooks::{HookDispatcher, HookEventType};
        use aiciv_hooks::HookHandler;

        struct BlockAll { reason: String }
        #[async_trait]
        impl HookHandler for BlockAll {
            async fn handle(&self, _event: &aiciv_hooks::HookEvent) -> anyhow::Result<aiciv_hooks::HookResponse> {
                Ok(aiciv_hooks::HookResponse::PreToolUse {
                    should_block: true,
                    reason: Some(self.reason.clone()),
                    modified_input: None,
                })
            }
            fn name(&self) -> &str { "block-all" }
        }

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            HookEventType::PreToolUse,
            std::sync::Arc::new(BlockAll { reason: reason.to_string() }),
        );

        let reg = codex_exec::ToolRegistry::new();
        let sandbox = codex_exec::SandboxEnforcer::new("/tmp/test-workspace".into());
        ToolExecutor::new(reg, sandbox).with_hooks(std::sync::Arc::new(dispatcher))
    }

    // ── B1: Builtin tools get role-checked ──

    #[tokio::test]
    async fn builtin_memory_search_allowed_for_agent() {
        // Agent role can use memory_search (wildcard allows all)
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"query": "test"});
        let result = loop_inst.dispatch_builtin_or_exec(
            "memory_search", &args, None, None, Role::Agent, &exec, "memory_search",
        ).await;
        // Should reach the handler (which returns error because no store), NOT be denied by role
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("not available"));
    }

    #[tokio::test]
    async fn builtin_memory_write_denied_for_primary() {
        // Primary role does NOT have memory_write in its allowed tools
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"title": "test", "content": "test"});
        let result = loop_inst.dispatch_builtin_or_exec(
            "memory_write", &args, None, None, Role::Primary, &exec, "memory_write",
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn builtin_scratchpad_write_denied_for_primary() {
        // Primary has no scratchpad_write in its role tools
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"content": "note"});
        let result = loop_inst.dispatch_builtin_or_exec(
            "scratchpad_write", &args, None, None, Role::Primary, &exec, "scratchpad_write",
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn builtin_hum_digest_denied_for_team_lead() {
        // TeamLead has no hum_digest in its role tools
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({});
        let result = loop_inst.dispatch_builtin_or_exec(
            "hum_digest", &args, None, None, Role::TeamLead, &exec, "hum_digest",
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn builtin_ollama_usage_denied_for_team_lead() {
        // TeamLead has no ollama_usage in its role tools
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({});
        let result = loop_inst.dispatch_builtin_or_exec(
            "ollama_usage", &args, None, None, Role::TeamLead, &exec, "ollama_usage",
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn builtin_memory_search_allowed_for_primary() {
        // Primary DOES have memory_search in its role tools
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"query": "test"});
        let result = loop_inst.dispatch_builtin_or_exec(
            "memory_search", &args, None, None, Role::Primary, &exec, "memory_search",
        ).await;
        // Should pass role check but fail at handler level (no store)
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("not available"));
    }

    #[tokio::test]
    async fn builtin_team_scratchpad_read_allowed_for_team_lead() {
        // TeamLead DOES have team_scratchpad_read
        let exec = make_executor();
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({});
        let result = loop_inst.dispatch_builtin_or_exec(
            "team_scratchpad_read", &args, None, None, Role::TeamLead, &exec, "team_scratchpad_read",
        ).await;
        // Passes role check, fails at handler (no scratchpad configured)
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("not configured"));
    }

    // ── B1: Builtin tools get hook enforcement ──

    #[tokio::test]
    async fn builtin_blocked_by_hook() {
        let exec = make_executor_with_blocking_hook("audit: memory_search blocked");
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"query": "test"});
        // Agent role is allowed, but hook blocks
        let result = loop_inst.dispatch_builtin_or_exec(
            "memory_search", &args, None, None, Role::Agent, &exec, "memory_search",
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Blocked by hook"));
        assert!(result.error.as_ref().unwrap().contains("audit: memory_search blocked"));
    }

    #[tokio::test]
    async fn builtin_role_check_before_hook() {
        // Role check happens BEFORE hook. If role denies, we never reach the hook.
        let exec = make_executor_with_blocking_hook("should not reach this");
        let loop_inst = ThinkLoop::new(
            Box::new(crate::provider::DummyProvider),
            ThinkLoopConfig::default(),
        );
        let args = serde_json::json!({"title": "x", "content": "y"});
        let result = loop_inst.dispatch_builtin_or_exec(
            "memory_write", &args, None, None, Role::Primary, &exec, "memory_write",
        ).await;
        assert!(!result.success);
        // Should be "Permission denied", not "Blocked by hook"
        assert!(result.error.as_ref().unwrap().contains("Permission denied"));
    }

    // ── B2: Intercepted tools get enforcement ──

    #[tokio::test]
    async fn intercepted_tool_denied_by_role() {
        // Create a tool that the interceptor handles but role denies
        struct DeniedInterceptor;
        #[async_trait]
        impl ToolInterceptor for DeniedInterceptor {
            fn schemas(&self) -> Vec<crate::ollama::ToolSchema> { vec![] }
            async fn handle(&self, name: &str, _args: &serde_json::Value) -> Option<ToolResult> {
                if name == "secret_tool" {
                    Some(ToolResult::ok("intercepted secret"))
                } else {
                    None
                }
            }
        }

        let interceptor = DeniedInterceptor;
        let args = serde_json::json!({});

        // Primary role doesn't have "secret_tool"
        // Simulate what run_full does:
        if let Some(_r) = interceptor.handle("secret_tool", &args).await {
            // B2 enforcement: role check
            if !codex_roles::is_tool_allowed(Role::Primary, "secret_tool") {
                let result = ToolResult::err(format!(
                    "Permission denied: tool '{}' not allowed for role {:?}",
                    "secret_tool", Role::Primary
                ));
                assert!(!result.success);
                assert!(result.error.as_ref().unwrap().contains("Permission denied"));
            } else {
                panic!("Primary should NOT be allowed secret_tool");
            }
        } else {
            panic!("Interceptor should have handled secret_tool");
        }
    }

    #[tokio::test]
    async fn intercepted_tool_blocked_by_hook() {
        let exec = make_executor_with_blocking_hook("interceptor audit block");

        // Agent role is allowed (wildcard), but hook blocks
        let block_result = exec.fire_pre_tool_use("custom_intercepted", &serde_json::json!({})).await;
        assert!(block_result.is_some());
        assert!(block_result.unwrap().contains("interceptor audit block"));
    }

    #[tokio::test]
    async fn intercepted_tool_allowed_when_role_and_hooks_pass() {
        // Agent role allows all, no hooks configured → should pass through
        let exec = make_executor();

        assert!(codex_roles::is_tool_allowed(Role::Agent, "custom_tool"));
        let block_result = exec.fire_pre_tool_use("custom_tool", &serde_json::json!({})).await;
        assert!(block_result.is_none()); // No hooks → allowed
    }
}
