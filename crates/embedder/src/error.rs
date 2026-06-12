use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: usize, got: usize },
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid ollama response: {0}")]
    BadResponse(String),
    #[error("fastembed error: {0}")]
    FastEmbed(String),
}

pub type Result<T> = std::result::Result<T, EmbedError>;
