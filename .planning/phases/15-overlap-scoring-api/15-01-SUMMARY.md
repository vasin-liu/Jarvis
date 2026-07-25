---
phase: 15-overlap-scoring-api
plan: 01
subsystem: api
tags: [retriever, related_sources, tdd, REL-02, MockEmbedder]

requires:
  - phase: 14-tech-debt-closeout
    provides: Stable retriever hybrid retrieve + store Source APIs
provides:
  - RelatedSource DTO stub (source_id, title, kind, snippet; no score)
  - related_sources stub returning Ok([])
  - Eight REL-02 #[tokio::test] cases failing RED on behavior assertions
  - Crate-root re-exports of related_sources and RelatedSource
affects:
  - 15-02-PLAN (GREEN implementation)
  - Phase 16 Library related-docs IPC consumer

tech-stack:
  added: []
  patterns:
    - "TDD RED stub: related_sources returns Ok(vec![]) so empty-list cases may pass; non-empty/Err cases fail assertions"
    - "FTS-shared fixture keywords (overlap-scoring-alpha) for MockEmbedder determinism"
    - "WikiPage neighbor via index_path + upsert_source kind retag (no new ingest dep)"

key-files:
  created:
    - crates/retriever/src/related.rs
  modified:
    - crates/retriever/src/lib.rs

key-decisions:
  - "Stub always Ok([]) — RED via assertion failures, not compile errors"
  - "mod related declared in Task 1 so cargo test can compile related.rs; Task 2 adds pub use"

patterns-established:
  - "related_* test names from 15-VALIDATION.md are the contract for Plan 15-02 GREEN"
  - "Snippet truncation expectation: ≤161 chars ending with U+2026 (160 + ellipsis)"

requirements-completed: [REL-02]

coverage:
  - id: D1
    description: "RelatedSource stub + related_sources stub compile; eight REL-02 integration tests exist and run RED"
    requirement: REL-02
    verification:
      - kind: unit
        ref: "cargo test -p retriever related — 2 passed; 6 failed (assertion)"
        status: pass
    human_judgment: false
  - id: D2
    description: "lib.rs exports related_sources and RelatedSource at crate root"
    requirement: REL-02
    verification:
      - kind: other
        ref: "cargo check -p retriever; crates/retriever/src/lib.rs pub use related"
        status: pass
    human_judgment: false

duration: 18min
completed: 2026-07-25
status: complete
---

# Phase 15 Plan 01: Overlap Scoring API RED Summary

**TDD RED wave: `related_sources` stub + eight REL-02 integration tests that fail on behavior assertions while compiling cleanly**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-25T11:11:30Z
- **Completed:** 2026-07-25T11:29:52Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Created `RelatedSource` (four fields, no `score`) and stub `related_sources` returning `Ok(vec![])`
- Added eight named `related_*` tests covering D-01..D-11 behaviors (seed exclude/rollup, empty seed, missing source, summary preference, top_n, snippet truncate, only-seed, Wiki neighbor)
- Wired `mod related` + crate-root `pub use` so Plan 15-02 can implement against a fixed public API

## Task Commits

Each task was committed atomically:

1. **Task 1: Add RelatedSource stub and failing REL-02 integration tests** - `0942a48` (test)
2. **Task 2: Wire related module exports in lib.rs** - `de42760` (feat)

**Plan metadata:** (this commit)

## Files Created/Modified

- `crates/retriever/src/related.rs` — DTO + stub + `#[cfg(test)]` REL-02 suite / fixtures
- `crates/retriever/src/lib.rs` — `mod related` + `pub use related::{related_sources, RelatedSource}`

## Decisions Made

- Stub returns empty `Ok` so empty-list cases (`related_empty_seed_text`, `related_only_seed`) pass; six other tests fail RED on assertions (meets “≥6 fail” gate)
- Declared `mod related` in Task 1 (with related.rs) so the RED suite compiles; Task 2 only added re-exports
- Wiki neighbor fixture: `index_path` then `upsert_source` with `SourceKind::WikiPage` — avoids new Cargo.toml deps

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Declare `mod related` in Task 1**
- **Found during:** Task 1 (RED suite verify)
- **Issue:** Plan Task 1 `files` listed only `related.rs`, but without `mod related` in `lib.rs` the module (and tests) would not compile or run
- **Fix:** Added `mod related;` in Task 1 commit; Task 2 added `pub use`
- **Files modified:** `crates/retriever/src/lib.rs`
- **Verification:** `cargo test -p retriever related` runs 8 tests
- **Committed in:** `0942a48` (Task 1)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Necessary for RED verify; no scope creep into GREEN logic

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 15-02 GREEN: implement seed query, truncate_snippet, retrieve + rollup
- Do not rename the eight `related_*` tests; fill stub body only

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED | `0942a48` test(15-01) | Pass — 6 assertion failures, compiles |
| GREEN | — | Deferred to Plan 15-02 |
| REFACTOR | — | N/A this plan |

## Self-Check: PASSED

- [x] `crates/retriever/src/related.rs` exists
- [x] `git log --grep=15-01` ≥1 commit
- [x] Acceptance: eight tests run; six fail assertions; no rusqlite; no score field
- [x] `cargo check -p retriever` succeeds; tests still RED

---
*Phase: 15-overlap-scoring-api*
*Completed: 2026-07-25*
