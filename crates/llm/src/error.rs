use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("empty messages")]
    EmptyMessages,
}

pub type Result<T> = std::result::Result<T, LlmError>;
