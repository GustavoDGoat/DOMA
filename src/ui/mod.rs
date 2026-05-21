pub mod layout;
pub mod chat;
pub mod status;
pub mod input;
pub mod header;

use ratatui::{
    Frame,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::Alignment,
};
use crate::app::{App, AppState};

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.state {
        AppState::ApiKeyInput => render_key_input(frame, app),
        _ => render_main(frame, app),
    }
}

fn render_key_input(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let title = Line::from("DOMA - API KEY REQUIRED");
    let prompt = if let Some(ref err) = app.key_error {
        format!("[!!] {} [!!]\n\nEnter your OpenCode API key:", err)
    } else {
        "Enter your OpenCode API key to initialize:\n(Press Enter to confirm, Esc to quit)".to_string()
    };

    let masked = if app.key_input_buffer.len() > 8 {
        let visible = &app.key_input_buffer[..4];
        format!("{}****", visible)
    } else {
        "*".repeat(app.key_input_buffer.len())
    };
    let input_display = if app.key_input_buffer.is_empty() {
        " > ".to_string()
    } else {
        format!(" > {}", masked)
    };

    let text = vec![
        title,
        Line::from(""),
        Line::from(prompt),
        Line::from(""),
        Line::from(input_display),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(Color::Green))
        .title(" AUTHENTICATION ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Rgb(255, 176, 0)))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_main(frame: &mut Frame, app: &mut App) {
    let chunks = layout::layout(frame.area());
    header::render(frame, chunks[0], app);
    chat::render(frame, chunks[1], app);
    input::render(frame, chunks[2], app);
    status::render(frame, chunks[3], app);
}
