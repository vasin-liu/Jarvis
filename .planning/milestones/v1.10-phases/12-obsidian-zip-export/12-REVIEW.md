---
phase: 12-obsidian-zip-export
reviewed: 2026-07-23T11:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - crates/insights/src/wiki_export.rs
  - crates/insights/src/error.rs
  - crates/insights/src/wiki.rs
  - crates/insights/src/lib.rs
  - Cargo.toml
  - crates/insights/Cargo.toml
  - src-tauri/src/commands/wiki.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
  - src/types/ipc.ts
  - src/lib/tauri.ts
  - src/hooks/useLibrary.ts
  - src/views/LibraryView.tsx
  - src/App.tsx
  - src/AppShell.tsx
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
advisory: true
---

# Phase 12: Code Review Report

**Reviewed:** 2026-07-23T11:00:00Z  
**Depth:** standard (per-file)  
**Files Reviewed:** 15  
**Status:** issues_found  
**Mode:** ADVISORY (no code fixes applied)

## Summary

Phase 12 delivers path-safe Obsidian zip export (WIKI-07): `export_wiki_zip` + `wiki_has_exportable_notes` in `insights`, thin Tauri IPC, and Library「导出 Wiki」with preflight → Save → export. Zip entry naming is Zip-Slip-aware (Normal-only components, `/` separators, symlink skip, inject-only `.obsidian` stub). Empty/disabled hard gates and exact Chinese empty copy match CONTEXT. One warning: dest overwrite is delete-then-rename (brief data-loss window if rename fails).

## Focus checklist

| Focus | Result | Notes |
|-------|--------|-------|
| Zip-Slip / entry path safety | **pass** | `relative_zip_entry_name` rejects non-`Normal` / absolute; tests lock D-16 |
| Path safety (dest + wiki_root) | **pass** | `wiki_root` from `config_path.parent()/wiki`; dest from Save dialog (same trust model as `index_file`) |
| Empty / disabled gates | **pass** | `WikiDisabled` then `WikiEmpty` before any dest write; FE preflight + crate hard reject |
| IPC thinness | **pass** | cmds delegate to crate; no DB; registration in `generate_handler!` |
| Chinese UI strings (CONTEXT) | **pass** | 「导出 Wiki」, 「还没有可导出的笔记，请先生成笔记」exact; success includes path |

## Critical Issues

None.

## Warnings

### WR-01: Dest overwrite deletes existing zip before rename completes

**File:** `crates/insights/src/wiki_export.rs:45-52`  
**Issue:** On success path, an existing `dest_zip` is `remove_file`d, then `rename(temp → dest)`. If `rename` fails after delete (AV lock, permission race, rare FS error), the previous zip is already gone; recovery depends on the leftover `{name}.tmp`. Windows cannot atomically replace via `rename` onto an existing file, so the pattern is understandable, but the failure window is real.  
**Fix (advisory):** Prefer replace-friendly flow where available (e.g. keep prior dest as `.bak` until rename succeeds, or document that `.tmp` is the recovery artifact and surface that path in the error). Add a unit test that simulates rename failure after remove if feasible with a test double / readonly dest parent.

```rust
// current: remove then rename
if dest_zip.exists() {
    fs::remove_file(dest_zip)?;
}
fs::rename(&temp, dest_zip)?;

// safer sketch: stash old, restore on rename failure
```

## Info

### IN-01: Backend empty/disabled errors are English at the IPC boundary

**Files:** `crates/insights/src/error.rs:13-16`, `src-tauri/src/commands/wiki.rs:60-65`, `src/hooks/useLibrary.ts:158-159`  
**Issue:** FE empty path correctly uses D-09 Chinese via preflight. If export hits `WikiEmpty` / `WikiDisabled` (race after preflight, or direct IPC), `map_err(|e| e.to_string())` surfaces Display English (`"wiki has no exportable notes"` / `"wiki is disabled"`) on the red err bar. Defense-in-depth works; copy is inconsistent with D-09.  
**Fix:** Map those variants to the same Chinese strings at the Tauri boundary, or keep FE-only copy and accept English as fallback.

### IN-02: Preflight IPC does not check `wiki.enabled`

**File:** `src-tauri/src/commands/wiki.rs:44-50`  
**Issue:** `wiki_export_preflight_cmd` only calls `wiki_has_exportable_notes`. UI hides the button when disabled; `export_wiki_zip_cmd` hard-rejects `WikiDisabled`. Preflight remains callable when disabled and still reports note presence. Low risk; slightly asymmetric vs compile/export.  
**Fix:** Optionally short-circuit `has_notes: false` or return `WikiDisabled` when `cfg.wiki.enabled` is false.

### IN-03: Soft-notice slot hardcodes `data-testid="wiki-export-done"`

**File:** `src/AppShell.tsx:101-104`  
**Issue:** Generic `notice` prop always renders with wiki-export test id. Correct for Phase 12 (only export sets notice). Future non-wiki notices would inherit a misleading selector for Phase 13 E2E.  
**Fix:** Pass `noticeTestId` from App, or only set the test id when the notice is wiki-export.

### IN-04: Empty-scan does not require regular files (packer skips symlinks)

**File:** `crates/insights/src/wiki_export.rs:17-31` vs `134-137`  
**Issue:** `wiki_has_exportable_notes` treats any dir entry whose path extension is `md` as exportable; packing skips symlinks. A symlink-only `sources/*.md` tree can pass preflight/export gates and produce a zip with stub (+ other regular files like `index.md`) but no note bodies. Compile never creates symlinks; practical risk on Windows is low (symlink privilege). Aligned with Phase 10 flat section scan otherwise.  
**Fix:** In the empty helper, require `file_type().is_file()` and `!is_symlink()` (same policy as packer).

## Per-file notes

| File | Assessment |
|------|------------|
| `wiki_export.rs` | Core export solid; stub inject-only; on-disk `.obsidian` skipped; symlink skip; WR-01 on dest replace |
| `error.rs` | `WikiEmpty` + `Zip` variants appropriate |
| `wiki.rs` | Test helper delegates to public API only — no logic drift |
| `insights/lib.rs` | Module + re-exports correct |
| `Cargo.toml` / `insights/Cargo.toml` | `zip = 7.2`, `default-features = false`, `deflate` — matches plan/MSRV |
| `commands/wiki.rs` | Thin; `WikiExportPreflight { hasNotes }`; IN-02 |
| `commands/mod.rs` / `lib.rs` | Re-export + handler registration present |
| `ipc.ts` / `tauri.ts` | Types + invoke wrappers match camelCase |
| `useLibrary.ts` | Preflight → exact D-09 → `save` → export; cancel no-ops; local YMD default name |
| `LibraryView.tsx` | 「导出 Wiki」after「选择文件索引」; hide when `wiki.enabled !== true`; busy disables |
| `App.tsx` | App-owned busy; soft notice `Wiki 已导出：${path}` (D-07) |
| `AppShell.tsx` | Emerald non-red notice; IN-03 testid coupling |

## Threat / decision cross-check

| Decision / threat | Disposition |
|-------------------|-------------|
| D-10 / D-12 empty hard reject | Mitigated — `WikiEmpty` before create; unit test |
| D-11 FE preflight | Mitigated — `wiki_export_preflight_cmd` + hook |
| D-13–D-15 Obsidian stub | Mitigated — inject-only stub body; skip on-disk `.obsidian` |
| D-16 path-safe entries | Mitigated — Normal-only + `/`; unit `assert_path_safe_names` |
| T-12-02 symlink skip | Mitigated on pack walk; empty helper weaker (IN-04) |
| wiki.enabled gate | UI hide + export `WikiDisabled`; preflight asymmetric (IN-02) |
| E2E wiki.spec.ts | Deferred Phase 13 per CONTEXT — not a Phase 12 defect |

## What looks solid

- Dual empty gate (FE Chinese + crate `WikiEmpty`) and disabled hard reject before any dest write
- Zip-Slip-oriented entry construction + dedicated path-safety unit test
- Inject-only `.obsidian/app.json` with exact `{"legacyEditor":false}`; never mkdir under `wiki_root`
- Temp sibling write with best-effort temp delete on error
- Library toolbar placement, hide-when-disabled, busy disable, emerald success vs red err
- Exact CONTEXT empty string; default `jarvis-wiki-YYYY-MM-DD.zip` via local calendar date
- Thin IPC; no Store/SQLite involvement in export

## Verdict

**WARNING** — 0 CRITICAL, 1 WARNING (dest delete-then-rename). Safe to merge with caution; WR-01 is edge-case data loss on overwrite failure, not Zip-Slip. No critical security bug found; advisory only — no code changes made.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0     | pass   |
| HIGH/WARNING | 1  | warn   |
| INFO     | 4     | note   |
| LOW      | 0     | —      |

**Verdict:** WARNING — 1 WARNING should be considered before merge (non-blocking for Zip-Slip / WIKI-07 gates).

---
*Phase: 12-obsidian-zip-export*  
*Report: `.planning/phases/12-obsidian-zip-export/12-REVIEW.md`*
