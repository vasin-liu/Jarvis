mod bootstrap;
mod commands;
mod e2e;
mod events;
mod index_ops;
mod insights_ops;
mod state;
mod sync_scheduler;

pub(crate) use events::{emit_index_complete, emit_index_progress};
pub(crate) use state::{init_state, AppState};

use commands::{
    add_memory_cmd, add_watch_folder, ask_agent_in_session, ask_agent_in_session_stream,
    ask_in_session, ask_in_session_stream, ask_question, check_lark_connection, clear_api_key,
    create_chat_session, delete_chat_session, delete_task, detect_lark_cli, extract_tasks_cmd,
    forget_memory_cmd, get_api_key_status, get_config, get_index_status, get_memory_content_cmd,
    get_sync_status, index_file, is_e2e_mode_cmd, list_agent_profiles, list_chat_messages,
    list_chat_sessions, list_cursor_transcripts, list_hooks, list_memories_cmd, list_plugins,
    list_skills, list_sources, list_tasks, rebuild_index, reinit_and_rebuild_index,
    remove_agent_profile, remove_source, remove_watch_folder, retry_source, run_insights_all_cmd,
    run_scheduled_sync_cmd, set_active_agent, set_api_key, set_config, source_count,
    start_ask_e2e, summarize_source_cmd, sync_cursor_transcripts_cmd, sync_lark_doc, sync_lark_im,
    sync_lark_mail, sync_lark_sheet, sync_lark_url, update_memory_cmd, update_task_status,
    upsert_agent_profile,
};

use e2e::{is_e2e_mode, seed_e2e_fixture};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = init_state(app)?;
            let handle = app.handle().clone();
            if is_e2e_mode() {
                seed_e2e_fixture(&state)?;
            } else {
                state.initial_scan_with_progress(&handle)?;
            }
            state.restart_watcher()?;
            state.restart_scheduler(&handle);
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            source_count,
            list_sources,
            remove_source,
            get_config,
            set_config,
            get_api_key_status,
            set_api_key,
            clear_api_key,
            get_index_status,
            rebuild_index,
            reinit_and_rebuild_index,
            retry_source,
            list_cursor_transcripts,
            sync_cursor_transcripts_cmd,
            add_watch_folder,
            remove_watch_folder,
            list_chat_sessions,
            create_chat_session,
            delete_chat_session,
            list_chat_messages,
            is_e2e_mode_cmd,
            start_ask_e2e,
            ask_in_session,
            ask_in_session_stream,
            check_lark_connection,
            detect_lark_cli,
            sync_lark_doc,
            sync_lark_url,
            sync_lark_sheet,
            sync_lark_mail,
            sync_lark_im,
            index_file,
            ask_question,
            summarize_source_cmd,
            extract_tasks_cmd,
            run_insights_all_cmd,
            list_tasks,
            update_task_status,
            delete_task,
            get_sync_status,
            run_scheduled_sync_cmd,
            list_memories_cmd,
            get_memory_content_cmd,
            forget_memory_cmd,
            update_memory_cmd,
            add_memory_cmd,
            list_agent_profiles,
            list_skills,
            list_hooks,
            list_plugins,
            upsert_agent_profile,
            remove_agent_profile,
            set_active_agent,
            ask_agent_in_session,
            ask_agent_in_session_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
