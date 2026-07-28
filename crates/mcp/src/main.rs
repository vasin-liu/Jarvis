use std::env;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use config::{load_config, AppConfig};
use mcp::{ensure_db_exists, parse_db_flag, resolve_db_path, JarvisMcp};
use rmcp::{ServiceExt, transport::stdio};
use store::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_flag = parse_db_flag(env::args());
    let data_dir = env::var_os("JARVIS_DATA_DIR").map(PathBuf::from);
    let db_path = resolve_db_path(db_flag.clone(), data_dir.clone());

    if let Err(err) = ensure_db_exists(&db_path) {
        eprintln!("{err}");
        eprintln!("Run the Jarvis desktop app once so kb.sqlite exists, then retry.");
        process::exit(1);
    }

    let config_dir = db_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.unwrap_or_else(mcp::default_app_data_dir));
    let config_path = config_dir.join("config.json");
    let dim = match load_config(&config_path) {
        Ok(cfg) => cfg.embedding_dim(),
        Err(_) => AppConfig::default().embedding_dim(),
    };

    let store = Arc::new(Store::open(&db_path, dim)?);
    let service = JarvisMcp::new(Some(store)).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
