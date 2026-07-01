use crate::index_ops::RebuildReport;
use crate::sync_scheduler::{run_scheduled_sync, sync_status_view, SyncStatusView};
use tauri::{AppHandle, State};

use crate::state::AppState;

#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatusView, String> {
    Ok(sync_status_view(&state))
}

#[tauri::command]
pub async fn run_scheduled_sync_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    run_scheduled_sync(&app, &state).await
}
