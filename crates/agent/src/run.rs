use chunker::ChunkerConfig;
use embedder::Embedder;
use llm::{ChatModel, Message, Role};
use retriever::RetrieverConfig;
use store::Store;

use crate::error::{AgentError, Result};
use crate::hooks::{run_hooks, HookContext, HookEvent};
use crate::plugins::PluginManifest;
use crate::tools::{build_tools_prompt, execute_tool};
use crate::tool_parse::{CompositeToolCallParser, ToolCallParseOutcome, ToolCallParser};
use crate::types::{AgentProfile, AgentResponse, Skill};

const MAX_TOOL_ROUNDS: usize = 3;

pub struct AgentRunContext<'a> {
    pub store: &'a Store,
    pub chunker: &'a ChunkerConfig,
    pub retriever: &'a RetrieverConfig,
    pub hooks: &'a [crate::hooks::Hook],
    pub enabled_hook_ids: &'a [String],
    pub plugins: &'a [PluginManifest],
    pub enabled_plugin_ids: &'a [String],
    pub granted_plugin_permissions: &'a [String],
}

pub async fn run_agent(
    ctx: &AgentRunContext<'_>,
    chat: &dyn ChatModel,
    embedder: &dyn Embedder,
    profile: &AgentProfile,
    skills: &[Skill],
    enabled_skill_ids: &[String],
    question: &str,
) -> Result<AgentResponse> {
    if !profile.enabled {
        return Err(AgentError::ProfileDisabled(profile.id.clone()));
    }

    let mut skill_block = String::new();
    for skill in skills {
        if enabled_skill_ids.iter().any(|id| id == &skill.id) {
            skill_block.push_str(&format!(
                "\n\n## Skill: {}\n{}\n",
                skill.name, skill.content
            ));
        }
    }

    let system = format!(
        "{}\n\n{}\n{skill_block}",
        profile.system_prompt,
        build_tools_prompt(
            ctx.plugins,
            ctx.enabled_plugin_ids,
            ctx.granted_plugin_permissions,
        )
    );

    let mut messages = vec![
        Message {
            role: Role::System,
            content: system,
        },
        Message {
            role: Role::User,
            content: question.to_string(),
        },
    ];

    let mut tool_calls = Vec::new();
    let mut citations = Vec::new();
    let mut tool_parse_warnings = Vec::new();
    let parser = CompositeToolCallParser::new();

    for _ in 0..MAX_TOOL_ROUNDS {
        let reply = chat.complete(&messages).await?;
        match parser.parse(&reply) {
            ToolCallParseOutcome::Found(payload) => {
                run_hooks(
                    ctx.hooks,
                    ctx.enabled_hook_ids,
                    HookEvent::BeforeToolCall,
                    &HookContext {
                        question: Some(question),
                        tool_name: Some(&payload.name),
                        tool_args: Some(&payload.arguments),
                        tool_result: None,
                        answer: None,
                    },
                );

                let (result, cites) = execute_tool(
                    ctx.store,
                    embedder,
                    ctx.chunker,
                    ctx.retriever,
                    ctx.plugins,
                    ctx.enabled_plugin_ids,
                    ctx.granted_plugin_permissions,
                    &payload.name,
                    &payload.arguments,
                )
                .await?;
                citations.extend(cites);

                run_hooks(
                    ctx.hooks,
                    ctx.enabled_hook_ids,
                    HookEvent::AfterToolCall,
                    &HookContext {
                        question: Some(question),
                        tool_name: Some(&payload.name),
                        tool_args: Some(&payload.arguments),
                        tool_result: Some(&result),
                        answer: None,
                    },
                );

                tool_calls.push(crate::types::ToolCallRecord {
                    name: payload.name.clone(),
                    arguments: payload.arguments.clone(),
                    result: result.clone(),
                });
                messages.push(Message {
                    role: Role::Assistant,
                    content: reply,
                });
                messages.push(Message {
                    role: Role::User,
                    content: format!(
                        "工具 `{name}` 返回：\n{result}\n\n请给出最终中文答案。",
                        name = payload.name
                    ),
                });
            }
            ToolCallParseOutcome::NotFound => {
                run_hooks(
                    ctx.hooks,
                    ctx.enabled_hook_ids,
                    HookEvent::BeforeAnswer,
                    &HookContext {
                        question: Some(question),
                        tool_name: None,
                        tool_args: None,
                        tool_result: None,
                        answer: Some(&reply),
                    },
                );

                return Ok(AgentResponse {
                    answer: reply,
                    citations,
                    tool_calls,
                    orchestration_steps: vec![],
                    tool_parse_warnings,
                });
            }
            ToolCallParseOutcome::Failed { error, raw_snippet } => {
                tool_parse_warnings.push(format!("{error}: {raw_snippet}"));
                return Ok(AgentResponse {
                    answer: reply,
                    citations,
                    tool_calls,
                    orchestration_steps: vec![],
                    tool_parse_warnings,
                });
            }
        }
    }

    let final_reply = chat.complete(&messages).await?;
    run_hooks(
        ctx.hooks,
        ctx.enabled_hook_ids,
        HookEvent::BeforeAnswer,
        &HookContext {
            question: Some(question),
            tool_name: None,
            tool_args: None,
            tool_result: None,
            answer: Some(&final_reply),
        },
    );

    Ok(AgentResponse {
        answer: final_reply,
        citations,
        tool_calls,
        orchestration_steps: vec![],
        tool_parse_warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use llm::MockChatModel;

    #[tokio::test]
    async fn agent_uses_search_tool() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("note.md");
        std::fs::write(&file, "Jarvis agent platform uses tools.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let profile = AgentProfile {
            id: "t".into(),
            name: "T".into(),
            system_prompt: "test".into(),
            enabled: true,
            chat_provider: None,
            embedder_provider: None,
        };

        let retriever = RetrieverConfig::default();
        let chunker = ChunkerConfig::default();
        let chat = MockChatModel;
        let ctx = AgentRunContext {
            store: &store,
            chunker: &chunker,
            retriever: &retriever,
            hooks: &[],
            enabled_hook_ids: &[],
            plugins: &[],
            enabled_plugin_ids: &[],
            granted_plugin_permissions: &[],
        };

        let resp = run_agent(
            &ctx,
            &chat,
            &embedder,
            &profile,
            &[],
            &[],
            "agent platform tools",
        )
            .await
            .unwrap();

        assert!(!resp.answer.is_empty());
        assert!(!resp.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn parse_failed_stops_loop() {
        use async_trait::async_trait;
        use llm::{ChatModel, LlmError, Message};

        struct FenceReplyModel;

        #[async_trait]
        impl ChatModel for FenceReplyModel {
            fn id(&self) -> &str {
                "fence-test"
            }

            async fn complete(&self, _messages: &[Message]) -> std::result::Result<String, LlmError> {
                Ok(r#"```json
{"name":"search_knowledge","arguments":{"query":"x"}}
```"#
                    .into())
            }

            async fn complete_stream(
                &self,
                messages: &[Message],
                on_token: &mut (dyn FnMut(String) + Send),
            ) -> std::result::Result<String, LlmError> {
                let answer = self.complete(messages).await?;
                on_token(answer.clone());
                Ok(answer)
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let profile = AgentProfile {
            id: "t".into(),
            name: "T".into(),
            system_prompt: "test".into(),
            enabled: true,
            chat_provider: None,
            embedder_provider: None,
        };
        let retriever = RetrieverConfig::default();
        let chunker = ChunkerConfig::default();
        let ctx = AgentRunContext {
            store: &store,
            chunker: &chunker,
            retriever: &retriever,
            hooks: &[],
            enabled_hook_ids: &[],
            plugins: &[],
            enabled_plugin_ids: &[],
            granted_plugin_permissions: &[],
        };

        let resp = run_agent(
            &ctx,
            &FenceReplyModel,
            &embedder,
            &profile,
            &[],
            &[],
            "test question",
        )
        .await
        .unwrap();

        assert!(!resp.tool_parse_warnings.is_empty());
        assert!(resp.tool_calls.is_empty());
        assert!(resp.answer.contains("```json"));
    }
}
