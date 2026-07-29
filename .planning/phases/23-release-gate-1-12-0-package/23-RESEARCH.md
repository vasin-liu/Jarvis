# Phase 23: Release gate + 1.12.0 package — Research

**Researched:** 2026-07-30  
**Domain:** Windows Tauri release packaging + version gate + changelog  
**Confidence:** HIGH

## Summary

Phases 20–22 shipped readiness API → IPC → Settings UI. Product manifests remain **1.8.0**. Phase 23 is a process gate: write smoke checklist, prove Mock-path automation green, record interactive cold-start PASS, then bump to **1.12.0**, write CHANGELOG, run `tauri build`, and record final install smoke. FastEmbed cold-start authority is the smoke doc — not CI.

## Standard Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Version | `package.json`, `src-tauri/tauri.conf.json` | Both `1.8.0` today → `1.12.0` after gate |
| Build | `npm run tauri build` | Uses `@tauri-apps/cli`; bundles under `src-tauri/target/release/bundle/` |
| Smoke | `docs/release/v1.12.0-smoke.md` | New dir `docs/release/` |
| Changelog | Root `CHANGELOG.md` | Does not exist yet |
| Trust tests | cargo embedder + tauri-app readiness_ + Settings Vitest | No FastEmbed download |

## Architecture Patterns

### Gate sequence (must not reorder)

```
smoke template commit
  → automated trust suite PASS
  → interactive cold-start smoke PASS (fill checklist)
  → bump versions + CHANGELOG
  → tauri build
  → final artifact launch smoke
  → release metadata commit
```

### TRUST-02

E2E uses `JARVIS_E2E=1` + Mock providers (`src-tauri/src/e2e.rs`). Do not add FastEmbed download steps to CI scripts in this phase.

### Artifact location

Tauri 2 on Windows typically emits NSIS/MSI under `src-tauri/target/release/bundle/`. Record exact path in smoke doc after build.

## Don't Do

- Do not bump before smoke Result = PASS
- Do not push tags or create GitHub Release
- Do not change RAG defaults
- Do not require FastEmbed model download in automated verify commands

## Open Questions

None — CONTEXT locks gate order and checkpoint for interactive smoke.
