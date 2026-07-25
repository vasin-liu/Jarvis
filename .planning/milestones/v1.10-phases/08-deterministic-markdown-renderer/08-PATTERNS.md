# Phase 08: Deterministic Markdown renderer - Pattern Map

**Mapped:** 2026-07-19
**Files analyzed:** 3
**Analogs found:** 3 / 3

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/insights/src/wiki.rs` | utility | transform | `crates/chunker/src/lib.rs` (+ `crates/retriever/src/rrf.rs` for test shape; `crates/ingest/src/hash.rs` for sha2) | exact |
| `crates/insights/src/lib.rs` | config | transform | `crates/ingest/src/lib.rs` (same barrel pattern as current `insights` lib) | exact |
| `crates/insights/Cargo.toml` | config | — | `crates/ingest/Cargo.toml` | exact |

**Anti-analogs (do not copy I/O/LLM):** `crates/insights/src/summarize.rs`, `crates/insights/src/tasks.rs` — use only for colocated `#[cfg(test)]` placement and crate module naming. Renderer must not take `Store` / `ChatModel`.

**UI:** N/A — `08-UI-SPEC.md` confirms no frontend files this phase.

## Pattern Assignments

### `crates/insights/src/wiki.rs` (utility, transform)

**Analog (primary):** `crates/chunker/src/lib.rs` — pure input→output, serde DTO types, private helpers, `#[cfg(test)] mod tests` in same file.

**Analog (hash):** `crates/ingest/src/hash.rs` — workspace `sha2` + `hex` for `e-{hash6}`.

**Analog (enum → snake_case string):** `crates/agent/src/hooks.rs` (`HookEvent`) or `crates/store/src/types.rs` (`SourceKind::as_str`) — frontmatter `type:` must use explicit `as_str`, not `Debug`.

**Imports pattern** (chunker lines 1–9 — serde DTOs for pure module):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkDraft {
    pub ord: i64,
    pub text: String,
    pub loc: String,
    pub token_count: i64,
}
```

**Adapt for wiki types** (from plan Task 2 + CONTEXT D-09; keep Serialize/Deserialize for Phase 09 JSON):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiPageType {
    SourceSummary, // frontmatter: source_summary
    Entity,
    Concept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiAnalysis { /* summary, entities, concepts */ }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiPageDraft {
    pub slug: String,           // e.g. "entities/acme"
    pub title: String,
    pub page_type: WikiPageType,
    pub body_markdown: String,  // YAML frontmatter + body; NO content_hash (D-08)
    pub source_uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCompileResult {
    pub pages: Vec<WikiPageDraft>,
    pub index_markdown: String,
}
```

**Serde snake_case + as_str pattern** (`crates/agent/src/hooks.rs` lines 7–22):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    BeforeToolCall,
    AfterToolCall,
    BeforeAnswer,
}

impl HookEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::BeforeToolCall => "before_tool_call",
            HookEvent::AfterToolCall => "after_tool_call",
            HookEvent::BeforeAnswer => "before_answer",
        }
    }
}
```

Copy this for `WikiPageType::as_str()` → `"source_summary" | "entity" | "concept"` (D-09). Do **not** put `WikiPageType` in `store` — `SourceKind::WikiPage` is a different concept (Phase 07).

**Hash pattern** (`crates/ingest/src/hash.rs` lines 1–6):
```rust
use sha2::{Digest, Sha256};

pub fn hash_text(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    hex::encode(digest)
}
```

Adapt to private `hash6(name: &str) -> String` taking first 6 hex chars of UTF-8 bytes of the **original name** (not the filtered slug). Prefer a private helper inside `wiki.rs` over depending on `ingest` (keep insights → ingest coupling out of scope).

**Core transform pattern** (`crates/chunker/src/lib.rs` lines 26–30 — public pure fn, no I/O):
```rust
/// Split plain text into chunks with line-range locations (`L{start}-L{end}`).
pub fn chunk_text(text: &str, config: &ChunkerConfig) -> Vec<ChunkDraft> {
    if text.is_empty() {
        return Vec::new();
    }
    // ...
}
```

**Core transform pattern (secondary)** (`crates/retriever/src/rrf.rs` lines 4–6 — infallible pure fn):
```rust
/// Reciprocal Rank Fusion across ranked hit lists (higher score = better).
pub fn reciprocal_rank_fusion(lists: &[&[ChunkHit]], rrf_k: f64, final_k: usize) -> Vec<ChunkHit> {
```

Phase 08 signature (locked):
```rust
pub fn render_wiki_pages(
    analysis: &WikiAnalysis,
    source_uri: &str,
    source_title: &str,
) -> WikiCompileResult;  // NOT Result — no thiserror needed
```

**String-build helpers (hand-rolled, no YAML crate):** private `fn frontmatter(...)`, `fn slugify(...)`, `fn uniquify_slug(...)`, `fn wikilink(path, display)` using `format!` — same style as chunker’s private `split_long_line` / `flush` closures. No analog exists for YAML frontmatter in-tree; follow RESEARCH frontmatter helper + D-08…D-10.

**Error handling:** None at API boundary (infallible). Do **not** extend `InsightsError` for Phase 08. Internal string sanitization (display `|` / `]`, title `"`) is discretionary strip/replace — no `Result`.

**Validation:** Unit assertions on output shape (slug prefixes, frontmatter keys, wikilink substrings, index sections). No schema middleware.

**Testing pattern** (`crates/chunker/src/lib.rs` lines 123–144 — colocated, sync `#[test]`, no tokio/tempfile):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_no_chunks() {
        let cfg = ChunkerConfig::default();
        assert!(chunk_text("", &cfg).is_empty());
    }

    #[test]
    fn short_text_becomes_single_chunk() {
        // construct input → call pure fn → assert fields
    }
}
```

Also mirror `crates/retriever/src/rrf.rs` lines 33–55 (local fixture helpers + focused asserts).

**Wave 0 tests to place in `wiki.rs` (CONTEXT/RESEARCH; override plan Task 2 bare `[[Acme]]` / `content_hash`):**
- `render_always_emits_source_summary` — empty entities/concepts; `sources:`; `generated: true`; no `content_hash`; index has `# Wiki` + `## Sources`, omits `## Entities`
- `render_links_entities_bidirectional` — `[[entities/…|…]]` + backlink `[[sources/…|{title}]]`
- concepts mirror of entity linking
- `cjk_name_uses_hash_slug` — `entities/e-` + 6 hex
- `slug_collision_suffix` — same-dir `-2` by input order; cross-dir not colliding
- `index_omits_empty_sections`

**Do not copy from summarize/tasks:**
```rust
// ANTI-PATTERN for wiki.rs — crates/insights/src/summarize.rs lines 8–12
pub async fn summarize_source(
    store: &Store,
    chat: &dyn ChatModel,
    source_id: &str,
) -> Result<String> {
```

---

### `crates/insights/src/lib.rs` (config / barrel, transform)

**Analog:** `crates/ingest/src/lib.rs` lines 1–9; also current insights barrel lines 1–7.

**Current insights barrel** (`crates/insights/src/lib.rs` lines 1–7):
```rust
mod error;
mod summarize;
mod tasks;

pub use error::{InsightsError, Result};
pub use summarize::summarize_source;
pub use tasks::extract_tasks_from_source;
```

**Ingest barrel pattern** (`crates/ingest/src/lib.rs` lines 1–9):
```rust
mod document;
mod error;
mod hash;
mod loader;

pub use document::Document;
pub use error::{IngestError, Result};
pub use hash::hash_text;
pub use loader::{load_path, resolve_existing_path};
```

**Apply:**
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
```

Keep `truncate_chars` as-is (summarize/tasks). Do not re-export private slug helpers unless tests need them via `pub(crate)`.

**Retriever-style split module** (`crates/retriever/src/lib.rs` lines 1–6) is optional precedent if slugify later moves to `wiki/slug.rs` — CONTEXT allows discretion; prefer single `wiki.rs` for Phase 08 simplicity.

---

### `crates/insights/Cargo.toml` (config)

**Analog:** `crates/ingest/Cargo.toml` lines 7–12.

```toml
[dependencies]
sha2 = { workspace = true }
hex = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
calamine = { workspace = true }
```

**Apply to insights** — add only:
```toml
sha2 = { workspace = true }
hex = { workspace = true }
```

Keep existing `llm`, `store`, `serde`, `serde_json`, `thiserror`, and `tokio` dev-dep. Do **not** add `serde_yaml`, `slug`, pinyin, or `tempfile` for this phase. Do **not** bump workspace `sha2`/`hex` pins.

## Shared Patterns

### Pure library transform (no I/O)
**Source:** `crates/chunker/src/lib.rs`, `crates/retriever/src/rrf.rs`  
**Apply to:** `wiki.rs` only  
- Sync function, returns owned DTOs  
- No `Store`, `ChatModel`, `std::fs`, `async`  
- Colocated `#[cfg(test)]` — no new `crates/insights/tests/` unless private-API isolation is required (crate has no `tests/` dir today)

### Workspace crypto helpers
**Source:** `crates/ingest/src/hash.rs` + `crates/ingest/Cargo.toml`  
**Apply to:** slug empty/CJK fallback in `wiki.rs` + deps in `insights/Cargo.toml`  
- `Sha256::digest(name.as_bytes())` + `hex::encode` + truncate to 6  
- Hash original name UTF-8, not filtered slug

### Snake_case type literals via `as_str`
**Source:** `crates/agent/src/hooks.rs`, `crates/store/src/types.rs` (`SourceKind`)  
**Apply to:** frontmatter `type:` field emission  
- Serde `rename_all = "snake_case"` on enum for Phase 09 JSON  
- Explicit `as_str()` for YAML frontmatter strings (D-09)

### Module re-export surface
**Source:** `crates/insights/src/lib.rs`, `crates/ingest/src/lib.rs`  
**Apply to:** `lib.rs` modification  
- `mod wiki;` + selective `pub use` of public types/fn

### TDD / unit layer
**Source:** `.cursor/rules/tdd-goal-driven.mdc` (chunker/RRF class of tests)  
**Apply to:** all Wave 0 tests in `wiki.rs`  
- Failing tests first; `cargo test -p insights wiki`  
- No E2E this phase (`e2e-required.mdc` / UI-SPEC: not user-facing)

### Error types
**Source:** `crates/insights/src/error.rs`  
**Apply to:** **none** for Phase 08 renderer API  
- Leave `InsightsError` unchanged; pure render is infallible per RESEARCH A1

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| *(hand-rolled YAML frontmatter helper inside `wiki.rs`)* | utility | transform | No in-repo YAML/frontmatter builder; use RESEARCH `format!` sketch + D-08…D-10 — still implement inside `wiki.rs`, not a separate crate |
| *(Obsidian `[[path\|display]]` builder)* | utility | transform | No wikilink helper in tree; trivial `format!("[[{path}|{display}]]")` + display sanitize |

These are not separate deliverable files — they are private helpers with no existing analog; planner should follow RESEARCH code examples, not invent new crates.

## Metadata

**Analog search scope:** `crates/insights/`, `crates/ingest/`, `crates/chunker/`, `crates/retriever/`, `crates/agent/src/hooks.rs`, `crates/store/src/types.rs`, `crates/config/src/types.rs` (WikiConfig — already shipped Phase 07, not modified here), `.cursor/rules/`, plan Task 2 interfaces  
**Files scanned:** ~12 primary + grep hits for serde/wiki  
**Pattern extraction date:** 2026-07-19  
**Locked overrides vs plan Task 2:** `[[path|display]]` (D-01), omit `content_hash` (D-08), tree `sources/`/`entities/`/`concepts/` (D-04), index omit-empty (D-12)
)
