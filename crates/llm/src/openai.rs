use async_trait::async_trait;
use futures_util::StreamExt;

use crate::{ChatModel, LlmError, Message, Result, Role};

pub struct OpenAiChat {
    base_url: String,
    api_key: String,
    model: String,
    id: String,
    client: reqwest::Client,
}

impl OpenAiChat {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let id = format!("cloud:chat:{model}");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            id,
            client: reqwest::Client::new(),
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
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

#[derive(Debug, serde::Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, serde::Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, serde::Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[async_trait]
impl ChatModel for OpenAiChat {
    fn id(&self) -> &str {
        &self.id
    }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        if messages.is_empty() {
            return Err(LlmError::EmptyMessages);
        }
        if self.api_key.is_empty() {
            return Err(LlmError::Http("cloud api key is empty".into()));
        }

        let resp = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": Self::payload_messages(messages),
                "stream": false,
            }))
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Http(format!(
                "cloud chat status {status}: {body}"
            )));
        }

        let body: ChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::BadResponse(e.to_string()))?;

        body.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| LlmError::BadResponse("missing chat choice".into()))
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String> {
        if messages.is_empty() {
            return Err(LlmError::EmptyMessages);
        }
        if self.api_key.is_empty() {
            return Err(LlmError::Http("cloud api key is empty".into()));
        }

        let resp = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": Self::payload_messages(messages),
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Http(format!(
                "cloud chat status {status}: {body}"
            )));
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
                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }
                let payload = line
                    .strip_prefix("data: ")
                    .ok_or_else(|| LlmError::BadResponse(format!("invalid sse line: {line}")))?;
                let parsed: StreamChunk = serde_json::from_str(payload)
                    .map_err(|e| LlmError::BadResponse(e.to_string()))?;
                if let Some(choice) = parsed.choices.into_iter().next() {
                    if let Some(token) = choice.delta.content.filter(|t| !t.is_empty()) {
                        answer.push_str(&token);
                        on_token(token);
                    }
                }
            }
        }

        Ok(answer)
    }
}
