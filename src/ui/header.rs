use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
    prelude::Stylize,
};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let status_text = match app.state {
        crate::app::AppState::Boot => "BOOT SEQUENCE",
        crate::app::AppState::ApiKeyInput => "AWAITING KEY",
        crate::app::AppState::SelectingModel => "MODEL SELECT",
        crate::app::AppState::PickingFile => "PAYLOAD INJECTION - AWAITING FILE",
        crate::app::AppState::ProcessingImage => "PAYLOAD INJECTION - PROCESSING",
        crate::app::AppState::Idle => "REACTING CORE - NOMINAL",
        crate::app::AppState::WaitingResponse => "PROCESSING - STREAM ACTIVE",
        crate::app::AppState::Error(_) => "CRITICAL EXCURSION",
    };

    let model_text = if app.model.is_empty() {
        "NO MODEL"
    } else {
        &app.model
    };

    let msgs_count = app.messages.len();

    let left = format!(
        "[!] SYS: {}  |  TUNNEL: {}  |  MSGS: {}",
        status_text, model_text, msgs_count
    );

    let right = "DOMA v0.1";

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Green).bold()),
        Span::raw("  "),
        Span::styled(right, Style::default().fg(Color::Rgb(255, 176, 0)).bold()),
    ]);

    let block = Block::default()
        .style(Style::default().fg(Color::Green));

    let paragraph = ratatui::widgets::Paragraph::new(line)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);
}
