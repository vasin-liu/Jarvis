---
phase: 14
slug: address-tech-debt-wiki-reindex-export-preflight-gate-nyquist
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-24
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

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

- **After every task commit:** Run task-scoped cargo filter **or** single Vitest file
- **After every plan wave:** Soft-skip + preflight Rust tests + LibraryView Vitest
- **Before `/gsd-verify-work`:** Full Phase 14 gate green + VALIDATION/ROADMAP docs updated
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | Soft skip D-01/D-05 | T-14-01 | Early-return before `delete_chunks`; no Failed | unit | `cargo test -p tauri-app index_ops` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | Hide retry D-03/D-04 | — | `retry-source-*` absent for wiki_page | unit | `npx vitest run src/views/LibraryView.test.tsx` | ✅ file / ❌ case | ⬜ pending |
| 14-02-01 | 02 | 1 | Preflight D-06/D-09 | T-14-02 | `enabled=false` → WikiDisabled | unit | `cargo test -p insights` | ❌ W0 | ⬜ pending |
| 14-03-01 | 03 | 2 | Nyquist D-10..D-12 | — | `nyquist_compliant: true` + evidence note | docs | Read 12/13 VALIDATION frontmatter | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*E2E waiver:* CONTEXT D-04/D-15 — no new E2E for hide-button / soft-skip.

---

## Wave 0 Requirements

- [ ] Rust unit: WikiPage soft skip — not Failed, chunks preserved, bulk continues
- [ ] Rust unit: preflight/disabled → `InsightsError::WikiDisabled` (`wiki is disabled`)
- [ ] Vitest: hide `retry-source-*` for `wiki_page`
- [ ] Docs: flip 12/13 `nyquist_compliant` + evidence notes (not tests)

*Existing infrastructure covers frameworks — no new install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Post-phase milestone audit `passed` | D-13 | Process gate | After Phase 14 VERIFICATION: `/gsd-audit-milestone` then `/gsd-complete-milestone v1.10` |
| ROADMAP WIKI-08/09 Coverage Complete | Audit item 4 | Already synced | Confirm Coverage table still Complete |

*All code behaviors have automated verification (Rust + Vitest).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter (after phase execute/verify)

**Approval:** pending
