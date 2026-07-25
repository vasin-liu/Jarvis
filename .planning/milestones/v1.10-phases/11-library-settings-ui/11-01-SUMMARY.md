---
phase: 11-library-settings-ui
plan: "01"
subsystem: ui
tags: [react, vitest, settings, wiki, feature-flag]

requires:
  - phase: 07-wiki-kind-config
    provides: AppConfig.wiki nested flags (enabled, auto_on_insights)
provides:
  - SettingsView Wiki 笔记 AccordionSection with wiki-enabled-toggle
  - SettingsView Vitest covering section, checked state, local-only setConfig
affects: [11-02-library-compile, 13-e2e]

tech-stack:
  added: []
  patterns:
    - "Settings AccordionSection sibling after sync / before Lark"
    - "wiki.enabled local setConfig; persist only via 保存配置"
    - "Preserve auto_on_insights when spreading wiki"

key-files:
  created:
    - src/views/SettingsView.test.tsx
  modified:
    - src/views/SettingsView.tsx

key-decisions:
  - "Independent Wiki 笔记 AccordionSection (D-01); only wiki.enabled exposed (D-02)"
  - "Toggle mutates local config via setConfig; no handleSaveConfig on checkbox (D-03)"
  - "Locked UI-SPEC Chinese copy; helper Discretion copy from PATTERNS"

patterns-established:
  - "SettingsView Vitest: mock useJarvisConfig + motion/react; expand Accordion before querying checkbox"
  - "Wiki toggle spread preserves auto_on_insights"

requirements-completed: [WIKI-06]

coverage:
  - id: D1
    description: "Settings AccordionSection Wiki 笔记 with data-testid settings-section-wiki"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/SettingsView.test.tsx#renders Wiki 笔记 section and 启用 Wiki 笔记层 toggle after expand"
        status: pass
    human_judgment: false
  - id: D2
    description: "wiki-enabled-toggle reflects config.wiki.enabled and label 启用 Wiki 笔记层"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/SettingsView.test.tsx#checkbox unchecked when wiki.enabled is false"
        status: pass
      - kind: unit
        ref: "src/views/SettingsView.test.tsx#checkbox checked when wiki.enabled is true"
        status: pass
    human_judgment: false
  - id: D3
    description: "Toggle calls setConfig with enabled flipped + auto_on_insights preserved; handleSaveConfig not called"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/SettingsView.test.tsx#toggle calls setConfig with enabled flipped and auto_on_insights preserved; does not save"
        status: pass
    human_judgment: false
  - id: D4
    description: "Wiki section exposes exactly one checkbox (no auto_on_insights control)"
    requirement: WIKI-06
    verification:
      - kind: unit
        ref: "src/views/SettingsView.test.tsx#wiki section has exactly one checkbox (no auto_on_insights control)"
        status: pass
    human_judgment: false

duration: 13min
completed: 2026-07-23
status: complete
---

# Phase 11 Plan 01: Settings Wiki Toggle Summary

**Independent「Wiki 笔记」Settings AccordionSection with local-only `wiki.enabled` toggle and green Vitest (WIKI-06 Settings half)**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-23T02:20:32Z
- **Completed:** 2026-07-23T02:33:10Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added failing SettingsView Vitest for wiki section, checked state, local-only mutate, single checkbox
- Shipped AccordionSection `settings-section-wiki` / `wiki-enabled-toggle` with locked UI-SPEC copy after sync, before Lark
- Vitest green: `npx vitest run src/views/SettingsView.test.tsx` (5 tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — SettingsView wiki section Vitest** - `a46d34d` (test)
2. **Task 2: GREEN — Wiki 笔记 AccordionSection + enabled toggle** - `0bf7a67` (feat)

**Plan metadata:** (this commit)

## Files Created/Modified

- `src/views/SettingsView.test.tsx` - jsdom Vitest; mocks `useJarvisConfig` + `motion/react`
- `src/views/SettingsView.tsx` - Wiki 笔记 AccordionSection + enabled checkbox

## Decisions Made

- Followed D-01/D-02/D-03/D-13/D-15/D-16 and PATTERNS prescribed block exactly
- Helper copy from UI-SPEC Discretion (PATTERNS): 开启后可在资料库…
- Split enabled true/false into separate tests + explicit `cleanup()` for jsdom isolation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test isolation for double-render / leftover DOM**
- **Found during:** Task 2 (GREEN verification)
- **Issue:** First GREEN run failed with multiple `settings-section-wiki` elements — checkbox case rendered twice without cleanup; subsequent tests inherited DOM
- **Fix:** Added `cleanup()` in beforeEach/afterEach; split checked/unchecked into two tests (plan allowed mock/test adjust for GREEN)
- **Files modified:** `src/views/SettingsView.test.tsx`
- **Verification:** `npx vitest run src/views/SettingsView.test.tsx` exits 0 (5 passed)
- **Committed in:** `0bf7a67` (Task 2)

---

**Total deviations:** 1 auto-fixed (1 bug/test harness)
**Impact on plan:** No scope creep; assertions still match CONTEXT/UI-SPEC must_haves.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED | `a46d34d` test(11-01) | Pass — 4 failures before UI |
| GREEN | `0bf7a67` feat(11-01) | Pass — 5 tests green |
| REFACTOR | — | Skipped (no cleanup needed beyond test isolation in GREEN) |

## Issues Encountered

None beyond the jsdom cleanup fix above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for **11-02** (Library「生成笔记」+ compileWiki wiring)
- Do not mark full WIKI-06 done until 11-02 ships Library half
- No Library/App/tauri changes in this plan; no wiki.spec.ts (Phase 13)

## Self-Check: PASSED

- [x] `src/views/SettingsView.test.tsx` exists and references `settings-section-wiki` / `wiki-enabled-toggle`
- [x] `settings-section-wiki` after sync, before lark in `SettingsView.tsx`
- [x] Locked copy: Wiki 笔记 / 启用 Wiki 笔记层
- [x] `git log --grep="11-01"` returns RED + GREEN commits
- [x] `npx vitest run src/views/SettingsView.test.tsx` exits 0

---
*Phase: 11-library-settings-ui*
*Completed: 2026-07-23*
