# Phase 22: Settings readiness UI — Context

**Gathered:** 2026-07-29  
**Status:** Ready for planning  
**Source:** Design + implementation plan express path (`/gsd-plan-phase 22 --chain`)

## Phase Goal

Users can see local embedder init status in Settings and get an actionable Failed hint (BOOT-03).

## Constraints

- Depends on Phase 21: `get_embedder_readiness` IPC
- Match existing Settings dark tech / glass styling (no marketing redesign)
- Stable `data-testid`s for Vitest (and future E2E)
- No version bump / release smoke (Phase 23)
- Spec: `docs/superpowers/specs/2026-07-29-release-hardening-design.md`
- Plan draft: `docs/superpowers/plans/2026-07-29-release-hardening.md` Task 3
- Frontend: `.cursor/rules/frontend-taste.mdc` — loading/empty/error honesty; honor reduced motion

## Decisions

### D-01: Types + tauri wrapper
**Choice:** `EmbedderReadinessView` / `EmbedderReadyState` in `src/types/ipc.ts`; `getEmbedderReadiness()` in `src/lib/tauri.ts`  
**Why:** Matches Phase 21 camelCase IPC payload

### D-02: Hook owns live readiness + poll
**Choice:** `useJarvisConfig` holds `embedderReadiness`, `refreshEmbedderReadiness`; include in `refreshSettingsData`; while `state === "pending"` and `settingsActive`, poll every 1000ms; clear interval on unmount / non-pending  
**Why:** Settings already uses this hook for indexStatus; keeps App.tsx thin

### D-03: Banner placement + copy (Chinese)
**Choice:** Banner near index-status accordion (above or inside); copy:
- pending: 「本地嵌入模型加载中…」
- ready: 「本地嵌入模型已就绪」
- failed: 「本地嵌入模型加载失败：{message}。请检查模型/缓存目录，或在上方改用 Mock / Ollama。」  
**Why:** BOOT-03 actionable Failed hint; product UI is Chinese

### D-04: Testids
**Choice:** `embedder-readiness`, `embedder-readiness-pending`, `embedder-readiness-ready`, `embedder-readiness-failed`  
**Why:** Spec + plan contract

### D-05: Vitest via hook mock
**Choice:** Extend `useJarvisConfig` return + `SettingsView.test.tsx` `mockUseJarvisConfig` with `embedderReadiness` / `refreshEmbedderReadiness`; Vitest sets pending/failed mocks  
**Why:** Existing Settings tests already mock the whole hook — consistent and simple

### D-06: Scope fence
**Choice:** No Rust changes (unless type drift discovered); no version bump; no full E2E FastEmbed smoke  
**Why:** Phase 23 owns release smoke; BOOT-03 is UI + Vitest

## Claude's Discretion

- Exact banner styling within existing zinc/amber panel patterns
- Whether ready state is visually quiet (subtle) vs same weight as pending

## Deferred Ideas

- Release smoke / 1.12.0 — Phase 23
- Dedicated WebDriver E2E for readiness — optional later; Vitest sufficient for BOOT-03
- Auto-fallback provider — out of scope

## Success Criteria (from ROADMAP)

1. Settings shows Pending / Ready / Failed with stable `data-testid`s
2. Failed state shows error message + actionable hint
3. While Pending, UI refreshes readiness (poll) until Ready or Failed
4. Vitest covers pending and failed rendering
