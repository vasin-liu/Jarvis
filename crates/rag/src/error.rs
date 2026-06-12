use thiserror::Error;

#[derive(Debug, Error)]
pub enum RagError {
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("retrieve error: {0}")]
    Retrieve(#[from] retriever::RetrieveError),
    #[error("llm error: {0}")]
    Llm(#[from] llm::LlmError),
}

pub type Result<T> = std::result::Result<T, RagError>;
