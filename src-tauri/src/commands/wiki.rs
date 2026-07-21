use insights::{compile_wiki_for_source, WikiCompileSummary};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn compile_wiki_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<WikiCompileSummary, String> {
    let cfg = state.config();
    let wiki_root = state
        .config_path
        .parent()
        .ok_or_else(|| "config_path has no parent".to_string())?
        .join("wiki");
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
