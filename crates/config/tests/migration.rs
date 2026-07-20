use std::fs;

use config::{is_e2e_mode, load_config_with_migration, save_config, AppConfig};
use tempfile::TempDir;

#[test]
fn load_config_with_migration_e2e_skips_keychain_and_leaves_runtime_empty() {
    let prev = std::env::var("JARVIS_E2E").ok();
    std::env::set_var("JARVIS_E2E", "1");

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    fs::write(
        &path,
        r#"{
            "watch_folders": [],
            "embedder": "mock",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli",
            "cloud_api_key": "sk-legacy"
        }"#,
    )
    .unwrap();

    let cfg = load_config_with_migration(&path).unwrap();
    assert!(cfg.embedding.cloud_api_key.is_empty());

    match prev {
        Some(v) => std::env::set_var("JARVIS_E2E", v),
        None => std::env::remove_var("JARVIS_E2E"),
    }
}

#[test]
fn saved_config_json_never_contains_cloud_api_key() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    let mut cfg = AppConfig::default();
    cfg.embedding.cloud_api_key = "sk-runtime-only".into();
    save_config(&path, &cfg).unwrap();
    let on_disk = fs::read_to_string(&path).unwrap();
    assert!(!on_disk.contains("cloud_api_key"));
}

#[test]
fn is_e2e_mode_reads_env_var() {
    let prev = std::env::var("JARVIS_E2E").ok();
    std::env::set_var("JARVIS_E2E", "1");
    assert!(is_e2e_mode());
    std::env::remove_var("JARVIS_E2E");
    assert!(!is_e2e_mode());
    if let Some(v) = prev {
        std::env::set_var("JARVIS_E2E", v);
    }
}
