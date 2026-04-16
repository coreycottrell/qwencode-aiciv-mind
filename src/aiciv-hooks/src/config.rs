//! Hook configuration — load from settings file.
//!
//! Hooks are configured as a list of entries, each mapping an event type to
//! a command (external executable) with optional tool-name filtering.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::types::HookEventType;

/// A single hook configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Which event type this hook fires on.
    pub event: HookEventType,

    /// External command to execute.
    pub command: String,

    /// Optional args to the command.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional: scope to specific tool names (for tool-use events only).
    #[serde(default)]
    pub tool_names: Option<Vec<String>>,

    /// Timeout in milliseconds. Default: 5000.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// If true, failure blocks the operation. If false, fail-open (Ack on error).
    #[serde(default)]
    pub required: bool,
}

fn default_timeout_ms() -> u64 {
    5000
}

impl HookConfig {
    /// The timeout as a Duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Whether this hook matches the given tool name.
    /// Returns true if no tool_names filter is set, or if the tool is in the list.
    pub fn matches_tool(&self, tool_name: &str) -> bool {
        match &self.tool_names {
            None => true,
            Some(names) => names.iter().any(|n| n == tool_name),
        }
    }
}

/// Top-level hooks configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksSettings {
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

impl HooksSettings {
    /// Load hooks from a JSON settings file.
    pub fn from_json_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("reading hooks config: {}", path.display()))?;
        let settings: Self = serde_json::from_str(&contents)
            .with_context(|| {
                format!("parsing hooks config: {}", path.display())
            })?;
        Ok(settings)
    }

    /// Load hooks from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("parsing hooks JSON")
    }

    /// Get all hooks for a given event type.
    pub fn hooks_for(&self, event_type: HookEventType) -> Vec<&HookConfig> {
        self.hooks.iter().filter(|h| h.event == event_type).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hooks_json() {
        let json = r#"{
            "hooks": [
                {
                    "event": "pre_tool_use",
                    "command": "/usr/local/bin/safety-checker",
                    "tool_names": ["bash", "shell"],
                    "timeout_ms": 3000,
                    "required": true
                },
                {
                    "event": "post_tool_use",
                    "command": "/usr/local/bin/memory-extractor",
                    "timeout_ms": 10000,
                    "required": false
                },
                {
                    "event": "session_start",
                    "command": "/usr/local/bin/init-hook"
                }
            ]
        }"#;
        let settings = HooksSettings::from_json(json).unwrap();
        assert_eq!(settings.hooks.len(), 3);

        let pre_hooks = settings.hooks_for(HookEventType::PreToolUse);
        assert_eq!(pre_hooks.len(), 1);
        assert!(pre_hooks[0].matches_tool("bash"));
        assert!(!pre_hooks[0].matches_tool("read"));
        assert!(pre_hooks[0].required);
        assert_eq!(pre_hooks[0].timeout(), Duration::from_millis(3000));

        let post_hooks = settings.hooks_for(HookEventType::PostToolUse);
        assert_eq!(post_hooks.len(), 1);
        assert!(post_hooks[0].matches_tool("anything"));

        let start_hooks = settings.hooks_for(HookEventType::SessionStart);
        assert_eq!(start_hooks.len(), 1);
        assert_eq!(start_hooks[0].timeout_ms, 5000); // default
    }

    #[test]
    fn empty_config() {
        let settings = HooksSettings::from_json("{}").unwrap();
        assert!(settings.hooks.is_empty());
        assert!(settings.hooks_for(HookEventType::Stop).is_empty());
    }
}
