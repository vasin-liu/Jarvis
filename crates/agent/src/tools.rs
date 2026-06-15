use embedder::Embedder;
use retriever::{retrieve, RetrieverConfig};
use serde::Deserialize;
use serde_json::json;
use store::{IndexStatus, Store, TaskStatus};

use crate::error::{AgentError, Result};
use crate::types::ToolCallRecord;
use rag::Citation;

#[derive(Debug, Deserialize)]
pub(crate) struct ToolCallPayload {
    pub name: String,
    pub arguments: serde_json::Value,
}

pub fn parse_tool_call(text: &str) -> Option<ToolCallPayload> {
    let start = text.find("<tool_call>")?;
    let end = text.find("</tool_call>")?;
    let json = text[start + 11..end].trim();
    serde_json::from_str(json).ok()
}

pub async fn execute_tool(
    store: &Store,
    embedder: &dyn Embedder,
    retriever: &RetrieverConfig,
    name: &str,
    args: &serde_json::Value,
) -> Result<(String, Vec<Citation>)> {
    match name {
        "search_knowledge" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AgentError::ToolArgs("query required".into()))?;
            let hits = retrieve(store, embedder, query, retriever).await?;
            if hits.is_empty() {
                return Ok(("未找到相关内容。".into(), vec![]));
            }
            let mut citations = Vec::new();
            let mut lines = Vec::new();
            for hit in hits {
                let source = store.get_source(&hit.source_id)?;
                let excerpt = if hit.text.len() > 200 {
                    format!("{}…", &hit.text[..200])
                } else {
                    hit.text.clone()
                };
                lines.push(format!(
                    "- {} · {}: {}",
                    source.title, hit.loc, hit.text
                ));
                citations.push(Citation {
                    chunk_id: hit.chunk_id,
                    source_id: hit.source_id,
                    source_title: source.title,
                    source_uri: source.uri,
                    loc: hit.loc,
                    excerpt,
                });
            }
            Ok((lines.join("\n"), citations))
        }
        "list_sources" => {
            let sources = store.list_sources()?;
            let lines: Vec<String> = sources
                .into_iter()
                .filter(|s| s.status == IndexStatus::Indexed)
                .map(|s| format!("- {} ({})", s.title, s.kind.as_str()))
                .collect();
            Ok((lines.join("\n"), vec![]))
        }
        "list_tasks" => {
            let tasks = store.list_tasks()?;
            let lines: Vec<String> = tasks
                .into_iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .map(|t| {
                    format!(
                        "- {}{}",
                        t.title,
                        t.description
                            .as_ref()
                            .map(|d| format!(": {d}"))
                            .unwrap_or_default()
                    )
                })
                .collect();
            Ok((lines.join("\n"), vec![]))
        }
        other => Err(AgentError::UnknownTool(other.to_string())),
    }
}

pub fn tools_prompt() -> &'static str {
    r#"可用工具（需要时仅回复一行 tool_call，不要其他文字）：
<tool_call>{"name":"search_knowledge","arguments":{"query":"关键词"}}</tool_call>
<tool_call>{"name":"list_sources","arguments":{}}</tool_call>
<tool_call>{"name":"list_tasks","arguments":{}}</tool_call>
拿到工具结果后，用中文给出最终答案。"#
}

pub async fn run_tool_call(
    store: &Store,
    embedder: &dyn Embedder,
    retriever: &RetrieverConfig,
    payload: &ToolCallPayload,
) -> Result<ToolCallRecord> {
    let (result, _) = execute_tool(
        store,
        embedder,
        retriever,
        &payload.name,
        &payload.arguments,
    )
    .await?;
    Ok(ToolCallRecord {
        name: payload.name.clone(),
        arguments: payload.arguments.clone(),
        result,
    })
}

pub fn tool_call_example_json() -> serde_json::Value {
    json!({"name":"search_knowledge","arguments":{"query":"example"}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_call_block() {
        let text = r#"prefix <tool_call>{"name":"list_tasks","arguments":{}}</tool_call>"#;
        let call = parse_tool_call(text).unwrap();
        assert_eq!(call.name, "list_tasks");
    }
}
