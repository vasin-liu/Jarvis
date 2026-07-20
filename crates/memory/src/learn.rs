use chunker::ChunkerConfig;
use embedder::Embedder;
use ingest::{hash_text, Document};
use indexer::index_document;
use llm::{ChatModel, Message, Role};
use serde::Deserialize;
use store::{Source, SourceKind, Store};
use uuid::Uuid;

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

pub fn get_memory_content(store: &Store, source_id: &str) -> Result<String> {
    let source = store.get_source(source_id)?;
    if source.kind != SourceKind::Memory {
        return Err(MemoryError::NotMemory(source_id.to_string()));
    }
    store.source_chunk_text(source_id).map_err(MemoryError::from)
}

pub fn resolve_memory_id(store: &Store, id_or_title: &str) -> Result<String> {
    let key = id_or_title.trim();
    if key.is_empty() {
        return Err(MemoryError::NotFound(id_or_title.to_string()));
    }
    if store.get_source(key).is_ok() {
        return Ok(key.to_string());
    }
    let memories = list_memories(store)?;
    if let Some(m) = memories.iter().find(|m| m.uri == key) {
        return Ok(m.id.clone());
    }
    if let Some(m) = memories.iter().find(|m| m.title == key) {
        return Ok(m.id.clone());
    }
    if let Some(m) = memories
        .iter()
        .find(|m| m.title.contains(key) || m.uri.contains(key))
    {
        eprintln!("[jarvis] memory title fuzzy match deprecated; use id or memory:// uri");
        return Ok(m.id.clone());
    }
    Err(MemoryError::NotFound(id_or_title.to_string()))
}

pub fn forget_memory(store: &Store, id_or_title: &str) -> Result<()> {
    let id = resolve_memory_id(store, id_or_title)?;
    let source = store.get_source(&id)?;
    if source.kind != SourceKind::Memory {
        return Err(MemoryError::NotMemory(id));
    }
    store.delete_source(&id)?;
    Ok(())
}

pub async fn update_memory(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    id_or_title: &str,
    content: &str,
    title: Option<&str>,
) -> Result<String> {
    let id = resolve_memory_id(store, id_or_title)?;
    let source = store.get_source(&id)?;
    if source.kind != SourceKind::Memory {
        return Err(MemoryError::NotMemory(id));
    }
    let text = content.trim();
    if text.is_empty() {
        return Err(MemoryError::EmptyExchange);
    }
    let doc_title = title
        .filter(|t| !t.trim().is_empty())
        .map(|t| format!("记忆: {t}"))
        .unwrap_or_else(|| format!("记忆: {}", truncate_title(text)));
    let doc = Document {
        uri: source.uri.clone(),
        title: doc_title,
        text: text.to_string(),
        content_hash: hash_text(text),
    };
    index_document(store, embedder, chunker, doc, SourceKind::Memory).await?;
    Ok(id)
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
    let uri = format!("memory://{}", Uuid::new_v4());
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
    async fn add_memory_uses_uuid_uri() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        let uri = add_memory(&store, &embedder, &chunker, "prefers tea", None)
            .await
            .unwrap();
        assert!(uri.starts_with("memory://"));
        let suffix = uri.strip_prefix("memory://").unwrap();
        assert!(Uuid::parse_str(suffix).is_ok());
    }

    #[tokio::test]
    async fn resolve_memory_by_exact_uri() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        let uri = add_memory(&store, &embedder, &chunker, "exact match body", None)
            .await
            .unwrap();
        let resolved = resolve_memory_id(&store, &uri).unwrap();
        assert_eq!(resolved, uri);
    }

    #[test]
    fn resolve_memory_fuzzy_still_works() {
        let store = Store::open_in_memory(4).unwrap();
        let source = Source {
            id: "memory://550e8400-e29b-41d4-a716-446655440000".to_string(),
            kind: SourceKind::Memory,
            uri: "memory://550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "记忆: dark mode preference".to_string(),
            content_hash: "h".to_string(),
            indexed_at: None,
            status: store::IndexStatus::Indexed,
            error: None,
            summary: None,
        };
        store.upsert_source(&source).unwrap();
        let id = resolve_memory_id(&store, "dark mode").unwrap();
        assert_eq!(id, source.id);
    }

    #[tokio::test]
    async fn forget_and_update_memory() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();

        let id = add_memory(&store, &embedder, &chunker, "likes tea", Some("pref"))
            .await
            .unwrap();
        assert_eq!(list_memories(&store).unwrap().len(), 1);
        assert!(get_memory_content(&store, &id).unwrap().contains("tea"));

        update_memory(
            &store,
            &embedder,
            &chunker,
            &id,
            "likes coffee",
            Some("pref"),
        )
        .await
        .unwrap();
        assert!(get_memory_content(&store, &id).unwrap().contains("coffee"));

        forget_memory(&store, &id).unwrap();
        assert!(list_memories(&store).unwrap().is_empty());
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
