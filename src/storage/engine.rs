use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct StorageEngine {
    #[allow(dead_code)]
    db: sled::Db,
    config: sled::Tree,
    sessions: sled::Tree,
    messages: sled::Tree,
}

impl StorageEngine {
    pub fn new() -> Result<Self> {
        let data_dir = Self::data_dir()?;
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("db");
        let db = sled::open(&db_path)?;

        let config = db.open_tree("config")?;
        let sessions = db.open_tree("sessions")?;
        let messages = db.open_tree("messages")?;

        Ok(Self {
            db,
            config,
            sessions,
            messages,
        })
    }

    fn data_dir() -> Result<PathBuf> {
        let base = dirs::data_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        Ok(base.join("doma"))
    }

    // ── Config tree ──

    pub fn config_get(&self, key: &str) -> Option<String> {
        self.config
            .get(key.as_bytes())
            .ok()
            .flatten()
            .map(|v| String::from_utf8_lossy(&v).to_string())
    }

    pub fn config_set(&self, key: &str, value: &str) -> Result<()> {
        self.config.insert(key.as_bytes(), value.as_bytes())?;
        self.config.flush()?;
        Ok(())
    }

    pub fn config_delete(&self, key: &str) -> Result<()> {
        self.config.remove(key.as_bytes())?;
        self.config.flush()?;
        Ok(())
    }

    // ── Sessions tree ──

    pub fn list_sessions(&self) -> Result<Vec<SessionMeta>> {
        let mut sessions = Vec::new();
        for entry in self.sessions.iter() {
            let (_, value) = entry?;
            let meta: SessionMeta = serde_json::from_slice(&value)?;
            sessions.push(meta);
        }
        sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at));
        Ok(sessions)
    }

    pub fn create_session(&self, title: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let meta = SessionMeta {
            id: id.clone(),
            title: title.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        let value = serde_json::to_vec(&meta)?;
        self.sessions.insert(id.as_bytes(), value)?;
        self.sessions.flush()?;
        Ok(id)
    }

    pub fn update_session_title(&self, id: &str, title: &str) -> Result<()> {
        let key = id.as_bytes();
        if let Some(existing) = self.sessions.get(key)? {
            let mut meta: SessionMeta = serde_json::from_slice(&existing)?;
            meta.title = title.to_string();
            let value = serde_json::to_vec(&meta)?;
            self.sessions.insert(key, value)?;
            self.sessions.flush()?;
        }
        Ok(())
    }

    pub fn delete_session(&self, id: &str) -> Result<()> {
        self.sessions.remove(id.as_bytes())?;
        let prefix = format!("{}_", id);
        let to_remove: Vec<Vec<u8>> = self
            .messages
            .scan_prefix(prefix.as_bytes())
            .filter_map(|r| r.ok().map(|(k, _)| k.to_vec()))
            .collect();
        for key in to_remove {
            self.messages.remove(key)?;
        }
        self.sessions.flush()?;
        self.messages.flush()?;
        Ok(())
    }

    // ── Messages tree ──

    pub fn list_messages(&self, session_id: &str) -> Result<Vec<StoredMessage>> {
        let prefix = format!("{}_", session_id);
        let mut msgs = Vec::new();
        for entry in self.messages.scan_prefix(prefix.as_bytes()) {
            let (_, value) = entry?;
            let msg: StoredMessage = serde_json::from_slice(&value)?;
            msgs.push(msg);
        }
        Ok(msgs)
    }

    pub fn append_message(&self, session_id: &str, msg: &StoredMessage) -> Result<()> {
        let key = format!("{}_{}", session_id, msg.timestamp);
        let value = serde_json::to_vec(msg)?;
        self.messages.insert(key.as_bytes(), value)?;
        self.messages.flush()?;
        Ok(())
    }
}
