use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::secrets;
use crate::types::AppConfig;

/// True when `JARVIS_E2E=1` — config crate must not touch the OS keychain in CI.
pub fn is_e2e_mode() -> bool {
    std::env::var("JARVIS_E2E").is_ok()
}

pub fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

/// Load config, migrate legacy plaintext `cloud_api_key` into the OS keychain, and
/// populate the runtime-only field from keychain when not in E2E mode.
pub fn load_config_with_migration(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    if !path.is_file() {
        return Ok(AppConfig::default());
    }

    let raw = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let legacy_key = value
        .get("cloud_api_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    let mut config: AppConfig = serde_json::from_str(&raw)?;

    if is_e2e_mode() {
        return Ok(config);
    }

    if let Some(key) = legacy_key {
        secrets::set_api_key(&key)?;
        save_config(path, &config)?;
        config.embedding.cloud_api_key = key;
    } else {
        config.embedding.cloud_api_key = secrets::get_api_key().unwrap_or_default();
    }

    Ok(config)
}

pub fn save_config(path: impl AsRef<Path>, config: &AppConfig) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(config)?;
    fs::write(path, raw)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_config_omits_cloud_api_key() {
        let mut cfg = AppConfig::default();
        cfg.embedding.cloud_api_key = "sk-secret".into();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(!json.contains("cloud_api_key"));
    }

    #[test]
    fn deserializes_legacy_config_without_cloud_fields() {
        let json = r#"{
            "watch_folders": [],
            "embedder": "mock",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli"
        }"#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.embedding.cloud_embed_model, "text-embedding-3-small");
        assert_eq!(cfg.embedding.cloud_embed_dim, 1536);
        assert!(cfg.embedding.cloud_api_key.is_empty());
        assert!(cfg.sync.cursor_projects_root.is_empty());
        assert!(!cfg.sync.auto_summarize_on_index);
        assert!(!cfg.sync.auto_extract_tasks_on_index);
    }

    #[test]
    fn roundtrips_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = AppConfig::default();
        cfg.sync.watch_folders.push("/tmp/docs".into());
        save_config(&path, &cfg).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.sync.watch_folders, cfg.sync.watch_folders);
    }
}
