mod error;

pub use error::{IndexError, Result};

use chunker::{chunk_text, ChunkerConfig};
use embedder::Embedder;
use ingest::Document;
use store::{IndexStatus, NewChunk, Source, SourceKind, Store};

pub async fn index_document(
    store: &Store,
    embedder: &dyn Embedder,
    config: &ChunkerConfig,
    doc: Document,
) -> Result<()> {
    if embedder.dim() != store.dim() {
        return Err(IndexError::DimMismatch {
            store_dim: store.dim(),
            embedder_dim: embedder.dim(),
        });
    }

    let source_id = doc.uri.clone();

    if let Ok(existing) = store.get_source(&source_id) {
        if existing.content_hash == doc.content_hash && existing.status == IndexStatus::Indexed {
            return Ok(());
        }
    }

    let pending = Source {
        id: source_id.clone(),
        kind: SourceKind::LocalFile,
        uri: doc.uri.clone(),
        title: doc.title.clone(),
        content_hash: doc.content_hash.clone(),
        indexed_at: None,
        status: IndexStatus::Pending,
        error: None,
    };
    store.upsert_source(&pending)?;

    let result = async {
        store.delete_chunks_for_source(&source_id)?;

        let drafts = chunk_text(&doc.text, config);
        if drafts.is_empty() {
            return Err(IndexError::EmptyDocument(doc.uri.clone()));
        }

        let texts: Vec<String> = drafts.iter().map(|d| d.text.clone()).collect();
        let embeddings = embedder
            .embed(&texts)
            .await
            .map_err(IndexError::Embed)?;

        let chunks: Vec<NewChunk> = drafts
            .into_iter()
            .zip(embeddings)
            .map(|(d, embedding)| NewChunk {
                ord: d.ord,
                text: d.text,
                loc: d.loc,
                token_count: d.token_count,
                embedding,
            })
            .collect();

        store.insert_chunks(&source_id, &chunks)?;
        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            let indexed = Source {
                id: source_id,
                kind: SourceKind::LocalFile,
                uri: doc.uri,
                title: doc.title,
                content_hash: doc.content_hash,
                indexed_at: Some(unix_now()),
                status: IndexStatus::Indexed,
                error: None,
            };
            store.upsert_source(&indexed)?;
            Ok(())
        }
        Err(e) => {
            let failed = Source {
                id: source_id.clone(),
                kind: SourceKind::LocalFile,
                uri: doc.uri,
                title: doc.title,
                content_hash: doc.content_hash,
                indexed_at: None,
                status: IndexStatus::Failed,
                error: Some(e.to_string()),
            };
            let _ = store.upsert_source(&failed);
            Err(e)
        }
    }
}

pub async fn index_path(
    store: &Store,
    embedder: &dyn Embedder,
    config: &ChunkerConfig,
    path: impl AsRef<std::path::Path>,
) -> Result<String> {
    let doc = ingest::load_path(path)?;
    let source_id = doc.uri.clone();
    index_document(store, embedder, config, doc).await?;
    Ok(source_id)
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
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn indexes_file_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("note.md");
        fs::write(&file, "# Hello\n\nRust knowledge base.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let id = index_path(&store, &embedder, &cfg, &file).await.unwrap();
        assert!(id.contains("note.md"));

        let source = store.get_source(&id).unwrap();
        assert_eq!(source.status, IndexStatus::Indexed);
        assert!(store.count_chunks().unwrap() >= 1);

        let hits = store.search_fts("Rust", 5).unwrap();
        assert!(!hits.is_empty());
    }

    #[tokio::test]
    async fn skips_unchanged_document() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("stable.txt");
        fs::write(&file, "same content").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        index_path(&store, &embedder, &cfg, &file).await.unwrap();
        let count_after_first = store.count_chunks().unwrap();

        index_path(&store, &embedder, &cfg, &file).await.unwrap();
        assert_eq!(store.count_chunks().unwrap(), count_after_first);
    }

    #[tokio::test]
    async fn reindexes_when_content_changes() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("mutate.txt");
        fs::write(&file, "version one").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        index_path(&store, &embedder, &cfg, &file).await.unwrap();

        {
            let mut f = fs::OpenOptions::new().write(true).open(&file).unwrap();
            f.set_len(0).unwrap();
            write!(f, "version two with more words").unwrap();
        }

        index_path(&store, &embedder, &cfg, &file).await.unwrap();
        let hits = store.search_fts("two", 5).unwrap();
        assert!(!hits.is_empty());
    }
}
