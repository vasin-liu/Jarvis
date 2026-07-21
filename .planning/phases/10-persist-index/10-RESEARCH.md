# Phase 10: Persist + index - Research

**Researched:** 2026-07-21  
**Domain:** Wiki vault FS persist + `WikiPage` indexing via existing ingest→`index_document` path  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### index.md rebuild
- **D-01:** After each successful compile, **rebuild `index.md` by scanning the wiki tree** (do not surgically patch a prior catalog).
- **D-02:** Scan **only** `sources/`, `entities/`, and `concepts/` `*.md`. Ignore root clutter; never treat `index.md` as a catalog entry source.
- **D-03:** Catalog link display names come from frontmatter **`title`**, rendered as `[[path|title]]`. If `title` is missing, fall back to the file slug.
- **D-04:** Within each section, sort entries by **path/slug lexicographic order** (stable, independent of compile order).
- **D-05:** Omit empty sections (same shape as Phase 08 `index_markdown`: `# Wiki` + `## Sources` / `## Entities` / `## Concepts` only when non-empty).

#### Stale page cleanup
- **D-06:** On re-compile of a source, **delete generated pages that disappeared** from this compile’s output for that source, and **remove the corresponding `wiki://` indexed sources**.
- **D-07:** A page is in scope for cleanup when its frontmatter **`sources` contains this source’s URI** and it was not produced by the current compile.
- **D-08:** **Never delete** pages that lack `generated: true` (user-curated / edited notes are retained even if still tagged with the source URI).
- **D-09:** If the compile wants to write a slug that already exists **without** `generated: true`, **skip writing** that file (keep the user version). Still proceed with other pages.

#### content_hash
- **D-10:** `content_hash` lives **only on the Document / DB indexer path**. Do **not** inject `content_hash` into Markdown frontmatter (Phase 08 omission stays permanent for v1.10).
- **D-11:** Hash input is the **full on-disk Markdown body** as written (including existing YAML: `title`, `type`, `sources`, `generated`).
- **D-12:** When write is skipped due to a user-edited file (D-09), still **index from the existing on-disk file** via normal hash-skip so Library/RAG reflect the user’s note.
- **D-13:** Root **`index.md` is not indexed** as a `WikiPage`. Only pages under `sources/` · `entities/` · `concepts/` enter the index.

#### Phase-10 API surface
- **D-14:** Ship **`compile_wiki_for_source` (library)** plus a **thin Tauri command** with **no UI** this phase (Library wiring is Phase 11). Prefer a dedicated command module (e.g. `commands/wiki.rs`) if that matches shell layout.
- **D-15:** When `wiki.enabled == false`, **hard-reject** compile (clear error); no disk writes, no index upserts.
- **D-16:** When the input source kind is **`WikiPage`**, **hard-reject** with a clear error (no analyze / write / index) — compile-loop guard.
- **D-17:** On success, the Tauri command returns a **compact summary**: pages written, created/updated/skipped(user-edit)/cleaned counts, and wiki root. **No** progress events in this phase.

### Claude's Discretion
- Exact Tauri command name and summary struct field names (as long as D-14…D-17 hold).
- Exact error variant/message strings for disabled wiki and WikiPage-input rejection.
- Whether cleanup+write+index run as one transactional-ish sequence vs best-effort with documented failure modes — prefer fail-closed where practical (align with Phase 09); planner/researcher decide feasibility against current indexer APIs.
- Test layout: integration tests with tempfile + MockChatModel + MockEmbedder (plan Task 4) vs additional unit tests for index rebuild/cleanup helpers.

### Deferred Ideas (OUT OF SCOPE)
- Library 「生成笔记」 + Settings wiki toggle UI — Phase 11
- Obsidian zip export — Phase 12
- E2E enable → compile → list → export + citation trust — Phase 13
- `auto_on_insights` wiring / bulk compile + progress events — Future (WIKI-F01/F02)
- Cross-corpus entity merge — Future (WIKI-F03)
- Progress-event streaming for compile — deferred (D-17 chose summary-only)

None — discussion stayed within phase scope (no folded todos)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIKI-04 | System writes pages under `{app_data}/wiki/` and indexes them through the existing ingest→index pipeline using stable `wiki://{slug}` URIs and `content_hash` skip (idempotent re-compile) | Evolve `write_wiki_pages_to_dir` with overwrite/skip/cleanup + scan-rebuild `index.md`; `compile_wiki_for_source` → `Document{uri: wiki://…}` → `index_document(..., SourceKind::WikiPage)`; hard gates D-15/D-16; integration tests prove hash skip + no WikiPage loop |
</phase_requirements>

## Summary

Phase 10 wires the already-shipped analyze→render path into a durable vault under `{app_data}/wiki/` and into the same SQLite index used by every other source. The critical product rules are: **scan-rebuild** `index.md` for multi-source vaults, **sacred user edits** (`generated: true` overwrite/cleanup only), **DB-only `content_hash`** (never re-introduce into frontmatter despite older plan text), and **hard rejection** of disabled wiki / `WikiPage` inputs so indexing cannot recurse. [CITED: `10-CONTEXT.md` D-01…D-17; `REQUIREMENTS.md` WIKI-04]

Existing building blocks are sufficient: `analyze_source_for_wiki` + `render_wiki_pages` + naive `write_wiki_pages_to_dir` (Phase 08/09), `indexer::index_document` hash-skip, `SourceKind::WikiPage`, and the `memory://` Document→index precedent. Gaps to implement: frontmatter-aware write policy, stale cleanup + Store delete, scan-rebuild index helper, `compile_wiki_for_source` orchestration, insights path deps (`indexer`/`ingest`/`embedder`/`chunker`), and thin `commands/wiki.rs`. [VERIFIED: `crates/insights/src/wiki.rs`; `crates/indexer/src/lib.rs`; `crates/memory/src/learn.rs`; `src-tauri/src/commands/`]

**Primary recommendation:** Implement `compile_wiki_for_source` in `crates/insights` as gate→analyze→render→policy-write→cleanup→scan-rebuild `index.md`→index-from-disk; expose `compile_wiki_cmd` in new `src-tauri/src/commands/wiki.rs`; prove with tempfile + MockChatModel + MockEmbedder integration tests. No new crates. E2E deferred to Phase 13.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Wiki enabled / WikiPage input gates | Library / domain (`insights`) | Tauri command maps `String` errors | D-15/D-16 must hold for any caller (tests, future UI, auto hooks) |
| Analyze → render | Library (`insights`) | — | Already shipped Phase 09/08 |
| Overwrite / skip / stale FS cleanup | Library (`insights`) | — | Frontmatter policy lives with vault writer |
| Scan-rebuild `index.md` | Library (`insights`) | — | Multi-source vault consistency (D-01) |
| `Document` + `content_hash` + `index_document` | Library (`insights` orchestration + `indexer`) | `store` sole DB | Normalize-to-Document; single DB owner |
| Delete stale `wiki://` sources | Library via `Store::delete_chunks_for_source` + `delete_source` | — | Match `remove_source` pattern |
| Thin IPC + wiki root resolution | Tauri shell (`commands/wiki.rs`) | — | No UI; resolve `{app_data}/wiki` from `config_path.parent()` |
| Library/Settings UX | Deferred Phase 11 | — | Out of scope |
| E2E journey | Deferred Phase 13 | — | No user-facing UI this phase |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `insights` (extend) | workspace path | `compile_wiki_for_source`, write policy, scan-rebuild | Already owns wiki analyze/render [VERIFIED: `crates/insights`] |
| `indexer::index_document` | path crate | Upsert + hash skip | Existing idempotent index path [VERIFIED: `crates/indexer/src/lib.rs` L26–28] |
| `ingest::Document` + `hash_text` | path crate | URI/title/text/hash shape | Same as Memory [VERIFIED: `crates/ingest/src/document.rs`, `hash.rs`] |
| `embedder::Embedder` / `MockEmbedder` | path crate | Embeddings for new/changed pages | Trait injection / tests |
| `chunker::ChunkerConfig` | path crate | Chunk before embed | Required by `index_document` |
| `store::SourceKind::WikiPage` | path crate | Kind + CRUD/delete | Shipped Phase 07 [VERIFIED: `crates/store/src/types.rs`] |
| `config::WikiConfig.enabled` | path crate | Hard gate | Defaults false [VERIFIED: `crates/config/src/types.rs` L129–142] |
| `llm::MockChatModel` | path crate | 「笔记编译」 happy-path JSON | Phase 09 [VERIFIED: `09-01-SUMMARY.md`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | `"3"` (registry **3.27.0**) | Temp wiki root + DB in tests | Already insights dev-dep [VERIFIED: `cargo search tempfile`; `crates/insights/Cargo.toml`] |
| `tokio` | workspace | `#[tokio::test]` async compile | Already insights / memory tests |
| `std::fs` | std | Read/write/delete markdown | Prefer over walkdir for 3 shallow dirs |
| Hand-rolled frontmatter peek | — | Detect `generated: true`, `title`, `sources` | Avoid `serde_yaml` (deprecated; Phase 08/STACK) [CITED: `.planning/research/STACK.md`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand frontmatter peek | `serde_yaml` / `yaml-rust` | Rejected — STACK + Phase 08 forbid; schema is tiny & stable |
| Scan-rebuild index | Merge-patch prior `index.md` | Rejected — D-01 locked |
| Second vector store / LanceDB | — | Forbidden — single DB owner [CITED: REQUIREMENTS Out of Scope] |
| Put write+index only in `src-tauri` | Library API | Rejected — D-14 + untestable without Tauri |

**Installation (path deps only — no new registry crates):**

```toml
# crates/insights/Cargo.toml — add production deps (mirror crates/memory):
chunker = { path = "../chunker" }
embedder = { path = "../embedder" }
indexer = { path = "../indexer" }
ingest = { path = "../ingest" }
```

**Version verification:** `tempfile = "3.27.0"` via `cargo search` (2026-07-21). No new npm packages. No `zip`/`walkdir` this phase (Phase 12). [VERIFIED]

## Package Legitimacy Audit

> No new external packages. Path-workspace crates only.

| Package | Registry | Verdict | Disposition |
|---------|----------|---------|-------------|
| *(none)* | — | — | N/A |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
Tauri command compile_wiki_cmd (no UI)
    │  wiki_root = config_path.parent()/wiki
    │  enabled = AppConfig.wiki.enabled
    ▼
compile_wiki_for_source (insights)
    │
    ├─ D-15: enabled? else Err (no I/O)
    ├─ load source; D-16: kind != WikiPage? else Err
    ├─ status Indexed? else NotIndexed (existing)
    │
    ├─ analyze_source_for_wiki  ──ChatModel──► WikiAnalysis
    │         └─ fail → Err (WIKI-05 / Phase 09 gate; zero writes)
    │
    ├─ render_wiki_pages(analysis, source.uri, source.title)
    │         └─ WikiCompileResult { pages[], index_markdown }  // index_markdown IGNORED for disk
    │
    ├─ for each draft page:
    │     if path exists && !generated:true → skip write (D-09), still queue for index (D-12)
    │     else → write body_markdown to wiki_root/{slug}.md
    │
    ├─ stale cleanup (D-06..D-08):
    │     scan sources|entities|concepts *.md whose frontmatter sources⊇input_uri
    │       && generated:true && slug ∉ current compile set
    │     → fs::remove_file + delete_chunks_for_source + delete_source(wiki://slug)
    │
    ├─ rebuild_index_md_from_disk (D-01..D-05) → write wiki_root/index.md
    │
    └─ for each page in compile set (written or user-skip):
          read on-disk bytes → Document {
            uri: wiki://{slug}, title, text: full md, content_hash: hash_text(text)
          }
          index_document(..., SourceKind::WikiPage)  // hash skip if unchanged
    │
    ▼
WikiCompileSummary { wiki_root, written, created, updated, skipped_user_edit, cleaned }
```

### Recommended Project Structure

```
crates/insights/src/
  wiki.rs              # extend: compile_wiki_for_source, write policy, scan-rebuild, cleanup helpers
  error.rs             # add WikiDisabled / WikiPageInput / Index(#[from])
  lib.rs               # export compile + summary types
crates/insights/Cargo.toml  # path deps: indexer, ingest, embedder, chunker
src-tauri/src/commands/
  wiki.rs              # NEW thin compile_wiki_cmd
  mod.rs               # mod wiki; re-export
src-tauri/src/lib.rs   # register invoke handler
```

Do **not** put orchestration solely in `insights_ops.rs` — that module is auto-insights glue; library ownership stays in `insights` so tests need no Tauri. Optional: one-liner helper in `insights_ops` later for `auto_on_insights` (out of scope).

### Pattern 1: Normalize-to-Document (Memory precedent)

**What:** Build `ingest::Document` with synthetic URI scheme, then `index_document(..., kind)`.  
**When to use:** Always for wiki pages.  
**Example:** [VERIFIED: `crates/memory/src/learn.rs` L110–120]

```rust
let uri = format!("wiki://{slug}"); // slug e.g. "entities/acme"
let text = std::fs::read_to_string(&path)?;
let doc = Document {
    uri: uri.clone(),
    title: title_from_frontmatter_or_slug(&text, &slug),
    text: text.clone(),
    content_hash: ingest::hash_text(&text), // D-11 full on-disk body
};
index_document(store, embedder, chunker, doc, SourceKind::WikiPage).await?;
```

Note: `index_document` uses `doc.uri` as `source.id` [VERIFIED: `crates/indexer/src/lib.rs` L24–28].

### Pattern 2: Thin Tauri command module

**What:** New `commands/wiki.rs` mirrors `commands/memory.rs`: call library, `.map_err(|e| e.to_string())`.  
**When to use:** D-14 — shell layout already has domain modules under `commands/`. [VERIFIED: `src-tauri/src/commands/mod.rs`]

### Pattern 3: Fail-closed analyze, best-effort persist

**What:** Keep Phase 09 ordering (no FS until `Ok(analysis)`). After analyze succeeds, FS+index is **ordered best-effort** (see Failure modes) — true cross-store transactions are not available.  
**When to use:** Aligns with discretion + Phase 09; indexer has no multi-doc transaction API. [VERIFIED: `index_document` per-call upsert]

### Anti-Patterns to Avoid

- **Writing `compiled.index_markdown` as the vault catalog:** Overwrites multi-source catalog with single-compile pages — violates D-01. Use scan-rebuild instead; may leave `WikiCompileResult.index_markdown` for unit tests of pure render only.
- **Injecting `content_hash` into frontmatter:** Older plan/FEATURES text; **overridden by D-10**. Keep Phase 08 tests that assert omission.
- **`delete_source` without `delete_chunks_for_source`:** Chunks + vec/FTS need explicit cleanup [VERIFIED: `commands/library.rs` L22–27; `delete_chunks_for_source`].
- **Opening SQLite in insights:** Forbidden — use Store APIs only [CITED: PITFALLS §8].
- **Watching `{app_data}/wiki`:** Compile-loop risk [CITED: PITFALLS §7] — document; do not add watch this phase.
- **Calling compile from `maybe_run_insights_for_source`:** `auto_on_insights` deferred; if touched, must skip WikiPage (D-16).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Chunk + embed + upsert | Custom wiki tables / second DB | `indexer::index_document` | Hash skip, embed cache, Failed status already handled |
| Content hashing | Ad-hoc digest | `ingest::hash_text` | Same SHA-256 hex as rest of app [VERIFIED: `crates/ingest/src/hash.rs`] |
| Source delete | Raw SQL in insights | `Store::delete_chunks_for_source` + `delete_source` | Vec/FTS consistency |
| Full YAML engine | `serde_yaml` | Tiny line/peek parser for `generated` / `title` / `sources` | STACK forbids serde_yaml; frontmatter shape fixed by Phase 08 |
| Progress streaming | Event bus for compile | Summary return (D-17) | UI phase later |
| Atomic FS+DB transaction | Custom journal | Ordered best-effort + re-compile recovery | No multi-resource txn in stack |

**Key insight:** Idempotency comes from stable `wiki://{slug}` IDs + `content_hash` skip, not from a new persistence layer.

## Common Pitfalls

### Pitfall 1: Single-compile `index.md` clobbers multi-source vault
**What goes wrong:** `write_wiki_pages_to_dir` today writes `compiled.index_markdown` (pages from this analysis only) [VERIFIED: `wiki.rs` L49–60, L261–301]. Second source compile wipes the first from the catalog.  
**Why it happens:** Phase 09 helper assumed one-shot tempfile trees.  
**How to avoid:** After writes/cleanup, **scan-rebuild** per D-01…D-05; stop using in-memory `index_markdown` for disk.  
**Warning signs:** `index.md` lists only the last compiled source.

### Pitfall 2: Dual-write / duplicate sources
**What goes wrong:** Unstable URIs or hashing title-only → new source rows every compile.  
**How to avoid:** URI = `wiki://{slug}` exactly matching draft slug (`sources/…`, `entities/…`, `concepts/…`); hash **full on-disk markdown** (D-11); rely on indexer skip when hash+Indexed match [VERIFIED: indexer L26–28].  
**Warning signs:** `list_sources` WikiPage count grows on identical re-compile.

### Pitfall 3: Compile loops via WikiPage inputs
**What goes wrong:** Indexed wiki pages fed back into compile/auto-insights.  
**How to avoid:** D-16 hard-reject at library entry; do not wire `auto_on_insights` this phase; keep wiki root out of watch folders [CITED: PITFALLS §7].  
**Warning signs:** Nested `wiki://` as analyze inputs; exploding page counts.

### Pitfall 4: Clobbering user-edited notes
**What goes wrong:** Blind overwrite of files where user removed `generated: true`.  
**How to avoid:** D-09 skip write; D-08 never delete; still index disk content (D-12).  
**Warning signs:** User prose disappears after re-compile.

### Pitfall 5: Partial write / index after successful analyze
**What goes wrong:** Mid-loop FS or embed failure leaves vault/index divergent.  
**How to avoid:** Analyze remains fail-closed; for persist use **prescribed order** (below); on index error return `Err` with clear message; recovery = re-run compile (idempotent). Do not soft-continue silent failures. Prefer staging only if planner finds cheap win — not required for v1.10.  
**Warning signs:** Files on disk without `WikiPage` rows (or vice versa for cleaned pages).

### Pitfall 6: Orphan chunks on cleanup
**What goes wrong:** `delete_source` alone leaves vec/FTS orphans if CASCADE insufficient for virtual tables.  
**How to avoid:** Always `delete_chunks_for_source` then `delete_source` (library `remove_source` pattern) [VERIFIED].

### Pitfall 7: Hashing rendered draft instead of on-disk file after skip
**What goes wrong:** D-09 skip but hash draft body → false “changed” or wrong RAG text.  
**How to avoid:** Always `read_to_string` the path that will be indexed (D-12).

### Pitfall 8: Indexing `index.md`
**What goes wrong:** Catalog becomes a WikiPage and pollutes RAG.  
**How to avoid:** D-13 — only the three content dirs.

## Code Examples / Integration Seams

### Current writer (must evolve)

```49:60:crates/insights/src/wiki.rs
pub fn write_wiki_pages_to_dir(compiled: &WikiCompileResult, wiki_root: &Path) -> Result<()> {
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

**Planner actions:** Replace/extend with: (1) D-09 skip logic, (2) do **not** write `compiled.index_markdown`, (3) call scan-rebuild after cleanup, (4) return counts for summary. Keep function for Phase 09 tests or evolve signature carefully and update `write_wiki_pages_to_dir_writes_tree`.

### Frontmatter shape (parse these fields)

```354:359:crates/insights/src/wiki.rs
fn build_frontmatter(title: &str, page_type_str: &str, source_uri: &str) -> String {
    let escaped_title = yaml_escape_double_quoted(title);
    let escaped_uri = yaml_escape_double_quoted(source_uri);
    format!(
        "---\ntitle: \"{escaped_title}\"\ntype: {page_type_str}\nsources: [\"{escaped_uri}\"]\ngenerated: true\n---\n"
    )
}
```

**Peek rules (prescriptive):**  
- `generated: true` present as a YAML line (trim) → writable/deletable.  
- Absence or `generated: false` → skip overwrite + skip cleanup delete.  
- `sources:` contains the input source URI as a quoted entry → ownership for cleanup (D-07).  
- `title: "..."` → catalog display (D-03).  
Do not require a full YAML AST.

### `index_document` hash skip

```26:28:crates/indexer/src/lib.rs
    if let Ok(existing) = store.get_source(&source_id) {
        if existing.content_hash == doc.content_hash && existing.status == IndexStatus::Indexed {
            return Ok(());
```

Identical re-compile → same on-disk bytes → same hash → **no chunk churn** (success criterion 3).

### Document shape

```3:8:crates/ingest/src/document.rs
pub struct Document {
    pub uri: String,
    pub title: String,
    pub text: String,
    pub content_hash: String,
}
```

### Store cleanup APIs

```106:110:crates/store/src/store.rs
    pub fn delete_source(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sources WHERE id = ?1", [id])?;
        Ok(())
    }
```

```187:202:crates/store/src/store.rs
    pub fn delete_chunks_for_source(&self, source_id: &str) -> Result<()> {
        // deletes vec_chunks + chunks_fts + chunks for source_id
```

Caller pattern [VERIFIED: `src-tauri/src/commands/library.rs`]:

```rust
store.delete_chunks_for_source(&wiki_uri)?;
store.delete_source(&wiki_uri)?;
```

### Wiki root resolution (Tauri)

`AppState` has `config_path` = `{app_data}/config.json` but no dedicated `wiki_dir` yet [VERIFIED: `state.rs` L115–128].  

**Prescribe:** `let wiki_root = state.config_path.parent().unwrap().join("wiki");` in the command (or add `wiki_dir` at init mirroring `skills_dir` — optional clarity, not required). Do not create wiki root at startup until first successful compile (lazy `create_dir_all`).

### Recommended library API (discretion filled)

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiCompileSummary {
    pub wiki_root: std::path::PathBuf,
    pub pages_written: usize,
    pub created: usize,           // new WikiPage rows (no prior source)
    pub updated: usize,           // re-indexed because hash changed
    pub skipped_user_edit: usize, // D-09
    pub cleaned: usize,           // files+sources removed
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

**Tauri (discretion):** `compile_wiki_cmd(source_id: String, state: State<AppState>) -> Result<WikiCompileSummary, String>` in `commands/wiki.rs`.

**Errors (discretion):**  
- `InsightsError::WikiDisabled` — `"wiki is disabled"`  
- `InsightsError::WikiPageInput(id)` — `"cannot compile a WikiPage source: {id}"`  
- `InsightsError::Index(#[from] indexer::IndexError)` — mirror `MemoryError`

### Prescribed persist order (failure modes)

1. **Gates** (enabled, kind, Indexed) — fail-closed, zero I/O.  
2. **Analyze** — fail-closed (WIKI-05).  
3. **Render** — pure.  
4. **Write loop** with D-09 — per-file; on IO error → `Err` (stop).  
5. **Stale cleanup** — FS then Store; on Store error after FS delete → `Err` (orphan file gone; re-compile ok).  
6. **Scan-rebuild `index.md`**.  
7. **Index each page** from disk — on first `IndexError` → `Err` (disk already updated; re-compile recovers via hash skip).  

Do **not** attempt rollback of successful embeds. Document in PLAN verification notes.

### Plan Task 4 overrides

`docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 4 still says merge/update `index.md` and implies frontmatter `content_hash` / “filled at write time”. **CONTEXT wins:** scan-rebuild; DB-only hash; summary return; thin command no UI. [CITED: Task 4 L280–338 vs `10-CONTEXT.md`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Plan: merge-patch `index.md` | Scan-rebuild from disk (D-01) | 2026-07-21 discuss | Multi-source vaults stay correct |
| Plan/FEATURES: `content_hash` in frontmatter | DB-only hash (D-10) | Phase 08 + Phase 10 lock | Obsidian edits don’t fight embedded hash |
| Phase 09: blind overwrite writer | `generated: true` policy + cleanup | Phase 10 | User notes sacred |
| UI+IPC in same Task 5 | Library+thin command now; UI Phase 11 | CONTEXT D-14 | Testable without FE |

**Deprecated/outdated:**  
- Writing `WikiCompileResult.index_markdown` to disk as the vault catalog.  
- Any Phase 10 work that adds `content_hash` YAML keys.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Slug URIs with slashes (`wiki://entities/acme`) are acceptable as `sources.id` TEXT PKs | Code Examples | If planner prefers `wiki://entities%2Facme`, must migrate — recommend keep slash form matching CONTEXT |
| A2 | Created/updated counts can be derived by `get_source` before/after `index_document` (skip = neither) | API | Slight ambiguity if Failed→Indexed; acceptable for D-17 |
| A3 | Shallow `std::fs::read_dir` of three dirs is enough (no `walkdir` yet) | Stack | Nested subdirs unsupported — matches Phase 08 flat layout |

**If empty beyond above:** Core integration claims are codebase-verified.

## Open Questions (RESOLVED)

1. **Should `write_wiki_pages_to_dir` remain as a naive helper?** — **RESOLVED**  
   - What we know: Phase 09 tests call it.  
   - Chosen answer: **Keep blind writer** for Phase 09 unit tests (`write_wiki_pages_to_dir_writes_tree`); compile path uses policy write + `rebuild_index_md_from_disk` and must not call the blind writer for the vault catalog. Dual writers accepted for v1.10.

2. **Cross-source entity slug collisions** (`entities/acme` from two docs) — **RESOLVED**  
   - What we know: uniquify is per-compile only; WIKI-F03 deferred.  
   - Chosen answer: **Out of scope (WIKI-F03)**; last writer with `generated: true` wins; documented in PLAN risks / deferred.

3. **Citation crowding** — **RESOLVED**  
   - Flagged in SUMMARY for Phase 10/13 — **not in WIKI-04**.  
   - Chosen answer: **Defer to Phase 13** (citation trust / E2E); no filter work in Phase 10. [CITED: SUMMARY Research Flags]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / cargo | Build + tests | ✓ | workspace MSRV 1.85 | — |
| `tempfile` (dev) | Integration tests | ✓ | 3.27.0 | already in insights |
| MockChatModel / MockEmbedder | Deterministic tests | ✓ | path crates | — |
| Live LLM / Feishu | — | N/A | — | Must not use (project rule) |
| E2E / tauri-driver | User journey | N/A this phase | — | Deferred Phase 13 |

**Step 2.6:** No blocking external services — code/config-only with in-process mocks.

**Missing dependencies with no fallback:** none

## Validation Architecture

> `workflow.nyquist_validation: true` in `.planning/config.json` [VERIFIED]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + `#[tokio::test]` + `tempfile` |
| Config file | none (crate-local `#[cfg(test)]` / optional `crates/insights/tests/`) |
| Quick run command | `cargo test -p insights compile_wiki -- --test-threads=1` |
| Full suite command | `cargo test -p insights -p indexer -p memory -- --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| WIKI-04 | Compile writes `sources|entities|concepts/*.md` + scan-built `index.md` under tempfile wiki root | integration | `cargo test -p insights compile_writes_files_and_indexes -- --exact` | ❌ Wave 0 |
| WIKI-04 | Indexed sources use `wiki://{slug}` + `SourceKind::WikiPage` | integration | same | ❌ Wave 0 |
| WIKI-04 | Second identical compile → same WikiPage count / unchanged chunk count (hash skip) | integration | `cargo test -p insights compile_idempotent_hash_skip` | ❌ Wave 0 |
| WIKI-04 / D-16 | `SourceKind::WikiPage` input → error, zero new files | unit/integration | `cargo test -p insights compile_rejects_wiki_page_input` | ❌ Wave 0 |
| D-15 | `wiki_enabled=false` → error, zero files / no new sources | unit/integration | `cargo test -p insights compile_rejects_when_disabled` | ❌ Wave 0 |
| D-01…D-05 | Scan-rebuild includes pages from two prior writes; lex sort; omit empty sections | unit | `cargo test -p insights rebuild_index_md_` | ❌ Wave 0 |
| D-06…D-09 | Stale generated page deleted + Store cleared; non-generated preserved + skip overwrite; still indexed | integration | `cargo test -p insights compile_cleanup_and_user_edit_` | ❌ Wave 0 |
| D-10 | On-disk frontmatter still omits `content_hash` after compile | unit | extend existing `frontmatter_omits_content_hash` / compile assert | ✅ exists (render); ❌ compile assert |
| WIKI-05 | Parse fail still zero files (regression) | unit | `cargo test -p insights wiki_parse_fail_` | ✅ |
| E2E | enable → compile → list | e2e | — | ❌ deferred Phase 13 |

### Sampling Rate

- **Per task commit:** `cargo test -p insights compile_wiki -- --test-threads=1`  
- **Per wave merge:** `cargo test -p insights -p indexer -- --test-threads=1`  
- **Phase gate:** Full insights+indexer green; **no** `npm run test:e2e:local` required this phase (no UI) — note deferral explicitly in PLAN/VERIFICATION

### Wave 0 Gaps

- [ ] Integration test scaffold: tempfile Store (dim=4) + seed Indexed local source + MockChatModel + MockEmbedder + `compile_wiki_for_source`  
- [ ] Unit tests for `rebuild_index_md_from_disk` / frontmatter peek helpers  
- [ ] User-edit + stale cleanup fixtures (pre-seed markdown files)  
- [ ] Framework install: none — add insights path deps only

*(Existing Phase 09 analyze/write/fail-closed tests remain; update if writer signature changes.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop; no new auth |
| V3 Session Management | no | — |
| V4 Access Control | no | Single-user app data dir |
| V5 Input Validation | yes | Reject WikiPage inputs; sanitize paths under `wiki_root` (refuse `..` in slug/relative paths when joining/scanning) |
| V6 Cryptography | no new | Use existing `ingest::hash_text` (SHA-256) — never hand-roll |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via slug | Tampering | Join under `wiki_root`; reject `..` / absolute components when scanning |
| Compile loop DoS | Denial of service | D-16 hard-reject WikiPage; no watch on wiki/ |
| Store ownership break | Tampering | No rusqlite outside `store` |
| Partial corrupt vault | Tampering | Analyze fail-closed; re-compile idempotent recovery |

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 10 |
|------|------------------------|
| `tdd-goal-driven.mdc` | Goal + acceptance first; failing tests before implementation; inject traits (`Embedder`, `ChatModel`); run relevant `cargo test` before done |
| `e2e-required.mdc` | User-facing needs E2E — **this phase has no UI**; E2E deferred to Phase 13 (`wiki.spec.ts`); do not claim user-facing done |
| `karpathy-guidelines.mdc` | Surgical diffs; no speculative abstractions; honor CONTEXT; fail-closed analyze already exists — extend don’t rewrite |
| `jarvis-stack.mdc` | Rust stable / Tauri 2; core logic in crates; thin Tauri commands; no pre-release deps |
| Single DB owner (AGENTS / architecture) | Only `store` opens SQLite; wiki pages are `sources` rows via indexer |
| No live LLM in tests | MockChatModel + MockEmbedder only |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/10-persist-index/10-CONTEXT.md` — locked D-01…D-17  
- `crates/insights/src/wiki.rs` — render/write/analyze  
- `crates/indexer/src/lib.rs` — hash skip  
- `crates/memory/src/learn.rs` — `memory://` Document precedent  
- `crates/store/src/store.rs` — delete APIs  
- `src-tauri/src/commands/{mod,memory,library}.rs` — command patterns  
- `.planning/config.json` — nyquist_validation true  
- `cargo search tempfile` — 3.27.0  

### Secondary (MEDIUM confidence)

- `.planning/research/{SUMMARY,FEATURES,STACK,PITFALLS}.md` — milestone risks  
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` Task 4 — superseded in part by CONTEXT  
- Phase 08/09 CONTEXT + SUMMARYs — prior contracts  

### Tertiary (LOW confidence)

- A1–A3 assumptions (URI slash form, count semantics, no walkdir)

## Metadata

**Confidence breakdown:**

| Area | Level | Reason |
|------|-------|--------|
| Standard stack | HIGH | Reuse existing crates; no new registry deps |
| Architecture | HIGH | CONTEXT locks behavior; seams verified in tree |
| Pitfalls | HIGH | PITFALLS + concrete writer/index bugs identified |
| Failure atomicity | MEDIUM | Best-effort order is reasoned, not empirically proven under fault injection |

**Research date:** 2026-07-21  
**Valid until:** ~2026-08-21 (stable brownfield APIs)

---

## RESEARCH COMPLETE
