---
phase: 3
slug: library-tasks-keychain
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-26
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace) + Vitest 3 (frontend) + WebdriverIO 9 (E2E) |
| **Config file** | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (E2E) |
| **Quick run command** | `cargo test -p config && npm test` |
| **Full suite command** | `cargo test --workspace && npm test && npm run test:e2e:local` |
| **Estimated runtime** | ~150 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p config && npm test`
- **After every plan wave:** Run `cargo test --workspace && npm run test:e2e:local`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 150 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | CFG-02/CFG-03 | T-03-01 | Plaintext key stripped from JSON on save | unit | `cargo test -p config` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | CFG-03 | T-03-01 | Legacy plaintext migrates to keychain | integration | `cargo test -p config` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 2 | SHELL-03 | — | index/lark commands compile; IPC names unchanged | unit | `cargo test --workspace` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 3 | FE-03 | — | useLibrary owns library IPC state | unit | `npm test` | ❌ W0 | ⬜ pending |
| 03-03-02 | 03 | 3 | FE-03/FE-06 | — | Library testids preserved; Lark URL paste hidden | e2e | `npm run test:e2e:local -- --spec e2e/specs/lark.spec.ts e2e/specs/full-ui.spec.ts` | ✅ | ⬜ pending |
| 03-04-01 | 04 | 4 | FE-03 | — | useTasks owns task IPC state | unit | `npm test` | ❌ W0 | ⬜ pending |
| 03-04-02 | 04 | 4 | FE-03/FE-06 | — | Tasks testids preserved | e2e | `npm run test:e2e:local -- --spec e2e/specs/full-ui.spec.ts` | ✅ | ⬜ pending |
| 03-05-01 | 05 | 5 | CFG-02/D-01/D-04 | T-03-02 | Masked placeholder + Clear API key button | e2e | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ✅ | ⬜ pending |
| 03-05-02 | 05 | 5 | D-26/D-28 | — | Full E2E suite green | e2e | `npm run test:e2e:local` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/config/src/secrets.rs` — unit tests with `#[ignore = "requires Windows Credential Manager"]` for live keyring
- [ ] `crates/config/tests/migration.rs` — plaintext key migration integration test
- [ ] `src/hooks/useLibrary.test.ts` — hook boundary tests
- [ ] `src/hooks/useTasks.test.ts` — hook boundary tests
- [ ] `e2e/specs/lark.spec.ts` — update for Settings-based sync (D-16)

*Wave 0 items are created as part of plan tasks if not pre-existing.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real Windows Credential Manager roundtrip | CFG-02 | CI skips keyring (D-03) | Dev machine: set API key in Settings, restart app, confirm masked placeholder and cloud provider works |
| Legacy config migration on real disk | CFG-03 | E2E uses fresh fixture config | Place plaintext `cloud_api_key` in config.json, launch app, confirm field removed from JSON and key in keychain |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 150s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-26
