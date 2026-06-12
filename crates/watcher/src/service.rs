use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::event::{EventKind, ModifyKind};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Result, WatchError};
use crate::scan::is_indexable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    Modified(PathBuf),
    Removed(PathBuf),
}

pub struct WatchHandle {
    stop: Sender<()>,
    _watcher: RecommendedWatcher,
    join: Option<JoinHandle<()>>,
}

impl WatchHandle {
    pub fn stop(mut self) {
        let _ = self.stop.send(());
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

pub fn spawn_watcher(
    folders: Vec<PathBuf>,
    debounce_ms: u64,
) -> Result<(Receiver<WatchEvent>, WatchHandle)> {
    let (raw_tx, raw_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let Ok(event) = res else { return };
            for ev in classify_event(event) {
                let _ = raw_tx.send(ev);
            }
        },
        Config::default(),
    )
    .map_err(|e| WatchError::Notify(e.to_string()))?;

    for folder in &folders {
        if folder.is_dir() {
            watcher
                .watch(folder, RecursiveMode::Recursive)
                .map_err(|e| WatchError::Notify(e.to_string()))?;
        }
    }

    let debounce = Duration::from_millis(debounce_ms);
    let join = thread::spawn(move || {
        let mut pending: HashMap<PathBuf, WatchEvent> = HashMap::new();
        let mut last_change = Instant::now();

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            while let Ok(ev) = raw_rx.try_recv() {
                pending.insert(ev.as_path().to_path_buf(), ev);
                last_change = Instant::now();
            }

            if !pending.is_empty() && last_change.elapsed() >= debounce {
                for (_, ev) in pending.drain() {
                    let _ = out_tx.send(ev);
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    });

    Ok((
        out_rx,
        WatchHandle {
            stop: stop_tx,
            _watcher: watcher,
            join: Some(join),
        },
    ))
}

fn classify_event(event: notify::Event) -> Vec<WatchEvent> {
    let mut out = Vec::new();
    for path in event.paths {
        if !is_indexable(&path) {
            continue;
        }
        match event.kind {
            EventKind::Remove(_) => out.push(WatchEvent::Removed(path)),
            EventKind::Modify(ModifyKind::Name(_)) => out.push(WatchEvent::Modified(path)),
            EventKind::Modify(_) | EventKind::Create(_) => out.push(WatchEvent::Modified(path)),
            _ => {}
        }
    }
    out
}

impl WatchEvent {
    fn as_path(&self) -> &Path {
        match self {
            WatchEvent::Modified(p) | WatchEvent::Removed(p) => p,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn emits_modified_on_file_write() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("note.txt");
        fs::write(&file, "v1").unwrap();

        let (rx, _handle) = spawn_watcher(vec![dir.path().to_path_buf()], 150).unwrap();
        fs::write(&file, "v2").unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut saw = false;
        while Instant::now() < deadline {
            if let Ok(WatchEvent::Modified(p)) = rx.try_recv() {
                if p == file {
                    saw = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        assert!(saw, "expected modified event");
    }
}
