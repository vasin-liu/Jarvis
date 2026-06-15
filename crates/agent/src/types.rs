use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentResponse {
    pub answer: String,
    pub citations: Vec<rag::Citation>,
    pub tool_calls: Vec<ToolCallRecord>,
}

pub fn default_profiles() -> Vec<AgentProfile> {
    vec![
        AgentProfile {
            id: "default".into(),
            name: "知识助手".into(),
            system_prompt: "你是 Jarvis 知识库助手。优先使用工具检索事实，回答简洁准确，使用中文。".into(),
            enabled: true,
        },
        AgentProfile {
            id: "tasks".into(),
            name: "任务助手".into(),
            system_prompt:
                "你是 Jarvis 任务助手。优先用 list_tasks 查看待办，用 complete_task 标记完成，必要时 search_knowledge 查背景。回答简洁，使用中文。"
                    .into(),
            enabled: true,
        },
    ]
}
