use super::types::*;
use anyhow::Result;
use futures_util::StreamExt;
use std::time::Duration;

fn parse_stream_line(json_str: &str) -> Vec<Result<StreamChunk, ClientError>> {
    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
        return vec![Ok(chunk)];
    }

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(error_obj) = val.get("error") {
            let msg = error_obj
                .get("message")
                .and_then(|v| v.as_str())
                .or_else(|| error_obj.as_str())
                .unwrap_or("Unknown API error");
            return vec![Err(ClientError::Stream(msg.to_string()))];
        }
    }

    if json_str.trim().is_empty() {
        return vec![];
    }

    vec![Err(ClientError::Stream(format!(
        "Unexpected response: {}",
        &json_str[..json_str.len().min(200)]
    )))]
}

pub struct OpenCodeClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenCodeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
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

        let byte_stream = response.bytes_stream().boxed();
        let initial_state = (String::new(), byte_stream);

        let stream = futures_util::stream::unfold(
            initial_state,
            |(mut buffer, mut byte_stream)| async move {
                loop {
                    if let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer.drain(..=pos);
                        buffer = buffer.trim_start().to_string();

                        let trimmed = line.trim().to_string();

                        if trimmed.is_empty() || trimmed == "data: [DONE]" {
                            continue;
                        }

                        let json_str = trimmed.strip_prefix("data: ").unwrap_or(&trimmed);

                        let items = parse_stream_line(json_str);

                        if items.is_empty() {
                            continue;
                        }

                        return Some((items, (buffer, byte_stream)));
                    }

                    match byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            let text = String::from_utf8_lossy(&bytes);
                            buffer.push_str(&text);
                        }
                        Some(Err(e)) => {
                            return Some((
                                vec![Err(ClientError::Network(e))],
                                (buffer, byte_stream),
                            ));
                        }
                        None => {
                            if !buffer.is_empty() {
                                let trimmed = buffer.trim().to_string();
                                if trimmed.is_empty() || trimmed == "data: [DONE]" {
                                    return None;
                                }
                                let json_str =
                                    trimmed.strip_prefix("data: ").unwrap_or(&trimmed);
                                let items = parse_stream_line(json_str);
                                return Some((items, (String::new(), byte_stream)));
                            }
                            return None;
                        }
                    }
                }
            },
        )
        .flat_map(futures_util::stream::iter)
        .boxed();

        Ok(stream)
    }
}
