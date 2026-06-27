---
phase: 5
slug: agent-protocol
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-27
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (agent/llm) + Vitest 3 + WebdriverIO 9 |
| **Quick run command** | `cargo test -p agent && npm test` |
| **Full suite command** | `cargo test -p agent -p llm && npm test && npm run build:e2e && npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts e2e/specs/full-ui.spec.ts` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task:** `cargo test -p agent && npm test`
- **After wave 2:** `npx tsc --noEmit`
- **After wave 4:** Full E2E agent + full-ui
- **Before verify-work:** Full suite green

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Test Type | Automated Command | Status |
|---------|------|------|-------------|------------|-----------|-------------------|--------|
| 05-01-01 | 01 | 1 | AGT-01 | T-05-01 | unit | `cargo test -p agent tool_parse` | ⬜ pending |
| 05-01-02 | 01 | 1 | AGT-02 | T-05-01 | unit | `cargo test -p agent` | ⬜ pending |
| 05-02-01 | 02 | 2 | AGT-01/03 | — | integration | `cargo test -p agent agent_uses_search_tool` | ⬜ pending |
| 05-02-02 | 02 | 2 | AGT-02 | T-05-02 | unit | `npm test && npx tsc --noEmit` | ⬜ pending |
| 05-03-01 | 03 | 3 | SHELL-04/05 | T-05-B1 | compile | `cargo build --manifest-path src-tauri/Cargo.toml` | ⬜ pending |
| 05-04-01 | 04 | 4 | AGT-03 | — | unit | `cargo test -p agent -p llm && npm test` | ⬜ pending |
| 05-04-02 | 04 | 4 | AGT-03 | T-05-D1 | e2e | `npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts` | ⬜ pending |

---

## Wave 0 Requirements

- [ ] `crates/agent/src/tool_parse.rs` — parser unit tests
- [ ] ChatView `agent-tool-parse-warnings` testid (Plan 02)

---

## Manual-Only Verifications

| Behavior | Why Manual | Instructions |
|----------|------------|--------------|
| Cloud OpenAI json_object tool replies | CI uses Mock | Dev: set cloud provider, ask agent question, confirm JSON parse works |
| XML fallback deprecation log | No E2E for stderr | Dev: force XML-only mock reply, check stderr warning |

---

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] E2E spec exists for agent journey
- [x] Sampling continuity maintained

---

*Phase 5 validation — 2026-06-27*
