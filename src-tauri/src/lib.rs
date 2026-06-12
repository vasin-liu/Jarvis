use std::sync::Arc;

use chunker::ChunkerConfig;
use embedder::MockEmbedder;
use indexer::index_path;
use store::Store;

struct AppState {
    store: Arc<Store>,
    embedder: Arc<MockEmbedder>,
    chunker: ChunkerConfig,
}

#[tauri::command]
fn source_count(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn index_file(path: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    index_path(
        state.store.as_ref(),
        state.embedder.as_ref(),
        &state.chunker,
        &path,
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = Arc::new(Store::open_in_memory(4).expect("open store"));
    let embedder = Arc::new(MockEmbedder::new(4));

    tauri::Builder::default()
        .manage(AppState {
            store,
            embedder,
            chunker: ChunkerConfig::default(),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![source_count, index_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
