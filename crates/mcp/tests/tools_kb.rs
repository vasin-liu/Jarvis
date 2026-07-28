use std::fs;
use std::sync::Arc;

use chunker::ChunkerConfig;
use embedder::MockEmbedder;
use indexer::index_path;
use mcp::{JarvisMcp, SearchParams};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use store::{IndexStatus, Source, SourceKind, Store};

fn first_text(result: &rmcp::model::CallToolResult) -> String {
    match &result.content[0] {
        ContentBlock::Text(t) => t.text.clone(),
        other => panic!("expected text block, got {other:?}"),
    }
}

fn is_tool_error(result: &rmcp::model::CallToolResult) -> bool {
    result.is_error == Some(true)
}

async fn seeded_server() -> (JarvisMcp, tempfile::TempDir) {
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

    // Non-indexed source must not appear in list_sources.
    store
        .upsert_source(&Source {
            id: "pending-only".into(),
            kind: SourceKind::LocalFile,
            uri: format!("file:///{}", dir.path().join("secret.md").display()),
            title: "Pending Doc".into(),
            content_hash: "x".into(),
            indexed_at: None,
            status: IndexStatus::Pending,
            error: None,
            summary: None,
        })
        .unwrap();

    let srv = JarvisMcp::new(Arc::new(store), Arc::new(embedder));
    (srv, dir)
}

#[tokio::test]
async fn search_returns_results_json() {
    let (srv, _dir) = seeded_server().await;
    let result = srv
        .search(Parameters(SearchParams {
            query: "vector search".into(),
            limit: None,
        }))
        .await
        .expect("ok");
    assert!(!is_tool_error(&result));
    let v: serde_json::Value = serde_json::from_str(&first_text(&result)).unwrap();
    let results = v["results"].as_array().expect("results array");
    assert!(!results.is_empty());
    let first = &results[0];
    assert!(first.get("chunk_id").is_some());
    assert!(first.get("source_id").is_some());
    assert!(first.get("title").is_some());
    assert!(first.get("loc").is_some());
    assert!(first.get("excerpt").is_some());
    assert!(first.get("uri").is_none());
    let blob = first_text(&result);
    assert!(!blob.contains("file:///"));
}

#[tokio::test]
async fn search_empty_query_is_error() {
    let (srv, _dir) = seeded_server().await;
    let result = srv
        .search(Parameters(SearchParams {
            query: "   ".into(),
            limit: None,
        }))
        .await
        .expect("handler returns Ok(error result)");
    assert!(is_tool_error(&result));
    let v: serde_json::Value = serde_json::from_str(&first_text(&result)).unwrap();
    assert_eq!(v["error"], "empty_query");
}

#[tokio::test]
async fn search_limit_clamped() {
    let (srv, _dir) = seeded_server().await;
    let result = srv
        .search(Parameters(SearchParams {
            query: "vector search".into(),
            limit: Some(100),
        }))
        .await
        .expect("ok");
    assert!(!is_tool_error(&result));
    let v: serde_json::Value = serde_json::from_str(&first_text(&result)).unwrap();
    let results = v["results"].as_array().unwrap();
    assert!(results.len() <= 8);
}

#[tokio::test]
async fn list_sources_indexed_only_no_uri() {
    let (srv, dir) = seeded_server().await;
    let result = srv.list_sources().await.expect("ok");
    assert!(!is_tool_error(&result));
    let text = first_text(&result);
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let sources = v["sources"].as_array().unwrap();
    assert!(!sources.is_empty());
    assert!(sources.iter().all(|s| s.get("id").is_some()
        && s.get("title").is_some()
        && s.get("kind").is_some()
        && s.get("uri").is_none()));
    assert!(sources.iter().all(|s| s["title"] != "Pending Doc"));
    assert!(v.get("truncated").is_some());
    assert!(v.get("total_indexed").is_some());
    let secret = dir.path().join("secret.md").display().to_string();
    assert!(!text.contains(&secret));
    assert!(!text.contains("file:///"));
}
