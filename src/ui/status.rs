use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().style(Style::default().fg(Color::Green));

    let scroll_indicator = if !app.follow_bottom {
        " [SCROLL LOCK] "
    } else {
        ""
    };

    let left = format!(
        "[Ctrl+M] Model  |  [Ctrl+N] New Session{}",
        scroll_indicator
    );
    let right = "[Ctrl+Q] Quit  |  [Esc] Cancel";

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
