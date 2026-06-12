use std::sync::Arc;

use embedder::{Embedder, MockEmbedder, OllamaEmbedder};
use llm::{ChatModel, MockChatModel, OllamaChat};

use crate::types::{AppConfig, ChatProvider, EmbedderProvider};

pub fn build_embedder(config: &AppConfig) -> Arc<dyn Embedder> {
    match config.embedder {
        EmbedderProvider::Mock => Arc::new(MockEmbedder::new(config.mock_embed_dim)),
        EmbedderProvider::Ollama => Arc::new(OllamaEmbedder::new(
            config.ollama_base_url.clone(),
            config.ollama_embed_model.clone(),
            config.ollama_embed_dim,
        )),
    }
}

pub fn build_chat_model(config: &AppConfig) -> Arc<dyn ChatModel> {
    match config.chat {
        ChatProvider::Mock => Arc::new(MockChatModel),
        ChatProvider::Ollama => Arc::new(OllamaChat::new(
            config.ollama_base_url.clone(),
            config.ollama_chat_model.clone(),
        )),
    }
}
