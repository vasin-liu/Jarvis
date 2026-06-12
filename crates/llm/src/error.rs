use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("empty messages")]
    EmptyMessages,
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid ollama response: {0}")]
    BadResponse(String),
}

pub type Result<T> = std::result::Result<T, LlmError>;
