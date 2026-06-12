use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("ingest error: {0}")]
    Ingest(#[from] ingest::IngestError),
    #[error("embed error: {0}")]
    Embed(#[from] embedder::EmbedError),
    #[error("empty document: {0}")]
    EmptyDocument(String),
    #[error("dimension mismatch: store={store_dim}, embedder={embedder_dim}")]
    DimMismatch {
        store_dim: usize,
        embedder_dim: usize,
    },
}

pub type Result<T> = std::result::Result<T, IndexError>;
