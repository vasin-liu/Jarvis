mod error;
mod mock;
mod ollama;
mod openai;
mod types;

pub use error::{LlmError, Result};
pub use mock::MockChatModel;
pub use ollama::OllamaChat;
pub use openai::OpenAiChat;
pub use types::{Message, Role};

use async_trait::async_trait;

#[async_trait]
pub trait ChatModel: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, messages: &[Message]) -> Result<String>;
    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String>;
}
