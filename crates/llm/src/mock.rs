use async_trait::async_trait;

use crate::{ChatModel, LlmError, Message, Result, Role};

/// Deterministic chat model for tests and M3 smoke runs.
pub struct MockChatModel;

#[async_trait]
impl ChatModel for MockChatModel {
    fn id(&self) -> &str {
        "mock-chat:v1"
    }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        if messages.is_empty() {
            return Err(LlmError::EmptyMessages);
        }

        let user = messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");

        if user.contains("NO_CONTEXT") {
            return Ok("知识库中未找到相关内容。".to_string());
        }

        let cited = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.matches('[').count())
            .unwrap_or(0);

        Ok(format!(
            "（Mock 回答）已基于 {cited} 条引用片段回答：{}",
            user.lines().next().unwrap_or(user)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_returns_no_context_message() {
        let model = MockChatModel;
        let out = model
            .complete(&[Message {
                role: Role::User,
                content: "NO_CONTEXT".into(),
            }])
            .await
            .unwrap();
        assert!(out.contains("未找到"));
    }
}
