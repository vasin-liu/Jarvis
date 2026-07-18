use agent::default_profiles;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentOrchestrationMode {
    #[default]
    Single,
    Pipeline,
    Router,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedderProvider {
    Mock,
    Ollama,
    #[serde(alias = "fastembed")]
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
pub struct EmbeddingConfig {
    pub embedder: EmbedderProvider,
    pub mock_embed_dim: usize,
    pub ollama_base_url: String,
    pub ollama_embed_model: String,
    pub ollama_embed_dim: usize,
    #[serde(default = "default_fastembed_model")]
    pub fastembed_model: String,
    #[serde(default = "default_fastembed_dim")]
    pub fastembed_dim: usize,
    #[serde(default = "default_cloud_base_url")]
    pub cloud_base_url: String,
    #[serde(skip)]
    pub cloud_api_key: String,
    #[serde(default = "default_cloud_embed_model")]
    pub cloud_embed_model: String,
    #[serde(default = "default_cloud_embed_dim")]
    pub cloud_embed_dim: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConfig {
    pub chat: ChatProvider,
    pub ollama_chat_model: String,
    #[serde(default = "default_cloud_chat_model")]
    pub cloud_chat_model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LarkConfig {
    pub lark_cli_bin: String,
    /// Lark API identity: user | bot | auto (default user for personal resources).
    #[serde(default = "default_lark_identity")]
    pub lark_identity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncConfig {
    pub watch_folders: Vec<String>,
    /// Root of Cursor projects (e.g. `%USERPROFILE%\.cursor\projects`).
    #[serde(default)]
    pub cursor_projects_root: String,
    /// After indexing, auto-generate source summaries via LLM.
    #[serde(default)]
    pub auto_summarize_on_index: bool,
    /// After indexing, auto-extract tasks from sources via LLM.
    #[serde(default)]
    pub auto_extract_tasks_on_index: bool,
    /// Enable periodic background sync.
    #[serde(default)]
    pub scheduled_sync_enabled: bool,
    /// Interval in minutes (minimum 5 when enabled).
    #[serde(default = "default_sync_interval_minutes")]
    pub scheduled_sync_interval_minutes: u32,
    /// Include watch-folder rescan in scheduled sync.
    #[serde(default = "default_true")]
    pub scheduled_sync_watch_folders: bool,
    /// Include Lark source re-fetch in scheduled sync.
    #[serde(default)]
    pub scheduled_sync_lark: bool,
    /// Include Cursor transcript sync in scheduled sync.
    #[serde(default = "default_true")]
    pub scheduled_sync_cursor: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    /// After each Q&A, extract long-term memories into the knowledge base.
    #[serde(default)]
    pub auto_learn_from_chat: bool,
    /// Configurable agent profiles for tool-augmented chat.
    #[serde(default = "default_profiles")]
    pub agents: Vec<agent::AgentProfile>,
    /// Active agent profile id for chat.
    #[serde(default = "default_active_agent_id")]
    pub active_agent_id: String,
    /// Enabled skill ids from the skills directory.
    #[serde(default)]
    pub enabled_skill_ids: Vec<String>,
    /// Enabled hook ids from the hooks directory.
    #[serde(default)]
    pub enabled_hook_ids: Vec<String>,
    /// Enabled plugin ids from the plugins directory.
    #[serde(default)]
    pub enabled_plugin_ids: Vec<String>,
    /// Agent orchestration: single profile or multi-agent pipeline.
    #[serde(default)]
    pub agent_orchestration_mode: AgentOrchestrationMode,
    /// Ordered agent ids for pipeline mode; empty means all enabled agents.
    #[serde(default)]
    pub pipeline_agent_ids: Vec<String>,
    /// Permissions granted to plugins (e.g. shell_exec).
    #[serde(default = "default_granted_plugin_permissions")]
    pub granted_plugin_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_on_insights: bool,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_on_insights: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    pub embedding: EmbeddingConfig,
    #[serde(flatten)]
    pub chat_cfg: ChatConfig,
    #[serde(flatten)]
    pub lark: LarkConfig,
    #[serde(flatten)]
    pub sync: SyncConfig,
    #[serde(flatten)]
    pub agent_cfg: AgentConfig,
    /// Nested wiki compile settings (not flattened — D-04).
    #[serde(default)]
    pub wiki: WikiConfig,
    /// Runtime-only: absolute directory for the FastEmbed model cache. Set at
    /// startup to `app_data_dir/fastembed_cache` so the model is found
    /// regardless of the process working directory. Not persisted.
    #[serde(skip)]
    pub fastembed_cache_dir: Option<std::path::PathBuf>,
}

fn default_granted_plugin_permissions() -> Vec<String> {
    vec!["shell_exec".to_string()]
}

fn default_active_agent_id() -> String {
    "default".to_string()
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

fn default_sync_interval_minutes() -> u32 {
    60
}

fn default_lark_identity() -> String {
    "user".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            embedder: EmbedderProvider::FastEmbed,
            mock_embed_dim: 4,
            ollama_base_url: "http://127.0.0.1:11434".to_string(),
            ollama_embed_model: "nomic-embed-text".to_string(),
            ollama_embed_dim: 768,
            fastembed_model: default_fastembed_model(),
            fastembed_dim: default_fastembed_dim(),
            cloud_base_url: default_cloud_base_url(),
            cloud_api_key: String::new(),
            cloud_embed_model: default_cloud_embed_model(),
            cloud_embed_dim: default_cloud_embed_dim(),
        }
    }
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            chat: ChatProvider::Mock,
            ollama_chat_model: "llama3.2".to_string(),
            cloud_chat_model: default_cloud_chat_model(),
        }
    }
}

impl Default for LarkConfig {
    fn default() -> Self {
        Self {
            lark_cli_bin: "lark-cli".to_string(),
            lark_identity: default_lark_identity(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            watch_folders: Vec::new(),
            cursor_projects_root: String::new(),
            auto_summarize_on_index: false,
            auto_extract_tasks_on_index: false,
            scheduled_sync_enabled: false,
            scheduled_sync_interval_minutes: default_sync_interval_minutes(),
            scheduled_sync_watch_folders: true,
            scheduled_sync_lark: false,
            scheduled_sync_cursor: true,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            auto_learn_from_chat: false,
            agents: default_profiles(),
            active_agent_id: default_active_agent_id(),
            enabled_skill_ids: Vec::new(),
            enabled_hook_ids: Vec::new(),
            enabled_plugin_ids: Vec::new(),
            agent_orchestration_mode: AgentOrchestrationMode::Single,
            pipeline_agent_ids: Vec::new(),
            granted_plugin_permissions: default_granted_plugin_permissions(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig::default(),
            chat_cfg: ChatConfig::default(),
            lark: LarkConfig::default(),
            sync: SyncConfig::default(),
            agent_cfg: AgentConfig::default(),
            wiki: WikiConfig::default(),
            fastembed_cache_dir: None,
        }
    }
}

impl AppConfig {
    pub fn embedding_dim(&self) -> usize {
        match self.embedding.embedder {
            EmbedderProvider::Mock => self.embedding.mock_embed_dim,
            EmbedderProvider::Ollama => self.embedding.ollama_embed_dim,
            EmbedderProvider::FastEmbed => {
                embedder::fastembed_model_dim(&self.embedding.fastembed_model)
                    .unwrap_or(self.embedding.fastembed_dim)
            }
            EmbedderProvider::Cloud => self.embedding.cloud_embed_dim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_flat_json_roundtrip() {
        let flat_json = r#"{
            "watch_folders": ["/tmp/docs"],
            "embedder": "fastembed",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli",
            "fastembed_model": "bge-small-zh-v1.5",
            "fastembed_dim": 512
        }"#;

        let cfg: AppConfig = serde_json::from_str(flat_json).expect("deserialize");

        assert_eq!(cfg.sync.watch_folders, vec!["/tmp/docs"]);
        assert_eq!(cfg.embedding.embedder, EmbedderProvider::FastEmbed);
        assert_eq!(cfg.lark.lark_cli_bin, "lark-cli");

        let output = serde_json::to_value(&cfg).expect("serialize");
        assert!(
            output.get("watch_folders").is_some(),
            "watch_folders must be top-level"
        );
        assert!(output.get("sync").is_none(), "no 'sync' nesting object in JSON");
        assert!(
            output.get("embedder").is_some(),
            "embedder must be top-level"
        );
        assert!(!cfg.wiki.enabled);
        assert!(!cfg.wiki.auto_on_insights);
    }

    #[test]
    fn pre_v110_config_defaults_wiki_off() {
        let flat_json = r#"{
            "watch_folders": ["/tmp/docs"],
            "embedder": "mock",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli"
        }"#;

        let cfg: AppConfig = serde_json::from_str(flat_json).expect("deserialize");
        assert!(!cfg.wiki.enabled);
        assert!(!cfg.wiki.auto_on_insights);
        assert_eq!(AppConfig::default().wiki, WikiConfig::default());
    }

    #[test]
    fn wiki_config_explicit_roundtrip() {
        let json = r#"{
            "watch_folders": [],
            "embedder": "mock",
            "chat": "mock",
            "mock_embed_dim": 4,
            "ollama_base_url": "http://127.0.0.1:11434",
            "ollama_embed_model": "nomic-embed-text",
            "ollama_chat_model": "llama3.2",
            "ollama_embed_dim": 768,
            "lark_cli_bin": "lark-cli",
            "wiki": {
                "enabled": true,
                "auto_on_insights": true,
                "future_key": "ignored"
            }
        }"#;

        let cfg: AppConfig = serde_json::from_str(json).expect("deserialize");
        assert!(cfg.wiki.enabled);
        assert!(cfg.wiki.auto_on_insights);

        let output = serde_json::to_value(&cfg).expect("serialize");
        let wiki = output.get("wiki").expect("nested wiki key");
        assert_eq!(wiki.get("enabled"), Some(&serde_json::json!(true)));
        assert_eq!(wiki.get("auto_on_insights"), Some(&serde_json::json!(true)));
    }
}
