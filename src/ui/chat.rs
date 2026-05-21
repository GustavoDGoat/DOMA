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

    while let Some(start) = s.find("```") {
        if let Some(end) = s[start + 3..].find("```") {
            s.replace_range(start..=start + 3 + end, "");
        } else {
            s.replace_range(start..start + 3, "");
        }
    }

    let mut result = String::with_capacity(s.len());
    let mut in_backtick = false;
    for ch in s.chars() {
        if ch == '`' {
            in_backtick = !in_backtick;
        } else if !in_backtick {
            result.push(ch);
        }
    }

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

pub fn count_rendered_lines(app: &App, width: u16) -> usize {
    let inner_width = width.saturating_sub(2) as usize;
    if inner_width == 0 {
        return 0;
    }

    let mut total = 0;

    for msg in &app.messages {
        let content = msg.text_content();
        let prefix_len = match msg.role.as_str() {
            "user" => 7,
            "assistant" => 11,
            _ => msg.role.len() + 3,
        };
        for (i, line_text) in content.lines().enumerate() {
            if line_text.is_empty() {
                total += 1;
            } else {
                let indent = if i == 0 { prefix_len } else { 2 };
                let text = strip_markdown(line_text);
                let line_len = indent + text.chars().count();
                total += (line_len + inner_width - 1) / inner_width.max(1);
            }
        }
    }

    if !app.current_response.is_empty() {
        for (i, line_text) in app.current_response.lines().enumerate() {
            if line_text.is_empty() {
                total += 1;
            } else {
                let indent = if i == 0 { 11 } else { 2 };
                let text = strip_markdown(line_text);
                let line_len = indent + text.chars().count();
                total += (line_len + inner_width - 1) / inner_width.max(1);
            }
        }
    }

    total
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

        for (i, line_text) in content.lines().enumerate() {
            if line_text.is_empty() {
                lines.push(Line::from(""));
            } else if i == 0 {
                let text = strip_markdown(line_text);
                lines.push(Line::from(vec![
                    Span::styled(format!("{} > ", role_label), style),
                    Span::styled(text, style),
                ]));
            } else {
                let text = strip_markdown(line_text);
                lines.push(Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(text, style),
                ]));
            }
        }
    }

    if !app.current_response.is_empty() {
        let style = Style::default().fg(Color::Green);

        for (i, line_text) in app.current_response.lines().enumerate() {
            if line_text.is_empty() {
                lines.push(Line::from(""));
            } else if i == 0 {
                let text = strip_markdown(line_text);
                lines.push(Line::from(vec![
                    Span::styled("ASSISTANT > ".to_string(), style),
                    Span::styled(text, style),
                ]));
            } else {
                let text = strip_markdown(line_text);
                lines.push(Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(text, style),
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

    let total_lines = count_rendered_lines(app, area.width);
    let visible_height = area.height.saturating_sub(2) as usize;

    let scroll = if app.follow_bottom {
        total_lines.saturating_sub(visible_height)
    } else {
        app.scroll_offset.min(total_lines.saturating_sub(visible_height))
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    frame.render_widget(paragraph, area);
}
