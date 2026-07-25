---
phase: 13-e2e-citation-trust
plan: "01"
subsystem: testing
tags: [e2e, wiki, citations, vitest, tauri, webdriver]

requires:
  - phase: 12-obsidian-zip-export
    provides: exportWiki preflight/save/zip + wiki-export-done notice
  - phase: 11-library-settings-ui
    provides: Settings wiki toggle + 保存配置 button
provides:
  - "data-source-uri on Chat citation buttons for WIKI-08 E2E"
  - "exportWiki __JARVIS_E2E_WIKI_EXPORT_PATH__ Save-dialog bypass (D-05)"
  - "settings-save-config testid for hybrid enable"
  - "e2e helpers setWikiExportPath / wikiE2eZipPath / assertWikiZipNonEmpty"
affects:
  - 13-e2e-citation-trust plan 02 wiki.spec

tech-stack:
  added: []
  patterns:
    - "Window __JARVIS_E2E_* hook for dialog bypass (ungated when path set)"
    - "E2E helpers assign window props via browser.execute"

key-files:
  created: []
  modified:
    - src/hooks/useLibrary.test.ts
    - src/hooks/useLibrary.ts
    - src/views/ChatView.tsx
    - src/views/SettingsView.tsx
    - e2e/helpers.ts

key-decisions:
  - "Honor __JARVIS_E2E_WIKI_EXPORT_PATH__ whenever set (no e2eMode gate) — RESEARCH A3 default"
  - "citation-excerpt stays on excerpt div; only citation button gets data-source-uri (D-11)"
  - "Defer WIKI-08/WIKI-09 requirements.mark-complete until Plan 02 ships wiki.spec"

patterns-established:
  - "Forced export path: e2ePath ?? await save(...); null/undefined early return"
  - "wikiE2eZipPath = path.join(os.tmpdir(), jarvis-e2e-wiki.zip)"

requirements-completed: [WIKI-08, WIKI-09]

coverage:
  - id: D1
    description: "exportWiki skips save when __JARVIS_E2E_WIKI_EXPORT_PATH__ is set; Vitest forced-path case"
    requirement: WIKI-09
    verification:
      - kind: unit
        ref: "src/hooks/useLibrary.test.ts#exportWiki uses __JARVIS_E2E_WIKI_EXPORT_PATH__ and skips save"
        status: pass
    human_judgment: false
  - id: D2
    description: "Citation buttons expose data-source-uri; citation-excerpt unchanged on excerpt div"
    requirement: WIKI-08
    verification:
      - kind: other
        ref: "rg -n data-source-uri src/views/ChatView.tsx; rg -n citation-excerpt src/views/ChatView.tsx"
        status: pass
    human_judgment: false
  - id: D3
    description: "Settings 保存配置 has data-testid settings-save-config"
    requirement: WIKI-09
    verification:
      - kind: other
        ref: "rg -n settings-save-config src/views/SettingsView.tsx"
        status: pass
    human_judgment: false
  - id: D4
    description: "e2e/helpers exports setWikiExportPath, wikiE2eZipPath, assertWikiZipNonEmpty"
    requirement: WIKI-09
    verification:
      - kind: other
        ref: "rg -n setWikiExportPath|wikiE2eZipPath|assertWikiZipNonEmpty e2e/helpers.ts"
        status: pass
    human_judgment: false

duration: 19min
completed: 2026-07-24
status: complete
---

# Phase 13 Plan 01: FE harness hooks Summary

**Chat `data-source-uri`, exportWiki Save bypass via `__JARVIS_E2E_WIKI_EXPORT_PATH__`, settings-save-config testid, and E2E zip helpers for Plan 02 wiki.spec**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-24T03:03:05Z
- **Completed:** 2026-07-24T03:22:46Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- TDD RED/GREEN for forced E2E export path skipping `save()` while calling `exportWikiZip`
- Citation buttons expose `data-source-uri` without moving `citation-excerpt` (qa.spec safe)
- Settings hybrid-enable click target `settings-save-config` + helpers for temp zip path/size

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — Vitest forced E2E export path skips save** - `402a9ed` (test)
2. **Task 2: GREEN — exportWiki bypass + citation URI + settings-save-config** - `b8b0f90` (feat)
3. **Task 3: E2E helpers for wiki export path and zip size** - `b494d4f` (feat)

**Plan metadata:** (this commit)

## Files Created/Modified
- `src/hooks/useLibrary.test.ts` - Forced-path Vitest case + window prop cleanup
- `src/hooks/useLibrary.ts` - `exportWiki` reads `__JARVIS_E2E_WIKI_EXPORT_PATH__`
- `src/views/ChatView.tsx` - `data-source-uri={c.source_uri}` on citation button
- `src/views/SettingsView.tsx` - `data-testid="settings-save-config"`
- `e2e/helpers.ts` - `setWikiExportPath`, `wikiE2eZipPath`, `assertWikiZipNonEmpty`

## Decisions Made
- Honor export path hook whenever set (no `e2eMode` gate) — matches RESEARCH A3 / plan interfaces
- Do not mark WIKI-08/WIKI-09 complete in REQUIREMENTS until Plan 02 delivers wiki.spec journey

## Deviations from Plan

None - plan executed exactly as written.

---

**Total deviations:** 0 auto-fixed
**Impact on plan:** N/A

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Ready for **13-02-PLAN.md** (`wiki.spec.ts` + full-ui default-off + docs). Harness hooks and Vitest contract are in place.

## Self-Check: PASSED
- key-files.modified exist on disk
- `git log --grep=13-01` shows task commits `402a9ed`, `b8b0f90`, `b494d4f`
- Acceptance: Vitest 10/10 pass; greps for `data-source-uri`, `settings-save-config`, `__JARVIS_E2E_WIKI_EXPORT_PATH__`, helpers match
- No `wiki.spec.ts` created (in scope for Plan 02 only)

---
*Phase: 13-e2e-citation-trust*
*Completed: 2026-07-24*
