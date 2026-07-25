---
phase: 14-address-tech-debt-wiki-reindex-export-preflight-gate-nyquist
reviewed: 2026-07-25T00:35:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src-tauri/src/index_ops.rs
  - src/views/LibraryView.tsx
  - src/views/LibraryView.test.tsx
  - crates/insights/src/wiki_export.rs
  - crates/insights/src/lib.rs
  - src-tauri/src/commands/wiki.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
advisory: true
---

# Phase 14: Code Review Report

**Reviewed:** 2026-07-25T00:35:00Z  
**Depth:** standard  
**Files Reviewed:** 6  
**Status:** clean  
**Mode:** ADVISORY (no code fixes applied)

## Summary

Phase 14 closes two audit tech-debt items with surgical, well-tested changes: (1) WikiPage reindex soft-skips with `Ok(false)` **before** `delete_chunks_for_source`, and Library hides retry for `wiki_page`; (2) `wiki_export_preflight` shares the `InsightsError::WikiDisabled` gate with `export_wiki_zip`, wired through a thin Tauri command. No critical or warning defects found. Residual notes are intentional plan deferrals / hardening suggestions only.

## Focus checklist

| Focus | Result | Notes |
|-------|--------|-------|
| Soft skip before chunk wipe (D-01 / T-14-01) | **pass** | Early `if WikiPage { return Ok(false) }` at L545–548, above `delete_chunks_for_source` |
| No `mark_failed` for WikiPage | **pass** | Fail-closed arm removed; match arm is `unreachable!` |
| Bulk / retry treat `Ok(false)` as skip (D-05) | **pass** | Existing consumers increment `skipped`; unit asserts `report.skipped >= 1` |
| Library hide retry kind-only (D-03) | **pass** | `s.kind !== "wiki_page"`; Vitest covers hide + local still shown |
| Preflight WikiDisabled gate (D-06/D-07) | **pass** | Crate helper + Display parity test; cmd passes `cfg.wiki.enabled` |
| IPC thinness | **pass** | `wiki_export_preflight_cmd` maps `map_err(|e| e.to_string())` |
| Secrets / injection / path traversal | **pass** | No new path inputs; `wiki_root` still from config parent |
| Tests at appropriate layer | **pass** | Rust unit + Vitest; E2E waived per D-04/D-15 |

## Critical Issues

None.

## Warnings

None.

## Info

### IN-01: Match-arm `unreachable!` panics if early soft-skip is removed

**File:** `src-tauri/src/index_ops.rs:658`  
**Issue:** Exhaustiveness uses `SourceKind::WikiPage => unreachable!(...)`. Correct today because the early return always fires first. If a future edit deletes the early return but leaves the arm, WikiPage reindex would panic after chunk delete (worst case: wipe then panic).  
**Fix (advisory):** Prefer `return Ok(false)` in the match arm instead of `unreachable!` so a broken guard still soft-skips without panicking (chunks would already be wiped in that broken ordering — the early return remains the real fix).

### IN-02: Pre-existing Failed `wiki_page` rows have no heal path

**Files:** `src-tauri/src/index_ops.rs:545-548`, `src/views/LibraryView.tsx:251-261`  
**Issue:** Soft skip leaves status untouched; UI hides retry. Rows marked Failed by the old stub stay Failed until the user removes the source. Explicitly out of scope (plan: do not heal Failed rows).  
**Fix:** Document in release notes, or a later one-shot migration / “clear failed wiki” action if users report stuck rows.

### IN-03: Disabled preflight now errors (closes Phase 12 IN-02)

**Files:** `crates/insights/src/wiki_export.rs:35-39`, `src-tauri/src/commands/wiki.rs:44-50`  
**Issue:** Preflight when `wiki.enabled=false` returns `Err(WikiDisabled)` → English IPC string via `to_string()`. Library hides「导出 Wiki」when disabled, so the happy path is unchanged; bypass / race surfaces English on the error bar (same as export). Matches D-07/D-08.  
**Fix:** None required for Phase 14; optional Chinese map at Tauri boundary remains a product polish item from Phase 12 IN-01.

## Per-file notes

| File | Assessment |
|------|------------|
| `src-tauri/src/index_ops.rs` | Soft skip correctly ordered; unit test proves non-Failed + chunk count preserved + skipped; kind coercion seed is acceptable Discretion |
| `src/views/LibraryView.tsx` | Kind-only hide (not `wiki.enabled`); mirrors compile hide pattern without coupling |
| `src/views/LibraryView.test.tsx` | Hide + local still-shown coverage; sufficient for D-03 |
| `crates/insights/src/wiki_export.rs` | Helper is minimal gate-then-scan; disabled/enabled unit tests lock D-06/D-07/D-09 |
| `crates/insights/src/lib.rs` | Barrel re-export correct |
| `src-tauri/src/commands/wiki.rs` | Thin; parity with `export_wiki_zip_cmd` config read |

## Threat / decision cross-check

| Decision / threat | Disposition |
|-------------------|-------------|
| D-01 soft skip before delete | Mitigated — early return + regression test |
| D-02 silent skip (no toast) | Mitigated — no FE error path added for skip |
| D-03 hide retry kind-only | Mitigated — Vitest |
| D-05 bulk skipped not abort | Mitigated — existing `Ok(false)` path + assert |
| Chunk wipe hazard (T-14-01) | Mitigated — skip before `delete_chunks_for_source` |
| D-06/D-07 WikiDisabled parity | Mitigated — shared variant + Display assert |
| D-08 FE unchanged | Mitigated — no useLibrary / Library export edits in 14-02 |
| D-09 unit-only preflight | Mitigated — crate tests; no Vitest/E2E |
| Phase 12 IN-02 ungated preflight | Closed by 14-02 |
| D-04/D-15 E2E waiver | Honored — no new `e2e/specs` |

## What looks solid

- Soft skip is the minimal correct fix for the chunk-wipe + Failed-stub double hazard
- UI hide + IPC soft skip is the right defense-in-depth split
- Preflight gate reuses existing error contract instead of inventing strings/variants
- TDD RED→GREEN commits present for both plans (`14-01`, `14-02`)
- Docs-only 14-03 correctly excluded from source review scope

## Verdict

**APPROVE** — 0 CRITICAL, 0 WARNING. Safe to merge; INFO items are residual debt / hardening only.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0     | pass   |
| WARNING  | 0     | pass   |
| INFO     | 3     | note   |

Verdict: APPROVE — no CRITICAL or WARNING issues.
