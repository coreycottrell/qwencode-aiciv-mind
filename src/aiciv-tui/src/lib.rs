//! # aiciv-tui — Minimal Terminal Chat UI for aiciv-mind
//!
//! A fresh, minimal terminal chat interface built with ratatui + crossterm
//! that connects to the ThinkLoop reasoning engine. This is NOT a port of the
//! full 56K-line Codex TUI — it is purpose-built for aiciv-mind's needs.
//!
//! ## Architecture
//!
//! - `app` — Core `App` struct with event loop, input handling, and state
//! - `ui` — UI rendering (header, chat area, input, status bar)
//! - `sanitize` — Output sanitization to prevent terminal injection attacks
//! - `markdown` — Simple markdown-to-styled-text conversion for terminal display

pub mod app;
pub mod markdown;
pub mod sanitize;
pub mod ui;
