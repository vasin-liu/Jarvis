---
phase: 16-related-docs-library-panel
plan: 04
subsystem: testing
tags: [e2e, related-docs, webdriver]

requires:
  - phase: 16-related-docs-library-panel
    provides: dual fixture seed (16-02) + Library panel (16-03)
provides:
  - related-docs.spec.ts focused journey
  - e2e-required.mdc Related-docs map row
affects: []

tech-stack:
  added: []
  patterns: [DOM execute click by index for Windows path source ids]

key-files:
  created:
    - e2e/specs/related-docs.spec.ts
  modified:
    - .cursor/rules/e2e-required.mdc

key-decisions:
  - "D-16 focused E2E only; Phase 19 owns citation-trust"
  - "Avoid CSS attribute selectors for Windows \\\\?\\ path source ids"

patterns-established:
  - "clickSourceRowByIndex / exact data-testid string match in browser.execute"

requirements-completed: [REL-01, REL-03]

coverage:
  - id: D1
    description: Select → related panel → navigate neighbor under JARVIS_E2E=1
    requirement: REL-03
    verification:
      - kind: e2e
        ref: npx wdio run e2e/wdio.conf.ts --spec e2e/specs/related-docs.spec.ts
        status: pass
    human_judgment: false

duration: 45min
completed: 2026-07-28
status: complete
---

# Phase 16: Plan 04 Summary

**Focused related-docs E2E is green under Mock providers: dual seed → select → panel → navigate → deselect.**

## Accomplishments

- Added `e2e/specs/related-docs.spec.ts` covering REL-01 panel + REL-03 navigate (D-16)
- Updated `.cursor/rules/e2e-required.mdc` Spec map with Related-docs row
- Fixed Windows path source-id CSS breakage via `browser.execute` index/exact-testid clicks
- Verified: `1 passing` (wdio filtered run, exit 0)

## Commits

- `8a35136` feat(16-04): add related-docs E2E spec and e2e-required map row
- (follow-up) fix E2E selectors for Windows path source ids

## Deviations

- `npm run test:e2e:local -- --spec` does not forward `--spec` through nested npm scripts; ran `npx wdio … --spec` after `build:e2e`
