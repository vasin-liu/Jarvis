# Phase 03 Plan 05 — Summary

**Plan:** 03-05 (keychain UX + phase gate)  
**Status:** Complete

## Goal

Deliver CFG-02 user-facing API key keychain UX, D-18 inline Lark sync errors in Settings, and wire dedicated IPC commands for secret management without persisting plaintext in `config.json`.

## Changes

### Backend (`src-tauri/src/commands/config.rs`)

- Added `get_api_key_status` → `{ hasKey: bool }` (camelCase; never returns key value)
- Added `set_api_key` / `clear_api_key` (E2E no-ops via `is_e2e_mode()`)
- Modified `set_config` to route non-empty `cloud_api_key` to keychain, strip before save, repopulate runtime field
- Registered commands in `commands/mod.rs` and `lib.rs` `generate_handler!`

### Frontend

- **`src/lib/tauri.ts`:** `getApiKeyStatus`, `setApiKey`, `clearApiKey` wrappers
- **`src/App.tsx`:**
  - Password input `data-testid="cloud-api-key-input"` with masked placeholder `••••••••（已保存）` when key exists
  - 「清除密钥」 button `data-testid="clear-api-key"`
  - Save flow calls `setApiKey` for typed keys; `set_config` payload omits plaintext
  - Lark sync inputs relocated from Library → Settings Lark panel
  - Inline Lark sync errors via `data-testid="lark-sync-error"`

### E2E

- Extended `e2e/specs/settings.spec.ts` (API key section + clear button)
- Updated `lark.spec.ts` and `full-ui.spec.ts` for Settings-based Lark sync (D-15)

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p config` | ✅ 7 passed, 1 ignored |
| `cargo test -p tauri-app` | ✅ 4 passed |
| `cargo test --workspace` (skip hung) | ⚠️ 1 pre-existing failure: `store::schema::tests::migrate_v1_to_v2_adds_chat_tables`; skipped hung: `extract_tasks_replaces_previous`, `summary_and_tasks_crud` |
| `npm test` | ✅ 11 passed |
| `npx tsc --noEmit` | ✅ clean |
| Full E2E suite | Not run (user scoped verify to cargo + npm) |

## Decisions satisfied

- **D-01:** Masked placeholder, no last-4
- **D-03:** E2E never touches keychain
- **D-04:** Clear API key button
- **D-07 / P7:** `set_config` strips + routes secret
- **D-18:** Inline Lark sync error in Settings
- **D-19:** No scheduler UI changes
