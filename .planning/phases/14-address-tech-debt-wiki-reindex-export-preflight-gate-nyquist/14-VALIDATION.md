---
phase: 14
slug: address-tech-debt-wiki-reindex-export-preflight-gate-nyquist
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-24
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

**Nyquist / verify gate (D-15):** Phase 14 ship gate is relevant **cargo test** + **Vitest** green + VALIDATION/ROADMAP docs updated. **No new E2E** (D-04/D-15 waiver vs `e2e-required.mdc`). Existing `wiki.spec.ts` remains the wiki journey.

**Closeout bar (D-13 / D-14):** After Phase 14 VERIFICATION, run `/gsd-audit-milestone` expecting `status: passed`, then `/gsd-complete-milestone v1.10`. **WIKI-F01** and dual `index.md` writers may remain **Deferred** / backlog — clearing the four Phase-14 audit items is enough for `passed` (D-14).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (`src-tauri` `index_ops` + `insights`) + Vitest ^3.2.4 |
| **Config file** | workspace `Cargo.toml` / `vite.config.ts` (`test.include`: `src/**/*.test.ts`) |
| **Quick run command** | `cargo test -p tauri-app index_ops -- --nocapture` && `cargo test -p insights wiki -- --nocapture` && `npx vitest run src/views/LibraryView.test.tsx` |
| **Full suite command** | Same filters green (Phase 14 gate per D-15); **no** `npm run test:e2e:local` |
| **Estimated runtime** | ~30–90 seconds |

---

## Sampling Rate

- **After every task commit:** Run task-scoped cargo filter **or** single Vitest file (docs tasks: frontmatter/rg only)
- **After every plan wave:** Soft-skip + preflight Rust tests + LibraryView Vitest; Wave 2 docs via rg/read
- **Before `/gsd-verify-work`:** Full Phase 14 gate green + VALIDATION/ROADMAP docs updated (no new E2E)
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | Soft skip D-01/D-05 | T-14-01 | Early-return before `delete_chunks`; no Failed | unit | `cargo test -p tauri-app index_ops` | ✅ | ✅ green |
| 14-01-02 | 01 | 1 | Hide retry D-03/D-04 | — | `retry-source-*` absent for wiki_page | unit | `npx vitest run src/views/LibraryView.test.tsx` | ✅ | ✅ green |
| 14-02-01 | 02 | 1 | Preflight D-06/D-09 | T-14-02 | `enabled=false` → WikiDisabled | unit | `cargo test -p insights` | ✅ | ✅ green |
| 14-03-01 | 03 | 2 | Nyquist D-10..D-12 | — | `nyquist_compliant: true` + evidence note | docs | `rg -n "nyquist_compliant: true" …/12-VALIDATION.md …/13-VALIDATION.md` | ✅ | ✅ green |
| 14-03-02 | 03 | 2 | ROADMAP/closeout D-13..D-15 | — | Coverage Complete; plan list; D-13/D-15 notes | docs | `rg -n "WIKI-08\|WIKI-09\|14-01-PLAN\|14-02-PLAN\|14-03-PLAN" .planning/ROADMAP.md` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*E2E waiver:* CONTEXT D-04/D-15 — no new E2E for hide-button / soft-skip / Nyquist docs.

---

## Wave 0 Requirements

- [x] Rust unit: WikiPage soft skip — not Failed, chunks preserved, bulk continues
- [x] Rust unit: preflight/disabled → `InsightsError::WikiDisabled` (`wiki is disabled`)
- [x] Vitest: hide `retry-source-*` for `wiki_page`
- [x] Docs: flip 12/13 `nyquist_compliant` + evidence notes (not tests)

*Existing infrastructure covers frameworks — no new install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Post-phase milestone audit `passed` | D-13 | Process gate | After Phase 14 VERIFICATION: `/gsd-audit-milestone` (expect `passed`) → `/gsd-complete-milestone v1.10` |
| ROADMAP WIKI-08/09 Coverage Complete | Audit item 4 | Docs confirm | Coverage table rows remain Complete (do not reopen to Pending) |
| Deferred F01 / dual writers OK | D-14 | Backlog allowed | Confirm audit may still list WIKI-F01 + dual `index.md` writers as Deferred |

*All code behaviors have automated verification (Rust + Vitest). Docs closeout has rg/read gates.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter (after Wave 1 green + 14-03 docs)
- [x] D-15 verify gate documented (cargo + Vitest + docs; no new E2E)
- [x] D-13 re-audit bar documented (`/gsd-audit-milestone` → passed → `/gsd-complete-milestone v1.10`)

**Approval:** pending phase VERIFICATION + milestone re-audit
