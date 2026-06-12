use std::sync::Arc;

use store::Store;

struct AppState {
    store: Arc<Store>,
}

#[tauri::command]
fn source_count(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = Arc::new(Store::open_in_memory(4).expect("open store"));

    tauri::Builder::default()
        .manage(AppState { store })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![source_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
