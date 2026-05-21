pub mod layout;
pub mod chat;
pub mod status;
pub mod input;
pub mod header;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use crate::app::{App, AppState};

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.state {
        AppState::ApiKeyInput => render_key_input(frame, app),
        AppState::SelectingModel => {
            render_main(frame, app);
            render_model_selection(frame, app);
        }
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

fn render_model_selection(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length((app.model_list.len() as u16 + 4).min(area.height.saturating_sub(4))),
            Constraint::Percentage(30),
        ])
        .split(area)[1];

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(40),
            Constraint::Percentage(25),
        ])
        .split(popup_area)[1];

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = app
        .model_list
        .iter()
        .map(|model| {
            let prefix = if *model == app.model { " > " } else { "   " };
            let style = if *model == app.model {
                Style::default().fg(Color::Rgb(255, 176, 0))
            } else {
                Style::default().fg(Color::Green)
            };
            ListItem::new(format!("{}{}", prefix, model)).style(style)
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.model_selection_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .style(Style::default().fg(Color::Green))
                .title(" SELECT MODEL ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().fg(Color::Rgb(255, 176, 0)));

    frame.render_stateful_widget(list, popup_area, &mut list_state);

    let hint = " [Up/Down] Navigate  [Enter] Confirm  [Esc] Skip ";
    let hint_area = Rect::new(popup_area.x, popup_area.bottom(), popup_area.width, 1);
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    frame.render_widget(hint_widget, hint_area);
}

fn render_main(frame: &mut Frame, app: &mut App) {
    let chunks = layout::layout(frame.area());
    header::render(frame, chunks[0], app);
    chat::render(frame, chunks[1], app);
    input::render(frame, chunks[2], app);
    status::render(frame, chunks[3], app);
}
