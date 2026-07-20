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
pub struct OrchestrationStepInfo {
    pub agent_id: String,
    pub agent_name: String,
    pub answer: String,
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskResponse {
    pub answer: String,
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallInfo>,
    #[serde(default)]
    pub orchestration_steps: Vec<OrchestrationStepInfo>,
    #[serde(default)]
    pub tool_parse_warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ask_response_deserializes_without_warnings() {
        let json = r#"{"answer":"hi","citations":[]}"#;
        let resp: AskResponse = serde_json::from_str(json).unwrap();
        assert!(resp.tool_parse_warnings.is_empty());
    }
}
