use std::sync::Arc;

use embedder::Embedder;

use crate::types::AgentProfile;

pub trait EmbedResolver: Send + Sync {
    fn embed_for(&self, profile: &AgentProfile) -> Arc<dyn Embedder>;
}
