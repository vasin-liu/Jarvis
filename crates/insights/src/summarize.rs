use llm::{ChatModel, Message, Role};
use store::{IndexStatus, Store};

use crate::error::{InsightsError, Result};

const MAX_SOURCE_CHARS: usize = 12_000;

pub async fn summarize_source(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<String> {
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
            content: "你是知识库助手。请根据原文生成简洁的中文摘要（3-6 句），只写原文中已有的信息，不要编造。".into(),
        },
        Message {
            role: Role::User,
            content: format!("标题：{}\n\n正文：\n{}", source.title, excerpt),
        },
    ];

    let summary = chat.complete(&messages).await?;
    store.set_source_summary(source_id, &summary)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm::MockChatModel;
    use store::{NewChunk, SourceKind};

    fn sample_indexed(store: &Store, id: &str, text: &str) {
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
                    text: text.into(),
                    loc: "L0".into(),
                    token_count: 3,
                    embedding: vec![0.1; 4],
                }],
            )
            .unwrap();
    }

    #[tokio::test]
    async fn summarize_persists_summary() {
        let store = Store::open_in_memory(4).unwrap();
        sample_indexed(&store, "a", "Rust async runtime design notes");

        let summary = summarize_source(&store, &MockChatModel, "a")
            .await
            .unwrap();
        assert!(!summary.is_empty());
        assert_eq!(
            store.get_source("a").unwrap().summary.as_deref(),
            Some(summary.as_str())
        );
    }
}
