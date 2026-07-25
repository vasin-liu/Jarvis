use embedder::Embedder;
use store::{SourceKind, Store};

use crate::error::Result;

/// Neighbor source returned by overlap scoring (no raw RRF score — D-09).
#[derive(Debug, Clone, PartialEq)]
pub struct RelatedSource {
    pub source_id: String,
    pub title: String,
    pub kind: SourceKind,
    pub snippet: String,
}

/// Rank other indexed sources by hybrid retrieval overlap with the seed.
///
/// Stub for Plan 15-01 (RED): always returns an empty list. Real rollup lands in 15-02.
pub async fn related_sources(
    _store: &Store,
    _embedder: &dyn Embedder,
    _source_id: &str,
    _top_n: Option<usize>,
) -> Result<Vec<RelatedSource>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use std::collections::HashSet;
    use std::fs;
    use store::{IndexStatus, Source, StoreError};

    use crate::error::RetrieveError;

    const OVERLAP_TOKEN: &str = "overlap-scoring-alpha";
    const DISJOINT_TOKEN: &str = "zeta-unrelated-only";

    struct FixtureStore {
        _dir: tempfile::TempDir,
        store: Store,
        embedder: MockEmbedder,
        seed_id: String,
        neighbor_id: String,
        #[allow(dead_code)]
        unrelated_id: String,
    }

    /// Shared FTS keyword corpus: seed (summary), neighbor (body), disjoint control.
    async fn index_fixture_store() -> FixtureStore {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");

        let seed_path = dir.path().join("seed-alpha.md");
        fs::write(
            &seed_path,
            format!("# Alpha\n\nSeed body without the shared keyword.\n"),
        )
        .unwrap();

        let neighbor_path = dir.path().join("neighbor-alpha.md");
        fs::write(
            &neighbor_path,
            format!(
                "# Neighbor Alpha\n\nBody shares {OVERLAP_TOKEN} for FTS overlap.\n"
            ),
        )
        .unwrap();

        let unrelated_path = dir.path().join("unrelated-zeta.md");
        fs::write(
            &unrelated_path,
            format!("# Unrelated Zeta\n\nVocabulary {DISJOINT_TOKEN} only.\n"),
        )
        .unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let seed_id = index_path(&store, &embedder, &cfg, &seed_path)
            .await
            .unwrap();
        let neighbor_id = index_path(&store, &embedder, &cfg, &neighbor_path)
            .await
            .unwrap();
        let unrelated_id = index_path(&store, &embedder, &cfg, &unrelated_path)
            .await
            .unwrap();

        // Title "Alpha"; summary carries distinctive FTS token (D-01).
        let mut seed = store.get_source(&seed_id).unwrap();
        seed.title = "Alpha".to_string();
        store.upsert_source(&seed).unwrap();
        store
            .set_source_summary(&seed_id, &format!("Summary mentions {OVERLAP_TOKEN}"))
            .unwrap();

        FixtureStore {
            _dir: dir,
            store,
            embedder,
            seed_id,
            neighbor_id,
            unrelated_id,
        }
    }

    #[tokio::test]
    async fn related_excludes_seed_and_rolls_up_by_source() {
        let fx = index_fixture_store().await;
        let related = related_sources(&fx.store, &fx.embedder, &fx.seed_id, None)
            .await
            .unwrap();

        assert!(
            related.iter().all(|r| r.source_id != fx.seed_id),
            "seed source_id must be absent from results"
        );
        let ids: HashSet<_> = related.iter().map(|r| r.source_id.as_str()).collect();
        assert_eq!(ids.len(), related.len(), "at most one row per neighbor source_id");
        assert!(
            related.iter().any(|r| r.source_id == fx.neighbor_id),
            "neighbor sharing {OVERLAP_TOKEN} should appear"
        );
    }

    #[tokio::test]
    async fn related_empty_seed_text() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);

        store
            .upsert_source(&Source {
                id: "blank-seed".into(),
                kind: SourceKind::LocalFile,
                uri: "blank-seed".into(),
                title: "".into(),
                content_hash: "blank".into(),
                indexed_at: Some(0),
                status: IndexStatus::Indexed,
                error: None,
                summary: None,
            })
            .unwrap();
        store.set_source_summary("blank-seed", "   ").unwrap();

        let related = related_sources(&store, &embedder, "blank-seed", None)
            .await
            .expect("empty seed text returns Ok, not Err");
        assert!(related.is_empty());
    }

    #[tokio::test]
    async fn related_missing_source() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);

        let err = related_sources(&store, &embedder, "no-such-source-id", None)
            .await
            .expect_err("unknown source_id must return Err");
        match err {
            RetrieveError::Store(StoreError::NotFound(_)) => {}
            other => panic!("expected StoreError::NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn related_prefers_summary() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");

        let seed_path = dir.path().join("pref-seed.md");
        fs::write(&seed_path, "# TitleOnlyMismatch\n\nSeed body.\n").unwrap();

        let summary_neighbor = dir.path().join("pref-summary-hit.md");
        fs::write(
            &summary_neighbor,
            format!("# Summary Hit\n\nContains {OVERLAP_TOKEN} in body.\n"),
        )
        .unwrap();

        let title_neighbor = dir.path().join("pref-title-hit.md");
        fs::write(
            &title_neighbor,
            "# Title Hit\n\nContains TitleOnlyMismatch token only.\n",
        )
        .unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let seed_id = index_path(&store, &embedder, &cfg, &seed_path)
            .await
            .unwrap();
        let summary_id = index_path(&store, &embedder, &cfg, &summary_neighbor)
            .await
            .unwrap();
        let _title_id = index_path(&store, &embedder, &cfg, &title_neighbor)
            .await
            .unwrap();

        let mut seed = store.get_source(&seed_id).unwrap();
        seed.title = "TitleOnlyMismatch".to_string();
        store.upsert_source(&seed).unwrap();
        store
            .set_source_summary(&seed_id, &format!("prefers {OVERLAP_TOKEN}"))
            .unwrap();

        let related = related_sources(&store, &embedder, &seed_id, Some(1))
            .await
            .unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(
            related[0].source_id, summary_id,
            "summary FTS terms should rank over title-only mismatch"
        );
    }

    #[tokio::test]
    async fn related_respects_top_n() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let seed_path = dir.path().join("topn-seed.md");
        fs::write(&seed_path, "# TopN Seed\n\nSeed.\n").unwrap();
        let seed_id = index_path(&store, &embedder, &cfg, &seed_path)
            .await
            .unwrap();
        store
            .set_source_summary(&seed_id, &format!("query {OVERLAP_TOKEN}"))
            .unwrap();

        for i in 0..6 {
            let p = dir.path().join(format!("topn-neighbor-{i}.md"));
            fs::write(
                &p,
                format!("# Neighbor {i}\n\nShared {OVERLAP_TOKEN} body text {i}.\n"),
            )
            .unwrap();
            index_path(&store, &embedder, &cfg, &p).await.unwrap();
        }

        let default = related_sources(&store, &embedder, &seed_id, None)
            .await
            .unwrap();
        assert_eq!(default.len(), 5, "default top_n is 5");

        let two = related_sources(&store, &embedder, &seed_id, Some(2))
            .await
            .unwrap();
        assert_eq!(two.len(), 2);

        let zero = related_sources(&store, &embedder, &seed_id, Some(0))
            .await
            .unwrap();
        assert!(zero.is_empty());
    }

    #[tokio::test]
    async fn related_snippet_truncate() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");

        let seed_path = dir.path().join("snip-seed.md");
        fs::write(&seed_path, "# Snip Seed\n\nSeed.\n").unwrap();

        // Long CJK body so truncation is forced and UTF-8 boundary safe.
        let long_cjk: String = "知识".chars().cycle().take(200).collect();
        let neighbor_path = dir.path().join("snip-neighbor.md");
        fs::write(
            &neighbor_path,
            format!("# Snip Neighbor\n\n{OVERLAP_TOKEN} {long_cjk}\n"),
        )
        .unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let seed_id = index_path(&store, &embedder, &cfg, &seed_path)
            .await
            .unwrap();
        let neighbor_id = index_path(&store, &embedder, &cfg, &neighbor_path)
            .await
            .unwrap();
        store
            .set_source_summary(&seed_id, &format!("snip {OVERLAP_TOKEN}"))
            .unwrap();

        let related = related_sources(&store, &embedder, &seed_id, Some(1))
            .await
            .unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].source_id, neighbor_id);
        let snippet = &related[0].snippet;
        assert!(
            snippet.chars().count() <= 161,
            "snippet must be ≤161 chars (160 + ellipsis), got {}",
            snippet.chars().count()
        );
        assert!(
            snippet.ends_with('…'),
            "truncated snippet must end with U+2026"
        );
        assert!(
            std::str::from_utf8(snippet.as_bytes()).is_ok(),
            "snippet must be valid UTF-8"
        );
    }

    #[tokio::test]
    async fn related_only_seed() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("solo.md");
        fs::write(&file, "# Solo\n\nOnly one indexed source.\n").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let seed_id = index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();
        store
            .set_source_summary(&seed_id, "solo summary token unique-solo-xyz")
            .unwrap();

        let related = related_sources(&store, &embedder, &seed_id, None)
            .await
            .unwrap();
        assert!(related.is_empty());
    }

    #[tokio::test]
    async fn related_allows_wiki_neighbor() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");

        let seed_path = dir.path().join("wiki-seed.md");
        fs::write(&seed_path, "# Wiki Seed Local\n\nLocal seed.\n").unwrap();

        let wiki_path = dir.path().join("wiki-neighbor.md");
        fs::write(
            &wiki_path,
            format!("# Wiki Neighbor\n\nWiki page with {OVERLAP_TOKEN}.\n"),
        )
        .unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let cfg = ChunkerConfig::default();

        let seed_id = index_path(&store, &embedder, &cfg, &seed_path)
            .await
            .unwrap();
        let wiki_id = index_path(&store, &embedder, &cfg, &wiki_path)
            .await
            .unwrap();

        // Re-tag neighbor as WikiPage (D-10: no kind filter).
        let mut wiki = store.get_source(&wiki_id).unwrap();
        wiki.kind = SourceKind::WikiPage;
        store.upsert_source(&wiki).unwrap();

        store
            .set_source_summary(&seed_id, &format!("wiki {OVERLAP_TOKEN}"))
            .unwrap();

        let related = related_sources(&store, &embedder, &seed_id, None)
            .await
            .unwrap();
        let wiki_hit = related
            .iter()
            .find(|r| r.source_id == wiki_id)
            .expect("WikiPage neighbor must be allowed when ranked");
        assert_eq!(wiki_hit.kind, SourceKind::WikiPage);
    }
}
