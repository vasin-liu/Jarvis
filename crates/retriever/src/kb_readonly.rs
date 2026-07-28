use embedder::Embedder;
use serde::Serialize;
use store::{IndexStatus, Store};

use crate::error::Result;
use crate::retrieve::{retrieve, RetrieverConfig};

/// Max Unicode chars in a search excerpt (agent citation parity — D-05).
pub const EXCERPT_MAX_CHARS: usize = 200;

/// Hard cap for Indexed source inventory (D-07).
pub const LIST_SOURCES_MAX: usize = 200;

/// One hybrid search hit with truncated excerpt (no full path / no score).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KbHit {
    pub chunk_id: i64,
    pub source_id: String,
    pub title: String,
    pub loc: String,
    pub excerpt: String,
}

/// Indexed source row for inventory — never includes `uri` (D-08).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KbSourceEntry {
    pub id: String,
    pub title: String,
    pub kind: String,
}

/// Bounded Indexed source list with truncation metadata (D-07).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KbSourceList {
    pub sources: Vec<KbSourceEntry>,
    pub truncated: bool,
    pub total_indexed: usize,
}

/// Char-safe truncation with U+2026 when over limit (D-05).
pub fn truncate_excerpt(text: &str) -> String {
    if text.chars().count() <= EXCERPT_MAX_CHARS {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(EXCERPT_MAX_CHARS).collect();
        format!("{truncated}…")
    }
}

/// Hybrid KB search shared by agent + MCP (D-03, D-04, MCP-01).
pub async fn search_kb(
    store: &Store,
    embedder: &dyn Embedder,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<KbHit>> {
    let default = RetrieverConfig::default();
    let mut config = default.clone();
    if let Some(n) = limit {
        config.final_k = n.clamp(1, default.final_k);
    }

    let hits = retrieve(store, embedder, query, &config).await?;
    let mut out = Vec::with_capacity(hits.len());
    for hit in hits {
        let source = store.get_source(&hit.source_id)?;
        out.push(KbHit {
            chunk_id: hit.chunk_id,
            source_id: hit.source_id,
            title: source.title,
            loc: hit.loc,
            excerpt: truncate_excerpt(&hit.text),
        });
    }
    Ok(out)
}

/// Indexed-only source inventory with hard cap (D-06, D-07, D-08, MCP-02).
pub fn list_indexed_sources(store: &Store) -> Result<KbSourceList> {
    let indexed: Vec<_> = store
        .list_sources()?
        .into_iter()
        .filter(|s| s.status == IndexStatus::Indexed)
        .collect();
    let total_indexed = indexed.len();
    let truncated = total_indexed > LIST_SOURCES_MAX;
    let sources = indexed
        .into_iter()
        .take(LIST_SOURCES_MAX)
        .map(|s| KbSourceEntry {
            id: s.id,
            title: s.title,
            kind: s.kind.as_str().to_string(),
        })
        .collect();
    Ok(KbSourceList {
        sources,
        truncated,
        total_indexed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use std::fs;
    use store::{IndexStatus, Source, SourceKind};

    fn sample_source(id: &str, status: IndexStatus, title: &str) -> Source {
        Source {
            id: id.to_string(),
            kind: SourceKind::LocalFile,
            uri: format!("file:///secret/path/{id}.md"),
            title: title.to_string(),
            content_hash: "h".into(),
            indexed_at: Some(1),
            status,
            error: None,
            summary: None,
        }
    }

    #[tokio::test]
    async fn search_kb_uses_default_final_k() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("doc.md");
        // Long enough content that hybrid can return multiple chunks if k were huge.
        let body = (0..40)
            .map(|i| format!("Rust vector search chunk {i} with sqlite-vec and FTS5."))
            .collect::<Vec<_>>()
            .join("\n\n");
        fs::write(&file, body).unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let hits = search_kb(&store, &embedder, "vector search", None)
            .await
            .unwrap();
        assert!(hits.len() <= RetrieverConfig::default().final_k);
        assert!(!hits.is_empty());
    }

    #[tokio::test]
    async fn search_kb_clamps_limit_to_final_k() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("doc.md");
        let body = (0..40)
            .map(|i| format!("Rust vector search chunk {i} with sqlite-vec and FTS5."))
            .collect::<Vec<_>>()
            .join("\n\n");
        fs::write(&file, body).unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let hits = search_kb(&store, &embedder, "vector search", Some(100))
            .await
            .unwrap();
        assert!(hits.len() <= RetrieverConfig::default().final_k);
    }

    #[tokio::test]
    async fn search_kb_empty_query_errors() {
        let store = Store::open_in_memory(4).unwrap();
        let embedder = MockEmbedder::new(4);
        let err = search_kb(&store, &embedder, "   ", None)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::RetrieveError::EmptyQuery));
    }

    #[tokio::test]
    async fn search_kb_excerpt_max_200_chars() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("doc.md");
        let long = "keyword ".repeat(80);
        fs::write(&file, format!("unique-xyzzy-token {long}")).unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let hits = search_kb(&store, &embedder, "unique-xyzzy-token", None)
            .await
            .unwrap();
        assert!(!hits.is_empty());
        for hit in hits {
            assert!(
                hit.excerpt.chars().count() <= EXCERPT_MAX_CHARS + 1,
                "excerpt too long: {}",
                hit.excerpt.chars().count()
            );
            // +1 allows the ellipsis character after 200 content chars
            if hit.excerpt.chars().count() > EXCERPT_MAX_CHARS {
                assert!(hit.excerpt.ends_with('…'));
                assert_eq!(hit.excerpt.chars().count(), EXCERPT_MAX_CHARS + 1);
            }
        }
    }

    #[test]
    fn list_indexed_sources_filters_status() {
        let store = Store::open_in_memory(4).unwrap();
        store
            .upsert_source(&sample_source("a", IndexStatus::Indexed, "A"))
            .unwrap();
        store
            .upsert_source(&sample_source("b", IndexStatus::Pending, "B"))
            .unwrap();
        store
            .upsert_source(&sample_source("c", IndexStatus::Failed, "C"))
            .unwrap();

        let list = list_indexed_sources(&store).unwrap();
        assert_eq!(list.total_indexed, 1);
        assert_eq!(list.sources.len(), 1);
        assert_eq!(list.sources[0].id, "a");
        assert!(!list.truncated);
    }

    #[test]
    fn list_indexed_sources_cap_200() {
        let store = Store::open_in_memory(4).unwrap();
        for i in 0..205 {
            store
                .upsert_source(&sample_source(
                    &format!("s{i:03}"),
                    IndexStatus::Indexed,
                    &format!("T{i}"),
                ))
                .unwrap();
        }
        let list = list_indexed_sources(&store).unwrap();
        assert_eq!(list.total_indexed, 205);
        assert!(list.truncated);
        assert_eq!(list.sources.len(), LIST_SOURCES_MAX);
    }

    #[test]
    fn list_indexed_sources_fields_omit_uri() {
        let store = Store::open_in_memory(4).unwrap();
        store
            .upsert_source(&sample_source("x", IndexStatus::Indexed, "Title"))
            .unwrap();
        let list = list_indexed_sources(&store).unwrap();
        let json = serde_json::to_value(&list.sources[0]).unwrap();
        assert!(json.get("uri").is_none());
        assert_eq!(json["id"], "x");
        assert_eq!(json["title"], "Title");
        assert_eq!(json["kind"], "local_file");
    }

    #[test]
    fn truncate_excerpt_uses_chars_not_bytes() {
        let s = "测".repeat(250);
        let t = truncate_excerpt(&s);
        assert_eq!(t.chars().count(), EXCERPT_MAX_CHARS + 1);
        assert!(t.ends_with('…'));
    }
}
