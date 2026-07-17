# Phase 3: Library/Tasks + Keychain - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Extract **Library** and **Tasks** views (with dedicated hooks) from `App.tsx`, complete **index/Lark Tauri command modules**, and migrate **`cloud_api_key`** from plaintext `config.json` to the **OS keychain** — **zero user-visible behavior regression** except intentional UX improvements (masked secret field, deprecated manual Lark URL paste, visible sync errors).

**In scope:**
- `LibraryView.tsx` + `useLibrary.ts` — sources list, index actions, bulk ops, stats UI
- `TasksView.tsx` + `useTasks.ts` — task list, complete/dismiss, insights UI
- `src/types/library.ts` + `src/types/tasks.ts`
- `src-tauri/src/commands/index.rs` — local/Cursor index commands (rebuild, retry, scan)
- `src-tauri/src/commands/lark.rs` — Lark sync, auth, health (migrated from `sync.rs` / `lib.rs`)
- Extend `commands/library.rs` — sources/stats + all task IPC commands
- `crates/config/src/secrets.rs` + `keyring` — read/write/clear `cloud_api_key`
- CFG-02/CFG-03: startup migration of legacy plaintext key; strip from JSON on save
- E2E: `lark.spec.ts` + `full-ui.spec.ts` + existing navigation/settings specs updated
- `cargo test --workspace` + `npm test` (QA-03)

**Out of scope (later phases):**
- `commands/tasks.rs` as separate module (tasks stay in `library.rs` this phase)
- Settings view extraction / `useJarvisConfig` (Phase 5–6)
- Scheduler interval/toggle UI changes
- Nested TS `AppConfig` mirroring Rust nesting (Phase 6)
- Main nav AppShell / Sidebar extraction (Phase 6)
- Agent protocol / `commands/agent.rs` (Phase 5)
- `memory://` URI scheme (Phase 4)

</domain>

<decisions>
## Implementation Decisions

### Keychain & Secrets (CFG-02, CFG-03)
- **D-01:** Settings API key input shows a **masked placeholder** when a key exists in keychain (not "last 4 chars", not "empty = keep existing").
- **D-02:** **Startup auto-migration**: if `config.json` contains plaintext `cloud_api_key`, write to keychain and remove the field from JSON on first load.
- **D-03:** **`JARVIS_E2E=1` fully skips keyring** — no Windows Credential Manager access in CI; E2E uses mock/empty secret path.
- **D-04:** Provide an explicit **「清除密钥」/ Clear API key** button in Settings to delete the keychain entry.
- **D-05:** Keyring identity: **service `jarvis`**, **account `cloud_api_key`**.
- **D-06:** On keyring read failure: **graceful degradation** — cloud provider calls surface error; Settings shows inline hint (do not block app startup; do not fail silently).
- **D-07:** **Remove `cloud_api_key` from `config.json` serialization**; `AppConfig` may hold a **runtime-only** field populated from keychain on load (not written back to disk).

### Library/Tasks Frontend Extraction (FE-03)
- **D-08:** **Controlled component pattern** (same as Phase 2 Chat): `App.tsx` holds shared state; `LibraryView`/`TasksView` receive props; hooks own domain IPC.
- **D-09:** **Two hooks**: `useLibrary.ts` + `useTasks.ts` (not merged).
- **D-10:** **`index-progress` event listen stays in `App.tsx`**; progress passed as props to `LibraryView`.
- **D-11:** **Move all Library + Tasks JSX** out of `App.tsx` this phase (full extraction, not incremental).
- **D-12:** Domain types in **`src/types/library.ts`** and **`src/types/tasks.ts`**.
- **D-13:** E2E: **update existing** `settings.spec.ts`, `navigation.spec.ts`, `full-ui.spec.ts` — **no new** `library.spec.ts` / `tasks.spec.ts` files.

### Lark Sync Primary Path (implements Phase 2 D-36)
- **D-14:** Primary Lark UX: **Settings「立即同步」+ background scheduler** — scheduler config UI unchanged this phase.
- **D-15:** E2E: **extend `lark.spec.ts`** and assert sync outcomes in **`full-ui.spec.ts`**.
- **D-16:** **Deprecate/hide manual URL paste** in Library — unified **`lark-cli`** discovery/sync path.
- **D-17:** **`lark-cli` auth/health status** displayed in **Settings** (not Library).
- **D-18:** Manual Lark sync failure: **visible inline error in Settings** (user-visible feedback, not silent background-only).
- **D-19:** **No scheduler interval/toggle UI changes** this phase — reuse existing `scheduled_sync_*` settings.

### Tauri Command Module Split (SHELL-03)
- **D-20:** **`commands/index.rs`**: rebuild, retry, folder scan, Cursor sync — local index orchestration.
- **D-21:** **`commands/lark.rs`**: all Lark IPC (sync, auth, health) — migrate from `sync.rs` / `lib.rs`.
- **D-22:** **`commands/library.rs`**: extend with sources/stats **and all task commands** (`list_tasks`, complete, etc.) — **no `tasks.rs`** this phase.
- **D-23:** Commands are **thin wrappers**; heavy logic remains in **`index_ops.rs`** (do not move orchestration into command files).
- **D-24:** Config save/load stays in **`commands/config.rs`**; secret read/write/clear goes through **`crates/config/src/secrets.rs`** (not a separate `commands/secrets.rs`).
- **D-25:** **`index_ops.rs` untouched structurally** — only `lib.rs` registration moves to new modules.

### Quality Gates
- **D-26:** Full E2E suite (`e2e/specs/**/*.spec.ts`) must pass before merge.
- **D-27:** Preserve or update all `data-testid` selectors in same PR as JSX moves (FE-06).
- **D-28:** `cargo test --workspace` + `npm test` required (QA-03).

### Claude's Discretion
- Exact masked-placeholder copy and input `type="password"` behavior when editing vs displaying stored key.
- `secrets.rs` internal API shape (`SecretStore` trait vs free functions) as long as E2E skip path is clean.
- Minor hook internal organization within `useLibrary` / `useTasks` as long as controlled-props contract holds.
- Whether `ipc.ts` re-exports from `types/library.ts` / `types/tasks.ts`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 3 goal, success criteria, key files
- `.planning/REQUIREMENTS.md` — FE-03, SHELL-03, CFG-02, CFG-03 traceability
- `.planning/PROJECT.md` — Core value, constraints, refactor framing
- `.planning/phases/02-chat-extraction-config-types/02-CONTEXT.md` — Phase 2 patterns (controlled views, D-36 Lark vision)

### Target architecture
- `.planning/research/ARCHITECTURE.md` — Command grouping map (index vs lark vs library)
- `.planning/research/SUMMARY.md` — Phase 3 rationale (keyring CI note)
- `.planning/research/PITFALLS.md` — Config serde, testid stability, App state coupling

### Codebase context
- `.planning/codebase/ARCHITECTURE.md` — Index pipeline, Lark subprocess flow
- `.planning/codebase/STRUCTURE.md` — Views/hooks/commands layout
- `.planning/codebase/TESTING.md` — E2E spec map (lark, full-ui, settings)

### Engineering policy
- `AGENTS.md` — TDD, E2E mandatory, file boundaries
- `.cursor/rules/e2e-required.mdc` — E2E mandatory for user-facing changes
- `.cursor/rules/tdd-goal-driven.mdc` — Red-green-refactor workflow

### Implementation targets (current state)
- `src/App.tsx` — Library (~line 1032) and Tasks (~line 1274) JSX still inline; Settings edits `cloud_api_key` (~line 1652)
- `src/views/LibraryView.tsx` — Stub `{children}` passthrough (mirror pre-Phase-2 ChatView)
- `src/views/TasksView.tsx` — Stub passthrough
- `src/views/ChatView.tsx` — Reference controlled extraction pattern (Phase 2 complete)
- `src/hooks/useChat.ts` — Reference hook pattern
- `src/lib/tauri.ts` — Extend with library/tasks/index/lark IPC wrappers
- `src-tauri/src/lib.rs` — ~38 `#[tauri::command]` handlers still inline
- `src-tauri/src/commands/{config,library,sync,chat}.rs` — Existing module pattern
- `src-tauri/src/index_ops.rs` — Index orchestration (delegate from new command modules)
- `src-tauri/src/sync_scheduler.rs` — Background Lark/watch/Cursor sync
- `crates/config/src/types.rs` — `cloud_api_key` in `EmbeddingConfig` (plaintext today)
- `crates/config/src/providers.rs` — Reads `cloud_api_key` for cloud embedder/chat
- `e2e/specs/lark.spec.ts` — Primary Lark journey spec to extend

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/views/ChatView.tsx` + `src/hooks/useChat.ts` — **Controlled extraction template** for Library/Tasks
- `src/views/LibraryView.tsx` / `TasksView.tsx` — Stubs exist; replace passthrough with full layout
- `src/lib/tauri.ts` — Typed `invoke` wrapper pattern (config/library/sync/chat from Phases 1–2)
- `src-tauri/src/commands/library.rs` — Already has `list_sources`, `source_count`, `remove_source`; extend for tasks
- `src-tauri/src/commands/sync.rs` — Lark-related commands to migrate into `lark.rs`
- `src-tauri/src/index_ops.rs` — Rebuild/retry/sync helpers; commands should delegate here
- `src-tauri/src/e2e.rs` — `JARVIS_E2E=1` mock path; must bypass keyring

### Established Patterns
- Tauri commands return `Result<T, String>`; thin wrappers over domain crates
- `snake_case` IPC names — no renames (frontend stability)
- Controlled view: App retains `busy`/`err`/global listeners; view + hook own domain state
- E2E: deterministic mocks + stable `data-testid` selectors

### Integration Points
- `App.tsx` view router: `{view === "library" && <LibraryView ... />}` — App passes index progress + global chrome props
- `lib.rs::generate_handler!` — register `commands::index::*` and `commands::lark::*`
- `config::load` / `config::save` — must integrate secrets migration on load and strip secret on save
- `build_embedder` / `build_chat_model` — must resolve API key from keychain (or E2E empty) at runtime
- `sync_scheduler.rs` — continues periodic Lark sync; no UI changes this phase

</code_context>

<specifics>
## Specific Ideas

- User prefers **中文** for discuss/plan communication; technical artifacts stay English.
- **Phase 2 D-36 locked**: Lark docs/IM/mail/tasks sync via **`lark-cli` scheduled fetch** — manual URL/token copy is **not** the primary path; Phase 3 implements this UX direction.
- API key field: when key exists, show **mask placeholder**; user types new value to replace; **Clear** button to remove.
- Lark health/auth belongs in **Settings**, not Library — Library focuses on indexed content.
- Follow Phase 2 single-PR discipline: FE extraction + shell commands + keychain land together with full E2E gate.

</specifics>

<deferred>
## Deferred Ideas

### Phase 4+
- `memory://` URI scheme + Memory view extraction → Phase 4
- `commands/tasks.rs` as separate module if `library.rs` grows unwieldy → future refactor
- Scheduler Lark interval toggle / dedicated Lark auto-sync switch UI → future Settings phase
- Nested TS `AppConfig` mirroring Rust structs → Phase 6
- Settings view extraction + `useJarvisConfig` → Phase 5–6
- Main nav AppShell / Sidebar component → Phase 6

### Out of scope (rejected this phase)
- E2E using real Windows Credential Manager
- Keeping manual Lark URL paste as primary import path
- Moving `index_ops.rs` logic into command modules

</deferred>

---

*Phase: 3-Library/Tasks + Keychain*
*Context gathered: 2026-06-26*
