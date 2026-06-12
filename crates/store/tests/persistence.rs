use store::{IndexStatus, NewChunk, Source, SourceKind, Store};

fn src(id: &str) -> Source {
    Source {
        id: id.to_string(),
        kind: SourceKind::LocalFile,
        uri: format!("/tmp/{id}.md"),
        title: format!("t {id}"),
        content_hash: "h".to_string(),
        indexed_at: Some(123),
        status: IndexStatus::Indexed,
        error: None,
    }
}

#[test]
fn data_survives_reopen_and_is_searchable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kb.sqlite");

    {
        let store = Store::open(&path, 4).unwrap();
        store.upsert_source(&src("a")).unwrap();
        store
            .insert_chunks(
                "a",
                &[NewChunk {
                    ord: 0,
                    text: "rust vector search".into(),
                    loc: "L0".into(),
                    token_count: 3,
                    embedding: vec![1.0, 0.0, 0.0, 0.0],
                }],
            )
            .unwrap();
        store.set_meta("embedder_id", "fastembed:bge-small").unwrap();
    }

    let store = Store::open(&path, 4).unwrap();
    assert_eq!(store.list_sources().unwrap().len(), 1);
    assert_eq!(store.count_chunks().unwrap(), 1);
    assert_eq!(
        store.get_meta("embedder_id").unwrap(),
        Some("fastembed:bge-small".to_string())
    );

    let vhits = store.search_vector(&[0.9, 0.1, 0.0, 0.0], 1).unwrap();
    assert_eq!(vhits[0].text, "rust vector search");

    let fhits = store.search_fts("vector", 5).unwrap();
    assert_eq!(fhits.len(), 1);
}

#[test]
fn cascade_delete_source_removes_chunks() {
    let store = Store::open_in_memory(4).unwrap();
    store.upsert_source(&src("a")).unwrap();
    store
        .insert_chunks(
            "a",
            &[NewChunk {
                ord: 0,
                text: "to be deleted".into(),
                loc: "L0".into(),
                token_count: 3,
                embedding: vec![0.0, 1.0, 0.0, 0.0],
            }],
        )
        .unwrap();

    store.delete_chunks_for_source("a").unwrap();
    store.delete_source("a").unwrap();

    assert_eq!(store.list_sources().unwrap().len(), 0);
    assert_eq!(store.count_chunks().unwrap(), 0);
}
