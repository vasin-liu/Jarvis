---
phase: 1
slug: scaffold-shell-foundation
status: passed
verified: 2026-06-17
---

# Phase 1 Verification — Scaffold + Shell Foundation

## Requirement Status

| ID | Status | Notes |
|----|--------|-------|
| **FE-01** | partial | Types (`src/types/`), view stubs (`src/views/`), IPC wrappers (`src/lib/tauri.ts`) extracted; `App.tsx` retains all logic, state, and direct `invoke()` calls. Full FE-01 completes Phase 6. |
| **SHELL-01** | partial | `AppState` in `state.rs`; 8 commands in `commands/{config,library,sync}.rs`; `lib.rs` retains ~44 commands + glue. Full SHELL-01 completes Phase 6. |
| **QA-01** | met | `smoke.spec.ts` + `navigation.spec.ts` pass via `test:e2e:local` |
| **QA-03** | met | `cargo test` (src-tauri) + `npm test` + `npm run build` green |

## Structural Checks

- **IPC registry:** `generate_handler!` lists **52** command identifiers (unchanged names)
- **index_ops.rs:** Not modified by Phase 1 shell extraction (pre-existing branch diffs may exist from prior work)
- **E2E env:** `JARVIS_E2E=1`, `JARVIS_E2E_FIXTURE` preserved via `test:e2e:local`
- **data-testid:** Unchanged on `App.tsx` DOM elements (nav-chat, settings-panel, index-status, etc.)

## Artifacts Created

### Plan 01-01 (Rust shell)
- `src-tauri/src/state.rs` — `AppState` struct + impl
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/config.rs` — get_config, set_config, get_index_status
- `src-tauri/src/commands/library.rs` — source_count, list_sources, remove_source
- `src-tauri/src/commands/sync.rs` — get_sync_status, run_scheduled_sync_cmd

### Plan 01-02 (Frontend scaffold)
- `src/types/view.ts` — `View` union
- `src/types/ipc.ts` — 18 IPC interfaces
- `src/lib/tauri.ts` — 8 typed wrappers (unused by App.tsx per D-14)
- `src/views/{Chat,Library,Tasks,Memory,Settings}View.tsx` — pass-through stubs

## Zero Behavior Change

Structural refactor only (D-19): no IPC renames, no command signature changes, no testid moves.
