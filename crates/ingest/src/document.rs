use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub uri: String,
    pub title: String,
    pub text: String,
    pub content_hash: String,
}
