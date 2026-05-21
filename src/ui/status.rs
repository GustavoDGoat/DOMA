use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, _app: &App) {
    let block = Block::default().style(Style::default().fg(Color::Green));

    let left = "[Ctrl+M] Select Model  |  [Ctrl+N] New Session";
    let right = "[Ctrl+Q] Purge  |  [Esc] Safe Mode";

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled(right, Style::default().fg(Color::Rgb(255, 176, 0))),
    ]);

    let paragraph = ratatui::widgets::Paragraph::new(line)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);
}
