//! Provider-agnostic LLM interface.
//!
//! Implement `LlmProvider` to add new model backends (Ollama, OpenAI, etc.).
//! ThinkLoop dispatches through this trait, making the harness model-agnostic.

use async_trait::async_trait;

use crate::ollama::{ChatMessage, ChatResponse, LlmError, ToolSchema};

/// Provider-agnostic LLM interface.
///
/// All model communication flows through this trait. The ThinkLoop,
/// qwen-mind, and future providers (OpenAI-compat, Anthropic) all
/// implement or consume this single interface.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a chat completion request with optional tool definitions.
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolSchema]>,
    ) -> Result<ChatResponse, LlmError>;

    /// The model name this provider is configured with.
    fn model_name(&self) -> &str;

    /// Simple chat helper — system + user prompt, no tools.
    ///
    /// Convenience method for callers that don't need tool use.
    /// Returns the text content from the first choice.
    async fn simple_chat(
        &self,
        system: &str,
        user: &str,
    ) -> Result<String, LlmError> {
        let messages = vec![
            ChatMessage::system(system),
            ChatMessage::user(user),
        ];
        let response = self.chat(&messages, None).await?;
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "Empty response.".to_string());
        Ok(content)
    }
}
