use memory::learn_from_exchange;
use rag::{ask, ask_stream, AskResponse};
use store::{ChatMessage, ChatRole, ChatSession};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::run_ask_agent_in_session;
use crate::e2e::is_e2e_mode;
use crate::state::AppState;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenEvent {
    pub token: String,
}

#[tauri::command]
pub fn is_e2e_mode_cmd() -> bool {
    is_e2e_mode()
}

#[derive(Clone, serde::Serialize)]
struct AskDonePayload {
    session_id: String,
    error: Option<String>,
    messages: Vec<ChatMessage>,
}

/// E2E only: start ask in background so the WebView IPC call returns immediately.
#[tauri::command]
pub fn start_ask_e2e(
    session_id: String,
    question: String,
    agent_mode: bool,
    app: AppHandle,
) -> Result<(), String> {
    if !is_e2e_mode() {
        return Err("start_ask_e2e is only available in E2E mode".to_string());
    }

    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let state = state.inner();
        let result = if agent_mode {
            run_ask_agent_in_session(state, &session_id, &question).await
        } else {
            run_ask_in_session(state, &session_id, &question).await
        };
        let messages = state
            .store
            .list_chat_messages(&session_id)
            .unwrap_or_default();
        let payload = AskDonePayload {
            session_id,
            error: result.err().map(|e| e.to_string()),
            messages,
        };
        let _ = app.emit("jarvis-ask-done", payload);
    });

    Ok(())
}

#[tauri::command]
pub async fn ask_question(
    question: String,
    state: State<'_, AppState>,
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

#[tauri::command]
pub fn list_chat_sessions(state: State<'_, AppState>) -> Result<Vec<ChatSession>, String> {
    state.store.list_chat_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_chat_session(
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<ChatSession, String> {
    let title = title.unwrap_or_else(|| "新对话".to_string());
    state
        .store
        .create_chat_session(&title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_chat_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.store.delete_chat_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_chat_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    state
        .store
        .list_chat_messages(&session_id)
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_ask_in_session(
    state: &AppState,
    session_id: &str,
    question: &str,
) -> Result<AskResponse, String> {
    state
        .store
        .append_chat_message(session_id, ChatRole::User, question, None)
        .map_err(|e| e.to_string())?;

    let resp = ask(
        state.store.as_ref(),
        state.embedder().as_ref(),
        state.chat().as_ref(),
        &state.retriever,
        question,
    )
    .await
    .map_err(|e| e.to_string())?;

    persist_assistant(state, session_id, question, &resp).await?;
    Ok(resp)
}

#[tauri::command]
pub async fn ask_in_session(
    session_id: String,
    question: String,
    state: State<'_, AppState>,
) -> Result<AskResponse, String> {
    run_ask_in_session(state.inner(), &session_id, &question).await
}

#[tauri::command]
pub async fn ask_in_session_stream(
    session_id: String,
    question: String,
    on_token: Channel<TokenEvent>,
    state: State<'_, AppState>,
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

pub(crate) async fn persist_assistant(
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

    if state.config().agent_cfg.auto_learn_from_chat {
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
