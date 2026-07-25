---
phase: 07
slug: wiki-kind-config
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-17
updated: 2026-07-18
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Vitest (frontend) |
| **Config file** | `Cargo.toml` workspace; `vite.config.ts` (Vitest) |
| **Quick run command** | `cargo test -p store -p config` && `npm test -- --run src/lib/sourceDisplay.test.ts src/types/config.test.ts` |
| **Full suite command** | `cargo test -p store -p config` && `npm test` |
| **Estimated runtime** | ~60–180 seconds (cold compile higher) |
| **Accepted cargo latency** | `cargo test -p store|config` and `cargo check -p tauri-app` may exceed 30s (often 60–180s). Plans use fast `rg` pre-checks before cargo; Nyquist feedback budget for this phase is **≤180s**, not ≤30s. |

---

## Sampling Rate

- **After every task commit:** Run the task's `<automated>` (rg pre-check + cargo/Vitest as specified)
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds (accepted for Rust compile/test on this phase)

---

## Per-Task Verification Map

Plans use **inline TDD** (RED→GREEN inside each `tdd="true"` task). No separate Wave 0 plan; no `MISSING` automated refs.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | WIKI-02 | T-07-03 | Kind round-trip | unit | `rg … types.rs && cargo test -p store wiki_page_kind_roundtrips source_kind_roundtrips` | ✅ extend `crates/store/src/types.rs` | ⬜ pending |
| 07-01-02 | 01 | 1 | WIKI-02 | T-07-01 | Fail-closed reindex | compile | `rg … index_ops.rs && cargo check -p tauri-app` | ✅ `src-tauri/src/index_ops.rs` | ⬜ pending |
| 07-02-01 | 02 | 1 | WIKI-01 | T-07-04/05 | Nested defaults + round-trip | unit | `cargo test -p config` | ✅ extend `crates/config/src/types.rs` + `lib.rs` | ⬜ pending |
| 07-03-01 | 03 | 1 | WIKI-02 | T-07-09 | Label only | unit | `npm test -- --run src/lib/sourceDisplay.test.ts` | ✅ | ⬜ pending |
| 07-03-02 | 03 | 1 | WIKI-01 | T-07-08 | FE pass-through | unit | `npm test -- --run src/types/config.test.ts src/lib/sourceDisplay.test.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Plan 02 is a single RED→GREEN TDD task (merged; mirrors 07-01 Task 1). Task IDs: 07-02-01 only.*

---

## Wave 0 Requirements

Wave 0 is **complete / N/A** for this phase:

- [x] No Wave 0 plan required — existing cargo + Vitest infrastructure is sufficient
- [x] Store wiki round-trip tests authored inline in plan 01 Task 1 (TDD)
- [x] Config default-off + round-trip tests authored inline in plan 02 Task 1 (TDD)
- [x] Vitest `sourceKindLabel("wiki_page")` case authored inline in plan 03 Task 1 (TDD)
- [x] No `MISSING` `<automated>` references in any PLAN.md

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No Settings wiki toggle | D-10 / WIKI-01 | Plan 03 greps SettingsView; optional visual confirm | Open Settings; confirm no wiki enable control |
| `auto_on_insights` unwired | D-09 | Plan 02 acceptance greps insights_ops | Confirm `insights_ops` does not read `wiki.auto_on_insights` |

*E2E wiki journey deferred to Phase 13.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify (inline TDD; no Wave 0 MISSING deps)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 N/A — plans author tests in-task; checklist aligned with 07-01/02/03 PLAN.md
- [x] No watch-mode flags
- [x] Feedback latency ≤180s accepted for cargo (documented above); rg pre-checks on plan 01
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ready for execute
