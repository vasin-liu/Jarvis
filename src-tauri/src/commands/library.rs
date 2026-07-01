use store::{Source, Task, TaskStatus};
use tauri::State;

use crate::insights_ops;
use crate::state::AppState;

#[tauri::command]
pub fn source_count(state: State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<Source>, String> {
    state.store.list_sources().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_source(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .store
        .delete_chunks_for_source(&id)
        .map_err(|e| e.to_string())?;
    state.store.delete_source(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn summarize_source_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    insights::summarize_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        &source_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_tasks_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Task>, String> {
    insights::extract_tasks_from_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        &source_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_insights_all_cmd(
    summarize: bool,
    extract_tasks: bool,
    state: State<'_, AppState>,
) -> Result<insights_ops::InsightsReport, String> {
    insights_ops::run_insights_for_all_indexed(
        state.store.as_ref(),
        state.chat().as_ref(),
        summarize,
        extract_tasks,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    state.store.list_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_status(
    id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let parsed = TaskStatus::parse(&status).ok_or_else(|| format!("invalid status: {status}"))?;
    state
        .store
        .update_task_status(&id, parsed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.store.delete_task(&id).map_err(|e| e.to_string())
}
