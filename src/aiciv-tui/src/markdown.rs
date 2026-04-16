//! Simple markdown rendering for terminal display.
//!
//! Uses pulldown-cmark to parse markdown and converts to styled ratatui Spans.
//! Keeps it minimal — headers, bold, italic, code blocks, inline code, lists.
//! We don't need full GFM support.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Convert a markdown string into styled ratatui Lines.
///
/// Supports:
/// - Headers (##) — bold + color
/// - Bold (**text**) — bold modifier
/// - Italic (*text*) — italic modifier
/// - Inline code (`code`) — gray background style
/// - Code blocks (```...```) — gray style, preserved as-is
/// - Lists (- or *) — bullet prefix
/// - Paragraphs — normal text with blank line separation
pub fn markdown_to_lines(input: &str) -> Vec<Line<'static>> {
    let options = Options::empty();
    let parser = Parser::new_ext(input, options);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default()];
    let mut in_code_block = false;
    let mut in_list = false;
    let mut list_item_started = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        }
                        pulldown_cmark::HeadingLevel::H2 => {
                            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                        }
                        _ => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    };
                    style_stack.push(style);
                }
                Tag::Strong => {
                    let base = *style_stack.last().unwrap_or(&Style::default());
                    style_stack.push(base.add_modifier(Modifier::BOLD));
                }
                Tag::Emphasis => {
                    let base = *style_stack.last().unwrap_or(&Style::default());
                    style_stack.push(base.add_modifier(Modifier::ITALIC));
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    // Flush current line
                    if !current_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                }
                Tag::List(_) => {
                    in_list = true;
                }
                Tag::Item => {
                    list_item_started = true;
                }
                Tag::Paragraph => {
                    // Add blank line between paragraphs
                    if !lines.is_empty() && !in_list {
                        lines.push(Line::from(""));
                    }
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    if !current_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                }
                TagEnd::Strong | TagEnd::Emphasis => {
                    style_stack.pop();
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                }
                TagEnd::List(_) => {
                    in_list = false;
                }
                TagEnd::Item => {
                    if !current_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                }
                TagEnd::Paragraph => {
                    if !current_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    let style = Style::default().fg(Color::DarkGray);
                    for line_text in text.split('\n') {
                        if !line_text.is_empty() {
                            lines.push(Line::from(Span::styled(
                                format!("  {line_text}"),
                                style,
                            )));
                        }
                    }
                } else {
                    let style = *style_stack.last().unwrap_or(&Style::default());
                    let text_str = text.to_string();
                    if list_item_started {
                        current_spans.push(Span::styled(
                            format!("  * {text_str}"),
                            style,
                        ));
                        list_item_started = false;
                    } else {
                        current_spans.push(Span::styled(text_str, style));
                    }
                }
            }
            Event::Code(code) => {
                let style = Style::default().fg(Color::Yellow);
                current_spans.push(Span::styled(format!("`{code}`"), style));
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                if !current_spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_spans)));
                }
            }
            _ => {}
        }
    }

    // Flush remaining spans
    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text() {
        let lines = markdown_to_lines("Hello, world!");
        assert!(!lines.is_empty());
    }

    #[test]
    fn header_renders() {
        let lines = markdown_to_lines("# Title\n\nBody text");
        assert!(lines.len() >= 2);
    }

    #[test]
    fn code_block_renders() {
        let lines = markdown_to_lines("```\nlet x = 1;\n```");
        assert!(!lines.is_empty());
    }

    #[test]
    fn list_renders() {
        let lines = markdown_to_lines("- item one\n- item two\n- item three");
        assert!(lines.len() >= 3);
    }

    #[test]
    fn empty_input() {
        let lines = markdown_to_lines("");
        assert!(lines.is_empty());
    }
}
