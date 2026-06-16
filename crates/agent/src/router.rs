use std::sync::Arc;

use llm::{ChatModel, Message, Role};

use crate::chat_resolver::ChatResolver;
use crate::embed_resolver::EmbedResolver;
use crate::error::{AgentError, Result};
use crate::run::{run_agent, AgentRunContext};
use crate::types::{AgentProfile, AgentResponse, Skill};

pub async fn run_routed(
    ctx: &AgentRunContext<'_>,
    router_chat: Arc<dyn ChatModel>,
    chat_resolver: &dyn ChatResolver,
    embed_resolver: &dyn EmbedResolver,
    profiles: &[AgentProfile],
    skills: &[Skill],
    enabled_skill_ids: &[String],
    question: &str,
) -> Result<AgentResponse> {
    let enabled: Vec<&AgentProfile> = profiles.iter().filter(|p| p.enabled).collect();
    if enabled.is_empty() {
        return Err(AgentError::ProfileNotFound("no enabled agents".into()));
    }

    let agent_id = if enabled.len() == 1 {
        enabled[0].id.clone()
    } else {
        route_agent_id(router_chat.as_ref(), profiles, question).await?
    };

    let profile = profiles
        .iter()
        .find(|p| p.id == agent_id)
        .ok_or_else(|| AgentError::ProfileNotFound(agent_id.clone()))?;

    let agent_chat = chat_resolver.chat_for(profile);
    let embedder = embed_resolver.embed_for(profile);
    let mut resp = run_agent(
        ctx,
        agent_chat.as_ref(),
        embedder.as_ref(),
        profile,
        skills,
        enabled_skill_ids,
        question,
    )
    .await?;

    if resp.orchestration_steps.is_empty() {
        resp.orchestration_steps.push(crate::types::OrchestrationStep {
            agent_id: profile.id.clone(),
            agent_name: format!("{} (路由)", profile.name),
            answer: resp.answer.clone(),
            tool_calls: resp.tool_calls.clone(),
        });
    }

    Ok(resp)
}

async fn route_agent_id(
    chat: &dyn ChatModel,
    profiles: &[AgentProfile],
    question: &str,
) -> Result<String> {
    let agents_list: String = profiles
        .iter()
        .filter(|p| p.enabled)
        .map(|p| {
            let hint = if p.system_prompt.chars().count() > 80 {
                format!("{}…", p.system_prompt.chars().take(80).collect::<String>())
            } else {
                p.system_prompt.clone()
            };
            format!("- id={} name={} hint={hint}", p.id, p.name)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let messages = vec![
        Message {
            role: Role::System,
            content: format!(
                "你是 Agent 路由器。根据用户问题选择最合适的 agent id。仅回复一行 JSON，不要 markdown：{{\"agent_id\":\"...\"}}\n\n可选 Agent：\n{agents_list}"
            ),
        },
        Message {
            role: Role::User,
            content: question.to_string(),
        },
    ];

    let raw = chat.complete(&messages).await?;
    parse_routed_agent_id(&raw, profiles, question)
}

fn parse_routed_agent_id(raw: &str, profiles: &[AgentProfile], question: &str) -> Result<String> {
    let trimmed = raw.trim();
    let json = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(id) = v.get("agent_id").and_then(|x| x.as_str()) {
            if profiles.iter().any(|p| p.id == id && p.enabled) {
                return Ok(id.to_string());
            }
        }
    }

    keyword_route_agent(question, profiles)
}

fn keyword_route_agent(question: &str, profiles: &[AgentProfile]) -> Result<String> {
    let q = question.to_lowercase();
    if (q.contains("任务") || q.contains("待办") || q.contains("todo"))
        && profiles.iter().any(|p| p.id == "tasks" && p.enabled)
    {
        return Ok("tasks".into());
    }

    profiles
        .iter()
        .find(|p| p.enabled && p.id == "default")
        .map(|p| p.id.clone())
        .or_else(|| profiles.iter().find(|p| p.enabled).map(|p| p.id.clone()))
        .ok_or_else(|| AgentError::ProfileNotFound("router could not pick agent".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_router_json() {
        let profiles = crate::types::default_profiles();
        let id = parse_routed_agent_id(r#"{"agent_id":"tasks"}"#, &profiles, "hello").unwrap();
        assert_eq!(id, "tasks");
    }

    #[test]
    fn keyword_routes_tasks() {
        let profiles = crate::types::default_profiles();
        let id = keyword_route_agent("我的待办有哪些", &profiles).unwrap();
        assert_eq!(id, "tasks");
    }
}
