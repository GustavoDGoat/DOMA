use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::app::App;

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

        for line_text in content.lines() {
            let prefix = format!("{} > ", role_label);
            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(line_text.to_string(), style),
            ]);
            lines.push(line);
        }
    }

    if !app.current_response.is_empty() {
        let style = Style::default().fg(Color::Green);
        for line_text in app.current_response.lines() {
            let prefix = "ASSISTANT > ".to_string();
            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(line_text.to_string(), style),
            ]);
            lines.push(line);
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
