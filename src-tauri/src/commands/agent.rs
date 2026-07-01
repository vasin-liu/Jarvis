use std::sync::Arc;

use agent::{
    load_hooks_from_dir, load_plugins_from_dir, load_skills_from_dir, run_agent, run_orchestrated,
    run_routed, AgentProfile, AgentResponse, AgentRunContext, ChatResolver, EmbedResolver, Hook,
    PluginManifest, Skill,
};
use config::{
    build_chat_model, build_chat_model_for_profile, build_embedder_with_provider,
    parse_embedder_provider, AgentOrchestrationMode, AppConfig,
};
use embedder::Embedder;
use llm::ChatModel;
use rag::{AskResponse, ToolCallInfo};
use store::ChatRole;
use tauri::ipc::Channel;
use tauri::State;

use crate::commands::chat::{persist_assistant, TokenEvent};
use crate::state::AppState;

pub(crate) async fn run_ask_agent_in_session(
    state: &AppState,
    session_id: &str,
    question: &str,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(session_id, ChatRole::User, question, None)
        .map_err(|e| e.to_string())?;

    let cfg = state.config();
    let agent_resp = execute_agent_question(state, &cfg, question).await?;

    let resp = agent_response_to_ask(agent_resp);
    persist_assistant(state, session_id, question, &resp).await?;
    Ok(resp)
}

fn resolve_active_agent(cfg: &AppConfig) -> Result<AgentProfile, String> {
    cfg.agent_cfg
        .agents
        .iter()
        .find(|a| a.id == cfg.agent_cfg.active_agent_id)
        .cloned()
        .ok_or_else(|| format!("agent not found: {}", cfg.agent_cfg.active_agent_id))
}

fn build_agent_context<'a>(
    state: &'a AppState,
    cfg: &'a AppConfig,
    hooks: &'a [Hook],
    plugins: &'a [PluginManifest],
) -> AgentRunContext<'a> {
    AgentRunContext {
        store: state.store.as_ref(),
        chunker: &state.chunker,
        retriever: &state.retriever,
        hooks,
        enabled_hook_ids: &cfg.agent_cfg.enabled_hook_ids,
        plugins,
        enabled_plugin_ids: &cfg.agent_cfg.enabled_plugin_ids,
        granted_plugin_permissions: &cfg.agent_cfg.granted_plugin_permissions,
    }
}

#[tauri::command]
pub fn list_agent_profiles(state: State<'_, AppState>) -> Result<Vec<AgentProfile>, String> {
    Ok(state.config().agent_cfg.agents)
}

#[tauri::command]
pub fn list_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, String> {
    Ok(load_skills_from_dir(&state.skills_dir))
}

#[tauri::command]
pub fn list_hooks(state: State<'_, AppState>) -> Result<Vec<Hook>, String> {
    Ok(load_hooks_from_dir(&state.hooks_dir))
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginManifest>, String> {
    Ok(load_plugins_from_dir(&state.plugins_dir))
}

#[tauri::command]
pub fn set_active_agent(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config();
    if !cfg.agent_cfg.agents.iter().any(|a| a.id == id && a.enabled) {
        return Err(format!("agent not found or disabled: {id}"));
    }
    cfg.agent_cfg.active_agent_id = id;
    state.save_config(&cfg)
}

#[tauri::command]
pub fn remove_agent_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    if id == "default" {
        return Err("cannot remove built-in default agent".into());
    }
    let mut cfg = state.config();
    cfg.agent_cfg.agents.retain(|a| a.id != id);
    if cfg.agent_cfg.agents.is_empty() {
        return Err("must keep at least one agent profile".into());
    }
    if cfg.agent_cfg.active_agent_id == id {
        cfg.agent_cfg.active_agent_id = "default".to_string();
    }
    cfg.agent_cfg.pipeline_agent_ids.retain(|x| x != &id);
    state.save_config(&cfg)
}

#[tauri::command]
pub fn upsert_agent_profile(
    profile: AgentProfile,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if profile.id.trim().is_empty() || profile.name.trim().is_empty() {
        return Err("agent id and name are required".into());
    }
    let mut cfg = state.config();
    if let Some(idx) = cfg.agent_cfg.agents.iter().position(|a| a.id == profile.id) {
        cfg.agent_cfg.agents[idx] = profile;
    } else {
        cfg.agent_cfg.agents.push(profile);
    }
    state.save_config(&cfg)
}

#[tauri::command]
pub async fn ask_agent_in_session(
    session_id: String,
    question: String,
    state: State<'_, AppState>,
) -> Result<AskResponse, String> {
    run_ask_agent_in_session(state.inner(), &session_id, &question).await
}

#[tauri::command]
pub async fn ask_agent_in_session_stream(
    session_id: String,
    question: String,
    on_token: Channel<TokenEvent>,
    state: State<'_, AppState>,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(&session_id, ChatRole::User, &question, None)
        .map_err(|e| e.to_string())?;

    let cfg = state.config();
    let agent_resp = execute_agent_question(state.inner(), &cfg, &question).await?;

    for chunk in agent_resp.answer.split_inclusive(' ') {
        let _ = on_token.send(TokenEvent {
            token: chunk.to_string(),
        });
    }

    let resp = agent_response_to_ask(agent_resp);
    persist_assistant(state.inner(), &session_id, &question, &resp).await?;
    Ok(resp)
}

fn agent_response_to_ask(resp: AgentResponse) -> AskResponse {
    AskResponse {
        answer: resp.answer,
        citations: resp.citations,
        tool_calls: resp
            .tool_calls
            .into_iter()
            .map(|t| ToolCallInfo {
                name: t.name,
                arguments: t.arguments,
                result: t.result,
            })
            .collect(),
        orchestration_steps: resp
            .orchestration_steps
            .into_iter()
            .map(|s| rag::OrchestrationStepInfo {
                agent_id: s.agent_id,
                agent_name: s.agent_name,
                answer: s.answer,
                tool_calls: s
                    .tool_calls
                    .into_iter()
                    .map(|t| ToolCallInfo {
                        name: t.name,
                        arguments: t.arguments,
                        result: t.result,
                    })
                    .collect(),
            })
            .collect(),
        tool_parse_warnings: resp.tool_parse_warnings,
    }
}

struct ConfigChatResolver {
    cfg: AppConfig,
}

impl ChatResolver for ConfigChatResolver {
    fn chat_for(&self, profile: &AgentProfile) -> Arc<dyn ChatModel> {
        build_chat_model_for_profile(&self.cfg, profile)
    }
}

struct ConfigEmbedResolver {
    cfg: AppConfig,
    default_embedder: Arc<dyn Embedder>,
}

impl EmbedResolver for ConfigEmbedResolver {
    fn embed_for(&self, profile: &AgentProfile) -> Arc<dyn Embedder> {
        let provider = profile
            .embedder_provider
            .as_deref()
            .and_then(parse_embedder_provider)
            .unwrap_or(self.cfg.embedding.embedder);
        if provider == self.cfg.embedding.embedder {
            return self.default_embedder.clone();
        }
        build_embedder_with_provider(&self.cfg, provider)
            .unwrap_or_else(|_| self.default_embedder.clone())
    }
}

async fn execute_agent_question(
    state: &AppState,
    cfg: &AppConfig,
    question: &str,
) -> Result<AgentResponse, String> {
    let skills = load_skills_from_dir(&state.skills_dir);
    let hooks = load_hooks_from_dir(&state.hooks_dir);
    let plugins = load_plugins_from_dir(&state.plugins_dir);
    let default_embedder = state.embedder();
    let ctx = build_agent_context(state, cfg, &hooks, &plugins);

    let chat_resolver = ConfigChatResolver { cfg: cfg.clone() };
    let embed_resolver = ConfigEmbedResolver {
        cfg: cfg.clone(),
        default_embedder: default_embedder.clone(),
    };

    match cfg.agent_cfg.agent_orchestration_mode {
        AgentOrchestrationMode::Single => {
            let profile = resolve_active_agent(cfg)?;
            let chat = chat_resolver.chat_for(&profile);
            let embedder = embed_resolver.embed_for(&profile);
            run_agent(
                &ctx,
                chat.as_ref(),
                embedder.as_ref(),
                &profile,
                &skills,
                &cfg.agent_cfg.enabled_skill_ids,
                question,
            )
            .await
            .map_err(|e| e.to_string())
        }
        AgentOrchestrationMode::Pipeline => run_orchestrated(
            &ctx,
            &chat_resolver,
            &embed_resolver,
            &cfg.agent_cfg.agents,
            &cfg.agent_cfg.pipeline_agent_ids,
            &skills,
            &cfg.agent_cfg.enabled_skill_ids,
            question,
        )
        .await
        .map_err(|e| e.to_string()),
        AgentOrchestrationMode::Router => {
            let router_chat = build_chat_model(cfg);
            run_routed(
                &ctx,
                router_chat,
                &chat_resolver,
                &embed_resolver,
                &cfg.agent_cfg.agents,
                &skills,
                &cfg.agent_cfg.enabled_skill_ids,
                question,
            )
            .await
            .map_err(|e| e.to_string())
        }
    }
}
