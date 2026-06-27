---
phase: 5
slug: agent-protocol
status: ready
nyquist_compliant: true
wave_0_complete: false
replanned: 2026-06-27
created: 2026-06-27
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

## Requirement → Test Map

| Req ID | Behavior | Test Type | Automated Command | Plan |
|--------|----------|-----------|-------------------|------|
| AGT-01 | Bare JSON line → Found | unit | `cargo test -p agent tool_parse` | 05-01 |
| AGT-01 | Fence / multi-object / bad schema → Failed | unit | `cargo test -p agent tool_parse` | 05-01 |
| AGT-01 | XML fallback + stderr deprecation | unit | `cargo test -p agent xml` or tool_parse composite tests | 05-01 |
| AGT-01 | Composite JSON before XML | unit | `cargo test -p agent composite` | 05-01 |
| AGT-01 | JSON-only build_tools_prompt | unit/review | `rg -v '^#' crates/agent/src/tools.rs \| rg -c '<tool_call>'` expect 0 in examples | 05-02 |
| AGT-01 | run_agent uses JSON Mock primary | integration | `cargo test -p agent agent_uses_search_tool` | 05-02 |
| AGT-02 | Failed stops loop; warnings on AgentResponse | unit/integration | `cargo test -p agent parse_failed` | 05-01 |
| AGT-02 | Pipeline merges `[name]:` warnings | integration | `cargo test -p agent orchestrate` | 05-02 |
| AGT-02 | AskResponse.tool_parse_warnings IPC | compile/unit | `cargo test -p rag` | 05-02 |
| AGT-02 | UI panel agent-tool-parse-warnings | unit + E2E | `npm test`; optional E2E assert | 05-04 |
| AGT-03 | Mock emits JSON not XML | unit | `cargo test -p llm` | 05-02 |
| AGT-03 | agent.spec.ts journey green | E2E | `npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts` | 05-04 |
| AGT-03 | full-ui agent sections green | E2E | `npm run test:e2e:local -- --spec e2e/specs/full-ui.spec.ts` | 05-04 |
| SHELL-05 | commands/config.rs complete | review/compile | `cargo build --manifest-path src-tauri/Cargo.toml` | 05-03 |
| SHELL-05 | Agent commands in commands/agent.rs | compile | `rg "ask_agent_in_session" src-tauri/src/commands/agent.rs` | 05-03 |
| SHELL-05 | e2e.rs bootstrap unchanged | review | `git diff src-tauri/src/e2e.rs` empty | 05-03 |

---

## Sampling Rate

- **After Plan 05-01 tasks:** `cargo test -p agent`
- **After Plan 05-02 tasks:** `cargo test -p agent -p llm && npm test`
- **After Plan 05-03 tasks:** `cargo build --manifest-path src-tauri/Cargo.toml`
- **After Plan 05-04 tasks:** Full E2E agent + full-ui
- **Before verify-work:** Full suite command above

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Test Type | Automated Command | Status |
|---------|------|------|-------------|------------|-----------|-------------------|--------|
| 05-01-01 | 01 | 1 | AGT-01, AGT-02 | T-05-01 | unit | `cargo test -p agent tool_parse` | ⬜ pending |
| 05-01-02 | 01 | 1 | AGT-02 | T-05-01, T-05-02 | unit/integration | `cargo test -p agent` | ⬜ pending |
| 05-02-01 | 02 | 2 | AGT-01, AGT-03 | T-05-03 | integration | `cargo test -p agent agent_uses_search_tool && cargo test -p llm` | ⬜ pending |
| 05-02-02 | 02 | 2 | AGT-02 | T-05-04 | unit/integration | `cargo test -p agent orchestrate && cargo test -p rag` | ⬜ pending |
| 05-03-01 | 03 | 3 | SHELL-05 | T-05-B1 | compile | `cargo build --manifest-path src-tauri/Cargo.toml` | ⬜ pending |
| 05-03-02 | 03 | 3 | SHELL-05 | T-05-B2 | review/compile | `cargo build --manifest-path src-tauri/Cargo.toml` | ⬜ pending |
| 05-04-01 | 04 | 4 | AGT-02 | T-05-D1 | unit | `npm test && npx tsc --noEmit` | ⬜ pending |
| 05-04-02 | 04 | 4 | AGT-03 | T-05-D2 | e2e | `npm run test:e2e:local -- --spec e2e/specs/agent.spec.ts` | ⬜ pending |

---

## Wave 0 Gaps (resolved by plan execution)

- [ ] `crates/agent/src/tool_parse.rs` — parser unit tests (Plan 05-01)
- [ ] `AgentResponse.tool_parse_warnings` + Failed stop loop (Plan 05-01)
- [ ] Mock JSON + JSON-only prompt (Plan 05-02)
- [ ] AskResponse + orchestration merge (Plan 05-02)
- [ ] `src-tauri/src/commands/agent.rs` (Plan 05-03)
- [ ] ChatView `agent-tool-parse-warnings` (Plan 05-04)

---

## Manual-Only Verifications

| Behavior | Why Manual | Instructions |
|----------|------------|--------------|
| Cloud OpenAI bare-line compliance | CI uses Mock | Dev: cloud provider agent ask; confirm no fence warnings on well-formed JSON line |
| XML fallback stderr | No dedicated E2E for stderr (D-17) | Dev: force XML-only reply in unit test or custom mock; confirm `[jarvis] XML tool_call format deprecated; use JSON line` on stderr |
| Orchestration warnings placement | Visual | Dev: pipeline mode with parse failure in step; warnings only on final message, prefixed `[profile name]:` |

---

## Context Decision Coverage (D-01..D-28)

| Decision | Plan | Verification |
|----------|------|--------------|
| D-01..D-04 strict JSON | 05-01 | tool_parse unit tests |
| D-05 JSON-only prompt | 05-02 | rg + agent_uses_search_tool |
| D-06..D-10 parser architecture | 05-01 | tool_parse + composite tests |
| D-11..D-12 stop loop + raw answer | 05-01 | parse_failed test |
| D-13..D-14 UI + IPC metadata | 05-02, 05-04 | rag + ChatView tests |
| D-15..D-17 XML fallback one release | 05-01 | xml/composite tests; no XML E2E |
| D-18..D-19 Mock JSON + E2E mock | 05-02, 05-04 | llm + agent E2E |
| D-20..D-22 orchestration merge | 05-02 | orchestrate test |
| D-23..D-25 agent command split | 05-03 | build + rg |
| D-26..D-28 quality gates + testids | 05-04 | full suite |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] E2E spec exists for agent journey
- [x] Every phase requirement ID mapped (AGT-01, AGT-02, AGT-03, SHELL-05)
- [x] Sampling continuity maintained across 4 waves
- [ ] Execution complete — flip task Status to ✅ after `/gsd-execute-phase 5`

---

*Phase 5 validation — replanned 2026-06-27*
