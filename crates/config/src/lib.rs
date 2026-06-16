mod error;
mod file;
mod providers;
mod types;

pub use agent::{AgentProfile, Skill};
pub use error::{ConfigError, Result};
pub use file::{load_config, save_config};
pub use providers::{
    build_chat_model, build_chat_model_for_profile, build_chat_model_with_provider,
    build_embedder, parse_chat_provider,
};
pub use types::{AgentOrchestrationMode, AppConfig, ChatProvider, EmbedderProvider};
