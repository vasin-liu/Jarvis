use std::path::Path;

use chunker::ChunkerConfig;
use config::AppConfig;
use cursor::{discover_transcripts, load_transcript, resolve_transcript_path};
use embedder::Embedder;
use llm::ChatModel;
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
#[serde(rename_all = "camelCase")]
pub struct IndexProgressEvent {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub source_title: String,
    pub outcome: Option<String>,
    pub message: Option<String>,
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

pub async fn rebuild_all_sources<F>(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    runner: &dyn CommandRunner,
    lark_cli_bin: &str,
    cursor_projects_root: &str,
    phase: &str,
    mut on_progress: F,
) -> Result<RebuildReport, String>
where
    F: FnMut(IndexProgressEvent),
{
    let sources = store.list_sources().map_err(|e| e.to_string())?;
    let total = sources.len();
    let mut indexed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (i, source) in sources.into_iter().enumerate() {
        let current = i + 1;
        on_progress(IndexProgressEvent {
            phase: phase.to_string(),
            current,
            total,
            source_title: source.title.clone(),
            outcome: None,
            message: None,
        });

        match reindex_source(
            store,
            embedder,
            chunker,
            runner,
            lark_cli_bin,
            cursor_projects_root,
            &source,
        )
        .await
        {
            Ok(true) => {
                indexed += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: source.title.clone(),
                    outcome: Some("indexed".into()),
                    message: None,
                });
            }
            Ok(false) => {
                skipped += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: source.title.clone(),
                    outcome: Some("skipped".into()),
                    message: source.error.clone(),
                });
            }
            Err(e) => {
                failed += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: source.title.clone(),
                    outcome: Some("failed".into()),
                    message: Some(e),
                });
            }
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

pub async fn index_local_paths<F>(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    phase: &str,
    paths: Vec<std::path::PathBuf>,
    insights: Option<(&dyn ChatModel, &AppConfig)>,
    mut on_progress: F,
) -> Result<RebuildReport, String>
where
    F: FnMut(IndexProgressEvent),
{
    let total = paths.len();
    let mut indexed = 0;
    let mut failed = 0;

    for (i, path) in paths.into_iter().enumerate() {
        let current = i + 1;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        on_progress(IndexProgressEvent {
            phase: phase.to_string(),
            current,
            total,
            source_title: title.clone(),
            outcome: None,
            message: None,
        });

        match index_path(store, embedder, chunker, &path).await {
            Ok(source_id) => {
                if let Some((chat, cfg)) = insights {
                    crate::insights_ops::maybe_run_insights_for_source(
                        store, chat, cfg, &source_id,
                    )
                    .await;
                }
                indexed += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: title,
                    outcome: Some("indexed".into()),
                    message: None,
                });
            }
            Err(e) => {
                failed += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: title,
                    outcome: Some("failed".into()),
                    message: Some(e.to_string()),
                });
            }
        }
    }

    if indexed > 0 {
        store
            .set_meta("embedder_id", embedder.id())
            .map_err(|e| e.to_string())?;
    }

    Ok(RebuildReport {
        indexed,
        failed,
        skipped: 0,
    })
}

pub async fn sync_cursor_transcripts<F>(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    cursor_projects_root: &str,
    insights: Option<(&dyn ChatModel, &AppConfig)>,
    mut on_progress: F,
) -> Result<RebuildReport, String>
where
    F: FnMut(IndexProgressEvent),
{
    if cursor_projects_root.is_empty() {
        return Err("cursor projects root is not configured".into());
    }

    let paths = discover_transcripts(Path::new(cursor_projects_root)).map_err(|e| e.to_string())?;
    let total = paths.len();
    let mut indexed = 0;
    let mut failed = 0;

    for (i, path) in paths.into_iter().enumerate() {
        let current = i + 1;
        let title = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("cursor-session")
            .to_string();

        on_progress(IndexProgressEvent {
            phase: "cursor".into(),
            current,
            total,
            source_title: title.clone(),
            outcome: None,
            message: None,
        });

        match load_transcript(&path).map_err(|e| e.to_string()) {
            Ok(doc) => {
                let source_id = doc.uri.clone();
                match index_document(
                    store,
                    embedder,
                    chunker,
                    doc,
                    SourceKind::CursorTranscript,
                )
                .await
                {
                    Ok(()) => {
                        if let Some((chat, cfg)) = insights {
                            crate::insights_ops::maybe_run_insights_for_source(
                                store,
                                chat,
                                cfg,
                                &source_id,
                            )
                            .await;
                        }
                        indexed += 1;
                        on_progress(IndexProgressEvent {
                            phase: "cursor".into(),
                            current,
                            total,
                            source_title: title,
                            outcome: Some("indexed".into()),
                            message: None,
                        });
                    }
                    Err(e) => {
                        failed += 1;
                        on_progress(IndexProgressEvent {
                            phase: "cursor".into(),
                            current,
                            total,
                            source_title: title,
                            outcome: Some("failed".into()),
                            message: Some(e.to_string()),
                        });
                    }
                }
            }
            Err(e) => {
                failed += 1;
                on_progress(IndexProgressEvent {
                    phase: "cursor".into(),
                    current,
                    total,
                    source_title: title,
                    outcome: Some("failed".into()),
                    message: Some(e),
                });
            }
        }
    }

    if indexed > 0 {
        store
            .set_meta("embedder_id", embedder.id())
            .map_err(|e| e.to_string())?;
    }

    Ok(RebuildReport {
        indexed,
        failed,
        skipped: 0,
    })
}

pub async fn retry_source_by_id<F>(
    store: &Store,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    runner: &dyn CommandRunner,
    lark_cli_bin: &str,
    cursor_projects_root: &str,
    source_id: &str,
    mut on_progress: F,
) -> Result<RebuildReport, String>
where
    F: FnMut(IndexProgressEvent),
{
    let source = store
        .get_source(source_id)
        .map_err(|e| e.to_string())?;

    on_progress(IndexProgressEvent {
        phase: "retry".into(),
        current: 1,
        total: 1,
        source_title: source.title.clone(),
        outcome: None,
        message: None,
    });

    let (indexed, failed, skipped) = match reindex_source(
        store,
        embedder,
        chunker,
        runner,
        lark_cli_bin,
        cursor_projects_root,
        &source,
    )
    .await
    {
        Ok(true) => {
            on_progress(IndexProgressEvent {
                phase: "retry".into(),
                current: 1,
                total: 1,
                source_title: source.title.clone(),
                outcome: Some("indexed".into()),
                message: None,
            });
            (1, 0, 0)
        }
        Ok(false) => {
            on_progress(IndexProgressEvent {
                phase: "retry".into(),
                current: 1,
                total: 1,
                source_title: source.title.clone(),
                outcome: Some("skipped".into()),
                message: source.error.clone(),
            });
            (0, 0, 1)
        }
        Err(e) => {
            on_progress(IndexProgressEvent {
                phase: "retry".into(),
                current: 1,
                total: 1,
                source_title: source.title.clone(),
                outcome: Some("failed".into()),
                message: Some(e),
            });
            (0, 1, 0)
        }
    };

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
    cursor_projects_root: &str,
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
        SourceKind::CursorTranscript => {
            if cursor_projects_root.is_empty() {
                mark_failed(store, source, "cursor projects root not configured")?;
                return Ok(false);
            }
            let path = resolve_transcript_path(Path::new(cursor_projects_root), &source.uri)
                .ok_or_else(|| format!("transcript not found for {}", source.uri))?;
            let doc = load_transcript(&path).map_err(|e| e.to_string())?;
            index_document(store, embedder, chunker, doc, SourceKind::CursorTranscript)
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
        summary: source.summary.clone(),
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
    async fn index_local_paths_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("scan-me.md");
        fs::write(&file, "folder scan progress").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();
        let mut events = Vec::new();

        let report = index_local_paths(
            &store,
            &embedder,
            &chunker,
            "scan",
            vec![file],
            None,
            |event| events.push(event),
        )
        .await
        .unwrap();

        assert_eq!(report.indexed, 1);
        assert!(events.iter().any(|e| e.outcome.as_deref() == Some("indexed")));
    }

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

        let report = rebuild_all_sources(
            &store,
            &embedder,
            &chunker,
            &runner,
            "lark-cli",
            "",
            "rebuild",
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(report.indexed, 1);
        assert!(store.count_chunks().unwrap() > 0);
    }

    fn write_cursor_transcript(root: &Path, project: &str, session_id: &str) {
        let dir = root
            .join(project)
            .join("agent-transcripts")
            .join(session_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{session_id}.jsonl")),
            r#"{"role":"user","message":{"content":"cursor sync test"}}"#,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn sync_cursor_transcripts_indexes_discovered_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let cursor_root = dir.path().join("cursor-projects");
        write_cursor_transcript(&cursor_root, "jarvis", "sess-1");
        write_cursor_transcript(&cursor_root, "other", "sess-2");

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        let chunker = ChunkerConfig::default();
        let mut events = Vec::new();

        let report = sync_cursor_transcripts(
            &store,
            &embedder,
            &chunker,
            cursor_root.to_str().unwrap(),
            None,
            |event| events.push(event),
        )
        .await
        .unwrap();

        assert_eq!(report.indexed, 2);
        assert_eq!(report.failed, 0);
        assert!(events.iter().any(|e| e.phase == "cursor"));
        assert_eq!(store.list_sources().unwrap().len(), 2);
    }
}
