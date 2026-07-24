# Phase 14: Tech debt closeout - Pattern Map

**Mapped:** 2026-07-24
**Files analyzed:** 9
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/index_ops.rs` | service | batch / request-response | same file: `Ok(false)` skip consumers + `reindex_source` (anti-pattern: fail-closed arms) | exact (surgical edit) |
| `src-tauri/src/commands/wiki.rs` | controller | request-response | same file: `export_wiki_zip_cmd` + `compile_wiki_cmd` | exact |
| `crates/insights/src/wiki_export.rs` | service | file-I/O / request-response | same file: `export_wiki_zip` enabled gate + `export_wiki_rejects_when_disabled` | exact |
| `crates/insights/src/error.rs` | model | — (reuse only) | `InsightsError::WikiDisabled` | exact (no edit expected) |
| `crates/insights/src/lib.rs` | config | — (re-export if helper added) | existing `pub use wiki_export::{...}` | exact |
| `src/views/LibraryView.tsx` | component | request-response (IPC via props) | same file: hide「生成笔记」for `wiki_page` | exact |
| `src/views/LibraryView.test.tsx` | test | — | same file: `hides compile for indexed wiki_page even when enabled` | exact |
| `.planning/phases/12-obsidian-zip-export/12-VALIDATION.md` | config/docs | — | `.planning/phases/11-library-settings-ui/11-VALIDATION.md` frontmatter | role-match |
| `.planning/phases/13-e2e-citation-trust/13-VALIDATION.md` | config/docs | — | Phase 11 VALIDATION frontmatter + Phase 12/13 VERIFICATION as evidence cite | role-match |

**Out of scope / no code change:** `src/hooks/useLibrary.ts` (D-08 — existing `catch` → `reportError`); `.planning/ROADMAP.md` Coverage WIKI-08/09 already Complete (confirm only).

---

## Pattern Assignments

### `src-tauri/src/index_ops.rs` (service, batch / request-response)

**Analog:** same file — soft-skip consumer + current WikiPage stub (replace) + fail-closed `Ok(false)` arms (do **not** copy for soft skip)

**Critical hazard (from RESEARCH):** `delete_chunks_for_source` runs **before** the `match` today. Soft skip must **early-return `Ok(false)` before** that call.

**Imports / test deps pattern** (lines 681–686):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use embedder::MockEmbedder;
    use lark::FakeRunner;
    use std::fs;
```

**Core skip-consumer pattern** — bulk does not abort (lines 125–135):
```rust
            Ok(false) => {
                skipped += 1;
                on_progress(IndexProgressEvent {
                    phase: phase.to_string(),
                    current,
                    total,
                    source_title: source.title.clone(),
                    outcome: Some("skipped".into()),
                    message: source.error.clone(),
                });
            }
```

**Single-retry skip consumer** (lines 501–510):
```rust
        Ok(false) => {
            on_progress(IndexProgressEvent {
                phase: "retry".into(),
                current: 1,
                total: 1,
                source_title: source.title.clone(),
                outcome: Some("skipped".into()),
                message: source.error.clone(),
            });
            (0, 0, 1)
        }
```

**Destructive order today** — soft skip must precede this (lines 555–557):
```rust
    store
        .delete_chunks_for_source(&source.id)
        .map_err(|e| e.to_string())?;
```

**Current WikiPage stub to replace** (lines 653–656) — **anti-pattern** (`mark_failed` + wipe already done):
```rust
        SourceKind::WikiPage => {
            mark_failed(store, source, "wiki page reindex not implemented")?;
            Ok(false)
        }
```

**Fail-closed skip (do NOT copy for WikiPage soft skip)** (lines 561–563, 638–640):
```rust
            if !std::path::Path::new(&source.uri).is_file() {
                mark_failed(store, source, "local file missing")?;
                return Ok(false);
            }
            // ...
            if text.trim().is_empty() {
                mark_failed(store, source, "memory text missing")?;
                return Ok(false);
            }
```

**Prescribed soft-skip shape** (Discretion: inline `if` vs helper — both OK):
```rust
async fn reindex_source(...) -> Result<bool, String> {
    if source.kind == SourceKind::WikiPage {
        return Ok(false); // soft skip: no mark_failed, no delete_chunks
    }
    // existing memory_text / delete_chunks / match...
}
```

**`mark_failed` helper** — stop calling for WikiPage (lines 666–678):
```rust
fn mark_failed(store: &Store, source: &Source, error: &str) -> Result<(), String> {
    let failed = Source {
        // ...
        status: IndexStatus::Failed,
        error: Some(error.to_string()),
        // ...
    };
    store.upsert_source(&failed).map_err(|e| e.to_string())
}
```

**Unit test analog** — seed + assert status/chunks (lines 717–749 style):
```rust
    #[tokio::test]
    async fn rebuild_local_file_source() {
        // tempfile Store + MockEmbedder + index_path / rebuild_all_sources
        assert_eq!(report.indexed, 1);
        assert!(store.count_chunks().unwrap() > 0);
    }
```

**Wave 0 test must assert (RESEARCH Pitfall 1):** `status != Failed`, **chunk count preserved**, and bulk `skipped` increments / does not abort. Package: `tauri-app` lib name `tauri_app_lib` (`src-tauri/Cargo.toml`).

---

### `src-tauri/src/commands/wiki.rs` (controller, request-response)

**Analog:** same file — `export_wiki_zip_cmd` / `compile_wiki_cmd` (thin shell + `cfg.wiki.enabled` + `map_err(|e| e.to_string())`)

**Imports pattern** (lines 1–7):
```rust
use insights::{
    compile_wiki_for_source, export_wiki_zip, wiki_has_exportable_notes, WikiCompileSummary,
};
use serde::Serialize;
use tauri::State;

use crate::state::AppState;
```

**Preflight DTO** (lines 9–13):
```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiExportPreflight {
    pub has_notes: bool,
}
```

**Current ungated preflight** — add enabled gate (lines 44–50):
```rust
pub async fn wiki_export_preflight_cmd(
    state: State<'_, AppState>,
) -> Result<WikiExportPreflight, String> {
    let wiki_root = wiki_root_from_state(&state)?;
    Ok(WikiExportPreflight {
        has_notes: wiki_has_exportable_notes(&wiki_root),
    })
}
```

**Gated export cmd to mirror** (lines 54–65):
```rust
pub async fn export_wiki_zip_cmd(
    dest_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cfg = state.config();
    let wiki_root = wiki_root_from_state(&state)?;
    export_wiki_zip(
        &wiki_root,
        std::path::Path::new(&dest_path),
        cfg.wiki.enabled,
    )
    .map_err(|e| e.to_string())
}
```

**Compile cmd also passes `cfg.wiki.enabled`** (lines 28–40) — same thin-command shape:
```rust
    compile_wiki_for_source(
        state.store.as_ref(),
        state.chat().as_ref(),
        state.embedder().as_ref(),
        &state.chunker,
        &source_id,
        &wiki_root,
        cfg.wiki.enabled,
    )
    .await
    .map_err(|e| e.to_string())
```

**Preferred Discretion (D-09 testability):** call a crate helper e.g. `wiki_export_preflight(&wiki_root, cfg.wiki.enabled)` that returns `Result<bool, InsightsError>`, then map to `WikiExportPreflight { has_notes }`. Keep cmd thin.

---

### `crates/insights/src/wiki_export.rs` (service, file-I/O / request-response)

**Analog:** same file — `export_wiki_zip` gate + `wiki_has_exportable_notes` + `export_wiki_rejects_when_disabled` test

**Imports pattern** (lines 3–10):
```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::{InsightsError, Result};
```

**Empty-tree helper to reuse** (lines 17–32):
```rust
pub fn wiki_has_exportable_notes(wiki_root: &Path) -> bool {
    for dir in ["sources", "entities", "concepts"] {
        // ... any .md under section dirs
    }
    false
}
```

**Enabled gate pattern to copy** (lines 37–43):
```rust
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path, wiki_enabled: bool) -> Result<()> {
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
    if !wiki_has_exportable_notes(wiki_root) {
        return Err(InsightsError::WikiEmpty);
    }
```

**Secondary analog — compile hard gate before side effects** (`crates/insights/src/wiki.rs` lines 88–91):
```rust
    // D-15 / D-16: hard gates before any FS or analyze I/O side effects.
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
```

**Prescribed thin helper** (Discretion name; behavior locked D-06/D-07):
```rust
pub fn wiki_export_preflight(wiki_root: &Path, wiki_enabled: bool) -> Result<bool> {
    if !wiki_enabled {
        return Err(InsightsError::WikiDisabled);
    }
    Ok(wiki_has_exportable_notes(wiki_root))
}
```

**Unit test pattern to copy** (lines 270–281):
```rust
    #[test]
    fn export_wiki_rejects_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let wiki_root = dir.path().join("wiki");
        write_md(&wiki_root.join("sources/foo.md"), "# Foo\n");

        let dest = dir.path().join("disabled.zip");
        let err = export_wiki_zip(&wiki_root, &dest, false).unwrap_err();
        assert!(
            matches!(err, InsightsError::WikiDisabled),
            "expected WikiDisabled, got {err:?}"
        );
        assert!(!dest.exists(), "dest must not be written when disabled");
    }
```

**Compile-reject analog** (`wiki.rs` lines 732–753) — same `matches!(err, InsightsError::WikiDisabled)` assertion style for disabled path.

If helper added: re-export from `crates/insights/src/lib.rs` line 15:
```rust
pub use wiki_export::{export_wiki_zip, wiki_has_exportable_notes};
// → also wiki_export_preflight
```

---

### `crates/insights/src/error.rs` (model — reuse, no change expected)

**Analog:** self — `WikiDisabled` Display string is the IPC contract

**Error / Display pattern** (lines 13–16):
```rust
    #[error("wiki is disabled")]
    WikiDisabled,
    #[error("wiki has no exportable notes")]
    WikiEmpty,
```

Do **not** invent a new variant or Chinese string (D-07/D-08). Tauri boundary: `.to_string()` → `"wiki is disabled"`.

---

### `src/views/LibraryView.tsx` (component, request-response)

**Analog:** same file — hide compile for `wiki_page` (lines 238–248)

**Hide (not disable) pattern:**
```tsx
                    {config?.wiki?.enabled === true && s.kind !== "wiki_page" && (
                      <button
                        type="button"
                        className="btn-ghost text-xs"
                        disabled={busy}
                        data-testid={`wiki-compile-${s.id}`}
                        onClick={() => onCompileWiki(s.id)}
                      >
                        生成笔记
                      </button>
                    )}
```

**Current always-shown retry** — wrap with kind gate only (lines 251–259):
```tsx
                <button
                  type="button"
                  className="btn-ghost text-xs"
                  disabled={busy}
                  data-testid={`retry-source-${s.id}`}
                  onClick={() => onRetrySource(s.id)}
                >
                  {s.status === "failed" ? "重试" : "重新同步"}
                </button>
```

**Prescribed (D-03):** wrap retry in `s.kind !== "wiki_page"` — hide entirely. Do **not** require `wiki.enabled` for the hide (unlike compile). Backend soft-skip remains IPC safety net.

**Toolbar export hide** (line 142) — unrelated but same hide-vs-disable idiom:
```tsx
          {config?.wiki?.enabled === true && (
```

---

### `src/views/LibraryView.test.tsx` (test)

**Analog:** same file — wiki_page fixture + `queryByTestId(...).toBeNull()`

**Fixture pattern** (lines 33–42):
```ts
const indexedWikiPage = {
  id: "wiki-1",
  kind: "wiki_page",
  uri: "wiki://page/a",
  title: "笔记 a",
  status: "indexed",
  indexed_at: 1,
  error: null,
  summary: null,
};
```

**Hide-assert pattern to extend** (lines 94–102):
```ts
  it("hides compile for indexed wiki_page even when enabled", () => {
    render(
      <LibraryView
        {...baseProps}
        sources={[indexedWikiPage]}
        config={{ wiki: { enabled: true, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("wiki-compile-wiki-1")).toBeNull();
  });
```

**Prescribed sibling case:**
```ts
  it("hides retry for wiki_page", () => {
    render(
      <LibraryView
        {...baseProps}
        sources={[indexedWikiPage]}
        config={{ wiki: { enabled: true, auto_on_insights: false } } as AppConfig}
      />,
    );
    expect(screen.queryByTestId("retry-source-wiki-1")).toBeNull();
  });
  // optional: local_file still has retry-source-src-1
```

**Setup imports** (lines 1–8) — reuse as-is:
```ts
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AppConfig } from "../types/ipc";
import { LibraryView } from "./LibraryView";
```

---

### `.planning/phases/12-*/12-VALIDATION.md` & `13-*/13-VALIDATION.md` (docs)

**Analog:** Phase 11 frontmatter flip shape + Phase 12/13 VERIFICATION as evidence sources

**Phase 11 pattern** (already compliant — copy frontmatter shape only):
```yaml
status: draft
nyquist_compliant: true
wave_0_complete: false
```

**Phases 12/13 today:**
```yaml
status: draft
nyquist_compliant: false
wave_0_complete: false
```

**Apply (D-10..D-12):** only change `nyquist_compliant: false` → `true`; leave `status` and `wave_0_complete` unchanged; add short body note citing VERIFICATION (`status: passed`) + green tests. Example wording (Discretion):

> **Nyquist backfill (Phase 14):** Marked compliant from existing VERIFICATION (`status: passed`) and green automated tests; full `/gsd-validate-phase` re-run not required.

**Evidence sources:**
- `12-VERIFICATION.md` — `status: passed`, score 13/13
- `13-VERIFICATION.md` — `status: passed`, score 8/8

---

### `src/hooks/useLibrary.ts` (no change — FE error path)

**Analog for planner awareness only** — disabled preflight hits existing catch (D-08):

```ts
  const reportError = useCallback(
    (error: unknown) => {
      onError?.(String(error));
    },
    [onError],
  );
```

```ts
      } catch (error) {
        reportError(error);
      }
```

No Chinese mapping for `"wiki is disabled"`.

---

## Shared Patterns

### Soft skip = `Ok(false)` without `mark_failed`, before destructive work

**Source:** `src-tauri/src/index_ops.rs` skip consumers (125–135, 501–510) + RESEARCH early-return prescription  
**Apply to:** WikiPage path in `reindex_source` (single + bulk share this function — D-05)  
**Do not copy:** LocalFile/Memory/Cursor `mark_failed` then `Ok(false)` (fail-closed)

### Feature-flag hard gate via `InsightsError::WikiDisabled`

**Source:** `crates/insights/src/wiki_export.rs` 37–40; `crates/insights/src/wiki.rs` 88–91; Display in `error.rs` 13–14  
**Apply to:** preflight helper/cmd parity with `export_wiki_zip`  
**IPC:** `map_err(|e| e.to_string())` → `"wiki is disabled"`

### Thin Tauri commands

**Source:** `src-tauri/src/commands/wiki.rs`  
**Apply to:** `wiki_export_preflight_cmd` — read `cfg.wiki.enabled`, call crate, map errors to `String`

### Hide (not disable) inappropriate Library controls

**Source:** `LibraryView.tsx` compile gate (238–248); export toolbar (142)  
**Apply to:** `retry-source-{id}` when `s.kind === "wiki_page"` (kind-only; no `wiki.enabled` requirement)

### Vitest hide matrix

**Source:** `LibraryView.test.tsx` — `indexedWikiPage` + `queryByTestId(...).toBeNull()`  
**Apply to:** new retry-hide case beside compile-hide

### Nyquist frontmatter-only backfill

**Source:** Phase 11 VALIDATION (`nyquist_compliant: true`, `status: draft`, `wave_0_complete: false`)  
**Apply to:** 12/13 VALIDATION + short evidence note (Phase 11 has no note; 12/13 need D-12 note)

### E2E waiver (locked)

**Source:** CONTEXT D-04/D-15  
**Apply to:** plans — cite waiver so plan-checker does not demand new E2E; existing `wiki.spec.ts` covers export/compile journeys

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | None — all Phase 14 touchpoints have in-repo analogs |

---

## Metadata

**Analog search scope:** `src-tauri/src/index_ops.rs`, `src-tauri/src/commands/wiki.rs`, `crates/insights/src/{error,wiki,wiki_export,lib}.rs`, `src/views/LibraryView.{tsx,test.tsx}`, `src/hooks/useLibrary.ts`, `.planning/phases/{11,12,13}-*/` VALIDATION + VERIFICATION, `src-tauri/Cargo.toml`  
**Files scanned:** ~15 primary touchpoints  
**Pattern extraction date:** 2026-07-24  
**Agent definition:** `C:\Users\vasin\.cursor\agents\gsd-pattern-mapper.md` (followed)

### Planner quick-copy checklist

1. Early `WikiPage` → `Ok(false)` **before** `delete_chunks_for_source`
2. Preflight: `WikiDisabled` same string as export; prefer crate helper + unit test
3. LibraryView: `s.kind !== "wiki_page"` around retry button; Vitest sibling of compile-hide
4. Flip only `nyquist_compliant` on 12/13 VALIDATION + evidence note
5. Verify: cargo (`tauri_app_lib` / `insights`) + Vitest LibraryView; **no new E2E**
