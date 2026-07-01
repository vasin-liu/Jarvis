use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use lark::{LarkCliOptions, LarkIdentity, ProcessRunner};

use crate::index_ops::{index_local_paths, sync_cursor_transcripts, sync_lark_sources, RebuildReport};
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
                if cfg.sync.scheduled_sync_enabled {
                    cfg.sync.scheduled_sync_interval_minutes.max(5) as u64 * 60
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
            if !cfg.sync.scheduled_sync_enabled {
                continue;
            }

            match rt.block_on(run_scheduled_sync(&app, &state)) {
                Ok(_) => {}
                Err(e) => persist_scheduled_sync_error(&state, &e),
            }
        }
    });

    SchedulerHandle {
        stop,
        thread: Some(thread),
    }
}

pub async fn run_scheduled_sync(app: &AppHandle, state: &AppState) -> Result<RebuildReport, String> {
    match run_scheduled_sync_inner(app, state).await {
        Ok(report) => Ok(report),
        Err(e) => {
            persist_scheduled_sync_error(state, &e);
            Err(e)
        }
    }
}

async fn run_scheduled_sync_inner(
    app: &AppHandle,
    state: &AppState,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let chat = state.chat();
    let mut indexed = 0usize;
    let mut failed = 0usize;

    if cfg.sync.scheduled_sync_watch_folders {
        let mut all_paths = Vec::new();
        for folder in &cfg.sync.watch_folders {
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

    if cfg.sync.scheduled_sync_cursor && !cfg.sync.cursor_projects_root.is_empty() {
        let report = sync_cursor_transcripts(
            state.store.as_ref(),
            state.embedder().as_ref(),
            &state.chunker,
            &cfg.sync.cursor_projects_root,
            Some((chat.as_ref(), &cfg)),
            |event| emit_index_progress(app, event),
        )
        .await?;
        indexed += report.indexed;
        failed += report.failed;
    }

    if cfg.sync.scheduled_sync_lark {
        let lark = LarkCliOptions::new(
            &cfg.lark.lark_cli_bin,
            LarkIdentity::parse(&cfg.lark.lark_identity),
        );
        let report = sync_lark_sources(
            state.store.as_ref(),
            state.embedder().as_ref(),
            &state.chunker,
            &ProcessRunner,
            &lark,
            &cfg.sync.cursor_projects_root,
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
    if combined.failed > 0 {
        let summary = format!("indexed {}, failed {}", combined.indexed, combined.failed);
        persist_scheduled_sync_error(state, &summary);
    } else {
        clear_scheduled_sync_error(state);
    }
    emit_index_complete(app, &combined);
    Ok(combined)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn persist_scheduled_sync_error(state: &AppState, error: &str) {
    let now = unix_now();
    let _ = state
        .store
        .set_meta("last_scheduled_sync_error", error);
    let _ = state
        .store
        .set_meta("last_scheduled_sync_error_at", &now.to_string());
}

fn clear_scheduled_sync_error(state: &AppState) {
    let _ = state.store.set_meta("last_scheduled_sync_error", "");
    let _ = state
        .store
        .set_meta("last_scheduled_sync_error_at", "");
}

fn meta_opt_string(state: &AppState, key: &str) -> Option<String> {
    state
        .store
        .get_meta(key)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn meta_opt_i64(state: &AppState, key: &str) -> Option<i64> {
    meta_opt_string(state, key).and_then(|s| s.parse().ok())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusView {
    pub last_scheduled_sync_at: Option<i64>,
    pub scheduled_sync_enabled: bool,
    pub scheduled_sync_interval_minutes: u32,
    #[serde(default)]
    pub last_scheduled_sync_error: Option<String>,
    #[serde(default)]
    pub last_scheduled_sync_error_at: Option<i64>,
}

pub fn sync_status_view(state: &AppState) -> SyncStatusView {
    let cfg = state.config();
    let last = meta_opt_i64(state, "last_scheduled_sync_at");
    SyncStatusView {
        last_scheduled_sync_at: last,
        scheduled_sync_enabled: cfg.sync.scheduled_sync_enabled,
        scheduled_sync_interval_minutes: cfg.sync.scheduled_sync_interval_minutes,
        last_scheduled_sync_error: meta_opt_string(state, "last_scheduled_sync_error"),
        last_scheduled_sync_error_at: meta_opt_i64(state, "last_scheduled_sync_error_at"),
    }
}
