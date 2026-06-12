use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedderProvider {
    Mock,
    Ollama,
    FastEmbed,
    Cloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatProvider {
    Mock,
    Ollama,
    Cloud,
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
    #[serde(default = "default_fastembed_model")]
    pub fastembed_model: String,
    #[serde(default = "default_fastembed_dim")]
    pub fastembed_dim: usize,
    #[serde(default = "default_cloud_base_url")]
    pub cloud_base_url: String,
    #[serde(default)]
    pub cloud_api_key: String,
    #[serde(default = "default_cloud_embed_model")]
    pub cloud_embed_model: String,
    #[serde(default = "default_cloud_chat_model")]
    pub cloud_chat_model: String,
    #[serde(default = "default_cloud_embed_dim")]
    pub cloud_embed_dim: usize,
    /// Root of Cursor projects (e.g. `%USERPROFILE%\.cursor\projects`).
    #[serde(default)]
    pub cursor_projects_root: String,
    /// After indexing, auto-generate source summaries via LLM.
    #[serde(default)]
    pub auto_summarize_on_index: bool,
    /// After indexing, auto-extract tasks from sources via LLM.
    #[serde(default)]
    pub auto_extract_tasks_on_index: bool,
}

fn default_fastembed_model() -> String {
    "bge-small-zh-v1.5".to_string()
}

fn default_fastembed_dim() -> usize {
    512
}

fn default_cloud_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_cloud_embed_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_cloud_chat_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_cloud_embed_dim() -> usize {
    1536
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watch_folders: Vec::new(),
            embedder: EmbedderProvider::FastEmbed,
            chat: ChatProvider::Mock,
            mock_embed_dim: 4,
            ollama_base_url: "http://127.0.0.1:11434".to_string(),
            ollama_embed_model: "nomic-embed-text".to_string(),
            ollama_chat_model: "llama3.2".to_string(),
            ollama_embed_dim: 768,
            lark_cli_bin: "lark-cli".to_string(),
            fastembed_model: "bge-small-zh-v1.5".to_string(),
            fastembed_dim: 512,
            cloud_base_url: default_cloud_base_url(),
            cloud_api_key: String::new(),
            cloud_embed_model: default_cloud_embed_model(),
            cloud_chat_model: default_cloud_chat_model(),
            cloud_embed_dim: default_cloud_embed_dim(),
            cursor_projects_root: String::new(),
            auto_summarize_on_index: false,
            auto_extract_tasks_on_index: false,
        }
    }
}

impl AppConfig {
    pub fn embedding_dim(&self) -> usize {
        match self.embedder {
            EmbedderProvider::Mock => self.mock_embed_dim,
            EmbedderProvider::Ollama => self.ollama_embed_dim,
            EmbedderProvider::FastEmbed => embedder::fastembed_model_dim(&self.fastembed_model)
                .unwrap_or(self.fastembed_dim),
            EmbedderProvider::Cloud => self.cloud_embed_dim,
        }
    }
}
