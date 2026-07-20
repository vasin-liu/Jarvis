use memory::migrate_legacy_memory_uris;
use store::{IndexStatus, NewChunk, Source, SourceKind, Store};

#[test]
fn migrate_legacy_memory_uris_is_idempotent() {
    let store = Store::open_in_memory(4).unwrap();
    let old_uri = "memory://12345";
    store
        .upsert_source(&Source {
            id: old_uri.to_string(),
            kind: SourceKind::Memory,
            uri: old_uri.to_string(),
            title: "记忆: legacy".to_string(),
            content_hash: "h0".to_string(),
            indexed_at: Some(1),
            status: IndexStatus::Indexed,
            error: None,
            summary: None,
        })
        .unwrap();
    store
        .insert_chunks(
            old_uri,
            &[NewChunk {
                ord: 0,
                text: "legacy memory content".to_string(),
                loc: "L0".to_string(),
                token_count: 3,
                embedding: vec![1.0, 0.0, 0.0, 0.0],
            }],
        )
        .unwrap();

    let first = migrate_legacy_memory_uris(&store).unwrap();
    assert_eq!(first, 1);

    let sources = store.list_sources().unwrap();
    assert_eq!(sources.len(), 1);
    assert!(memory::is_uuid_memory_uri(&sources[0].uri));
    assert!(store
        .source_chunk_text(&sources[0].id)
        .unwrap()
        .contains("legacy memory content"));
    assert!(store.get_source(old_uri).is_err());

    let second = migrate_legacy_memory_uris(&store).unwrap();
    assert_eq!(second, 0);
}
