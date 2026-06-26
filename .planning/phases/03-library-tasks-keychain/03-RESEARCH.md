# Phase 3: Library/Tasks + Keychain — Research

**Phase:** 3-Library/Tasks + Keychain  
**Version target:** v1.9.2  
**Requirements:** FE-03, SHELL-03, CFG-02, CFG-03  
**Research date:** 2026-06-26

---

## User Constraints

> Verbatim from `03-CONTEXT.md` — all decisions D-01 through D-28 are locked.

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

### Deferred (do NOT implement)
- `commands/tasks.rs` as separate module
- Settings view extraction / `useJarvisConfig`
- Scheduler interval/toggle UI changes
- Nested TS `AppConfig` mirroring Rust nesting
- Main nav AppShell / Sidebar extraction
- `memory://` URI scheme

---

## Standard Stack

[VERIFIED] From `jarvis-stack.mdc`, `Cargo.toml` workspace, `package.json`:

| Layer | Version |
|-------|---------|
| Rust toolchain | stable (`rust-toolchain.toml`) |
| MSRV | 1.85, edition 2021 |
| Tauri | 2.x (`tauri = "2"`) |
| keyring | **`keyring = "3"`** — add to `crates/config/Cargo.toml` (new dep this phase) |
| serde / serde_json | 1 (workspace) |
| thiserror | 1 (workspace) |
| React | 19.x |
| TypeScript | ~5.8.3 |
| Tailwind CSS | v4 |
| motion | ^12 (`motion/react`) |
| Icons | `@tabler/icons-react` ^3.44 |

**New dependency to add:** [VERIFIED] `keyring` crate is **not currently in any Cargo.toml**. Must be added to `crates/config/Cargo.toml` only — the `config` crate owns the secrets abstraction (D-24).

```toml
# crates/config/Cargo.toml — add:
keyring = "3"
```

Verify latest on crates.io before pinning. As of 2026, `keyring = "3"` is the current major.

---

## Architecture Patterns

### 1. Controlled View Pattern (Phase 2 reference) [VERIFIED]

`ChatView.tsx` is the template. App.tsx retains shared state and event listeners; view components are presentational with typed props; hooks own IPC.

```
App.tsx
  ├── useState: sources, tasks, indexProgress, busy, err
  ├── listen("index-progress") → setIndexProgress  [stays in App.tsx — D-10]
  ├── <LibraryView
  │     sources={sources}
  │     tasks={tasks}              // tasks needed for count display
  │     busy={busy}
  │     indexProgress={indexProgress}
  │     onRebuild={...}
  │     onRemoveSource={...}
  │     ... />
  └── <TasksView
        tasks={tasks}
        busy={busy}
        onToggleTask={...}
        onDeleteTask={...} />
```

**useLibrary.ts** owns: `listSources`, `refreshSources`, source-scoped actions (retry, remove, summarize, extract tasks, run insights).  
**useTasks.ts** owns: `listTasks`, `updateTaskStatus`, `deleteTask`, refresh.

Both hooks accept `onError` callback matching `useChat.ts` pattern [CITED: `src/hooks/useChat.ts` line 11-13].

### 2. Tauri Command Module Pattern [VERIFIED]

Existing modules (`commands/library.rs`, `commands/config.rs`, `commands/sync.rs`) define the pattern:

```rust
// commands/index.rs — example thin wrapper
#[tauri::command]
pub async fn rebuild_index(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RebuildReport, String> {
    let cfg = state.config();
    let lark = lark_opts(&cfg);
    let report = rebuild_all_sources(
        state.store.as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &ProcessRunner,
        &lark,
        &cfg.sync.cursor_projects_root,
        "rebuild",
        |event| emit_index_progress(&app, event),
    )
    .await?;
    emit_index_complete(&app, &report);
    Ok(report)
}
```

`lark_opts()` helper currently in `lib.rs` — move to `state.rs` or inline in `commands/lark.rs` [ASSUMED: move to `state.rs` as a helper on `AppState` so both index and lark commands can access it without duplication].

### 3. Secrets Architecture [VERIFIED from D-07, PITFALLS §11]

`cloud_api_key` currently lives in `EmbeddingConfig` with `#[serde(default)]` — it serializes to JSON today.

**Target state:**
```rust
// crates/config/src/types.rs — EmbeddingConfig
pub struct EmbeddingConfig {
    // ... all other fields unchanged ...
    #[serde(skip)]  // strip from JSON (D-07)
    pub cloud_api_key: String,  // runtime-only, populated from keychain on load
}
```

`#[serde(skip)]` will:
- Omit the field on `serde_json::to_string_pretty` → JSON never contains the key  
- Use `Default::default()` (empty String) when deserializing existing JSON that still has the field (legacy JSON with the key will have it silently dropped on next save — this is correct migration behavior)

**Important:** `#[serde(skip)]` on a field with `#[serde(default)]` already present needs careful ordering. Remove `#[serde(default)]` and replace with `#[serde(skip)]` since `skip` implies skip-both-directions and uses Default for deserialization [VERIFIED: Rust serde docs].

### 4. Secrets Migration on Load [VERIFIED from D-02, PITFALLS §11]

Migration happens in `load_config` (or a new wrapper) in `crates/config/src/file.rs`:

```rust
pub fn load_config_with_migration(path: impl AsRef<Path>) -> Result<AppConfig> {
    // 1. Read raw JSON before serde to check for plaintext key
    let raw = if path.as_ref().exists() {
        fs::read_to_string(path.as_ref())?
    } else {
        return Ok(AppConfig::default());
    };
    
    // 2. Peek for plaintext key before deserializing
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let legacy_key = json.get("cloud_api_key")
        .or_else(|| json.get("embedding").and_then(|e| e.get("cloud_api_key")))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned());
    
    // 3. Deserialize (cloud_api_key serde(skip) → empty string)
    let mut config: AppConfig = serde_json::from_str(&raw)?;
    
    // 4. Migrate: write to keychain, then resave without key
    if let Some(key) = legacy_key {
        if !is_e2e_mode() {   // D-03: skip keyring in E2E
            secrets::set_api_key(&key)?;  // keyring write
            save_config(path.as_ref(), &config)?;  // resave strips the key
        }
        config.embedding.cloud_api_key = key;  // populate runtime field
    } else {
        // 5. Normal load: read from keychain into runtime field
        if !is_e2e_mode() {
            config.embedding.cloud_api_key = secrets::get_api_key().unwrap_or_default();
        }
    }
    
    Ok(config)
}
```

**E2E path (D-03):** `is_e2e_mode()` → skip all keyring calls → `cloud_api_key` stays empty → `MockChatModel`/`MockEmbedder` are used anyway (forced by `apply_e2e_config`). No Windows Credential Manager access.

### 5. secrets.rs Module [ASSUMED shape, D-24]

Located at `crates/config/src/secrets.rs`. Exposes free functions (simpler than a trait for this phase since there's only one secret):

```rust
// crates/config/src/secrets.rs
use keyring::Entry;

const SERVICE: &str = "jarvis";       // D-05
const ACCOUNT: &str = "cloud_api_key"; // D-05

pub fn get_api_key() -> Result<String, keyring::Error> {
    Entry::new(SERVICE, ACCOUNT)?.get_password()
}

pub fn set_api_key(key: &str) -> Result<(), keyring::Error> {
    Entry::new(SERVICE, ACCOUNT)?.set_password(key)
}

pub fn delete_api_key() -> Result<(), keyring::Error> {
    Entry::new(SERVICE, ACCOUNT)?.delete_credential()
}
```

**Error handling (D-06):** Callers of `get_api_key` must NOT panic on failure. `unwrap_or_default()` for startup load; surface error in Settings IPC for user-visible degradation.

**E2E bypass:** All callers check `is_e2e_mode()` before calling into `secrets.rs`. The `e2e.rs` `apply_e2e_config` forces Mock providers regardless, so even if key is empty, no cloud calls are made.

### 6. providers.rs Integration [VERIFIED: `crates/config/src/providers.rs`]

Currently `build_embedder` and `build_chat_model` read `config.embedding.cloud_api_key` directly. After migration, `cloud_api_key` in `AppConfig` is populated at load time (runtime-only, D-07). **No change needed in providers.rs** — it keeps reading `config.embedding.cloud_api_key` as before. The key is simply sourced from keychain instead of JSON at load.

When `set_config` is called from Settings (saving new key), `commands/config.rs` must:
1. Extract `cloud_api_key` from the incoming `AppConfig`
2. Call `secrets::set_api_key(&key)` if non-empty, or `secrets::delete_api_key()` if empty
3. Clear `config.embedding.cloud_api_key` before `state.save_config()` (so JSON stays clean)
4. Populate runtime field for the in-memory `AppState` config

### 7. Command Module Registration Pattern [VERIFIED: `lib.rs` lines 947–1000]

Current `commands/mod.rs` re-exports from sub-modules and `lib.rs` uses them in `generate_handler!`. Adding `index.rs` and `lark.rs`:

```rust
// commands/mod.rs — additions
pub(crate) mod index;
pub(crate) mod lark;

pub use index::{
    rebuild_index, reinit_and_rebuild_index, retry_source,
    add_watch_folder, remove_watch_folder,
    list_cursor_transcripts, sync_cursor_transcripts_cmd,
    index_file,
};
pub use lark::{
    check_lark_connection, detect_lark_cli,
    sync_lark_doc, sync_lark_url, sync_lark_sheet,
    sync_lark_mail, sync_lark_im,
};
```

And `commands/library.rs` extended with task commands:
```rust
pub use library::{
    list_sources, remove_source, source_count,
    list_tasks, update_task_status, delete_task,
    summarize_source_cmd, extract_tasks_cmd, run_insights_all_cmd,
};
```

### 8. Frontend IPC Types [VERIFIED: `src/types/ipc.ts`, `src/lib/tauri.ts`]

Add to `src/types/library.ts`:
```typescript
export interface Source {
  id: string;
  uri: string;
  title: string;
  status: "pending" | "indexed" | "failed";
  source_kind: string;
  summary?: string | null;
}
```

Add to `src/types/tasks.ts`:
```typescript
export interface Task {
  id: string;
  title: string;
  description?: string | null;
  status: "pending" | "done";
  source_title?: string | null;
  source_id?: string | null;
}
```

Extend `src/lib/tauri.ts` with new typed wrappers:
```typescript
export function listTasks() { return invoke<Task[]>("list_tasks"); }
export function updateTaskStatus(id: string, status: string) {
  return invoke<void>("update_task_status", { id, status });
}
export function deleteTask(id: string) { return invoke<void>("delete_task", { id }); }
export function rebuildIndex() { return invoke<RebuildReport>("rebuild_index"); }
export function retrySource(id: string) { return invoke<RebuildReport>("retry_source", { id }); }
export function summarizeSource(id: string) { return invoke<void>("summarize_source_cmd", { id }); }
export function extractTasks(id: string) { return invoke<void>("extract_tasks_cmd", { id }); }
export function runInsightsAll() { return invoke<void>("run_insights_all_cmd"); }
export function syncLarkDoc(token: string) { return invoke<string>("sync_lark_doc", { token }); }
export function checkLarkConnection() { return invoke<LarkAuthStatus>("check_lark_connection"); }
export function getApiKeyStatus() { return invoke<{ has_key: boolean }>("get_api_key_status"); }
export function setApiKey(key: string) { return invoke<void>("set_api_key"); }
export function clearApiKey() { return invoke<void>("clear_api_key"); }
```

**Note:** `get_api_key_status` / `set_api_key` / `clear_api_key` are **new commands** needed for the Settings UI. These go in `commands/config.rs` (D-24).

---

## Don't Hand-Roll

### Keyring
- **Don't** implement Windows Credential Manager via `winapi` directly — use `keyring = "3"` crate which handles Windows/macOS/Linux via a single API [CITED: PITFALLS §11].
- **Don't** implement a custom retry loop on keyring failure — fail gracefully with `unwrap_or_default()` + surface in UI.
- **Don't** serialize keyring errors with key contents — redact in error messages.

### Serde skip pattern
- **Don't** use a custom `Serialize` impl to hide `cloud_api_key` — `#[serde(skip)]` is the idiomatic approach and already used for `fastembed_cache_dir` in this codebase [VERIFIED: `types.rs` line 143].

### State migration
- **Don't** write a separate migration binary — do it inline in `load_config_with_migration` on startup (same startup path as `fastembed_cache_dir` injection today).
- **Don't** add a migration table to SQLite — config migration belongs entirely in `crates/config`.

### IPC types
- **Don't** reach into `src/types/ipc.ts` from hooks directly in a way that bypasses `src/lib/tauri.ts` wrappers — all `invoke` calls go through `tauri.ts` per existing pattern [CITED: `src/hooks/useChat.ts` imports from `../lib/tauri`].

### Lark commands
- **Don't** add a new Lark fetching mechanism — existing `fetch_doc`, `fetch_from_url`, `fetch_sheet`, `fetch_mail`, `fetch_im_chat`, `ProcessRunner` in `index_ops.rs` cover all cases. Commands just move the wrapper code from `lib.rs`.

---

## Common Pitfalls

### P1: Keyring in E2E mode breaks CI [PITFALLS §11, D-03]
`JARVIS_E2E=1` does not get a real Windows Credential Manager. Any unconditional keyring call in startup crashes CI.  
**Prevention:** Every `secrets::*` call is guarded by `if !is_e2e_mode()`. Test this by running `JARVIS_E2E=1 cargo test -p config`.

### P2: cloud_api_key still in JSON after migration [PITFALLS §11]
If `#[serde(skip)]` is applied but `save_config` is not called after migration, the next startup re-reads the old JSON and migrates again (idempotent, but messy). Call `save_config` immediately after migrating.  
**Prevention:** Integration test: write config with `cloud_api_key`, load with migration, assert field absent in saved JSON.

### P3: serde(skip) breaks existing round-trip tests [PITFALLS §12]
The existing test in `file.rs::roundtrips_config_file` will still pass because `cloud_api_key` was already `Default`. The `deserializes_legacy_config_without_cloud_fields` test may need update if it asserts `cfg.embedding.cloud_api_key.is_empty()` — that assertion stays valid with `#[serde(skip)]`.  
**Prevention:** Run `cargo test -p config` after changing `#[serde(default)]` to `#[serde(skip)]`.

### P4: data-testid breakage during Library/Tasks extraction [PITFALLS §2, D-27]
`lark-sync-url-input`, `lark-sync-submit`, `library-stats`, `source-list`, `task-list`, `task-list-items`, `toggle-task-{id}`, `run-insights-all` — all must move with their JSX.  
**Prevention:** Audit all `data-testid` in Library/Tasks JSX in `App.tsx` before moving; grep `e2e/` to confirm coverage.

### P5: Index-progress event listen duplication [D-10]
If `useLibrary.ts` also calls `listen("index-progress")`, App.tsx and the hook will both receive events → double progress updates.  
**Prevention:** D-10 is explicit: `listen("index-progress")` stays in `App.tsx` only. `LibraryView` receives `indexProgress` as a prop.

### P6: Lark URL paste deprecation breaking lark.spec.ts [D-16, D-15]
`lark.spec.ts` currently uses `lark-sync-url-input` / `lark-sync-submit` testids to trigger an E2E Lark sync. If manual URL paste is hidden, the existing E2E test breaks.  
**Prevention:** Update `lark.spec.ts` to use the new Settings-based sync trigger **in the same PR** as the UI change. D-15 explicitly requires extending `lark.spec.ts`.

### P7: set_config writing cloud_api_key back to JSON [D-07]
The frontend currently sets `config.cloud_api_key` directly in the config object. After the migration, `set_config` in `commands/config.rs` must intercept the key, route it to `secrets::set_api_key`, and clear it from the struct before `state.save_config()`.  
**Prevention:** Unit test for `set_config`: pass AppConfig with non-empty `cloud_api_key`, verify saved JSON has no key field.

### P8: `lark_opts` helper duplicated across command modules [VERIFIED: currently in lib.rs line 56]
`lark_opts` builds a `LarkCliOptions` from config. Both `commands/index.rs` and `commands/lark.rs` need it.  
**Prevention:** Move `lark_opts` as a method on `AppState` in `state.rs` (e.g., `state.lark_opts()`), or as a free function in a `commands/helpers.rs`. Do not inline-duplicate.

### P9: Tasks commands going in wrong module [D-22]
D-22 is explicit: tasks commands (`list_tasks`, `update_task_status`, `delete_task`) go in `commands/library.rs`, NOT a new `commands/tasks.rs`.  
**Prevention:** Follow D-22 literally.

### P10: Keyring crate not in workspace deps [VERIFIED]
`keyring` is not in the workspace `Cargo.toml` or any crate today. Must be added directly to `crates/config/Cargo.toml` only (not workspace-wide).

---

## Code Examples

### From codebase — `#[serde(skip)]` pattern

Already used for `fastembed_cache_dir` [VERIFIED: `crates/config/src/types.rs` lines 141-144]:

```141:144:crates/config/src/types.rs
    /// Runtime-only: absolute directory for the FastEmbed model cache. Set at
    /// startup to `app_data_dir/fastembed_cache` so the model is found
    /// regardless of the process working directory. Not persisted.
    #[serde(skip)]
    pub fastembed_cache_dir: Option<std::path::PathBuf>,
```

Apply the same `#[serde(skip)]` to `cloud_api_key`:

```44:46:crates/config/src/types.rs
    #[serde(default)]
    pub cloud_api_key: String,
```
→ becomes `#[serde(skip)]` (remove `#[serde(default)]`).

### From codebase — E2E mock bypass pattern

[VERIFIED: `src-tauri/src/e2e.rs` lines 22-28]:

```22:28:src-tauri/src/e2e.rs
pub fn apply_e2e_config(config: &mut AppConfig) {
    config.embedding.embedder = EmbedderProvider::Mock;
    config.chat_cfg.chat = ChatProvider::Mock;
    config.embedding.mock_embed_dim = 4;
    config.sync.watch_folders.clear();
    config.lark.lark_identity = "user".to_string();
}
```

For keyring bypass, check `is_e2e_mode()` before any `secrets::*` call. In E2E mode, `cloud_api_key` stays empty and Mock providers are forced by `apply_e2e_config` anyway.

### From codebase — thin command wrapper pattern

[VERIFIED: `src-tauri/src/commands/library.rs`]:

```6:27:src-tauri/src/commands/library.rs
#[tauri::command]
pub fn source_count(state: State<'_, AppState>) -> Result<i64, String> {
    state
        .store
        .list_sources()
        .map(|v| v.len() as i64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<Source>, String> {
    state.store.list_sources().map_err(|e| e.to_string())
}
```

Task commands follow the same shape. The tasks in `lib.rs` [VERIFIED: lines 698-704] are:

```698:704:src-tauri/src/lib.rs
#[tauri::command]
fn delete_task(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.store.delete_task(&id).map_err(|e| e.to_string())
}
```

### From codebase — hook pattern (controlled component)

[VERIFIED: `src/hooks/useChat.ts` lines 15-58]:

```15:58:src/hooks/useChat.ts
export function useChat({ onError }: UseChatOptions = {}) {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  // ...
  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );

  const refreshSessions = useCallback(async () => {
    const list = await listChatSessions();
    setSessions(list);
    return list;
  }, []);
```

`useLibrary` and `useTasks` follow the same pattern: `onError` callback, `useCallback` for each refresh, `useEffect` for initial load.

### From codebase — `index-progress` event stays in App.tsx

The `index-progress` listener is currently wired at the App level [CITED: `03-CONTEXT.md` D-10]. The `LibraryView` currently receives `indexProgress` as a prop — this is already the pattern from Phase 2.

### From codebase — Lark commands currently in lib.rs

[VERIFIED: `src-tauri/src/lib.rs` lines 760-834] — `sync_lark_doc`, `sync_lark_url`, `sync_lark_sheet`, `sync_lark_mail`, `sync_lark_im`. These move verbatim to `commands/lark.rs`, replacing `tauri::State<'_, AppState>` imports from `tauri::State` re-exported from crate root.

### From codebase — `lark_opts` helper [VERIFIED: lib.rs lines 56-61]

```56:61:src-tauri/src/lib.rs
fn lark_opts(cfg: &AppConfig) -> LarkCliOptions<'_> {
    LarkCliOptions::new(
        &cfg.lark.lark_cli_bin,
        LarkIdentity::parse(&cfg.lark.lark_identity),
    )
}
```

Move this to `state.rs` as `AppState::lark_opts(&self) -> LarkCliOptions<'_>` so both `commands/index.rs` and `commands/lark.rs` can call `state.lark_opts()`.

### From codebase — existing lark.spec.ts test to update [VERIFIED]

```1:37:e2e/specs/lark.spec.ts
const E2E_LARK_URL = "https://e2e.feishu.cn/file/e2e-mock-token";
// ...
it("syncs a Feishu URL into the library without live lark-cli", async () => {
  await openNav("library", '[data-testid="library-stats"]');
  const input = await $('[data-testid="lark-sync-url-input"]');
  await setReactInputValue(input, E2E_LARK_URL);
  await clickViaDom('[data-testid="lark-sync-submit"]');
  // ...
});
```

D-16 hides manual URL paste. This test must be rewritten to use the Settings sync trigger. The testids `lark-sync-url-input` / `lark-sync-submit` either become hidden or point to Settings.

---

## Validation Architecture

Test layers per deliverable:

### Deliverable 1: `crates/config/src/secrets.rs` + keyring migration

| Layer | Test | Location |
|-------|------|----------|
| Unit | `set_api_key` → `get_api_key` roundtrip (skip if CI) | `crates/config/src/secrets.rs` `#[cfg(test)]` |
| Unit | `is_e2e_mode()=true` → migration skipped entirely | `crates/config/src/file.rs` |
| Integration | Load config with plaintext `cloud_api_key` → saved file has no key → in-memory config has key populated | `crates/config/tests/migration.rs` |
| Integration | `#[serde(skip)]` roundtrip: config with `cloud_api_key` serializes without it, deserializes with empty default | `crates/config/tests/` |
| E2E | `JARVIS_E2E=1` → no keyring access; Settings does not show API key error | none (covered by existing settings.spec.ts passing) |

**Keyring unit test guard:** Use `#[cfg_attr(ci, ignore)]` or `#[ignore = "requires Windows Credential Manager"]` for tests that call actual keyring. The migration test can use a `FakeSecrets` or just test the JSON-stripping logic separately.

### Deliverable 2: `commands/index.rs` + `commands/lark.rs`

| Layer | Test | Location |
|-------|------|----------|
| Compile | `cargo build` succeeds with commands moved | CI Rust job |
| Integration | `rebuild_index` command delegates to `index_ops::rebuild_all_sources` (no logic change) | `cargo test --workspace` |
| E2E | `lark.spec.ts` updated: Settings-based sync trigger works | `npm run test:e2e:local` |
| E2E | `full-ui.spec.ts` navigation to Library/Tasks views (testids preserved) | D-13, D-26 |

### Deliverable 3: Library + Tasks view extraction

| Layer | Test | Location |
|-------|------|----------|
| Unit (FE) | `src/hooks/useLibrary.ts` — mock `invoke`, assert `refreshSources` sets state | `src/hooks/useLibrary.test.ts` (new) |
| Unit (FE) | `src/hooks/useTasks.ts` — mock `invoke`, assert `refreshTasks` sets state | `src/hooks/useTasks.test.ts` (new) |
| E2E | `library-stats` testid present in Library view | `full-ui.spec.ts` line 19 preserved |
| E2E | `task-list` testid present in Tasks view | `full-ui.spec.ts` line 22 preserved |
| E2E | `source-list` testid works after extraction | `lark.spec.ts` line 27 preserved |

### Deliverable 4: Settings API key UI (clear + masked display)

| Layer | Test | Location |
|-------|------|----------|
| E2E | Settings panel shows masked placeholder when key exists | extend `settings.spec.ts` |
| E2E | 「清除密钥」button calls `clear_api_key` command; placeholder resets | extend `settings.spec.ts` |
| E2E | `JARVIS_E2E=1` → API key section does not error out | existing settings.spec.ts passing |

### Full-phase gate

```
Goal: Library and Tasks extracted; API key in keychain; Lark via Settings sync
Verify:
 - [ ] cargo test --workspace passes (D-28)
 - [ ] npm test passes (D-28)
 - [ ] npm run test:e2e:local passes (D-26)
 - [ ] config.json on disk has no cloud_api_key field after migration
 - [ ] Library/Tasks JSX gone from App.tsx (all data-testid moved with JSX)
 - [ ] commands/index.rs + commands/lark.rs exist; lib.rs generate_handler updated
 - [ ] Manual Lark URL paste hidden/deprecated (D-16)
```

---

## Vertical Slice Recommendation

MVP mode — ordered slices. Each slice is independently testable; earlier slices unblock later ones.

### Slice A: Config/Secrets foundation (unblocks everything else)

**What:** `crates/config/src/secrets.rs`, `#[serde(skip)]` on `cloud_api_key`, `load_config_with_migration`, `providers.rs` unchanged, unit + integration tests.

**Why first:** Providers need a populated `cloud_api_key` at runtime. This slice makes the runtime field correct before the UI changes. Also: the E2E bypass must be verified working before any UI or command changes land on top.

**Files touched:**
- `crates/config/Cargo.toml` — add `keyring = "3"`
- `crates/config/src/secrets.rs` — new file
- `crates/config/src/types.rs` — `#[serde(skip)]` on `cloud_api_key`
- `crates/config/src/file.rs` — `load_config_with_migration` wrapper
- `crates/config/src/lib.rs` — re-export `secrets`
- `src-tauri/src/lib.rs` — call `load_config_with_migration` instead of `load_config`

**Test:** `cargo test -p config`

---

### Slice B: commands/index.rs + commands/lark.rs (SHELL-03)

**What:** Move commands from `lib.rs` → `commands/index.rs` and `commands/lark.rs`. Move `lark_opts` to `state.rs`. Update `commands/mod.rs` and `lib.rs` `generate_handler!`. `index_ops.rs` untouched (D-25).

**Why second:** No logic change — pure relocation. Low risk, establishes the structure that Slice C (library.rs tasks extension) needs. Also cleans `lib.rs` significantly, making D-28 `cargo test` faster to verify.

**Files touched:**
- `src-tauri/src/commands/index.rs` — new
- `src-tauri/src/commands/lark.rs` — new
- `src-tauri/src/commands/library.rs` — add task commands
- `src-tauri/src/commands/mod.rs` — add exports
- `src-tauri/src/state.rs` — add `lark_opts()` method
- `src-tauri/src/lib.rs` — remove moved commands, update `generate_handler!`

**Test:** `cargo build` + `cargo test --workspace`

---

### Slice C: LibraryView + useLibrary extraction (FE-03 part 1)

**What:** Replace Library JSX passthrough stub with full component. Create `useLibrary.ts`. Add `src/types/library.ts`. Extend `src/lib/tauri.ts` with library/index IPC wrappers. Move all Library JSX from `App.tsx` to `LibraryView.tsx`. Preserve all `data-testid`. Hide manual Lark URL paste per D-16.

**Files touched:**
- `src/views/LibraryView.tsx` — full component (replaces `{children}` stub)
- `src/hooks/useLibrary.ts` — new
- `src/types/library.ts` — new
- `src/lib/tauri.ts` — extend with library/index/lark wrappers
- `src/App.tsx` — remove Library JSX, pass props to `<LibraryView>`
- `e2e/specs/lark.spec.ts` — update to Settings-based sync (D-15, D-16)

**Test:** `npm test` + `npm run test:e2e:local` (lark.spec.ts + full-ui.spec.ts)

---

### Slice D: TasksView + useTasks extraction (FE-03 part 2)

**What:** Replace Tasks JSX passthrough stub with full component. Create `useTasks.ts`. Add `src/types/tasks.ts`. Move all Tasks JSX from `App.tsx` to `TasksView.tsx`. Preserve `task-list`, `task-list-items`, `toggle-task-{id}`.

**Files touched:**
- `src/views/TasksView.tsx` — full component
- `src/hooks/useTasks.ts` — new
- `src/types/tasks.ts` — new
- `src/App.tsx` — remove Tasks JSX, pass props to `<TasksView>`

**Test:** `npm test` + `npm run test:e2e:local` (full-ui.spec.ts task-list testid)

---

### Slice E: Settings API key UI (CFG-02 UX)

**What:** Add `get_api_key_status`, `set_api_key`, `clear_api_key` commands in `commands/config.rs`. Update Settings section in `App.tsx`: masked placeholder when key exists (D-01), 「清除密钥」button (D-04), Lark auth status display in Settings (D-17), sync error inline (D-18).

**Files touched:**
- `src-tauri/src/commands/config.rs` — add 3 API key commands
- `src-tauri/src/commands/mod.rs` — export new commands
- `src-tauri/src/lib.rs` — register new commands in `generate_handler!`
- `src/App.tsx` — Settings section API key UI update
- `src/lib/tauri.ts` — add `getApiKeyStatus`, `setApiKey`, `clearApiKey`
- `e2e/specs/settings.spec.ts` — add API key mask + clear assertions

**Test:** `npm test` + `npm run test:e2e:local` (settings.spec.ts extended)

---

### Slice ordering summary

```
A (secrets foundation)
  ↓
B (command modules)       C (LibraryView)
  ↓                            ↓
  └──────────────── D (TasksView)
                         ↓
                    E (Settings API key UI)
```

Slices B, C, D can be developed in parallel after A lands. E requires B (for new commands) and C (Settings UI changes touch the same App.tsx area).

---

## RESEARCH COMPLETE

**Summary:** Phase 3 has three interlocking concerns:

1. **Keychain (CFG-02/CFG-03):** Add `keyring = "3"` to `crates/config`; add `secrets.rs` with 3 free functions; change `cloud_api_key` to `#[serde(skip)]`; wrap `load_config` with migration logic; guard everything with `is_e2e_mode()`. No changes to `providers.rs` — it reads the runtime field as before.

2. **Command modules (SHELL-03):** Pure relocation — `rebuild_index`, `retry_source`, Cursor sync, watch folder commands → `commands/index.rs`; all Lark IPC → `commands/lark.rs`; task commands → extend `commands/library.rs`. Move `lark_opts` to `state.rs`. `index_ops.rs` untouched per D-25.

3. **View extraction (FE-03):** Replace `LibraryView`/`TasksView` stubs with full controlled components. Two hooks (`useLibrary`, `useTasks`). `index-progress` listener stays in `App.tsx`. Hide manual Lark URL paste; move Lark health to Settings. Preserve all `data-testid`. Update `lark.spec.ts` for new UX.

**Critical risks:** Keyring in E2E (D-03 guard required); `cloud_api_key` lingering in JSON after migration (resave immediately); `lark-sync-url-input` testid in `lark.spec.ts` must be updated when manual paste is hidden (D-16); `lark_opts` duplication between new command modules (move to `state.rs`).
