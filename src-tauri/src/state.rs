use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chunker::ChunkerConfig;
use config::{
    build_chat_model, build_embedder, load_config_with_migration, save_config, AppConfig,
};
use embedder::Embedder;
use lark::{LarkCliOptions, LarkIdentity};
use memory::migrate_legacy_memory_uris;

use crate::bootstrap::{seed_hooks_dir, seed_plugins_dir, seed_skills_dir};
use crate::e2e::{apply_e2e_config, e2e_data_dir, is_e2e_mode};
use crate::index_ops::index_local_paths;
use indexer::index_path;
use llm::ChatModel;
use retriever::RetrieverConfig;
use store::Store;
use crate::sync_scheduler::{spawn_scheduler, SchedulerHandle};
use tauri::{App, AppHandle, Manager};
use watcher::{scan_folder, spawn_watcher, unindex_path, WatchEvent, WatchHandle};

pub(crate) fn init_state(app: &App) -> Result<AppState, String> {
    let app_data = if is_e2e_mode() {
        let dir = e2e_data_dir(&app.path().app_cache_dir().map_err(|e| e.to_string())?);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        dir
    } else {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        dir
    };

    let config_path = app_data.join("config.json");
    let db_path = app_data.join("kb.sqlite");
    let skills_dir = app_data.join("skills");
    let hooks_dir = app_data.join("hooks");
    let plugins_dir = app_data.join("plugins");
    seed_skills_dir(&skills_dir)?;
    seed_hooks_dir(&hooks_dir)?;
    seed_plugins_dir(&plugins_dir)?;
    let mut config = if is_e2e_mode() {
        AppConfig::default()
    } else {
        load_config_with_migration(&config_path).map_err(|e| e.to_string())?
    };
    if is_e2e_mode() {
        apply_e2e_config(&mut config);
        save_config(&config_path, &config).map_err(|e| e.to_string())?;
    }
    config.fastembed_cache_dir = Some(app_data.join("fastembed_cache"));
    let config_dim = config.embedding_dim();

    let store = Arc::new(Store::open(&db_path, config_dim).map_err(|e| e.to_string())?);

    if !is_e2e_mode() {
        if let Ok(n) = migrate_legacy_memory_uris(store.as_ref()) {
            if n > 0 {
                eprintln!("migrated {n} legacy memory URIs to UUID");
            }
        }
    }

    let embedder = build_embedder(&config).map_err(|e| e.to_string())?;
    let chat = build_chat_model(&config);

    if store.get_meta("embedder_id").ok().flatten().is_none() {
        let _ = store.set_meta("embedder_id", embedder.id());
        let _ = store.set_meta("vector_dim", &config_dim.to_string());
    }

    Ok(AppState {
        store,
        embedder: Mutex::new(embedder),
        chat: Mutex::new(chat),
        chunker: ChunkerConfig::default(),
        retriever: RetrieverConfig::default(),
        config_path,
        db_path,
        skills_dir,
        hooks_dir,
        plugins_dir,
        config: Mutex::new(config),
        watch: Mutex::new(None),
        scheduler: Mutex::new(None),
    })
}

pub(crate) struct AppState {
    pub(crate) store: Arc<Store>,
    pub(crate) embedder: Mutex<Arc<dyn Embedder>>,
    pub(crate) chat: Mutex<Arc<dyn ChatModel>>,
    pub(crate) chunker: ChunkerConfig,
    pub(crate) retriever: RetrieverConfig,
    pub(crate) config_path: PathBuf,
    pub(crate) db_path: PathBuf,
    pub(crate) skills_dir: PathBuf,
    pub(crate) hooks_dir: PathBuf,
    pub(crate) plugins_dir: PathBuf,
    pub(crate) config: Mutex<AppConfig>,
    pub(crate) watch: Mutex<Option<WatchHandle>>,
    pub(crate) scheduler: Mutex<Option<SchedulerHandle>>,
}

impl AppState {
    pub(crate) fn config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    pub(crate) fn embedder(&self) -> Arc<dyn Embedder> {
        self.embedder.lock().unwrap().clone()
    }

    pub(crate) fn chat(&self) -> Arc<dyn ChatModel> {
        self.chat.lock().unwrap().clone()
    }

    pub(crate) fn lark_opts<'a>(&self, cfg: &'a AppConfig) -> LarkCliOptions<'a> {
        LarkCliOptions::new(
            &cfg.lark.lark_cli_bin,
            LarkIdentity::parse(&cfg.lark.lark_identity),
        )
    }

    pub(crate) fn reload_providers(&self, cfg: &AppConfig) -> Result<(), String> {
        *self.embedder.lock().unwrap() = build_embedder(cfg).map_err(|e| e.to_string())?;
        *self.chat.lock().unwrap() = build_chat_model(cfg);
        Ok(())
    }

    pub(crate) fn save_config(&self, cfg: &AppConfig) -> Result<(), String> {
        save_config(&self.config_path, cfg).map_err(|e| e.to_string())?;
        *self.config.lock().unwrap() = cfg.clone();
        self.reload_providers(cfg)
    }

    pub(crate) fn restart_watcher(&self) -> Result<(), String> {
        if let Some(handle) = self.watch.lock().unwrap().take() {
            handle.stop();
        }

        let folders: Vec<PathBuf> = self
            .config()
            .sync
            .watch_folders
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect();

        if folders.is_empty() {
            return Ok(());
        }

        let (rx, handle) = spawn_watcher(folders, 400).map_err(|e| e.to_string())?;
        *self.watch.lock().unwrap() = Some(handle);

        let store = self.store.clone();
        let embedder = self.embedder();
        let chunker = self.chunker.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            while let Ok(ev) = rx.recv() {
                match ev {
                    WatchEvent::Modified(path) => {
                        let _ = rt.block_on(index_path(
                            store.as_ref(),
                            embedder.as_ref(),
                            &chunker,
                            &path,
                        ));
                    }
                    WatchEvent::Removed(path) => {
                        let _ = unindex_path(store.as_ref(), &path);
                    }
                }
            }
        });

        Ok(())
    }

    pub(crate) fn restart_scheduler(&self, app: &AppHandle) {
        if let Some(handle) = self.scheduler.lock().unwrap().take() {
            handle.stop();
        }
        if !is_e2e_mode() {
            *self.scheduler.lock().unwrap() = Some(spawn_scheduler(app.clone()));
        }
    }

    pub(crate) fn initial_scan_with_progress(&self, app: &AppHandle) -> Result<(), String> {
        let cfg = self.config();
        let mut all_paths = Vec::new();
        for folder in &cfg.sync.watch_folders {
            let paths = scan_folder(folder).map_err(|e| e.to_string())?;
            all_paths.extend(paths);
        }
        if all_paths.is_empty() {
            return Ok(());
        }

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let cfg = self.config();
        let chat = self.chat();
        let report = rt.block_on(index_local_paths(
            self.store.as_ref(),
            self.embedder().as_ref(),
            &self.chunker,
            "scan",
            all_paths,
            Some((chat.as_ref(), &cfg)),
            |event| crate::emit_index_progress(app, event),
        ))?;
        crate::emit_index_complete(app, &report);
        Ok(())
    }
}
