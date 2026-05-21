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

    let input_style = if app.input.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Rgb(255, 176, 0))
    };

    let input_text = if app.input.is_empty() {
        "Awaiting input...".to_string()
    } else {
        app.input.clone()
    };

    let lines = vec![
        Line::from(Span::styled("USER > ", Style::default().fg(Color::Rgb(255, 176, 0)))),
        Line::from(Span::styled(input_text, input_style)),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);

    if app.cursor_visible && !app.input.is_empty() {
        let x = area.x + 1 + 7 + app.input.len() as u16;
        let y = area.y + 1;
        frame.set_cursor_position((x, y));
    }
}
