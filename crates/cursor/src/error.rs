use thiserror::Error;

#[derive(Debug, Error)]
pub enum CursorError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid transcript: {0}")]
    Invalid(String),
    #[error("transcripts root not configured")]
    RootNotConfigured,
}

pub type Result<T> = std::result::Result<T, CursorError>;
