use embedder::Embedder;
use llm::{ChatModel, Message, Role};
use retriever::{retrieve, RetrieverConfig};
use store::{ChunkHit, Store};

use crate::error::Result;
use crate::types::{AskResponse, Citation};

const NO_CONTEXT: &str = "知识库中未找到相关内容。";

pub async fn ask(
    store: &Store,
    embedder: &dyn Embedder,
    chat: &dyn ChatModel,
    retriever: &RetrieverConfig,
    question: &str,
) -> Result<AskResponse> {
    let hits = retrieve(store, embedder, question, retriever).await?;

    if hits.is_empty() {
        return Ok(AskResponse {
            answer: NO_CONTEXT.to_string(),
            citations: vec![],
            tool_calls: vec![],
        });
    }

    let citations = hits_to_citations(store, &hits)?;
    let messages = build_messages(question, &hits, &citations);

    let answer = chat.complete(&messages).await?;
    Ok(AskResponse {
        answer,
        citations,
        tool_calls: vec![],
    })
}

pub async fn ask_stream(
    store: &Store,
    embedder: &dyn Embedder,
    chat: &dyn ChatModel,
    retriever: &RetrieverConfig,
    question: &str,
    on_token: &mut (dyn FnMut(String) + Send),
) -> Result<AskResponse> {
    let hits = retrieve(store, embedder, question, retriever).await?;

    if hits.is_empty() {
        let answer = NO_CONTEXT.to_string();
        on_token(answer.clone());
        return Ok(AskResponse {
            answer,
            citations: vec![],
            tool_calls: vec![],
        });
    }

    let citations = hits_to_citations(store, &hits)?;
    let messages = build_messages(question, &hits, &citations);
    let answer = chat.complete_stream(&messages, on_token).await?;
    Ok(AskResponse {
        answer,
        citations,
        tool_calls: vec![],
    })
}

fn hits_to_citations(store: &Store, hits: &[ChunkHit]) -> Result<Vec<Citation>> {
    let mut out = Vec::with_capacity(hits.len());
    for hit in hits {
        let source = store.get_source(&hit.source_id)?;
        let excerpt = if hit.text.len() > 240 {
            format!("{}…", &hit.text[..240])
        } else {
            hit.text.clone()
        };
        out.push(Citation {
            chunk_id: hit.chunk_id,
            source_id: hit.source_id.clone(),
            source_title: source.title,
            source_uri: source.uri,
            loc: hit.loc.clone(),
            excerpt,
        });
    }
    Ok(out)
}

fn build_messages(question: &str, hits: &[ChunkHit], citations: &[Citation]) -> Vec<Message> {
    let mut context = String::from("以下是从知识库检索到的片段，请仅依据它们回答。\n\n");
    for (i, hit) in hits.iter().enumerate() {
        let title = citations
            .get(i)
            .map(|c| c.source_title.as_str())
            .unwrap_or("source");
        context.push_str(&format!(
            "[{}] {} · {} · {}\n{}\n\n",
            i + 1,
            title,
            hit.loc,
            citations.get(i).map(|c| c.source_uri.as_str()).unwrap_or(""),
            hit.text
        ));
    }

    vec![
        Message {
            role: Role::System,
            content: context,
        },
        Message {
            role: Role::User,
            content: question.to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ask_stream;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use llm::MockChatModel;
    use std::fs;

    #[tokio::test]
    async fn ask_returns_citations_for_indexed_doc() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("kb.md");
        fs::write(&file, "Jarvis uses sqlite-vec for embeddings.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let resp = ask(
            &store,
            &embedder,
            &MockChatModel,
            &RetrieverConfig::default(),
            "sqlite-vec embeddings",
        )
        .await
        .unwrap();

        assert!(!resp.citations.is_empty());
        assert!(!resp.answer.is_empty());
    }

    #[tokio::test]
    async fn ask_stream_emits_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("kb.md");
        fs::write(&file, "Jarvis streams answers token by token.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let mut streamed = String::new();
        let resp = ask_stream(
            &store,
            &embedder,
            &MockChatModel,
            &RetrieverConfig::default(),
            "streams answers",
            &mut |t| streamed.push_str(&t),
        )
        .await
        .unwrap();

        assert!(!streamed.is_empty());
        assert_eq!(streamed, resp.answer);
    }

    #[tokio::test]
    async fn ask_without_hits_returns_no_context() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);

        let resp = ask(
            &store,
            &embedder,
            &MockChatModel,
            &RetrieverConfig::default(),
            "anything",
        )
        .await
        .unwrap();

        assert_eq!(resp.answer, NO_CONTEXT);
        assert!(resp.citations.is_empty());
    }
}
