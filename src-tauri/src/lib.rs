mod e2e;
mod index_ops;
mod insights_ops;
mod sync_scheduler;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chunker::ChunkerConfig;
use config::{
    build_chat_model, build_chat_model_for_profile, build_embedder, load_config, save_config,
    AppConfig,
};
use embedder::Embedder;
use e2e::{apply_e2e_config, e2e_data_dir, is_e2e_mode, seed_e2e_fixture};
use cursor::list_transcript_summaries;
use index_ops::{
    index_local_paths, index_status_view, rebuild_all_sources, retry_source_by_id,
    sync_cursor_transcripts, IndexProgressEvent, IndexStatusView, RebuildReport,
};
use indexer::{index_document, index_path};
use insights::{extract_tasks_from_source, summarize_source};
use insights_ops::{maybe_run_insights_for_source, run_insights_for_all_indexed, InsightsReport};
use lark::{check_auth, fetch_doc, fetch_im_chat, fetch_mail, fetch_sheet, ProcessRunner};
use llm::ChatModel;
use agent::{
    load_hooks_from_dir, load_plugins_from_dir, load_skills_from_dir, run_agent,
    run_orchestrated, run_routed, AgentProfile, AgentResponse, AgentRunContext, ChatResolver,
    Hook, PluginManifest, Skill,
};
use config::AgentOrchestrationMode;
use memory::{add_memory, learn_from_exchange, list_memories};
use rag::{ask, ask_stream, AskResponse};
use retriever::RetrieverConfig;
use serde::Serialize;
use sync_scheduler::{run_scheduled_sync, spawn_scheduler, sync_status_view, SchedulerHandle, SyncStatusView};
use store::{ChatMessage, ChatRole, ChatSession, Source, SourceKind, Store, Task, TaskStatus};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};
use watcher::{scan_folder, spawn_watcher, unindex_path, WatchEvent, WatchHandle};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenEvent {
    token: String,
}

struct AppState {
    store: Arc<Store>,
    embedder: Mutex<Arc<dyn Embedder>>,
    chat: Mutex<Arc<dyn ChatModel>>,
    chunker: ChunkerConfig,
    retriever: RetrieverConfig,
    config_path: PathBuf,
    db_path: PathBuf,
    skills_dir: PathBuf,
    hooks_dir: PathBuf,
    plugins_dir: PathBuf,
    config: Mutex<AppConfig>,
    watch: Mutex<Option<WatchHandle>>,
    scheduler: Mutex<Option<SchedulerHandle>>,
}

impl AppState {
    fn config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    fn embedder(&self) -> Arc<dyn Embedder> {
        self.embedder.lock().unwrap().clone()
    }

    fn chat(&self) -> Arc<dyn ChatModel> {
        self.chat.lock().unwrap().clone()
    }

    fn reload_providers(&self, cfg: &AppConfig) -> Result<(), String> {
        *self.embedder.lock().unwrap() = build_embedder(cfg).map_err(|e| e.to_string())?;
        *self.chat.lock().unwrap() = build_chat_model(cfg);
        Ok(())
    }

    fn save_config(&self, cfg: &AppConfig) -> Result<(), String> {
        save_config(&self.config_path, cfg).map_err(|e| e.to_string())?;
        *self.config.lock().unwrap() = cfg.clone();
        self.reload_providers(cfg)
    }

    fn restart_watcher(&self) -> Result<(), String> {
        if let Some(handle) = self.watch.lock().unwrap().take() {
            handle.stop();
        }

        let folders: Vec<PathBuf> = self
            .config()
            .watch_folders
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect();

        if folders.is_empty() {
            return Ok(());
        }

        let (rx, handle) = spawn_watcher(folders, 400).map_err(|e| e.to_string())?;
        *self.watch.lock().unwrap() = Some(handle);

        let store = self.store.clone();
        let embedder = self.embedder();
        let chunker = self.chunker.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            while let Ok(ev) = rx.recv() {
                match ev {
                    WatchEvent::Modified(path) => {
                        let _ = rt.block_on(index_path(
                            store.as_ref(),
                            embedder.as_ref(),
                            &chunker,
                            &path,
                        ));
                    }
                    WatchEvent::Removed(path) => {
                        let _ = unindex_path(store.as_ref(), &path);
                    }
                }
            }
        });

        Ok(())
    }

    fn restart_scheduler(&self, app: &AppHandle) {
        if let Some(handle) = self.scheduler.lock().unwrap().take() {
            handle.stop();
        }
        if !is_e2e_mode() {
            *self.scheduler.lock().unwrap() = Some(spawn_scheduler(app.clone()));
        }
    }

    fn initial_scan_with_progress(&self, app: &AppHandle) -> Result<(), String> {
        let cfg = self.config();
        let mut all_paths = Vec::new();
        for folder in &cfg.watch_folders {
            let paths = scan_folder(folder).map_err(|e| e.to_string())?;
            all_paths.extend(paths);
        }
        if all_paths.is_empty() {
            return Ok(());
        }

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let cfg = self.config();
        let chat = self.chat();
        let report = rt.block_on(index_local_paths(
            self.store.as_ref(),
            self.embedder().as_ref(),
            &self.chunker,
            "scan",
            all_paths,
            Some((chat.as_ref(), &cfg)),
            |event| emit_index_progress(app, event),
        ))?;
        emit_index_complete(app, &report);
        Ok(())
    }
}

#[tauri::command]
fn source_count(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_sources(state: tauri::State<'_, AppState>) -> Result<Vec<Source>, String> {
    state.store.list_sources().map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_source(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state
        .store
        .delete_chunks_for_source(&id)
        .map_err(|e| e.to_string())?;
    state.store.delete_source(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config())
}

#[tauri::command]
fn get_index_status(state: tauri::State<'_, AppState>) -> Result<IndexStatusView, String> {
    let cfg = state.config();
    Ok(index_status_view(
        state.store.as_ref(),
        &cfg,
        state.embedder().as_ref(),
    ))
}

#[tauri::command]
fn set_config(config: AppConfig, app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.save_config(&config)?;
    state.restart_watcher()?;
    state.restart_scheduler(&app);
    Ok(())
}

pub(crate) fn emit_index_progress(app: &AppHandle, event: IndexProgressEvent) {
    let _ = app.emit("index-progress", event);
}

pub(crate) fn emit_index_complete(app: &AppHandle, report: &RebuildReport) {
    let _ = app.emit("index-complete", report);
}

#[tauri::command]
async fn rebuild_index(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let report = rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &cfg.lark_cli_bin,
        &cfg.cursor_projects_root,
        "rebuild",
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
async fn reinit_and_rebuild_index(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let new_dim = cfg.embedding_dim();
    state.reload_providers(&cfg)?;
    state
        .store
        .reinit_vectors(new_dim)
        .map_err(|e| e.to_string())?;
    let report = rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &cfg.lark_cli_bin,
        &cfg.cursor_projects_root,
        "reinit",
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
async fn retry_source(
    id: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let report = retry_source_by_id(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &cfg.lark_cli_bin,
        &cfg.cursor_projects_root,
        &id,
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
fn list_cursor_transcripts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<cursor::TranscriptSummary>, String> {
    let cfg = state.config();
    if cfg.cursor_projects_root.is_empty() {
        return Ok(Vec::new());
    }
    list_transcript_summaries(std::path::Path::new(&cfg.cursor_projects_root))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_cursor_transcripts_cmd(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let chat = state.chat();
    let report = sync_cursor_transcripts(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &cfg.cursor_projects_root,
        Some((chat.as_ref(), &cfg)),
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
async fn add_watch_folder(
    path: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let folder = PathBuf::from(&path);
    if !folder.is_dir() {
        return Err(format!("not a directory: {path}"));
    }

    let mut cfg = state.config();
    if !cfg.watch_folders.iter().any(|f| f == &path) {
        cfg.watch_folders.push(path);
    }
    state.save_config(&cfg)?;

    let paths = scan_folder(&folder).map_err(|e| e.to_string())?;
    if !paths.is_empty() {
        let cfg = state.config();
        let chat = state.chat();
        let report = index_local_paths(
            state.store.as_ref(),
            state.embedder().as_ref(),
            &state.chunker,
            "scan",
            paths,
            Some((chat.as_ref(), &cfg)),
            |event| emit_index_progress(&app, event),
        )
        .await?;
        emit_index_complete(&app, &report);
    }

    state.restart_watcher()
}

#[tauri::command]
fn remove_watch_folder(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config();
    cfg.watch_folders.retain(|f| f != &path);
    state.save_config(&cfg)?;
    state.restart_watcher()
}

#[tauri::command]
fn list_chat_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    state.store.list_chat_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_chat_session(
    title: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<ChatSession, String> {
    let title = title.unwrap_or_else(|| "新对话".to_string());
    state
        .store
        .create_chat_session(&title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_chat_session(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.store.delete_chat_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_chat_messages(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    state
        .store
        .list_chat_messages(&session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ask_in_session(
    session_id: String,
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(&session_id, ChatRole::User, &question, None)
        .map_err(|e| e.to_string())?;

    let resp = ask(
        state.store.as_ref(),
        state.embedder().as_ref(),
        state.chat().as_ref(),
        &state.retriever,
        &question,
    )
    .await
    .map_err(|e| e.to_string())?;

    persist_assistant(&state, &session_id, &question, &resp).await?;
    Ok(resp)
}

#[tauri::command]
async fn ask_in_session_stream(
    session_id: String,
    question: String,
    on_token: Channel<TokenEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(&session_id, ChatRole::User, &question, None)
        .map_err(|e| e.to_string())?;

    let resp = ask_stream(
        state.store.as_ref(),
        state.embedder().as_ref(),
        state.chat().as_ref(),
        &state.retriever,
        &question,
        &mut |token| {
            let _ = on_token.send(TokenEvent { token });
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    persist_assistant(&state, &session_id, &question, &resp).await?;
    Ok(resp)
}

async fn persist_assistant(
    state: &AppState,
    session_id: &str,
    question: &str,
    resp: &AskResponse,
) -> Result<(), String> {
    let citations_json = serde_json::to_string(&resp.citations).map_err(|e| e.to_string())?;
    state
        .store
        .append_chat_message(
            session_id,
            ChatRole::Assistant,
            &resp.answer,
            Some(&citations_json),
        )
        .map_err(|e| e.to_string())?;

    if let Ok(session) = state.store.get_chat_session(session_id) {
        if session.title == "新对话" {
            let title: String = question.chars().take(32).collect();
            let _ = state.store.rename_chat_session(session_id, title.trim());
        }
    }

    if state.config().auto_learn_from_chat {
        let _ = learn_from_exchange(
            state.store.as_ref(),
            state.embedder().as_ref(),
            state.chat().as_ref(),
            &state.chunker,
            question,
            &resp.answer,
        )
        .await;
    }
    Ok(())
}

fn resolve_active_agent(cfg: &AppConfig) -> Result<AgentProfile, String> {
    cfg.agents
        .iter()
        .find(|a| a.id == cfg.active_agent_id)
        .cloned()
        .ok_or_else(|| format!("agent not found: {}", cfg.active_agent_id))
}

fn seed_skills_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let example = dir.join("concise-answers.md");
    if !example.exists() {
        std::fs::write(
            &example,
            "---\nname: 简洁回答\ndescription: 回答尽量简短、分点列出\n---\n回答时使用要点列表，避免冗长铺垫。",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_hooks_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let example = dir.join("log-tool-calls.json");
    if !example.exists() {
        let hook = serde_json::json!({
            "id": "log-tool-calls",
            "name": "记录工具调用",
            "description": "在 after_tool_call 时输出工具名（示例 Hook）",
            "event": "after_tool_call",
            "command": "echo tool called: %JARVIS_TOOL_NAME%"
        });
        std::fs::write(&example, serde_json::to_string_pretty(&hook).unwrap())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_plugins_dir(dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let plugin_dir = dir.join("datetime");
    std::fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    let manifest = plugin_dir.join("plugin.json");
    if !manifest.exists() {
        #[cfg(windows)]
        let command = r#"powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'""#;
        #[cfg(not(windows))]
        let command = "date '+%Y-%m-%d %H:%M:%S'";
        let plugin = serde_json::json!({
            "id": "datetime",
            "name": "日期时间",
            "description": "提供当前本地日期时间",
            "permissions": ["shell_exec"],
            "tools": [{
                "name": "current_time",
                "description": "返回当前本地日期时间",
                "command": command
            }]
        });
        std::fs::write(&manifest, serde_json::to_string_pretty(&plugin).unwrap())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn build_agent_context<'a>(
    state: &'a AppState,
    cfg: &'a AppConfig,
    embedder: &'a dyn Embedder,
    hooks: &'a [Hook],
    plugins: &'a [PluginManifest],
) -> AgentRunContext<'a> {
    AgentRunContext {
        store: state.store.as_ref(),
        embedder,
        chunker: &state.chunker,
        retriever: &state.retriever,
        hooks,
        enabled_hook_ids: &cfg.enabled_hook_ids,
        plugins,
        enabled_plugin_ids: &cfg.enabled_plugin_ids,
        granted_plugin_permissions: &cfg.granted_plugin_permissions,
    }
}

#[tauri::command]
fn list_agent_profiles(state: tauri::State<'_, AppState>) -> Result<Vec<AgentProfile>, String> {
    Ok(state.config().agents)
}

#[tauri::command]
fn list_skills(state: tauri::State<'_, AppState>) -> Result<Vec<Skill>, String> {
    Ok(load_skills_from_dir(&state.skills_dir))
}

#[tauri::command]
fn list_hooks(state: tauri::State<'_, AppState>) -> Result<Vec<Hook>, String> {
    Ok(load_hooks_from_dir(&state.hooks_dir))
}

#[tauri::command]
fn list_plugins(state: tauri::State<'_, AppState>) -> Result<Vec<PluginManifest>, String> {
    Ok(load_plugins_from_dir(&state.plugins_dir))
}

#[tauri::command]
fn set_active_agent(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config();
    if !cfg.agents.iter().any(|a| a.id == id && a.enabled) {
        return Err(format!("agent not found or disabled: {id}"));
    }
    cfg.active_agent_id = id;
    state.save_config(&cfg)
}

#[tauri::command]
fn remove_agent_profile(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if id == "default" {
        return Err("cannot remove built-in default agent".into());
    }
    let mut cfg = state.config();
    cfg.agents.retain(|a| a.id != id);
    if cfg.agents.is_empty() {
        return Err("must keep at least one agent profile".into());
    }
    if cfg.active_agent_id == id {
        cfg.active_agent_id = "default".to_string();
    }
    cfg.pipeline_agent_ids.retain(|x| x != &id);
    state.save_config(&cfg)
}

#[tauri::command]
fn upsert_agent_profile(
    profile: AgentProfile,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if profile.id.trim().is_empty() || profile.name.trim().is_empty() {
        return Err("agent id and name are required".into());
    }
    let mut cfg = state.config();
    if let Some(idx) = cfg.agents.iter().position(|a| a.id == profile.id) {
        cfg.agents[idx] = profile;
    } else {
        cfg.agents.push(profile);
    }
    state.save_config(&cfg)
}

#[tauri::command]
async fn ask_agent_in_session(
    session_id: String,
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(&session_id, ChatRole::User, &question, None)
        .map_err(|e| e.to_string())?;

    let cfg = state.config();
    let agent_resp = execute_agent_question(&state, &cfg, &question).await?;

    let resp = agent_response_to_ask(agent_resp);
    persist_assistant(&state, &session_id, &question, &resp).await?;
    Ok(resp)
}

#[tauri::command]
async fn ask_agent_in_session_stream(
    session_id: String,
    question: String,
    on_token: Channel<TokenEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(&session_id, ChatRole::User, &question, None)
        .map_err(|e| e.to_string())?;

    let cfg = state.config();
    let agent_resp = execute_agent_question(&state, &cfg, &question).await?;

    for chunk in agent_resp.answer.split_inclusive(' ') {
        let _ = on_token.send(TokenEvent {
            token: chunk.to_string(),
        });
    }

    let resp = agent_response_to_ask(agent_resp);
    persist_assistant(&state, &session_id, &question, &resp).await?;
    Ok(resp)
}

fn agent_response_to_ask(resp: AgentResponse) -> AskResponse {
    AskResponse {
        answer: resp.answer,
        citations: resp.citations,
        tool_calls: resp
            .tool_calls
            .into_iter()
            .map(|t| rag::ToolCallInfo {
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
                    .map(|t| rag::ToolCallInfo {
                        name: t.name,
                        arguments: t.arguments,
                        result: t.result,
                    })
                    .collect(),
            })
            .collect(),
    }
}

struct ConfigChatResolver {
    cfg: AppConfig,
}

impl ChatResolver for ConfigChatResolver {
    fn chat_for(&self, profile: &AgentProfile) -> std::sync::Arc<dyn ChatModel> {
        build_chat_model_for_profile(&self.cfg, profile)
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
    let embedder = state.embedder();
    let ctx = build_agent_context(
        state,
        cfg,
        embedder.as_ref(),
        &hooks,
        &plugins,
    );

    let chat_resolver = ConfigChatResolver { cfg: cfg.clone() };

    match cfg.agent_orchestration_mode {
        AgentOrchestrationMode::Single => {
            let profile = resolve_active_agent(cfg)?;
            let chat = chat_resolver.chat_for(&profile);
            run_agent(
                &ctx,
                chat.as_ref(),
                &profile,
                &skills,
                &cfg.enabled_skill_ids,
                question,
            )
            .await
            .map_err(|e| e.to_string())
        }
        AgentOrchestrationMode::Pipeline => run_orchestrated(
            &ctx,
            &chat_resolver,
            &cfg.agents,
            &cfg.pipeline_agent_ids,
            &skills,
            &cfg.enabled_skill_ids,
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
                &cfg.agents,
                &skills,
                &cfg.enabled_skill_ids,
                question,
            )
            .await
            .map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
fn check_lark_connection(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    check_auth(&runner, &cfg.lark_cli_bin).map_err(|e| e.to_string())
}

async fn index_lark(
    state: &AppState,
    doc: ingest::Document,
    kind: SourceKind,
) -> Result<String, String> {
    let id = doc.uri.clone();
    index_document(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        doc,
        kind,
    )
    .await
    .map_err(|e| e.to_string())?;
    state
        .store
        .set_meta("embedder_id", state.embedder().id())
        .map_err(|e| e.to_string())?;
    let cfg = state.config();
    maybe_run_insights_for_source(state.store.as_ref(), state.chat().as_ref(), &cfg, &id).await;
    Ok(id)
}

#[tauri::command]
async fn summarize_source_cmd(
    source_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    summarize_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        &source_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn extract_tasks_cmd(
    source_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Task>, String> {
    extract_tasks_from_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        &source_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_insights_all_cmd(
    summarize: bool,
    extract_tasks: bool,
    state: tauri::State<'_, AppState>,
) -> Result<InsightsReport, String> {
    run_insights_for_all_indexed(
        state.store.as_ref(),
        state.chat().as_ref(),
        summarize,
        extract_tasks,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_tasks(state: tauri::State<'_, AppState>) -> Result<Vec<Task>, String> {
    state.store.list_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_task_status(
    id: String,
    status: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let parsed = TaskStatus::parse(&status).ok_or_else(|| format!("invalid status: {status}"))?;
    state
        .store
        .update_task_status(&id, parsed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_task(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.store.delete_task(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_sync_status(state: tauri::State<'_, AppState>) -> Result<SyncStatusView, String> {
    Ok(sync_status_view(&state))
}

#[tauri::command]
async fn run_scheduled_sync_cmd(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    run_scheduled_sync(&app, &state).await
}

#[tauri::command]
fn list_memories_cmd(state: tauri::State<'_, AppState>) -> Result<Vec<Source>, String> {
    list_memories(state.store.as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_memory_cmd(
    content: String,
    title: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    add_memory(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &content,
        title.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_lark_doc(token: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let doc = fetch_doc(&runner, &cfg.lark_cli_bin, &token).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkDoc).await
}

#[tauri::command]
async fn sync_lark_sheet(
    token: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let doc = fetch_sheet(&runner, &cfg.lark_cli_bin, &token).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkSheet).await
}

#[tauri::command]
async fn sync_lark_mail(
    message_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let doc = fetch_mail(&runner, &cfg.lark_cli_bin, &message_id).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkMail).await
}

#[tauri::command]
async fn sync_lark_im(
    chat_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let doc = fetch_im_chat(&runner, &cfg.lark_cli_bin, &chat_id).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkMsg).await
}

#[tauri::command]
async fn index_file(path: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let id = index_path(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &path,
    )
    .await
    .map_err(|e| e.to_string())?;
    state
        .store
        .set_meta("embedder_id", state.embedder().id())
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
async fn ask_question(
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    ask(
        state.store.as_ref(),
        state.embedder().as_ref(),
        state.chat().as_ref(),
        &state.retriever,
        &question,
    )
    .await
    .map_err(|e| e.to_string())
}

fn init_state(app: &tauri::App) -> Result<AppState, String> {
    let app_data = if is_e2e_mode() {
        let dir = e2e_data_dir(&app.path().app_cache_dir().map_err(|e| e.to_string())?);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        dir
    } else {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        dir
    };

    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
    let skills_dir = app_data.join("skills");
    let hooks_dir = app_data.join("hooks");
    let plugins_dir = app_data.join("plugins");
    seed_skills_dir(&skills_dir)?;
    seed_hooks_dir(&hooks_dir)?;
    seed_plugins_dir(&plugins_dir)?;
    let mut config = if is_e2e_mode() {
        AppConfig::default()
    } else {
        load_config(&config_path).map_err(|e| e.to_string())?
    };
    if is_e2e_mode() {
        apply_e2e_config(&mut config);
        save_config(&config_path, &config).map_err(|e| e.to_string())?;
    }
    let config_dim = config.embedding_dim();

    let store = Arc::new(Store::open(&db_path, config_dim).map_err(|e| e.to_string())?);
    let embedder = build_embedder(&config).map_err(|e| e.to_string())?;
    let chat = build_chat_model(&config);

    if store.get_meta("embedder_id").ok().flatten().is_none() {
        let _ = store.set_meta("embedder_id", embedder.id());
        let _ = store.set_meta("vector_dim", &config_dim.to_string());
    }

    Ok(AppState {
        store,
        embedder: Mutex::new(embedder),
        chat: Mutex::new(chat),
        chunker: ChunkerConfig::default(),
        retriever: RetrieverConfig::default(),
        config_path,
        db_path,
        skills_dir,
        hooks_dir,
        plugins_dir,
        config: Mutex::new(config),
        watch: Mutex::new(None),
        scheduler: Mutex::new(None),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = init_state(app)?;
            let handle = app.handle().clone();
            if is_e2e_mode() {
                seed_e2e_fixture(&state)?;
            } else {
                state.initial_scan_with_progress(&handle)?;
            }
            state.restart_watcher()?;
            state.restart_scheduler(&handle);
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            source_count,
            list_sources,
            remove_source,
            get_config,
            set_config,
            get_index_status,
            rebuild_index,
            reinit_and_rebuild_index,
            retry_source,
            list_cursor_transcripts,
            sync_cursor_transcripts_cmd,
            add_watch_folder,
            remove_watch_folder,
            list_chat_sessions,
            create_chat_session,
            delete_chat_session,
            list_chat_messages,
            ask_in_session,
            ask_in_session_stream,
            check_lark_connection,
            sync_lark_doc,
            sync_lark_sheet,
            sync_lark_mail,
            sync_lark_im,
            index_file,
            ask_question,
            summarize_source_cmd,
            extract_tasks_cmd,
            run_insights_all_cmd,
            list_tasks,
            update_task_status,
            delete_task,
            get_sync_status,
            run_scheduled_sync_cmd,
            list_memories_cmd,
            add_memory_cmd,
            list_agent_profiles,
            list_skills,
            list_hooks,
            list_plugins,
            upsert_agent_profile,
            remove_agent_profile,
            set_active_agent,
            ask_agent_in_session,
            ask_agent_in_session_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
