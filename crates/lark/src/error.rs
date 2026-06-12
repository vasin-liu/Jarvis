use thiserror::Error;

#[derive(Debug, Error)]
pub enum LarkError {
    #[error("command failed: {0}")]
    Command(String),
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing field in lark response: {0}")]
    MissingField(String),
    #[error("empty document: {0}")]
    Empty(String),
}

pub type Result<T> = std::result::Result<T, LarkError>;
