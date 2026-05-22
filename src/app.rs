use crate::client::{ChatMessage, OpenCodeClient};
use crate::config::Settings;
use crate::storage::{SessionMeta, StorageEngine, StoredMessage};
use crate::ui;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use image::GenericImageView;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

const GO_MODELS_FREE: &[&str] = &[
    "deepseek-v4-flash",
    "qwen3.5-plus",
    "qwen3.6-plus",
    "minimax-m2.5",
    "minimax-m2.7",
    "mimo-v2.5",
    "mimo-v2.5-pro",
    "kimi-k2.5",
    "kimi-k2.6",
    "glm-5",
    "glm-5.1",
    "deepseek-v4-pro",
];

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 1024;

fn is_openai_compatible(model_id: &str) -> bool {
    !model_id.starts_with("minimax")
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub data_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Boot,
    ApiKeyInput,
    SelectingModel,
    SessionList,
    PickingFile,
    ProcessingImage,
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
    pub follow_bottom: bool,
    pub should_quit: bool,

    stream_rx: Option<mpsc::Receiver<String>>,
    stream_handle: Option<tokio::task::JoinHandle<()>>,

    pub cursor_visible: bool,

    pub key_input_buffer: String,
    pub key_error: Option<String>,

    pub model: String,
    pub model_list: Vec<String>,
    pub model_selection_index: usize,

    pub session_selection_index: usize,

    pub attached_image: Option<Attachment>,
    file_dialog_rx: Option<std_mpsc::Receiver<Result<PathBuf>>>,
    image_process_rx: Option<mpsc::Receiver<Result<Attachment>>>,
}

fn process_image(path: PathBuf) -> Result<Attachment> {
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());

    let metadata = std::fs::metadata(&path)?;
    if metadata.len() as usize > MAX_IMAGE_BYTES {
        anyhow::bail!("Image too large (max 20MB)");
    }

    let img = image::open(&path)?;
    let mut img = img;

    let (w, h) = img.dimensions();
    if w > MAX_IMAGE_DIMENSION || h > MAX_IMAGE_DIMENSION {
        img = img.resize(MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION, image::imageops::FilterType::Lanczos3);
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "png".to_string());

    let mime = match ext.as_str() {
        "jpg" | "jpeg" => {
            img.write_to(&mut buf, image::ImageFormat::Jpeg)?;
            "image/jpeg"
        }
        "gif" => {
            img.write_to(&mut buf, image::ImageFormat::Gif)?;
            "image/gif"
        }
        "webp" => {
            img.write_to(&mut buf, image::ImageFormat::WebP)?;
            "image/webp"
        }
        _ => {
            img.write_to(&mut buf, image::ImageFormat::Png)?;
            "image/png"
        }
    };

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, buf.get_ref());
    let data_url = format!("data:{};base64,{}", mime, b64);

    Ok(Attachment {
        filename,
        data_url,
    })
}

impl App {
    fn new(storage: StorageEngine, settings: Settings) -> Self {
        let saved_model = settings.model().unwrap_or_else(|| "deepseek-v4-flash".to_string());
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
            follow_bottom: true,
            should_quit: false,
            stream_rx: None,
            stream_handle: None,
            cursor_visible: true,
            key_input_buffer: String::new(),
            key_error: None,
            model: saved_model,
            model_list: Vec::new(),
            model_selection_index: 0,
            session_selection_index: 0,
            attached_image: None,
            file_dialog_rx: None,
            image_process_rx: None,
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

    fn open_file_dialog(&mut self) {
        let (tx, rx) = std_mpsc::channel();
        self.file_dialog_rx = Some(rx);

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("Images", IMAGE_EXTENSIONS)
                .pick_file();
            match result {
                Some(path) => { let _ = tx.send(Ok(path)); }
                None => { let _ = tx.send(Err(anyhow::anyhow!("No file selected"))); }
            }
        });

        self.state = AppState::PickingFile;
    }

    fn start_image_processing(&mut self, path: PathBuf) {
        let (tx, rx) = mpsc::channel(1);
        self.image_process_rx = Some(rx);

        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || process_image(path)).await;
            match result {
                Ok(Ok(attachment)) => { let _ = tx.send(Ok(attachment)).await; }
                Ok(Err(e)) => { let _ = tx.send(Err(anyhow::anyhow!("{}", e))).await; }
                Err(e) => { let _ = tx.send(Err(anyhow::anyhow!("Thread error: {}", e))).await; }
            }
        });

        self.state = AppState::ProcessingImage;
    }

    fn send_message(&mut self) {
        if self.input.trim().is_empty() && self.attached_image.is_none() {
            return;
        }

        let text = if self.input.trim().is_empty() {
            "[image]".to_string()
        } else {
            self.input.trim().to_string()
        };

        if text.starts_with('/') {
            self.input.clear();
            let cmd = text.to_lowercase();
            match cmd.as_str() {
                "/help" => {
                    let help = "Available commands:\n  /help    - Show this help\n  /clear   - Clear current session\n  /models  - Select model\n  /new     - New session\n  /undo    - Remove last response\n\nKeybindings:\n  Ctrl+P   Attach image\n  Ctrl+S   Switch session\n  Ctrl+M   Select model\n  Ctrl+N   New session\n  Ctrl+D   Detach image\n  Ctrl+Q   Quit\n  PgUp/Dn  Scroll\n  Esc      Cancel stream";
                    self.messages.push(ChatMessage::new_text("assistant", help));
                    return;
                }
                "/clear" => {
                    self.messages.clear();
                    self.current_response.clear();
                    return;
                }
                "/models" => {
                    self.state = AppState::SelectingModel;
                    return;
                }
                "/new" => {
                    if let Ok(id) = self.storage.create_session("New Session") {
                        self.active_session_id = id;
                        self.messages.clear();
                        self.current_response.clear();
                        self.attached_image = None;
                        self.follow_bottom = true;
                        self.scroll_offset = 0;
                    }
                    return;
                }
                "/undo" => {
                    if let Some(pos) = self.messages.iter().rposition(|m| m.role == "assistant") {
                        self.messages.remove(pos);
                    }
                    return;
                }
                _ => {
                    self.messages.push(ChatMessage::new_text("assistant", &format!("Unknown command: {}\nType /help for available commands.", text)));
                    return;
                }
            }
        }

        let user_msg = if let Some(attachment) = self.attached_image.take() {
            let msg = ChatMessage::new_multimodal("user", &text, &attachment.data_url);
            let _ = self.save_message("user", &format!("[ATTACHED: {}] {}", attachment.filename, text));
            msg
        } else {
            let _ = self.save_message("user", &text);
            ChatMessage::new_text("user", &text)
        };

        let mut api_messages = self.messages.clone();
        api_messages.push(user_msg.clone());
        self.messages.push(user_msg);

        let text_for_title = if self.attached_image.is_some() {
            text.replace("[image]", "")
        } else {
            text.clone()
        };
        if !text_for_title.is_empty() {
            if let Ok(sessions) = self.storage.list_sessions() {
                if let Some(current) = sessions.iter().find(|s| s.id == self.active_session_id) {
                    if current.title == "New Session" {
                        let first_line = text_for_title.lines().next().unwrap_or(&text_for_title);
                        let title = if first_line.len() > 40 {
                            format!("{}...", &first_line[..40])
                        } else {
                            first_line.to_string()
                        };
                        let _ = self.storage.update_session_title(&self.active_session_id, &title);
                    }
                }
            }
        }

        self.input.clear();
        self.current_response.clear();
        self.follow_bottom = true;
        self.scroll_offset = 0;

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
                app.model_list = models
                    .into_iter()
                    .filter(|m| is_openai_compatible(m))
                    .collect();
                if app.model_list.is_empty() {
                    app.model_list = GO_MODELS_FREE
                        .iter()
                        .map(|&s| s.to_string())
                        .collect();
                }
                app.model_selection_index = app
                    .model_list
                    .iter()
                    .position(|m| *m == app.model)
                    .unwrap_or(0);
                if app.settings.model().is_some() {
                    app.state = AppState::Idle;
                } else {
                    app.state = AppState::SelectingModel;
                }
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

        // Check file dialog result
        if let Some(rx) = &app.file_dialog_rx {
            match rx.try_recv() {
                Ok(Ok(path)) => {
                    app.file_dialog_rx = None;
                    app.start_image_processing(path);
                }
                Ok(Err(_)) => {
                    app.file_dialog_rx = None;
                    app.state = AppState::Idle;
                }
                Err(std_mpsc::TryRecvError::Empty) => {}
                Err(std_mpsc::TryRecvError::Disconnected) => {
                    app.file_dialog_rx = None;
                    app.state = AppState::Idle;
                }
            }
        }

        // Check image processing result
        if let Some(rx) = &mut app.image_process_rx {
            match rx.try_recv() {
                Ok(Ok(attachment)) => {
                    app.attached_image = Some(attachment);
                    app.image_process_rx = None;
                    app.state = AppState::Idle;
                }
                Ok(Err(e)) => {
                    app.image_process_rx = None;
                    app.state = AppState::Error(format!("Image processing failed: {}", e));
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    app.image_process_rx = None;
                    app.state = AppState::Idle;
                }
            }
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

        if !content_buf.is_empty() {
            app.current_response.push_str(&content_buf);
        }

        if stream_done {
            let response = std::mem::take(&mut app.current_response);
            let _ = app.save_message("assistant", &response);
            app.messages
                .push(ChatMessage::new_text("assistant", &response));
            app.follow_bottom = true;
            app.state = AppState::Idle;
            app.stream_rx = None;
            app.stream_handle = None;
        }
    }

    Ok(())
}

async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.state {
        AppState::ApiKeyInput => handle_key_input_state(app, key).await?,
        AppState::SelectingModel => handle_model_selection_state(app, key)?,
        AppState::SessionList => handle_session_list_state(app, key)?,
        AppState::Idle => handle_idle_state(app, key)?,
        AppState::WaitingResponse => handle_waiting_state(app, key)?,
        AppState::Error(_) => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                app.state = AppState::Idle;
            }
        }
        AppState::PickingFile | AppState::ProcessingImage => {
            if key.code == KeyCode::Esc {
                app.file_dialog_rx = None;
                app.image_process_rx = None;
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
                    app.model_list = models
                        .into_iter()
                        .filter(|m| is_openai_compatible(m))
                        .collect();
                    if app.model_list.is_empty() {
                        app.model_list = GO_MODELS_FREE
                            .iter()
                            .map(|&s| s.to_string())
                            .collect();
                    }
                    app.model_selection_index = app
                        .model_list
                        .iter()
                        .position(|m| *m == app.model)
                        .unwrap_or(0);
                    app.key_input_buffer.clear();
                    app.key_error = None;
                    if app.settings.model().is_some() {
                        app.state = AppState::Idle;
                    } else {
                        app.state = AppState::SelectingModel;
                    }
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

fn handle_session_list_state(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if app.session_selection_index > 0 {
                app.session_selection_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.session_selection_index + 1 < app.sessions.len() {
                app.session_selection_index += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(session) = app.sessions.get(app.session_selection_index) {
                let new_id = session.id.clone();
                app.messages.clear();
                app.current_response.clear();
                app.input.clear();
                app.attached_image = None;
                app.follow_bottom = true;
                app.scroll_offset = 0;
                app.active_session_id = new_id.clone();
                let _ = app.load_messages();
                let _ = app.settings.set_active_session_id(&new_id);
            }
            app.state = AppState::Idle;
        }
        KeyCode::Esc => {
            app.state = AppState::Idle;
        }
        _ => {}
    }
    Ok(())
}

fn handle_model_selection_state(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if app.model_selection_index > 0 {
                app.model_selection_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.model_selection_index + 1 < app.model_list.len() {
                app.model_selection_index += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(model) = app.model_list.get(app.model_selection_index) {
                app.model = model.clone();
                let _ = app.settings.set_model(model);
            }
            app.state = AppState::Idle;
        }
        KeyCode::Esc => {
            app.state = AppState::Idle;
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
                        app.attached_image = None;
                        app.follow_bottom = true;
                        app.scroll_offset = 0;
                        return Ok(());
                    }
                    's' | 'S' => {
                        app.sessions = app.storage.list_sessions().unwrap_or_default();
                        app.session_selection_index = app
                            .sessions
                            .iter()
                            .position(|s| s.id == app.active_session_id)
                            .unwrap_or(0);
                        app.state = AppState::SessionList;
                        return Ok(());
                    }
                    'm' | 'M' => {
                        app.state = AppState::SelectingModel;
                        return Ok(());
                    }
                    'p' | 'P' => {
                        app.open_file_dialog();
                        return Ok(());
                    }
                    'd' | 'D' => {
                        app.attached_image = None;
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
            app.follow_bottom = false;
            if app.scroll_offset >= 5 {
                app.scroll_offset -= 5;
            } else {
                app.scroll_offset = 0;
            }
        }
        KeyCode::PageDown => {
            app.scroll_offset += 5;
        }
        KeyCode::Up => {
            app.follow_bottom = false;
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        KeyCode::Down => {
            app.scroll_offset += 1;
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
        KeyCode::PageUp => {
            app.follow_bottom = false;
            if app.scroll_offset >= 5 {
                app.scroll_offset -= 5;
            } else {
                app.scroll_offset = 0;
            }
        }
        KeyCode::PageDown => {
            app.scroll_offset += 5;
        }
        KeyCode::Up => {
            app.follow_bottom = false;
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        KeyCode::Down => {
            app.scroll_offset += 1;
        }
        _ => {}
    }
    Ok(())
}
