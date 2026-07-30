---
phase: 22-settings-readiness-ui
plan: 01
subsystem: ui
tags: [react, vitest, tdd, settings, embedder, readiness]

# Dependency graph
requires:
  - phase: 21-readiness-ipc
    provides: get_embedder_readiness IPC camelCase { state, message }
provides:
  - EmbedderReadinessView TS types + getEmbedderReadiness wrapper
  - useJarvisConfig embedderReadiness + poll while pending
  - Settings banner with pending/ready/failed testids and Chinese Failed hint
affects:
  - 23-release-gate (manual smoke may observe banner)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Hook owns readiness refresh + 1s poll while settingsActive && pending"
    - "Vitest mocks useJarvisConfig.embedderReadiness (no live invoke)"

key-files:
  created: []
  modified:
    - src/types/ipc.ts
    - src/lib/tauri.ts
    - src/hooks/useJarvisConfig.ts
    - src/views/SettingsView.tsx
    - src/views/SettingsView.test.tsx

key-decisions:
  - "D-01: EmbedderReadinessView + getEmbedderReadiness match Phase 21 IPC"
  - "D-02: useJarvisConfig owns refresh/poll; Settings consumes hook"
  - "D-05: Vitest extends mockUseJarvisConfig with embedderReadiness"
  - "D-06: No Rust/version bump/E2E smoke"

requirements-completed:
  - BOOT-03

---

# Phase 22 Plan 01 Summary

Settings shows local embedder readiness (Pending / Ready / Failed) with actionable Failed copy and Vitest coverage.

## What shipped

- Types + `getEmbedderReadiness()` wrapper
- Hook refresh in `refreshSettingsData` + 1s poll while pending + settings active
- Banner in index accordion (outside `indexStatus` guard) with Chinese copy + testids
- Vitest: pending + failed cases (7/7 SettingsView tests green)

## Verify

```
npx vitest run src/views/SettingsView.test.tsx
# 7 passed
```

## Commits

- `d08af2b` test(settings): RED Vitest for embedder readiness banner
- `d4657b1` feat(settings): show embedder readiness banner with pending poll
