---
phase: 14-address-tech-debt-wiki-reindex-export-preflight-gate-nyquist
plan: "01"
subsystem: indexing
tags: [wiki, reindex, soft-skip, library, vitest, tdd]

requires:
  - phase: 10-persist-index
    provides: SourceKind::WikiPage indexed via wiki:// pipeline
provides:
  - WikiPage reindex soft-skips Ok(false) before delete_chunks_for_source
  - Library retry button hidden for kind wiki_page
affects:
  - 14-02 export preflight gate
  - milestone audit closeout

tech-stack:
  added: []
  patterns:
    - "WikiPage soft skip: early-return Ok(false) before destructive chunk delete"
    - "Hide (not disable) Library retry for wiki_page — kind-only gate"

key-files:
  created: []
  modified:
    - src-tauri/src/index_ops.rs
    - src/views/LibraryView.tsx
    - src/views/LibraryView.test.tsx

key-decisions:
  - "D-01 soft skip before delete_chunks — never mark_failed for WikiPage"
  - "D-03 hide retry for wiki_page (kind-only); no wiki.enabled gate"
  - "D-04/D-15 E2E waiver — Vitest + Rust unit only for this debt item"

patterns-established:
  - "Soft skip WikiPage at top of reindex_source; exhaustiveness arm is unreachable"
  - "Library kind-only hide mirrors compile hide pattern without enabled check"

requirements-completed: [AUDIT-CLOSEOUT]

coverage:
  - id: D1
    description: "WikiPage reindex soft-skips with Ok(false), no mark_failed, chunks preserved"
    requirement: AUDIT-CLOSEOUT
    verification:
      - kind: unit
        ref: "src-tauri/src/index_ops.rs#rebuild_wiki_page_soft_skips_without_failed_or_chunk_wipe"
        status: pass
    human_judgment: false
  - id: D2
    description: "Bulk rebuild increments skipped and does not abort / mark Failed for WikiPage"
    requirement: AUDIT-CLOSEOUT
    verification:
      - kind: unit
        ref: "cargo test -p tauri-app index_ops"
        status: pass
    human_judgment: false
  - id: D3
    description: "LibraryView omits retry-source-{id} for wiki_page; local_file still shows retry"
    requirement: AUDIT-CLOSEOUT
    verification:
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#hides retry for wiki_page"
        status: pass
    human_judgment: false
  - id: D4
    description: "No new E2E for retry hide (D-04/D-15 waiver vs e2e-required.mdc)"
    requirement: AUDIT-CLOSEOUT
    verification:
      - kind: other
        ref: "waiver: 14-CONTEXT D-04 D-15; no new e2e/specs from 14-01"
        status: pass
    human_judgment: false

duration: 33min
completed: 2026-07-24
status: complete
---

# Phase 14 Plan 01: WikiPage Soft Skip + Hide Retry Summary

**WikiPage「重新同步」fail-closed stub replaced with silent soft skip before chunk wipe; Library hides retry for `wiki_page` (Vitest + Rust unit; E2E waived per D-04/D-15).**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-24T23:03:34Z
- **Completed:** 2026-07-24T23:36:35Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Soft-skip `SourceKind::WikiPage` in `reindex_source` with early `Ok(false)` **before** `delete_chunks_for_source` (D-01, T-14-01)
- Removed fail-closed `mark_failed` WikiPage arm; status/chunks unchanged; bulk `skipped` increments (D-05)
- LibraryView hides `retry-source-{id}` when `s.kind !== "wiki_page"`; Vitest covers hide + local still shown (D-03)
- No toast/FE error path for soft skip (D-02); no new E2E (D-04/D-15 waiver)

## Task Commits

Each task was committed atomically (TDD RED → GREEN):

1. **Task 1 RED:** `b4bce5e` — `test(14-01): add failing WikiPage soft-skip reindex test`
2. **Task 1 GREEN:** `7001327` — `feat(14-01): soft-skip WikiPage reindex before chunk delete`
3. **Task 2 RED:** `dd1542b` — `test(14-01): add failing Vitest for hidden wiki_page retry`
4. **Task 2 GREEN:** `2e46ebb` — `feat(14-01): hide Library retry button for wiki_page`

**Plan metadata:** (this commit)

## Files Created/Modified

- `src-tauri/src/index_ops.rs` — WikiPage early soft skip + unit test `rebuild_wiki_page_soft_skips_without_failed_or_chunk_wipe`
- `src/views/LibraryView.tsx` — kind-only gate around retry button
- `src/views/LibraryView.test.tsx` — hides retry for wiki_page; shows retry for local_file

## Decisions Made

- Inline early-return for WikiPage (Discretion) rather than a separate helper
- Exhaustiveness arm kept as `unreachable!` after soft skip (match still covers WikiPage)
- E2E waiver D-04/D-15 honored — no `e2e/specs` changes

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Initial test seed used `reinit_vectors` which zeroed chunk count before soft-skip assert; removed from seed so chunks exist pre-rebuild (test-only fix during RED)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for **14-02** (export preflight WikiDisabled gate)
- Soft skip is IPC safety net; UI hide is primary UX for Library
- AUDIT-CLOSEOUT still shared with 14-02/14-03 — do not treat milestone audit as closed yet

## Self-Check: PASSED

- [x] `cargo test -p tauri-app index_ops` — 4 passed
- [x] `npx vitest run src/views/LibraryView.test.tsx` — 14 passed
- [x] Soft skip textually above `delete_chunks_for_source`
- [x] No `mark_failed` WikiPage arm
- [x] No new `e2e/specs` from this plan
- [x] git log grep `14-01` ≥ 4 commits

---
*Phase: 14-address-tech-debt-wiki-reindex-export-preflight-gate-nyquist*
*Completed: 2026-07-24*
