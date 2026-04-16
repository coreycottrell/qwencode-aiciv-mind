//! LLM integration — delegates to codex-llm's unified OllamaClient.
//!
//! Previously qwen-mind had its own OllamaClient. Now it re-exports
//! the production-grade client from codex-llm (rate limiting, circuit
//! breaker, retry with backoff, tool support) via the LlmProvider trait.

pub use codex_llm::ollama::{
    ChatMessage, ChatResponse, LlmError, OllamaClient, OllamaConfig,
};
pub use codex_llm::LlmProvider;
