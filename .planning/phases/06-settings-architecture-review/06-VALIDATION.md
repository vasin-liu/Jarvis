---
phase: 6
slug: settings-architecture-review
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-30
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (workspace); Vitest 3; WebdriverIO 9 + Mocha |
| **Config file** | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (WdIO) |
| **Quick run command** | `npm test` (Vitest only) |
| **Full suite command** | `cargo test --workspace && npm test && npm run test:e2e:local` |
| **Estimated runtime** | ~120 seconds (E2E dominates) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tauri-app` (Rust) or `npm test` (TS)
- **After every plan wave:** `cargo test --workspace && npm test && npx tsc --noEmit`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | FE-05 | — | N/A | unit | `npm test -- useJarvisConfig` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | FE-05 | — | N/A | unit | `npm test -- config` | ❌ W0 | ⬜ pending |
| 06-01-03 | 01 | 1 | FE-05 | T-06-01 | Error strings rendered as text nodes | E2E | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ✅ | ⬜ pending |
| 06-02-01 | 02 | 2 | QA-02 | — | N/A | integration | `cargo test -p store` | ✅ | ⬜ pending |
| 06-02-02 | 02 | 2 | FE-05 | — | N/A | E2E | `npm run test:e2e:local -- --spec e2e/specs/settings.spec.ts` | ✅ | ⬜ pending |
| 06-03-01 | 03 | 3 | SHELL-01 | — | N/A | compile | `cargo build -p tauri-app` | ✅ | ⬜ pending |
| 06-03-02 | 03 | 3 | QA-02 | — | N/A | doc | `06-VERIFICATION.md` checklist | ❌ W0 | ⬜ pending |
| 06-03-03 | 03 | 3 | QA-04 | — | No new IPC commands | review | manual diff check | manual | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/hooks/useJarvisConfig.ts` + unit test — covers FE-05 hook API
- [ ] `src/types/config.ts` + `src/types/config.test.ts` — covers FE-05 D-29 roundtrip
- [ ] `src-tauri/src/bootstrap.rs` — covers SHELL-01 D-30
- [ ] `src-tauri/src/events.rs` — covers SHELL-01 D-31
- [ ] `e2e/specs/settings.spec.ts` extension — accordion sections + sync error
- [ ] `06-VERIFICATION.md` — QA-02 architecture checklist

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| QA-04 refactor-only scope | QA-04 | Diff review for new user-facing features | Verify no new Tauri commands or routes beyond locked UX |
| Architecture sign-off | QA-02 | Checklist judgment | Complete `06-VERIFICATION.md` with line counts and module boundaries |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
