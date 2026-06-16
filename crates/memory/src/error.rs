use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("empty exchange")]
    EmptyExchange,
    #[error("failed to parse memory json: {0}")]
    InvalidMemoryJson(String),
    #[error("llm: {0}")]
    Llm(#[from] llm::LlmError),
    #[error("index: {0}")]
    Index(#[from] indexer::IndexError),
    #[error("store: {0}")]
    Store(#[from] store::StoreError),
    #[error("memory not found: {0}")]
    NotFound(String),
    #[error("not a memory source: {0}")]
    NotMemory(String),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
