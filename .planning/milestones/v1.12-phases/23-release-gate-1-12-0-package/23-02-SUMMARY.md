---
phase: 23-release-gate-1-12-0-package
plan: 02
subsystem: release
tags: [release, changelog, tauri-build, windows, version-bump]

requires:
  - phase: 23-release-gate-1-12-0-package
    plan: 01
    provides: smoke template + trust suite
provides:
  - package.json / tauri.conf.json 1.12.0
  - CHANGELOG.md
  - Windows MSI + NSIS 1.12.0 bundles
affects:
  - milestone complete / optional tag

requirements-completed:
  - SHIP-01
  - SHIP-02
  - SHIP-03
  - SHIP-04

key-files:
  created:
    - CHANGELOG.md
  modified:
    - package.json
    - src-tauri/tauri.conf.json
    - docs/release/v1.12.0-smoke.md

---

# Phase 23 Plan 02 Summary

After smoke PASS: bumped to **1.12.0**, wrote CHANGELOG, ran `npm run tauri build` successfully.

## Artifacts

- `target/release/bundle/nsis/tauri-app_1.12.0_x64-setup.exe`
- `target/release/bundle/msi/tauri-app_1.12.0_x64_en-US.msi`
- `target/release/tauri-app.exe`

## Commit

- `chore: release 1.12.0 — hardening + Windows package metadata`
