mod error;
mod mock;
mod types;

pub use error::{LlmError, Result};
pub use mock::MockChatModel;
pub use types::{Message, Role};

use async_trait::async_trait;

#[async_trait]
pub trait ChatModel: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, messages: &[Message]) -> Result<String>;
}
