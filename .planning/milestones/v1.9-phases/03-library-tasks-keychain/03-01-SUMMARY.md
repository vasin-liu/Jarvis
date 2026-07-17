---
phase: 03-library-tasks-keychain
plan: "01"
subsystem: config
tags: [keyring, secrets, migration, serde]
requires: []
provides:
  - OS keychain abstraction in crates/config
  - load_config_with_migration with legacy plaintext migration
  - cloud_api_key runtime-only via #[serde(skip)]
affects: [03-05, providers]
tech-stack:
  added: [keyring = "3"]
  patterns: [secrets module in config crate, JARVIS_E2E keychain bypass]
key-files:
  created: [crates/config/src/secrets.rs, crates/config/tests/migration.rs]
  modified: [crates/config/Cargo.toml, crates/config/src/types.rs, crates/config/src/file.rs, crates/config/src/lib.rs, crates/config/src/error.rs, src-tauri/src/lib.rs]
key-decisions:
  - "keyring dep scoped to crates/config only (P10)"
  - "E2E guard via std::env::var JARVIS_E2E in config crate (P1)"
patterns-established:
  - "Secrets: get/set/delete via keyring service jarvis account cloud_api_key"
requirements-completed: [CFG-02, CFG-03]
duration: 25min
completed: 2026-06-27
---

# Phase 03 Plan 01 Summary

**Keychain foundation: cloud API key never persists to config.json; legacy plaintext migrates to OS credential store on first load.**

## Accomplishments
- Added `crates/config/src/secrets.rs` with keyring-backed get/set/delete
- Marked `EmbeddingConfig.cloud_api_key` with `#[serde(skip)]`
- Implemented `load_config_with_migration` with E2E bypass and graceful keyring read failures
- Wired startup in `src-tauri/src/lib.rs` to use migrating loader

## Self-Check: PASSED
- `cargo test -p config` green (7 tests, 1 ignored keychain roundtrip)
