use embedder::Embedder;
use store::{ChunkHit, Store};

use crate::error::{Result, RetrieveError};
use crate::rrf::reciprocal_rank_fusion;

#[derive(Debug, Clone)]
pub struct RetrieverConfig {
    pub vector_k: usize,
    pub fts_k: usize,
    pub final_k: usize,
    pub rrf_k: f64,
}

impl Default for RetrieverConfig {
    fn default() -> Self {
        Self {
            vector_k: 20,
            fts_k: 20,
            final_k: 8,
            rrf_k: 60.0,
        }
    }
}

pub async fn retrieve(
    store: &Store,
    embedder: &dyn Embedder,
    query: &str,
    config: &RetrieverConfig,
) -> Result<Vec<ChunkHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }

    let query_vec = embedder.embed(&[trimmed.to_string()]).await?;
    let vector_hits = store.search_vector(&query_vec[0], config.vector_k)?;

    let fts_query = build_fts_query(trimmed);
    let fts_hits = if fts_query.is_empty() {
        Vec::new()
    } else {
        store.search_fts(&fts_query, config.fts_k)?
    };

    if vector_hits.is_empty() && fts_hits.is_empty() {
        return Ok(Vec::new());
    }

    if fts_hits.is_empty() {
        return Ok(vector_hits.into_iter().take(config.final_k).collect());
    }
    if vector_hits.is_empty() {
        return Ok(fts_hits.into_iter().take(config.final_k).collect());
    }

    Ok(reciprocal_rank_fusion(
        &[&vector_hits, &fts_hits],
        config.rrf_k,
        config.final_k,
    ))
}

fn build_fts_query(question: &str) -> String {
    let terms: Vec<String> = question
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .collect();
    terms.join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use std::fs;

    #[tokio::test]
    async fn hybrid_retrieve_finds_indexed_content() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("doc.md");
        fs::write(&file, "Rust vector search with sqlite-vec and FTS5.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let hits = retrieve(
            &store,
            &embedder,
            "vector search",
            &RetrieverConfig::default(),
        )
        .await
        .unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].text.contains("vector"));
    }

    #[test]
    fn fts_query_joins_terms() {
        assert_eq!(
            build_fts_query("rust vector"),
            "\"rust\" OR \"vector\""
        );
    }
}
