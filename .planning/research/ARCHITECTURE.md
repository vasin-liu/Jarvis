# Architecture Research

**Domain:** Local-first knowledge hub — optional Wiki Compile Layer (v1.10)
**Researched:** 2026-07-17
**Confidence:** HIGH

## Standard Architecture

### System Overview

Wiki compile is an **optional side path** off the existing insights → Document → indexer pipeline. It does not replace hybrid retrieval or citation authority.

```
┌─────────────────────────────────────────────────────────────┐
│  Presentation — React (Library / Settings)                   │
│  wiki-enabled toggle · 生成笔记 · 导出 Wiki · wiki_page list │
├─────────────────────────────────────────────────────────────┤
│  IPC / Shell — src-tauri                                     │
│  commands/wiki.rs (or library) · insights_ops glue · e2e    │
├─────────────────────────────────────────────────────────────┤
│  Domain crates                                               │
│  ┌──────────────┐  ┌────────────┐  ┌─────────────────────┐ │
│  │ insights     │  │ indexer    │  │ config (WikiConfig) │ │
│  │ wiki.rs      │→ │ index_doc  │  └─────────────────────┘ │
│  │ wiki_export  │  └─────┬──────┘                          │
│  └──────┬───────┘        │                                   │
│         │ LLM analyze    │ Document + SourceKind::WikiPage   │
├─────────┴────────────────┴───────────────────────────────────┤
│  store (sole SQLite) · {app_data}/wiki/*.md on disk          │
│  RAG/retriever unchanged — WikiPage = additional sources     │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `insights::wiki` | Analyze source → `WikiAnalysis`; render Markdown drafts; write `{app_data}/wiki/`; build Documents | New `wiki.rs` (+ optional `wiki_export.rs`); reuse `truncate_chars`, `ChatModel` |
| `insights_ops` | Optional auto-compile after summarize when `wiki.auto_on_insights` | Extend `maybe_run_insights_for_source` |
| `store` | Persist `SourceKind::WikiPage`; list/upsert like any source | Enum + `as_str`/`parse`; no new tables required |
| `indexer` | Chunk + embed wiki Markdown via existing `index_document` | **Unchanged API** — called with `SourceKind::WikiPage` |
| `config::WikiConfig` | Feature flag + auto-on-insights | Nested in `AppConfig`, `#[serde(default)]`, both default `false` |
| Tauri wiki commands | Thin IPC: compile / export zip / list pages | `commands/wiki.rs` or extend `commands/library.rs` |
| Library / Settings UI | Enable flag; per-source compile; export; kind label | `LibraryView`, Settings accordion, `sourceDisplay.ts` |
| RAG / retriever | Retrieve wiki chunks as normal hits | **No code change** — citations still prefer original sources in product policy |

## Recommended Project Structure

```
crates/
├── store/src/types.rs          # + SourceKind::WikiPage ("wiki_page")
├── config/src/types.rs         # + WikiConfig { enabled, auto_on_insights }
├── insights/
│   ├── src/lib.rs              # mod wiki; re-export compile_wiki_for_source
│   ├── src/wiki.rs             # analyze, render, write, index wiring
│   ├── src/wiki_export.rs      # export_wiki_zip (Obsidian stub)
│   └── tests/wiki_compile.rs   # tempfile + MockChatModel + MockEmbedder
src-tauri/src/
├── commands/wiki.rs            # compile_wiki, export_wiki_zip, list_wiki_pages
├── insights_ops.rs             # optional auto_on_insights hook
└── e2e.rs                      # Mock chat JSON sequence for wiki compile
src/
├── views/LibraryView.tsx       # 生成笔记 / 导出 Wiki (gated)
├── lib/sourceDisplay.ts        # wiki_page → Wiki / 笔记页
└── (Settings)                  # wiki.enabled toggle
e2e/specs/wiki.spec.ts          # enable → compile → list → export
{app_data}/wiki/                # runtime tree: index.md, entities/, concepts/, …
```

### Structure Rationale

- **`crates/insights` (not a new crate):** Wiki is LLM-derived structured output from indexed sources — same concern as summarize/tasks. Avoids a workspace crate for one feature.
- **Disk tree under `wiki/` + DB as `WikiPage`:** Dual representation matches the product (Obsidian export + RAG). Stable `wiki://{slug}` URIs make recompile idempotent via `content_hash`.
- **Thin Tauri commands:** Mirror `summarize_source_cmd` / memory list patterns; all I/O and LLM logic stay in crates for `cargo test`.
- **Feature flag in config:** Default-off preserves v1.8 UX; E2E can toggle without schema migrations beyond kind string.

## Architectural Patterns

### Pattern 1: Normalize-to-Document (existing)

**What:** Every ingest path builds `ingest::Document` + `SourceKind` and calls `indexer::index_document`.
**When to use:** Wiki pages after Markdown write — same as Memory, Lark, Cursor.
**Trade-offs:** Wiki pages appear in Library and RRF like any source (desired); risk of citation noise if UI does not distinguish kinds (mitigate with labels + product rule: prefer original chunks).

**Example:**
```rust
// After writing wiki_root/{slug}.md
let doc = Document { uri: format!("wiki://{slug}"), title, text: body, content_hash: sha256(body) };
index_document(store, embedder, chunker, doc, SourceKind::WikiPage).await?;
```

### Pattern 2: Insights-style LLM → structured parse

**What:** `ChatModel::complete` with bare-JSON system prompt; parse into `WikiAnalysis`; fail closed on parse error (no partial disk write).
**When to use:** `analyze_source_for_wiki` — parallel to `summarize_source` / `extract_tasks_from_source`.
**Trade-offs:** Text-protocol JSON is Mock-friendly and provider-agnostic; brittle if model wraps fences (strip fences / first `{...}`).

### Pattern 3: Pure render then impure persist

**What:** `render_wiki_pages(analysis, …) -> WikiCompileResult` is pure (unit-tested); `compile_wiki_for_source` owns FS + index.
**When to use:** Always for wiki Markdown generation.
**Trade-offs:** Slightly more types (`WikiPageDraft`); much easier TDD and slug/wikilink contracts.

### Pattern 4: Generated-page overwrite guard

**What:** Overwrite on-disk pages only when frontmatter has `generated: true`; skip re-embed when `content_hash` unchanged.
**When to use:** Recompile / auto_on_insights loops.
**Trade-offs:** Protects light user edits in v1.10 without full sync; does not support deep Obsidian bidirectional merge (out of scope).

## Data Flow

### Request Flow — Manual compile

```
Library "生成笔记" (wiki.enabled)
    ↓ invoke compile_wiki({ sourceId })
Tauri command → gate on WikiConfig.enabled
    ↓
insights::compile_wiki_for_source
    ├─ store: load Indexed source + chunk text
    ├─ ChatModel: analyze → WikiAnalysis (JSON)
    ├─ render_wiki_pages → drafts + index.md body
    ├─ FS: write {app_data}/wiki/{slug}.md (generated: true)
    └─ indexer::index_document × N (SourceKind::WikiPage, uri wiki://…)
    ↓
{ pageCount } → Library refresh (list_sources / list_wiki_pages)
```

### Request Flow — Auto after insights

```
index_ops success → insights_ops::maybe_run_insights_for_source
    ↓ (existing) summarize / extract_tasks if sync flags
    ↓ (new) if wiki.enabled && wiki.auto_on_insights
compile_wiki_for_source(…)  // same path as manual
```

### Request Flow — Export

```
"导出 Wiki" → dialog destPath → export_wiki_zip(wiki_root, dest)
    ↓ zip wiki_root/** + .obsidian/app.json stub
{ path } success
```

### Key Data Flows

1. **Compile:** Indexed source chunks → LLM JSON → Markdown files → Documents → vectors/FTS as `WikiPage`.
2. **RAG:** Retriever hybrid search may hit `wiki_page` chunks; product policy keeps original-source citations authoritative when both exist (no retriever fork required in v1.10).
3. **Idempotent recompile:** Same mock/LLM output → same body hash → indexer skip; same `wiki://{slug}` upsert.
4. **List:** `list_wiki_pages` filters `SourceKind::WikiPage` (or reads disk index.md); Library uses existing `list_sources` + kind label.

### State Management

```
AppConfig.wiki (persisted config.json)
    ↓ load at startup / Settings save
UI gates buttons + IPC rejects when disabled
    ↓
{app_data}/wiki/ + Store sources (WikiPage) — rebuildable from originals
```

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Personal desktop (Jarvis default) | Per-source compile on demand; sequential LLM; fine |
| Large libraries (1k+ sources) | Batch compile with progress events (reuse index-progress pattern); rate-limit auto_on_insights |
| Multi-device / sync | Out of scope — zip export is the sync unit; no cloud wiki store |

### Scaling Priorities

1. **First bottleneck:** LLM calls per source on auto_on_insights — keep default `false`; batch UI later.
2. **Second bottleneck:** Embed cost on many entity pages — rely on `content_hash` skip; avoid deleting/recreating URIs.

## Anti-Patterns

### Anti-Pattern 1: New SQLite owner or wiki tables outside store

**What people do:** Open a second DB for wiki graph, or write wiki rows from `insights` via raw SQL.
**Why it's wrong:** Breaks single-DB-owner invariant; schema drift.
**Do this instead:** `SourceKind::WikiPage` + existing sources/chunks/FTS/vec; only `store` opens connections.

### Anti-Pattern 2: Replace RAG citations with wiki pages

**What people do:** Point Q&A only at compiled summaries.
**Why it's wrong:** Violates core value — answers must cite user originals when available.
**Do this instead:** Index wiki as *additional* sources; keep retriever/RAG contracts; E2E qa journeys must stay green.

### Anti-Pattern 3: Fat Tauri command with LLM + FS + zip

**What people do:** Implement compile/export inside `#[tauri::command]`.
**Why it's wrong:** Untestable without WebView; duplicates insights patterns.
**Do this instead:** Logic in `crates/insights`; commands map `Result` → `String` like `summarize_source_cmd`.

### Anti-Pattern 4: New workspace crate for wiki alone

**What people do:** Add `crates/wiki` prematurely.
**Why it's wrong:** Extra deps/boundaries for one optional feature; insights already owns LLM-over-source.
**Do this instead:** `insights::wiki` module; extract a crate only if a later milestone adds graph sync / MCP surface that outgrows insights.

### Anti-Pattern 5: Dual-write without hash / overwrite user edits

**What people do:** Always rewrite every `.md` and force re-embed.
**Why it's wrong:** Wastes embeds; clobbers Obsidian edits.
**Do this instead:** `generated: true` guard + `content_hash` indexer skip; stable `wiki://` URIs.

### Anti-Pattern 6: Live LLM in unit/E2E

**What people do:** Call Ollama/cloud in CI for wiki JSON.
**Why it's wrong:** Flaky, slow, non-deterministic.
**Do this instead:** `MockChatModel` fixed JSON; extend `e2e.rs` mock sequences for compile command.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| ChatModel (Mock / Ollama / Cloud) | Trait injection via AppState | Same resolver as summarize; Mock returns wiki JSON in E2E |
| Embedder | Passed into `compile_wiki_for_source` | Reuse per-agent / global embedder; MockEmbedder in tests |
| Filesystem `{app_data}/wiki` | Path from Tauri app data dir | Created on first compile; zip reads this tree only |
| Obsidian (export only) | Zip + `.obsidian/app.json` stub | No live Obsidian API; unidirectional export |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| UI ↔ Tauri | `invoke` camelCase payloads | Gate on `wiki.enabled`; clear error if disabled |
| Tauri ↔ insights | Direct async fn calls | Pass `&Store`, `&dyn ChatModel`, `&dyn Embedder`, `wiki_root` |
| insights ↔ indexer | `index_document` | Do not reimplement chunk/embed |
| insights ↔ store | Read source/chunks; list WikiPage | No Connection outside store |
| insights_ops ↔ wiki | Optional post-summarize hook | Only if `auto_on_insights` |
| RAG ↔ wiki sources | Indirect via Store search | No special retriever mode in v1.10 |
| Frontend display ↔ kinds | `sourceDisplay.ts` | Exhaustive kind labels include `wiki_page` |

### New vs Modified (explicit)

| Status | Artifact |
|--------|----------|
| **New** | `crates/insights/src/wiki.rs`, optional `wiki_export.rs`, insights wiki tests |
| **New** | `src-tauri/src/commands/wiki.rs` (or equivalent), `e2e/specs/wiki.spec.ts` |
| **New** | Runtime `{app_data}/wiki/` tree |
| **Modified** | `store` `SourceKind` (+ exhaustive matches / frontend labels) |
| **Modified** | `config` `AppConfig` + `WikiConfig` |
| **Modified** | `insights/src/lib.rs` exports |
| **Modified** | `insights_ops.rs` for auto_on_insights |
| **Modified** | Library / Settings UI + IPC registration + `e2e.rs` mocks |
| **Unchanged** | Retriever RRF algorithm, RAG ask prompt core, SQLite open policy, Document→index contract |

## Suggested Build Order (Phases 07+)

Dependency-aware order aligned with plan Tasks 1–7; treat as GSD phases after v1.9 gate:

| Phase | Focus | Depends on | Verify |
|-------|--------|------------|--------|
| **07** | `SourceKind::WikiPage` + `WikiConfig` + `sourceDisplay` | v1.9 E2E green | `cargo test -p store -p config`; Vitest labels |
| **08** | Pure `render_wiki_pages` (no LLM/IO) | 07 kinds optional but useful for later | `cargo test -p insights` render tests |
| **09** | `analyze_source_for_wiki` + JSON parse | 08 types; `ChatModel` | MockChatModel unit tests |
| **10** | `compile_wiki_for_source` write + `index_document` | 07–09; indexer; app_data path | Integration test tempfile; hash idempotency |
| **11** | Tauri IPC + Library/Settings UI (flagged) | 10 | Vitest gate; manual/IPC smoke |
| **12** | `export_wiki_zip` + UI export | 10 disk tree | Zip unit test; dialog path |
| **13** | E2E `wiki.spec.ts` + full-ui “hidden when off” | 11–12; e2e mocks | `npm run test:e2e:local` |

**Do not** start UI (11) before compile indexes (10). **Do not** wire `auto_on_insights` until 10 is green. Export (12) can stub in 11 then complete.

## Sources

- Implementation plan: `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`
- Existing insights: `crates/insights/src/{lib,summarize,tasks}.rs`
- Insights glue: `src-tauri/src/insights_ops.rs`, `src-tauri/src/commands/library.rs`
- Source kinds: `crates/store/src/types.rs`
- Analogous normalize path: `crates/memory` → `index_document(..., SourceKind::Memory)`
- Project invariants: `AGENTS.md`, `.cursor/rules/e2e-required.mdc`, CLAUDE.md architecture notes

---
*Architecture research for: Wiki Compile Layer (Jarvis v1.10)*
*Researched: 2026-07-17*
