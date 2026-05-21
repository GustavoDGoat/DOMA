use crate::client::{ChatMessage, OpenCodeClient};
use crate::config::Settings;
use crate::storage::{SessionMeta, StorageEngine, StoredMessage};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Boot,
    ApiKeyInput,
    Idle,
    WaitingResponse,
    Error(String),
}

pub struct App {
    pub state: AppState,
    storage: StorageEngine,
    settings: Settings,

    pub sessions: Vec<SessionMeta>,
    pub active_session_id: String,
    pub messages: Vec<ChatMessage>,

    pub input: String,
    pub current_response: String,
    pub scroll_offset: usize,
    pub should_quit: bool,

    stream_rx: Option<mpsc::Receiver<String>>,
    stream_handle: Option<tokio::task::JoinHandle<()>>,

    pub cursor_visible: bool,

    pub key_input_buffer: String,
    pub key_error: Option<String>,

    pub model: String,
}

impl App {
    fn new(storage: StorageEngine, settings: Settings) -> Self {
        Self {
            state: AppState::Boot,
            storage,
            settings,
            sessions: Vec::new(),
            active_session_id: String::new(),
            messages: Vec::new(),
            input: String::new(),
            current_response: String::new(),
            scroll_offset: 0,
            should_quit: false,
            stream_rx: None,
            stream_handle: None,
            cursor_visible: true,
            key_input_buffer: String::new(),
            key_error: None,
            model: String::new(),
        }
    }

    fn ensure_session(&mut self) -> Result<()> {
        if self.active_session_id.is_empty() {
            let id = self.storage.create_session("New Session")?;
            self.active_session_id = id;
            self.settings.set_active_session_id(&self.active_session_id)?;
        }
        Ok(())
    }

    fn load_messages(&mut self) -> Result<()> {
        let stored = self.storage.list_messages(&self.active_session_id)?;
        self.messages = stored
            .into_iter()
            .map(|m| ChatMessage::new_text(&m.role, &m.content))
            .collect();
        Ok(())
    }

    fn save_message(&self, role: &str, content: &str) -> Result<()> {
        let msg = StoredMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.storage.append_message(&self.active_session_id, &msg)?;
        Ok(())
    }

    fn send_message(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        let user_msg = ChatMessage::new_text("user", self.input.trim());
        let _ = self.save_message("user", self.input.trim());

        let mut api_messages = self.messages.clone();
        api_messages.push(user_msg.clone());
        self.messages.push(user_msg);
        self.input.clear();
        self.current_response.clear();

        let base_url = self.settings.api_base_url();
        let api_key = self.settings.api_key().unwrap_or_default();
        let model = self.model.clone();

        let (tx, rx) = mpsc::channel::<String>(256);
        self.stream_rx = Some(rx);

        let handle = tokio::spawn(async move {
            let client = OpenCodeClient::new(&base_url, &api_key);
            match client.chat_completions(api_messages, &model).await {
                Ok(mut stream) => {
                    use futures_util::StreamExt;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        let _ = tx.send(content).await;
                                    }
                                    if choice.finish_reason.is_some() {
                                        let _ = tx.send("\n[DONE]".to_string()).await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(format!("\n[ERROR: {}]", e)).await;
                                let _ = tx.send("\n[DONE]".to_string()).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("\n[ERROR: {}]", e)).await;
                    let _ = tx.send("\n[DONE]".to_string()).await;
                }
            }
        });

        self.stream_handle = Some(handle);
        self.state = AppState::WaitingResponse;
    }
}

pub async fn run(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
    let storage = StorageEngine::new()?;
    let settings = Settings::new(storage.clone());
    let mut app = App::new(storage, settings);

    app.state = AppState::Boot;
    terminal.draw(|f| ui::render(f, &mut app))?;

    if app.settings.api_key().is_none() {
        app.state = AppState::ApiKeyInput;
    } else {
        let base_url = app.settings.api_base_url();
        let api_key = app.settings.api_key().unwrap();
        let client = OpenCodeClient::new(&base_url, &api_key);
        match client.validate_key().await {
            Ok(true) => {
                let models = client.list_models().await.unwrap_or_default();
                app.model = models.first().cloned().unwrap_or_else(|| "gpt-4o".to_string());
                app.state = AppState::Idle;
            }
            Ok(false) => {
                app.state = AppState::ApiKeyInput;
                app.key_error = Some("Invalid API key. Please re-enter.".to_string());
            }
            Err(_) => {
                app.state = AppState::ApiKeyInput;
                app.key_error = Some("Could not validate key. Check your network.".to_string());
            }
        }
    }

    app.ensure_session()?;
    app.load_messages()?;

    let tick_rate = Duration::from_millis(16);

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if app.should_quit {
            break;
        }

        let event_available = event::poll(tick_rate)?;

        if event_available {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key).await?;
            }
        }

        let mut content_buf = String::new();
        let mut stream_done = false;

        if let Some(rx) = &mut app.stream_rx {
            while let Ok(content) = rx.try_recv() {
                if content == "\n[DONE]" {
                    stream_done = true;
                } else {
                    content_buf.push_str(&content);
                }
            }
        }

        if stream_done {
            let _ = app.save_message("assistant", &content_buf);
            app.messages
                .push(ChatMessage::new_text("assistant", &content_buf));
            app.current_response.clear();
            app.state = AppState::Idle;
            app.stream_rx = None;
            app.stream_handle = None;
        } else if !content_buf.is_empty() {
            app.current_response.push_str(&content_buf);
        }
    }

    Ok(())
}

async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.state {
        AppState::ApiKeyInput => handle_key_input_state(app, key).await?,
        AppState::Idle => handle_idle_state(app, key)?,
        AppState::WaitingResponse => handle_waiting_state(app, key)?,
        AppState::Error(_) => {
            if key.code == KeyCode::Esc {
                app.state = AppState::Idle;
            }
        }
        AppState::Boot => {}
    }
    Ok(())
}

async fn handle_key_input_state(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            app.key_input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.key_input_buffer.pop();
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            if app.key_input_buffer.is_empty() {
                return Ok(());
            }
            let base_url = app.settings.api_base_url();
            let client = OpenCodeClient::new(&base_url, &app.key_input_buffer);
            match client.validate_key().await {
                Ok(true) => {
                    let _ = app.settings.set_api_key(&app.key_input_buffer);
                    let models = client.list_models().await.unwrap_or_default();
                    app.model = models.first().cloned().unwrap_or_else(|| "gpt-4o".to_string());
                    app.key_input_buffer.clear();
                    app.key_error = None;
                    app.state = AppState::Idle;
                }
                Ok(false) => {
                    app.key_error = Some("Invalid API key".to_string());
                }
                Err(e) => {
                    app.key_error = Some(format!("Connection error: {}", e));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_idle_state(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers == KeyModifiers::CONTROL {
                match c {
                    'q' | 'Q' => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    'n' | 'N' => {
                        let id = app.storage.create_session("New Session")?;
                        app.active_session_id = id;
                        app.messages.clear();
                        app.current_response.clear();
                        app.input.clear();
                        return Ok(());
                    }
                    'p' | 'P' => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
            app.input.push(c);
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Enter => {
            app.send_message();
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_add(5);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_sub(5);
        }
        _ => {}
    }
    Ok(())
}

fn handle_waiting_state(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            if let Some(handle) = app.stream_handle.take() {
                handle.abort();
            }
            app.stream_rx = None;
            app.state = AppState::Idle;
        }
        KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
            if let Some(handle) = app.stream_handle.take() {
                handle.abort();
            }
            app.should_quit = true;
        }
        _ => {}
    }
    Ok(())
}
