# Phase 12: Obsidian zip export - Pattern Map

**Mapped:** 2026-07-23
**Files analyzed:** 16
**Analogs found:** 15 / 16

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/insights/src/wiki_export.rs` (NEW) | service | file-I/O | `crates/insights/src/wiki.rs` (`resolve_wiki_page_path`, `any_content_md`, `count_files_recursive`) | exact |
| `crates/insights/src/wiki.rs` (MODIFY — promote empty helper or leave private + re-use logic) | service | file-I/O / transform | same file (`any_content_md` tests) | exact |
| `crates/insights/src/error.rs` | model | request-response | `crates/insights/src/error.rs` (`WikiDisabled`, `Io`) | exact |
| `crates/insights/src/lib.rs` | config | — | `crates/insights/src/lib.rs` (re-export `wiki::*`) | exact |
| `Cargo.toml` (workspace `zip`) | config | — | `Cargo.toml` `[workspace.dependencies]` | exact |
| `crates/insights/Cargo.toml` | config | — | `crates/insights/Cargo.toml` (`serde` / `tempfile` style) | exact |
| `src-tauri/src/commands/wiki.rs` | controller | request-response / file-I/O | `src-tauri/src/commands/wiki.rs` (`compile_wiki_cmd`) | exact |
| `src-tauri/src/commands/mod.rs` | route | — | `src-tauri/src/commands/mod.rs` (`pub use wiki::…`) | exact |
| `src-tauri/src/lib.rs` | route | — | `src-tauri/src/lib.rs` (`generate_handler![…, compile_wiki_cmd]`) | exact |
| `src/lib/tauri.ts` | utility | request-response | `src/lib/tauri.ts` (`compileWiki`) | exact |
| `src/types/ipc.ts` | model | — | `src/types/ipc.ts` (`WikiCompileSummary`) | exact |
| `src/hooks/useLibrary.ts` | hook | request-response / file-I/O | `src/hooks/useLibrary.ts` (`pickAndIndex`, `compileWiki`) | exact |
| `src/views/LibraryView.tsx` | component | request-response | `src/views/LibraryView.tsx` (toolbar + wiki hide gate) | exact |
| `src/App.tsx` | component | request-response | `src/App.tsx` (`handleCompileWiki` busy wrapper) | exact |
| `src/AppShell.tsx` (optional soft notice) | component | event-driven | **partial** — only red `err` alert today | role-match |
| `src/views/LibraryView.test.tsx` | test | — | `src/views/LibraryView.test.tsx` (wiki compile visibility) | exact |
| `src/hooks/useLibrary.test.ts` | test | — | `src/hooks/useLibrary.test.ts` (`compileWiki` + dialog mock) | exact |
| `crates/insights` unit tests for export (in `wiki_export.rs` or `wiki.rs` `#[cfg(test)]`) | test | file-I/O | `crates/insights/src/wiki.rs` tests (`compile_rejects_when_disabled`, `any_content_md`, `tempfile`) | exact |

## Pattern Assignments

### `crates/insights/src/wiki_export.rs` (service, file-I/O)

**Analog:** `crates/insights/src/wiki.rs`

**API surface to implement** (from RESEARCH / design sketch):
```rust
pub fn wiki_has_exportable_notes(wiki_root: &Path) -> bool;
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path) -> Result<()>;
```

**Path-safety pattern** (lines 301–314) — reuse for every zip entry name after `strip_prefix(wiki_root)`:
```rust
fn resolve_wiki_page_path(wiki_root: &Path, slug: &str) -> Result<PathBuf> {
    use std::path::Component;
    let rel = Path::new(slug);
    if rel.is_absolute()
        || rel.components().any(|c| {
            !matches!(c, Component::Normal(_))
        })
    {
        return Err(InsightsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unsafe wiki slug: {slug}"),
        )));
    }
    Ok(wiki_root.join(format!("{slug}.md")))
}
```
For zip: allow multi-segment relative paths (`sources/foo.md`) where **every** component is `Normal` (reject `ParentDir`, `Prefix`, absolute). Normalize separators to `/`. Skip first component `.obsidian` when walking disk (D-15).

**Empty semantics (D-10)** — promote test helper (lines 727–741):
```rust
fn any_content_md(wiki_root: &Path) -> bool {
    for dir in ["sources", "entities", "concepts"] {
        let d = wiki_root.join(dir);
        if !d.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&d) {
            for e in entries.flatten() {
                if e.path().extension().is_some_and(|x| x == "md") {
                    return true;
                }
            }
        }
    }
    false
}
```
→ public `wiki_has_exportable_notes`; call from preflight + hard-reject inside `export_wiki_zip` (D-12).

**Hard gate when disabled** (lines 88–91) — mirror in export entry that receives `wiki_enabled`:
```rust
if !wiki_enabled {
    return Err(InsightsError::WikiDisabled);
}
```

**Recursive FS walk** (test helper lines 1384–1403) — prefer stack `read_dir` over new `walkdir` dep:
```rust
fn count_files_recursive(root: &Path) -> usize {
    // … stack = vec![root]; pop dirs; count files
}
```
Adapt to collect `(relative_name, absolute_path)` for regular files only; do not follow symlinks out of root.

**Zip write pattern** (external, RESEARCH — zip 7.2; no in-repo analog):
```rust
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated);
zip.start_file("sources/demo.md", options)?;
zip.write_all(b"# Demo\n")?;
zip.start_file(".obsidian/app.json", options)?;
zip.write_all(br#"{"legacyEditor":false}"#)?; // inject-only (D-14)
zip.finish()?;
```
Prefer write to temp then rename (RESEARCH Pitfall 3); assert after export `!wiki_root.join(".obsidian").exists()`.

**Unit-test style** (lines 744–771) — `tempfile::tempdir`, assert error variant + zero side effects:
```rust
#[tokio::test] // or sync #[test] for pure FS export
async fn compile_rejects_when_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let wiki_root = dir.path().join("wiki");
    let err = /* … */.unwrap_err();
    assert!(matches!(err, InsightsError::WikiDisabled), …);
    assert_eq!(count_files_recursive(&wiki_root), 0, …);
}
```

---

### `crates/insights/src/error.rs` (model)

**Analog:** same file

**Error enum pattern** (lines 3–25):
```rust
#[derive(Debug, Error)]
pub enum InsightsError {
    #[error("wiki is disabled")]
    WikiDisabled,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    // …
}
```

**Add for Phase 12:**
- `WikiEmpty` with Display usable by FE / `to_string()` at Tauri boundary (D-09/D-12)
- `Zip(#[from] zip::result::ZipError)` (or map zip errors into `Io`) — prefer `#[from]` if zip errors are `Send + Sync`

---

### `crates/insights/src/lib.rs` (config / barrel)

**Analog:** same file (lines 1–13)

```rust
mod error;
mod summarize;
mod tasks;
mod wiki;
// add: mod wiki_export;

pub use error::{InsightsError, Result};
pub use wiki::{
    analyze_source_for_wiki, compile_wiki_for_source, /* … */
};
// add: pub use wiki_export::{export_wiki_zip, wiki_has_exportable_notes};
```

---

### `Cargo.toml` + `crates/insights/Cargo.toml` (config)

**Analog:** workspace deps (root `Cargo.toml` lines 27–41) + crate deps (`crates/insights/Cargo.toml`)

**Workspace pin** (RESEARCH — MSRV 1.85 fit):
```toml
# Cargo.toml [workspace.dependencies]
zip = { version = "7.2", default-features = false, features = ["deflate"] }
```

**Crate dep:**
```toml
# crates/insights/Cargo.toml [dependencies]
zip = { workspace = true }
```

**Dev:** keep existing `tempfile = "3"` for export unit tests.

---

### `src-tauri/src/commands/wiki.rs` (controller, request-response / file-I/O)

**Analog:** existing `compile_wiki_cmd` (full file)

**Thin-command pattern:**
```rust
use insights::{compile_wiki_for_source, WikiCompileSummary};
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn compile_wiki_cmd(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<WikiCompileSummary, String> {
    let cfg = state.config();
    let wiki_root = state
        .config_path
        .parent()
        .ok_or_else(|| "config_path has no parent".to_string())?
        .join("wiki");
    compile_wiki_for_source(
        /* … deps … */,
        &wiki_root,
        cfg.wiki.enabled,
    )
    .await
    .map_err(|e| e.to_string())
}
```

**Copy for Phase 12:**
1. Resolve `wiki_root` the same way (`config_path.parent()/wiki`).
2. Pass `cfg.wiki.enabled` into crate (or gate before call).
3. `.map_err(|e| e.to_string())` at boundary.
4. New cmds (RESEARCH discretion names):
   - `wiki_export_preflight_cmd` → serde `{ hasNotes: bool }` (`#[serde(rename_all = "camelCase")]`)
   - `export_wiki_zip_cmd(dest_path: String)` → `Result<(), String>` (or small `{ path }` summary)

Keep commands thin — **no** zip logic in Tauri.

---

### `src-tauri/src/commands/mod.rs` + `src-tauri/src/lib.rs` (route)

**Analog:** registration of `compile_wiki_cmd`

`commands/mod.rs` line 39:
```rust
pub use wiki::compile_wiki_cmd;
// extend: pub use wiki::{compile_wiki_cmd, wiki_export_preflight_cmd, export_wiki_zip_cmd};
```

`lib.rs` `generate_handler!` (~line 107):
```rust
compile_wiki_cmd,
// add wiki_export_preflight_cmd, export_wiki_zip_cmd,
```

Also update `use commands::{ … }` import list at top of `lib.rs`.

---

### `src/lib/tauri.ts` (utility, request-response)

**Analog:** `compileWiki` (lines 109–111) + invoke import style (lines 1–19)

```typescript
export function compileWiki(sourceId: string) {
  return invoke<WikiCompileSummary>("compile_wiki_cmd", { sourceId });
}
```

**Add:**
```typescript
export function wikiExportPreflight() {
  return invoke<{ hasNotes: boolean }>("wiki_export_preflight_cmd");
}
export function exportWikiZip(destPath: string) {
  return invoke<void>("export_wiki_zip_cmd", { destPath });
}
```
Arg names must match Rust `snake_case` command params (`dest_path` ↔ `{ destPath }` via Tauri camelCase convention already used by `sourceId`).

---

### `src/types/ipc.ts` (model)

**Analog:** `WikiCompileSummary` (lines 146–153)

```typescript
export interface WikiCompileSummary {
  wikiRoot: string;
  pagesWritten: number;
  // …
}
```

**Add** if preflight returns a struct:
```typescript
export interface WikiExportPreflight {
  hasNotes: boolean;
}
```

---

### `src/hooks/useLibrary.ts` (hook, request-response / file-I/O)

**Analog A — dialog cancel + filter** (`pickAndIndex`, lines 116–128):
```typescript
const pickAndIndex = useCallback(async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Documents", extensions: ["txt", "md", "markdown"] }],
    });
    if (selected === null) return; // D-05 equivalent
    await indexFile(selected);
    await refreshSources();
  } catch (error) {
    reportError(error);
  }
}, [refreshSources, reportError]);
```

**Analog B — IPC + reportError** (`compileWiki`, lines 95–105):
```typescript
const compileWiki = useCallback(
  async (sourceId: string) => {
    try {
      await compileWikiCmd(sourceId);
      await refreshSources();
    } catch (error) {
      reportError(error);
    }
  },
  [refreshSources, reportError],
);
```

**Export flow to compose** (CONTEXT D-09..D-11 + RESEARCH):
1. `wikiExportPreflight()` → if `!hasNotes`, `reportError("还没有可导出的笔记，请先生成笔记")` and return (no dialog).
2. `save({ defaultPath: \`jarvis-wiki-${localDate}.zip\`, filters: [{ name: "Zip", extensions: ["zip"] }] })`.
3. If `null` → return (D-05).
4. `exportWikiZip(destPath)` → on success signal path/filename to caller (or set notice); on catch `reportError`.

Import `save` from `@tauri-apps/plugin-dialog` beside existing `open`.

**Busy ownership:** stay in App (`setBusy`) like compile — hook should **not** own `busy` (see test “does not expose … busy”).

---

### `src/views/LibraryView.tsx` (component, request-response)

**Analog — toolbar placement** (lines 118–140):
```tsx
<div className="flex flex-wrap gap-2">
  <button type="button" className="btn-ghost" data-testid="run-insights-all" …>
    一键洞察
  </button>
  <button type="button" className="btn-ghost" disabled={busy} onClick={onPickAndIndex}>
    选择文件索引
  </button>
  {/* D-01: insert 「导出 Wiki」 HERE — after 选择文件索引 */}
</div>
```

**Analog — hide when disabled** (lines 225–235), apply same gate to toolbar export:
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

**New button contract (D-02/D-03/D-08):**
```tsx
{config?.wiki?.enabled === true && (
  <button
    type="button"
    className="btn-ghost"
    disabled={busy}
    data-testid="wiki-export"
    onClick={onExportWiki}
  >
    导出 Wiki
  </button>
)}
```

Extend `LibraryViewProps` with `onExportWiki: () => void` (mirror `onPickAndIndex`).

---

### `src/App.tsx` (component, request-response)

**Analog — busy wrapper** (lines 254–262):
```typescript
async function handleCompileWiki(sourceId: string) {
  setErr(null);
  setBusy(true);
  try {
    await libCompileWiki(sourceId);
  } finally {
    setBusy(false);
  }
}
```

**Copy for export:** `handleExportWiki` → `setErr(null)` → `setBusy(true)` → call hook `exportWiki` → `finally setBusy(false)`. Wire `onExportWiki={() => void handleExportWiki()}` next to other Library props.

**Success notice (D-07 / RESEARCH Pitfall 6):** AppShell today only has red `err` — do **not** put success on `setErr`. Discretion: soft notice state + pass to AppShell, or local Library status with `data-testid="wiki-export-done"`.

`useLibrary({ onError: setErr })` already wires failures (line 38).

---

### `src/AppShell.tsx` (optional soft notice — partial analog)

**Existing error bar only** (lines 79–95):
```tsx
{err && (
  <div
    role="alert"
    className="glass-panel border-red-400/30 bg-red-950/40 px-4 py-3 text-sm text-red-200"
  >
    …
  </div>
)}
```

**No exact success-notice analog.** If planner adds notice: parallel slot with non-red styling (`border-cyan` / emerald), `data-testid="wiki-export-done"`, dismissible like `err`. Empty preflight message **does** use `err` / `reportError` with exact Chinese string (D-09).

---

### `src/views/LibraryView.test.tsx` (test)

**Analog — visibility / hide** (lines 71–86, 117–121):
```typescript
it("shows 生成笔记 when wiki.enabled and indexed local_file", () => {
  render(<LibraryView {...baseProps} />);
  const btn = screen.getByTestId("wiki-compile-src-1");
  expect(btn.textContent).toContain("生成笔记");
});

it("hides compile when wiki.enabled is false", () => {
  render(
    <LibraryView
      {...baseProps}
      config={{ wiki: { enabled: false, auto_on_insights: false } } as AppConfig}
    />,
  );
  expect(screen.queryByTestId("wiki-compile-src-1")).toBeNull();
});
```

**Extend:** assert `wiki-export` visible when enabled; hidden when false/missing; appears after file-index in toolbar; click calls `onExportWiki`; `busy` disables button. Add `onExportWiki: vi.fn()` to `baseProps`.

---

### `src/hooks/useLibrary.test.ts` (test)

**Analog — mocks + compileWiki** (lines 10–47, 83–102):
```typescript
vi.mock("../lib/tauri", () => ({
  // … existing
  compileWiki: vi.fn().mockResolvedValue({ /* … */ }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));
```

**Extend mocks:** `wikiExportPreflight`, `exportWikiZip`, and `save` from plugin-dialog.

**Cases to add:**
| Case | Expect |
|------|--------|
| preflight `hasNotes: false` | `onError` called with exact empty string; `save` **not** called |
| preflight ok + `save` → `null` | `exportWikiZip` **not** called |
| preflight ok + path | `exportWikiZip` called with path |
| export throws | `onError` called |

---

## Shared Patterns

### Thin Tauri + fat crate
**Source:** `src-tauri/src/commands/wiki.rs` + `insights::compile_wiki_for_source`  
**Apply to:** `wiki_export_preflight_cmd`, `export_wiki_zip_cmd`  
Resolve `wiki_root` from `config_path.parent()/wiki`; map errors with `.map_err(|e| e.to_string())`; never open SQLite outside `store`.

### `wiki.enabled` dual gate
**Source:** `wiki.rs` hard reject + `LibraryView` `config?.wiki?.enabled === true`  
**Apply to:** export IPC + toolbar button (hide, don’t disable-only).

### Path-safe relative paths
**Source:** `resolve_wiki_page_path` component allowlist  
**Apply to:** every zip entry name; unit-test reject `..` / absolute / drive prefixes; force `/` separators.

### Dialog cancel = no side effects
**Source:** `useLibrary.pickAndIndex` `if (selected === null) return`  
**Apply to:** Save dialog cancel (D-05) and empty preflight (no dialog at all).

### App-owned `busy` + hook `reportError`
**Source:** `App.handleCompileWiki` + `useLibrary({ onError: setErr })`  
**Apply to:** export handler; failures → shared red alert; success → separate soft notice (not red).

### Empty-tree definition
**Source:** `any_content_md` section dirs only  
**Apply to:** preflight + export hard-reject; root-only `index.md` = empty.

### Serde camelCase IPC structs
**Source:** `WikiCompileSummary` `#[serde(rename_all = "camelCase")]`  
**Apply to:** preflight response `{ hasNotes: bool }`.

### Vitest + jsdom wiki UI tests
**Source:** `LibraryView.test.tsx` / `useLibrary.test.ts`  
**Apply to:** Phase 12 FE verification; **no** `wiki.spec.ts` E2E in this phase (deferred Phase 13).

---

## No Analog Found

| File / concern | Role | Data Flow | Reason |
|----------------|------|-----------|--------|
| Soft success notice UI (`wiki-export-done`) | component | event-driven | AppShell only has red `err` alert — planner invents dual-tone/notice slot per RESEARCH discretion |
| `zip` crate usage in-repo | service | file-I/O | No workspace `zip` dep yet — copy RESEARCH / docs.rs 7.2 `ZipWriter` examples |

---

## Metadata

**Analog search scope:** `crates/insights/`, `src-tauri/src/commands/`, `src-tauri/src/lib.rs`, `src/views/LibraryView*`, `src/hooks/useLibrary*`, `src/lib/tauri.ts`, `src/types/ipc.ts`, `src/App.tsx`, `src/AppShell.tsx`, root + insights `Cargo.toml`  
**Files scanned:** ~18 primary + design sketch Task 6  
**Pattern extraction date:** 2026-07-23  
**E2E note:** Phase 12 CONTEXT defers `e2e/specs/wiki.spec.ts` to Phase 13 — do not treat missing E2E analog as a Phase 12 gap.
