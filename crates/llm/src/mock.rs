use async_trait::async_trait;

use crate::{ChatModel, LlmError, Message, Result, Role};

/// Deterministic chat model for tests and M3 smoke runs.
pub struct MockChatModel;

fn build_answer(messages: &[Message]) -> Result<String> {
    if messages.is_empty() {
        return Err(LlmError::EmptyMessages);
    }

    let system = messages
        .iter()
        .find(|m| m.role == Role::System)
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let user = messages
        .iter()
        .rev()
        .find(|m| m.role == Role::User)
        .map(|m| m.content.as_str())
        .unwrap_or("");

    if user.contains("NO_CONTEXT") {
        return Ok("知识库中未找到相关内容。".to_string());
    }

    if system.contains("Agent 路由器") {
        let user = messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");
        if user.contains("任务") || user.contains("待办") {
            return Ok(r#"{"agent_id":"tasks"}"#.to_string());
        }
        return Ok(r#"{"agent_id":"default"}"#.to_string());
    }

    if system.contains("可用工具") {
        let has_tool_result = messages
            .iter()
            .any(|m| m.content.contains("工具 `") && m.content.contains("返回"));
        if has_tool_result {
            return Ok("（Mock Agent 回答）已结合工具检索结果作答。".to_string());
        }
        return Ok(r#"{"name":"search_knowledge","arguments":{"query":"agent"}}"#.to_string());
    }

    // Wiki compile (笔记编译) before 记忆/任务/摘要 so wiki prompts do not fall through.
    if system.contains("笔记编译") {
        return Ok(
            r#"{"summary":"该来源讨论相关主题与要点。","entities":[{"name":"Jarvis","blurb":"本地知识库助手"}],"concepts":[]}"#.to_string(),
        );
    }

    if system.contains("记忆提取") || (system.contains("JSON 数组") && user.contains("问题：")) {
        return Ok(r#"[{"content":"用户关注 Jarvis 知识库相关主题"}]"#.to_string());
    }

    if system.contains("任务提取") {
        return Ok(r#"[{"title":"整理要点","description":"根据来源整理可执行待办"}]"#.to_string());
    }

    if system.contains("摘要") {
        return Ok("（Mock 摘要）该来源主要讨论相关主题与待办事项。".to_string());
    }

    let cited = system.matches('[').count();

    Ok(format!(
        "（Mock 回答）已基于 {cited} 条引用片段回答：{}",
        user.lines().next().unwrap_or(user)
    ))
}

#[async_trait]
impl ChatModel for MockChatModel {
    fn id(&self) -> &str {
        "mock-chat:v1"
    }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        build_answer(messages)
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String> {
        let answer = build_answer(messages)?;
        for chunk in answer.split_inclusive(' ') {
            on_token(chunk.to_string());
        }
        Ok(answer)
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

    #[tokio::test]
    async fn mock_agent_tool_emits_json_line() {
        let model = MockChatModel;
        let out = model
            .complete(&[
                Message {
                    role: Role::System,
                    content: "可用工具".into(),
                },
                Message {
                    role: Role::User,
                    content: "test".into(),
                },
            ])
            .await
            .unwrap();
        assert!(out.contains(r#"{"name""#));
        assert!(!out.contains("tool_call"));
    }

    #[tokio::test]
    async fn mock_stream_emits_tokens() {
        let model = MockChatModel;
        let mut tokens = String::new();
        let out = model
            .complete_stream(
                &[Message {
                    role: Role::User,
                    content: "hello".into(),
                }],
                &mut |t| tokens.push_str(&t),
            )
            .await
            .unwrap();
        assert!(!tokens.is_empty());
        assert_eq!(tokens, out);
    }

    #[tokio::test]
    async fn mock_wiki_compile_json() {
        let model = MockChatModel;
        let out = model
            .complete(&[
                Message {
                    role: Role::System,
                    content: "你是笔记编译助手。仅输出裸 JSON。".into(),
                },
                Message {
                    role: Role::User,
                    content: "标题：示例\n\n正文：\n相关内容".into(),
                },
            ])
            .await
            .unwrap();

        assert!(out.contains("summary"), "reply must include summary key: {out}");
        assert!(out.contains("entities"), "reply must include entities key: {out}");

        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let summary = value["summary"].as_str().unwrap_or("");
        assert!(!summary.is_empty(), "summary must be non-empty");
        let entities = value["entities"].as_array().expect("entities array");
        assert_eq!(entities.len(), 1, "exactly one entity");
        let concepts = value["concepts"].as_array().expect("concepts array");
        assert!(concepts.is_empty(), "concepts must be empty array");
    }
}
