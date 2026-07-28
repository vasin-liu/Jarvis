use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("knowledge base not found at {0}; run the Jarvis app once to create kb.sqlite")]
    DbNotFound(PathBuf),
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("{0}")]
    Other(String),
}
