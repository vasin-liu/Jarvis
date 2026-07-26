---
phase: 16
slug: related-docs-library-panel
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-26
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 3.x (FE) + `cargo test` (Rust) + WebdriverIO 9 / Mocha (E2E) |
| **Config file** | `vite.config.ts`; `e2e/wdio.conf.ts` |
| **Quick run command** | `npx vitest run src/views/LibraryView.test.tsx` |
| **Full suite command** | `cargo test -p retriever` && `npm test` && `npm run test:e2e:local` |
| **Estimated runtime** | ~120–300 seconds (E2E dominates) |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run src/views/LibraryView.test.tsx` (+ `cargo test -p retriever` if serde/IPC touched)
- **After every plan wave:** Run `npm test` + `cargo test -p retriever`
- **Before `/gsd-verify-work`:** Full suite must be green including related-docs E2E
- **Max feedback latency:** 60 seconds for unit; E2E at wave/phase gate

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-W0-01 | 00 | 0 | REL-03 | — | N/A | fixture | `Test-Path e2e/fixtures/related-neighbor.md` | ❌ W0 | ⬜ pending |
| 16-01-01 | 01 | 1 | REL-02 | T-16-01 | source_id NotFound → IPC error string | unit | `cargo test -p retriever related` | ✅ | ⬜ pending |
| 16-01-02 | 01 | 1 | REL-01 | T-16-01 | thin command; store sole DB owner | unit | `cargo test -p tauri-app` / compile | ❌ | ⬜ pending |
| 16-02-01 | 02 | 2 | REL-01 | — | loading / empty / error / hidden | unit | `npx vitest run src/views/LibraryView.test.tsx` | ❌ W0 | ⬜ pending |
| 16-02-02 | 02 | 2 | REL-04 | T-16-03 | title+kind+snippet text; no scores | unit | same | ❌ W0 | ⬜ pending |
| 16-02-03 | 02 | 2 | REL-03 | — | click related → select + scroll | unit | same | ❌ W0 | ⬜ pending |
| 16-03-01 | 03 | 3 | REL-01/03 | — | focused E2E under JARVIS_E2E=1 | E2E | `npm run test:e2e:local` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `e2e/fixtures/related-neighbor.md` — shares keyword with `sample.md` for MockEmbedder neighbor ranking
- [ ] `src-tauri/src/e2e.rs` — seed second fixture in `seed_e2e_fixture`
- [ ] `e2e/specs/related-docs.spec.ts` — select → panel states → click navigate
- [ ] Extend `src/views/LibraryView.test.tsx` — selection toggle, D-10/D-11/D-12, related row content
- [ ] Stable `data-testid`s on source rows + related panel (`source-row-*`, related panel/list/error)

*If none: "Existing infrastructure covers all phase requirements."* — **not applicable; Wave 0 required**

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (unit)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
