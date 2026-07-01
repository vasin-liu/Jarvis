use std::path::PathBuf;

use cursor::list_transcript_summaries;
use crate::index_ops::{
    index_local_paths, rebuild_all_sources, retry_source_by_id, sync_cursor_transcripts, RebuildReport,
};
use indexer::index_path;
use lark::ProcessRunner;
use tauri::{AppHandle, State};
use watcher::scan_folder;

use crate::{emit_index_complete, emit_index_progress};
use crate::state::AppState;

#[tauri::command]
pub async fn rebuild_index(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let lark = state.lark_opts(&cfg);
    let report = rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &lark,
        &cfg.sync.cursor_projects_root,
        "rebuild",
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
pub async fn reinit_and_rebuild_index(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let new_dim = cfg.embedding_dim();
    state.reload_providers(&cfg)?;
    state
        .store
        .reinit_vectors(new_dim)
        .map_err(|e| e.to_string())?;
    let lark = state.lark_opts(&cfg);
    let report = rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &lark,
        &cfg.sync.cursor_projects_root,
        "reinit",
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
pub async fn retry_source(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let lark = state.lark_opts(&cfg);
    let report = retry_source_by_id(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &lark,
        &cfg.sync.cursor_projects_root,
        &id,
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
pub fn list_cursor_transcripts(
    state: State<'_, AppState>,
) -> Result<Vec<cursor::TranscriptSummary>, String> {
    let cfg = state.config();
    if cfg.sync.cursor_projects_root.is_empty() {
        return Ok(Vec::new());
    }
    list_transcript_summaries(std::path::Path::new(&cfg.sync.cursor_projects_root))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_cursor_transcripts_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let chat = state.chat();
    let report = sync_cursor_transcripts(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &cfg.sync.cursor_projects_root,
        Some((chat.as_ref(), &cfg)),
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}

#[tauri::command]
pub async fn add_watch_folder(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let folder = PathBuf::from(&path);
    if !folder.is_dir() {
        return Err(format!("not a directory: {path}"));
    }

    let mut cfg = state.config();
    if !cfg.sync.watch_folders.iter().any(|f| f == &path) {
        cfg.sync.watch_folders.push(path);
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
pub fn remove_watch_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config();
    cfg.sync.watch_folders.retain(|f| f != &path);
    state.save_config(&cfg)?;
    state.restart_watcher()
}

#[tauri::command]
pub async fn index_file(path: String, state: State<'_, AppState>) -> Result<String, String> {
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
