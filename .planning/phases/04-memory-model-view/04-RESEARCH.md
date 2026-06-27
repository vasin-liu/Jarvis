# Phase 4: Memory Model + View — Research

**Phase:** 4-Memory Model + View  
**Version target:** v1.9.3  
**Requirements:** FE-04, SHELL-04, MEM-01, MEM-02  
**Research date:** 2026-06-27

---

## User Constraints

> Verbatim from `04-CONTEXT.md` — all decisions D-01 through D-20 are locked.

### Memory URI & Migration (MEM-01, MEM-02)
- **D-01:** New memories use **`uri = memory://{uuid-v4}`** (RFC 4122, lowercase hex with hyphens). Add `uuid` crate to `crates/memory`.
- **D-02:** **Startup migration**: scan `SourceKind::Memory` sources with `memory://` + numeric timestamp URI; assign new `memory://{uuid}`; persist; log count at info level. Idempotent — skip already-UUID URIs.
- **D-03:** **`source.id`** remains store PK; for memories **`id == uri`** today via `index_document` — migration rekeys both together.
- **D-04:** Migration runs after store open in non-E2E startup; under `JARVIS_E2E=1` skip or no-op when URIs already valid (E2E uses fresh DB each run).

### Strict ID Resolution (MEM-01, MEM-02)
- **D-05:** `resolve_memory_id` order: exact id → exact uri → exact title → **DEPRECATED fuzzy `contains()`**.
- **D-06:** Fuzzy match emits one-line deprecation warning (`memory title fuzzy match deprecated; use id or memory:// uri`).
- **D-07:** UI/agent callers prefer id or full uri; no UI behavior change beyond migration.

### MemoryView Extraction (FE-04)
- **D-08:** Controlled component: App owns `busy`/`err`; `useMemory` owns list + CRUD IPC; `MemoryView` receives props.
- **D-09:** Inline edit panel unchanged (`memory-edit-panel` below list item).
- **D-10:** `useMemory` separate from `useLibrary` / `useTasks`.
- **D-11:** Types in `src/types/memory.ts`; IPC via `src/lib/tauri.ts` only.
- **D-12:** Save edit passes **`title: null`** so backend regenerates title from content.
- **D-13:** E2E: `memory.spec.ts` + `full-ui.spec.ts` memory block; preserve all `memory-*` testids.

### Tauri Command Split (SHELL-04)
- **D-14:** **`commands/memory.rs`**: move `list_memories_cmd`, `get_memory_content_cmd`, `forget_memory_cmd`, `update_memory_cmd`, `add_memory_cmd` from `lib.rs`.
- **D-15:** **`learn_from_exchange`** stays in **`commands/chat.rs`**.
- **D-16:** Thin wrappers over `crates/memory` only.
- **D-17:** Register in `commands/mod.rs` + `generate_handler!` — **no IPC renames**.

### Quality Gates
- **D-18:** `memory.spec.ts` green.
- **D-19:** `cargo test -p memory` + `npm test` + `npx tsc --noEmit`.
- **D-20:** `full-ui.spec.ts` memory block green after rebuild.

### Claude's Discretion
- Migration hook location (`init_state` vs `memory::migrate_legacy_uris`).
- `eprintln!` vs adding `log` crate for deprecation (prefer no new deps).
- `ipc.ts` re-export from `types/memory.ts`.

### Deferred (do NOT implement)
- Dedicated `memories` table (ARCH-02)
- Removing fuzzy fallback entirely (post–Phase 4)
- Agent tool JSON protocol (Phase 5)
- Settings / Agent view extraction (Phase 5–6)

---

## Standard Stack

[VERIFIED] From workspace manifests:

| Layer | Version |
|-------|---------|
| Rust | stable, MSRV 1.85, edition 2021 |
| Tauri | 2.x |
| uuid | **new** — add to `crates/memory/Cargo.toml` |
| React | 19.x |
| TypeScript | ~5.8.3 |
| Vitest | 3 |
| WebdriverIO | 9 |

**New dependency:**

```toml
# crates/memory/Cargo.toml — add:
uuid = { version = "1", features = ["v4"] }
```

Verify latest on crates.io before pinning. `uuid = "1"` is current stable major.

---

## Architecture Patterns

### 1. Memory identity: id == uri [VERIFIED]

`index_document` sets `source_id = doc.uri.clone()` and uses it as both `Source.id` and `Source.uri`:

```24:35:crates/indexer/src/lib.rs
    let source_id = doc.uri.clone();
    // ...
    let pending = Source {
        id: source_id.clone(),
        kind,
        uri: doc.uri.clone(),
```

**Implication:** Changing a memory's URI requires **rekeying** the `sources` row and updating `chunks.source_id` FK references. `vec_chunks` / `chunks_fts` are keyed by chunk `rowid`, not source_id — no vec/fts rewrite needed on rename.

**Current `add_memory`:**

```105:117:crates/memory/src/learn.rs
    let now = unix_now();
    let uri = format!("memory://{now}");
```

**Target `add_memory`:**

```rust
let uri = format!("memory://{}", Uuid::new_v4());
```

Return value stays the uri string (which equals source id).

### 2. Legacy URI migration [VERIFIED need + design]

**Legacy pattern:** `memory://{unix_timestamp}` — suffix is all digits (e.g. `memory://1719491234`).

**UUID pattern:** `memory://550e8400-e29b-41d4-a716-446655440000` — match with regex or parse suffix as `Uuid`.

**Idempotency:** Skip sources where suffix parses as valid UUID v4.

**Store support needed:** `Store::rename_source_id(old_id, new_id)` — transaction:
1. `get_source(old_id)` — bail if missing
2. Insert new `sources` row with `id = new_id`, `uri = new_id` (same fields otherwise)
3. `UPDATE chunks SET source_id = new_id WHERE source_id = old_id`
4. `UPDATE tasks SET source_id = new_id WHERE source_id = old_id` (if any)
5. `DELETE FROM sources WHERE id = old_id`

Add integration test in `crates/store/tests/` or `crates/memory/tests/`.

**Migration entry point:** `memory::migrate_legacy_memory_uris(store: &Store) -> Result<usize>` called from `init_state` after `Store::open`, before E2E seed:

```rust
if !is_e2e_mode() {
    let n = memory::migrate_legacy_memory_uris(store.as_ref())?;
    if n > 0 { eprintln!("migrated {n} legacy memory URIs to UUID"); }
}
```

E2E uses fresh DB + fixture index — no legacy timestamp URIs in normal path.

### 3. resolve_memory_id deprecation [VERIFIED current code]

```32:50:crates/memory/src/learn.rs
pub fn resolve_memory_id(store: &Store, id_or_title: &str) -> Result<String> {
    // ... exact id, exact title/uri ...
    if let Some(m) = memories.iter().find(|m| m.title.contains(key) || m.uri.contains(key)) {
        return Ok(m.id.clone());
    }
```

**Change:** Before returning fuzzy match, call `eprintln!("[jarvis] memory title fuzzy match deprecated; use id or memory:// uri")` (or equivalent once per call). Add unit test asserting fuzzy still works + warning emitted (capture stderr or test behavior only).

**Resolution order (locked D-05):** exact id → exact uri → exact title → deprecated fuzzy.

Note: exact uri check should compare full `memory://...` string against `m.uri`.

### 4. Controlled View Pattern (Phase 3 reference) [VERIFIED]

Mirror `useTasks.ts` / `TasksView.tsx`:

```
App.tsx
  ├── useState: busy, err (shared)
  ├── useMemory({ onError: setErr })
  │     ├── memories, refreshMemories
  │     ├── addMemory, forgetMemory, edit/save handlers
  ├── refreshLibrary() calls memoryHook.refreshMemories()
  └── <MemoryView
        memories={memories}
        busy={busy}
        newMemoryText / editing state (either in hook or App — prefer hook)
        onAdd / onEdit / onSave / onForget
      />
```

**App.tsx today:** Memory JSX inline ~1052–1177; direct `invoke("list_memories_cmd")` etc.; `MemoryView` is `{children}` stub.

**Target:** All memory JSX in `MemoryView.tsx`; hook owns IPC via `tauri.ts` wrappers.

### 5. Tauri Command Module Pattern [VERIFIED]

Existing `commands/library.rs` pattern — thin `#[tauri::command]` wrappers:

```432:467:src-tauri/src/lib.rs
#[tauri::command]
fn list_memories_cmd(state: tauri::State<'_, AppState>) -> Result<Vec<Source>, String> {
    list_memories(state.store.as_ref()).map_err(|e| e.to_string())
}
// ... get_memory_content_cmd, forget_memory_cmd, update_memory_cmd, add_memory_cmd
```

Move verbatim to `commands/memory.rs`; export via `commands/mod.rs`; remove bodies from `lib.rs`; `generate_handler!` unchanged command names.

`learn_from_exchange` is only used from chat path — stays in `commands/chat.rs` (already imports `memory::learn_from_exchange`).

### 6. Frontend IPC wrappers [VERIFIED gap]

`src/lib/tauri.ts` has library/task/chat wrappers but **no memory functions** yet. Add:

- `listMemories()` → `list_memories_cmd`
- `getMemoryContent(sourceId)` → `get_memory_content_cmd`
- `addMemory(content, title?)` → `add_memory_cmd`
- `updateMemory(id, content, title?)` → `update_memory_cmd`
- `forgetMemory(id)` → `forget_memory_cmd`

Types: `MemorySource` or reuse `Source` from `types/library.ts` with kind filter — prefer **`src/types/memory.ts`** exporting `MemorySource` (subset of Source fields used in UI) or re-export `Source` if identical shape.

---

## Pitfalls

### P1: Migration without chunk FK update [HIGH]
Renaming only `sources.uri` while `id` stays timestamp breaks `get_source(id)` and chunk lookups. Must rekey `sources.id` + `chunks.source_id` atomically.

### P2: add_memory return type confusion [MEDIUM]
`add_memory` returns uri string; UI uses `m.id` from `list_memories` for edit/forget testids. After UUID change both remain identical — no frontend change needed for ids.

### P3: E2E edit title regression [MEDIUM]
Phase 3 fixed full-ui by passing `title: null` on save. Memory extraction must preserve this in `useMemory.saveEdit` (D-12).

### P4: IPC rename accident [HIGH]
SHELL-04 is relocation only. `generate_handler!` must list same five `*_cmd` names. Grep diff before merge.

### P5: refreshLibrary coupling [LOW]
`refreshLibrary` calls `refreshMemories`. After extraction, wire `useMemory.refreshMemories` into `refreshLibrary` callback deps — same pattern as `refreshTasks`.

### P6: Fuzzy match removal too early [MEDIUM]
Keep fuzzy path one release with warn log only (D-05/D-06). Do not remove contains() this phase.

---

## E2E Contract [VERIFIED]

**`e2e/specs/memory.spec.ts`:** add → edit → forget; testids:
- `new-memory-input`, `add-memory-submit`, `memory-list`
- `memory-edit-{id}`, `memory-edit-panel`, `memory-edit-content`, `memory-save-edit`
- `memory-forget-{id}`

**`e2e/specs/full-ui.spec.ts`:** memory block ~lines 24–92 — same testids; uses `setReactInputValue` / `clickViaDom`.

No new spec file required (D-13) — update existing only if selectors move.

---

## Validation Architecture

### Deliverable 1: Memory URI model + migration (crates/memory + crates/store)

| Layer | Test | Location |
|-------|------|----------|
| Unit | `add_memory` creates `memory://{uuid}` uri | `crates/memory/src/learn.rs` `#[cfg(test)]` |
| Unit | `resolve_memory_id` exact id/uri/title; fuzzy still works | `crates/memory/src/learn.rs` |
| Unit | Legacy URI detector skips UUID, matches timestamp | `crates/memory/src/migrate.rs` or `learn.rs` |
| Integration | `rename_source_id` preserves chunks | `crates/store/tests/` or `crates/memory/tests/` |
| Integration | `migrate_legacy_memory_uris` idempotent | `crates/memory/tests/migration.rs` |

### Deliverable 2: commands/memory.rs

| Layer | Test | Location |
|-------|------|----------|
| Compile | `cargo build --manifest-path src-tauri/Cargo.toml` | CI |
| Integration | `cargo test -p tauri-app` or workspace | unchanged IPC |

### Deliverable 3: MemoryView + useMemory

| Layer | Test | Location |
|-------|------|----------|
| Unit (FE) | `useMemory` mock tauri, `refreshMemories` sets state | `src/hooks/useMemory.test.ts` (new) |
| Typecheck | `npx tsc --noEmit` | CI |
| Unit (FE) | `npm test` | Vitest |

### Deliverable 4: E2E gate

| Layer | Test | Location |
|-------|------|----------|
| E2E | Memory CRUD journey | `npm run test:e2e:local -- --spec e2e/specs/memory.spec.ts` |
| E2E | Full UI memory block | `npm run test:e2e:local -- --spec e2e/specs/full-ui.spec.ts` |
| E2E | Navigation to memory view | `navigation.spec.ts` if memory nav testid touched |

---

## Key File Map

| File | Action |
|------|--------|
| `crates/memory/Cargo.toml` | Add `uuid` |
| `crates/memory/src/learn.rs` | UUID uri, resolve deprecation |
| `crates/memory/src/migrate.rs` | New: legacy URI migration |
| `crates/memory/src/lib.rs` | Export migrate |
| `crates/store/src/store.rs` | Add `rename_source_id` |
| `src-tauri/src/commands/memory.rs` | New: 5 memory commands |
| `src-tauri/src/commands/mod.rs` | Export memory commands |
| `src-tauri/src/lib.rs` | Remove command bodies; call migration in init |
| `src/types/memory.ts` | New types |
| `src/lib/tauri.ts` | Memory IPC wrappers |
| `src/hooks/useMemory.ts` | New hook |
| `src/hooks/useMemory.test.ts` | Vitest |
| `src/views/MemoryView.tsx` | Full controlled view |
| `src/App.tsx` | Slim memory section |

---

*Research complete — ready for planning.*
