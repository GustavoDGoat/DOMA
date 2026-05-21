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

    let line = if app.input.is_empty() {
        Line::from(Span::styled(
            "USER > Awaiting input...",
            Style::default().fg(Color::Green),
        ))
    } else {
        Line::from(vec![
            Span::styled("USER > ", Style::default().fg(Color::Rgb(255, 176, 0))),
            Span::styled(
                app.input.clone(),
                Style::default().fg(Color::Rgb(255, 176, 0)),
            ),
        ])
    };

    let paragraph = Paragraph::new(line)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);

    if app.cursor_visible && !app.input.is_empty() {
        let x = area.x + 1 + 7 + app.input.len() as u16;
        let y = area.y + 1;
        frame.set_cursor_position((x, y));
    }
}
