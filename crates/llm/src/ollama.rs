use async_trait::async_trait;
use serde::Deserialize;

use crate::{ChatModel, LlmError, Message, Result, Role};

pub struct OllamaChat {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaChat {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

#[async_trait]
impl ChatModel for OllamaChat {
    fn id(&self) -> &str {
        "ollama:chat"
    }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        if messages.is_empty() {
            return Err(LlmError::EmptyMessages);
        }

        let payload_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                serde_json::json!({ "role": role, "content": m.content })
            })
            .collect();

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": payload_messages,
                "stream": false,
            }))
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(LlmError::Http(format!("ollama chat status {}", resp.status())));
        }

        let body: ChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::BadResponse(e.to_string()))?;

        Ok(body.message.content)
    }
}
