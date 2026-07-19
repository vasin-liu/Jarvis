# Phase 09: LLM wiki analysis - Pattern Map

**Mapped:** 2026-07-19
**Files analyzed:** 5
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/insights/src/wiki.rs` | service | request-response (+ transform parse; FS write helper) | `crates/insights/src/tasks.rs` (primary load→LLM→parse); `summarize.rs` for message shape only | exact |
| `crates/insights/src/error.rs` | model | — | `crates/insights/src/error.rs` (`InvalidTasksJson`); optional Io from `crates/ingest/src/error.rs` / `crates/config/src/error.rs` | exact |
| `crates/insights/src/lib.rs` | config | — | current `insights` barrel + Phase 08 re-export pattern | exact |
| `crates/insights/Cargo.toml` | config | — | sibling crates (`crates/ingest/Cargo.toml`, `crates/agent/Cargo.toml`) `tempfile = "3"` under `[dev-dependencies]` | exact |
| `crates/llm/src/mock.rs` | provider | request-response | existing `build_answer` keyword branches (`任务提取` / `摘要`) | exact |

**Anti-analogs (do not copy):**
- `summarize_source` → `set_source_summary` — wiki must **not** touch `sources.summary` (D-07)
- `extract_tasks_from_source` → `delete_tasks_for_source` / `insert_task` — analyze returns `WikiAnalysis` only; no store writes
- Plan Task 3 `MockChatModel::new(vec![…])` — superseded by D-01 keyword heuristic
- Phase 10 `WikiPage` indexing / `indexer` / `embedder` — out of scope

**UI / E2E:** N/A — library-only; Phase 13 owns wiki E2E.

## Pattern Assignments

### `crates/insights/src/wiki.rs` (service, request-response + transform + FS)

**Analog (primary — analyze):** `crates/insights/src/tasks.rs` — Indexed guard, empty text, truncate, Chinese system prompt, `chat.complete`, private parse → typed error.

**Analog (message shape):** `crates/insights/src/summarize.rs` — same user `标题` / `正文` format; **omit** summary persist.

**Analog (render payload):** existing `render_wiki_pages` in same file (Phase 08) — writer consumes `WikiCompileResult` only.

**Analog (test double ChatModel):** `crates/agent/src/run.rs` `FenceReplyModel` (lines 255–284) — inline `struct` + `async_trait` `ChatModel` in `#[cfg(test)]`.

**Analog (FS write):** `crates/config/src/file.rs` `save_config` (create parent dirs + `fs::write`); test layout like `crates/agent/src/plugins.rs` tempfile + `create_dir_all`.

**Existing types to keep** (`wiki.rs` lines 3–35 — already shipped; do not add `#[serde(default)]`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiAnalysis {
    pub summary: String,
    pub entities: Vec<WikiEntity>,
    pub concepts: Vec<WikiConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCompileResult {
    pub pages: Vec<WikiPageDraft>,
    pub index_markdown: String,
}
```

**Imports pattern** (adapt from `tasks.rs` lines 1–5):
```rust
use llm::{ChatModel, Message, Role};
use store::{IndexStatus, Store};

use crate::error::{InsightsError, Result};
// WikiAnalysis / WikiCompileResult / render_wiki_pages already in this module
```

**Core load → LLM pattern** (`tasks.rs` lines 16–44 — copy guards; drop task persist):
```rust
pub async fn extract_tasks_from_source(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<Vec<Task>> {
    let source = store.get_source(source_id)?;
    if source.status != IndexStatus::Indexed {
        return Err(InsightsError::NotIndexed(source_id.to_string()));
    }

    let text = store.source_chunk_text(source_id)?;
    if text.trim().is_empty() {
        return Err(InsightsError::EmptySource(source_id.to_string()));
    }

    let excerpt = crate::truncate_chars(&text, MAX_SOURCE_CHARS);
    let messages = vec![
        Message {
            role: Role::System,
            content: "你是任务提取助手。…".into(),
        },
        Message {
            role: Role::User,
            content: format!("标题：{}\n\n正文：\n{}", source.title, excerpt),
        },
    ];

    let raw = chat.complete(&messages).await?;
    let drafts = parse_task_drafts(&raw)?;
    // … store writes — DO NOT copy for wiki
```

**Apply as `analyze_source_for_wiki`:**
```rust
const MAX_SOURCE_CHARS: usize = 12_000; // or reuse crate-level; same as summarize/tasks

pub async fn analyze_source_for_wiki(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<WikiAnalysis> {
    // same Indexed / EmptySource / truncate as tasks
    // system MUST contain 「笔记编译」 + bare-JSON + sparse guidance (D-02, D-05, D-06)
    // user: format!("标题：{}\n\n正文：\n{}", source.title, excerpt)  (D-08)
    let raw = chat.complete(&messages).await?;
    parse_wiki_analysis(&raw)  // NO set_source_summary, NO task CRUD
}
```

**ANTI-PATTERN — do not copy** (`summarize.rs` lines 35–37):
```rust
let summary = chat.complete(&messages).await?;
store.set_source_summary(source_id, &summary)?;  // FORBIDDEN for wiki (D-07)
Ok(summary)
```

**Fence-strip parse baseline** (`tasks.rs` lines 67–77 — extend, don't stop here):
```rust
fn parse_task_drafts(raw: &str) -> Result<Vec<TaskDraft>> {
    let trimmed = raw.trim();
    let json = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    serde_json::from_str(json).map_err(|e| InsightsError::InvalidTasksJson(e.to_string()))
}
```

**Wiki parse (D-09…D-12) — fence strip + first `{…}` + strict serde:**
```rust
fn parse_wiki_analysis(raw: &str) -> Result<WikiAnalysis> {
    let json = extract_first_json_object(raw)
        .map_err(InsightsError::InvalidWikiJson)?;
    serde_json::from_str(json)
        .map_err(|e| InsightsError::InvalidWikiJson(e.to_string()))
}

/// After fence strip: find first `{`, brace-depth scan; skip braces inside JSON strings.
fn extract_first_json_object(raw: &str) -> std::result::Result<&str, String> { /* … */ }
```

Structurally valid empty (`""` + `[]` + `[]`) → `Ok` (D-11). Missing/wrong-typed fields → `InvalidWikiJson` (D-10). Do **not** add `#[serde(default)]` on `WikiAnalysis` fields.

**Write helper** (D-13 — thin public; slug paths only from `WikiCompileResult`):
```rust
/// Writes `{slug}.md` for each page and `index.md` under `wiki_root`.
/// Creates parent directories. Does not index or embed.
pub fn write_wiki_pages_to_dir(
    compiled: &WikiCompileResult,
    wiki_root: &std::path::Path,
) -> Result<()> {
    std::fs::create_dir_all(wiki_root)?;
    for page in &compiled.pages {
        let path = wiki_root.join(format!("{}.md", page.slug));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &page.body_markdown)?;
    }
    std::fs::write(wiki_root.join("index.md"), &compiled.index_markdown)?;
    Ok(())
}
```

**FS create-parent pattern** (`config/src/file.rs` lines 55–62):
```rust
pub fn save_config(path: impl AsRef<Path>, config: &AppConfig) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(config)?;
    fs::write(path, raw)?;
    Ok(())
}
```

**Fail-closed gate at call site** (tests / Phase 10 callers — not a combined public API):
```rust
let dir = tempfile::tempdir().unwrap();
let wiki = dir.path().join("wiki");
match analyze_source_for_wiki(&store, &bad_chat, "s1").await {
    Ok(analysis) => {
        let source = store.get_source("s1").unwrap();
        let compiled = render_wiki_pages(&analysis, &source.uri, &source.title);
        write_wiki_pages_to_dir(&compiled, &wiki).unwrap();
    }
    Err(e) => {
        assert!(matches!(e, InsightsError::InvalidWikiJson(_)));
        assert!(!wiki.exists() || /* zero files under wiki */);
    }
}
```

**Garbage ChatModel test double** (adapt `agent/src/run.rs` lines 260–284; RESEARCH FixedReplyChat sketch):
```rust
struct FixedReplyChat(&'static str);

#[async_trait::async_trait]
impl ChatModel for FixedReplyChat {
    fn id(&self) -> &str { "test-fixed" }
    async fn complete(&self, _: &[Message]) -> llm::Result<String> {
        Ok(self.0.to_string())
    }
    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> llm::Result<String> {
        let answer = self.complete(messages).await?;
        on_token(answer.clone());
        Ok(answer)
    }
}
```

Use fixed strings for D-14: (a) prose, (b) truncated `{…`, (c) object missing/wrong-typed fields.

**Testing pattern** (`tasks.rs` lines 79–135 + `summarize.rs` lines 40–88):
- Colocated `#[cfg(test)]` in `wiki.rs` (no `crates/insights/tests/` today)
- `sample_indexed` helper mirroring tasks/summarize (`Store::open_in_memory` + `upsert_source` + `insert_chunks`)
- `#[tokio::test]` for analyze; sync `#[test]` for pure parse helpers
- Happy path: `&MockChatModel` (after 「笔记编译」 branch lands)
- Fail-closed: `FixedReplyChat` + `tempfile::tempdir` + assert zero files

**Wave 0 tests (CONTEXT/RESEARCH):**
- `analyze_parses_mock_json` — Mock → non-empty summary + ≥1 entity
- `wiki_parse_fail_prose_writes_zero_files` / `_truncated_` / `_schema_` — `InvalidWikiJson` + zero files
- `parse_wiki_allows_empty_struct` — D-11
- `analyze_does_not_touch_source_summary` — D-07 (recommended)
- `write_wiki_pages_to_dir_writes_tree` — happy write creates `index.md` (recommended)

---

### `crates/insights/src/error.rs` (model)

**Analog (primary):** same file — `InvalidTasksJson` (lines 9–10).

**Analog (optional Io):** `crates/ingest/src/error.rs` / `crates/config/src/error.rs` — `Io(#[from] std::io::Error)`.

**Current error enum** (`error.rs` lines 3–15):
```rust
#[derive(Debug, Error)]
pub enum InsightsError {
    #[error("source not indexed: {0}")]
    NotIndexed(String),
    #[error("no chunk text for source: {0}")]
    EmptySource(String),
    #[error("failed to parse tasks json: {0}")]
    InvalidTasksJson(String),
    #[error("llm: {0}")]
    Llm(#[from] llm::LlmError),
    #[error("store: {0}")]
    Store(#[from] store::StoreError),
}
```

**Apply (D-12 + discretionary Io for writer):**
```rust
#[error("failed to parse wiki json: {0}")]
InvalidWikiJson(String),
#[error("io error: {0}")]
Io(#[from] std::io::Error),  // optional but recommended for write_wiki_pages_to_dir
```

Do **not** add a generic `Parse` variant (CONTEXT override vs old plan Task 3).

---

### `crates/insights/src/lib.rs` (config / barrel)

**Analog:** current barrel (lines 1–12) + Phase 08 `pub use wiki::{…}`.

**Current** (`lib.rs` lines 1–20):
```rust
mod error;
mod summarize;
mod tasks;
mod wiki;

pub use error::{InsightsError, Result};
pub use summarize::summarize_source;
pub use tasks::extract_tasks_from_source;
pub use wiki::{
    render_wiki_pages, WikiAnalysis, WikiCompileResult, WikiConcept, WikiEntity, WikiPageDraft,
    WikiPageType,
};

pub(crate) fn truncate_chars(text: &str, max: usize) -> String { /* … */ }
```

**Apply — extend wiki re-exports only:**
```rust
pub use wiki::{
    analyze_source_for_wiki, render_wiki_pages, write_wiki_pages_to_dir, WikiAnalysis,
    WikiCompileResult, WikiConcept, WikiEntity, WikiPageDraft, WikiPageType,
};
```

Keep `truncate_chars` as `pub(crate)`. Do not re-export private `parse_wiki_analysis` / `extract_first_json_object` unless tests need `pub(crate)`.

---

### `crates/insights/Cargo.toml` (config)

**Analog:** `crates/ingest/Cargo.toml`, `crates/agent/Cargo.toml` — `tempfile = "3"` under `[dev-dependencies]`.

**Current** (`insights/Cargo.toml`):
```toml
[dependencies]
llm = { path = "../llm" }
store = { path = "../store" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
sha2 = { workspace = true }
hex = { workspace = true }

[dev-dependencies]
tokio = { workspace = true }
```

**Apply — add only:**
```toml
[dev-dependencies]
tokio = { workspace = true }
tempfile = "3"
```

No new production deps. Do **not** add `indexer`, `embedder`, JSON-repair crates, or queue-mock APIs.

---

### `crates/llm/src/mock.rs` (provider, request-response)

**Analog:** existing `build_answer` keyword branches (lines 57–63) — insert 「笔记编译」 **before** `任务提取` / `摘要` (and before memory JSON-array branch if collision risk).

**Branch order pattern** (`mock.rs` lines 53–63):
```rust
if system.contains("记忆提取") || (system.contains("JSON 数组") && user.contains("问题：")) {
    return Ok(r#"[{"content":"用户关注 Jarvis 知识库相关主题"}]"#.to_string());
}

if system.contains("任务提取") {
    return Ok(r#"[{"title":"整理要点","description":"根据来源整理可执行待办"}]"#.to_string());
}

if system.contains("摘要") {
    return Ok("（Mock 摘要）该来源主要讨论相关主题与待办事项。".to_string());
}
```

**Apply (D-01…D-03) — early branch, minimal valid wiki JSON:**
```rust
if system.contains("笔记编译") {
    return Ok(
        r#"{"summary":"要点摘要","entities":[{"name":"示例实体","blurb":"说明"}],"concepts":[]}"#
            .to_string(),
    );
}
// then existing 记忆提取 / 任务提取 / 摘要 …
```

Happy-path only — do **not** make production Mock return garbage (D-04).

**ChatModel trait** (`llm/src/lib.rs` lines 15–24) — unchanged; test doubles implement same three methods:
```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, messages: &[Message]) -> Result<String>;
    async fn complete_stream(
        &self,
        messages: &[Message],
        on_token: &mut (dyn FnMut(String) + Send),
    ) -> Result<String>;
}
```

**Mock unit test pattern** (`mock.rs` lines 100–131) — add `mock_wiki_compile_json`:
```rust
#[tokio::test]
async fn mock_wiki_compile_json() {
    let out = MockChatModel
        .complete(&[
            Message { role: Role::System, content: "…笔记编译…".into() },
            Message { role: Role::User, content: "标题：…".into() },
        ])
        .await
        .unwrap();
    assert!(out.contains(r#""summary""#));
    assert!(out.contains(r#""entities""#));
}
```

## Shared Patterns

### Insights load → LLM → parse (no store side effect for wiki)
**Source:** `crates/insights/src/tasks.rs`, `summarize.rs`  
**Apply to:** `analyze_source_for_wiki`  
- `get_source` → `Indexed` → `source_chunk_text` → empty check → `truncate_chars(…, 12_000)`  
- Chinese system + `标题`/`正文` user message  
- Wiki: return parsed DTO only — **no** `set_source_summary` / task writes

### Typed JSON parse errors
**Source:** `InsightsError::InvalidTasksJson` in `error.rs` + `parse_task_drafts`  
**Apply to:** `InvalidWikiJson` + `parse_wiki_analysis`  
- Map extract + serde failures into message string  
- Symmetric naming / `#[error("failed to parse … json: {0}")]`

### Keyword Mock heuristics
**Source:** `crates/llm/src/mock.rs` `build_answer`  
**Apply to:** 「笔记编译」 branch  
- Match `system.contains(…)`; unique Chinese product phrase  
- Insert **above** colliding branches (`任务提取`, `摘要`, memory JSON)

### Inline ChatModel test doubles
**Source:** `crates/agent/src/run.rs` `FenceReplyModel`  
**Apply to:** D-14 fail-path fixtures in `wiki.rs` tests  
- Fixed reply string; ignore messages; implement `complete` + `complete_stream`

### Tempfile + create_dir_all in unit tests
**Source:** sibling crates (`agent`, `ingest`, `config`); pin `tempfile = "3"`  
**Apply to:** fail-closed + happy write tests  
- `tempfile::tempdir()`; assert wiki root empty/nonexistent on parse fail

### Fail-closed ordering (WIKI-05)
**Source:** CONTEXT D-13/D-14; RESEARCH Pattern 3  
**Apply to:** all callers of `write_wiki_pages_to_dir`  
- Analyze (and full deserialize) **before** any FS write  
- Writer accepts only `WikiCompileResult` (slugs from Phase 08 renderer)

### Module re-export surface
**Source:** `crates/insights/src/lib.rs`  
**Apply to:** public `analyze_source_for_wiki` + `write_wiki_pages_to_dir`

### TDD / unit layer
**Source:** `.cursor/rules/tdd-goal-driven.mdc`  
**Apply to:** Wave 0 tests  
- Failing tests first; `cargo test -p insights wiki` and `cargo test -p llm`  
- No E2E this phase (`e2e-required.mdc` N/A until Phase 13)

### IO error `#[from]`
**Source:** `crates/ingest/src/error.rs`, `crates/config/src/error.rs`  
**Apply to:** optional `InsightsError::Io` when writer returns `insights::Result<()>`

## No Analog Found

| File / concern | Role | Data Flow | Reason |
|----------------|------|-----------|--------|
| *(brace-balanced `extract_first_json_object`)* | utility | transform | Tasks only fence-strip arrays; no in-tree first-`{…}` scanner — implement private helper in `wiki.rs` per RESEARCH Pattern 2 + D-14 truncated fixture |
| *(public analyze-then-write combined API)* | service | request-response | Intentionally **not** shipped (RESEARCH Open Q1) — gate stays explicit in tests / Phase 10 |

These are not separate deliverable files. Planner should follow RESEARCH sketches, not invent new crates.

## Metadata

**Analog search scope:** `crates/insights/` (`wiki`, `tasks`, `summarize`, `error`, `lib`, `Cargo.toml`), `crates/llm/` (`mock`, `lib`), `crates/agent/src/run.rs` (ChatModel test double), `crates/config/src/file.rs` + `error.rs` (FS write + Io), sibling `Cargo.toml` tempfile pins, `.cursor/rules/`, `09-CONTEXT.md` / `09-RESEARCH.md`  
**Files scanned:** ~14 primary  
**Pattern extraction date:** 2026-07-19  
**Locked overrides vs plan Task 3:** keyword Mock (not queue ctor); `InvalidWikiJson` (not generic `Parse`); public write helper for fail-closed proof; no `sources.summary` coupling
)
