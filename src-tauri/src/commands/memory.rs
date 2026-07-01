use memory::{add_memory, forget_memory, get_memory_content, list_memories, update_memory};
use store::Source;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_memories_cmd(state: State<'_, AppState>) -> Result<Vec<Source>, String> {
    list_memories(state.store.as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_memory_content_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    get_memory_content(state.store.as_ref(), &source_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn forget_memory_cmd(id: String, state: State<'_, AppState>) -> Result<(), String> {
    forget_memory(state.store.as_ref(), &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_memory_cmd(
    id: String,
    content: String,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    update_memory(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &id,
        &content,
        title.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_memory_cmd(
    content: String,
    title: Option<String>,
    state: State<'_, AppState>,
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
