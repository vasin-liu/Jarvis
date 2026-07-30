---
phase: 23-release-gate-1-12-0-package
verified: 2026-07-30T10:30:00+08:00
status: passed
score: 10/10 must-haves verified
---

# Phase 23 Verification: Release gate + 1.12.0 package

**Verifier:** orchestrator (chain execute)  
**Date:** 2026-07-30

## ROADMAP Success Criteria

| # | Criterion | Result | Evidence |
|---|-----------|--------|----------|
| 1 | Smoke doc exists; cold-start PASS before bump | ✅ | `docs/release/v1.12.0-smoke.md` Result PASS; bump after operator PASS |
| 2 | package.json + tauri.conf.json are 1.12.0 after PASS | ✅ | Both `"version": "1.12.0"` |
| 3 | CHANGELOG covers 1.9–1.11 + 1.12.0 hardening | ✅ | Root `CHANGELOG.md` |
| 4 | tauri build Windows artifact; Mock tests green; no CI FastEmbed | ✅ | NSIS+MSI 1.12.0; Plan 01 trust suite; FastEmbed network test ignored |

## Requirements

| ID | Requirement | Status | Notes |
|----|-------------|--------|-------|
| SHIP-01 | Smoke checklist executed before bump | ✅ | Operator PASS 2026-07-30 |
| SHIP-02 | Version bump to 1.12.0 | ✅ | package.json + tauri.conf.json |
| SHIP-03 | CHANGELOG.md | ✅ | Created |
| SHIP-04 | tauri build Windows artifact | ✅ | NSIS + MSI under target/release/bundle |
| TRUST-01 | Mock-path tests green; RAG defaults untouched | ✅ | Plan 01 suite; no RAG edits in phase |
| TRUST-02 | CI/E2E no FastEmbed download | ✅ | Network FastEmbed test ignored; smoke is authority |

## Must-Haves

| # | Must-have | Verified |
|---|-----------|----------|
| 1 | Smoke template exists | ✅ |
| 2 | Trust suite green pre-bump | ✅ |
| 3 | No premature bump (1.8.0 until PASS) | ✅ |
| 4 | Smoke Result PASS before bump | ✅ |
| 5 | Versions 1.12.0 | ✅ |
| 6 | CHANGELOG.md | ✅ |
| 7 | `npm run tauri build` exit 0 | ✅ |
| 8 | NSIS 1.12.0 artifact | ✅ |
| 9 | MSI 1.12.0 artifact | ✅ |
| 10 | Release metadata committed | ✅ |

## Automated Verification

```
cargo test -p embedder -- --test-threads=4
# 9 passed; 1 ignored

cargo test -p tauri-app readiness_ -- --test-threads=4
# 3 passed

npx vitest run src/views/SettingsView.test.tsx
# 7 passed

npm run tauri build
# exit 0 — bundles at:
#   target/release/bundle/nsis/tauri-app_1.12.0_x64-setup.exe
#   target/release/bundle/msi/tauri-app_1.12.0_x64_en-US.msi
```

## Gaps

None blocking. Optional broken-cache Failed path left unchecked in smoke doc.
