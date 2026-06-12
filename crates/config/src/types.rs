use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedderProvider {
    Mock,
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatProvider {
    Mock,
    Ollama,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub watch_folders: Vec<String>,
    pub embedder: EmbedderProvider,
    pub chat: ChatProvider,
    pub mock_embed_dim: usize,
    pub ollama_base_url: String,
    pub ollama_embed_model: String,
    pub ollama_chat_model: String,
    pub ollama_embed_dim: usize,
    pub lark_cli_bin: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watch_folders: Vec::new(),
            embedder: EmbedderProvider::Mock,
            chat: ChatProvider::Mock,
            mock_embed_dim: 4,
            ollama_base_url: "http://127.0.0.1:11434".to_string(),
            ollama_embed_model: "nomic-embed-text".to_string(),
            ollama_chat_model: "llama3.2".to_string(),
            ollama_embed_dim: 768,
            lark_cli_bin: "lark-cli".to_string(),
        }
    }
}

impl AppConfig {
    pub fn embedding_dim(&self) -> usize {
        match self.embedder {
            EmbedderProvider::Mock => self.mock_embed_dim,
            EmbedderProvider::Ollama => self.ollama_embed_dim,
        }
    }
}
