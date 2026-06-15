use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: i64,
    pub source_id: String,
    pub source_title: String,
    pub source_uri: String,
    pub loc: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskResponse {
    pub answer: String,
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallInfo>,
}
