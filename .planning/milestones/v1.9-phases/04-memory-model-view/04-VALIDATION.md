---
phase: 4
slug: memory-model-view
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-27
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Vitest 3 (frontend) + WebdriverIO 9 (E2E) |
| **Config file** | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (E2E) |
| **Quick run command** | `cargo test -p memory && npm test` |
| **Full suite command** | `cargo test -p memory -p store && npm test && npm run build:e2e && npm run test:e2e:local -- --spec e2e/specs/memory.spec.ts e2e/specs/full-ui.spec.ts` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p memory && npm test`
- **After every plan wave:** Run wave-appropriate E2E subset after `npm run build:e2e`
- **Before `/gsd-verify-work`:** memory.spec + full-ui green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | MEM-02 | T-04-01 | rename_source_id preserves chunks atomically | unit | `cargo test -p store rename_source_id` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | MEM-01/MEM-02 | T-04-01 | UUID uris + migration idempotent | integration | `cargo test -p memory` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | MEM-02 | — | init_state wires migration (non-E2E) | compile | `cargo build --manifest-path src-tauri/Cargo.toml` | — | ⬜ pending |
| 04-02-01 | 02 | 2 | SHELL-04 | T-04-B1 | IPC names unchanged after relocation | compile | `cargo test -p tauri-app` | — | ⬜ pending |
| 04-03-01 | 03 | 3 | FE-04 | — | tauri.ts memory wrappers typecheck | unit | `npx tsc --noEmit` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 3 | FE-04 | T-04-C2 | useMemory save passes title: null | unit | `npm test -- src/hooks/useMemory.test.ts` | ❌ W0 | ⬜ pending |
| 04-03-03 | 03 | 3 | FE-04/FE-06 | T-04-C1 | memory-* testids on MemoryView | unit | `npm test && npx tsc --noEmit` | — | ⬜ pending |
| 04-04-01 | 04 | 4 | MEM-01 | — | memory crate tests green | unit | `cargo test -p memory && npm test` | — | ⬜ pending |
| 04-04-02 | 04 | 4 | D-18/D-20 | T-04-D1 | memory + full-ui E2E journeys | e2e | `npm run test:e2e:local -- --spec e2e/specs/memory.spec.ts e2e/specs/full-ui.spec.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/memory/tests/migration.rs` — legacy URI migration integration test
- [ ] `crates/store` rename_source_id unit test (inline in store.rs or tests/)
- [ ] `src/hooks/useMemory.test.ts` — hook boundary test
- [ ] `src/types/memory.ts` — domain types

*Wave 0 items are created as part of plan tasks if not pre-existing.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Legacy DB migration on real user disk | MEM-02 | E2E uses fresh DB | Dev: insert memory://1234567890 source in kb.sqlite, launch app, confirm URI becomes UUID form |
| Deprecation log on fuzzy agent match | MEM-02 | No agent E2E this phase | Dev: call forget_memory with partial title via agent tool; confirm stderr warning |

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING test file references
- [x] E2E specs exist for user-facing memory journey (D-18)

---

*Phase 4 validation strategy — 2026-06-27*
