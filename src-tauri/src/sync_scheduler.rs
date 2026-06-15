use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::index_ops::{index_local_paths, sync_cursor_transcripts, RebuildReport};
use crate::{emit_index_complete, emit_index_progress, AppState};
use watcher::scan_folder;

pub struct SchedulerHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl SchedulerHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

pub fn spawn_scheduler(app: AppHandle) -> SchedulerHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();

    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("scheduler runtime");
        while !stop_flag.load(Ordering::Relaxed) {
            let sleep_secs = {
                let state = match app.try_state::<AppState>() {
                    Some(s) => s,
                    None => break,
                };
                let cfg = state.config();
                if cfg.scheduled_sync_enabled {
                    cfg.scheduled_sync_interval_minutes.max(5) as u64 * 60
                } else {
                    60
                }
            };

            for _ in 0..sleep_secs {
                if stop_flag.load(Ordering::Relaxed) {
                    return;
                }
                std::thread::sleep(Duration::from_secs(1));
            }

            if stop_flag.load(Ordering::Relaxed) {
                return;
            }

            let state = match app.try_state::<AppState>() {
                Some(s) => s,
                None => break,
            };
            let cfg = state.config();
            if !cfg.scheduled_sync_enabled {
                continue;
            }

            let _ = rt.block_on(run_scheduled_sync(&app, &state));
        }
    });

    SchedulerHandle {
        stop,
        thread: Some(thread),
    }
}

pub async fn run_scheduled_sync(app: &AppHandle, state: &AppState) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let chat = state.chat();
    let mut indexed = 0usize;
    let mut failed = 0usize;

    if cfg.scheduled_sync_watch_folders {
        let mut all_paths = Vec::new();
        for folder in &cfg.watch_folders {
            if let Ok(paths) = scan_folder(folder) {
                all_paths.extend(paths);
            }
        }
        if !all_paths.is_empty() {
            let report = index_local_paths(
                state.store.as_ref(),
                state.embedder().as_ref(),
                &state.chunker,
                "scheduled",
                all_paths,
                Some((chat.as_ref(), &cfg)),
                |event| emit_index_progress(app, event),
            )
            .await?;
            indexed += report.indexed;
            failed += report.failed;
        }
    }

    if cfg.scheduled_sync_cursor && !cfg.cursor_projects_root.is_empty() {
        let report = sync_cursor_transcripts(
            state.store.as_ref(),
            state.embedder().as_ref(),
            &state.chunker,
            &cfg.cursor_projects_root,
            Some((chat.as_ref(), &cfg)),
            |event| emit_index_progress(app, event),
        )
        .await?;
        indexed += report.indexed;
        failed += report.failed;
    }

    let combined = RebuildReport {
        indexed,
        failed,
        skipped: 0,
    };
    let now = unix_now();
    state
        .store
        .set_meta("last_scheduled_sync_at", &now.to_string())
        .map_err(|e| e.to_string())?;
    emit_index_complete(app, &combined);
    Ok(combined)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusView {
    pub last_scheduled_sync_at: Option<i64>,
    pub scheduled_sync_enabled: bool,
    pub scheduled_sync_interval_minutes: u32,
}

pub fn sync_status_view(state: &AppState) -> SyncStatusView {
    let cfg = state.config();
    let last = state
        .store
        .get_meta("last_scheduled_sync_at")
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok());
    SyncStatusView {
        last_scheduled_sync_at: last,
        scheduled_sync_enabled: cfg.scheduled_sync_enabled,
        scheduled_sync_interval_minutes: cfg.scheduled_sync_interval_minutes,
    }
}
