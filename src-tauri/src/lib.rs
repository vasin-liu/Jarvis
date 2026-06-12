use std::sync::Arc;

use chunker::ChunkerConfig;
use embedder::MockEmbedder;
use indexer::index_path;
use llm::MockChatModel;
use rag::{ask, AskResponse};
use retriever::RetrieverConfig;
use store::Store;

struct AppState {
    store: Arc<Store>,
    embedder: Arc<MockEmbedder>,
    chat: Arc<MockChatModel>,
    chunker: ChunkerConfig,
    retriever: RetrieverConfig,
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

#[tauri::command]
async fn ask_question(
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<AskResponse, String> {
    ask(
        state.store.as_ref(),
        state.embedder.as_ref(),
        state.chat.as_ref(),
        &state.retriever,
        &question,
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = Arc::new(Store::open_in_memory(4).expect("open store"));
    let embedder = Arc::new(MockEmbedder::new(4));
    let chat = Arc::new(MockChatModel);

    tauri::Builder::default()
        .manage(AppState {
            store,
            embedder,
            chat,
            chunker: ChunkerConfig::default(),
            retriever: RetrieverConfig::default(),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            source_count,
            index_file,
            ask_question
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
