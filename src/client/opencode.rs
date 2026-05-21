use super::types::*;
use anyhow::Result;
use futures_util::StreamExt;
use std::time::Duration;

pub struct OpenCodeClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenCodeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("reqwest client should build");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn validate_key(&self) -> Result<bool> {
        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(true),
            401 => Ok(false),
            status => Err(anyhow::anyhow!("Unexpected status: {}", status)),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        let models: ModelList = resp.json().await?;
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }

    pub async fn chat_completions(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<StreamChunk, ClientError>>> {
        let url = format!("{}/chat/completions", self.base_url);

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error {}: {}", status, body));
        }

        let stream = response
            .bytes_stream()
            .flat_map(|result| {
                let items: Vec<Result<StreamChunk, ClientError>> = match result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        let mut parsed = Vec::new();
                        for line in text.lines() {
                            let line = line.strip_prefix("data: ").unwrap_or(line);
                            if line == "[DONE]" || line.is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<StreamChunk>(line) {
                                Ok(chunk) => parsed.push(Ok(chunk)),
                                Err(e) => parsed.push(Err(ClientError::Stream(
                                    format!("Parse error: {}", e),
                                ))),
                            }
                        }
                        parsed
                    }
                    Err(e) => vec![Err(ClientError::Network(e))],
                };
                futures_util::stream::iter(items)
            })
            .boxed();

        Ok(stream)
    }
}
