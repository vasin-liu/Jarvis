use std::path::PathBuf;

use mcp::{
    default_app_data_dir, ensure_db_exists, parse_db_flag, resolve_db_path,
};

#[test]
fn resolve_db_prefers_db_flag() {
    let flag = PathBuf::from("/explicit/kb.sqlite");
    let env_dir = PathBuf::from("/from-env");
    let resolved = resolve_db_path(Some(flag.clone()), Some(env_dir));
    assert_eq!(resolved, flag);
}

#[test]
fn resolve_db_from_data_dir_env() {
    let env_dir = PathBuf::from("/data/jarvis");
    let resolved = resolve_db_path(None, Some(env_dir.clone()));
    assert_eq!(resolved, env_dir.join("kb.sqlite"));
}

#[test]
fn resolve_db_default_app_data() {
    let resolved = resolve_db_path(None, None);
    assert_eq!(resolved, default_app_data_dir().join("kb.sqlite"));
    assert!(
        resolved
            .to_string_lossy()
            .contains("com.jarvis.app"),
        "default path should use Tauri identifier com.jarvis.app, got {}",
        resolved.display()
    );
}

#[test]
fn ensure_db_missing_errors_without_create() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("kb.sqlite");
    assert!(!missing.exists());
    let err = ensure_db_exists(&missing).unwrap_err();
    assert!(!missing.exists(), "must not create DB on missing path");
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("kb.sqlite"),
        "unexpected error: {msg}"
    );
}

#[test]
fn parse_db_flag_from_argv() {
    let args = vec![
        "jarvis-mcp".into(),
        "--verbose".into(),
        "--db".into(),
        r"C:\tmp\kb.sqlite".into(),
    ];
    assert_eq!(
        parse_db_flag(args),
        Some(PathBuf::from(r"C:\tmp\kb.sqlite"))
    );
}
