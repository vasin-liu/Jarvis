# Phase 2: Chat Extraction + Config Types - Context

**Gathered:** 2026-06-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Extract the highest-traffic **Chat view** and its RAG session logic from `App.tsx`, complete the **chat Tauri command module**, and introduce **nested `AppConfig` structs** with backward-compatible flat JSON — **zero user-visible behavior regression** except intentional RAG/agent submit control separation (with E2E updates in same PR).

**In scope:**
- `src/views/ChatView.tsx` owns chat layout JSX (sessions sidebar, messages, input areas, index banner, error bar, agent UI chrome)
- `src/hooks/useChat.ts` owns RAG session state + IPC (via `src/lib/tauri.ts`)
- `src/types/chat.ts` for chat-specific TS types
- `src-tauri/src/commands/chat.rs` — research-mapped chat commands + RAG helpers
- `crates/config/src/types.rs` — five nested config structs, flat JSON on disk
- Config unit test: flat JSON fixture roundtrip
- E2E: full suite green; `qa.spec.ts`, `agent.spec.ts`, `full-ui.spec.ts` updated for split submit controls
- `cargo test --workspace` + `npm test` (QA-03)

**Out of scope (later phases):**
- `useAgent` hook / agent invoke migration (Phase 5)
- Streaming Channel logic migration from App (Phase 5)
- `commands/agent.rs`, agent protocol, structured tool calls (Phase 5)
- OS keychain for `cloud_api_key` (Phase 3)
- Main nav Sidebar / AppShell extraction (Phase 6)
- Lark continuous auto-sync implementation (Phase 3 — captured as requirement below)
- Library/Tasks/Memory/Settings view extraction (Phases 3–6)

</domain>

<decisions>
## Implementation Decisions

### Hook State Boundary (`useChat`)
- **D-01:** `useChat` owns **RAG session state only**: `sessions`, `activeSessionId`, `messages`, `question`, `streamingDraft`. Agent-related state stays in `App.tsx`.
- **D-02:** Global `busy` / `err` remain in `App.tsx`; `useChat` reports via callbacks (shared error bar pattern).
- **D-03:** E2E probe state (`e2eMode`, `askHandleCount`, `chat-session-ready`) stays in `App.tsx`; passed as props to `ChatView`.
- **D-04:** **Controlled component** pattern: App passes `busy`, `err`, `setErr`, and all agent props; `ChatView`/`useChat` do not own global app state.
- **D-05:** `indexProgress` listen + state stays in `App.tsx`; passed to `ChatView` as props.
- **D-06:** Streaming `Channel` subscription logic **stays in App.tsx until Phase 5**; `src/lib/tauri.ts` still wraps `ask_in_session_stream` IPC.
- **D-07:** Session list refresh is **autonomous in `useChat`** (hook loads/refreshes sessions on create/delete/switch).
- **D-08:** Phase 2 `tauri.ts` migration covers **all chat IPC paths** including stream wrapper functions.

### Agent UI in ChatView
- **D-09:** Agent UI JSX moves into `ChatView.tsx`; state and handlers remain App-controlled via **inline explicit props**.
- **D-10:** **Separate RAG vs Agent submit controls** — not a single shared submit path. Visually distinct input/submit areas.
- **D-11:** Preserve existing `data-testid` values where applicable; **add `agent-submit`** (and related agent-specific ids). Update `agent.spec.ts` in same PR.
- **D-12:** Tool-call panel (`agent-tool-calls`) and orchestration steps stay **below messages** (current layout).
- **D-13:** Add **「管理 Agent」** quick link in ChatView header → navigates to Settings view.
- **D-14:** `agent-mode-toggle` and `agent-profile-select` live in **Chat header bar** (same row as session controls).
- **D-15:** `ask_agent_in_session` / `ask_agent_in_session_stream` invoke calls **stay in App until Phase 5** (aligns with Phase 1 D-07 agent glue deferral).

### Tauri `commands/chat.rs`
- **D-16:** Extract **research-mapped chat commands only** (8): `list_chat_sessions`, `create_chat_session`, `delete_chat_session`, `list_chat_messages`, `ask_in_session`, `ask_in_session_stream`, `run_ask_in_session` — **exclude all `ask_agent_*`**.
- **D-17:** Move RAG private helpers with commands: `run_ask_in_session`, persist assistant, `auto_learn_from_chat` trigger logic → `chat.rs`.
- **D-18:** Access store/embedder/chat model via `State<AppState>` — no new service module in Phase 2.
- **D-19:** `start_ask_e2e` **stays in `e2e.rs` / `lib.rs`**, not `commands/chat.rs`.

### Config Nesting (CFG-01)
- **D-20:** Five nested Rust structs: `EmbeddingConfig`, `ChatConfig`, `LarkConfig`, `SyncConfig`, `AgentConfig`.
- **D-21:** **Flat JSON on disk** via `#[serde(flatten)]` — existing `config.json` key names unchanged (no migration file).
- **D-22:** `SyncConfig` bucket includes: `watch_folders`, `cursor_projects_root`, all `scheduled_sync_*`, `auto_summarize_on_index`, `auto_extract_tasks_on_index`.
- **D-23:** Frontend `src/types/ipc.ts` `AppConfig` **stays flat** — TS does not mirror Rust nesting in Phase 2.
- **D-24:** Add **config crate unit test**: deserialize flat JSON fixture → serialize → key-level equality roundtrip.

### Shared Chrome & Layout
- **D-25:** `IndexProgressBanner` and `chat-error` move **inside `ChatView`** (testids unchanged).
- **D-26:** Chat session sidebar (new/switch/delete) stays **inside `ChatView`**.
- **D-27:** Main nav Sidebar (`nav-*`) **unchanged in Phase 2** — defer AppShell to Phase 6 (Phase 1 D-10).
- **D-28:** Citation parse/render moves with `ChatView`; continues using `src/lib/citations.ts` pure functions.

### File Layout & PR Strategy
- **D-29:** **Single PR** for Phase 2: FE extraction + `chat.rs` + config nesting land together; full E2E gate once.
- **D-30:** `useChat.ts` at `src/hooks/useChat.ts` (standard hooks directory).
- **D-31:** **Monolithic `ChatView.tsx`** for Phase 2 — no `SessionSidebar`/`MessageList` subcomponent split.
- **D-32:** Chat types in `src/types/chat.ts` (`ChatSession`, `ChatMessage`, etc.); `ipc.ts` may re-export.

### Quality Gates
- **D-33:** **Full E2E suite** (`e2e/specs/**/*.spec.ts`) must pass before merge — not just qa/agent/full-ui.
- **D-34:** `cargo test --workspace` + `npm test` required (QA-03).
- **D-35:** Agent E2E updated in same PR for split submit controls (no hidden compat shim elements).

### Phase 3 Prerequisite (User Vision — Lark/Local Auto Sync)
- **D-36:** [informational] Record for Phase 3 planning: Feishu/Lark docs, chat, mail, tasks, and local documents should sync via **`lark-cli` continuous/scheduled fetch** — **manual URL/token copy should not be the primary path**. Phase 3 CONTEXT should treat this as a requirement, not stretch. No Phase 2 implementation.

### Claude's Discretion
- Exact nested struct field assignments within the five config buckets (as long as flatten preserves flat JSON keys).
- Minor ChatView internal organization (single file) as long as testids and controlled props contract hold.
- Whether `ipc.ts` re-exports from `types/chat.ts` vs direct imports.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 2 goal, success criteria, key files
- `.planning/REQUIREMENTS.md` — FE-02, FE-06, SHELL-02, CFG-01 traceability
- `.planning/PROJECT.md` — Core value, constraints, refactor framing
- `.planning/phases/01-scaffold-shell-foundation/01-CONTEXT.md` — Phase 1 locked decisions (D-07 agent glue deferral, D-14 tauri.ts migration start)

### Target architecture
- `.planning/research/ARCHITECTURE.md` — Command grouping map (chat vs agent), target directory layout, S1 config nesting
- `.planning/research/SUMMARY.md` — Phase 2 rationale (highest-traffic path)
- `.planning/research/PITFALLS.md` — §4 testid stability, §9 App state coupling, §10 config serde flatten risks, §13 agent protocol

### Codebase context
- `.planning/codebase/ARCHITECTURE.md` — Layer model, chat/RAG data flow
- `.planning/codebase/STRUCTURE.md` — Where to add views/hooks/commands
- `.planning/codebase/CONCERNS.md` — App.tsx / lib.rs monolith debt
- `.planning/codebase/TESTING.md` — E2E spec map (qa, agent, full-ui)

### Engineering policy
- `AGENTS.md` — TDD, E2E mandatory, file boundaries
- `.cursor/rules/e2e-required.mdc` — E2E mandatory for user-facing changes
- `.cursor/rules/tdd-goal-driven.mdc` — Red-green-refactor workflow

### Implementation targets (current state)
- `src/App.tsx` — Monolith; ~15 chat-related useState; ChatView currently passthrough wrapper
- `src/views/ChatView.tsx` — Stub `{children}` wrapper; target for full chat JSX
- `src/lib/tauri.ts` — Partial wrappers (config/library/sync); extend for chat
- `src-tauri/src/lib.rs` — Chat commands still inline (~8 commands)
- `src-tauri/src/commands/mod.rs` — config, library, sync extracted (Phase 1)
- `crates/config/src/types.rs` — Flat `AppConfig` (~30+ fields); target for nesting

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/views/ChatView.tsx` — Stub exists; replace passthrough with full layout
- `src/lib/tauri.ts` — Pattern for typed `invoke` wrappers (Phase 1 config/library/sync)
- `src/lib/citations.ts` — Pure citation parsing; call from ChatView message rendering
- `src/types/ipc.ts` — IPC types extracted in Phase 1; chat types fork to `types/chat.ts`
- `src-tauri/src/commands/{config,library,sync}.rs` — Command module pattern to follow for `chat.rs`
- `src-tauri/src/state.rs` — `AppState` home for provider/store access from commands

### Established Patterns
- Tauri commands return `Result<T, String>`; thin wrappers delegate to domain crates
- `snake_case` IPC names — no renames (frontend stability)
- Controlled view extraction: Phase 1 stubs → Phase 2+ logic migration
- E2E deterministic mocks: `JARVIS_E2E=1` + `data-testid` selectors (`chat-input`, `agent-mode-toggle`, etc.)

### Integration Points
- `App.tsx` view router: `{view === "chat" && <ChatView ...props />}` — App retains agent state + streaming Channel
- `lib.rs::generate_handler!` — register `commands::chat::*` alongside existing modules
- `config::AppConfig` — used by `build_embedder` / `build_chat_model` in providers.rs; nesting must not break factories
- E2E specs: `e2e/specs/qa.spec.ts`, `agent.spec.ts`, `full-ui.spec.ts` — primary chat journey verification

</code_context>

<specifics>
## Specific Ideas

- User prefers **中文** for discuss/plan communication; technical artifacts stay English.
- RAG and Agent should feel like **distinct modes** in the UI (separate submit controls), not one ambiguous form.
- **「管理 Agent」** shortcut in chat header for power-user workflow.
- **飞书/本地自动同步愿景**: Lark docs, IM, mail, tasks, and local files should be kept current via `lark-cli` + scheduled sync — not manual copy-paste of URLs/tokens. Must be captured as Phase 3 requirement (D-36).
- Phase 1 carry-forward: agent glue (`build_agent_context`, `execute_agent_question`) stays in `lib.rs` until Phase 5.

</specifics>

<deferred>
## Deferred Ideas

### Phase 3 — Lark/Local Continuous Sync (User Requirement D-36)
- Primary sync path: `lark-cli` scheduled/discovery-based fetch for docs, sheets, mail, IM, tasks
- Manual token/URL inputs demoted to advanced/debug fallback, not primary UX
- Local watch folders already exist; ensure Phase 3 Library view aligns with "always current" expectation

### Other Phases
- `useAgent` hook + agent invoke migration → Phase 5
- Streaming Channel logic in hook → Phase 5
- Main nav AppShell / Sidebar component → Phase 6
- OS keychain for API secrets → Phase 3
- Nested TS `AppConfig` mirroring Rust → Phase 6 Settings extraction (optional)

</deferred>

---

*Phase: 2-Chat Extraction + Config Types*
*Context gathered: 2026-06-25*
