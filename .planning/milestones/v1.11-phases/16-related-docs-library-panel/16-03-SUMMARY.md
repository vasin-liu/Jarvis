---
phase: 16-related-docs-library-panel
plan: 03
subsystem: ui
tags: [react, library, related-docs, vitest]

requires:
  - phase: 16-related-docs-library-panel
    provides: listRelatedSources IPC (16-01)
provides:
  - Library selection + under-list related panel
  - Vitest related-docs contract (D-02..D-13)
affects: [16-04-e2e]

tech-stack:
  added: []
  patterns: [fetchSeq last-wins; stopPropagation on action column; cyan selection]

key-files:
  created: []
  modified:
    - src/views/LibraryView.tsx
    - src/views/LibraryView.test.tsx

key-decisions:
  - "D-01..D-15 panel UX locked in CONTEXT/UI-SPEC"
  - "React text children only for snippet/title (no dangerouslySetInnerHTML)"

patterns-established:
  - "source-row-{id} toggle selection; related-docs-row-{id} navigate+refetch"

requirements-completed: [REL-01, REL-03, REL-04]

coverage:
  - id: D1
    description: Related panel loading/empty/error/hidden + navigate
    requirement: REL-01
    verification:
      - kind: unit
        ref: npx vitest run src/views/LibraryView.test.tsx
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-07-28
status: complete
---

# Phase 16: Plan 03 Summary

**Library selection shows an under-list related-docs panel (loading/empty/error/non-indexed) with title+kind+snippet rows and navigate-on-click — no scores.**

## Accomplishments

- Selectable `source-row-{id}` with cyan highlight; action buttons `stopPropagation`
- `related-docs-panel` with D-10/D-11/D-12 copy and last-wins `fetchSeq`
- Related row click selects neighbor, optional `scrollIntoView`, refetches
- Vitest describe `LibraryView related docs` — 8 cases; full suite 22/22 green

## Commits

- `c388258` feat(16-03): Library related-docs panel with Vitest coverage

## Deviations

- Combined RED+GREEN in one implementation pass after Wave 1 resume (executor Task interruptions)
- Guard `typeof scrollIntoView === "function"` for jsdom
