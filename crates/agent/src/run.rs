use embedder::Embedder;
use llm::{ChatModel, Message, Role};
use retriever::RetrieverConfig;
use store::Store;

use crate::error::{AgentError, Result};
use crate::tools::{execute_tool, parse_tool_call, tools_prompt};
use crate::types::{AgentProfile, AgentResponse, Skill};

const MAX_TOOL_ROUNDS: usize = 3;

pub async fn run_agent(
    store: &Store,
    embedder: &dyn Embedder,
    chat: &dyn ChatModel,
    retriever: &RetrieverConfig,
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
        tools_prompt()
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

    for _ in 0..MAX_TOOL_ROUNDS {
        let reply = chat.complete(&messages).await?;
        if let Some(payload) = parse_tool_call(&reply) {
            let (result, cites) = execute_tool(
                store,
                embedder,
                retriever,
                &payload.name,
                &payload.arguments,
            )
            .await?;
            citations.extend(cites);
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
                content: format!("工具 `{name}` 返回：\n{result}\n\n请给出最终中文答案。", name = payload.name),
            });
            continue;
        }

        return Ok(AgentResponse {
            answer: reply,
            citations,
            tool_calls,
        });
    }

    let final_reply = chat.complete(&messages).await?;
    Ok(AgentResponse {
        answer: final_reply,
        citations,
        tool_calls,
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
        };

        let resp = run_agent(
            &store,
            &embedder,
            &MockChatModel,
            &RetrieverConfig::default(),
            &profile,
            &[],
            &[],
            "agent platform tools",
        )
        .await
        .unwrap();

        assert!(!resp.answer.is_empty());
    }
}
