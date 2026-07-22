---
phase: 11
slug: library-settings-ui
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-22
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Aligned to WIKI-06 and `11-RESEARCH.md` Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 3 + `@testing-library/react` + jsdom |
| **Config file** | `vite.config.ts` (`test.include: ["src/**/*.test.ts"]` — may need `.tsx` include if missing) |
| **Quick run command** | `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts src/views/SettingsView.test.tsx` |
| **Full suite command** | `npx vitest run` |
| **Estimated runtime (per-task)** | ≤15 seconds |
| **Estimated runtime (wave/phase gate)** | ≤60 seconds |
| **E2E** | Deferred to Phase 13 (`wiki.spec.ts`); default E2E config keeps wiki disabled |

---

## Sampling Rate

- **After every task commit:** Filtered Vitest files touched by that task
- **After every plan wave:** `npx vitest run` for phase-related specs
- **Before `/gsd-verify-work`:** Full `npx vitest run` green
- **Max feedback latency (task):** 15 seconds
- **Max feedback latency (wave/phase gate):** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | WIKI-06 | — | N/A | unit | `npx vitest run src/views/SettingsView.test.tsx` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 2 | WIKI-06 | — | N/A | unit | `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` | ❌ W0 / ✅ hook | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/views/LibraryView.test.tsx` — visibility stubs (enabled / disabled / wiki_page)
- [ ] `src/views/SettingsView.test.tsx` — section + toggle stubs (or co-locate if planner prefers)
- [ ] Confirm `vite.config.ts` `test.include` matches `*.test.tsx` if new view tests use `.tsx`

*Existing: `src/hooks/useLibrary.test.ts`, `src/types/config.test.ts` (wiki round-trip).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Full enable → compile → note rows in packaged app | WIKI-06 / WIKI-08 | Owned by Phase 13 E2E | Phase 13 `wiki.spec.ts` |

*Phase 11 ship gate is Vitest visibility + wiring.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s for task verifies
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
