---
phase: 06-settings-architecture-review
plan: "01"
subsystem: ui
tags: [react, settings, accordion, motion, tauri]
requires:
  - phase: 06-settings-architecture-review
    provides: useJarvisConfig, useAppEvents, AppShell, nested config types
provides:
  - SettingsView with 6 accordion sections (Providers, Index, Sync, Lark, Agent, Advanced)
  - All settings JSX and state moved out of App.tsx
  - Preserved settings data-testid contract
affects: [phase-06-02]
tech-stack:
  added: []
  patterns: ["SettingsView owns settings UI via useJarvisConfig", "AccordionSection with motion AnimatePresence"]
key-files:
  created:
    - src/views/SettingsView.tsx
  modified:
    - src/App.tsx
key-decisions:
  - "App.tsx retains config + refreshConfig for ChatView/LibraryView; settings state lives in useJarvisConfig"
  - "E2E orchestration hook stays in App with inline set_config when SettingsView is not mounted"
  - "保存配置 button kept at panel bottom; orchestration mode saves immediately on radio change"
patterns-established:
  - "AccordionSection: 300ms ease-out, useReducedMotion, settings-section-* testids"
requirements-completed: [FE-05]
duration: 45min
completed: 2026-07-01
status: complete
---

# Phase 06 Plan 01 Summary

**Settings extracted to accordion SettingsView with useJarvisConfig; App.tsx slimmed to 412-line view router.**

## Performance

- **App.tsx line count:** 412 (from ~1753; soft target &lt;300 deferred — cross-view handlers remain)
- **SettingsView.tsx:** ~750 lines (full settings surface)
- **Tasks completed:** 3/3 (06-01-01 through 06-01-03)

## Accomplishments

- Rewrote `SettingsView.tsx` with internal `AccordionSection` (motion/react, 300ms ease-out, reduced-motion)
- Six sections: Providers + Index (default expanded), Sync, Lark, Agent, Advanced
- Moved all settings JSX from App.tsx; preserved existing data-testid selectors
- Sync section shows inline `lastScheduledSyncError`; Lark manual sync under Advanced with hint text
- App.tsx: settings-only state/handlers removed; thin `handleSetActiveAgent` + E2E orchestration via inline `set_config`

## Self-Check: PASSED

- `npm test`: PASS (20 tests)
- `npx tsc --noEmit`: PASS

## Files Created/Modified

- `src/views/SettingsView.tsx` — accordion settings UI + useJarvisConfig integration
- `src/App.tsx` — view router only (412 lines)

## Deviations from Plan

None — plan executed as specified. App.tsx above soft 300-line target because library/memory/tasks coordination handlers remain in App per prior tasks.

## Next Phase Readiness

- Ready for 06-02 backend sync error wiring (`lastScheduledSyncError` field already on `SyncStatusView`)
- E2E `settings.spec.ts` should be run locally to validate accordion testids

---
*Phase: 06-settings-architecture-review*
*Completed: 2026-07-01*
