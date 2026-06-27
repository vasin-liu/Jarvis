# Phase 4: Memory Model + View - Context

**Gathered:** 2026-06-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Stabilize the **memory identity model** (`memory://{uuid}` URIs, strict ID resolution with one-release deprecation) and **extract the Memory view** from `App.tsx` — **zero user-visible behavior regression** except intentional deprecation logging for fuzzy title matching.

**In scope:**
- `memory://{uuid-v4}` for **new** memories (replace `memory://{unix_timestamp}`)
- **Startup migration** rewrites legacy `memory://{timestamp}` URIs to `memory://{uuid}`
- `resolve_memory_id`: strict id/uri/title exact match; **deprecate fuzzy `contains()`** with warn log (one release)
- `MemoryView.tsx` + `useMemory.ts` + `src/types/memory.ts`
- `src-tauri/src/commands/memory.rs` — relocate memory CRUD IPC from `lib.rs`
- Extend `src/lib/tauri.ts` with memory IPC wrappers
- Preserve all `memory-*` testids; `memory.spec.ts` + `full-ui.spec.ts` memory block green
- `cargo test -p memory` + targeted workspace tests + E2E

**Out of scope (later phases):**
- Dedicated `memories` table (ARCH-02) — URI scheme is sufficient this phase
- Agent tool protocol changes (Phase 5)
- Settings / Agent view extraction (Phase 5–6)
- Removing title fallback entirely (after one-release deprecation window)
- Auto-learn-from-chat behavior changes (`learn_from_exchange` stays in chat path)

</domain>

<decisions>
## Implementation Decisions

### Memory URI & Migration (MEM-01, MEM-02)
- **D-01:** New memories use **`uri = memory://{uuid-v4}`** (RFC 4122 UUID, lowercase hex with hyphens). Add `uuid` crate to `crates/memory` (or workspace) as needed.
- **D-02:** **Startup migration** (same load path as config migration pattern): scan `SourceKind::Memory` sources whose URI matches `memory://` + numeric timestamp; assign new `memory://{uuid}`, persist via store update, log count at info level. Idempotent — skip already-UUID URIs.
- **D-03:** **`source.id`** remains the store's primary key (unchanged); URI is the stable external reference for agent tools and display. Do not force `id == uri suffix`.
- **D-04:** Migration runs in **non-E2E** startup after store open; under `JARVIS_E2E=1` skip migration side effects that could break deterministic fixtures (or make migration a no-op when URIs already valid).

### Strict ID Resolution (MEM-01, MEM-02)
- **D-05:** `resolve_memory_id` resolution order: **(1) exact source id** → **(2) exact uri** → **(3) exact title** → **(4) DEPRECATED fuzzy `contains()` on title/uri**.
- **D-06:** When step 4 matches: emit **`log::warn!` or equivalent one-line deprecation** (`memory title fuzzy match deprecated; use id or memory:// uri`) — fuzzy path removed in a future release after one cycle.
- **D-07:** Agent/chat callers should prefer **id** or full **`memory://` uri** in new code; no behavior change required in UI this phase beyond migration.

### MemoryView Extraction (FE-04)
- **D-08:** **Controlled component pattern** (same as Phase 2–3): `App.tsx` holds `busy`/`err`; `useMemory` owns memory list + CRUD IPC; `MemoryView` receives props.
- **D-09:** **Inline edit panel unchanged** — `memory-edit-panel` expands below list item (not modal). All Memory JSX moves out of `App.tsx`.
- **D-10:** **`useMemory` is separate** from `useLibrary` / `useTasks` — no merged hook.
- **D-11:** Domain types in **`src/types/memory.ts`**; IPC via **`src/lib/tauri.ts`** only (no direct `invoke` in view/hook).
- **D-12:** On save edit: **regenerate title from content** (`title: null` to backend — same fix as Phase 3 full-ui) so list text reflects edits.
- **D-13:** E2E: update **`memory.spec.ts`** + **`full-ui.spec.ts`** memory journey in same PR; preserve all `memory-*` testids (FE-06).

### Tauri Command Split (SHELL-04)
- **D-14:** Create **`src-tauri/src/commands/memory.rs`** and move from `lib.rs`: `list_memories_cmd`, `get_memory_content_cmd`, `forget_memory_cmd`, `update_memory_cmd`, `add_memory_cmd`.
- **D-15:** **`learn_from_exchange`** stays in **`commands/chat.rs`** (chat-triggered learning, not memory CRUD).
- **D-16:** Commands remain **thin wrappers** over `crates/memory` — no business logic in command file.
- **D-17:** Register in `commands/mod.rs` + `generate_handler!` — **additions/moves only**, no IPC renames.

### Quality Gates
- **D-18:** `memory.spec.ts` passes (add → edit → forget journey).
- **D-19:** `cargo test -p memory` + `npm test` + `npx tsc --noEmit` required.
- **D-20:** Regression: `full-ui.spec.ts` memory block passes after rebuild.

### Claude's Discretion
- Exact migration hook location (store init vs dedicated `memory::migrate_uris` called from `init_state`).
- Whether to use `tracing` vs `eprintln!` for deprecation if no logging crate in memory — prefer minimal deps.
- `ipc.ts` re-export from `types/memory.ts`.
- UUID crate version pin at implementation time.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` — Phase 4 goal, success criteria, key files
- `.planning/REQUIREMENTS.md` — FE-04, SHELL-04, MEM-01, MEM-02
- `.planning/PROJECT.md` — Core value, refactor constraints
- `.planning/phases/03-library-tasks-keychain/03-CONTEXT.md` — Controlled view + hook patterns (D-08–D-11)

### Codebase context
- `.planning/codebase/ARCHITECTURE.md` — Memory indexing pipeline
- `.planning/codebase/STRUCTURE.md` — Views/hooks/commands layout
- `.planning/codebase/TESTING.md` — E2E spec map (`memory.spec.ts`)

### Engineering policy
- `AGENTS.md` — TDD, E2E mandatory
- `.cursor/rules/e2e-required.mdc` — E2E for user-facing changes
- `.cursor/rules/tdd-goal-driven.mdc` — Red-green-refactor

### Implementation targets (current state)
- `crates/memory/src/learn.rs` — `add_memory` uses `memory://{unix_now}`; `resolve_memory_id` has fuzzy contains
- `src/App.tsx` — Memory JSX inline (~lines 1064–1160); `refreshMemories` via `list_memories_cmd`
- `src/views/MemoryView.tsx` — Stub `{children}` passthrough
- `src-tauri/src/lib.rs` — 5 memory commands still inline
- `e2e/specs/memory.spec.ts` — CRUD journey testids

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/hooks/useLibrary.ts` / `useTasks.ts` — Hook pattern for `useMemory`
- `src/views/LibraryView.tsx` / `TasksView.tsx` — Controlled extraction reference (Phase 3)
- `src/lib/tauri.ts` — IPC wrapper pattern
- `crates/config/src/file.rs` — `load_config_with_migration` pattern for startup migration

### Established Patterns
- Controlled views: App owns `busy`/`err`; hooks own domain state + tauri.ts IPC
- Thin Tauri commands in `src-tauri/src/commands/*.rs`
- E2E testids stable across JSX moves

### Integration Points
- `index_document` in `add_memory` / `update_memory` — URI change affects re-index path
- Agent tools (`forget_memory`, `update_memory`) call `resolve_memory_id` — deprecation log matters
- `refreshLibrary()` in App currently calls `refreshMemories()` — after extraction, App calls `useMemory.refreshMemories` or equivalent

</code_context>

<specifics>
## Specific Ideas

- User confirmed **中文沟通** for discuss-phase; decisions captured in English for downstream agents.
- URI migration: **uuid-v4 for new + migrate old timestamp URIs at startup** (not leave old URIs forever).
- Resolution: **one-release warn then remove fuzzy** — not immediate hard break.
- Edit UX: **keep inline panel** — no modal redesign this phase.

</specifics>

<deferred>
## Deferred Ideas

- **ARCH-02 dedicated `memories` table** — deferred; `memory://` URI scheme sufficient for v1.9.3
- **Remove fuzzy title fallback entirely** — after one-release deprecation window (post–Phase 4)
- **Agent tool JSON protocol** — Phase 5

</deferred>

---

*Phase: 4-Memory Model + View*
*Context gathered: 2026-06-27*
