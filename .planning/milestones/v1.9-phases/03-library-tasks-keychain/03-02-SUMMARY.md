---
phase: 03-library-tasks-keychain
plan: "02"
subsystem: shell
tags: [tauri, commands, refactor]
requires:
  - phase: 03-01
    provides: stable config load path
provides:
  - commands/index.rs and commands/lark.rs modules
  - AppState::lark_opts shared helper
  - task + insights commands in commands/library.rs
affects: [03-03, 03-05]
tech-stack:
  added: []
  patterns: [thin Tauri command modules delegating to index_ops]
key-files:
  created: [src-tauri/src/commands/index.rs, src-tauri/src/commands/lark.rs]
  modified: [src-tauri/src/commands/library.rs, src-tauri/src/commands/mod.rs, src-tauri/src/state.rs, src-tauri/src/lib.rs]
key-decisions:
  - "No commands/tasks.rs — tasks live in library.rs (P9)"
  - "lark_opts on AppState with cfg borrow for lifetimes"
patterns-established:
  - "Index/Lark IPC grouped in dedicated command modules (D-20/D-21)"
requirements-completed: [SHELL-03]
duration: 30min
completed: 2026-06-27
---

# Phase 03 Plan 02 Summary

**Relocated index, Lark, task, and insights Tauri commands out of lib.rs monolith into dedicated modules with unchanged IPC names.**

## Accomplishments
- Created `commands/index.rs` (8 index commands)
- Created `commands/lark.rs` (7 Lark commands + helpers)
- Extended `commands/library.rs` with tasks + insights commands
- Moved `lark_opts` to `AppState::lark_opts(&cfg)`

## Self-Check: PASSED
- `cargo build --manifest-path src-tauri/Cargo.toml` succeeds
- `cargo test -p tauri-app` green
