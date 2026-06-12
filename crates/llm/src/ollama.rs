use async_trait::async_trait;
use futures_util::StreamExt;
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

    fn payload_messages(messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                serde_json::json!({ "role": role, "content": m.content })
            })
            .collect()
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

#[derive(Debug, Deserialize)]
struct StreamChunk {
    message: Option<StreamMessage>,
    done: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    content: Option<String>,
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

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": Self::payload_messages(messages),
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

    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String> {
        if messages.is_empty() {
            return Err(LlmError::EmptyMessages);
        }

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": Self::payload_messages(messages),
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(LlmError::Http(format!("ollama chat status {}", resp.status())));
        }

        let mut answer = String::new();
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LlmError::Http(e.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();
                if line.is_empty() {
                    continue;
                }
                let parsed: StreamChunk = serde_json::from_str(&line)
                    .map_err(|e| LlmError::BadResponse(e.to_string()))?;
                if let Some(msg) = parsed.message {
                    if let Some(token) = msg.content.filter(|t| !t.is_empty()) {
                        answer.push_str(&token);
                        on_token(token);
                    }
                }
            }
        }

        Ok(answer)
    }
}
