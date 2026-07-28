use std::path::{Path, PathBuf};

use crate::error::{McpError, Result};

const APP_DIR_NAME: &str = "com.jarvis.app";
const KB_FILE_NAME: &str = "kb.sqlite";

/// Default Jarvis app data directory (same identifier as Tauri `com.jarvis.app`).
pub fn default_app_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return appdata.join(APP_DIR_NAME);
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return home
            .join("Library")
            .join("Application Support")
            .join(APP_DIR_NAME);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
            return xdg.join(APP_DIR_NAME);
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return home.join(".local").join("share").join(APP_DIR_NAME);
    }
    #[cfg(not(any(windows, unix)))]
    {
        PathBuf::from(APP_DIR_NAME)
    }
}

/// Resolve `kb.sqlite` path: `--db` > `JARVIS_DATA_DIR` > default app data dir.
pub fn resolve_db_path(db_flag: Option<PathBuf>, data_dir_env: Option<PathBuf>) -> PathBuf {
    if let Some(path) = db_flag {
        return path;
    }
    if let Some(dir) = data_dir_env {
        return dir.join(KB_FILE_NAME);
    }
    default_app_data_dir().join(KB_FILE_NAME)
}

/// Fail closed if the DB file is missing — never create it.
pub fn ensure_db_exists(path: &Path) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(McpError::DbNotFound(path.to_path_buf()))
    }
}

/// Hand-rolled `--db <path>` parse (no clap). Unknown flags ignored.
pub fn parse_db_flag(args: impl IntoIterator<Item = String>) -> Option<PathBuf> {
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--db" {
            return iter.next().map(PathBuf::from);
        }
        if let Some(path) = arg.strip_prefix("--db=") {
            return Some(PathBuf::from(path));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_db_flag_reads_next_arg() {
        let args = vec![
            "jarvis-mcp".into(),
            "--db".into(),
            r"C:\tmp\kb.sqlite".into(),
            "--other".into(),
        ];
        assert_eq!(
            parse_db_flag(args),
            Some(PathBuf::from(r"C:\tmp\kb.sqlite"))
        );
    }

    #[test]
    fn parse_db_flag_equals_form() {
        assert_eq!(
            parse_db_flag(vec!["jarvis-mcp".into(), "--db=/tmp/kb.sqlite".into()]),
            Some(PathBuf::from("/tmp/kb.sqlite"))
        );
    }
}
