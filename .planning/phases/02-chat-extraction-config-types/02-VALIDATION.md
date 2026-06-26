---
phase: 2
slug: chat-extraction-config-types
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-26
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace) + Vitest 3 (frontend) + WebdriverIO 9 (E2E) |
| **Config file** | `vite.config.ts` (Vitest); `e2e/wdio.conf.ts` (E2E) |
| **Quick run command** | `cargo test -p config && npm test` |
| **Full suite command** | `cargo test --workspace && npm test && npm run test:e2e:local` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p config && npm test`
- **After every plan wave:** Run `cargo test --workspace && npm run test:e2e:local`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | CFG-01 | — | Flat config.json loads unchanged | unit | `cargo test -p config` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | SHELL-02 | — | Chat IPC names unchanged | unit | `cargo test --workspace` | ✅ | ⬜ pending |
| 02-03-01 | 03 | 3 | FE-02 | — | useChat owns RAG state only | unit | `npm test` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 3 | FE-06 | — | testids preserved + agent-submit | e2e | `npm run test:e2e:local` | ✅ | ⬜ pending |
| 02-03-03 | 03 | 3 | FE-02/FE-06 | — | RAG + agent journeys green | e2e | `npm run test:e2e:local -- --spec e2e/specs/qa.spec.ts e2e/specs/agent.spec.ts e2e/specs/full-ui.spec.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/config/src/types.rs` — `config_flat_json_roundtrip` unit test for CFG-01
- [ ] `src/hooks/useChat.test.ts` — hook boundary tests for FE-02
- [ ] `e2e/helpers.ts` — `askAgentQuestion()` helper using `agent-submit` testid

*Wave 0 items are created as part of plan tasks if not pre-existing.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Existing user config.json loads | CFG-01 | Real user data path | Launch app with existing config; confirm no load error |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
