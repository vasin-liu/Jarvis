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
    kind: SourceKind,
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
        kind,
        uri: doc.uri.clone(),
        title: doc.title.clone(),
        content_hash: doc.content_hash.clone(),
        indexed_at: None,
        status: IndexStatus::Pending,
        error: None,
        summary: None,
    };
    store.upsert_source(&pending)?;

    let result = async {
        store.delete_chunks_for_source(&source_id)?;

        let drafts = chunk_text(&doc.text, config);
        if drafts.is_empty() {
            return Err(IndexError::EmptyDocument(doc.uri.clone()));
        }

        let texts: Vec<String> = drafts.iter().map(|d| d.text.clone()).collect();
        let embeddings = embed_with_cache(store, embedder, &texts).await?;

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
                kind,
                uri: doc.uri,
                title: doc.title,
                content_hash: doc.content_hash,
                indexed_at: Some(unix_now()),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            };
            store.upsert_source(&indexed)?;
            Ok(())
        }
        Err(e) => {
            let failed = Source {
                id: source_id.clone(),
                kind,
                uri: doc.uri,
                title: doc.title,
                content_hash: doc.content_hash,
                indexed_at: None,
                status: IndexStatus::Failed,
                error: Some(e.to_string()),
                summary: None,
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
    index_document(store, embedder, config, doc, SourceKind::LocalFile).await?;
    Ok(source_id)
}

async fn embed_with_cache(
    store: &Store,
    embedder: &dyn Embedder,
    texts: &[String],
) -> Result<Vec<Vec<f32>>> {
    use ingest::hash_text;

    let mut results: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
    let mut to_embed: Vec<String> = Vec::new();
    let mut to_embed_idx: Vec<usize> = Vec::new();

    for (i, text) in texts.iter().enumerate() {
        let hash = hash_text(text);
        if let Some(vec) = store.get_embed_cache(&hash)? {
            if vec.len() == embedder.dim() {
                results[i] = Some(vec);
                continue;
            }
        }
        to_embed.push(text.clone());
        to_embed_idx.push(i);
    }

    if !to_embed.is_empty() {
        let fresh = embedder.embed(&to_embed).await.map_err(IndexError::Embed)?;
        for (idx, vec) in to_embed_idx.into_iter().zip(fresh) {
            let hash = hash_text(&texts[idx]);
            store.put_embed_cache(&hash, &vec)?;
            results[idx] = Some(vec);
        }
    }

    Ok(results.into_iter().map(|v| v.unwrap()).collect())
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
    async fn populates_embed_cache_while_indexing() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("cached.txt");
        fs::write(&file, "cache me once").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        index_path(&store, &embedder, &cfg, &file).await.unwrap();
        let hash = ingest::hash_text("cache me once");
        assert!(store.get_embed_cache(&hash).unwrap().is_some());
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
