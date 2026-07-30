---
phase: 21-readiness-ipc
plan: 02
subsystem: api
tags: [rust, tauri, tdd, green, embedder, readiness, ipc, deferred-embedder]

# Dependency graph
requires:
  - phase: 21-readiness-ipc
    plan: 01
    provides: RED unit tests for readiness_from_watch (None→Ready, Pending, Failed)
  - phase: 20-deferredembedder-readiness-api
    provides: DeferredEmbedder, EmbedderReadyState, ready_state()
provides:
  - AppState.deferred_embedder watch handle on FastEmbed cold-start path
  - readiness_from_watch pure helper + embedder_readiness method
  - get_embedder_readiness IPC with EmbedderReadinessView camelCase JSON
affects:
  - 22-settings-readiness-ui (React invoke consumer)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Typed Arc<DeferredEmbedder> cloned before trait cast to retain watch handle (D-01)"
    - "Non-blocking ready_state() in IPC path — never embed().await (D-02, Pitfall 4)"
    - "Cold-start-only deferred_embedder; reload_providers sync build_embedder unchanged (D-04)"

key-files:
  created: []
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/config.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "D-01: deferred_embedder = Some(deferred.clone()) before Arc<dyn Embedder> cast"
  - "D-02: readiness_from_watch delegates to ready_state() without blocking"
  - "D-03: EmbedderReadinessView maps pending|ready|failed with optional message"
  - "D-04: reload_providers body untouched; field comment documents cold-start-only watch"
  - "D-05: No React, E2E, Vitest, or version bump — shell IPC only"

patterns-established:
  - "readiness_view in config.rs mirrors index_status_view separation"
  - "get_embedder_readiness registered adjacent to get_index_status in generate_handler!"

requirements-completed:
  - BOOT-02

coverage:
  - id: D1
    description: "readiness_from_watch(None) returns Ready for non-deferred/E2E path"
    requirement: BOOT-02
    verification:
      - kind: unit
        ref: "src-tauri/src/state.rs#no_deferred_is_ready"
        status: pass
    human_judgment: false
  - id: D2
    description: "readiness_from_watch on pending DeferredEmbedder returns Pending"
    requirement: BOOT-02
    verification:
      - kind: unit
        ref: "src-tauri/src/state.rs#deferred_pending"
        status: pass
    human_judgment: false
  - id: D3
    description: "readiness_from_watch after fail() returns Failed with message"
    requirement: BOOT-02
    verification:
      - kind: unit
        ref: "src-tauri/src/state.rs#deferred_failed"
        status: pass
    human_judgment: false
  - id: D4
    description: "get_embedder_readiness IPC compiles and registers in Tauri handler"
    requirement: BOOT-02
    verification:
      - kind: unit
        ref: "cargo check -p tauri-app"
        status: pass
    human_judgment: false

duration: 12min
completed: 2026-07-29
status: complete
---

# Phase 21 Plan 02: Readiness IPC GREEN Summary

**AppState retains typed DeferredEmbedder watch on FastEmbed cold-start; get_embedder_readiness IPC exposes non-blocking pending|ready|failed status for Phase 22 UI**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-07-29T11:42:00Z
- **Completed:** 2026-07-29T11:55:00Z
- **Tasks:** 5
- **Files modified:** 4

## Accomplishments

- Added `readiness_from_watch` helper and `AppState::embedder_readiness()` delegating to non-blocking `ready_state()`
- Wired `deferred_embedder: Option<Arc<DeferredEmbedder>>` on FastEmbed cold-start path; `None` for E2E/Mock/Ollama/Cloud
- Implemented `EmbedderReadinessView` + `get_embedder_readiness` IPC command in `commands/config.rs`
- Registered handler in `mod.rs` re-export and `lib.rs` `generate_handler!`
- All three Plan 21-01 RED tests now GREEN

## Task Commits

1. **Tasks 1–5: GREEN implementation** — (see commit below)

## Files Created/Modified

- `src-tauri/src/state.rs` — `deferred_embedder` field, `init_state` wiring, `readiness_from_watch`, `embedder_readiness`
- `src-tauri/src/commands/config.rs` — `EmbedderReadinessView`, `readiness_view`, `get_embedder_readiness`
- `src-tauri/src/commands/mod.rs` — re-export `get_embedder_readiness`
- `src-tauri/src/lib.rs` — import + `generate_handler!` registration

## Decisions Made

None — followed plan as specified. F-01 applied: field + helper + `init_state` wiring in same edit wave.

## Deviations from Plan

None - plan executed exactly as written

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Test Results

```
cargo test -p tauri-app readiness_ -- --test-threads=4 --nocapture
```

- **Exit code:** 0
- **Tests:** 3 passed (`no_deferred_is_ready`, `deferred_pending`, `deferred_failed`)

```
cargo check -p tauri-app
```

- **Exit code:** 0 — Finished

## Next Phase Readiness

- Phase 22 may invoke `get_embedder_readiness` from React Settings UI
- IPC contract stable: `{ state: "pending"|"ready"|"failed", message: string|null }` camelCase
- E2E Mock path reports `ready` via `deferred_embedder: None`

---
*Phase: 21-readiness-ipc*
*Completed: 2026-07-29*
