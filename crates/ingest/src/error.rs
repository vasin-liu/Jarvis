use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("unsupported file type: {0}")]
    UnsupportedType(String),
    #[error("empty document: {0}")]
    Empty(String),
}

pub type Result<T> = std::result::Result<T, IngestError>;
