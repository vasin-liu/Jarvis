---
phase: 05-agent-protocol
status: passed
verified: 2026-07-17
score: 12/12
---

# Phase 05 Verification Report

**Phase goal:** As a Jarvis user, I want to use agent mode with structured JSON tool calls and see parse warnings in chat, so that I can trust agent answers when tool parsing fails.

## Must-Have Verification

| ID | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| AGT-01 | JSON line tool call protocol | PASS | `tool_parse.rs` JsonToolCallParser + Mock JSON line |
| AGT-02 | Parse warnings surfaced in UI | PASS | `AgentResponse.tool_parse_warnings` → `AskResponse` → `agent-tool-parse-warnings` panel; `ChatView.test.tsx` |
| AGT-03 | E2E agent journey green with JSON Mock | PASS | `agent.spec.ts` 3/3; `full-ui.spec.ts` agent section |
| SHELL-05 | Agent IPC in `commands/agent.rs` | PASS | Module created; `lib.rs` thin; `start_ask_e2e` retained |

## Human UAT (2026-07-17)

- [x] `05-UAT.md` — 7/7 passed (0 issues). Parse-warning checks (tests 4–5) accepted via automated evidence after healthy-path manual skip.
- [x] Mid-UAT launch hang fixed: `DeferredEmbedder` + deferred initial scan (product fix outside original phase plans; verified by UAT retest).

## Automated Checks

- [x] `cargo test -p agent -p llm -p rag` — 31+ passed
- [x] `cargo build --manifest-path src-tauri/Cargo.toml` — release build OK
- [x] `npm test` — Vitest green (incl. `ChatView.test.tsx`)
- [x] `npx tsc --noEmit` — clean
- [x] `agent.spec.ts` — 3 passing (agent ask + Chinese regression + orchestration UI)
- [x] `full-ui.spec.ts` — 7 passing (includes agent mode journey)

## E2E Fix (gap closure)

`submitAgentForm` raced on `ask-busy` because fast E2E agent asks completed before WebDriver polled. Fixed by incrementing `askHandleCount` in `handleAgentAsk` (matching `handleRagAsk`) and checking counter in `submitAgentForm`.

## Gaps

None.
