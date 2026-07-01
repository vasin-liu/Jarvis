use std::path::Path;

use config::{AppConfig, ChatProvider, EmbedderProvider};
use indexer::index_path;
use ingest::{hash_text, Document};
use lark::LarkAuthStatus;
use store::SourceKind;

use crate::AppState;

pub fn is_e2e_mode() -> bool {
    std::env::var("JARVIS_E2E").ok().as_deref() == Some("1")
}

pub fn e2e_data_dir(base: &Path) -> std::path::PathBuf {
    base.join("jarvis-e2e")
}

pub const E2E_LARK_URL: &str = "https://e2e.feishu.cn/file/e2e-mock-token";
pub const E2E_LARK_MARKER: &str = "xyzzy-lark-e2e";

pub fn apply_e2e_config(config: &mut AppConfig) {
    config.embedding.embedder = EmbedderProvider::Mock;
    config.chat_cfg.chat = ChatProvider::Mock;
    config.embedding.mock_embed_dim = 4;
    config.sync.watch_folders.clear();
    config.lark.lark_identity = "user".to_string();
}

pub fn seed_e2e_fixture(state: &AppState) -> Result<(), String> {
    let fixture = std::env::var("JARVIS_E2E_FIXTURE")
        .map_err(|_| "JARVIS_E2E_FIXTURE is not set".to_string())?;
    if !Path::new(&fixture).is_file() {
        return Err(format!("e2e fixture missing: {fixture}"));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(index_path(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &fixture,
    ))
    .map_err(|e| e.to_string())?;

    state
        .store
        .set_meta("embedder_id", state.embedder().id())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn lark_auth_status() -> LarkAuthStatus {
    LarkAuthStatus {
        ok: true,
        identity: "user".to_string(),
        user_name: Some("E2E User".to_string()),
        user_open_id: Some("ou_e2e_mock".to_string()),
        token_status: Some("valid".to_string()),
        user_available: true,
        bot_available: true,
        note: Some("E2E mock — no live lark-cli".to_string()),
        hint: None,
        app_id: Some("cli_e2e".to_string()),
        brand: Some("feishu".to_string()),
    }
}

pub fn lark_fixture_from_input(input: &str) -> Option<(Document, SourceKind)> {
    if !is_e2e_mode() {
        return None;
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("e2e-mock")
        || trimmed == "e2e-lark"
        || trimmed.starts_with("https://e2e.feishu.cn/")
    {
        return Some((lark_document(), SourceKind::LarkFile));
    }
    None
}

pub fn lark_fixture_from_uri(uri: &str) -> Option<(Document, SourceKind)> {
    if !is_e2e_mode() {
        return None;
    }
    if uri.starts_with("lark://file/e2e-mock") {
        return Some((lark_document(), SourceKind::LarkFile));
    }
    None
}

fn lark_document() -> Document {
    let text = format!(
        "{E2E_LARK_MARKER}\n# Sheet1\ncol1\tcol2\nalpha\tbeta"
    );
    Document {
        uri: "lark://file/e2e-mock-token".to_string(),
        title: "E2E Lark Cloud File".to_string(),
        text: text.clone(),
        content_hash: hash_text(&text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lark_fixture_matches_e2e_url() {
        std::env::set_var("JARVIS_E2E", "1");
        let (doc, kind) = lark_fixture_from_input(E2E_LARK_URL).unwrap();
        assert_eq!(kind, SourceKind::LarkFile);
        assert!(doc.text.contains(E2E_LARK_MARKER));
        std::env::remove_var("JARVIS_E2E");
    }
}
