---
phase: 11-library-settings-ui
plan: "02"
subsystem: ui
tags: [react, vitest, library, wiki, compile, tauri]

requires:
  - phase: 11-01
    provides: Settings wiki.enabled toggle + AppConfig.wiki round-trip
  - phase: 10-persist-index
    provides: compile_wiki_cmd backend
provides:
  - WikiCompileSummary FE type + tauri.compileWiki wrapper
  - useLibrary.compileWiki (invoke + refreshSources + onError)
  - LibraryView 生成笔记 gated by wiki.enabled
  - App handleCompileWiki busy wrapper
affects: [12-obsidian-export, 13-e2e]

tech-stack:
  added: []
  patterns:
    - "Library compile button gated config?.wiki?.enabled === true && kind !== wiki_page"
    - "App busy wrapper owns setBusy; useLibrary never exposes busy"
    - "WikiCompileSummary typed but discarded in UX (D-11)"

key-files:
  created:
    - src/views/LibraryView.test.tsx
  modified:
    - src/types/ipc.ts
    - src/lib/tauri.ts
    - src/hooks/useLibrary.ts
    - src/hooks/useLibrary.test.ts
    - src/views/LibraryView.tsx
    - src/App.tsx

key-decisions:
  - "Visibility: config?.wiki?.enabled === true && indexed && kind !== wiki_page (D-05..D-08)"
  - "Busy in App handleCompileWiki only; hook mirrors summarizeSource (D-09)"
  - "Discard WikiCompileSummary return in UX; errors via reportError/setErr (D-10, D-11)"

patterns-established:
  - "LibraryView Vitest: motion mock + baseProps + wiki visibility matrix"
  - "compileWiki: tauri → useLibrary refresh → App busy → onCompileWiki prop"

requirements-completed: [WIKI-06]

coverage:
  - id: D1
    description: "WikiCompileSummary IPC type + compileWiki → compile_wiki_cmd"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/hooks/useLibrary.test.ts#compileWiki calls tauri and refreshSources"
        status: pass
    human_judgment: false
  - id: D2
    description: "useLibrary.compileWiki invoke + refresh + onError; no busy on hook"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/hooks/useLibrary.test.ts#compileWiki reports errors via onError"
        status: pass
      - kind: unit
        ref: "src/hooks/useLibrary.test.ts#does not expose indexProgress or busy"
        status: pass
    human_judgment: false
  - id: D3
    description: "Library shows 生成笔记 (wiki-compile-{id}) when enabled + indexed non-wiki_page"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#shows 生成笔记 when wiki.enabled and indexed local_file"
        status: pass
    human_judgment: false
  - id: D4
    description: "No wiki-compile controls when wiki disabled or missing"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#hides compile when wiki.enabled is false"
        status: pass
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#hides compile when wiki is missing"
        status: pass
    human_judgment: false
  - id: D5
    description: "Indexed wiki_page never shows 生成笔记; rows remain when disabled (D-04/D-06)"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#hides compile for indexed wiki_page even when enabled"
        status: pass
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#still lists wiki_page rows when wiki is disabled"
        status: pass
    human_judgment: false
  - id: D6
    description: "App handleCompileWiki busy wrapper + onCompileWiki wiring"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/LibraryView.test.tsx#clicking 生成笔记 calls onCompileWiki with source id"
        status: pass
    human_judgment: false

duration: 18min
completed: 2026-07-23
status: complete
---

# Phase 11 Plan 02: Library Compile Wiring Summary

**WIKI-06 Library half: gated「生成笔记」→ compile_wiki_cmd with Vitest visibility + invoke path**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-23T02:39:46Z
- **Completed:** 2026-07-23T02:57:26Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added LibraryView Vitest gate matrix (enabled / disabled / missing wiki / wiki_page / row retention / click)
- Extended useLibrary tests for compileWiki typeof, invoke+refresh, onError, no busy
- Shipped WikiCompileSummary type, tauri.compileWiki, useLibrary.compileWiki, Library button + App busy wrapper

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — Library visibility + useLibrary compileWiki tests** - `754650a` (test)
2. **Task 2: GREEN — WikiCompileSummary + tauri + useLibrary** - `e54c12f` (feat)
3. **Task 3: GREEN — Library 生成笔记 + App busy wiring** - `283f839` (feat)

**Plan metadata:** (this commit)

## Files Created/Modified

- `src/views/LibraryView.test.tsx` - jsdom Vitest visibility + click for wiki-compile
- `src/hooks/useLibrary.test.ts` - compileWiki mock + invoke/onError cases
- `src/types/ipc.ts` - `WikiCompileSummary` camelCase interface
- `src/lib/tauri.ts` - `compileWiki` → `compile_wiki_cmd`
- `src/hooks/useLibrary.ts` - `compileWiki` helper (no busy)
- `src/views/LibraryView.tsx` - gated「生成笔记」+ `onCompileWiki`
- `src/App.tsx` - `handleCompileWiki` + LibraryView prop

## Decisions Made

- Followed D-04..D-12, D-14, D-16 and PATTERNS prescribed blocks exactly
- Full enable→compile→export E2E deferred to Phase 13; Vitest is phase gate
- No Rust / Settings / export changes in this plan

## Deviations from Plan

None - plan executed exactly as written.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED | `754650a` test(11-02) | Pass — 5 failures before implementation |
| GREEN (hook/IPC) | `e54c12f` feat(11-02) | Pass — useLibrary 5/5 green |
| GREEN (Library/App) | `283f839` feat(11-02) | Pass — Library+hook+Settings 16/16 green |
| REFACTOR | — | Skipped (no cleanup needed) |

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 11 plans complete — ready for `/gsd-verify-work 11` / next phase planning
- WIKI-06 satisfied for Phase 11 (Settings 11-01 + Library 11-02)
- Phase 12 owns export UI; Phase 13 owns `wiki.spec.ts`

## Self-Check: PASSED

- [x] `src/views/LibraryView.test.tsx` exists and references `wiki-compile-` / `生成笔记`
- [x] `useLibrary.test.ts` references `compileWiki`
- [x] `WikiCompileSummary` / `compile_wiki_cmd` / `handleCompileWiki` present
- [x] `git log --grep="11-02"` returns RED + two GREEN commits
- [x] `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts src/views/SettingsView.test.tsx` exits 0 (16 passed)

---
*Phase: 11-library-settings-ui*
*Completed: 2026-07-23*
