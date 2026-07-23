use insights::{
    compile_wiki_for_source, export_wiki_zip, wiki_has_exportable_notes, WikiCompileSummary,
};
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiExportPreflight {
    pub has_notes: bool,
}

fn wiki_root_from_state(state: &AppState) -> Result<std::path::PathBuf, String> {
    state
        .config_path
        .parent()
        .ok_or_else(|| "config_path has no parent".to_string())
        .map(|p| p.join("wiki"))
}

#[tauri::command]
pub async fn compile_wiki_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<WikiCompileSummary, String> {
    let cfg = state.config();
    let wiki_root = wiki_root_from_state(&state)?;
    compile_wiki_for_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &source_id,
        &wiki_root,
        cfg.wiki.enabled,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn wiki_export_preflight_cmd(
    state: State<'_, AppState>,
) -> Result<WikiExportPreflight, String> {
    let wiki_root = wiki_root_from_state(&state)?;
    Ok(WikiExportPreflight {
        has_notes: wiki_has_exportable_notes(&wiki_root),
    })
}

#[tauri::command]
pub async fn export_wiki_zip_cmd(
    dest_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cfg = state.config();
    let wiki_root = wiki_root_from_state(&state)?;
    export_wiki_zip(
        &wiki_root,
        std::path::Path::new(&dest_path),
        cfg.wiki.enabled,
    )
    .map_err(|e| e.to_string())
}
