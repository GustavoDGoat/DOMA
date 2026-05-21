use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::app::App;

fn strip_markdown(text: &str) -> String {
    let mut s = text.to_string();

    // Remove code fences ```...```
    while let Some(start) = s.find("```") {
        if let Some(end) = s[start + 3..].find("```") {
            s.replace_range(start..=start + 3 + end, "");
        } else {
            s.replace_range(start..start + 3, "");
        }
    }

    // Remove inline code `...`
    let mut result = String::with_capacity(s.len());
    let mut in_backtick = false;
    for ch in s.chars() {
        if ch == '`' {
            in_backtick = !in_backtick;
        } else if !in_backtick {
            result.push(ch);
        }
    }

    // Remove ** and __ (bold/italic markers)
    let mut cleaned = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(ch) = chars.next() {
        let double = (ch == '*' || ch == '_') && chars.peek() == Some(&ch);
        if double {
            chars.next();
        } else {
            cleaned.push(ch);
        }
    }

    // Replace leading # markers
    let lines: Vec<String> = cleaned
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("### ") {
                format!("   {}", rest)
            } else if let Some(rest) = trimmed.strip_prefix("## ") {
                format!("  {}", rest)
            } else if let Some(rest) = trimmed.strip_prefix("# ") {
                format!(" {}", rest)
            } else {
                line.to_string()
            }
        })
        .collect();

    lines.join("\n")
}

fn make_prefix(role_label: &str, style: Style) -> Span<'static> {
    Span::styled(format!("{} > ", role_label), style)
}

fn make_content_span(text: &str, style: Style) -> Span<'static> {
    let stripped = strip_markdown(text);
    Span::styled(stripped, style)
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        let role_label = match msg.role.as_str() {
            "user" => "USER",
            "assistant" => "ASSISTANT",
            _ => &msg.role,
        };
        let content = msg.text_content();
        let style = match msg.role.as_str() {
            "user" => Style::default().fg(Color::Rgb(255, 176, 0)),
            _ => Style::default().fg(Color::Green),
        };
        let prefix_style = style;
        let prefix = make_prefix(role_label, prefix_style);

        for line_text in content.lines() {
            if line_text.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(vec![
                    prefix.clone(),
                    make_content_span(line_text, style),
                ]));
            }
        }
    }

    if !app.current_response.is_empty() {
        let style = Style::default().fg(Color::Green);
        let prefix = make_prefix("ASSISTANT", style);

        for line_text in app.current_response.lines() {
            if line_text.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(vec![
                    prefix.clone(),
                    make_content_span(line_text, style),
                ]));
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(Color::Green));

    if app.messages.is_empty() && app.current_response.is_empty() {
        let paragraph = Paragraph::new("")
            .block(block)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(paragraph, area);
        return;
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}
