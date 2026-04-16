//! # codex-llm — LLM Integration for Cortex
//!
//! Provides the thinking loop: prompt construction → LLM call → tool call
//! parsing → tool execution → result injection → loop until done.
//!
//! Uses Ollama's OpenAI-compatible API with open source models only:
//! - Gemma 4 (26B MoE) — orchestration, planning, complex reasoning
//! - M2.7 — red team, memory extraction, lightweight tasks

pub mod ollama;
pub mod prompt;
pub mod provider;
pub mod rate_limiter;
pub mod think_loop;

pub use ollama::{OllamaClient, OllamaConfig, ModelRouter, ToolSchema, FunctionSchema};
pub use prompt::PromptBuilder;
pub use provider::LlmProvider;
pub use rate_limiter::RateLimiter;
pub use think_loop::{ThinkLoop, ThinkLoopConfig, ThinkResult, ToolInterceptor, CompositeInterceptor};
