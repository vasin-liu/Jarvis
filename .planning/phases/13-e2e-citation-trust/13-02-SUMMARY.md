---
phase: 13-e2e-citation-trust
plan: "02"
subsystem: testing
tags: [e2e, wiki, citations, webdriver, wdio]

requires:
  - phase: 13-e2e-citation-trust
    provides: Plan 01 harness — data-source-uri, export path bypass, settings-save-config, zip helpers
provides:
  - "wiki.spec.ts WIKI-09 enable→compile→笔记页→export + WIKI-08 citation URI trust"
  - "full-ui Library default-off absence of wiki-export / wiki-compile-*"
  - "Spec map Wiki → wiki.spec.ts in e2e-required.mdc and e2e/README.md"
  - "snake_case IPC for IndexStatus/SourceKind (Library indexed gates)"
affects:
  - phase verification / milestone close

tech-stack:
  added: []
  patterns:
    - "Prefix querySelector click for data-testid values that embed Windows paths"
    - "App onAfterIndexChange refreshes config after Settings save"

key-files:
  created:
    - e2e/specs/wiki.spec.ts
  modified:
    - e2e/helpers.ts
    - e2e/specs/full-ui.spec.ts
    - .cursor/rules/e2e-required.mdc
    - e2e/README.md
    - crates/store/src/types.rs
    - src/App.tsx
    - src/views/SettingsView.test.tsx

key-decisions:
  - "Click first [data-testid^=wiki-compile-] via querySelector — fixture source ids are \\\\?\\ Windows paths that break CSS attribute selectors"
  - "Add serde rename_all=snake_case on IndexStatus and SourceKind so FE status===indexed and kind labels match IPC"
  - "Settings save refreshes App config (onAfterIndexChange) so Library sees wiki.enabled without relying only on E2E refresh hook"
  - "qa.spec.ts left untouched (D-11)"

patterns-established:
  - "Hybrid wiki enable: Settings toggle + settings-save-config + optional __JARVIS_E2E_REFRESH_CONFIG__"
  - "Export assert: setWikiExportPath(wikiE2eZipPath) → wiki-export-done → assertWikiZipNonEmpty (no unzip)"

requirements-completed: [WIKI-08, WIKI-09]

coverage:
  - id: D1
    description: "wiki.spec default-off Library has no wiki-export / wiki-compile-*"
    requirement: WIKI-09
    verification:
      type: e2e
      ref: e2e/specs/wiki.spec.ts
  - id: D2
    description: "enable→compile→笔记页→export zip size>0"
    requirement: WIKI-09
    verification:
      type: e2e
      ref: e2e/specs/wiki.spec.ts
  - id: D3
    description: "After wiki on+compile, What is xyzzy-plugh? has ≥1 data-source-uri not starting with wiki://"
    requirement: WIKI-08
    verification:
      type: e2e
      ref: e2e/specs/wiki.spec.ts
  - id: D4
    description: "full-ui Library default-off only; dual spec maps list wiki.spec.ts"
    requirement: WIKI-09
    verification:
      type: e2e
      ref: e2e/specs/full-ui.spec.ts

---

# Plan 02: wiki.spec + citation trust

**WIKI-08/WIKI-09 E2E journey green under mocks — enable→compile→笔记页→export + non-wiki citation URI trust.**

## Commits

| Hash | Message |
|------|---------|
| `15ca742` | `test(13-02): add wiki.spec journey with path-safe compile click` |
| `4a72c57` | `test(13-02): full-ui wiki default-off and dual spec-map docs` |

## What shipped

- `e2e/specs/wiki.spec.ts` — default-off it + positive hybrid Settings enable → first wiki-compile → 笔记页 → zip export → `What is xyzzy-plugh?` citation URI trust; optional disable at end
- `e2e/helpers.ts` — click-based `setReactCheckbox`; path-safe compile click pattern in spec
- `full-ui.spec.ts` — Library absence of wiki controls (D-12)
- Spec maps in `.cursor/rules/e2e-required.mdc` and `e2e/README.md` (D-13)
- Blockers fixed for the journey: `IndexStatus`/`SourceKind` snake_case serde; App config refresh after Settings save; SettingsView Vitest mock completeness

## Verification

- Focused `wiki.spec.ts`: 2 passing
- Focused `full-ui` + `qa`: 2 files / 11 its passing; `qa.spec.ts` untouched
- `npm run build:e2e` + full WDIO suite: **9 passed, 9 total** (exit 0)

## Deviations

- **Auto-fixed blockers (Rule 2):** IPC enum casing and Settings→App config refresh were required for compile buttons / wiki.enabled to appear; documented above. Not in original `files_modified` but necessary for WIKI-09.
- Did **not** weaken D-10 citation assert; did **not** unzip vault; did **not** edit `qa.spec.ts`.

## Self-Check: PASSED

- [x] wiki.spec asserts non-wiki `data-source-uri` after xyzzy-plugh
- [x] full-ui has wiki-export / wiki-compile absence only
- [x] Spec maps include wiki.spec.ts
- [x] Full local E2E suite green
- [x] qa.spec.ts unchanged in plan commits
