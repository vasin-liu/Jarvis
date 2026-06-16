use std::sync::Arc;

use embedder::{
    Embedder, EmbedError, FastEmbedder, MockEmbedder, OllamaEmbedder, OpenAiEmbedder,
};
use llm::{ChatModel, MockChatModel, OllamaChat, OpenAiChat};

use agent::AgentProfile;
use crate::types::{AppConfig, ChatProvider, EmbedderProvider};

pub fn parse_chat_provider(name: &str) -> Option<ChatProvider> {
    match name.to_ascii_lowercase().as_str() {
        "mock" => Some(ChatProvider::Mock),
        "ollama" => Some(ChatProvider::Ollama),
        "cloud" => Some(ChatProvider::Cloud),
        _ => None,
    }
}

pub fn build_embedder(config: &AppConfig) -> Result<Arc<dyn Embedder>, EmbedError> {
    match config.embedder {
        EmbedderProvider::Mock => Ok(Arc::new(MockEmbedder::new(config.mock_embed_dim))),
        EmbedderProvider::Ollama => Ok(Arc::new(OllamaEmbedder::new(
            config.ollama_base_url.clone(),
            config.ollama_embed_model.clone(),
            config.ollama_embed_dim,
        ))),
        EmbedderProvider::FastEmbed => Ok(Arc::new(FastEmbedder::try_new(
            &config.fastembed_model,
        )?)),
        EmbedderProvider::Cloud => Ok(Arc::new(OpenAiEmbedder::new(
            config.cloud_base_url.clone(),
            config.cloud_api_key.clone(),
            config.cloud_embed_model.clone(),
            config.cloud_embed_dim,
        ))),
    }
}

pub fn build_chat_model_with_provider(
    config: &AppConfig,
    provider: ChatProvider,
) -> Arc<dyn ChatModel> {
    match provider {
        ChatProvider::Mock => Arc::new(MockChatModel),
        ChatProvider::Ollama => Arc::new(OllamaChat::new(
            config.ollama_base_url.clone(),
            config.ollama_chat_model.clone(),
        )),
        ChatProvider::Cloud => Arc::new(OpenAiChat::new(
            config.cloud_base_url.clone(),
            config.cloud_api_key.clone(),
            config.cloud_chat_model.clone(),
        )),
    }
}

pub fn build_chat_model(config: &AppConfig) -> Arc<dyn ChatModel> {
    build_chat_model_with_provider(config, config.chat)
}

pub fn build_chat_model_for_profile(
    config: &AppConfig,
    profile: &AgentProfile,
) -> Arc<dyn ChatModel> {
    let provider = profile
        .chat_provider
        .as_deref()
        .and_then(parse_chat_provider)
        .unwrap_or(config.chat);
    build_chat_model_with_provider(config, provider)
}
