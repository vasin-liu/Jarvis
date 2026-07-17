---
phase: 5
slug: agent-protocol
status: ready
nyquist_compliant: true
wave_0_complete: true
replanned: 2026-06-27
created: 2026-06-27
updated: 2026-06-29
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.  
> Replanned 2026-06-27 to align with strict bare-line JSON (D-01..D-04), stop-on-Failed loop (D-11), orchestration warning merge (D-20..D-22), and commands/agent.rs relocation (D-23).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (agent/llm/rag) + Vitest 3 + WebdriverIO 9 |
| **Quick run command** | `cargo test -p agent && npm test` |
| **Full suite command** | `cargo test --workspace && npm test && npx tsc --noEmit && npm run build:e2e && npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts e2e/specs/full-ui.spec.ts` |
| **Estimated runtime** | ~180 seconds |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 05-01-01 | 01 | 1 | AGT-01, AGT-02 | unit | `cargo test -p agent tool_parse` | ✅ |
| 05-01-02 | 01 | 1 | AGT-02 | unit/integration | `cargo test -p agent` | ✅ |
| 05-02-01 | 02 | 2 | AGT-01, AGT-03 | integration | `cargo test -p agent agent_uses_search_tool && cargo test -p llm` | ✅ |
| 05-02-02 | 02 | 2 | AGT-02 | unit/integration | `cargo test -p agent orchestrate && cargo test -p rag` | ✅ |
| 05-03-01 | 03 | 3 | SHELL-05 | compile | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ |
| 05-03-02 | 03 | 3 | SHELL-05 | review/compile | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ |
| 05-04-01 | 04 | 4 | AGT-02 | unit | `npm test && npx tsc --noEmit` | ✅ |
| 05-04-02 | 04 | 4 | AGT-03 | e2e | `npx wdio run e2e/wdio.conf.ts --spec e2e/specs/agent.spec.ts` | ✅ |

---

## Wave 0 Gaps (resolved by plan execution)

- [x] `crates/agent/src/tool_parse.rs` — parser unit tests (Plan 05-01)
- [x] `AgentResponse.tool_parse_warnings` + Failed stop loop (Plan 05-01)
- [x] Mock JSON + JSON-only prompt (Plan 05-02)
- [x] AskResponse + orchestration merge (Plan 05-02)
- [x] `src-tauri/src/commands/agent.rs` (Plan 05-03)
- [x] ChatView `agent-tool-parse-warnings` (Plan 05-04)

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] E2E spec exists for agent journey
- [x] Every phase requirement ID mapped (AGT-01, AGT-02, AGT-03, SHELL-05)
- [x] Sampling continuity maintained across 4 waves
- [x] E2E gate green — `agent.spec.ts` + `full-ui.spec.ts`

---

*Phase 5 validation — updated 2026-06-29 after execute-phase*
