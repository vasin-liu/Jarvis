mod deferred;
mod error;
mod fastembed;
mod mock;
mod ollama;
mod openai;

pub use deferred::DeferredEmbedder;
pub use error::{EmbedError, Result};
pub use fastembed::{fastembed_model_dim, FastEmbedder};
pub use mock::MockEmbedder;
pub use ollama::OllamaEmbedder;
pub use openai::OpenAiEmbedder;

use async_trait::async_trait;

#[async_trait]
pub trait Embedder: Send + Sync {
    fn id(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
