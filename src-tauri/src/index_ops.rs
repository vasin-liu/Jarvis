use chunker::ChunkerConfig;
use config::AppConfig;
use embedder::Embedder;
use indexer::{index_document, index_path};
use lark::{fetch_doc, fetch_im_chat, fetch_mail, fetch_sheet, CommandRunner};
use store::{IndexStatus, Source, SourceKind, Store};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RebuildReport {
    pub indexed: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStatusView {
    pub store_dim: usize,
    pub config_embed_dim: usize,
    pub stored_embedder_id: Option<String>,
    pub config_embedder_id: String,
    pub needs_rebuild: bool,
    pub dim_mismatch: bool,
    pub source_count: usize,
    pub chunk_count: usize,
}

pub fn index_status_view(store: &Store, config: &AppConfig, embedder: &dyn Embedder) -> IndexStatusView {
    let health = store.index_health().unwrap_or(store::IndexHealth {
        store_dim: store.dim(),
        stored_embedder_id: None,
        source_count: 0,
        chunk_count: 0,
    });
    let config_embedder_id = embedder.id().to_string();
    let config_embed_dim = config.embedding_dim();
    let dim_mismatch = health.store_dim != config_embed_dim;
    let needs_rebuild = dim_mismatch
        || health
            .stored_embedder_id
            .as_deref()
            .is_some_and(|id| id != config_embedder_id)
        || health.chunk_count == 0 && health.source_count > 0;

    IndexStatusView {
        store_dim: health.store_dim,
        config_embed_dim,
        stored_embedder_id: health.stored_embedder_id,
        config_embedder_id,
        needs_rebuild,
        dim_mismatch,
        source_count: health.source_count,
        chunk_count: health.chunk_count,
    }
}

pub async fn rebuild_all_sources(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    runner: &dyn CommandRunner,
    lark_cli_bin: &str,
) -> Result<RebuildReport, String> {
    let sources = store.list_sources().map_err(|e| e.to_string())?;
    let mut indexed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for source in sources {
        match reindex_source(store, embedder, chunker, runner, lark_cli_bin, &source).await {
            Ok(true) => indexed += 1,
            Ok(false) => skipped += 1,
            Err(_) => failed += 1,
        }
    }

    store
        .set_meta("embedder_id", embedder.id())
        .map_err(|e| e.to_string())?;

    Ok(RebuildReport {
        indexed,
        failed,
        skipped,
    })
}

async fn reindex_source(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    runner: &dyn CommandRunner,
    lark_cli_bin: &str,
    source: &Source,
) -> Result<bool, String> {
    store
        .delete_chunks_for_source(&source.id)
        .map_err(|e| e.to_string())?;

    match source.kind {
        SourceKind::LocalFile => {
            if !std::path::Path::new(&source.uri).is_file() {
                mark_failed(store, source, "local file missing")?;
                return Ok(false);
            }
            index_path(store, embedder, chunker, &source.uri)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        SourceKind::LarkDoc => {
            let token = lark_suffix(&source.uri, "lark://doc/")?;
            let doc = fetch_doc(runner, lark_cli_bin, &token).map_err(|e| e.to_string())?;
            index_document(store, embedder, chunker, doc, SourceKind::LarkDoc)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        SourceKind::LarkSheet => {
            let token = lark_suffix(&source.uri, "lark://sheet/")?;
            let doc = fetch_sheet(runner, lark_cli_bin, &token).map_err(|e| e.to_string())?;
            index_document(store, embedder, chunker, doc, SourceKind::LarkSheet)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        SourceKind::LarkMail => {
            let id = lark_suffix(&source.uri, "lark://mail/")?;
            let doc = fetch_mail(runner, lark_cli_bin, &id).map_err(|e| e.to_string())?;
            index_document(store, embedder, chunker, doc, SourceKind::LarkMail)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        SourceKind::LarkMsg => {
            let id = lark_suffix(&source.uri, "lark://im/")?;
            let doc = fetch_im_chat(runner, lark_cli_bin, &id).map_err(|e| e.to_string())?;
            index_document(store, embedder, chunker, doc, SourceKind::LarkMsg)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
    }
}

fn lark_suffix(uri: &str, prefix: &str) -> Result<String, String> {
    uri.strip_prefix(prefix)
        .map(str::to_string)
        .ok_or_else(|| format!("invalid lark uri: {uri}"))
}

fn mark_failed(store: &Store, source: &Source, error: &str) -> Result<(), String> {
    let failed = Source {
        id: source.id.clone(),
        kind: source.kind,
        uri: source.uri.clone(),
        title: source.title.clone(),
        content_hash: source.content_hash.clone(),
        indexed_at: None,
        status: IndexStatus::Failed,
        error: Some(error.to_string()),
    };
    store.upsert_source(&failed).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedder::MockEmbedder;
    use lark::FakeRunner;
    use std::fs;

    #[tokio::test]
    async fn rebuild_local_file_source() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("note.md");
        fs::write(&file, "rebuild me").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();
        let runner = FakeRunner::new();

        index_path(&store, &embedder, &chunker, &file)
            .await
            .unwrap();
        store.reinit_vectors(4).unwrap();

        let report = rebuild_all_sources(&store, &embedder, &chunker, &runner, "lark-cli")
            .await
            .unwrap();
        assert_eq!(report.indexed, 1);
        assert!(store.count_chunks().unwrap() > 0);
    }
}
