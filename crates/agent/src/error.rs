use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent profile not found: {0}")]
    ProfileNotFound(String),
    #[error("agent profile disabled: {0}")]
    ProfileDisabled(String),
    #[error("failed to parse tool call: {0}")]
    InvalidToolCall(String),
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("tool args: {0}")]
    ToolArgs(String),
    #[error("llm: {0}")]
    Llm(#[from] llm::LlmError),
    #[error("retrieve: {0}")]
    Retrieve(#[from] retriever::RetrieveError),
    #[error("store: {0}")]
    Store(#[from] store::StoreError),
}

pub type Result<T> = std::result::Result<T, AgentError>;
