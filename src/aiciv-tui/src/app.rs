//! Core application state and event loop.
//!
//! The `App` struct holds all TUI state: messages, input buffer, scroll offset,
//! and flags for running/thinking state. The main loop processes keyboard events
//! via crossterm and renders via ratatui.

use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tracing::info;

use codex_exec::ToolExecutor;
use codex_llm::ollama::ToolSchema;
use codex_llm::prompt::PromptBuilder;
use codex_llm::think_loop::ThinkLoop;
use codex_roles::Role;

/// A single entry in the chat history.
#[derive(Debug, Clone)]
pub enum ChatEntry {
    /// User's input message.
    User(String),
    /// Assistant's text response.
    Assistant(String),
    /// A tool call made by the assistant.
    ToolCall { name: String, args: String },
    /// Result from a tool execution.
    ToolResult { name: String, result: String },
    /// System message (informational).
    System(String),
    /// Error message.
    Error(String),
}

/// Core application state.
pub struct App {
    /// Chat messages displayed in the UI.
    messages: Vec<ChatEntry>,
    /// Current text input buffer.
    input: String,
    /// Cursor position within the input buffer (byte offset).
    cursor_pos: usize,
    /// Scroll offset for the chat view (reserved for future use).
    #[allow(dead_code)]
    scroll_offset: u16,
    /// Whether the app is running.
    running: bool,
    /// Whether we are waiting for an LLM response.
    thinking: bool,
    /// Status message displayed in the status bar.
    status: String,
    /// Model name for display.
    model_name: String,
}

impl App {
    /// Create a new App with the given model name for display.
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            running: true,
            thinking: false,
            status: "Ready".into(),
            model_name: model_name.into(),
        }
    }

    /// Run the main event loop.
    ///
    /// This is the heart of the TUI. It:
    /// 1. Renders the current state
    /// 2. Waits for keyboard input
    /// 3. Processes input (typing, submit, quit)
    /// 4. On submit: sends to ThinkLoop, displays results
    /// 5. Loops until quit
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        think_loop: &ThinkLoop,
        executor: &ToolExecutor,
        prompt_builder: &PromptBuilder,
        role: Role,
        tool_schemas: &[ToolSchema],
    ) -> Result<()> {
        self.messages.push(ChatEntry::System(
            "Welcome to aiciv-mind TUI. Type a message and press Enter.".into(),
        ));

        while self.running {
            // Render
            terminal.draw(|frame| crate::ui::render(self, frame))?;

            // Wait for event with 100ms timeout for responsive feel
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.thinking {
                        // While thinking, only allow Ctrl+C to quit
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            self.running = false;
                        }
                        continue;
                    }

                    match self.handle_key_event(key) {
                        KeyAction::None => {}
                        KeyAction::Submit => {
                            self.handle_submit(
                                think_loop,
                                executor,
                                prompt_builder,
                                role,
                                tool_schemas,
                            )
                            .await;
                        }
                        KeyAction::Quit => {
                            self.running = false;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a key event and return what action to take.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                KeyAction::Quit
            }
            KeyCode::Enter => {
                if self.input.trim().is_empty() {
                    KeyAction::None
                } else {
                    KeyAction::Submit
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += c.len_utf8();
                KeyAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    // Find the previous character boundary
                    let prev = self.input[..self.cursor_pos]
                        .char_indices()
                        .next_back()
                        .map(|(idx, _)| idx)
                        .unwrap_or(0);
                    self.input.drain(prev..self.cursor_pos);
                    self.cursor_pos = prev;
                }
                KeyAction::None
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    let next = self.input[self.cursor_pos..]
                        .char_indices()
                        .nth(1)
                        .map(|(idx, _)| self.cursor_pos + idx)
                        .unwrap_or(self.input.len());
                    self.input.drain(self.cursor_pos..next);
                }
                KeyAction::None
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos = self.input[..self.cursor_pos]
                        .char_indices()
                        .next_back()
                        .map(|(idx, _)| idx)
                        .unwrap_or(0);
                }
                KeyAction::None
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos = self.input[self.cursor_pos..]
                        .char_indices()
                        .nth(1)
                        .map(|(idx, _)| self.cursor_pos + idx)
                        .unwrap_or(self.input.len());
                }
                KeyAction::None
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                KeyAction::None
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    /// Take the current input, send it to the ThinkLoop, and display results.
    async fn handle_submit(
        &mut self,
        think_loop: &ThinkLoop,
        executor: &ToolExecutor,
        prompt_builder: &PromptBuilder,
        role: Role,
        tool_schemas: &[ToolSchema],
    ) {
        let user_input = std::mem::take(&mut self.input);
        self.cursor_pos = 0;

        // Add user message to chat
        self.messages.push(ChatEntry::User(user_input.clone()));
        self.thinking = true;
        self.status = "Thinking...".into();

        info!(input = %user_input, "Sending to ThinkLoop");

        // Run the ThinkLoop
        match think_loop
            .run(prompt_builder, &user_input, tool_schemas, executor, role)
            .await
        {
            Ok(result) => {
                // Display tool calls and results
                for record in &result.tool_calls_made {
                    self.messages.push(ChatEntry::ToolCall {
                        name: record.tool_name.clone(),
                        args: record.arguments.to_string(),
                    });
                    self.messages.push(ChatEntry::ToolResult {
                        name: record.tool_name.clone(),
                        result: if record.result.success {
                            truncate_output(&record.result.output, 500)
                        } else {
                            record
                                .result
                                .error
                                .as_deref()
                                .unwrap_or("Unknown error")
                                .to_string()
                        },
                    });
                }

                // Display final response
                if !result.response.is_empty() {
                    self.messages
                        .push(ChatEntry::Assistant(result.response.clone()));
                }

                self.status = format!(
                    "Done ({} iterations, {} tool calls)",
                    result.iterations,
                    result.tool_calls_made.len()
                );
            }
            Err(e) => {
                self.messages
                    .push(ChatEntry::Error(format!("ThinkLoop error: {e}")));
                self.status = "Error".into();
            }
        }

        self.thinking = false;
    }

    // ── Accessors for UI rendering ──

    pub fn messages(&self) -> &[ChatEntry] {
        &self.messages
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn is_thinking(&self) -> bool {
        self.thinking
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// Action resulting from a key event.
pub enum KeyAction {
    /// No action needed.
    None,
    /// Submit the current input.
    Submit,
    /// Quit the application.
    Quit,
}

/// Truncate output to a maximum number of characters for display.
fn truncate_output(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}... ({} bytes total)", &s[..max_chars], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_defaults() {
        let app = App::new("test-model");
        assert!(app.is_running());
        assert!(!app.is_thinking());
        assert_eq!(app.status(), "Ready");
        assert_eq!(app.model_name(), "test-model");
        assert!(app.messages().is_empty());
        assert!(app.input().is_empty());
        assert_eq!(app.cursor_pos(), 0);
    }

    #[test]
    fn typing_characters() {
        let mut app = App::new("test");
        app.handle_key_event(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        app.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        assert_eq!(app.input(), "hi");
        assert_eq!(app.cursor_pos(), 2);
    }

    #[test]
    fn backspace() {
        let mut app = App::new("test");
        app.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        app.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.input(), "a");
        assert_eq!(app.cursor_pos(), 1);
    }

    #[test]
    fn empty_submit_ignored() {
        let mut app = App::new("test");
        let action = app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(action, KeyAction::None));
    }

    #[test]
    fn ctrl_c_quits() {
        let mut app = App::new("test");
        let action =
            app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(matches!(action, KeyAction::Quit));
    }

    #[test]
    fn cursor_movement() {
        let mut app = App::new("test");
        app.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));

        // Move left
        app.handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.cursor_pos(), 2);

        // Move home
        app.handle_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(app.cursor_pos(), 0);

        // Move end
        app.handle_key_event(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(app.cursor_pos(), 3);
    }

    #[test]
    fn truncate_output_short() {
        assert_eq!(truncate_output("hello", 10), "hello");
    }

    #[test]
    fn truncate_output_long() {
        let long = "a".repeat(100);
        let truncated = truncate_output(&long, 10);
        assert!(truncated.starts_with("aaaaaaaaaa..."));
        assert!(truncated.contains("100 bytes total"));
    }
}
