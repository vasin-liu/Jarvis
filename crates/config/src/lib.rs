mod error;
mod file;
mod providers;
mod secrets;
mod types;

pub use agent::{AgentProfile, Skill};
pub use error::{ConfigError, Result};
pub use file::{is_e2e_mode, load_config, load_config_with_migration, save_config};
pub use secrets::{delete_api_key, get_api_key, set_api_key};
pub use providers::{
    build_chat_model, build_chat_model_for_profile, build_chat_model_with_provider,
    build_embedder, build_embedder_for_profile, build_embedder_with_provider,
    parse_chat_provider, parse_embedder_provider,
};
pub use types::{AgentOrchestrationMode, AppConfig, ChatProvider, EmbedderProvider, WikiConfig};
