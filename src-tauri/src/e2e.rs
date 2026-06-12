use std::path::Path;

use config::{AppConfig, ChatProvider, EmbedderProvider};
use indexer::index_path;
use crate::AppState;

pub fn is_e2e_mode() -> bool {
    std::env::var("JARVIS_E2E").ok().as_deref() == Some("1")
}

pub fn e2e_data_dir(base: &Path) -> std::path::PathBuf {
    base.join("jarvis-e2e")
}

pub fn apply_e2e_config(config: &mut AppConfig) {
    config.embedder = EmbedderProvider::Mock;
    config.chat = ChatProvider::Mock;
    config.mock_embed_dim = 4;
    config.watch_folders.clear();
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
