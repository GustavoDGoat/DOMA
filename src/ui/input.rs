use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(Color::Green));

    let mut spans: Vec<Span> = vec![
        Span::styled("USER > ", Style::default().fg(Color::Rgb(255, 176, 0))),
    ];

    if let Some(ref attachment) = app.attached_image {
        spans.push(Span::styled(
            format!("[ATTACHED: {}] ", attachment.filename),
            Style::default().fg(Color::Green),
        ));
    }

    if app.input.is_empty() && app.attached_image.is_none() {
        spans.push(Span::styled(
            "Awaiting input...",
            Style::default().fg(Color::Green),
        ));
    } else {
        spans.push(Span::styled(
            app.input.clone(),
            Style::default().fg(Color::Rgb(255, 176, 0)),
        ));
    }

    let line = Line::from(spans);

    let paragraph = Paragraph::new(line)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);

    if app.cursor_visible && !app.input.is_empty() {
        let prefix_len = 7 + app.attached_image.as_ref().map(|a| a.filename.len() + 13).unwrap_or(0);
        let x = area.x + 1 + prefix_len as u16 + app.input.len() as u16;
        let y = area.y + 1;
        frame.set_cursor_position((x, y));
    }
}
