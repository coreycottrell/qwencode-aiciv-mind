//! Built-in tool implementations — the agent's hands.
//!
//! These are real, functional tools that interact with the filesystem
//! and shell. They produce output the LLM can reason about.

pub mod bash;
pub mod read;
pub mod write;
pub mod glob;
pub mod grep;
pub mod web_search;
pub mod web_fetch;

use crate::registry::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;

/// Register all built-in tools into a registry.
pub fn register_builtins(registry: &mut ToolRegistry, workspace_root: PathBuf) {
    registry.register(Arc::new(bash::BashTool::new(workspace_root.clone())));
    registry.register(Arc::new(read::ReadTool));
    registry.register(Arc::new(write::WriteTool));
    registry.register(Arc::new(glob::GlobTool::new(workspace_root.clone())));
    registry.register(Arc::new(grep::GrepTool::new(workspace_root)));
    registry.register(Arc::new(web_search::WebSearchTool));
    registry.register(Arc::new(web_fetch::WebFetchTool));
}
