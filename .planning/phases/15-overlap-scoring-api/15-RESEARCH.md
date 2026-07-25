# Phase 15: Overlap Scoring API - Research

**Researched:** 2026-07-25
**Domain:** Hybrid retrieval → source-level overlap ranking (Rust library API)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- Library related-docs panel / empty copy / loading / open-neighbor — **Phase 16** (REL-01,03,04)
- MCP stdio / tools — **Phases 17–18**
- Citation trust gate — **Phase 19** (TRUST-*)
- Explicit link graph / co-citation edges — out of milestone (research deferred)
- Score threshold / “strong affinity only” — deferred unless product revisits; this phase is top-N only
- Exposing numeric overlap scores in UI — deferred; not in Phase 15 DTO
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REL-02 | Related results are other **sources** (chunk hits rolled up by `source_id`, seed excluded), capped (~3–8), empty when affinity is weak | Implement rollup + seed exclude + `top_n` (default 5) via `retriever::related_sources` over existing hybrid `retrieve`. **CONTEXT D-06 overrides** REL-02 / ROADMAP SC#2 “empty when affinity is weak” for this phase — no score threshold; weak hits may appear. Affinity gating deferred. |
</phase_requirements>

## Summary

Phase 15 is a **backend-only** building block: a library function that, given a seed `source_id`, builds a query from **summary → title** (never chunk-body fallback — that ARCHITECTURE sketch is superseded by D-01..D-03), runs the existing hybrid retriever (vector + FTS + RRF), **excludes the seed**, collapses `ChunkHit`s to unique sources (first/best hit wins snippet), and returns up to `top_n` DTOs with `source_id`, `title`, `kind`, `snippet` — **no raw scores**.

**ROADMAP tension (locked override):** ROADMAP success criterion 2 and REQUIREMENTS REL-02 wording say “empty when affinity is weak.” **CONTEXT D-06 explicitly locks no score threshold** for this phase. Planner and verifier must treat empty-on-weak as **deferred**; Phase 15 success is top-N after exclude/rollup only.

**Primary recommendation:** Add `crates/retriever/src/related.rs` with `related_sources(...)` that reuses `retrieve` + `Store::get_source`; over-fetch `final_k`; unit/integration tests with `MockEmbedder` + tempfile DB. **No new crates, no Tauri IPC, no E2E** this phase (not user-facing).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Seed query text (summary→title) | API / Backend (`retriever`) | Database (`store.get_source`) | Domain rule D-01..D-03; metadata from Store |
| Hybrid overlap retrieve | API / Backend (`retriever::retrieve`) | Database (`search_vector` / `search_fts`) | Existing RRF path — do not fork |
| Source rollup + seed exclude | API / Backend (`retriever::related_sources`) | — | Pure post-process over `ChunkHit` |
| Snippet truncation | API / Backend (`retriever`) | — | Deterministic string helper; no FE |
| Public DTO (no score) | API / Backend | — | Stable shape for Phase 16 IPC |
| Library panel / navigate | Browser / Client | — | **Deferred Phase 16** |
| SQLite open / SQL | Database / Storage (`store` only) | — | Sole DB owner rule |

## Standard Stack

### Core

| Library / component | Version | Purpose | Why Standard |
|---------------------|---------|---------|--------------|
| `retriever::retrieve` + `RetrieverConfig` | workspace path crate | Hybrid vector+FTS+RRF | Already ships; default `rrf_k=60`, `final_k=8` [VERIFIED: crates/retriever/src/retrieve.rs] |
| `store::Store` / `Source` / `ChunkHit` | workspace | Seed metadata + hits | Sole SQLite owner [VERIFIED: crates/store] |
| `embedder::Embedder` + `MockEmbedder` | workspace | Query embedding | Trait inject; deterministic tests [VERIFIED: crates/embedder/src/mock.rs] |
| `thiserror` | workspace | `RetrieveError` | Existing crate pattern [VERIFIED: crates/retriever/src/error.rs] |
| `tokio` (dev) | workspace | `#[tokio::test]` | Existing retriever tests [VERIFIED: crates/retriever/Cargo.toml] |
| `tempfile` (dev) | 3 | Temp DB fixtures | Existing retriever/indexer tests [VERIFIED: crates/retriever/Cargo.toml] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `indexer` + `chunker` (dev-deps) | path | Seed fixture docs into Store | Integration tests only |
| `serde` | workspace `1` | IPC DTO | **Phase 16** when exposing Tauri command — optional this phase [VERIFIED: Cargo.toml] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hybrid retrieve + rollup | Centroid cosine / second vector table | Forbidden by milestone research; trust drift vs RAG [CITED: .planning/research/FEATURES.md] |
| Library-only API | Thin Tauri `list_related_sources` now | Useful for Phase 16, but CONTEXT = backend-only + no UI; keep shell out of Phase 15 |
| Score threshold for “weak empty” | Top-N only (D-06) | Threshold deferred; may ship weak neighbors until product revisits |

**Installation:** None — no new packages.

**Version verification:** No new registry deps. Rust toolchain observed: `rustc 1.97.0` (stable channel) [VERIFIED: local `rustc --version`]. Reuse existing workspace crates only.

## Package Legitimacy Audit

> No external packages to install this phase.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — zero new deps |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
Caller (cargo test / future Tauri Phase 16 / future MCP)
    │
    ▼
related_sources(store, embedder, source_id, top_n?)
    ├─ store.get_source(source_id) ──► NotFound → Err
    ├─ seed_query = trim(summary) or title
    │     └─ empty? → Ok([])                    [D-03]
    ├─ cfg.final_k = over_fetch(top_n)          [discretion]
    ├─ retrieve(store, embedder, seed_query, cfg)
    │     ├─ embedder.embed(query)
    │     ├─ store.search_vector / search_fts
    │     └─ RRF → Vec<ChunkHit> (ranked)
    ├─ walk hits in order:
    │     skip seed source_id
    │     first hit per source_id wins (snippet = hit.text)
    │     resolve title/kind via store.get_source(neighbor_id)
    └─ take top_n → Vec<RelatedSource>          [no score field]
```

### Recommended Project Structure

```
crates/retriever/
├── src/
│   ├── lib.rs          # + mod related; pub use related::{related_sources, RelatedSource}
│   ├── retrieve.rs     # existing hybrid entry (unchanged behavior)
│   ├── rrf.rs          # existing fusion
│   ├── error.rs        # reuse RetrieveError (+ Store/Embed/EmptyQuery)
│   └── related.rs      # NEW: related_sources + truncate_snippet + rollup
└── Cargo.toml          # no new deps; existing dev-deps suffice
```

**Do not** add SQLite SQL in `related.rs`. **Do not** open connections outside `store`.

### Pattern 1: Thin ranker over existing retrieve
**What:** `related_sources` is post-processing around `retrieve` — not a parallel search engine.  
**When to use:** Always for Phase 15.  
**Example:** See Code Examples below. Aligns with milestone research [CITED: .planning/research/ARCHITECTURE.md] but **query text follows D-01..D-03** (summary→title→`[]`), **not** `source_chunk_text` fallback from the older sketch.

### Pattern 2: First-hit rollup (stable order)
**What:** Iterate RRF-ordered `ChunkHit`s; skip seed; `HashSet`/`IndexMap` by `source_id`; keep first occurrence’s `text` as snippet; enrich with `get_source` for `title`/`kind`.  
**When to use:** Collapsing chunk hits → sources.  
**Why:** RRF already ranked best chunks first; first-seen = best snippet without re-scoring [ASSUMED: common RAG “group by doc” pattern; matches CONTEXT D-08].

### Pattern 3: Short-circuit empty seed text before `retrieve`
**What:** If summary+title yield empty/whitespace, return `Ok(vec![])` **without** calling `retrieve`.  
**When to use:** D-03.  
**Why:** `retrieve` returns `Err(RetrieveError::EmptyQuery)` on empty query [VERIFIED: crates/retriever/src/retrieve.rs] — that would violate D-03’s “empty list, not error.”

### Anti-Patterns to Avoid
- **Score threshold in Phase 15:** Contradicts D-06; do not implement ROADMAP SC#2 literally.  
- **`source_chunk_text` as seed query:** Superseded by locked D-01..D-03.  
- **Exposing `score` on DTO:** Violates D-09.  
- **Kind filters (Wiki/Memory demote):** Deferred (REL-F01); D-10/D-11 lock no filter.  
- **Logic in `src-tauri` only:** Untestable; fat crates rule [CITED: CLAUDE.md / ARCHITECTURE].  
- **Opening SQLite in retriever:** Forbidden — call `Store` methods only.  
- **E2E for this phase:** Not user-facing; E2E belongs to Phase 16/19 [VERIFIED: .cursor/rules/e2e-required.mdc].

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hybrid fusion | Custom score blend / cosine % | Existing `retrieve` + `reciprocal_rank_fusion` | Already tested; RRF is standard for uncalibrated lists [CITED: cormack.uwaterloo.ca/cormacksigir09-rrf] |
| Source metadata | Duplicate SQL in retriever | `Store::get_source` | Sole DB owner |
| Grapheme truncation crate | `unicode-segmentation` dep | `.chars().take(N)` like rag/agent/insights | Project convention; deterministic UTF-8 [VERIFIED: crates/rag/src/ask.rs, crates/agent/src/tools.rs] |
| Graph / edge table | Neo4j, petgraph, link store | On-demand retrieve rollup | Explicitly out of scope [CITED: .planning/REQUIREMENTS.md Out of Scope] |
| Affinity threshold math | Heuristic on RRF scores | Top-N only (D-06) | Locked; threshold deferred |

**Key insight:** Overlap quality for v1.11 is “same brain as RAG, rolled up by source” — not a new ranking science project.

## Discretion Recommendations (planner must bake these in)

### 1. Snippet truncation — **160 Unicode scalar values (chars), not bytes, not grapheme clusters**
- Cap: **`RELATED_SNIPPET_CHARS = 160`**.
- Algorithm: if `text.chars().count() > 160`, take `.chars().take(160)` and append `…` (U+2026), matching agent tool excerpts (`200`) / rag (`240`) style [VERIFIED: crates/agent/src/tools.rs, crates/rag/src/ask.rs].
- **Not grapheme-aware:** Avoid new deps; Chinese/emoji edge cases are acceptable for preview snippets [ASSUMED: product OK for Library preview].
- **Not bytes:** Byte slicing risks mid-UTF-8 panic/corruption.

### 2. Crate placement — **`retriever::related_sources`, no Tauri this phase**
- Primary: `crates/retriever/src/related.rs` + re-export from `lib.rs`.
- Naming: prefer **`related_sources`** (matches `Source` / research / STATE) over CONTEXT’s informal `related_documents` — same API, clearer domain name.
- Tauri `list_related_sources`: **Phase 16** (thin wrapper). Phase 15 verified solely via `cargo test -p retriever`.
- Keep `store` as sole SQLite owner; related only calls `get_source` + `retrieve`’s existing store search APIs.

### 3. Internal over-fetch — **`final_k = max(24, top_n * 5)`**
- Default `RetrieverConfig::default()` has `final_k: 8` [VERIFIED: retrieve.rs] — too small once seed self-hits consume slots and multiple chunks share one source.
- For `related_sources`, clone/adjust config:
  - `top_n` default **5** (D-04); clamp caller `top_n` to `1..=32` (treat `0` as `[]`) [ASSUMED: sane guard].
  - `final_k = max(24, top_n.saturating_mul(5))` (e.g. top_n=5 → 25).
  - Keep `vector_k` / `fts_k` at least `max(20, final_k)` so fusion window covers rollup.
  - `rrf_k` stays **60.0**.
- After exclude+dedupe, `take(top_n)`; if fewer unique neighbors, return what you have (D-05).

## Common Pitfalls

### Pitfall 1: Calling `retrieve` with empty seed text
**What goes wrong:** `Err(EmptyQuery)` instead of `Ok([])`.  
**Why:** `retrieve` trims and errors on empty [VERIFIED: retrieve.rs].  
**How to avoid:** Short-circuit after building seed query (D-03).  
**Warning signs:** Integration test for blank title/summary expects error.

### Pitfall 2: Under-fetching `final_k`
**What goes wrong:** Fewer than `top_n` neighbors even when corpus has them — seed chunks dominate top-8.  
**Why:** Default `final_k=8` + multi-chunk same source.  
**How to avoid:** Over-fetch formula above; assert in test with ≥2 multi-chunk sources.  
**Warning signs:** Flaky “only 1 related” with rich fixtures.

### Pitfall 3: Treating ROADMAP “empty on weak affinity” as Phase 15 scope
**What goes wrong:** Planner adds score cutoff contradicting D-06, or verifier fails SC#2.  
**Why:** REQUIREMENTS REL-02 / ROADMAP SC#2 wording vs CONTEXT lock.  
**How to avoid:** Document override in PLAN acceptance; defer threshold.  
**Warning signs:** Tests asserting empty for “unrelated” via score gate under MockEmbedder (hash vectors rarely separate cleanly without FTS design).

### Pitfall 4: MockEmbedder “related vs unrelated” without shared FTS terms
**What goes wrong:** Vector-only similarity is near-random under SHA256 mock; tests flake or always empty.  
**Why:** `MockEmbedder` hashes full text [VERIFIED: mock.rs].  
**How to avoid:** Fixture **related** pair sharing distinctive keywords in summary/body so FTS hits; **unrelated** pair with disjoint vocab; assert related seed surfaces neighbor and unrelated seed does not *necessarily* require empty (D-06) — prefer assert **seed excluded** + **rollup uniqueness** + **related keyword neighbor ranks** when FTS can match.  
**Warning signs:** Tests that only compare vector cosine.

### Pitfall 5: Holding Store lock across embed
**What goes wrong:** UI/indexer stalls (worse once Phase 16 IPC lands).  
**Why:** Embed is slow; Store uses `Mutex<Connection>` [CITED: .planning/research/PITFALLS.md].  
**How to avoid:** `get_source` → drop → `retrieve` (which already embeds then searches); never embed inside a custom long lock. Existing `retrieve` already sequences correctly.

### Pitfall 6: Putting scores or kind filters in the DTO
**What goes wrong:** Phase 16 UI leaks raw RRF; Wiki hard-filter fights D-10.  
**How to avoid:** Struct fields exactly D-07; no `score`; no kind skip.

## Code Examples

### Seed query + empty short-circuit

```rust
// Pattern derived from CONTEXT D-01..D-03 + retrieve EmptyQuery behavior
// [VERIFIED: crates/store/src/types.rs Source.summary; crates/retriever/src/retrieve.rs]
fn seed_query(source: &store::Source) -> Option<String> {
    let from_summary = source
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Some(q) = from_summary {
        return Some(q);
    }
    let title = source.title.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}
```

### Related sources sketch (locked behavior)

```rust
// crates/retriever/src/related.rs — recommended shape
// Aligns with ARCHITECTURE related_sources sketch but honors D-01..D-09
// [CITED: .planning/research/ARCHITECTURE.md] [VERIFIED: retrieve.rs API]

pub struct RelatedSource {
    pub source_id: String,
    pub title: String,
    pub kind: store::SourceKind,
    pub snippet: String,
    // intentionally no score — D-09
}

pub async fn related_sources(
    store: &store::Store,
    embedder: &dyn embedder::Embedder,
    source_id: &str,
    top_n: Option<usize>,
) -> Result<Vec<RelatedSource>> {
    let top_n = top_n.unwrap_or(5); // D-04
    if top_n == 0 {
        return Ok(vec![]);
    }
    let top_n = top_n.min(32);

    let seed = store.get_source(source_id)?; // NotFound → Err
    let Some(query) = seed_query(&seed) else {
        return Ok(vec![]); // D-03
    };

    let mut config = RetrieverConfig::default();
    config.final_k = (top_n.saturating_mul(5)).max(24);
    config.vector_k = config.vector_k.max(config.final_k);
    config.fts_k = config.fts_k.max(config.final_k);

    let hits = retrieve(store, embedder, &query, &config).await?;
    // rollup: skip seed, first source_id wins, enrich title/kind, truncate snippet, take top_n
    Ok(rollup_related(store, hits, source_id, top_n)?)
}

fn truncate_snippet(text: &str, max_chars: usize) -> String {
    const ELLIPSIS: char = '…';
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let taken: String = text.chars().take(max_chars).collect();
    format!("{taken}{ELLIPSIS}")
}
```

### Integration test intent (Wave 0)

```rust
// cargo test -p retriever related -- --nocapture
// [ASSUMED: planner expands into concrete fixtures]
#[tokio::test]
async fn related_excludes_seed_and_rolls_up_by_source() { /* ... */ }

#[tokio::test]
async fn empty_seed_title_returns_empty_ok() { /* ... */ }

#[tokio::test]
async fn missing_source_is_not_found_error() { /* ... */ }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Score-normalized hybrid blend | RRF rank fusion (k≈60) | Cormack et al. SIGIR 2009; OpenSearch 2.19 hybrid | No score calibration between vector & FTS [CITED: OpenSearch RRF blog] |
| Graph / co-citation for “related” | On-demand retrieve→rollup | Jarvis v1.11 research | Fits local SQLite RAG; no second DB [CITED: .planning/research/SUMMARY.md] |
| Expose cosine % in UI | Title + kind + snippet, no score | CONTEXT D-09 | Trust-preserving; Phase 16 UI-ready |

**Deprecated/outdated for this phase:**
- ARCHITECTURE sketch using `source_chunk_text` as query fallback — **do not implement**; locked to summary→title→`[]`.
- Empty-on-weak-affinity as Phase 15 gate — deferred by D-06.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | First RRF-ordered hit per `source_id` is the correct snippet source | Patterns / Discretion | Suboptimal preview text; still functionally OK |
| A2 | 160-char scalar truncation is enough for Phase 16 panel | Discretion | UI may want 120 or 200 — trivial constant change |
| A3 | `top_n` clamp `1..=32` and `0 → []` are acceptable guards | Discretion | Need discuss if callers pass large N |
| A4 | FTS-shared-keyword fixtures are the right MockEmbedder strategy for “related” asserts | Pitfalls | Tests may need richer corpora if FTS tokenization strips terms |
| A5 | Deferring Tauri IPC entirely to Phase 16 is OK | Placement | Phase 16 must add command + serde DTO; slightly larger Phase 16 scope |

**If this table is empty:** N/A — assumptions listed above for planner confirmation.

## Open Questions

1. **ROADMAP SC#2 vs D-06 documentation**
   - What we know: CONTEXT overrides weak-affinity empty for Phase 15.
   - What's unclear: Whether ROADMAP text will be amended.
   - Recommendation: PLAN notes “SC#2 deferred by D-06”; do not fail verify on threshold absence.

2. **API name `related_sources` vs `related_documents`**
   - What we know: CONTEXT uses documents; research/STATE use sources.
   - Recommendation: Implement `related_sources`; mention alias in PLAN notes.

3. **Should Phase 15 add a no-op Tauri command for DTO freeze?**
   - Recommendation: **No** — CONTEXT backend-only; Phase 16 owns IPC + panel.

## Environment Availability

Step 2.6: **SKIPPED for new external services** — phase is in-repo Rust only.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / cargo | Build + tests | ✓ | rustc 1.97.0 | — |
| Existing crates (`store`, `embedder`, `indexer`) | Implementation + fixtures | ✓ | workspace | — |
| Live LLM / Ollama | — | N/A | — | Must not use; MockEmbedder only |
| Graph index (`.planning/graphs`) | Optional discovery | ✗ | — | Codebase read only |

**Missing dependencies with no fallback:** none  
**Missing dependencies with fallback:** none

## Validation Architecture

> `workflow.nyquist_validation` is **true** in `.planning/config.json` [VERIFIED: .planning/config.json].

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + `tokio` async tests; `tempfile` DB; `MockEmbedder` |
| Config file | crate-local `#[cfg(test)]` / workspace Cargo |
| Quick run command | `cargo test -p retriever related -- --test-threads=4` |
| Full suite command | `cargo test -p retriever` |
| E2E this phase | **Not required** (backend-only; E2E in Phase 16) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REL-02 | Seed excluded from results | integration | `cargo test -p retriever related_excludes_seed` | ❌ Wave 0 |
| REL-02 | Chunk hits rolled up by `source_id` (unique neighbors) | integration | `cargo test -p retriever related_rolls_up_by_source` | ❌ Wave 0 |
| REL-02 | Cap to `top_n` (default 5) | unit/integration | `cargo test -p retriever related_respects_top_n` | ❌ Wave 0 |
| REL-02 / D-03 | Empty title+summary → `Ok([])` | unit/integration | `cargo test -p retriever related_empty_seed_text` | ❌ Wave 0 |
| REL-02 / D-05 | Only seed in corpus → `Ok([])` | integration | `cargo test -p retriever related_only_seed` | ❌ Wave 0 |
| D-01/D-02 | Prefers non-empty summary over title | integration | `cargo test -p retriever related_prefers_summary` | ❌ Wave 0 |
| D-07/D-08/D-09 | DTO fields + truncated snippet + no score | unit | `cargo test -p retriever related_snippet_truncate` | ❌ Wave 0 |
| D-10 | Wiki/Memory may appear (no filter) | integration | `cargo test -p retriever related_allows_wiki_neighbor` | ❌ Wave 0 |
| — | Missing `source_id` → error | integration | `cargo test -p retriever related_missing_source` | ❌ Wave 0 |
| — | Uses hybrid path (regression) | existing | `cargo test -p retriever hybrid_retrieve` | ✅ retrieve.rs |

### Sampling Rate
- **Per task commit:** `cargo test -p retriever related`
- **Per wave merge:** `cargo test -p retriever`
- **Phase gate:** `cargo test -p retriever` green; no `npm run test:e2e:local` required for Phase 15 alone

### Wave 0 Gaps
- [ ] `crates/retriever/src/related.rs` (+ `#[cfg(test)]` module) — REL-02 behaviors above
- [ ] Fixture helpers: ≥2 indexed markdown files with shared distinctive terms; 1 disjoint-vocab control
- [ ] Export `related_sources` / `RelatedSource` from `lib.rs`
- [ ] Framework install: none — existing `cargo test -p retriever` infrastructure

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Local desktop; no new auth surface |
| V3 Session Management | no | — |
| V4 Access Control | no | Single-user local KB; same trust as existing retrieve |
| V5 Input Validation | yes | Validate `source_id` via `get_source` (NotFound); clamp `top_n`; never interpolate SQL in related (Store parameterized) [VERIFIED: store.rs `get_source`] |
| V6 Cryptography | no | No new crypto; Mock/SHA256 only in tests |

### Known Threat Patterns for Rust + SQLite RAG

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via source_id | Tampering | Parameterized queries in `Store` only |
| Over-large top_n DoS / huge retrieve | Denial of Service | Clamp `top_n`; bounded `final_k` |
| Information leak of raw scores as “confidence” | Elevation / Spoofing trust | D-09 — omit scores from DTO [CITED: PITFALLS overlap garbage] |
| Second SQLite connection | Tampering / Availability | Forbidden — `store` sole owner |

`security_enforcement`: treat as enabled (not set false in config).

## Project Constraints (from .cursor/rules/)

| Rule | Directive for Phase 15 |
|------|------------------------|
| `tdd-goal-driven.mdc` | Failing tests first; unit/integration with `MockEmbedder` + tempfile; run `cargo test -p retriever` before done |
| `e2e-required.mdc` | E2E mandatory for **user-facing** only — Phase 15 backend API → **no E2E**; Phase 16 adds panel E2E |
| `jarvis-stack.mdc` | Rust stable / MSRV 1.85; logic in library crates; thin Tauri later; no new stack |
| `karpathy-guidelines.mdc` | Surgical diff; no speculative threshold/UI; honor locked CONTEXT over ROADMAP wording |
| `frontend-taste.mdc` | N/A this phase (no UI) |

Additional always-on from project docs:
- **Store sole SQLite owner** — related must not open DB connections.
- **Trait inject Embedder** — no live FastEmbed/Ollama in tests.
- **Map errors to `String` only at Tauri boundary** — library keeps `RetrieveError` / `StoreError`.

## Sources

### Primary (HIGH confidence)
- `.planning/phases/15-overlap-scoring-api/15-CONTEXT.md` — locked D-01..D-11
- `crates/retriever/src/retrieve.rs`, `rrf.rs`, `error.rs`, `lib.rs` — hybrid API surface
- `crates/store/src/types.rs`, `store.rs` — `Source`, `ChunkHit`, `get_source`, `source_chunk_text` (available but **not** used for seed query)
- `crates/embedder/src/mock.rs` — deterministic hash embedder
- `.planning/config.json` — nyquist_validation true
- `.cursor/rules/*` — TDD / E2E / stack / taste

### Secondary (MEDIUM confidence)
- `.planning/research/{SUMMARY,ARCHITECTURE,FEATURES,STACK,PITFALLS}.md` — v1.11 approach (note ARCHITECTURE seed-query sketch superseded by CONTEXT)
- OpenSearch RRF hybrid search intro — rank fusion motivation [CITED: opensearch.org/blog/introducing-reciprocal-rank-fusion-hybrid-search]
- Cormack et al. RRF SIGIR 2009 [CITED: cormack.uwaterloo.ca/cormacksigir09-rrf]

### Tertiary (LOW confidence)
- Grapheme vs char UX preference for Chinese panel previews — assumed char truncation sufficient
- Exact over-fetch multiplier (`* 5`, min 24) — engineering judgment pending fixture tuning

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — reuse existing crates; no new deps
- Architecture: **HIGH** — CONTEXT + verified retrieve/store code; clear D-06 override
- Pitfalls: **HIGH** — EmptyQuery trap, under-fetch, MockEmbedder/FTS, ROADMAP wording verified in-repo
- Discretion constants (160 chars, over-fetch formula): **MEDIUM** — recommended, tunable

**Research date:** 2026-07-25  
**Valid until:** 2026-08-24 (stable domain; recheck if CONTEXT or `RetrieverConfig` defaults change)

## RESEARCH COMPLETE
