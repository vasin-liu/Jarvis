# Phase 15: Overlap Scoring API - Pattern Map

**Mapped:** 2026-07-25
**Files analyzed:** 3 (2 modified/new source + 1 inline test module)
**Analogs found:** 3 / 3

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/retriever/src/related.rs` | service | transform | `crates/rag/src/ask.rs` | exact |
| `crates/retriever/src/lib.rs` | config (module wiring) | — | `crates/rag/src/lib.rs` | exact |
| `crates/retriever/src/related.rs` (`#[cfg(test)]`) | test | batch | `crates/retriever/src/retrieve.rs` (tests mod) | exact |

**Out of scope this phase (no edits):** `crates/store/*`, `crates/retriever/src/retrieve.rs`, `crates/retriever/src/rrf.rs`, `src-tauri/*`, frontend, E2E.

## Pattern Assignments

### `crates/retriever/src/related.rs` (service, transform)

**Analog:** `crates/rag/src/ask.rs` — same orchestration: `retrieve` → enrich via `Store::get_source` → truncate chunk text → return DTO without exposing raw RRF scores to callers.

**Imports pattern** (mirror `retrieve.rs` + `ask.rs` deps):

```1:5:crates/retriever/src/retrieve.rs
use embedder::Embedder;
use store::{ChunkHit, Store};

use crate::error::{Result, RetrieveError};
use crate::rrf::reciprocal_rank_fusion;
```

For `related.rs`, use:

```rust
use embedder::Embedder;
use store::{Source, SourceKind, Store};

use crate::error::Result;
use crate::retrieve::{retrieve, RetrieverConfig};
```

**Public DTO pattern** (analog: `Citation` in rag — stable fields, no score):

```3:11:crates/rag/src/types.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: i64,
    pub source_id: String,
    pub source_title: String,
    pub source_uri: String,
    pub loc: String,
    pub excerpt: String,
}
```

Phase 15 `RelatedSource` should follow the same shape discipline (D-07/D-09): `source_id`, `title`, `kind`, `snippet` only — **no `score` field**. Serde optional this phase (Phase 16 IPC adds it); use `Debug, Clone, PartialEq` minimum like `ChunkHit`.

**Core orchestration pattern** (analog: `ask` calls `retrieve` then post-processes hits):

```11:18:crates/rag/src/ask.rs
pub async fn ask(
    store: &Store,
    embedder: &dyn Embedder,
    chat: &dyn ChatModel,
    retriever: &RetrieverConfig,
    question: &str,
) -> Result<AskResponse> {
    let hits = retrieve(store, embedder, question, retriever).await?;
```

`related_sources` copies this shell: async fn, `&Store`, `&dyn Embedder`, `Result<T>` — but seed lookup + empty short-circuit happen **before** `retrieve`.

**Seed metadata lookup** (analog: per-hit enrichment in `hits_to_citations`):

```77:96:crates/rag/src/ask.rs
fn hits_to_citations(store: &Store, hits: &[ChunkHit]) -> Result<Vec<Citation>> {
    let mut out = Vec::with_capacity(hits.len());
    for hit in hits {
        let source = store.get_source(&hit.source_id)?;
        let excerpt = if hit.text.chars().count() > 240 {
            let truncated: String = hit.text.chars().take(240).collect();
            format!("{truncated}…")
        } else {
            hit.text.clone()
        };
        out.push(Citation {
            chunk_id: hit.chunk_id,
            source_id: hit.source_id.clone(),
            source_title: source.title,
            source_uri: source.uri,
            loc: hit.loc.clone(),
            excerpt,
        });
    }
    Ok(out)
}
```

Adapt for related rollup:
- Walk RRF-ordered hits once; **skip** seed `source_id`.
- Track seen `source_id` with `HashSet` or `IndexMap` — **first hit wins** snippet (D-08).
- Call `store.get_source(neighbor_id)?` for `title` + `kind` only.
- Apply `truncate_snippet` at **160 chars** (research discretion), not 240.

**Snippet truncation pattern** (analog: agent tool excerpt — char-safe, ellipsis):

```68:74:crates/agent/src/tools.rs
                let excerpt = if hit.text.chars().count() > 200 {
                    let truncated: String = hit.text.chars().take(200).collect();
                    format!("{truncated}…")
                } else {
                    hit.text.clone()
                };
```

Use `RELATED_SNIPPET_CHARS = 160` and U+2026 `…` — **never byte slice** (regression documented in `crates/rag/tests/citation_excerpt.rs`).

**Seed query helper** (new logic; analog for `Source` fields):

```75:87:crates/store/src/types.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    pub uri: String,
    pub title: String,
    pub content_hash: String,
    pub indexed_at: Option<i64>,
    pub status: IndexStatus,
    pub error: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}
```

```rust
fn seed_query(source: &Source) -> Option<String> {
    if let Some(s) = source.summary.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return Some(s.to_string());
    }
    let title = source.title.trim();
    if title.is_empty() { None } else { Some(title.to_string()) }
}
```

**Empty seed short-circuit** (critical divergence from `retrieve`):

```32:35:crates/retriever/src/retrieve.rs
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }
```

`related_sources` must return `Ok(vec![])` when `seed_query` is `None` (D-03) — **do not call** `retrieve`.

**Over-fetch config** (adjust defaults before `retrieve`):

```15:23:crates/retriever/src/retrieve.rs
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
```

Clone default, then set `final_k = max(24, top_n * 5)`, `vector_k = vector_k.max(final_k)`, `fts_k = fts_k.max(final_k)`, keep `rrf_k = 60.0`.

**Error handling pattern** (reuse existing `RetrieveError`; no new enum variant):

```3:13:crates/retriever/src/error.rs
#[derive(Debug, Error)]
pub enum RetrieveError {
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("embed error: {0}")]
    Embed(#[from] embedder::EmbedError),
    #[error("empty query")]
    EmptyQuery,
}

pub type Result<T> = std::result::Result<T, RetrieveError>;
```

Missing seed propagates via `store.get_source(source_id)?` → `StoreError::NotFound` → `RetrieveError::Store` (D-03 vs not-found distinction).

**NotFound source** (store pattern):

```78:90:crates/store/src/store.rs
    pub fn get_source(&self, id: &str) -> Result<Source> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, kind, uri, title, content_hash, indexed_at, status, error, summary
             FROM sources WHERE id = ?1",
            [id],
            Self::row_to_source,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_string()),
            other => StoreError::Sqlite(other),
        })
    }
```

**Rollup dedupe pattern** (analog: RRF dedupe by `chunk_id`, but rollup is by `source_id`):

```7:16:crates/retriever/src/rrf.rs
pub fn reciprocal_rank_fusion(lists: &[&[ChunkHit]], rrf_k: f64, final_k: usize) -> Vec<ChunkHit> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    let mut by_id: HashMap<i64, ChunkHit> = HashMap::new();

    for list in lists {
        for (rank, hit) in list.iter().enumerate() {
            let contribution = 1.0 / (rrf_k + rank as f64 + 1.0);
            *scores.entry(hit.chunk_id).or_default() += contribution;
            by_id.entry(hit.chunk_id).or_insert_with(|| hit.clone());
```

Related rollup: iterate fused hits in order; `if hit.source_id == seed_id { continue }`; `if seen.insert(hit.source_id.clone())` then push `RelatedSource`; `take(top_n)`.

**Lock scope** (follow `retrieve` — embed then search; no custom long-held Store lock):

```37:45:crates/retriever/src/retrieve.rs
    let query_vec = embedder.embed(&[trimmed.to_string()]).await?;
    let vector_hits = store.search_vector(&query_vec[0], config.vector_k)?;

    let fts_query = build_fts_query(trimmed);
    let fts_hits = if fts_query.is_empty() {
        Vec::new()
    } else {
        store.search_fts(&fts_query, config.fts_k)?
    };
```

Sequence: `get_source(seed)` → build query → `retrieve(...)` (which handles embed+search internally).

---

### `crates/retriever/src/lib.rs` (config, module wiring)

**Analog:** `crates/rag/src/lib.rs`

```1:7:crates/rag/src/lib.rs
mod ask;
mod error;
mod types;

pub use ask::{ask, ask_stream};
pub use error::{RagError, Result};
pub use types::{AskResponse, Citation, OrchestrationStepInfo, ToolCallInfo};
```

Current retriever `lib.rs`:

```1:6:crates/retriever/src/lib.rs
mod error;
mod rrf;
mod retrieve;

pub use error::{Result, RetrieveError};
pub use retrieve::{retrieve, RetrieverConfig};
```

**Target change:**

```rust
mod error;
mod related;
mod rrf;
mod retrieve;

pub use error::{Result, RetrieveError};
pub use related::{related_sources, RelatedSource};
pub use retrieve::{retrieve, RetrieverConfig};
```

Keep module order alphabetical-ish (`related` before `retrieve` is fine); re-export only public API surface.

---

### `crates/retriever/src/related.rs` — `#[cfg(test)]` (test, batch)

**Analog:** `crates/retriever/src/retrieve.rs` integration test fixture

```75:106:crates/retriever/src/retrieve.rs
#[cfg(test)]
mod tests {
    use super::*;
    use chunker::ChunkerConfig;
    use embedder::MockEmbedder;
    use indexer::index_path;
    use std::fs;

    #[tokio::test]
    async fn hybrid_retrieve_finds_indexed_content() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("doc.md");
        fs::write(&file, "Rust vector search with sqlite-vec and FTS5.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();

        let hits = retrieve(
            &store,
            &embedder,
            "vector search",
            &RetrieverConfig::default(),
        )
        .await
        .unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].text.contains("vector"));
    }
```

**Dev-deps** (already present — no Cargo.toml change):

```12:17:crates/retriever/Cargo.toml
[dev-dependencies]
chunker = { path = "../chunker" }
indexer = { path = "../indexer" }
embedder = { path = "../embedder" }
tempfile = "3"
tokio = { workspace = true }
```

**Test cases to mirror** (from RESEARCH validation map):

| Test name | Assert |
|-----------|--------|
| `related_excludes_seed_and_rolls_up_by_source` | Seed absent; one entry per neighbor `source_id` |
| `related_empty_seed_text` | Blank title+summary → `Ok(vec![])` |
| `related_missing_source` | Bad id → `Err` with NotFound |
| `related_prefers_summary` | Non-empty summary used as query (neighbor with shared FTS term ranks) |
| `related_respects_top_n` | Default 5; clamp; `top_n=0` → `[]` |
| `related_snippet_truncate` | Long chunk → 160 chars + `…`; valid UTF-8 |
| `related_only_seed` | Single-source corpus → `Ok(vec![])` |

**MockEmbedder pitfall** (from research): fixture pairs must share **distinctive FTS keywords** in summary/body — vector-only hash similarity is unreliable. Copy multi-file indexing from rag tests:

```138:149:crates/rag/src/ask.rs
    #[tokio::test]
    async fn ask_returns_citations_for_indexed_doc() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kb.sqlite");
        let file = dir.path().join("kb.md");
        fs::write(&file, "Jarvis uses sqlite-vec for embeddings.").unwrap();

        let store = Store::open(&db, 4).unwrap();
        let embedder = MockEmbedder::new(4);
        index_path(&store, &embedder, &ChunkerConfig::default(), &file)
            .await
            .unwrap();
```

Index ≥2 markdown files: seed + neighbor sharing keyword (e.g. `"overlap-scoring-alpha"`), plus disjoint control corpus.

**UTF-8 truncation regression pattern** (optional integration test analog):

```44:49:crates/rag/tests/citation_excerpt.rs
    assert!(!resp.citations.is_empty(), "expected at least one citation");
    // A truncated excerpt must remain valid UTF-8 and end with the ellipsis.
    assert!(resp
        .citations
        .iter()
        .any(|c| c.excerpt.ends_with('…') || !c.excerpt.is_empty()));
```

Apply same assertion to `RelatedSource.snippet` when seed neighbor has long Chinese chunk text.

---

## Shared Patterns

### Error propagation (Store → RetrieveError)

**Source:** `crates/retriever/src/error.rs`  
**Apply to:** `related.rs` — use `?` on `store.get_source` and `retrieve`; no `String` mapping in library crate.

```rust
// Missing source_id
let seed = store.get_source(source_id)?; // RetrieveError::Store(StoreError::NotFound(...))
```

### Hybrid retrieve entry (do not fork RRF)

**Source:** `crates/retriever/src/retrieve.rs`  
**Apply to:** `related_sources` only via `retrieve(...)` — never call `search_vector` / `search_fts` / `reciprocal_rank_fusion` directly from `related.rs`.

```26:62:crates/retriever/src/retrieve.rs
pub async fn retrieve(
    store: &Store,
    embedder: &dyn Embedder,
    query: &str,
    config: &RetrieverConfig,
) -> Result<Vec<ChunkHit>> {
    // ... embed, search_vector, search_fts, RRF ...
    Ok(reciprocal_rank_fusion(
        &[&vector_hits, &fts_hits],
        config.rrf_k,
        config.final_k,
    ))
}
```

### Char-safe snippet truncation

**Source:** `crates/agent/src/tools.rs` (200 chars), `crates/rag/src/ask.rs` (240 chars)  
**Apply to:** `truncate_snippet` in `related.rs` at **160 chars**

```rust
const RELATED_SNIPPET_CHARS: usize = 160;

fn truncate_snippet(text: &str) -> String {
    if text.chars().count() <= RELATED_SNIPPET_CHARS {
        return text.to_string();
    }
    let taken: String = text.chars().take(RELATED_SNIPPET_CHARS).collect();
    format!("{taken}…")
}
```

### Trait-injected Embedder in async tests

**Source:** `crates/retriever/src/retrieve.rs` tests  
**Apply to:** all `related_*` integration tests — `MockEmbedder::new(dim)` only; no live FastEmbed/Ollama.

### TDD / validation (project rules)

**Source:** `.cursor/rules/tdd-goal-driven.mdc`  
**Apply to:** write failing `cargo test -p retriever related` tests before implementation; phase gate `cargo test -p retriever`.

**Source:** `.cursor/rules/e2e-required.mdc`  
**Apply to:** **skip E2E** this phase (backend-only); Phase 16 owns panel E2E.

### Sole SQLite owner

**Source:** project architecture / `karpathy-guidelines.mdc`  
**Apply to:** `related.rs` calls `Store` methods only — no `rusqlite`, no new connections.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All in-scope files have strong analogs |

**Net-new logic (no single-file analog):** seed query `summary → title → Ok([])` and source-level rollup with seed exclusion. Planner should compose from `ask.rs` enrichment + `retrieve.rs` hybrid + manual rollup loop per RESEARCH sketch.

---

## Metadata

**Analog search scope:** `crates/retriever/`, `crates/rag/`, `crates/agent/`, `crates/store/`, `crates/insights/`, `.cursor/rules/`  
**Files scanned:** ~18  
**Pattern extraction date:** 2026-07-25

**Key planner notes:**
- API name: implement `related_sources` (CONTEXT says `related_documents`; same behavior).
- D-06 overrides ROADMAP “empty on weak affinity” — no score threshold in Phase 15.
- Do **not** use `Store::source_chunk_text` for seed query (superseded by D-01..D-03).
- No Tauri command, no serde on DTO required until Phase 16.
