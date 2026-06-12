use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetrieveError {
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("embed error: {0}")]
    Embed(#[from] embedder::EmbedError),
    #[error("empty query")]
    EmptyQuery,
}

pub type Result<T> = std::result::Result<T, RetrieveError>;
