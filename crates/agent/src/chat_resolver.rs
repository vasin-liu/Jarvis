use std::sync::Arc;

use llm::ChatModel;

use crate::types::AgentProfile;

pub trait ChatResolver: Send + Sync {
    fn chat_for(&self, profile: &AgentProfile) -> Arc<dyn ChatModel>;
}
