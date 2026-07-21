# Phase 10: Persist + index - Pattern Map

**Mapped:** 2026-07-21
**Files analyzed:** 7
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/insights/src/wiki.rs` | service | transform + file-I/O + batch index | evolve existing `write_wiki_pages_to_dir` / `analyze_source_for_wiki`; index via `crates/memory/src/learn.rs` `add_memory` | exact |
| `crates/insights/src/error.rs` | model | — | `crates/memory/src/error.rs` (`Index(#[from])` + domain rejects) | exact |
| `crates/insights/src/lib.rs` | config | — | current barrel + Phase 09 re-exports | exact |
| `crates/insights/Cargo.toml` | config | — | `crates/memory/Cargo.toml` path deps (`chunker`/`embedder`/`indexer`/`ingest`) | exact |
| `src-tauri/src/commands/wiki.rs` | controller | request-response | `src-tauri/src/commands/memory.rs` (`add_memory_cmd`) | exact |
| `src-tauri/src/commands/mod.rs` | route | — | existing `mod memory` + `pub use memory::{…}` | exact |
| `src-tauri/src/lib.rs` | route | — | `generate_handler![…, add_memory_cmd, …]` registration | exact |

**Consume-only (no new files; call existing APIs):**

| API | Role | Analog usage |
|-----|------|--------------|
| `indexer::index_document` | service / batch | hash-skip + `SourceKind` upsert |
| `ingest::{Document, hash_text}` | model / transform | full-body hash for Document |
| `store::{delete_chunks_for_source, delete_source, get_source}` | store / CRUD | cleanup pair from `remove_source` |
| `config::WikiConfig.enabled` | config | gate via `state.config().wiki.enabled` |

**Anti-analogs (do not copy):**
- Writing `compiled.index_markdown` to disk as vault catalog (`wiki.rs` L59) — violates D-01; use scan-rebuild
- Injecting `content_hash` into frontmatter — D-10; keep `frontmatter_omits_content_hash` green
- `forget_memory` → `delete_source` only (`learn.rs` L64) — **orphan risk**; wiki cleanup must use library `remove_source` pair (`delete_chunks_for_source` then `delete_source`)
- Putting compile orchestration only in `insights_ops.rs` — library owns `compile_wiki_for_source` (D-14)
- Watching `{app_data}/wiki` or wiring `auto_on_insights` — deferred; compile-loop risk
- `serde_yaml` for frontmatter peek — STACK forbids; use line/peek like `agent/skills.rs`

**UI / E2E:** N/A this phase — thin IPC only; Phase 11 UI / Phase 13 E2E.

**Related gap (optional follow-up, not required by WIKI-04):** `src-tauri/src/index_ops.rs` L653–656 marks `SourceKind::WikiPage` rebuild as unimplemented. Phase 10 library path indexes via compile; planner may note rebuild/retry still needs a disk-read path later.

---

## Pattern Assignments

### `crates/insights/src/wiki.rs` (service, transform + file-I/O + batch)

**Role in Phase 10:** Add `compile_wiki_for_source`, evolve write policy (D-09), stale cleanup (D-06…D-08), `rebuild_index_md_from_disk` (D-01…D-05), index-from-disk with `wiki://{slug}`.

**Analog (orchestration + Document URI):** `crates/memory/src/learn.rs` — synthetic URI + `Document` + `index_document` + trait injection.

**Analog (analyze gate, already shipped):** same file `analyze_source_for_wiki` L18–47 — Indexed / empty / fail-closed before FS.

**Analog (blind writer to evolve):** same file `write_wiki_pages_to_dir` L49–60.

**Analog (index catalog shape):** same file `build_index_markdown` L268–301 — reuse section headers / `wikilink`; feed **disk scan** entries sorted by slug, not in-memory compile pages alone.

**Analog (frontmatter peek):** `crates/agent/src/skills.rs` L39–51 — split on `---` fences, line-scan keys (no YAML crate).

**Analog (cleanup Store):** `src-tauri/src/commands/library.rs` L22–27 — `delete_chunks_for_source` then `delete_source`.

**Imports pattern** (mirror memory learn + existing wiki):
```rust
use std::path::Path;

use chunker::ChunkerConfig;
use embedder::Embedder;
use ingest::{hash_text, Document};
use indexer::index_document;
use llm::{ChatModel, Message, Role};
use serde::Serialize;
use store::{IndexStatus, SourceKind, Store};

use crate::error::{InsightsError, Result};
```

**Core Document + index pattern** (`learn.rs` L110–120 — copy shape; change URI scheme + kind):
```rust
let uri = format!("memory://{}", Uuid::new_v4());
let doc = Document {
    uri: uri.clone(),
    title: /* … */,
    text: text.to_string(),
    content_hash: hash_text(text),
};
index_document(store, embedder, chunker, doc, SourceKind::Memory).await?;
```
Wiki variant (prescribed):
```rust
let uri = format!("wiki://{slug}"); // slug e.g. "entities/acme"
let text = std::fs::read_to_string(&path)?; // D-11/D-12 always from disk
let doc = Document {
    uri: uri.clone(),
    title: title_from_frontmatter_or_slug(&text, &slug),
    text: text.clone(),
    content_hash: hash_text(&text),
};
index_document(store, embedder, chunker, doc, SourceKind::WikiPage).await?;
```

**Hash-skip consumer** (`indexer/src/lib.rs` L24–28 — do not reimplement):
```rust
let source_id = doc.uri.clone();
if let Ok(existing) = store.get_source(&source_id) {
    if existing.content_hash == doc.content_hash && existing.status == IndexStatus::Indexed {
        return Ok(());
    }
}
```

**Current writer (must evolve — anti-pattern today)** (`wiki.rs` L49–60):
```rust
pub fn write_wiki_pages_to_dir(compiled: &WikiCompileResult, wiki_root: &Path) -> Result<()> {
    std::fs::create_dir_all(wiki_root)?;
    for page in &compiled.pages {
        let path = wiki_root.join(format!("{}.md", page.slug));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &page.body_markdown)?;
    }
    std::fs::write(wiki_root.join("index.md"), &compiled.index_markdown)?; // ← D-01: stop using for vault
    Ok(())
}
```

**Frontmatter emit (unchanged; peek these keys)** (`wiki.rs` L354–359):
```rust
fn build_frontmatter(title: &str, page_type_str: &str, source_uri: &str) -> String {
    let escaped_title = yaml_escape_double_quoted(title);
    let escaped_uri = yaml_escape_double_quoted(source_uri);
    format!(
        "---\ntitle: \"{escaped_title}\"\ntype: {page_type_str}\nsources: [\"{escaped_uri}\"]\ngenerated: true\n---\n"
    )
}
```

**Frontmatter peek pattern** (`skills.rs` L39–51 — adapt keys `generated` / `title` / `sources`):
```rust
if raw.starts_with("---") {
    if let Some(end) = raw[3..].find("\n---") {
        let front = &raw[3..3 + end];
        for line in front.lines() {
            if let Some((k, v)) = line.split_once(':') {
                match k.trim() {
                    "name" => /* … */,
                    "description" => /* … */,
                    _ => {}
                }
            }
        }
    }
}
```
Peek rules for wiki: trim line equals `generated: true` → writable/deletable; absence/`false` → skip overwrite + skip cleanup; `sources:` contains quoted input URI → ownership (D-07); `title: "..."` → catalog display (D-03).

**Catalog rebuild shape** (`wiki.rs` L268–301 — reuse headers; sort by path; omit empty):
```rust
fn build_index_markdown(pages: &[WikiPageDraft]) -> String {
    let mut out = String::from("# Wiki\n");
    // ## Sources / ## Entities / ## Concepts only when non-empty
    // entries: `- [[slug|title]]` via wikilink()
    …
}
```
Scan dirs: only `sources/`, `entities/`, `concepts/` `*.md` (D-02). Lex sort by slug (D-04). Never index root `index.md` (D-13).

**Cleanup Store pair** (`library.rs` L22–27):
```rust
store.delete_chunks_for_source(&id).map_err(|e| e.to_string())?;
store.delete_source(&id).map_err(|e| e.to_string())
```
Wiki: `id` / URI = `wiki://{slug}` matching draft slug.

**Recommended API surface** (from RESEARCH discretion — planner may rename fields):
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiCompileSummary {
    pub wiki_root: std::path::PathBuf,
    pub pages_written: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped_user_edit: usize,
    pub cleaned: usize,
}

pub async fn compile_wiki_for_source(
    store: &Store,
    chat: &dyn ChatModel,
    embedder: &dyn Embedder,
    chunker: &ChunkerConfig,
    source_id: &str,
    wiki_root: &Path,
    wiki_enabled: bool,
) -> Result<WikiCompileSummary>;
```

**Prescribed order:** gates (enabled, kind≠WikiPage, Indexed) → analyze (fail-closed) → render → write w/ D-09 → stale cleanup FS+Store → scan-rebuild `index.md` → index each compile-set page from disk.

**Test pattern** (`learn.rs` L196–208 / L271–289 + existing wiki tempfile tests L480–503):
```rust
#[tokio::test]
async fn add_memory_uses_uuid_uri() {
    let store = Store::open_in_memory(4).unwrap();
    let embedder = MockEmbedder::new(4);
    let chunker = ChunkerConfig::default();
    // …
}
```
Wiki compile tests: `tempfile` wiki root + seed Indexed local source + `MockChatModel` + `MockEmbedder` + assert `wiki://` rows + second compile chunk count stable.

---

### `crates/insights/src/error.rs` (model)

**Analog:** `crates/memory/src/error.rs` — add `Index(#[from])` + hard-reject variants.

**Current insights errors** (`error.rs` L3–19):
```rust
#[derive(Debug, Error)]
pub enum InsightsError {
    #[error("source not indexed: {0}")]
    NotIndexed(String),
    #[error("no chunk text for source: {0}")]
    EmptySource(String),
    #[error("failed to parse tasks json: {0}")]
    InvalidTasksJson(String),
    #[error("failed to parse wiki json: {0}")]
    InvalidWikiJson(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("llm: {0}")]
    Llm(#[from] llm::LlmError),
    #[error("store: {0}")]
    Store(#[from] store::StoreError),
}
```

**Copy Index + domain rejects from memory** (`memory/src/error.rs` L3–18):
```rust
#[error("index: {0}")]
Index(#[from] indexer::IndexError),
#[error("memory not found: {0}")]
NotFound(String),
#[error("not a memory source: {0}")]
NotMemory(String),
```

**Add (discretion strings from RESEARCH):**
```rust
#[error("wiki is disabled")]
WikiDisabled,
#[error("cannot compile a WikiPage source: {0}")]
WikiPageInput(String),
#[error("index: {0}")]
Index(#[from] indexer::IndexError),
```

---

### `crates/insights/src/lib.rs` (config / barrel)

**Analog:** current re-exports L9–12 — extend with compile + summary.

```rust
pub use wiki::{
    analyze_source_for_wiki, render_wiki_pages, write_wiki_pages_to_dir, WikiAnalysis,
    WikiCompileResult, WikiConcept, WikiEntity, WikiPageDraft, WikiPageType,
};
```

**Evolve to also export:** `compile_wiki_for_source`, `WikiCompileSummary` (and any public helpers if tests need them). Keep `write_wiki_pages_to_dir` until tests migrate, or evolve in place and update `write_wiki_pages_to_dir_writes_tree`.

---

### `crates/insights/Cargo.toml` (config)

**Analog:** `crates/memory/Cargo.toml` L7–17 — production path deps.

```toml
[dependencies]
chunker = { path = "../chunker" }
embedder = { path = "../embedder" }
indexer = { path = "../indexer" }
ingest = { path = "../ingest" }
llm = { path = "../llm" }
store = { path = "../store" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

**Current insights** already has `llm`, `store`, `serde`, `serde_json`, `thiserror`, `sha2`, `hex`, and `tempfile`/`tokio` as dev-deps — **add** `chunker`, `embedder`, `indexer`, `ingest` only. Keep `sha2`/`hex` for slug helpers. No new registry crates.

---

### `src-tauri/src/commands/wiki.rs` (controller, request-response) — NEW

**Analog:** `src-tauri/src/commands/memory.rs` — thin async command, resolve deps from `AppState`, `.map_err(|e| e.to_string())`.

**Imports + async command pattern** (`memory.rs` L1–5, L45–59):
```rust
use memory::{add_memory, forget_memory, get_memory_content, list_memories, update_memory};
use store::Source;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn add_memory_cmd(
    content: String,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    add_memory(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &content,
        title.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}
```

**Wiki command (prescribed):**
```rust
use insights::{compile_wiki_for_source, WikiCompileSummary};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn compile_wiki_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<WikiCompileSummary, String> {
    let cfg = state.config();
    let wiki_root = state
        .config_path
        .parent()
        .ok_or_else(|| "config_path has no parent".to_string())?
        .join("wiki");
    compile_wiki_for_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &source_id,
        &wiki_root,
        cfg.wiki.enabled,
    )
    .await
    .map_err(|e| e.to_string())
}
```

**Wiki root resolution analog:** `state.rs` L38–40 / L104–108 — `config_path = app_data/config.json`; siblings `skills_dir`, `hooks_dir`. Prefer lazy `parent()/wiki` (RESEARCH); do **not** create wiki root at startup until first compile.

**AppState accessors to use** (`state.rs` L132–142): `config()`, `embedder()`, `chat()`, fields `store`, `chunker`, `config_path`.

**No progress events** (D-17) — return summary only; do not emit index-progress style events.

---

### `src-tauri/src/commands/mod.rs` (route)

**Analog:** existing module registration L1–8, L29–31.

```rust
mod memory;
// …
pub use memory::{
    add_memory_cmd, forget_memory_cmd, get_memory_content_cmd, list_memories_cmd, update_memory_cmd,
};
```

**Add:**
```rust
mod wiki;
pub use wiki::compile_wiki_cmd;
```

---

### `src-tauri/src/lib.rs` (route)

**Analog:** `generate_handler!` list L60–116 — append next to memory cmds.

```rust
list_memories_cmd,
get_memory_content_cmd,
forget_memory_cmd,
update_memory_cmd,
add_memory_cmd,
// add:
compile_wiki_cmd,
```

Ensure `use commands::*` / re-exports already pull from `mod.rs` (same as memory).

---

## Shared Patterns

### Normalize-to-Document (synthetic URI)

**Source:** `crates/memory/src/learn.rs` L110–120  
**Apply to:** Wiki page index loop  
**Rule:** `uri` == Store source `id`; kind = `SourceKind::WikiPage`; hash = `ingest::hash_text` of **full on-disk markdown**.

### Thin Tauri command

**Source:** `src-tauri/src/commands/memory.rs`  
**Apply to:** `commands/wiki.rs`  
**Rule:** Library does work; command maps `Result<T, InsightsError>` → `Result<T, String>`; pull `store`/`chat`/`embedder`/`chunker` from `AppState`.

### Source delete (chunks + row)

**Source:** `src-tauri/src/commands/library.rs` L22–27 + `store.rs` L187–202  
**Apply to:** Stale wiki page cleanup  
**Rule:** Always `delete_chunks_for_source` then `delete_source`. Do **not** copy `forget_memory`’s delete-only path.

### Fail-closed analyze, ordered best-effort persist

**Source:** Phase 09 `analyze_source_for_wiki` (zero FS until `Ok`) + RESEARCH persist order  
**Apply to:** `compile_wiki_for_source`  
**Rule:** Gates + analyze fail closed. After analyze Ok, write→cleanup→index is ordered best-effort; first IO/Index error → `Err` (re-compile recovers via hash skip). No multi-doc transaction.

### Hand-rolled frontmatter peek

**Source:** `crates/agent/src/skills.rs` L39–51 + wiki `build_frontmatter` schema  
**Apply to:** D-09 skip, D-07/D-08 cleanup, D-03 catalog titles  
**Rule:** No `serde_yaml`; line/fence scan only.

### Config gate

**Source:** `crates/config/src/types.rs` L129–141 (`WikiConfig.enabled` default false)  
**Apply to:** D-15 at library entry (also readable in Tauri via `state.config().wiki.enabled`)  
**Rule:** Hard-reject before any disk/index work.

### Error boundary at IPC

**Source:** All `commands/*.rs`  
**Apply to:** `compile_wiki_cmd`  
**Rule:** `.map_err(|e| e.to_string())` only; typed errors live in `insights`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| *(none)* | — | — | Closest analogs exist for every target file |

**Partial / first-of-kind helpers (implement beside analogs):**
- `rebuild_index_md_from_disk` — no prior vault scanner; shape from `build_index_markdown` + `skills::load_skills_from_dir` read_dir loop
- `is_generated_frontmatter` / `frontmatter_sources_contain` — first wiki policy peeks; copy skills fence style

---

## Anti-Patterns Checklist (planner/executor)

| Avoid | Why | Do instead |
|-------|-----|------------|
| Write `compiled.index_markdown` as vault `index.md` | Clobbers multi-source catalog | Scan-rebuild after write/cleanup |
| Put `content_hash` in YAML | D-10; fights Obsidian edits | Hash only in `Document` |
| `delete_source` without chunk delete | Orphan vec/FTS | `delete_chunks_for_source` + `delete_source` |
| Open SQLite in insights | Single DB owner | Store APIs only |
| Index `index.md` | Pollutes RAG | Three content dirs only |
| Hash draft body after D-09 skip | Wrong RAG / false change | `read_to_string` on-disk path |
| Compile `WikiPage` inputs | Loop | `InsightsError::WikiPageInput` |
| Progress events this phase | D-17 | Summary struct only |
| Orchestration only in Tauri | Untestable | Library `compile_wiki_for_source` |

---

## Metadata

**Analog search scope:** `crates/insights`, `crates/memory`, `crates/indexer`, `crates/ingest`, `crates/store`, `crates/config`, `crates/agent/src/skills.rs`, `src-tauri/src/commands`, `src-tauri/src/state.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/index_ops.rs`  
**Files scanned:** ~20 primary + prior phase PATTERNS style  
**Pattern extraction date:** 2026-07-21  
**CONTEXT locks honored:** D-01…D-17 (scan-rebuild, sacred user edits, DB-only hash, thin command, hard gates)
