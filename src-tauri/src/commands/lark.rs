use indexer::index_document;
use crate::insights_ops::maybe_run_insights_for_source;
use lark::{
    check_auth, detect_cli_bin, fetch_doc, fetch_from_url, fetch_im_chat, fetch_mail, fetch_sheet,
    LarkAuthStatus, ProcessRunner,
};
use store::SourceKind;
use tauri::State;

use crate::e2e::{self, is_e2e_mode};
use crate::state::AppState;

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

fn lark_kind_for_uri(uri: &str) -> SourceKind {
    if uri.starts_with("lark://file/") {
        SourceKind::LarkFile
    } else if uri.starts_with("lark://sheet/") {
        SourceKind::LarkSheet
    } else {
        SourceKind::LarkDoc
    }
}

#[tauri::command]
pub fn check_lark_connection(state: State<'_, AppState>) -> Result<LarkAuthStatus, String> {
    if is_e2e_mode() {
        return Ok(e2e::lark_auth_status());
    }
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    check_auth(&runner, &opts).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_lark_cli(_state: State<'_, AppState>) -> Result<Option<String>, String> {
    if is_e2e_mode() {
        return Ok(Some("lark-cli".to_string()));
    }
    Ok(detect_cli_bin(&ProcessRunner))
}

#[tauri::command]
pub async fn sync_lark_doc(token: String, state: State<'_, AppState>) -> Result<String, String> {
    if let Some((doc, kind)) = e2e::lark_fixture_from_input(&token) {
        return index_lark(&state, doc, kind).await;
    }
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    let doc = if token.trim().starts_with("http://") || token.trim().starts_with("https://") {
        fetch_from_url(&runner, &opts, token.trim()).map_err(|e| e.to_string())?
    } else {
        fetch_doc(&runner, &opts, &token).map_err(|e| e.to_string())?
    };
    let kind = lark_kind_for_uri(&doc.uri);
    index_lark(&state, doc, kind).await
}

#[tauri::command]
pub async fn sync_lark_url(url: String, state: State<'_, AppState>) -> Result<String, String> {
    if let Some((doc, kind)) = e2e::lark_fixture_from_input(&url) {
        return index_lark(&state, doc, kind).await;
    }
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    let doc = fetch_from_url(&runner, &opts, url.trim()).map_err(|e| e.to_string())?;
    let kind = lark_kind_for_uri(&doc.uri);
    index_lark(&state, doc, kind).await
}

#[tauri::command]
pub async fn sync_lark_sheet(
    token: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    let doc = fetch_sheet(&runner, &opts, &token).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkSheet).await
}

#[tauri::command]
pub async fn sync_lark_mail(
    message_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    let doc = fetch_mail(&runner, &opts, &message_id).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkMail).await
}

#[tauri::command]
pub async fn sync_lark_im(
    chat_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cfg = state.config();
    let runner = ProcessRunner;
    let opts = state.lark_opts(&cfg);
    let doc = fetch_im_chat(&runner, &opts, &chat_id).map_err(|e| e.to_string())?;
    index_lark(&state, doc, SourceKind::LarkMsg).await
}
