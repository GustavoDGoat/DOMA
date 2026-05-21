use crate::storage::StorageEngine;
use anyhow::Result;

const DEFAULT_API_BASE_URL: &str = "https://opencode.ai/zen/go/v1";

pub struct Settings {
    storage: StorageEngine,
}

impl Settings {
    pub fn new(storage: StorageEngine) -> Self {
        Self { storage }
    }

    pub fn api_key(&self) -> Option<String> {
        self.storage.config_get("api_key")
    }

    pub fn set_api_key(&self, key: &str) -> Result<()> {
        self.storage.config_set("api_key", key)?;
        Ok(())
    }

    pub fn api_base_url(&self) -> String {
        std::env::var("DOMA_API_BASE_URL")
            .ok()
            .or_else(|| self.storage.config_get("api_base_url"))
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string())
    }

    pub fn set_api_base_url(&self, url: &str) -> Result<()> {
        self.storage.config_set("api_base_url", url)?;
        Ok(())
    }

    pub fn model(&self) -> Option<String> {
        self.storage.config_get("model")
    }

    pub fn set_model(&self, model: &str) -> Result<()> {
        self.storage.config_set("model", model)?;
        Ok(())
    }

    pub fn active_session_id(&self) -> Option<String> {
        self.storage.config_get("active_session_id")
    }

    pub fn set_active_session_id(&self, id: &str) -> Result<()> {
        self.storage.config_set("active_session_id", id)?;
        Ok(())
    }
}
