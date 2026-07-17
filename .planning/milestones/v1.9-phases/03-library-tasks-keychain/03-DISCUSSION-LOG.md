# Phase 3: Library/Tasks + Keychain - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-26
**Phase:** 3-Library/Tasks + Keychain
**Areas discussed:** Keychain UX, Library/Tasks extraction, Lark sync primary path, Command module split

---

## Keychain UX & Migration

| Option | Description | Selected |
|--------|-------------|----------|
| masked_placeholder | Settings shows masked placeholder when key exists in keychain | ✓ |
| show_last4 | Show last 4 characters of stored key | |
| empty_keep | Empty field on save means keep existing key | |

**User's choice:** Masked placeholder (掩码占位符)
**Notes:** User did not want last-4 display or empty-means-keep semantics.

| Option | Description | Selected |
|--------|-------------|----------|
| auto_migrate_startup | Migrate plaintext key to keychain on app startup | ✓ |
| migrate_on_save | Migrate on first save only | |
| prompt_user | Prompt user before migrating | |

**User's choice:** Startup auto-migration

| Option | Description | Selected |
|--------|-------------|----------|
| e2e_skip_keyring | E2E fully skips keyring (no CI credential manager) | ✓ |
| e2e_inmemory | E2E uses in-memory mock store | |
| e2e_real_keyring | E2E uses real Windows Credential Manager | |

**User's choice:** E2E skip keyring

| Option | Description | Selected |
|--------|-------------|----------|
| clear_button | Explicit "Clear API key" button in Settings | ✓ |
| empty_save_clears | Empty + save clears key | |
| no_clear_ui | No clear UI this phase | |

**User's choice:** Clear button

| Option | Description | Selected |
|--------|-------------|----------|
| jarvis_cloud_api_key | service: jarvis, account: cloud_api_key | ✓ |
| com_jarvis_app | service: com.jarvis.app, account: cloud_api_key | |

**User's choice:** service `jarvis`, account `cloud_api_key`

| Option | Description | Selected |
|--------|-------------|----------|
| fail_graceful | Read failure → cloud errors + Settings inline hint | ✓ |
| fail_silent | Treat as no key silently | |
| fail_block_startup | Block app startup | |

**User's choice:** Graceful degradation

| Option | Description | Selected |
|--------|-------------|----------|
| remove_from_json_only | Strip from JSON; runtime field on AppConfig OK | ✓ |
| separate_secrets_struct | SecretsManager only, no field on AppConfig | |
| serde_skip | Keep field with #[serde(skip)] | |

**User's choice:** Remove from JSON; runtime field acceptable

---

## Library/Tasks Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| controlled_like_chat | App holds state; controlled View + hook (Phase 2 pattern) | ✓ |
| self_contained | Hook inside View only | |
| single_hook | Merged useLibraryTasks | |

**User's choice:** Controlled pattern like Chat

| Option | Description | Selected |
|--------|-------------|----------|
| separate_hooks | useLibrary + useTasks | ✓ |
| single_hook | useLibraryTasks | |
| no_hooks | Logic in View files | |

**User's choice:** Separate hooks

| Option | Description | Selected |
|--------|-------------|----------|
| parent_index_events | App listens index-progress, passes props | ✓ |
| hook_listen | useLibrary listens internally | |
| context | React Context for progress | |

**User's choice:** Parent listens, props down

| Option | Description | Selected |
|--------|-------------|----------|
| move_all_ui | Move all Library + Tasks JSX from App | ✓ |
| library_only | Library only; Tasks later | |
| incremental | Lists first, bulk ops later | |

**User's choice:** Full extraction both views

| Option | Description | Selected |
|--------|-------------|----------|
| types_per_domain | src/types/library.ts + tasks.ts | ✓ |
| single_types | Unified types file | |
| inline_view | Types in View files | |

**User's choice:** Per-domain type files

| Option | Description | Selected |
|--------|-------------|----------|
| same_specs | Update existing settings/navigation/full-ui specs | ✓ |
| new_spec | New library.spec.ts / tasks.spec.ts | |
| minimal_e2e | Smoke only | |

**User's choice:** Update existing E2E specs

---

## Lark Sync Primary Path (D-36)

| Option | Description | Selected |
|--------|-------------|----------|
| settings_sync_button | Settings manual sync + background scheduler | ✓ |
| scheduler_only | Scheduler only, no manual button | |
| library_sync | Library as primary sync entry | |

**User's choice:** Settings sync button + scheduler

| Option | Description | Selected |
|--------|-------------|----------|
| extend_lark_spec | Extend lark.spec.ts + full-ui | ✓ |
| full_ui_only | full-ui only | |
| unit_rust_only | Rust tests only | |

**User's choice:** Extend lark.spec.ts + full-ui

| Option | Description | Selected |
|--------|-------------|----------|
| deprecate_url_paste | Hide/remove manual URL paste; lark-cli primary | ✓ |
| keep_both | Keep URL paste as backup | |
| url_oneoff | URL for one-off import only | |

**User's choice:** Deprecate URL paste

| Option | Description | Selected |
|--------|-------------|----------|
| settings_health | Lark auth/health in Settings | ✓ |
| library_health | In Library | |
| both_places | Both Settings and Library | |

**User's choice:** Settings only

| Option | Description | Selected |
|--------|-------------|----------|
| lark_rs_all_lark | All Lark IPC → lark.rs; index → index.rs | ✓ |
| sync_rs_lark | Lark stays in sync.rs | |
| merge_sync_lark | Merge sync + lark | |

**User's choice:** lark.rs for all Lark IPC

| Option | Description | Selected |
|--------|-------------|----------|
| toast_on_fail | Visible error feedback on manual sync failure | ✓ |
| silent_log | Background only | |
| modal_error | Blocking modal | |

**User's choice:** Visible error (inline in Settings per CONTEXT)

| Option | Description | Selected |
|--------|-------------|----------|
| no_scheduler_ui_change | No scheduler config UI changes | ✓ |
| add_interval_ui | New Lark interval setting | |
| enable_toggle | New auto-sync toggle | |

**User's choice:** No scheduler UI changes

---

## Command Module Split

| Option | Description | Selected |
|--------|-------------|----------|
| index_local_cursor | index.rs: local/Cursor; lark.rs: Lark | ✓ |
| index_all_index | index.rs includes Lark | |
| thin_lib_only | Move bodies only, lib.rs registers all | |

**User's choice:** index.rs + lark.rs split

| Option | Description | Selected |
|--------|-------------|----------|
| library_tasks_split | library.rs sources + tasks (no tasks.rs) | ✓ |
| keep_in_lib | Tasks stay in lib.rs | |
| insights_ops | Tasks via insights_ops | |

**User's choice:** Tasks in library.rs

| Option | Description | Selected |
|--------|-------------|----------|
| commands_delegate_index_ops | Thin commands; logic in index_ops.rs | ✓ |
| move_to_commands | Move logic into index.rs | |
| no_change_index_ops | Registration only | |

**User's choice:** Delegate to index_ops.rs

| Option | Description | Selected |
|--------|-------------|----------|
| config_rs_secrets | config.rs + secrets module | ✓ |
| new_secrets_cmd | commands/secrets.rs | |
| inline_lib | Secrets in lib.rs setup | |

**User's choice:** config.rs + secrets.rs module

| Option | Description | Selected |
|--------|-------------|----------|
| extend_library_rs | Extend existing library.rs | ✓ |
| new_files_only | Only new index/lark files | |
| rename_library | Rename to sources.rs | |

**User's choice:** Extend library.rs

| Option | Description | Selected |
|--------|-------------|----------|
| tasks_in_library_rs | All task commands in library.rs | ✓ |
| new_tasks_rs | New commands/tasks.rs | |
| defer_tasks_cmds | Don't migrate task commands | |

**User's choice:** Tasks in library.rs

---

## Claude's Discretion

- Masked placeholder exact copy and password-input edit flow
- `secrets.rs` API shape (trait vs functions)
- Minor hook file organization

## Deferred Ideas

- Separate `commands/tasks.rs` if library.rs grows too large
- Scheduler Lark interval / auto-sync toggle UI
- Nested TS AppConfig mirroring Rust
- Settings view extraction (Phase 5–6)
- Real keyring in E2E CI
