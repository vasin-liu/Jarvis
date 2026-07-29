# Phase 15: Overlap Scoring API - Context

**Gathered:** 2026-07-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a **backend-only** `related_documents(source_id, top_n?)` API that ranks other indexed sources by hybrid retrieval overlap (reuse vector+FTS+RRF), excludes the seed source, and returns a truncated top-N list. No Library UI, no MCP, no citation trust changes — those are Phases 16–19.

</domain>

<decisions>
## Implementation Decisions

### Seed query text
- **D-01:** Prefer `Source.summary` when non-empty (after trim); otherwise use `Source.title`.
- **D-02:** Blank/whitespace-only summary falls back to title (same as missing summary).
- **D-03:** Empty title (and no usable summary) → return **empty list**, not an error. Missing `source_id` still returns not-found / error as usual.

### Weak affinity / empty results
- **D-04:** Default `top_n = 5` when omitted.
- **D-05:** After excluding the seed `source_id`, if no other sources remain → return `[]` (success, not error).
- **D-06:** No score threshold — only truncate to top-N after ranking; weak hits may appear in the list.

### Return payload
- **D-07:** Each hit includes `source_id`, `title`, `kind`, and `snippet`.
- **D-08:** `snippet` = text from the top-ranked `ChunkHit` for that source, truncated to a fixed length (planner picks exact char/byte cap; keep deterministic).
- **D-09:** Do **not** expose raw RRF/`ChunkHit.score` on the public related-docs DTO in this phase.

### Source kind policy
- **D-10:** No kind filter — Wiki, Memory, Local, Lark, Cursor, etc. may all appear as neighbors when ranked.
- **D-11:** Wiki (and any other) seed uses the same summary→title→empty-list rules; do not special-case Wiki seed to always return empty.

### Claude's Discretion
- Exact snippet truncation length and whether truncation is char- or grapheme-aware
- Exact crate placement (`retriever` helper vs thin `src-tauri` wrapper) as long as hybrid retrieve is reused and `store` remains sole SQLite owner
- Internal retrieve `top_k` / over-fetch strategy before collapsing chunk hits → unique sources (must be enough to fill `top_n` after seed exclude + dedupe)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / requirements
- `.planning/ROADMAP.md` — Phase 15 goal, success criteria, REL-02
- `.planning/REQUIREMENTS.md` — REL-02 (overlap scoring API)
- `.planning/PROJECT.md` — v1.11 Active Requirements (Related docs + MCP)

### Research (v1.11)
- `.planning/research/SUMMARY.md` — Phase 15 as first building block; reuse hybrid retriever
- `.planning/research/ARCHITECTURE.md` — related-docs as thin ranker over existing retrieve
- `.planning/research/FEATURES.md` — related-docs feature shape / deferred link graph
- `.planning/research/STACK.md` — stack constraints (MCP later; this phase is Rust API only)

### Code anchors
- `crates/retriever/src/retrieve.rs` — `retrieve()` hybrid entry; returns `Vec<ChunkHit>`
- `crates/retriever/src/rrf.rs` — RRF fusion (`rrf_k` default 60)
- `crates/store/src/types.rs` — `Source`, `SourceKind`, `ChunkHit` (`text`, `source_id`, `score`)
- `crates/store` — sole SQLite owner; source lookup by id for seed metadata

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `retriever::retrieve` — already hybrid (vector + FTS + RRF); primary engine for overlap scoring
- `Store` source getters / `ChunkHit` — seed metadata + snippet text without new tables

### Established Patterns
- Trait inject `Embedder` for query embedding (same as RAG/agent search)
- Map domain errors to `String` only at Tauri boundary if a command is exposed this phase
- Prefer library-crate logic + thin shell; unit/integration tests with `MockEmbedder` + tempfile DB

### Integration Points
- Collapse `ChunkHit` rows → unique `source_id` (best/first hit wins for snippet)
- Exclude seed `source_id` before applying `top_n`
- Phase 16 will call this API from Library UI; keep DTO stable for that consumer

</code_context>

<specifics>
## Specific Ideas

- Empty seed text → empty list (graceful), not a hard error — keeps UI/MCP callers simple later
- Snippet from winning chunk text is enough for Phase 16 list preview; no separate highlight API

</specifics>

<deferred>
## Deferred Ideas

- Library related-docs panel / empty copy / loading / open-neighbor — **Phase 16** (REL-01,03,04)
- MCP stdio / tools — **Phases 17–18**
- Citation trust gate — **Phase 19** (TRUST-*)
- Explicit link graph / co-citation edges — out of milestone (research deferred)
- Score threshold / “strong affinity only” — deferred unless product revisits; this phase is top-N only
- Exposing numeric overlap scores in UI — deferred; not in Phase 15 DTO

</deferred>

---

*Phase: 15-Overlap Scoring API*
*Context gathered: 2026-07-25*
