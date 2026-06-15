use chunker::ChunkerConfig;
use embedder::Embedder;
use ingest::{hash_text, Document};
use indexer::index_document;
use llm::{ChatModel, Message, Role};
use serde::Deserialize;
use store::{Source, SourceKind, Store};

use crate::error::{MemoryError, Result};

#[derive(Debug, Deserialize)]
struct MemoryDraft {
    content: String,
}

pub fn list_memories(store: &Store) -> Result<Vec<Source>> {
    Ok(store
        .list_sources()?
        .into_iter()
        .filter(|s| s.kind == SourceKind::Memory)
        .collect())
}

pub async fn add_memory(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    content: &str,
    title: Option<&str>,
) -> Result<String> {
    let text = content.trim();
    if text.is_empty() {
        return Err(MemoryError::EmptyExchange);
    }
    let now = unix_now();
    let uri = format!("memory://{now}");
    let doc = Document {
        uri: uri.clone(),
        title: title
            .filter(|t| !t.trim().is_empty())
            .map(|t| format!("记忆: {t}"))
            .unwrap_or_else(|| format!("记忆: {}", truncate_title(text))),
        text: text.to_string(),
        content_hash: hash_text(text),
    };
    index_document(store, embedder, chunker, doc, SourceKind::Memory).await?;
    Ok(uri)
}

pub async fn learn_from_exchange(
    store: &Store,
    embedder: &dyn Embedder,
    chat: &dyn ChatModel,
    chunker: &ChunkerConfig,
    question: &str,
    answer: &str,
) -> Result<Vec<String>> {
    let q = question.trim();
    let a = answer.trim();
    if q.is_empty() || a.is_empty() {
        return Err(MemoryError::EmptyExchange);
    }

    let messages = vec![
        Message {
            role: Role::System,
            content: "你是记忆提取助手。从问答对话中提取值得长期记住的事实、偏好或结论（0-3 条）。仅输出 JSON 数组，不要 markdown。每项格式：{\"content\":\"...\"}。若无值得记住的内容输出 []。".into(),
        },
        Message {
            role: Role::User,
            content: format!("问题：{q}\n\n回答：{a}"),
        },
    ];

    let raw = chat.complete(&messages).await?;
    let drafts = parse_memory_drafts(&raw)?;
    let mut ids = Vec::new();
    for draft in drafts {
        let content = draft.content.trim();
        if content.is_empty() {
            continue;
        }
        let id = add_memory(store, embedder, chunker, content, None).await?;
        ids.push(id);
    }
    Ok(ids)
}

fn parse_memory_drafts(raw: &str) -> Result<Vec<MemoryDraft>> {
    let trimmed = raw.trim();
    let json = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    serde_json::from_str(json).map_err(|e| MemoryError::InvalidMemoryJson(e.to_string()))
}

fn truncate_title(text: &str) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() <= 40 {
        one_line
    } else {
        format!("{}…", one_line.chars().take(40).collect::<String>())
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedder::MockEmbedder;
    use llm::MockChatModel;

    #[test]
    fn parse_memory_json() {
        let drafts = parse_memory_drafts(r#"[{"content":"prefers Rust"}]"#).unwrap();
        assert_eq!(drafts.len(), 1);
    }

    #[tokio::test]
    async fn learn_from_exchange_indexes_memory() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        let ids = learn_from_exchange(
            &store,
            &embedder,
            &MockChatModel,
            &chunker,
            "What is Jarvis?",
            "Jarvis is a knowledge hub.",
        )
        .await
        .unwrap();

        assert!(!ids.is_empty());
        assert_eq!(list_memories(&store).unwrap().len(), ids.len());
    }
}
