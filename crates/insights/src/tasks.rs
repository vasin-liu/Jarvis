use llm::{ChatModel, Message, Role};
use serde::Deserialize;
use store::{IndexStatus, Store, Task};

use crate::error::{InsightsError, Result};

const MAX_SOURCE_CHARS: usize = 12_000;

#[derive(Debug, Deserialize)]
struct TaskDraft {
    title: String,
    #[serde(default)]
    description: Option<String>,
}

pub async fn extract_tasks_from_source(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<Vec<Task>> {
    let source = store.get_source(source_id)?;
    if source.status != IndexStatus::Indexed {
        return Err(InsightsError::NotIndexed(source_id.to_string()));
    }

    let text = store.source_chunk_text(source_id)?;
    if text.trim().is_empty() {
        return Err(InsightsError::EmptySource(source_id.to_string()));
    }

    let excerpt = crate::truncate_chars(&text, MAX_SOURCE_CHARS);
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是任务提取助手。从正文中提取可执行的待办任务，仅输出 JSON 数组，不要 markdown 代码块。每项格式：{\"title\":\"...\",\"description\":\"...\"}。若无明确任务则输出 []。".into(),
        },
        Message {
            role: Role::User,
            content: format!("标题：{}\n\n正文：\n{}", source.title, excerpt),
        },
    ];

    let raw = chat.complete(&messages).await?;
    let drafts = parse_task_drafts(&raw)?;
    store.delete_tasks_for_source(source_id)?;

    let mut out = Vec::new();
    for draft in drafts {
        let title = draft.title.trim();
        if title.is_empty() {
            continue;
        }
        let description = draft
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());
        let task = store.insert_task(
            Some(source_id),
            title,
            description.as_deref(),
        )?;
        out.push(task);
    }
    Ok(out)
}

fn parse_task_drafts(raw: &str) -> Result<Vec<TaskDraft>> {
    let trimmed = raw.trim();
    let json = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    serde_json::from_str(json).map_err(|e| InsightsError::InvalidTasksJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm::MockChatModel;
    use store::{NewChunk, SourceKind};

    fn sample_indexed(store: &Store, id: &str) {
        store
            .upsert_source(&store::Source {
                id: id.to_string(),
                kind: SourceKind::LocalFile,
                uri: format!("/tmp/{id}.md"),
                title: format!("Title {id}"),
                content_hash: "h".into(),
                indexed_at: Some(1),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store
            .insert_chunks(
                id,
                &[NewChunk {
                    ord: 0,
                    text: "finish report and schedule review".into(),
                    loc: "L0".into(),
                    token_count: 5,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();
    }

    #[test]
    fn parse_task_json_accepts_array() {
        let drafts = parse_task_drafts(r#"[{"title":"A","description":"B"}]"#).unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].title, "A");
    }

    #[tokio::test]
    async fn extract_tasks_replaces_previous() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "a");

        let first = extract_tasks_from_source(&store, &MockChatModel, "a")
            .await
            .unwrap();
        assert!(!first.is_empty());

        let second = extract_tasks_from_source(&store, &MockChatModel, "a")
            .await
            .unwrap();
        assert_eq!(first.len(), second.len());
        assert_eq!(store.list_tasks().unwrap().len(), second.len());
    }
}
