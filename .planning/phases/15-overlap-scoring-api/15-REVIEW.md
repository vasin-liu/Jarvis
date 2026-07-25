---
phase: 15-overlap-scoring-api
reviewed: 2026-07-25T12:22:00Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - crates/retriever/src/related.rs
  - crates/retriever/src/lib.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
advisory: true
---

# Phase 15: Code Review Report

**Reviewed:** 2026-07-25T12:22:00Z  
**Depth:** standard  
**Files Reviewed:** 2  
**Status:** clean  
**Mode:** ADVISORY (no code fixes applied)

## Summary

Phase 15 delivers `related_sources` / `RelatedSource` in `crates/retriever`: seed query (summary→title), hybrid `retrieve` reuse, seed exclude, first-hit source rollup, 160-char UTF-8-safe snippets, `top_n` default/clamp. Implementation matches CONTEXT D-01..D-11 and Plan 15-02 (including D-06 no score threshold). No critical or warning defects. `cargo test -p retriever related` → 12 passed.

## Narrative Findings (AI reviewer)

### Focus checklist

| Focus | Result | Notes |
|-------|--------|-------|
| Reuse `retrieve` only (no direct FTS/vector/RRF) | **pass** | `related.rs` imports `retrieve` / `RetrieverConfig` only |
| Seed query D-01..D-03 | **pass** | `seed_query` trim summary → title → `None` → `Ok([])` |
| Missing seed → NotFound | **pass** | `get_source` + `RetrieveError::Store` |
| Seed exclude + rollup | **pass** | Skip seed; `HashSet` first-hit wins snippet |
| top_n default 5 / 0→[] / clamp 32 | **pass** | L65–68; tested for 5/2/0 |
| No score on DTO (D-09) | **pass** | Four fields only |
| No kind filter (D-10) | **pass** | Wiki neighbor test green |
| Snippet ≤160 + U+2026 (D-08) | **pass** | `chars()` truncation; CJK covered |
| Over-fetch config | **pass** | `final_k = max(24, top_n*5)`; bump vector/fts; `rrf_k=60` |
| Sole DB owner | **pass** | Store APIs only; no rusqlite in related |
| lib.rs exports | **pass** | `pub use related::{related_sources, RelatedSource}` |

## Critical Issues

None.

## Warnings

None.

## Info

### IN-01: Public rustdoc omits `top_n` contract

**File:** `crates/retriever/src/related.rs:56-64`  
**Issue:** Doc comment covers hybrid reuse and D-06, but not caller-visible `top_n` rules: `None`→5, `Some(0)`→`[]`, values `>32` silently clamped to 32 (T-15-02 / Plan 15-02). Phase 16 IPC callers may assume unbounded `top_n`.  
**Fix:** Extend the doc comment, e.g.:

```rust
/// Rank other indexed sources by hybrid retrieval overlap with the seed.
///
/// `top_n`: `None` defaults to 5; `Some(0)` returns `[]`; values above 32 are clamped to 32.
/// Reuses `retrieve` (vector+FTS+RRF). No score threshold (D-06) — weak hits may appear.
```

### IN-02: Clamp `top_n > 32` has no automated assertion

**File:** `crates/retriever/src/related.rs:68` / `related_respects_top_n` (~L352–393)  
**Issue:** Implementation clamps with `.min(32)`, and `related_respects_top_n` covers default/2/0, but nothing asserts `Some(100)` yields at most 32 neighbors. Regression risk if clamp is removed.  
**Fix:** Add one assertion in `related_respects_top_n` (or a unit test) calling `related_sources(..., Some(100))` and `assert!(related.len() <= 32)` (with a fixture that has ≥32 neighbors only if you need exact fill; otherwise assert the effective cap via a thin wrapper test of the clamp expression / config builder).

---

_Reviewed: 2026-07-25T12:22:00Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: standard_
