# Architecture Research — v1.9.x Incremental Refactor

> **Date:** 2026-06-17  
> **Scope:** Structural refactor of Jarvis v1.8.0 brownfield  
> **Inputs:** `.planning/PROJECT.md`, `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`, `.planning/codebase/CONCERNS.md`  
> **Question:** How should refactored Jarvis be structured? Component boundaries, migration order, data flow preservation, balanced phasing.

---

## Executive Summary

Jarvis already has a sound **layered architecture**: React presentation → Tauri IPC shell → domain crates → `store` (sole SQLite owner). The v1.9.x refactor does **not** change the stack or the core data flows (index → store, retrieve → RAG, agent tool loop). It **decomposes two monoliths** (`src/App.tsx` ~2,800 lines, `src-tauri/src/lib.rs` ~1,300 lines + 52 commands) and hardens three cross-cutting concerns (config/secrets, agent protocol, memory identity).

**Strategy:** Vertical migration slices that touch frontend + shell + (when needed) domain crate in the same phase, with E2E green after every merge. **Balanced phasing** interleaves layers so no single area is frozen for long.

**Non-goals:** New workspace crates unless a boundary is proven insufficient; big-bang rewrite; v2.0 breaking changes; replacing SQLite/Tauri/React.

---

## Preserved Invariants (Do Not Break)

These constraints survive every phase. Violating them is a refactor failure, not a style preference.

| Invariant | Rationale |
|-----------|-----------|
| **Single DB owner** (`crates/store` only opens SQLite) | Lock/schema authority; no connection races |
| **Normalize-to-Document** | All sources → `ingest::Document` + `SourceKind` → `indexer::index_document` |
| **Trait-based providers** | `Embedder`, `ChatModel`, `CommandRunner` — mocks in unit/E2E |
| **Thin shell, fat crates** | Tauri commands validate + delegate; business logic stays testable without Tauri |
| **IPC name stability** | `snake_case` command names unchanged unless explicit migration + E2E update |
| **E2E gate** | `npm run test:e2e:local` green after every phase |
| **Existing user data** | `config.json` and `jarvis.db` migrate without data loss |
| **Composition root** | `src-tauri` wires providers, watcher, scheduler; crates do not depend on Tauri |

---

## Target Architecture

### Layer Model (unchanged semantics, clearer file boundaries)

```
┌─────────────────────────────────────────────────────────────┐
│  Presentation   src/                                        │
│  App shell + views + hooks + lib/tauri.ts                   │
├─────────────────────────────────────────────────────────────┤
│  IPC / Shell    src-tauri/                                  │
│  state.rs + commands/* + index_ops + sync_scheduler + e2e   │
├─────────────────────────────────────────────────────────────┤
│  Application    index_ops, insights_ops, sync_scheduler     │
│  (orchestration glue — may absorb thin command logic)       │
├─────────────────────────────────────────────────────────────┤
│  Domain         crates: rag, agent, indexer, retriever, …   │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure config (+ secrets), embedder, llm, lark, …    │
├─────────────────────────────────────────────────────────────┤
│  Persistence    crates/store                                │
└─────────────────────────────────────────────────────────────┘
```

**Dependency rule (unchanged):** Domain → `store` + traits. Domain ↛ Tauri/React. `src-tauri` is the only upward dependency from crates to the desktop shell.

---

## Target Directory Layout

### Frontend (`src/`)

Current: single `App.tsx` owns types, layout, five views, ~30 `useState`, IPC, event listeners.

**Target:**

```
src/
├── main.tsx
├── index.css
├── App.tsx                      # Shell only: layout, nav, view router, global banners
├── types/
│   └── ipc.ts                   # ChatSession, Source, AppConfig, … (mirror Rust serde shapes)
├── lib/
│   ├── tauri.ts                 # Typed invoke wrappers (one fn per command group)
│   ├── citations.ts
│   ├── sourceDisplay.ts
│   └── events.ts                # listen("index-progress", …) setup helpers
├── hooks/
│   ├── useJarvisConfig.ts       # get/set config, refresh, save side effects
│   ├── useChat.ts               # sessions, messages, ask/stream, agent mode
│   ├── useLibrary.ts            # sources, index status, lark/cursor sync
│   ├── useTasks.ts
│   ├── useMemory.ts
│   └── useIndexProgress.ts      # shared progress banner state
├── components/
│   ├── layout/
│   │   ├── Sidebar.tsx
│   │   ├── IndexProgressBanner.tsx
│   │   └── ErrorBanner.tsx
│   └── ui/
│       ├── StatusBadge.tsx
│       └── SourceKindIcon.tsx
└── views/
    ├── ChatView.tsx
    ├── LibraryView.tsx
    ├── TasksView.tsx
    ├── MemoryView.tsx
    └── SettingsView.tsx
```

**Boundary rules:**

| Module | Owns | Must not own |
|--------|------|--------------|
| `views/*` | View-local UI state, render, `data-testid` | Cross-view global state |
| `hooks/*` | IPC calls, refresh logic, derived state for one domain | JSX layout |
| `lib/tauri.ts` | `invoke` typing, error mapping | React hooks |
| `App.tsx` | `view` route, mount hooks, compose layout | Feature-specific forms |

**Size targets:** `App.tsx` < 200 lines; each view < 400 lines; hooks < 250 lines.

---

### Tauri shell (`src-tauri/src/`)

Current: `lib.rs` = `AppState` + 52 `#[tauri::command]` + `run()` + seeds + agent/RAG/Lark/memory glue.

**Target:**

```
src-tauri/src/
├── main.rs
├── lib.rs                       # mod declarations, run(), command registration only
├── state.rs                     # AppState, impl helpers (config, reload_providers, watcher)
├── commands/
│   mod.rs                       # re-exports all command fns for generate_handler!
│   config.rs                    # get_config, set_config, get_index_status
│   index.rs                     # rebuild, retry, watch folders, cursor sync
│   library.rs                   # list_sources, remove_source, source_count
│   chat.rs                      # sessions, ask_in_session(_stream), persist_assistant
│   agent.rs                     # profiles, skills/hooks/plugins, ask_agent_*
│   lark.rs                      # check_lark, index_lark, sync_lark_*
│   insights.rs                  # summarize, extract_tasks, run_insights_all
│   tasks.rs                     # list/update/delete tasks
│   memory.rs                    # list/add/update/forget memory
│   sync.rs                      # get_sync_status, run_scheduled_sync_cmd
│   e2e.rs                       # (existing) is_e2e_mode_cmd, fixtures
├── index_ops.rs                 # (existing) rebuild/sync orchestration
├── insights_ops.rs              # (existing)
├── sync_scheduler.rs            # (existing)
└── e2e.rs                       # (existing module file — keep or fold into commands/e2e)
```

**Boundary rules:**

| Module | Owns | Delegates to |
|--------|------|--------------|
| `commands/*` | Input validation, `State<AppState>`, spawn async, emit events | `index_ops`, domain crates |
| `state.rs` | `AppState` struct, watcher/scheduler lifecycle, provider reload | `config::build_*` |
| `lib.rs` | `tauri::Builder`, `setup`, `generate_handler![…]` | — |
| `index_ops.rs` | Multi-step index/rebuild/sync + progress events | `indexer`, `lark`, `cursor` |

**Command grouping map (from current `lib.rs`):**

| `commands/` module | Commands (representative) |
|--------------------|---------------------------|
| `config` | `get_config`, `set_config`, `get_index_status` |
| `index` | `rebuild_index`, `reinit_and_rebuild_index`, `retry_source`, `add_watch_folder`, `remove_watch_folder`, `list_cursor_transcripts`, `sync_cursor_transcripts_cmd` |
| `library` | `source_count`, `list_sources`, `remove_source` |
| `chat` | `list_chat_sessions`, `create_chat_session`, `delete_chat_session`, `list_chat_messages`, `ask_in_session`, `ask_in_session_stream`, `run_ask_in_session`, `start_ask_e2e` |
| `agent` | `list_agent_profiles`, `upsert_agent_profile`, `set_active_agent`, `remove_agent_profile`, `list_skills`, `list_hooks`, `list_plugins`, `ask_agent_in_session`, `ask_agent_in_session_stream` |
| `lark` | `check_lark_connection`, `detect_lark_cli`, `index_lark`, `sync_lark_doc`, `sync_lark_url` |
| `insights` | `summarize_source_cmd`, `extract_tasks_cmd`, `run_insights_all_cmd` |
| `tasks` | `list_tasks`, `update_task_status`, `delete_task` |
| `memory` | `list_memories_cmd`, `get_memory_content_cmd`, `add_memory_cmd`, `update_memory_cmd`, `forget_memory_cmd` |
| `sync` | `get_sync_status`, `run_scheduled_sync_cmd` |

**Size targets:** `lib.rs` < 150 lines; each `commands/*.rs` < 200 lines; shared helpers (`build_agent_context`, `agent_response_to_ask`) → `commands/agent.rs` or `state.rs`.

---

### Domain crates (`crates/`)

**No new crates required for v1.9.x** unless a refactor slice proves a boundary leak. Target changes are **internal** to existing crates:

| Crate | Refactor touch | Notes |
|-------|----------------|-------|
| `config` | Nested `AppConfig`, secrets split | `EmbeddingConfig`, `ChatConfig`, `SyncConfig`, `AgentConfig`; `SecretsStore` trait |
| `agent` | Structured tool protocol | Replace XML+JSON parse; keep `execute_tool` registry |
| `memory` | Strict URI identity | `memory://{uuid}`; deprecate fuzzy title `resolve_memory_id` |
| `store` | Optional `memories` meta or table | Only if memory slice needs dedicated storage; still sole DB owner |
| Others | **Stable** | `ingest`, `indexer`, `retriever`, `rag`, `lark`, `cursor`, `watcher`, `insights` — no structural moves |

---

## Module Boundaries (Refactor-Specific)

### What moves out of monoliths

```
App.tsx                          lib.rs
────────                         ──────
types/*           ←──────────→   (serde types stay in Rust; TS mirrors in types/ipc.ts)
view JSX            → views/*
refresh/invoke      → hooks/* + lib/tauri.ts
shared widgets      → components/*
AppState            → state.rs
#[tauri::command]   → commands/*
agent/RAG glue      → commands/chat.rs + commands/agent.rs
Lark IPC            → commands/lark.rs
seed_skills_*       → state.rs or commands/agent.rs (startup only)
```

### What stays centralized

| Concern | Location | Why |
|---------|----------|-----|
| SQLite schema/migrations | `crates/store` | Single owner |
| Index pipeline | `indexer` + `index_ops` | Already extracted |
| Hybrid retrieval | `retriever` | Pure domain |
| RAG prompt assembly | `rag` | No Tauri dependency |
| Agent tool execution | `agent::tools` | Testable without IPC |
| E2E mocks | `e2e.rs` + `JARVIS_E2E=1` | One switch for CI |

### Forbidden splits

- Do **not** open SQLite outside `store` to “fix” memory — extend `Store` API instead.
- Do **not** duplicate RAG/retrieval logic in React hooks — hooks call `invoke`, crates compute.
- Do **not** create `crates/tauri-commands` — shell stays in `src-tauri`.
- Do **not** rename IPC commands without a compatibility shim and E2E pass.

---

## Data Flow Preservation

Every migration slice must preserve these end-to-end paths. Use as phase exit checklist.

### 1. Index pipeline (unchanged path)

```
Source → ingest/lark/cursor/memory → Document
  → indexer::index_document(store, embedder, chunker, doc, SourceKind)
  → store (sources, chunks, vec_chunks, chunks_fts)
  → optional insights_ops → insights → store.tasks / source.summary
```

**Triggers preserved:** file pick, watch events, startup scan, rebuild, Lark/Cursor sync, scheduled sync, memory add/update.

**Refactor risk:** Moving commands must not drop `Emitter` progress events (`IndexProgressEvent`). `useIndexProgress` hook must still `listen` to the same event name.

### 2. Q&A / RAG (unchanged path)

```
ChatView → invoke("ask_in_session" | "ask_in_session_stream")
  → commands/chat.rs → rag::ask(_stream)
  → retriever::retrieve → store.search_vector + search_fts → RRF
  → llm::complete(_stream) → store.append_chat_message
  → optional memory::learn_from_exchange
```

**Refactor risk:** `Channel<TokenEvent>` streaming contract and `persist_assistant` timing must remain identical for E2E `streaming-answer` / `ask-busy` testids.

### 3. Agent mode (unchanged path, protocol may harden)

```
ChatView (agent mode) → invoke("ask_agent_in_session" | "_stream")
  → commands/agent.rs → agent::run_agent | run_orchestrated | run_routed
  → tool loop → execute_tool → hooks
  → AgentResponse → agent_response_to_ask → UI citations/tool_calls/orchestration_steps
```

**Refactor risk:** Agent protocol change is a **separate sub-slice** with Mock LLM fixtures; do not mix with pure file moves in the same PR.

### 4. Config / providers (evolving path)

```
SettingsView → get_config / set_config
  → state.save_config → config file (+ future keychain for secrets)
  → reload_providers → build_embedder / build_chat_model
  → may trigger reinit_and_rebuild_index on dim change
```

**Refactor risk:** Nested config structs must use `#[serde(flatten)]` or custom migrate for backward-compatible `config.json` load.

### 5. Memory (evolving path)

```
MemoryView → memory commands → memory crate → index_document(Memory)
  → list_memories filters SourceKind::Memory
```

**Refactor risk:** URI scheme change requires migration: existing memories get `memory://{id}` URIs; `resolve_memory_id` keeps title fallback one release, then remove.

---

## Migration Slices

Each slice is a **vertical MVP**: shippable, E2E-green, architecture-reviewable. Slices are ordered by dependency and risk.

### Slice dependency graph

```
                    ┌─────────────┐
                    │  S0: Harness │  (typed tauri.ts, state.rs extract — no behavior change)
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │ S1: Config │  │ S2: Shell  │  │ S3: FE nav │
    │  hardening │  │  commands  │  │  + layout  │
    └─────┬──────┘  └─────┬──────┘  └─────┬──────┘
          │               │               │
          └───────┬───────┴───────┬───────┘
                  ▼               ▼
           ┌────────────┐  ┌────────────┐
           │ S4: Chat   │  │ S5: Library│
           │  view      │  │  + tasks   │
           └─────┬──────┘  └─────┬──────┘
                 │               │
                 ▼               ▼
           ┌────────────┐  ┌────────────┐
           │ S6: Memory │  │ S7: Agent  │
           │  model     │  │  protocol  │
           └────────────┘  └────────────┘
                 │               │
                 └───────┬───────┘
                         ▼
                  ┌────────────┐
                  │ S8: Settings│
                  │  + secrets  │
                  └────────────┘
```

**Legend:** Arrows = “should complete or start in parallel after”. S0 is prerequisite for all. S1 and S2 can interleave. S4–S6 depend on S3 (views exist). S7 is independent of FE splits but touches chat commands. S8 absorbs config UI + keychain.

---

### S0 — Scaffold (no user-visible change)

**Goal:** Create target folders and extraction points without moving behavior.

| Layer | Work |
|-------|------|
| Frontend | Add `src/types/ipc.ts`, `src/lib/tauri.ts` (thin wrappers calling existing `invoke` names), empty `views/` stubs re-exporting from `App.tsx` |
| Shell | Extract `AppState` + impl to `state.rs`; `commands/mod.rs` re-exports from `lib.rs` |
| Tests | E2E smoke + navigation green |

**Exit:** File structure exists; line counts unchanged; zero regression.

---

### S1 — Config hardening (domain-first)

**Goal:** Nested `AppConfig` + secrets abstraction; plaintext key deprecated.

| Layer | Work |
|-------|------|
| `config` | Split `AppConfig` → `EmbeddingConfig`, `ChatConfig`, `LarkConfig`, `SyncConfig`, `AgentSettings`; `#[serde(flatten)]` for backward compat |
| `config` | `SecretsStore` trait; `KeyringSecrets` impl; migrate `cloud_api_key` on load |
| Shell | `set_config` reads/writes secrets via store, strips from JSON save |
| Frontend | Settings cloud key field → save triggers secret store; display “configured” not value |
| Tests | Unit: roundtrip old JSON; integration: keychain mock; E2E: settings spec (no key in file assertion) |

**Exit:** `config.json` has no `cloud_api_key`; E2E green; arch review pass.

**Blocks:** Nothing. **Enables:** safer Settings refactor (S8).

---

### S2 — Tauri command modules (shell)

**Goal:** `lib.rs` is registration + setup only.

| Order | Module | Commands moved |
|-------|--------|----------------|
| 2a | `commands/config.rs` + `library.rs` | Low coupling, read-heavy |
| 2b | `commands/tasks.rs` + `memory.rs` | Isolated domains |
| 2c | `commands/lark.rs` + `insights.rs` + `sync.rs` | Share `lark_opts` via `state.rs` |
| 2d | `commands/chat.rs` | RAG streaming |
| 2e | `commands/agent.rs` | Largest; move `build_agent_context`, `execute_agent_question` |
| 2f | `commands/index.rs` | Delegates to `index_ops` |

**Per step:** Move fns, update `generate_handler!`, `cargo test`, targeted E2E spec.

**Exit:** `lib.rs` < 150 lines; no command logic in `lib.rs`.

---

### S3 — Frontend shell + layout

**Goal:** `App.tsx` becomes router + global chrome.

| Layer | Work |
|-------|------|
| Extract | `IndexProgressBanner`, `StatusBadge`, `SourceKindIcon`, `Sidebar`, `ErrorBanner` |
| Hook | `useIndexProgress` — `listen("index-progress")` |
| Hook | `useJarvisConfig` — initial `get_config`, shared refresh |
| `App.tsx` | `view` state + `{view === "chat" && <ChatView />}` (views still inline or stubbed) |

**Exit:** Navigation E2E green; `data-testid` unchanged.

---

### S4 — Chat view extraction

**Goal:** Chat/agent UI out of `App.tsx`.

| Layer | Work |
|-------|------|
| `views/ChatView.tsx` | Messages, streaming, agent toggles, orchestration UI |
| `hooks/useChat.ts` | Sessions, messages, ask/stream, tool calls state |
| `lib/tauri.ts` | `askInSession`, `askInSessionStream`, session CRUD |

**Preserves:** `Channel` callback behavior, `ask-handle-count` E2E hook, `agent-mode-toggle`, `orchestration-steps`.

**Exit:** `qa.spec.ts`, `agent.spec.ts`, `full-ui.spec.ts` green.

---

### S5 — Library + Tasks views

**Goal:** Index management and task list isolated.

| Layer | Work |
|-------|------|
| `views/LibraryView.tsx` | Sources, Lark/Cursor sync, per-source actions |
| `views/TasksView.tsx` | Task list, toggle, delete |
| `hooks/useLibrary.ts`, `useTasks.ts` | Refresh on view focus |

**Exit:** `lark.spec.ts`, navigation tasks visible, full-ui green. (Tasks CRUD E2E may be added here per CONCERNS.md gap.)

---

### S6 — Memory model improvement

**Goal:** Stable memory identity; reduce fuzzy title collisions.

| Layer | Work |
|-------|------|
| `memory` | New memories: `uri = memory://{uuid}`; strict `resolve_memory_id` (id + exact uri only) |
| `store` | Optional: `list_sources_by_kind(Memory)` helper; migration script sets URI on existing rows |
| `commands/memory.rs` | Thin wrappers unchanged names |
| `views/MemoryView.tsx` + `useMemory.ts` | Extract from App |

**Compatibility:** One release with title fallback + deprecation log; then remove fuzzy `contains` match.

**Exit:** `memory.spec.ts` green; arch review for URI scheme.

---

### S7 — Agent protocol improvement

**Goal:** Structured tool calls; surface parse errors to UI.

| Layer | Work |
|-------|------|
| `agent` | `ToolCallParser` trait; JSON-mode / structured output path for cloud; keep XML parser as fallback |
| `llm` | Optional `complete_json` or response_format for OpenAI-compatible |
| `commands/agent.rs` | Return parse errors in `AskResponse` metadata |
| `ChatView` | Show tool parse failures (non-silent `None`) |

**Exit:** Agent unit tests for parse edge cases; `agent.spec.ts` green with Mock emitting structured JSON.

**Note:** Can run parallel to S4–S5 if Mock protocol updated first.

---

### S8 — Settings view + sync surfacing

**Goal:** Complete settings extraction; scheduled sync errors visible.

| Layer | Work |
|-------|------|
| `views/SettingsView.tsx` | Providers, watch folders, agents, hooks/plugins, rebuild |
| `sync_scheduler` | Persist `last_scheduled_sync_error` in `store.meta` |
| Shell | `get_sync_status` includes error field |
| E2E | Extend `settings.spec.ts` for scheduled sync + reinit |

**Exit:** Settings E2E expanded; S1 keychain UI fully wired.

---

## Balanced Build Order (Recommended Phase Sequence)

Interleave layers so each milestone delivers user-visible stability and avoids “all backend, no UI” or vice versa.

| Phase | Slices | Primary layer | E2E focus |
|-------|--------|---------------|-----------|
| **1.9.0** | S0 + S2a–b + S3 | Shell + FE scaffold | smoke, navigation |
| **1.9.1** | S2c–f | Shell completion | lark, memory, settings (read paths) |
| **1.9.2** | S4 + S1 (start) | Chat FE + config types | qa, agent |
| **1.9.3** | S5 + S1 (finish) | Library/tasks + keychain | lark, full-ui |
| **1.9.4** | S6 | Memory model | memory |
| **1.9.5** | S7 | Agent protocol | agent |
| **1.9.6** | S8 | Settings + sync UX | settings |

**Parallelism within a phase:**

- While **S2** moves commands, **S3** can extract layout (different files).
- **S1** config nesting can start in 1.9.2 while chat view splits; keychain write lands in 1.9.3 with Settings.
- **S7** should not ship in the same release as large **S2e** agent command moves — split to reduce debug surface.

---

## Dependencies Between Refactor Areas

| Area | Depends on | Blocks | Can parallel with |
|------|------------|--------|-------------------|
| FE scaffold (S3) | S0 | S4–S6, S8 | S2 |
| Command split (S2) | S0 | Cleaner agent/chat work | S3, S1 types |
| Config hardening (S1) | — | S8 secrets UI | S2, S3 |
| Chat view (S4) | S3, S2d | — | S5 |
| Library/tasks (S5) | S3, S2b/c | — | S4 |
| Memory model (S6) | S2b, S5 or S3 | — | S7 |
| Agent protocol (S7) | S2e | — | S5, S6 |
| Settings (S8) | S1, S3, S2a | — | S6 |

**Critical path:** S0 → S2 + S3 → S4/S5 → S8. S1 and S7 are important but can slip one phase without blocking file structure.

---

## Cross-Cutting Concerns

### Error handling at IPC boundary

**Current:** `Result<T, String>` everywhere.

**Refactor stance (v1.9.x):** Keep `String` errors for compatibility; optionally add `error_code` in JSON body later. Hooks/`lib/tauri.ts` centralize `invoke` catch → `setErr`.

### Types duplication (Rust ↔ TypeScript)

**Approach:** Manual `src/types/ipc.ts` mirroring serde `camelCase` exports. Do not introduce codegen in v1.9.x unless maintenance proves painful.

### E2E stability

- Preserve all existing `data-testid` values during moves.
- `e2e/helpers.ts` unchanged unless selectors move — update once per view extraction.
- Run `npm run test:e2e:local` per slice, not only at phase end.

### Architecture review gate (per PROJECT.md)

Each phase exit checklist:

1. Module boundaries match this document’s target layout.
2. No new SQLite open sites.
3. Data flow checklist (above) manually verified for touched paths.
4. E2E green.
5. No secrets in `config.json` (after S1 complete).

---

## What We Are Not Refactoring (v1.9.x)

| Area | Reason |
|------|--------|
| `index_ops.rs` / `sync_scheduler.rs` merge | Separate concerns; optional dedup in v2 |
| Store WAL / connection pooling | Performance; not structural |
| New ingest formats (PDF) | Feature, not refactor |
| `sqlite-vec` unsafe registration | Pin versions; upstream watch |
| Plugin/hook sandbox | Design non-goal |
| IPC `Result<T, String>` → typed errors | v2 consideration |

---

## Success Metrics

| Metric | Current (v1.8.0) | Target (v1.9.6) |
|--------|------------------|-----------------|
| `App.tsx` lines | ~2,800 | < 200 |
| `lib.rs` lines | ~1,300 | < 150 |
| Largest `commands/*.rs` | — | < 200 |
| Tauri commands in `lib.rs` | 52 | 0 |
| `cloud_api_key` in JSON | plaintext | keychain only |
| Agent tool parse | silent fail | structured + fallback |
| Memory ID resolution | fuzzy title | `memory://` UUID |
| E2E specs green | yes | yes (expanded gaps optional) |

---

## References

- Project goals: `.planning/PROJECT.md`
- Current architecture: `.planning/codebase/ARCHITECTURE.md`
- Directory map: `.planning/codebase/STRUCTURE.md`
- Debt catalog: `.planning/codebase/CONCERNS.md`
- Engineer policy: `AGENTS.md`
- E2E policy: `.cursor/rules/e2e-required.mdc`

---

*Research complete. Use this document as the architecture dimension input for GSD phase planning and `/gsd-plan-phase`.*
