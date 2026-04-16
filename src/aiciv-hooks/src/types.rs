//! Hook event and response types.
//!
//! Self-contained in aiciv-hooks per mind-coordination recommendation:
//! "Self-contained approach initially (less risk to existing crates),
//! then migrate to codex-types once interfaces stabilize."

use serde::{Deserialize, Serialize};

/// Discriminant for matching hooks to events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEventType {
    SessionStart,
    PreToolUse,
    PostToolUse,
    Stop,
    UserPromptSubmit,
    // aiciv-mind extensions
    PreDelegation,
    PostDelegation,
    MemoryWrite,
    DriveEvent,
}

/// A hook event with its payload.
///
/// Fired by consumers (codex-exec, cortex, codex-drive) into the dispatcher.
/// The dispatcher matches by event type and routes to registered handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum HookEvent {
    SessionStart {
        session_id: String,
        #[serde(default)]
        metadata: serde_json::Value,
    },
    PreToolUse {
        session_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
    },
    PostToolUse {
        session_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        tool_output: serde_json::Value,
    },
    Stop {
        session_id: String,
        reason: String,
    },
    UserPromptSubmit {
        session_id: String,
        prompt: String,
    },
    // aiciv-mind extensions
    PreDelegation {
        session_id: String,
        target_mind: String,
        task: String,
    },
    PostDelegation {
        session_id: String,
        target_mind: String,
        result: serde_json::Value,
    },
    MemoryWrite {
        session_id: String,
        namespace: String,
        content: String,
    },
    DriveEvent {
        session_id: String,
        drive_event: codex_types::DriveEvent,
    },
}

impl HookEvent {
    /// The event type discriminant, used for handler matching.
    pub fn event_type(&self) -> HookEventType {
        match self {
            HookEvent::SessionStart { .. } => HookEventType::SessionStart,
            HookEvent::PreToolUse { .. } => HookEventType::PreToolUse,
            HookEvent::PostToolUse { .. } => HookEventType::PostToolUse,
            HookEvent::Stop { .. } => HookEventType::Stop,
            HookEvent::UserPromptSubmit { .. } => HookEventType::UserPromptSubmit,
            HookEvent::PreDelegation { .. } => HookEventType::PreDelegation,
            HookEvent::PostDelegation { .. } => HookEventType::PostDelegation,
            HookEvent::MemoryWrite { .. } => HookEventType::MemoryWrite,
            HookEvent::DriveEvent { .. } => HookEventType::DriveEvent,
        }
    }

    /// The tool name, if this is a tool-use event.
    pub fn tool_name(&self) -> Option<&str> {
        match self {
            HookEvent::PreToolUse { tool_name, .. }
            | HookEvent::PostToolUse { tool_name, .. } => Some(tool_name),
            _ => None,
        }
    }

    /// The session ID for this event.
    pub fn session_id(&self) -> &str {
        match self {
            HookEvent::SessionStart { session_id, .. }
            | HookEvent::PreToolUse { session_id, .. }
            | HookEvent::PostToolUse { session_id, .. }
            | HookEvent::Stop { session_id, .. }
            | HookEvent::UserPromptSubmit { session_id, .. }
            | HookEvent::PreDelegation { session_id, .. }
            | HookEvent::PostDelegation { session_id, .. }
            | HookEvent::MemoryWrite { session_id, .. }
            | HookEvent::DriveEvent { session_id, .. } => session_id,
        }
    }
}

/// Response from a hook handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookResponse {
    /// Response to PreToolUse — can block the tool call.
    PreToolUse {
        #[serde(default)]
        should_block: bool,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        modified_input: Option<serde_json::Value>,
    },
    /// Response to PostToolUse — can stop session or inject context.
    PostToolUse {
        #[serde(default)]
        should_stop: bool,
        #[serde(default)]
        additional_contexts: Vec<String>,
        #[serde(default)]
        feedback_message: Option<String>,
    },
    /// Response to PreDelegation — can block delegation.
    PreDelegation {
        #[serde(default)]
        should_block: bool,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        modified_task: Option<String>,
    },
    /// Acknowledgement for fire-and-forget events.
    Ack,
}

impl HookResponse {
    /// Whether this response blocks the operation (for pre-* events).
    pub fn should_block(&self) -> bool {
        match self {
            HookResponse::PreToolUse { should_block, .. } => *should_block,
            HookResponse::PreDelegation { should_block, .. } => *should_block,
            _ => false,
        }
    }

    /// Block reason, if any.
    pub fn block_reason(&self) -> Option<&str> {
        match self {
            HookResponse::PreToolUse { reason, .. } => reason.as_deref(),
            HookResponse::PreDelegation { reason, .. } => reason.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_discrimination() {
        let event = HookEvent::PreToolUse {
            session_id: "s1".into(),
            tool_name: "bash".into(),
            tool_input: serde_json::json!({"command": "ls"}),
        };
        assert_eq!(event.event_type(), HookEventType::PreToolUse);
        assert_eq!(event.tool_name(), Some("bash"));
        assert_eq!(event.session_id(), "s1");
    }

    #[test]
    fn response_should_block() {
        let blocking = HookResponse::PreToolUse {
            should_block: true,
            reason: Some("dangerous".into()),
            modified_input: None,
        };
        assert!(blocking.should_block());
        assert_eq!(blocking.block_reason(), Some("dangerous"));

        let passing = HookResponse::Ack;
        assert!(!passing.should_block());
        assert_eq!(passing.block_reason(), None);
    }

    #[test]
    fn event_serialization_roundtrip() {
        let event = HookEvent::PostToolUse {
            session_id: "s1".into(),
            tool_name: "read".into(),
            tool_input: serde_json::json!({"path": "/tmp/x"}),
            tool_output: serde_json::json!({"content": "hello"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type(), HookEventType::PostToolUse);
        assert_eq!(parsed.tool_name(), Some("read"));
    }

    #[test]
    fn response_serialization_roundtrip() {
        let resp = HookResponse::PostToolUse {
            should_stop: false,
            additional_contexts: vec!["ctx1".into()],
            feedback_message: Some("ok".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: HookResponse = serde_json::from_str(&json).unwrap();
        assert!(!parsed.should_block());
    }
}
