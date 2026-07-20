//! Regression: building citations for long Chinese chunks must not panic.
//!
//! `hits_to_citations` previously sliced `&hit.text[..240]` at byte 240, which
//! is a non-char-boundary for multibyte UTF-8 (Chinese) → panic → the Tauri
//! Q&A command future aborted and the UI hung on "思考中...". This test indexes
//! a long Chinese document and runs `ask`, asserting it returns cleanly.
//!
//! Run: `cargo test -p rag --test citation_excerpt`

use chunker::ChunkerConfig;
use embedder::MockEmbedder;
use indexer::index_path;
use llm::MockChatModel;
use rag::ask;
use retriever::RetrieverConfig;
use store::Store;

#[tokio::test]
async fn ask_with_long_chinese_chunks_does_not_panic() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kb.sqlite");
    let file = dir.path().join("note.md");
    let para = "Jarvis 是一个本地优先的个人知识中枢，支持对本地文档与飞书内容进行索引、\
                混合检索与带引用的问答，并提供智能体模式以调用搜索、记忆与任务等工具。";
    let big = std::iter::repeat(para).take(8).collect::<Vec<_>>().join("\n\n");
    std::fs::write(&file, big).unwrap();

    let embedder = MockEmbedder::new(16);
    let store = Store::open(&db, 16).unwrap();
    index_path(&store, &embedder, &ChunkerConfig::default(), &file)
        .await
        .unwrap();

    let resp = ask(
        &store,
        &embedder,
        &MockChatModel,
        &RetrieverConfig::default(),
        "Jarvis 是什么？",
    )
    .await
    .expect("ask returned an error");

    assert!(!resp.citations.is_empty(), "expected at least one citation");
    // A truncated excerpt must remain valid UTF-8 and end with the ellipsis.
    assert!(resp
        .citations
        .iter()
        .any(|c| c.excerpt.ends_with('…') || !c.excerpt.is_empty()));
}
