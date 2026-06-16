use chunker::ChunkerConfig;
use embedder::Embedder;
use memory::{add_memory, forget_memory, get_memory_content, list_memories, update_memory};
use retriever::{retrieve, RetrieverConfig};
use serde::Deserialize;
use serde_json::json;
use store::{IndexStatus, Store, TaskStatus};

use crate::error::{AgentError, Result};
use crate::plugins::{
    enabled_plugin_tools, execute_plugin_tool, find_plugin_tool, plugin_has_permissions,
    PluginManifest,
};
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
    chunker: &ChunkerConfig,
    retriever: &RetrieverConfig,
    plugins: &[PluginManifest],
    enabled_plugin_ids: &[String],
    granted_plugin_permissions: &[String],
    name: &str,
    args: &serde_json::Value,
) -> Result<(String, Vec<Citation>)> {
    if let Some((plugin, tool)) =
        find_plugin_tool(plugins, enabled_plugin_ids, granted_plugin_permissions, name)
    {
        if !plugin_has_permissions(plugin, granted_plugin_permissions) {
            return Err(AgentError::PluginPermission(format!(
                "plugin `{}` requires {:?}",
                plugin.id, plugin.permissions
            )));
        }
        let result = execute_plugin_tool(tool, args)?;
        return Ok((format!("[插件 {}] {result}", plugin.name), vec![]));
    }

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
                        "- [{}] {}{}",
                        t.id,
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
        "list_memories" => {
            let memories = list_memories(store)?;
            if memories.is_empty() {
                return Ok(("暂无长期记忆。".into(), vec![]));
            }
            let lines: Vec<String> = memories
                .into_iter()
                .map(|m| format!("- {} ({})", m.title, m.uri))
                .collect();
            Ok((lines.join("\n"), vec![]))
        }
        "add_memory" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AgentError::ToolArgs("content required".into()))?;
            let title = args.get("title").and_then(|v| v.as_str());
            let uri = add_memory(store, embedder, chunker, content, title).await?;
            Ok((format!("已写入记忆：{uri}"), vec![]))
        }
        "get_memory" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AgentError::ToolArgs("id required".into()))?;
            let content = get_memory_content(store, id)?;
            let source = store.get_source(id)?;
            Ok((
                format!("{} ({})\n\n{content}", source.title, source.uri),
                vec![],
            ))
        }
        "forget_memory" => {
            let id = args
                .get("id")
                .or_else(|| args.get("title"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| AgentError::ToolArgs("id or title required".into()))?;
            forget_memory(store, id)?;
            Ok((format!("已删除记忆：{id}"), vec![]))
        }
        "update_memory" => {
            let id = args.get("id").and_then(|v| v.as_str());
            let title_arg = args.get("title").and_then(|v| v.as_str());
            let lookup = id
                .or(title_arg)
                .ok_or_else(|| AgentError::ToolArgs("id or title required".into()))?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AgentError::ToolArgs("content required".into()))?;
            let new_title = if id.is_some() { title_arg } else { None };
            let updated =
                update_memory(store, embedder, chunker, lookup, content, new_title).await?;
            Ok((format!("已更新记忆：{updated}"), vec![]))
        }
        "complete_task" => {
            let id = args.get("id").and_then(|v| v.as_str());
            let title = args.get("title").and_then(|v| v.as_str());
            let task = if let Some(id) = id {
                store.get_task(id).ok()
            } else if let Some(title) = title {
                store
                    .list_tasks()?
                    .into_iter()
                    .find(|t| t.status == TaskStatus::Pending && t.title.contains(title))
            } else {
                None
            };
            let Some(task) = task else {
                return Ok(("未找到匹配的待办任务。".into(), vec![]));
            };
            store.update_task_status(&task.id, TaskStatus::Done)?;
            Ok((format!("已将任务「{}」标记为完成。", task.title), vec![]))
        }
        other => Err(AgentError::UnknownTool(other.to_string())),
    }
}

pub fn build_tools_prompt(
    plugins: &[PluginManifest],
    enabled_plugin_ids: &[String],
    granted_plugin_permissions: &[String],
) -> String {
    let mut lines = vec![
        "可用工具（需要时仅回复一行 tool_call，不要其他文字）：".to_string(),
        r#"<tool_call>{"name":"search_knowledge","arguments":{"query":"关键词"}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"list_sources","arguments":{}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"list_tasks","arguments":{}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"list_memories","arguments":{}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"add_memory","arguments":{"content":"要记住的事实","title":"可选标题"}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"get_memory","arguments":{"id":"memory://..."}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"forget_memory","arguments":{"id":"memory://..."}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"update_memory","arguments":{"id":"memory://...","content":"新内容","title":"可选标题"}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"complete_task","arguments":{"id":"任务id"}}</tool_call>"#.into(),
        r#"<tool_call>{"name":"complete_task","arguments":{"title":"任务标题关键词"}}</tool_call>"#.into(),
    ];

    for (_, tool) in enabled_plugin_tools(plugins, enabled_plugin_ids, granted_plugin_permissions) {
        lines.push(format!(
            r#"<tool_call>{{"name":"{}","arguments":{{}}}}</tool_call>"#,
            tool.name
        ));
    }

    lines.push("拿到工具结果后，用中文给出最终答案。".into());
    lines.join("\n")
}

pub async fn run_tool_call(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    retriever: &RetrieverConfig,
    plugins: &[PluginManifest],
    enabled_plugin_ids: &[String],
    granted_plugin_permissions: &[String],
    payload: &ToolCallPayload,
) -> Result<ToolCallRecord> {
    let (result, _) = execute_tool(
        store,
        embedder,
        chunker,
        retriever,
        plugins,
        enabled_plugin_ids,
        granted_plugin_permissions,
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

    #[test]
    fn build_prompt_includes_plugin_tool() {
        let plugins = vec![PluginManifest {
            id: "demo".into(),
            name: "Demo".into(),
            description: String::new(),
            permissions: vec![],
            tools: vec![crate::plugins::PluginTool {
                name: "ping".into(),
                description: "ping".into(),
                command: "echo pong".into(),
            }],
        }];
        let prompt = build_tools_prompt(&plugins, &["demo".into()], &["shell_exec".into()]);
        assert!(prompt.contains("ping"));
    }
}
