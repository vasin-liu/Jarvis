use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("notify error: {0}")]
    Notify(String),
    #[error("folder not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, WatchError>;
