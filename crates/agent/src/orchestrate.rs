use std::collections::HashSet;

use crate::error::{AgentError, Result};
use crate::run::{run_agent, AgentRunContext};
use crate::types::{AgentProfile, AgentResponse, OrchestrationStep, Skill};
use rag::Citation;

pub fn resolve_pipeline_agent_ids(profiles: &[AgentProfile], configured: &[String]) -> Vec<String> {
    if !configured.is_empty() {
        return configured
            .iter()
            .filter(|id| profiles.iter().any(|p| p.id == **id && p.enabled))
            .cloned()
            .collect();
    }
    profiles
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.id.clone())
        .collect()
}

pub async fn run_orchestrated(
    ctx: &AgentRunContext<'_>,
    profiles: &[AgentProfile],
    pipeline_ids: &[String],
    skills: &[Skill],
    enabled_skill_ids: &[String],
    question: &str,
) -> Result<AgentResponse> {
    let ids = resolve_pipeline_agent_ids(profiles, pipeline_ids);
    if ids.is_empty() {
        return Err(AgentError::ProfileNotFound(
            "pipeline has no enabled agents".into(),
        ));
    }

    if ids.len() == 1 {
        let profile = profiles
            .iter()
            .find(|p| p.id == ids[0])
            .ok_or_else(|| AgentError::ProfileNotFound(ids[0].clone()))?;
        return run_agent(ctx, profile, skills, enabled_skill_ids, question).await;
    }

    let mut steps = Vec::new();
    let mut accumulated = String::new();
    let mut all_citations = Vec::new();
    let mut all_tool_calls = Vec::new();

    for (i, id) in ids.iter().enumerate() {
        let profile = profiles
            .iter()
            .find(|p| p.id == *id)
            .ok_or_else(|| AgentError::ProfileNotFound(id.clone()))?;

        let is_last = i + 1 == ids.len();
        let prompt = if i == 0 {
            question.to_string()
        } else if is_last {
            format!(
                "用户问题：{question}\n\n以下是其他 Agent 的分析：\n{accumulated}\n\n请综合以上信息，给出最终中文回答。"
            )
        } else {
            format!(
                "用户问题：{question}\n\n前置 Agent 输出：\n{accumulated}\n\n请从你的专业角度补充分析，必要时使用工具检索事实。"
            )
        };

        let resp = run_agent(ctx, profile, skills, enabled_skill_ids, &prompt).await?;
        steps.push(OrchestrationStep {
            agent_id: profile.id.clone(),
            agent_name: profile.name.clone(),
            answer: resp.answer.clone(),
            tool_calls: resp.tool_calls.clone(),
        });
        accumulated.push_str(&format!("\n\n## {}\n{}", profile.name, resp.answer));
        all_citations.extend(resp.citations);
        all_tool_calls.extend(resp.tool_calls);
    }

    let final_answer = steps
        .last()
        .map(|s| s.answer.clone())
        .unwrap_or_default();

    Ok(AgentResponse {
        answer: final_answer,
        citations: dedupe_citations(all_citations),
        tool_calls: all_tool_calls,
        orchestration_steps: steps,
    })
}

fn dedupe_citations(citations: Vec<Citation>) -> Vec<Citation> {
    let mut seen = HashSet::new();
    citations
        .into_iter()
        .filter(|c| seen.insert(c.chunk_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use llm::MockChatModel;
    use retriever::RetrieverConfig;
    use store::Store;

    #[test]
    fn resolve_pipeline_uses_configured_order() {
        let profiles = crate::types::default_profiles();
        let ids = resolve_pipeline_agent_ids(&profiles, &["tasks".into(), "default".into()]);
        assert_eq!(ids, vec!["tasks", "default"]);
    }

    #[tokio::test]
    async fn pipeline_runs_multiple_agents() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("note.md");
        std::fs::write(&file, "Jarvis orchestration pipeline.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let profiles = crate::types::default_profiles();
        let retriever = RetrieverConfig::default();
        let chunker = ChunkerConfig::default();
        let chat = MockChatModel;
        let ctx = AgentRunContext {
            store: &store,
            embedder: &embedder,
            chat: &chat,
            chunker: &chunker,
            retriever: &retriever,
            hooks: &[],
            enabled_hook_ids: &[],
            plugins: &[],
            enabled_plugin_ids: &[],
        };

        let resp = run_orchestrated(
            &ctx,
            &profiles,
            &["default".into(), "tasks".into()],
            &[],
            &[],
            "orchestration pipeline",
        )
        .await
        .unwrap();

        assert_eq!(resp.orchestration_steps.len(), 2);
        assert!(!resp.answer.is_empty());
    }
}
