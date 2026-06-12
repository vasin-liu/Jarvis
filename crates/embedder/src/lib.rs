mod error;
mod mock;

pub use error::{EmbedError, Result};
pub use mock::MockEmbedder;

use async_trait::async_trait;

#[async_trait]
pub trait Embedder: Send + Sync {
    fn id(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
