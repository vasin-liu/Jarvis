---
phase: 1
slug: scaffold-shell-foundation
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-17
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (workspace) + Vitest + WebdriverIO (tauri-driver) |
| **Config file** | `e2e/wdio.conf.ts`, `vite.config.ts` (Vitest) |
| **Quick run command** | `cargo test --workspace` (Rust touch) / `npm test` (frontend touch) |
| **Full suite command** | `cargo test --workspace && npm test && npm run test:e2e:local` |
| **Estimated runtime** | ~5–8 minutes (full gate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace` (Rust) or `npm test` (frontend-only)
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~300 seconds (E2E build + wdio)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01 | 01 | 1 | SHELL-01 | T-01 / — | No new network/shell surfaces | unit/integration | `cargo test --workspace` | ✅ | ⬜ pending |
| 01-02 | 01 | 1 | FE-01 | — | N/A | build | `npm run build` | ✅ | ⬜ pending |
| 01-03 | 01 | 2 | QA-01, QA-03 | T-01 | Config write path unchanged | e2e | `npm run test:e2e:local` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — no new test files required for zero-behavior-change scaffold.

- [x] `e2e/specs/smoke.spec.ts` — smoke gate
- [x] `e2e/specs/navigation.spec.ts` — navigation gate
- [x] `cargo test --workspace` — Rust regression
- [x] `npm test` — Vitest helpers

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| IPC registry count (52 commands) | SHELL-01 | Structural grep | Verify `generate_handler!` lists same command names as pre-refactor |

All phase behaviors have automated verification except optional IPC count spot-check.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 300s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
