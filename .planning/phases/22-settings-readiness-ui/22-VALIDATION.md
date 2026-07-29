---
phase: 22
slug: settings-readiness-ui
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-29
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 3 + Testing Library |
| **Config file** | `vite.config.ts` |
| **Quick run command** | `npx vitest run src/views/SettingsView.test.tsx` |
| **Full suite command** | `npx vitest run` |
| **Estimated runtime** | ~15–40 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run src/views/SettingsView.test.tsx`
- **After every plan wave:** Run `npx vitest run src/views/SettingsView.test.tsx`
- **Before `/gsd-verify-work`:** SettingsView Vitest green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------------|-----------|-------------------|-------------|--------|
| 22-01-01 | 01 | 1 | BOOT-03 | — | N/A | unit (RED) | `npx vitest run src/views/SettingsView.test.tsx` | ✅ | ⬜ pending |
| 22-01-02 | 01 | 1 | BOOT-03 | — | N/A | unit (GREEN) | `npx vitest run src/views/SettingsView.test.tsx` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements (Vitest + SettingsView.test.tsx mock pattern).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live poll while FastEmbed pending | BOOT-03 | Needs real/deferred FastEmbed cold start | Phase 23 smoke; open Settings during first load |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-29
