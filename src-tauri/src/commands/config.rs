use config::{
    delete_api_key as delete_stored_api_key, get_api_key as read_stored_api_key,
    set_api_key as store_api_key, AppConfig,
};
use crate::e2e::is_e2e_mode;
use crate::index_ops::{index_status_view, IndexStatusView};
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    pub has_key: bool,
}

fn keyring_err<E: std::fmt::Display>(_: E) -> String {
    "Failed to access credential store".to_string()
}

fn repopulate_runtime_api_key(state: &AppState, runtime_key: String) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.embedding.cloud_api_key = runtime_key;
    }
    state.reload_providers(&state.config())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config())
}

#[tauri::command]
pub fn get_index_status(state: State<'_, AppState>) -> Result<IndexStatusView, String> {
    let cfg = state.config();
    Ok(index_status_view(
        state.store.as_ref(),
        &cfg,
        state.embedder().as_ref(),
    ))
}

#[tauri::command]
pub fn get_api_key_status(_state: State<'_, AppState>) -> Result<ApiKeyStatus, String> {
    if is_e2e_mode() {
        return Ok(ApiKeyStatus { has_key: false });
    }
    let has_key = !read_stored_api_key().unwrap_or_default().is_empty();
    Ok(ApiKeyStatus { has_key })
}

#[tauri::command]
pub fn set_api_key(key: String, state: State<'_, AppState>) -> Result<(), String> {
    if is_e2e_mode() {
        return Ok(());
    }
    store_api_key(&key).map_err(keyring_err)?;
    repopulate_runtime_api_key(&state, key)
}

#[tauri::command]
pub fn clear_api_key(state: State<'_, AppState>) -> Result<(), String> {
    if is_e2e_mode() {
        return Ok(());
    }
    let _ = delete_stored_api_key();
    repopulate_runtime_api_key(&state, String::new())
}

#[tauri::command]
pub fn set_config(
    mut config: AppConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let incoming_key = std::mem::take(&mut config.embedding.cloud_api_key);

    if !is_e2e_mode() && !incoming_key.is_empty() {
        store_api_key(&incoming_key).map_err(keyring_err)?;
    }

    state.save_config(&config)?;

    if !is_e2e_mode() {
        let runtime_key = if incoming_key.is_empty() {
            read_stored_api_key().unwrap_or_default()
        } else {
            incoming_key
        };
        repopulate_runtime_api_key(&state, runtime_key)?;
    }

    state.restart_watcher()?;
    state.restart_scheduler(&app);
    Ok(())
}
