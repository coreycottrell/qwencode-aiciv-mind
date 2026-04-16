//! HookDispatcher — the central event bus for lifecycle hooks.
//!
//! Components register handlers for event types. When an event fires,
//! the dispatcher routes it to all matching handlers and collects responses.
//!
//! Two firing modes:
//! - `fire_blocking` — for pre-* events. Returns a Decision (allow/block).
//!   If ANY handler blocks, the operation is blocked.
//! - `fire` — for all events. Returns all responses. Caller decides what to do.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, info, warn};

use crate::config::HooksSettings;
use crate::handler::{ExternalCommandHandler, HookHandler};
use crate::types::{HookEvent, HookEventType, HookResponse};

/// Result of a blocking fire — either allow the operation or block it.
#[derive(Debug)]
pub enum Decision {
    /// All handlers approved (or no handlers registered).
    Allow,
    /// At least one handler blocked. Contains the first block reason.
    Block { reason: String },
}

impl Decision {
    pub fn is_blocked(&self) -> bool {
        matches!(self, Decision::Block { .. })
    }
}

/// Registration entry: a handler bound to an event type with optional tool filter.
struct Registration {
    handler: Arc<dyn HookHandler>,
    tool_names: Option<Vec<String>>,
}

impl Registration {
    fn matches_event(&self, event: &HookEvent) -> bool {
        match &self.tool_names {
            None => true,
            Some(names) => match event.tool_name() {
                Some(tool) => names.iter().any(|n| n == tool),
                None => true, // non-tool events always match
            },
        }
    }
}

/// The central hook dispatcher.
///
/// Thread-safe: handlers are `Arc<dyn HookHandler>`, registrations are
/// built at startup and not mutated during event dispatch.
pub struct HookDispatcher {
    handlers: HashMap<HookEventType, Vec<Registration>>,
}

impl HookDispatcher {
    /// Create an empty dispatcher with no handlers.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Create a dispatcher from a settings file, building ExternalCommandHandlers.
    pub fn from_settings(settings: &HooksSettings) -> Self {
        let mut dispatcher = Self::new();
        for hook_config in &settings.hooks {
            let handler = ExternalCommandHandler::new(hook_config.command.clone())
                .with_args(hook_config.args.clone())
                .with_timeout(hook_config.timeout())
                .with_fail_open(!hook_config.required);
            dispatcher.register_with_filter(
                hook_config.event,
                Arc::new(handler),
                hook_config.tool_names.clone(),
            );
        }
        dispatcher
    }

    /// Register a handler for an event type (no tool filter).
    pub fn register(
        &mut self,
        event_type: HookEventType,
        handler: Arc<dyn HookHandler>,
    ) {
        self.register_with_filter(event_type, handler, None);
    }

    /// Register a handler with an optional tool-name filter.
    pub fn register_with_filter(
        &mut self,
        event_type: HookEventType,
        handler: Arc<dyn HookHandler>,
        tool_names: Option<Vec<String>>,
    ) {
        self.handlers
            .entry(event_type)
            .or_default()
            .push(Registration {
                handler,
                tool_names,
            });
    }

    /// Fire an event and collect all responses.
    ///
    /// Handlers run sequentially (order matters for blocking decisions).
    /// Errors from individual handlers are logged but don't stop other handlers.
    pub async fn fire(&self, event: &HookEvent) -> Vec<HookResponse> {
        let event_type = event.event_type();
        let registrations = match self.handlers.get(&event_type) {
            Some(regs) => regs,
            None => {
                debug!(?event_type, "no handlers registered");
                return Vec::new();
            }
        };

        let mut responses = Vec::new();
        for reg in registrations {
            if !reg.matches_event(event) {
                continue;
            }
            match reg.handler.handle(event).await {
                Ok(resp) => {
                    debug!(
                        handler = reg.handler.name(),
                        ?event_type,
                        "hook responded"
                    );
                    responses.push(resp);
                }
                Err(e) => {
                    warn!(
                        handler = reg.handler.name(),
                        ?event_type,
                        error = %e,
                        "hook handler failed"
                    );
                }
            }
        }
        responses
    }

    /// Fire a blocking event (PreToolUse, PreDelegation).
    ///
    /// If ANY handler returns should_block=true, returns Decision::Block.
    /// Short-circuits on first block.
    pub async fn fire_blocking(&self, event: &HookEvent) -> Decision {
        let responses = self.fire(event).await;
        for resp in &responses {
            if resp.should_block() {
                let reason = resp
                    .block_reason()
                    .unwrap_or("blocked by hook")
                    .to_string();
                info!(
                    event_type = ?event.event_type(),
                    %reason,
                    "hook blocked operation"
                );
                return Decision::Block { reason };
            }
        }
        Decision::Allow
    }

    /// Number of handlers registered for an event type.
    pub fn handler_count(&self, event_type: HookEventType) -> usize {
        self.handlers
            .get(&event_type)
            .map_or(0, |regs| regs.len())
    }
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use crate::types::{HookEvent, HookResponse};
    use async_trait::async_trait;

    /// A simple in-process handler that always approves.
    struct ApproveHandler;
    #[async_trait]
    impl HookHandler for ApproveHandler {
        async fn handle(&self, _event: &HookEvent) -> Result<HookResponse> {
            Ok(HookResponse::Ack)
        }
        fn name(&self) -> &str {
            "approve"
        }
    }

    /// A handler that blocks with a reason.
    struct BlockHandler {
        reason: String,
    }
    #[async_trait]
    impl HookHandler for BlockHandler {
        async fn handle(&self, _event: &HookEvent) -> Result<HookResponse> {
            Ok(HookResponse::PreToolUse {
                should_block: true,
                reason: Some(self.reason.clone()),
                modified_input: None,
            })
        }
        fn name(&self) -> &str {
            "blocker"
        }
    }

    /// A handler that injects additional context.
    struct ContextInjector {
        context: String,
    }
    #[async_trait]
    impl HookHandler for ContextInjector {
        async fn handle(&self, _event: &HookEvent) -> Result<HookResponse> {
            Ok(HookResponse::PostToolUse {
                should_stop: false,
                additional_contexts: vec![self.context.clone()],
                feedback_message: None,
            })
        }
        fn name(&self) -> &str {
            "context-injector"
        }
    }

    /// A handler scoped to "bash" tool only.
    struct BashOnlyBlocker;
    #[async_trait]
    impl HookHandler for BashOnlyBlocker {
        async fn handle(&self, _event: &HookEvent) -> Result<HookResponse> {
            Ok(HookResponse::PreToolUse {
                should_block: true,
                reason: Some("bash not allowed".into()),
                modified_input: None,
            })
        }
        fn name(&self) -> &str {
            "bash-blocker"
        }
    }

    fn pre_tool_event(tool: &str) -> HookEvent {
        HookEvent::PreToolUse {
            session_id: "test".into(),
            tool_name: tool.into(),
            tool_input: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn empty_dispatcher_allows() {
        let dispatcher = HookDispatcher::new();
        let decision = dispatcher.fire_blocking(&pre_tool_event("bash")).await;
        assert!(!decision.is_blocked());
    }

    #[tokio::test]
    async fn single_approve_allows() {
        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(HookEventType::PreToolUse, Arc::new(ApproveHandler));
        let decision = dispatcher.fire_blocking(&pre_tool_event("bash")).await;
        assert!(!decision.is_blocked());
    }

    #[tokio::test]
    async fn single_block_blocks() {
        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            HookEventType::PreToolUse,
            Arc::new(BlockHandler {
                reason: "dangerous".into(),
            }),
        );
        let decision = dispatcher.fire_blocking(&pre_tool_event("bash")).await;
        assert!(decision.is_blocked());
        if let Decision::Block { reason } = decision {
            assert_eq!(reason, "dangerous");
        }
    }

    #[tokio::test]
    async fn approve_then_block() {
        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(HookEventType::PreToolUse, Arc::new(ApproveHandler));
        dispatcher.register(
            HookEventType::PreToolUse,
            Arc::new(BlockHandler {
                reason: "second hook blocks".into(),
            }),
        );
        let decision = dispatcher.fire_blocking(&pre_tool_event("bash")).await;
        assert!(decision.is_blocked());
    }

    #[tokio::test]
    async fn tool_filter_matches() {
        let mut dispatcher = HookDispatcher::new();
        dispatcher.register_with_filter(
            HookEventType::PreToolUse,
            Arc::new(BashOnlyBlocker),
            Some(vec!["bash".into()]),
        );

        // bash is blocked
        let decision = dispatcher.fire_blocking(&pre_tool_event("bash")).await;
        assert!(decision.is_blocked());

        // read is allowed (filter doesn't match)
        let decision = dispatcher.fire_blocking(&pre_tool_event("read")).await;
        assert!(!decision.is_blocked());
    }

    #[tokio::test]
    async fn fire_collects_all_responses() {
        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            HookEventType::PostToolUse,
            Arc::new(ContextInjector {
                context: "ctx1".into(),
            }),
        );
        dispatcher.register(
            HookEventType::PostToolUse,
            Arc::new(ContextInjector {
                context: "ctx2".into(),
            }),
        );

        let event = HookEvent::PostToolUse {
            session_id: "test".into(),
            tool_name: "read".into(),
            tool_input: serde_json::json!({}),
            tool_output: serde_json::json!({}),
        };
        let responses = dispatcher.fire(&event).await;
        assert_eq!(responses.len(), 2);
    }

    #[tokio::test]
    async fn handler_count() {
        let mut dispatcher = HookDispatcher::new();
        assert_eq!(dispatcher.handler_count(HookEventType::PreToolUse), 0);
        dispatcher.register(HookEventType::PreToolUse, Arc::new(ApproveHandler));
        dispatcher.register(HookEventType::PreToolUse, Arc::new(ApproveHandler));
        assert_eq!(dispatcher.handler_count(HookEventType::PreToolUse), 2);
        assert_eq!(dispatcher.handler_count(HookEventType::PostToolUse), 0);
    }

    #[tokio::test]
    async fn from_settings_builds_handlers() {
        let settings = crate::config::HooksSettings::from_json(
            r#"{
                "hooks": [
                    {
                        "event": "session_start",
                        "command": "true"
                    },
                    {
                        "event": "pre_tool_use",
                        "command": "true",
                        "tool_names": ["bash"]
                    }
                ]
            }"#,
        )
        .unwrap();
        let dispatcher = HookDispatcher::from_settings(&settings);
        assert_eq!(dispatcher.handler_count(HookEventType::SessionStart), 1);
        assert_eq!(dispatcher.handler_count(HookEventType::PreToolUse), 1);
    }
}
