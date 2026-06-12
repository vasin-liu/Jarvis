use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimMismatch { expected: usize, got: usize },
}

pub type Result<T> = std::result::Result<T, EmbedError>;
