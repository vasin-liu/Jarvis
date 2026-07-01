use crate::index_ops::{IndexProgressEvent, RebuildReport};
use tauri::{AppHandle, Emitter};

pub(crate) fn emit_index_progress(app: &AppHandle, event: IndexProgressEvent) {
    let _ = app.emit("index-progress", event);
}

pub(crate) fn emit_index_complete(app: &AppHandle, report: &RebuildReport) {
    let _ = app.emit("index-complete", report);
}
