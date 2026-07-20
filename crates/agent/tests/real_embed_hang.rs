//! Reproduction harness for the Test-5 blocker: real-provider agent queries hang.
//!
//! Mirrors the Tauri runtime (multi-thread Tokio) and the exact prod path:
//! real `FastEmbedder` + real `Store` + `run_agent` + Mock chat. The matching
//! unit test `agent_uses_search_tool` uses `MockEmbedder` and passes, so this
//! isolates whether the real embedder path deadlocks.
//!
//! Run: `cargo test -p agent --test real_embed_hang -- --nocapture`

use std::path::PathBuf;
use std::time::Duration;

use agent::{run_agent, AgentProfile, AgentRunContext};
use chunker::ChunkerConfig;
use embedder::{Embedder, FastEmbedder};
use indexer::index_path;
use llm::MockChatModel;
use retriever::RetrieverConfig;
use store::Store;

/// Locate the already-downloaded model cache so the test never hits the network.
fn fastembed_cache_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = PathBuf::from(appdata)
        .join("com.jarvis.app")
        .join("fastembed_cache");
    dir.exists().then_some(dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn real_fastembed_agent_does_not_hang() {
    // Clean trace log for an isolated run.
    let _ = std::fs::remove_file(std::env::temp_dir().join("jarvis-trace.log"));

    let cache = fastembed_cache_dir();
    if cache.is_none() {
        eprintln!(
            "SKIP: no cached fastembed model at %APPDATA%/com.jarvis.app/fastembed_cache; \
             run the app once to download it, then re-run this test."
        );
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("kb.sqlite");
    let file = dir.path().join("note.md");
    // Long Chinese paragraph so chunk text exceeds 200 BYTES (≈67 CJK chars).
    // search_knowledge slices `&hit.text[..200]` at byte 200 — a non-char-boundary
    // for multibyte UTF-8 — which panics and silently hangs the IPC command.
    let para = "Jarvis 是一个本地优先的个人知识中枢，支持对本地文档与飞书内容进行索引、\
                混合检索与带引用的问答，并提供智能体模式以调用搜索、记忆与任务等工具。";
    let big = std::iter::repeat(para).take(8).collect::<Vec<_>>().join("\n\n");
    std::fs::write(&file, big).unwrap();

    let embedder = FastEmbedder::try_new("bge-small-zh-v1.5", cache).unwrap();
    let store = Store::open(&db, embedder.dim()).unwrap();
    index_path(&store, &embedder, &ChunkerConfig::default(), &file)
        .await
        .unwrap();

    let profile = AgentProfile {
        id: "t".into(),
        name: "知识助手".into(),
        system_prompt: "你是知识助手".into(),
        enabled: true,
        chat_provider: None,
        embedder_provider: None,
    };

    let retriever = RetrieverConfig::default();
    let chunker = ChunkerConfig::default();
    let chat = MockChatModel;
    let ctx = AgentRunContext {
        store: &store,
        chunker: &chunker,
        retriever: &retriever,
        hooks: &[],
        enabled_hook_ids: &[],
        plugins: &[],
        enabled_plugin_ids: &[],
        granted_plugin_permissions: &[],
    };

    let fut = run_agent(&ctx, &chat, &embedder, &profile, &[], &[], "Jarvis 是什么？");

    match tokio::time::timeout(Duration::from_secs(30), fut).await {
        Err(_) => panic!(
            "HANG REPRODUCED: run_agent did not return within 30s on the real FastEmbed path. \
             See %TEMP%/jarvis-trace.log for the last stage reached."
        ),
        Ok(res) => {
            let resp = res.expect("run_agent returned an error");
            assert!(!resp.answer.is_empty(), "agent answer was empty");
            assert!(!resp.tool_calls.is_empty(), "agent did not call any tool");
        }
    }
}
