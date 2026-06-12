use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsightsError {
    #[error("source not indexed: {0}")]
    NotIndexed(String),
    #[error("no chunk text for source: {0}")]
    EmptySource(String),
    #[error("failed to parse tasks json: {0}")]
    InvalidTasksJson(String),
    #[error("llm: {0}")]
    Llm(#[from] llm::LlmError),
    #[error("store: {0}")]
    Store(#[from] store::StoreError),
}

pub type Result<T> = std::result::Result<T, InsightsError>;
