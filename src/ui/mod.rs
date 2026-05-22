pub mod layout;
pub mod chat;
pub mod status;
pub mod input;
pub mod header;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use crate::app::{App, AppState};

const MULTIMODAL_MODELS: &[&str] = &[
    "glm-5", "glm-5.1",
    "kimi-k2.5", "kimi-k2.6",
    "mimo-v2.5", "mimo-v2.5-pro",
    "qwen3.5-plus", "qwen3.6-plus",
];

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.state {
        AppState::ApiKeyInput => render_key_input(frame, app),
        AppState::SelectingModel => {
            render_main(frame, app);
            render_model_selection(frame, app);
        }
        AppState::SessionList => {
            render_main(frame, app);
            render_session_list(frame, app);
        }
        AppState::Searching => {
            render_main(frame, app);
            render_search_overlay(frame, app);
        }
        AppState::Exporting => {
            render_main(frame, app);
            render_overlay(frame, "EXPORTING SESSION...", "Choose save location", Color::Green);
        }
        AppState::Importing => {
            render_main(frame, app);
            render_overlay(frame, "IMPORTING SESSION...", "Pick a session file", Color::Green);
        }
        AppState::PickingFile => {
            render_main(frame, app);
            render_overlay(frame, "AWAITING PAYLOAD INJECTION...", "Select an image file from the dialog", Color::Green);
        }
        AppState::ProcessingImage => {
            render_main(frame, app);
            render_overlay(frame, "PROCESSING PAYLOAD...", "Compressing and encoding image", Color::Rgb(255, 176, 0));
        }
        AppState::Error(ref msg) => {
            let msg = msg.clone();
            render_main(frame, app);
            render_error_overlay(frame, &msg);
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
            let base_style = if *model == app.model {
                Style::default().fg(Color::Rgb(255, 176, 0))
            } else {
                Style::default().fg(Color::Green)
            };
            let img_tag = if MULTIMODAL_MODELS.contains(&model.as_str()) {
                " [IMG]"
            } else {
                ""
            };
            ListItem::new(format!("{}{}{}", prefix, model, img_tag)).style(base_style)
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

fn render_session_list(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_height = (app.sessions.len() as u16 + 4).min(area.height.saturating_sub(4));

    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(popup_height),
            Constraint::Percentage(30),
        ])
        .split(area)[1];

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(50),
            Constraint::Percentage(20),
        ])
        .split(popup_area)[1];

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .map(|session| {
            let is_active = session.id == app.active_session_id;
            let prefix = if is_active { " > " } else { "   " };
            let style = if is_active {
                Style::default().fg(Color::Rgb(255, 176, 0))
            } else {
                Style::default().fg(Color::Green)
            };

            let ts = chrono::DateTime::from_timestamp(session.created_at as i64, 0)
                .map(|dt| dt.format("%b %d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let title = if session.title.len() > 32 {
                format!("{}...", &session.title[..32])
            } else {
                session.title.clone()
            };

            ListItem::new(format!("{}{}  {}", prefix, title, ts)).style(style)
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.session_selection_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .style(Style::default().fg(Color::Green))
                .title(" SESSIONS ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().fg(Color::Rgb(255, 176, 0)));

    frame.render_stateful_widget(list, popup_area, &mut list_state);

    let hint = " [Up/Down] Navigate  [Enter] Switch  [Esc] Cancel ";
    let hint_area = Rect::new(popup_area.x, popup_area.bottom(), popup_area.width, 1);
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    frame.render_widget(hint_widget, hint_area);
}

fn render_search_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(
                (app.search_results.len() as u16 + 6).min(area.height.saturating_sub(8)),
            ),
            Constraint::Percentage(20),
        ])
        .split(area)[1];

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Length(55),
            Constraint::Percentage(15),
        ])
        .split(popup_area)[1];

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(Color::Green))
        .title(" SEARCH ")
        .title_alignment(Alignment::Center);

    let search_display = if app.search_query.is_empty() {
        "Type to search...".to_string()
    } else {
        format!("> {}  ({} results)", app.search_query, app.search_results.len())
    };

    let mut lines = vec![
        Line::from(Span::styled(search_display, Style::default().fg(Color::Rgb(255, 176, 0)))),
        Line::from(""),
    ];

    let max_results = (popup_area.height.saturating_sub(6) as usize).max(1);
    for (i, (_, msg)) in app.search_results.iter().take(max_results).enumerate() {
        let prefix = if i == app.search_index { " > " } else { "   " };
        let role = if msg.role == "user" { "U" } else { "A" };
        let text = msg.content.lines().next().unwrap_or(&msg.content);
        let display = if text.len() > 40 {
            format!("{}...", &text[..40])
        } else {
            text.to_string()
        };
        let style = if i == app.search_index {
            Style::default().fg(Color::Rgb(255, 176, 0))
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}: {}", prefix, role, display),
            style,
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, popup_area);

    let hint = " Type query  [Up/Down] Navigate  [Enter] Jump  [Esc] Cancel ";
    let hint_area = Rect::new(popup_area.x, popup_area.bottom(), popup_area.width, 1);
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    frame.render_widget(hint_widget, hint_area);
}

fn render_overlay(frame: &mut Frame, title: &str, subtitle: &str, color: Color) {
    let area = frame.area();

    let overlay_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area)[1];

    let overlay_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(40),
            Constraint::Percentage(30),
        ])
        .split(overlay_area)[1];

    frame.render_widget(Clear, overlay_area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(title.to_string(), Style::default().fg(color))),
        Line::from(""),
        Line::from(Span::styled(subtitle.to_string(), Style::default().fg(Color::Green))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(color))
        .title(" OPERATION ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, overlay_area);
}

fn render_error_overlay(frame: &mut Frame, msg: &str) {
    let area = frame.area();

    let overlay_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area)[1];

    let overlay_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(50),
            Constraint::Percentage(20),
        ])
        .split(overlay_area)[1];

    frame.render_widget(Clear, overlay_area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "[!!] CRITICAL EXCURSION [!!]".to_string(),
            Style::default().fg(Color::Rgb(255, 176, 0)),
        )),
        Line::from(""),
        Line::from(Span::styled(msg.to_string(), Style::default().fg(Color::Red))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .style(Style::default().fg(Color::Red))
        .title(" SYSTEM FAULT ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);

    let hint = " [Esc/Enter] Dismiss ";
    let hint_area = Rect::new(overlay_area.x, overlay_area.bottom(), overlay_area.width, 1);
    let hint_widget = Paragraph::new(hint)
        .style(Style::default().fg(Color::Red))
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
