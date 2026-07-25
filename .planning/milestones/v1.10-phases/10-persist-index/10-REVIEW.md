---
phase: 10-persist-index
reviewed: 2026-07-21T10:17:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - crates/insights/src/wiki.rs
  - crates/insights/src/error.rs
  - crates/insights/src/lib.rs
  - crates/insights/Cargo.toml
  - src-tauri/src/commands/wiki.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
findings:
  critical: 1
  warning: 2
  info: 3
  total: 6
status: issues
---

# Phase 10: Code Review Report

**Reviewed:** 2026-07-21T10:17:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues

## Summary

Phase 10 delivers `compile_wiki_for_source` (gates → analyze → D-09 write → D-06…D-08 cleanup → scan-rebuild `index.md` → `wiki://` index) and a thin `compile_wiki_cmd`. Architecture matches plans: Store-only DB access, path-safe slug joins on the compile write path, hash-skip via `ingest::hash_text` of on-disk bytes, and library remove_source pair on cleanup.

The main defect is **D-07 ownership peek asymmetry**: pages are written with YAML-escaped `sources` URIs, but cleanup matches the raw URI. On Windows (primary platform), local-file URIs contain `\`, so stale generated cleanup silently never fires. Secondary risks: URI substring false-positives in ownership matching, and the Phase 09 blind writer still lacking path confinement.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Cleanup ownership peek ignores YAML escapes (Windows local paths broken)

**File:** `crates/insights/src/wiki.rs:336-345`
**Issue:** `build_frontmatter` persists `sources` via `yaml_escape_double_quoted` (so `\` → `\\`, `"` → `\"`). `frontmatter_sources_contains` builds the needle as `format!("\"{source_uri}\"")` with **no** matching escape. Local ingest URIs come from `canonical_uri` (`path.to_string_lossy()`), which on Windows is typically `D:\…` or `\\?\D:\…`. Written frontmatter then contains doubled backslashes; the raw-URI needle never matches → `cleanup_stale_wiki_pages_for_source` skips every owned page → D-06…D-08 / `cleaned` stay ineffective in production while `/tmp/…` tests stay green.
**Fix:** Escape the needle the same way as write:

```rust
fn frontmatter_sources_contains(raw: &str, source_uri: &str) -> bool {
    let Some(front) = peek_frontmatter(raw) else {
        return false;
    };
    let needle = format!("\"{}\"", yaml_escape_double_quoted(source_uri));
    front.lines().any(|line| {
        let t = line.trim();
        t.starts_with("sources:") && t.contains(&needle)
    })
}
```

Add a unit test with a Windows-style URI (`D:\\docs\\a.md` or `C:\\Users\\x\\note.md`) proving round-trip ownership match + stale cleanup.

## Warnings

### WR-01: Ownership match is substring-based (prefix URI false positive)

**File:** `crates/insights/src/wiki.rs:336-345`
**Issue:** Even after escape alignment, `t.contains(&needle)` treats a shorter quoted URI as matching a longer one (e.g. needle `"…/a.md"` matches `sources: ["…/a.md.bak"]`). Compiling the shorter source can delete another source’s generated pages (FS + Store) when the slug is outside the current draft set — weakens T-10-06.
**Fix:** Prefer token-boundary matching (split the `sources:` list on commas / parse quoted entries) or require the quoted entry to be followed by `,` or `]`:

```rust
// after finding needle index, ensure next char is ',' or ']' (trim spaces)
```

Lock with a regression test: page tagged `"/tmp/a.md.bak"` must not clean when compiling `"/tmp/a.md"`.

### WR-02: Blind `write_wiki_pages_to_dir` still joins slugs without path confinement

**File:** `crates/insights/src/wiki.rs:53-63`
**Issue:** Compile path correctly uses `resolve_wiki_page_path` (rejects `..` / absolute / non-Normal components — T-10-01). The retained Phase 09 writer does `wiki_root.join(format!("{}.md", page.slug))` with no guard. Slugs from `render_wiki_pages`/`slugify` are safe today, but any future caller passing a crafted `WikiCompileResult` can escape `wiki_root`. Dual-writer footgun called out in Plan 01 decisions.
**Fix:** Route writes through `resolve_wiki_page_path` (or share one helper) inside `write_wiki_pages_to_dir` as well.

## Info

### IN-01: FS delete before Store delete can orphan index rows on Store error

**File:** `crates/insights/src/wiki.rs:220-226`
**Issue:** Order is `remove_file` then `delete_chunks_for_source` / `delete_source`. Plan 02 accepts “Store error after FS delete → Err; re-compile recovers,” but a failed Store step leaves a missing file with a live `wiki://` row until the next successful cleanup/compile.
**Fix:** Optional hardening later: delete Store first, or best-effort continue cleaning remaining stale paths and aggregate errors. Not blocking given documented disposition.

### IN-02: `write_wiki_pages_to_dir` still writes `compiled.index_markdown`

**File:** `crates/insights/src/wiki.rs:62`
**Issue:** Compile correctly uses `rebuild_index_md_from_disk` (D-01). The blind writer still dumps analysis-scoped `index_markdown`, which is wrong as a multi-source vault catalog if reused. Intentional for Phase 09 tests — document or deprecate in API comments so Phase 11 callers do not use it for vault persist.

### IN-03: Thin Tauri command and error/re-exports look correct

**Files:** `src-tauri/src/commands/wiki.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `crates/insights/src/error.rs`, `crates/insights/src/lib.rs`, `crates/insights/Cargo.toml`
**Issue:** No defects found at standard depth. `compile_wiki_cmd` mirrors memory thickness (wiki_root = `config_path.parent()/wiki`, `cfg.wiki.enabled`, `map_err(to_string)`), no progress events (D-17), registered in `generate_handler!`. Path deps `chunker`/`embedder`/`indexer`/`ingest` present. `WikiDisabled` / `WikiPageInput` / `Index` variants match Plan 01. Chat/embedder Arcs are cloned under short mutex locks (no lock held across await).

---

## Threat model cross-check

| Threat | Disposition in code | Review note |
|--------|---------------------|-------------|
| T-10-01 path join | Mitigated on compile write/scan via `resolve_wiki_page_path` + stem checks | Residual on blind writer (WR-02) |
| T-10-02 WikiPage input | Hard gate before analyze | OK |
| T-10-06 cleanup ownership | Intended mitigate via sources+generated+slug∉set | Broken for escaped Windows URIs (CR-01); substring risk (WR-01) |
| T-10-07 user-edit | D-09 skip + D-08 no-delete + D-12 index on-disk | Covered by tests |
| T-10-08 orphan chunks | `delete_chunks_for_source` then `delete_source` | OK when ownership matches |
| T-10-04 Store ownership | No rusqlite in insights | OK |

## What looks solid

- D-15/D-16 gates before FS; fail-closed analyze preserved
- Scan-rebuild D-01…D-05 locked by `rebuild_index_md_*` tests
- Hash-skip idempotency + D-10 digest omission on compile output
- User-edit sacred path + still-index on-disk body
- No UI/E2E scope creep (deferred Phase 11/13)

---

_Reviewed: 2026-07-21T10:17:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
