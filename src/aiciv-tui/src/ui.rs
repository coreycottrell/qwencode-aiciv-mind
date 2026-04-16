//! UI rendering — draws the terminal interface.
//!
//! Layout:
//! ```text
//! +-------------------------------------------+
//! | aiciv-mind v0.1.0 | model: devstral-24b   |  <- header (1 line)
//! +-------------------------------------------+
//! | [user] What is 2+2?                       |
//! | [assistant] 2+2 = 4                       |  <- chat area (flexible)
//! | [tool:bash] {"command": "ls"}             |
//! | [result:bash] src/ Cargo.toml             |
//! +-------------------------------------------+
//! | > type your message here_                 |  <- input area (3 lines)
//! +-------------------------------------------+
//! | Ready | Ctrl+C: quit | Enter: send       |  <- status bar (1 line)
//! +-------------------------------------------+
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, ChatEntry};
use crate::sanitize::sanitize_for_terminal;

/// Render the complete UI.
pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Min(5),    // chat area
            Constraint::Length(3), // input area
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_chat(app, frame, chunks[1]);
    render_input(app, frame, chunks[2]);
    render_status(app, frame, chunks[3]);
}

/// Render the header bar.
fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " aiciv-mind v0.1.0",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | model: "),
        Span::styled(
            app.model_name(),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(header, area);
}

/// Render the chat message area.
fn render_chat(app: &App, frame: &mut Frame, area: Rect) {
    let messages = app.messages();
    let items: Vec<ListItem> = messages
        .iter()
        .map(|entry| {
            let (prefix, style, content) = match entry {
                ChatEntry::User(text) => (
                    "[you] ",
                    Style::default().fg(Color::Green),
                    text.clone(),
                ),
                ChatEntry::Assistant(text) => (
                    "[assistant] ",
                    Style::default().fg(Color::Blue),
                    sanitize_for_terminal(text),
                ),
                ChatEntry::ToolCall { name, args } => (
                    "",
                    Style::default().fg(Color::Magenta),
                    format!("[tool:{name}] {args}"),
                ),
                ChatEntry::ToolResult { name, result } => {
                    let sanitized = sanitize_for_terminal(result);
                    (
                        "",
                        Style::default().fg(Color::DarkGray),
                        format!("[result:{name}] {sanitized}"),
                    )
                }
                ChatEntry::System(text) => (
                    "[system] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                    text.clone(),
                ),
                ChatEntry::Error(text) => (
                    "[error] ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                    text.clone(),
                ),
            };

            let line = Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::styled(content, style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let chat_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Chat "),
        );

    frame.render_widget(chat_list, area);
}

/// Render the input area.
fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let input_text = app.input();
    let cursor_pos = app.cursor_pos();

    let input = Paragraph::new(input_text.to_string())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.is_thinking() {
                    " Thinking... "
                } else {
                    " Input (Enter to send, Ctrl+C to quit) "
                }),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);

    // Position the cursor
    // +1 for border
    let cursor_x = area.x + 1 + cursor_pos as u16;
    let cursor_y = area.y + 1;
    if cursor_x < area.x + area.width - 1 {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Render the status bar.
fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let status_style = if app.is_thinking() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {} ", app.status()), status_style),
        Span::styled(
            "| Ctrl+C: quit | Enter: send",
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status, area);
}
