mod index_ops;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chunker::ChunkerConfig;
use config::{build_chat_model, build_embedder, load_config, save_config, AppConfig};
use embedder::Embedder;
use index_ops::{index_status_view, rebuild_all_sources, IndexStatusView, RebuildReport};
use indexer::{index_document, index_path};
use lark::{check_auth, fetch_doc, fetch_im_chat, fetch_mail, fetch_sheet, ProcessRunner};
use llm::ChatModel;
use rag::{ask, ask_stream, AskResponse};
use retriever::RetrieverConfig;
use serde::Serialize;
use store::{ChatMessage, ChatRole, ChatSession, Source, SourceKind, Store};
use tauri::ipc::Channel;
use tauri::Manager;
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
    config: Mutex<AppConfig>,
    watch: Mutex<Option<WatchHandle>>,
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

    fn initial_scan(&self) -> Result<(), String> {
        let cfg = self.config();
        for folder in &cfg.watch_folders {
            let paths = scan_folder(folder).map_err(|e| e.to_string())?;
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            for path in paths {
                let _ = rt.block_on(index_path(
                    self.store.as_ref(),
                    self.embedder().as_ref(),
                    &self.chunker,
                    &path,
                ));
            }
        }
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
fn set_config(config: AppConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.save_config(&config)?;
    state.restart_watcher()
}

#[tauri::command]
async fn rebuild_index(state: tauri::State<'_, AppState>) -> Result<RebuildReport, String> {
    let cfg = state.config();
    rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &cfg.lark_cli_bin,
    )
    .await
}

#[tauri::command]
async fn reinit_and_rebuild_index(
    state: tauri::State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let new_dim = cfg.embedding_dim();
    state.reload_providers(&cfg)?;
    state
        .store
        .reinit_vectors(new_dim)
        .map_err(|e| e.to_string())?;
    rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &cfg.lark_cli_bin,
    )
    .await
}

#[tauri::command]
fn add_watch_folder(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
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
    for file in paths {
        tauri::async_runtime::block_on(index_path(
            state.store.as_ref(),
            state.embedder().as_ref(),
            &state.chunker,
            &file,
        ))
        .map_err(|e| e.to_string())?;
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

    persist_assistant(&state, &session_id, &question, &resp)?;
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

    persist_assistant(&state, &session_id, &question, &resp)?;
    Ok(resp)
}

fn persist_assistant(
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
    Ok(())
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
    Ok(id)
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
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;

    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
    let config = load_config(&config_path).map_err(|e| e.to_string())?;
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
        config: Mutex::new(config),
        watch: Mutex::new(None),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = init_state(app)?;
            state.initial_scan()?;
            state.restart_watcher()?;
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
            ask_question
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
