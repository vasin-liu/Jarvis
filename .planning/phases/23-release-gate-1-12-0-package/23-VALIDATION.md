---
phase: 23
slug: release-gate-1-12-0-package
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-30
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test + Vitest 3 |
| **Config file** | workspace Cargo.toml / vite.config.ts |
| **Quick run command** | `cargo test -p tauri-app readiness_ -- --test-threads=4` |
| **Full suite command** | `cargo test -p embedder -- --test-threads=4` + readiness_ + Settings Vitest |
| **Estimated runtime** | ~2–10 min (tests); tauri build much longer |

---

## Sampling Rate

- **After Plan 01 tasks:** Run full trust suite above
- **After Plan 02 bump:** Confirm version strings 1.12.0
- **After Plan 02 build:** Confirm `src-tauri/target/release/bundle` exists
- **Max feedback latency:** 600 seconds for tests; build may exceed 30 min

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------------|-----------|-------------------|-------------|--------|
| 23-01-01 | 01 | 1 | SHIP-01 | — | N/A | file | `Test-Path docs/release/v1.12.0-smoke.md` | ✅ | ⬜ pending |
| 23-01-02 | 01 | 1 | TRUST-01/02 | — | No FastEmbed download in CI cmds | unit | cargo + vitest suite | ✅ | ⬜ pending |
| 23-02-01 | 02 | 2 | SHIP-01 | — | N/A | manual smoke | checklist Result PASS | ✅ | ⬜ pending |
| 23-02-02 | 02 | 2 | SHIP-02/03 | — | N/A | file | version + CHANGELOG | ✅ | ⬜ pending |
| 23-02-03 | 02 | 2 | SHIP-04 | — | N/A | build | `npm run tauri build` | ✅ | ⬜ pending |

---

## Wave 0 Requirements

Existing infrastructure covers automated trust. Interactive smoke is intentionally manual.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cold-start window interactive ~5s | SHIP-01 | Needs real Windows GUI + FastEmbed | Fill `docs/release/v1.12.0-smoke.md` |
| Settings pending→ready | SHIP-01 | Live deferred FastEmbed | Open Settings during cold start |
| Installer launches | SHIP-04 | Install/extract GUI | After `tauri build` |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or explicit manual checkpoint
- [x] Sampling continuity maintained
- [x] Wave 0 covers MISSING refs (none)
- [x] No watch-mode flags
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-30
