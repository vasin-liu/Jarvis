mod error;
mod file;
mod providers;
mod types;

pub use error::{ConfigError, Result};
pub use file::{load_config, save_config};
pub use providers::{build_chat_model, build_embedder};
pub use types::{AppConfig, ChatProvider, EmbedderProvider};
