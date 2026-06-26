# Phase 2: Chat Extraction + Config Types - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-25
**Phase:** 2-Chat Extraction + Config Types
**Areas discussed:** Hook boundary, Agent UI, commands/chat.rs scope, Config nesting, Shared chrome, E2E scope, PR slicing, File layout, Types split, Lark auto-sync vision

---

## Hook State Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| RAG session only | useChat owns sessions/messages/question/streaming; agent state in App | ✓ |
| RAG + Agent UI state | All chat/agent state in useChat | |
| You decide | Minimal diff + E2E stability | |

**Additional decisions:**
- busy/err: App global with callbacks ✓
- E2E probes: stay in App ✓
- Interface: controlled component (App passes props) ✓
- indexProgress: App owns listen ✓
- Streaming Channel: App until Phase 5; tauri.ts wraps all chat IPC including stream ✓
- Session refresh: autonomous in useChat ✓

---

## Agent UI in ChatView

| Option | Description | Selected |
|--------|-------------|----------|
| JSX in ChatView, props from App | Agent UI moves to ChatView; inline explicit props | ✓ |
| JSX stays in App | Minimal diff passthrough | |
| Separate submit controls | RAG and Agent distinct submit areas | ✓ |
| Shared form | Single chat-form path (status quo) | |

**Additional decisions:**
- testids: keep existing + add agent-submit; update E2E same PR ✓
- tool-call panel: below messages ✓
- profile CRUD: Settings primary + Chat「管理 Agent」link ✓
- toggles: Chat header bar ✓
- ask_agent invoke: migrate Phase 5 ✓

---

## commands/chat.rs Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Research map (8 commands) | Session CRUD + ask paths; no ask_agent_* | ✓ |
| Include agent commands | ask_agent_* in chat.rs | |
| Helpers move with commands | run_ask_in_session, persist, auto_learn → chat.rs | ✓ |
| start_ask_e2e in e2e.rs | E2E command not in chat.rs | ✓ |

---

## Config Nesting

| Option | Description | Selected |
|--------|-------------|----------|
| Five structs | Embedding + Chat + Lark + Sync + Agent | ✓ |
| Three structs (REQ) | Embedding + Sync + Agent only | |
| Flat JSON on disk | serde flatten; keys unchanged | ✓ |
| Nested JSON | Requires migration | |
| TS stays flat | ipc.ts unchanged shape | ✓ |
| SyncConfig bucket | cursor, watch_folders, scheduled_sync, auto_insights | ✓ |

---

## Shared Chrome

| Option | Description | Selected |
|--------|-------------|----------|
| Banner/error in ChatView | IndexProgressBanner + chat-error inside ChatView | ✓ |
| Session sidebar in ChatView | New/switch/delete sessions in chat layout | ✓ |
| Nav Sidebar defer Phase 6 | No AppShell extraction this phase | ✓ |
| Citations with ChatView | Uses lib/citations.ts | ✓ |

---

## E2E Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full suite | All e2e/specs/**/*.spec.ts | ✓ |
| Phase minimum | qa + agent + full-ui only | |
| cargo test --workspace + npm test | QA-03 gate | ✓ |
| Config unit roundtrip test | Flat JSON fixture in config crate | ✓ |
| Update agent.spec same PR | No hidden compat shim | ✓ |

---

## PR Slicing & File Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single PR | FE + chat.rs + config together | ✓ |
| useChat at src/hooks/useChat.ts | Standard hooks dir | ✓ |
| Monolithic ChatView | No subcomponent split Phase 2 | ✓ |
| types/chat.ts | Chat types extracted from ipc.ts | ✓ |

---

## Lark Auto-Sync Vision (Deferred to Phase 3)

**User freeform (中文):** 飞书文档、聊天、邮件、任务及本地文档不应需手动 copy 地址同步；应通过 lark-cli 持续获取。

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 3 requirement | Record in Phase 3 CONTEXT as primary sync path | ✓ |
| Roadmap backlog only | Stretch goal | |

---

## Claude's Discretion

- Nested struct field assignments within five buckets (flatten keys preserved)
- ChatView internal code organization (single file)
- ipc.ts re-export pattern for chat types

## Deferred Ideas

- Lark/local continuous auto-sync → Phase 3 (D-36)
- useAgent hook, streaming in hook, ask_agent invoke → Phase 5
- AppShell / main nav → Phase 6
- Keychain secrets → Phase 3
