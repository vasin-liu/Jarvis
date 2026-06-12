mod error;
mod scan;
mod service;

pub use error::{Result, WatchError};
pub use scan::{is_indexable, scan_folder, unindex_path, uri_for_path};
pub use service::{spawn_watcher, WatchEvent, WatchHandle};
