---
phase: 10-persist-index
plan: "02"
subsystem: wiki
tags: [wiki, cleanup, rebuild-index, user-edit, insights, hash-skip]

requires:
  - phase: 10-persist-index
    provides: compile_wiki_for_source, rebuild_index_md_from_disk, WikiCompileSummary, D-09 skip-write
provides:
  - rebuild_index_md_* unit coverage locking D-01…D-05 / D-13
  - cleanup_stale_wiki_pages_for_source (FS + delete_chunks_for_source then delete_source)
  - WikiCompileSummary.cleaned populated on re-compile
  - compile_cleanup_and_user_edit_* proving D-06…D-09 / D-12
  - D-10 frontmatter digest omission locked on compile output
affects: [11-library-settings-ui, 13-e2e-citation-trust]

tech-stack:
  added: []
  patterns:
    - "Stale cleanup: generated:true AND sources contains URI AND slug ∉ current set"
    - "Library remove_source pair for wiki:// cleanup (never forget_memory delete-only)"
    - "Scan-rebuild index.md after cleanup so multi-source vault catalog matches disk"

key-files:
  created: []
  modified:
    - crates/insights/src/wiki.rs

key-decisions:
  - "Collect stale paths before delete to avoid mutating read_dir mid-scan"
  - "frontmatter_sources_contains matches quoted URI on sources: line (no serde_yaml)"
  - "Plan 01 rebuild_index_md_from_disk already satisfied D-01…D-05; Task 1 locked with unit tests only"

patterns-established:
  - "compile order: write(D-09) → cleanup(D-06…D-08) → rebuild_index_md_from_disk → index-from-disk"
  - "User notes without generated:true are never deleted and never overwritten; still indexed via hash_text(on-disk)"

requirements-completed: [WIKI-04]

coverage:
  - id: D1
    description: "rebuild_index_md_from_disk multi-dir catalog with Sources+Entities (D-01, D-02)"
    requirement: WIKI-04
    verification:
      - kind: unit
        ref: crates/insights/src/wiki.rs#rebuild_index_md_includes_multi_dir_pages
        status: pass
    human_judgment: false
  - id: D2
    description: "Lex sort + frontmatter title display; omit empty sections; ignore root clutter/index.md (D-03…D-05, D-13)"
    requirement: WIKI-04
    verification:
      - kind: unit
        ref: crates/insights/src/wiki.rs#rebuild_index_md_lex_sorts_and_uses_title
        status: pass
      - kind: unit
        ref: crates/insights/src/wiki.rs#rebuild_index_md_omits_empty_sections
        status: pass
      - kind: unit
        ref: crates/insights/src/wiki.rs#rebuild_index_md_ignores_root_clutter
        status: pass
    human_judgment: false
  - id: D3
    description: "Stale generated pages deleted from FS+Store; cleaned counted (D-06…D-08)"
    requirement: WIKI-04
    verification:
      - kind: integration
        ref: crates/insights/src/wiki.rs#compile_removes_stale_generated_page
        status: pass
      - kind: integration
        ref: crates/insights/src/wiki.rs#compile_cleanup_and_user_edit_preserves_note
        status: pass
    human_judgment: false
  - id: D4
    description: "User-edit skip write + still index on-disk bytes; no digest in frontmatter (D-09, D-10, D-12)"
    requirement: WIKI-04
    verification:
      - kind: integration
        ref: crates/insights/src/wiki.rs#compile_cleanup_and_user_edit_preserves_note
        status: pass
      - kind: integration
        ref: crates/insights/src/wiki.rs#compile_output_frontmatter_has_no_digest_key
        status: pass
      - kind: unit
        ref: crates/insights/src/wiki.rs#frontmatter_omits_content_hash
        status: pass
    human_judgment: false

duration: 11 min
completed: 2026-07-21
status: complete
---

# Phase 10 Plan 02: Vault cleanup + rebuild coverage Summary

**WIKI-04 vault correctness: rebuild_index_md unit locks, stale generated cleanup with Store pair-delete, and user-edit sacred path that still indexes on-disk notes.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-21T10:03:09Z
- **Completed:** 2026-07-21T10:13:59Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Locked `rebuild_index_md_from_disk` with four `rebuild_index_md_*` unit tests (D-01…D-05, D-13)
- Wired `cleanup_stale_wiki_pages_for_source` into compile after write / before scan-rebuild; `cleaned` populated
- Integration: stale generated removal + user-edit skip + still-indexed user note; D-10 digest stays out of Markdown frontmatter
- Phase gate `cargo test -p insights -p indexer -p memory -- --test-threads=1` green; E2E deferred to Phase 13

## Task Commits

Each task was committed atomically:

1. **Task 1: rebuild_index_md unit coverage D-01…D-05** - `01e31a2` (test)
2. **Task 2: Stale cleanup D-06…D-08 wired into compile** - `1326eb8` (feat)
3. **Task 3: User-edit D-09/D-12 + D-10 frontmatter assert** - `38f86db` (test)

**Plan metadata:** (this commit)

## Files Created/Modified

- `crates/insights/src/wiki.rs` — `frontmatter_sources_contains`, `cleanup_stale_wiki_pages_for_source`, compile wiring, rebuild/cleanup/user-edit tests

## Decisions Made

- Plan 01 scan-rebuild already met D-01…D-05; Task 1 added unit locks without behavior change
- Collect stale path list per directory before FS/Store deletes (safe under `read_dir`)
- Ownership peek: quoted URI on `sources:` line; no `serde_yaml`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 10 (persist-index) complete — WIKI-04 fully covered across Plan 01+02
- Ready for Phase 11 Library/Settings UI (compile UX) and Phase 13 wiki E2E (`wiki.spec.ts`)
- No UI / no `e2e/specs/` changes this plan

## Self-Check: PASSED

- [x] `rebuild_index_md_*` four tests exist and exit 0
- [x] `delete_chunks_for_source` then `delete_source` in cleanup path; `cleaned` updated
- [x] `compile_cleanup_and_user_edit_preserves_note` + `compile_removes_stale_generated_page` exit 0
- [x] `frontmatter_omits_content_hash` + `compile_output_frontmatter_has_no_digest_key` exit 0
- [x] Phase gate insights+indexer+memory exit 0
- [x] `git log --grep=10-02` shows ≥3 task commits
- [x] No `e2e/specs/` or React UI modifications

---
*Phase: 10-persist-index*
*Completed: 2026-07-21*
