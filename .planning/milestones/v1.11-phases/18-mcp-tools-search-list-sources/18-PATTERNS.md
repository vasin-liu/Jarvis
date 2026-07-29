# Phase 18: MCP tools search + list_sources - Pattern Map

**Mapped:** 2026-07-28
**Files analyzed:** 10 (2 create, 8 modify)
**Analogs found:** 10 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/retriever/src/kb_readonly.rs` | service | request-response + transform | `crates/retriever/src/related.rs` | exact |
| `crates/retriever/src/lib.rs` | config / barrel | — | self (`related` re-export) | exact |
| `crates/agent/src/tools.rs` | service | request-response | self (`search_knowledge` / `list_sources` arms) | exact |
| `crates/mcp/src/server.rs` | controller | request-response | self (stubs) + `agent/src/tools.rs` semantics | exact |
| `crates/mcp/src/main.rs` | controller | request-response + file-I/O | self + `config::build_embedder` | exact |
| `crates/mcp/Cargo.toml` | config | — | self (add path deps like Phase 17) | exact |
| `crates/mcp/tests/allowlist.rs` | test | transform (assert JSON) | self (keep allowlist; replace stub bodies) | exact |
| `crates/mcp/tests/tools_kb.rs` (optional NEW) | test | request-response | `allowlist.rs` + `retrieve.rs` `#[cfg(test)]` | role-match |
| `docs/mcp.md` | docs | — | self (Phase 17 stub wording → real tools) | exact |
| `crates/agent` focused tests (inline or new) | test | request-response | `tools.rs` `#[cfg(test)]` + `retrieve.rs` fixture | role-match |

## Pattern Assignments

### `crates/retriever/src/kb_readonly.rs` (service, request-response + transform)

**Analog:** `crates/retriever/src/related.rs` — structured helper over `retrieve` + char-safe truncation + typed DTOs; **not** agent string formatting.

**Module ownership pattern** — sibling of `related` / `retrieve`; uses crate `Result` / `RetrieveError`:

**Imports + DTO + truncate pattern** (`related.rs` lines 1–46):

```rust
use embedder::Embedder;
use serde::Serialize;
use store::{Source, SourceKind, Store};

use crate::error::Result;
use crate::retrieve::{retrieve, RetrieverConfig};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedSource {
    pub source_id: String,
    pub title: String,
    pub kind: SourceKind,
    pub snippet: String,
}

const RELATED_SNIPPET_CHARS: usize = 160;

fn truncate_snippet(text: &str) -> String {
    if text.chars().count() <= RELATED_SNIPPET_CHARS {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(RELATED_SNIPPET_CHARS).collect();
        format!("{truncated}…")
    }
}
```

**Core retrieve-wrapper pattern** (`related.rs` lines 58–101): call `retrieve`, map hits via `store.get_source`, return structured vec — copy this shape for `search_kb` (map → `KbHit` with excerpt ≤200) and `list_indexed_sources` (filter + cap, no `retrieve`).

```rust
pub async fn related_sources(
    store: &Store,
    embedder: &dyn Embedder,
    source_id: &str,
    top_n: Option<usize>,
) -> Result<Vec<RelatedSource>> {
    // ...
    let config = related_retriever_config(top_n);
    let hits = retrieve(store, embedder, &query, &config).await?;
    // map hits → RelatedSource { … snippet: truncate_snippet(&hit.text) }
    Ok(out)
}
```

**Hybrid defaults + empty query** (`retrieve.rs` lines 15–35) — `search_kb` must use these defaults then clamp `final_k`:

```rust
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

pub async fn retrieve(...) -> Result<Vec<ChunkHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }
    // ...
}
```

**Excerpt / citation truncate already in agent** (`tools.rs` lines 69–74) — move into shared `truncate_excerpt` (200 chars + `…`); prefer formatting LLM lines from shared excerpt (RESEARCH Pattern 3):

```rust
let excerpt = if hit.text.chars().count() > 200 {
    let truncated: String = hit.text.chars().take(200).collect();
    format!("{truncated}…")
} else {
    hit.text.clone()
};
```

**Indexed filter + omit URI** (`tools.rs` lines 90–97) — lift into `list_indexed_sources`; add hard-cap **200** + `truncated` / `total_indexed`; **never** map `s.uri`:

```rust
"list_sources" => {
    let sources = store.list_sources()?;
    let lines: Vec<String> = sources
        .into_iter()
        .filter(|s| s.status == IndexStatus::Indexed)
        .map(|s| format!("- {} ({})", s.title, s.kind.as_str()))
        .collect();
    Ok((lines.join("\n"), vec![]))
}
```

**Store list order** (`store.rs` lines 92–96) — preserve `ORDER BY id`; do not invent new sort:

```rust
"SELECT id, kind, uri, title, content_hash, indexed_at, status, error, summary
 FROM sources ORDER BY id",
```

**Error type** (`error.rs` lines 3–13) — helpers return `RetrieveError` (`EmptyQuery`, `Store`, `Embed`); MCP maps to tool error JSON; agent maps via existing `?` / `AgentError`.

**Fixture test pattern** (`retrieve.rs` lines 75–106) — tempfile + `MockEmbedder` + `indexer::index_path`:

```rust
#[tokio::test]
async fn hybrid_retrieve_finds_indexed_content() {
    let dir = tempfile::tempdir().unwrap();
    // Store::open(&db, 4); MockEmbedder::new(4); index_path(...);
    let hits = retrieve(&store, &embedder, "vector search", &RetrieverConfig::default())
        .await
        .unwrap();
    assert!(!hits.is_empty());
}
```

**Phase 18 deltas vs related.rs:**
- Caps: `EXCERPT_MAX_CHARS = 200`, `LIST_SOURCES_MAX = 200` (not 160 / top_n×5).
- `search_kb`: start from `RetrieverConfig::default()`, then `cfg.final_k = limit.clamp(1, default_final_k)`.
- `KbHit` fields: `chunk_id`, `source_id`, `title`, `loc`, `excerpt` — **no** `uri` / score / full text.
- `KbSourceEntry`: `id`, `title`, `kind` as string (`SourceKind::as_str()`); MCP JSON may use snake_case (discretion) — related uses `camelCase` for UI; MCP docs recommend snake_case JSON.

---

### `crates/retriever/src/lib.rs` (barrel)

**Analog:** self — add `mod kb_readonly` + `pub use` beside `related`:

```rust
mod error;
mod related;
mod rrf;
mod retrieve;

pub use error::{Result, RetrieveError};
pub use related::{related_sources, RelatedSource};
pub use retrieve::{retrieve, RetrieverConfig};
```

**Copy:** `mod kb_readonly;` + `pub use kb_readonly::{search_kb, list_indexed_sources, /* types + consts */};`

---

### `crates/agent/src/tools.rs` (service, request-response)

**Analog:** self — surgical replace of arms only; keep plugin/mutate arms untouched.

**Imports today** (lines 1–15):

```rust
use embedder::Embedder;
use retriever::{retrieve, RetrieverConfig};
use store::{IndexStatus, Store, TaskStatus};
use rag::Citation;
```

**Target:** `use retriever::{list_indexed_sources, search_kb, RetrieverConfig};` (drop direct `retrieve` / `IndexStatus` if unused elsewhere in file).

**Core pattern to replace** (`tools.rs` lines 56–98):

```rust
"search_knowledge" => {
    let query = args.get("query").and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::ToolArgs("query required".into()))?;
    let hits = retrieve(store, embedder, query, retriever).await?;
    // … Chinese lines + Citation …
}
"list_sources" => {
    let sources = store.list_sources()?;
    // filter Indexed → Chinese lines
}
```

**After extract:**
1. `let hits = search_kb(store, embedder, query, None).await?;` — D-03 locks **default** config inside helper (note: today arm uses passed `retriever`; Phase 18 intentionally aligns on default).
2. Empty → `"未找到相关内容。"` + `vec![]` (keep).
3. Build `Citation` from `KbHit` (`chunk_id`, `source_id`, `title`→`source_title`, `loc`, `excerpt`); **`source_uri`**: agent Citations today include URI via `get_source` — shared `KbHit` omits URI; either keep one `get_source` for citation URI only, or leave `source_uri` empty for search citations. Prefer one `get_source` only if Citation consumers require URI (existing RAG UI) — surgical: call `store.get_source` for `uri` when building `Citation`, do **not** put URI in MCP path.
4. Prefer LLM lines from **shared excerpt** (not full `hit.text`) per RESEARCH Pattern 3.
5. `list_sources`: `list_indexed_sources(store)?` → format `- {title} ({kind})`; if `truncated`, append footer `（已截断，共 {total_indexed} 条）`.

**Citation shape** (`rag/src/types.rs` lines 3–11):

```rust
pub struct Citation {
    pub chunk_id: i64,
    pub source_id: String,
    pub source_title: String,
    pub source_uri: String,
    pub loc: String,
    pub excerpt: String,
}
```

**Do not** wrap `execute_tool` from MCP (Pitfall F).

---

### `crates/mcp/src/server.rs` (controller, request-response)

**Analog:** self (rmcp `#[tool]` / `CallToolResult` / `SearchParams`) + agent semantics via `kb_readonly`.

**Current stub scaffold** (lines 1–60) — keep router / allowlist / `SearchParams`; replace body + struct fields:

```rust
pub struct JarvisMcp {
    #[allow(dead_code)]
    store: Option<Arc<Store>>,
}

fn stub_result(tool: &str) -> CallToolResult {
    let body = serde_json::json!({
        "status": "not_implemented",
        "phase": 18,
        "tool": tool,
        "message": "KB-backed MCP tools land in Phase 18"
    });
    CallToolResult::success(vec![ContentBlock::text(body.to_string())])
}

pub fn new(store: Option<Arc<Store>>) -> Self { Self { store } }
```

**Target constructor / fields:**

```rust
pub struct JarvisMcp {
    store: Arc<Store>,
    embedder: Arc<dyn Embedder>,
}

pub fn new(store: Arc<Store>, embedder: Arc<dyn Embedder>) -> Self { … }
```

**Success / error JSON via text content** (Phase 17 continuity — keep `ContentBlock::text` so `first_text()` works):

```rust
CallToolResult::success(vec![ContentBlock::text(body.to_string())])
// empty query / retrieve error:
CallToolResult::error(vec![ContentBlock::text(
    r#"{"error":"empty_query","message":"query must be non-empty"}"#.into(),
)])
```

**Handler flow:**
- `search`: read `SearchParams { query, limit }`; `search_kb(&self.store, self.embedder.as_ref(), &query, limit.map(|n| n as usize)).await`; map `RetrieveError::EmptyQuery` / embed/store errors → `CallToolResult::error` JSON; success → `{"results":[…]}` **without** `uri`.
- `list_sources`: `list_indexed_sources`; JSON `{"sources":[…],"truncated":bool,"total_indexed":usize}` — fields `id`/`title`/`kind` only.
- Update tool `description` strings to drop “stub — Phase 18”.
- Keep `annotations(read_only_hint = true)` and exactly two `#[tool]` methods.

**Anti-pattern:** `JarvisMcp::new(None)` / returning empty results when embedder failed.

---

### `crates/mcp/src/main.rs` (controller, request-response + file-I/O)

**Analog:** self (path resolve + `Store::open`) + `config::build_embedder` fail-closed.

**Today** (lines 23–35) — loads dim only; no embedder:

```rust
let config_path = config_dir.join("config.json");
let dim = match load_config(&config_path) {
    Ok(cfg) => cfg.embedding_dim(),
    Err(_) => AppConfig::default().embedding_dim(),
};
let store = Arc::new(Store::open(&db_path, dim)?);
let service = JarvisMcp::new(Some(store)).serve(stdio()).await?;
```

**Embedder factory** (`providers.rs` lines 30–56):

```rust
pub fn build_embedder_with_provider(
    config: &AppConfig,
    provider: EmbedderProvider,
) -> Result<Arc<dyn Embedder>, EmbedError> { /* Mock | Ollama | FastEmbed | Cloud */ }

pub fn build_embedder(config: &AppConfig) -> Result<Arc<dyn Embedder>, EmbedError> {
    build_embedder_with_provider(config, config.embedding.embedder)
}
```

**Target pattern:**
1. `let cfg = load_config(&config_path).unwrap_or_else(|_| AppConfig::default());`
2. `let dim = cfg.embedding_dim();` — **same** `cfg` for store dim + embedder (Pitfall E).
3. `let embedder = config::build_embedder(&cfg).unwrap_or_else(|e| { eprintln!(...); process::exit(1); });`
4. `JarvisMcp::new(store, embedder).serve(stdio())…`
5. Keep `ensure_db_exists` fail-closed; still **no** `windows_subsystem`.

---

### `crates/mcp/Cargo.toml` (config)

**Analog:** self — add path deps only (no new registry crates):

```toml
[dependencies]
rmcp = { workspace = true }
store = { path = "../store" }
config = { path = "../config" }
# ADD:
retriever = { path = "../retriever" }
embedder = { path = "../embedder" }
```

**Dev-deps for real KB tests** (mirror `retriever/Cargo.toml` lines 13–19):

```toml
[dev-dependencies]
tempfile = "3"
# likely need for seed fixtures:
chunker = { path = "../chunker" }
indexer = { path = "../indexer" }
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

Keep `rmcp = 2.2.0` workspace pin — do not bump.

---

### `crates/mcp/tests/allowlist.rs` (test)

**Analog:** self — **keep** name allowlist test; replace stub body tests.

**Allowlist assertion to preserve** (lines 7–17):

```rust
#[test]
fn tool_allowlist_is_exactly_search_and_list_sources() {
    let tools = JarvisMcp::tool_router().list_all();
    let names: HashSet<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(names, HashSet::from(["search", "list_sources"]), …);
}
```

**JSON text helper to keep** (lines 19–24):

```rust
fn first_text(result: &rmcp::model::CallToolResult) -> String {
    match &result.content[0] {
        ContentBlock::Text(t) => t.text.clone(),
        other => panic!("expected text block, got {other:?}"),
    }
}
```

**Replace** `stub_*_returns_not_implemented_json` with tempfile `Store` + `MockEmbedder` + indexed fixture; assert `results` / `sources` shapes, empty-query error, limit clamp, **no** `uri` key, cap/`truncated` if seeded >200 (or unit-test cap in `kb_readonly`).

**Constructor break:** `JarvisMcp::new(None)` → `JarvisMcp::new(store, embedder)`.

---

### `crates/mcp/tests/tools_kb.rs` (optional NEW test)

**Analog:** `allowlist.rs` (`first_text` + tokio) + `retrieve.rs` fixture seeding.

Use when allowlist file would grow too large. Same Match Quality as extending `allowlist.rs` — planner may colocate either way (RESEARCH Validation Architecture prefers replace stub asserts or new `tools_kb.rs`).

---

### `docs/mcp.md` (docs)

**Analog:** self — remove Phase 17 stub status; document real JSON + caps.

**Stub wording to delete** (lines 5–6, 63–68):

```markdown
**Phase 17 status:** transport + tool allowlist only. Tools `search` and `list_sources` return stub `not_implemented` JSON until **Phase 18** wires hybrid retrieval.
```

**Copy structure:** keep Build / Prerequisite / Path precedence / host config tables; replace Tools section with:
- `search` — hybrid RRF, optional `limit` ≤ `final_k` (8), excerpt ≤200 chars, JSON `results` with `chunk_id`/`source_id`/`title`/`loc`/`excerpt`, empty query → error
- `list_sources` — Indexed only, fields `id`/`title`/`kind`, hard-cap 200 + `truncated`/`total_indexed`, **no paths**
- Embedder from sibling `config.json` via `build_embedder`; fail closed at startup

---

### Agent focused tests (test)

**Analog:** `tools.rs` `#[cfg(test)]` (lines 260–287) is parse/prompt-only today — add `#[tokio::test]` with MockEmbedder + tempfile **or** assert helpers in `retriever` and thin agent smoke that arms call helpers.

Hang regression note (`agent/tests/real_embed_hang.rs`): do **not** byte-slice at 200 — shared helper must use `chars()` (already the agent pattern).

## Shared Patterns

### Structured core, dual presenters
**Source:** `related.rs` (DTO + retrieve) + `tools.rs` (Chinese + Citation) + `server.rs` (JSON text)
**Apply to:** `kb_readonly`, agent arms, MCP handlers
- Shared: filter/limit/retrieve/truncate
- Agent: strings + `Citation`
- MCP: `serde_json` → `CallToolResult::success|error` + `ContentBlock::text`

### Char-safe truncation
**Source:** `tools.rs` 69–74; `related.rs` 38–45
**Apply to:** `truncate_excerpt` (200) in `kb_readonly`
```rust
text.chars().take(N).collect::<String>() + "…"
```

### Trait injection + MockEmbedder fixtures
**Source:** `retrieve.rs` tests; `providers.rs` Mock branch
**Apply to:** `cargo test -p retriever|mcp|agent` — never live LLM/Feishu

### Fail closed vs empty hits
**Source:** CONTEXT D-09; `main.rs` `ensure_db_exists` exit(1)
**Apply to:** `build_embedder` at startup; per-call retrieve/embed errors → tool **error** JSON; empty KB → success `"results":[]`

### Single DB owner
**Source:** workspace rule — only `store` opens SQLite
**Apply to:** MCP continues `Store::open` only; helpers take `&Store`

### Tool allowlist continuity (MCP-04)
**Source:** `allowlist.rs` + `server.rs` exactly two `#[tool]`s
**Apply to:** Phase 18 — names stay `{search, list_sources}`; no `execute_tool` wrap

### Privacy — no absolute paths in MCP
**Source:** agent list omits URI (`tools.rs` 94–95); D-08
**Apply to:** MCP JSON never serializes `Source.uri` / `file://`

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All Phase 18 files have in-repo analogs (`related.rs` for new module; Phase 17 MCP for handlers) |

## Metadata

**Analog search scope:** `crates/retriever/`, `crates/agent/src/tools.rs`, `crates/mcp/`, `crates/config/src/providers.rs`, `crates/store/src/store.rs`, `crates/rag/src/types.rs`, `docs/mcp.md`, prior `17-PATTERNS.md`
**Files scanned:** ~15 primary + Phase 17 pattern map
**Pattern extraction date:** 2026-07-28

### Discretion locks (from RESEARCH — for planner)
| Item | Lock |
|------|------|
| Module path | `crates/retriever/src/kb_readonly.rs` |
| List cap N | **200** |
| Search clamp | ≤ `RetrieverConfig::default().final_k` (**8**) |
| Excerpt | **200** Unicode chars + `…` |
| MCP JSON | text `ContentBlock`; include `chunk_id`/`loc`; no `uri` |
| Agent refactor | same phase (Wave 2) |
