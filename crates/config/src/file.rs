use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::types::AppConfig;

pub fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
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
        assert_eq!(cfg.cloud_embed_model, "text-embedding-3-small");
        assert_eq!(cfg.cloud_embed_dim, 1536);
        assert!(cfg.cloud_api_key.is_empty());
    }

    #[test]
    fn roundtrips_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = AppConfig::default();
        cfg.watch_folders.push("/tmp/docs".into());
        save_config(&path, &cfg).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.watch_folders, cfg.watch_folders);
    }
}
