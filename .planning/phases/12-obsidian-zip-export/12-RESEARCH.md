# Phase 12: Obsidian zip export - Research

**Researched:** 2026-07-23
**Domain:** Rust zip packaging + Tauri Save dialog + Library toolbar IPC (WIKI-07)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Export button placement
- **D-01:** Export control lives **only on the Library toolbar** (not Settings). Place it **after**「选择文件索引」.
- **D-02:** Button label **「导出 Wiki」** with `data-testid="wiki-export"`.
- **D-03:** When `wiki.enabled !== true`, **do not render** the export button (same hide pattern as「生成笔记」).

#### Save destination UX
- **D-04:** Use the system **Save** dialog (`@tauri-apps/plugin-dialog` `save`). User picks destination; FE passes `destPath` into export IPC.
- **D-05:** If the user **cancels** the dialog: **no side effects** — do not invoke export, do not write any file.
- **D-06:** Default suggested filename: **`jarvis-wiki-YYYY-MM-DD.zip`** with a `.zip` file filter.
- **D-07:** On success, show a **brief success message** including path or filename. On failure, use existing Library/App **`reportError`** / shared error bar (no per-button inline error UI).
- **D-08:** While export runs, use the existing App-level **`busy`** flag (disable Library toolbar actions including export — same as compile/index).

#### Empty / missing wiki behavior
- **D-09:** If the wiki tree is empty, **block before** opening the Save dialog and show: **「还没有可导出的笔记，请先生成笔记」**.
- **D-10:** Empty means: **no `.md` files under `sources/`, `entities/`, or `concepts/`** (root-only `index.md` or missing dirs count as empty — aligned with Phase 10 scan semantics).
- **D-11:** Add a **lightweight preflight IPC** (disk-based) that FE calls on click; empty → message + no dialog; non-empty → Save dialog → export.
- **D-12:** `export_wiki_zip` must **hard-reject** empty trees as defense-in-depth (even after a successful preflight).

#### `.obsidian` stub & zip payload
- **D-13:** Stub is **minimal**: only `.obsidian/app.json` with `{"legacyEditor":false}`.
- **D-14:** Inject the stub **into the zip only** — do **not** write `.obsidian/` into `{app_data}/wiki/` on disk.
- **D-15:** If `.obsidian/` already exists on disk under `wiki_root`, **skip packing it**; the zip **always** uses Jarvis’s minimal stub (deterministic export).
- **D-16:** Zip includes **all regular files** under `wiki_root` as **relative, path-safe** entries (reject `..` / absolute paths) **plus** the injected stub. Skip on-disk `.obsidian/` per D-15. No secrets / no `target/`.

### Claude's Discretion
- Preflight command naming (`wiki_export_preflight` vs similar) and exact success-message UI slot (reuse error bar styling vs soft status text) left to planner — must satisfy D-07/D-09/D-11.
- Add workspace `zip` crate (plan suggests `zip = "2"`) if still absent — researcher confirms latest stable.
- Whether preflight returns a boolean vs a small status struct — planner chooses; FE needs a clear empty/ok signal.

### Deferred Ideas (OUT OF SCOPE)
- Full `wiki.spec.ts` enable → compile → list → export journey — **Phase 13** (WIKI-09)
- Citation trust / RAG still cites originals — **Phase 13** (WIKI-08)
- Bidirectional Obsidian sync — out of product scope
- Settings duplicate export button — explicitly rejected (Library only)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIKI-07 | User can export the wiki tree as an Obsidian-compatible zip (includes minimal `.obsidian/` stub) | `export_wiki_zip` in `insights` + thin Tauri cmds + Library toolbar + `save` dialog; path-safe zip entries; stub inject-only; gated on `wiki.enabled` |
</phase_requirements>

## Summary

Phase 12 is a FS-only export feature: walk `{app_data}/wiki`, pack relative path-safe entries into a Deflated `.zip`, always inject `.obsidian/app.json` with `{"legacyEditor":false}`, never write that stub to disk, and gate both UI and backend on `wiki.enabled`. Empty trees (no `.md` under `sources|entities|concepts`) are blocked by a preflight IPC before the Save dialog, with a hard reject inside export.

The workspace has **no** `zip` dependency today. Absolute latest non-pre on crates.io is **`8.6.0`** (MSRV **1.88**), which exceeds Jarvis workspace MSRV **1.85**. Highest non-pre that fits without an MSRV bump is **`7.2.0`** (MSRV **1.83**). Plan sketch `zip = "2"` still works (`2.4.2`) but is not the newest MSRV-compatible stable — recommend **`7.2`** with `default-features = false, features = ["deflate"]`.

Frontend patterns already exist: Library toolbar hide-when-disabled, App `busy` wrappers, `useLibrary` + `reportError`, `@tauri-apps/plugin-dialog` `open` (add `save`). E2E for this journey is deferred to Phase 13 per locked context (overrides in-phase E2E completion; Vitest + Rust unit cover Phase 12).

**Primary recommendation:** Implement `insights::export_wiki_zip` + `wiki_has_exportable_notes` with `zip 7.2` (deflate-only), thin Tauri `wiki_export_preflight_cmd` / `export_wiki_zip_cmd`, Library toolbar「导出 Wiki」after file-index, FE flow preflight → `save` → export under App `busy`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Empty-wiki detection (D-10) | API / Backend (`insights`) | Browser (display copy) | Disk scan must match Phase 10 semantics; FE only shows locked Chinese string |
| Path-safe zip packing + stub inject | API / Backend (`insights`) | — | Zip Slip / `..` / absolute paths are server-side safety; no DB |
| `wiki.enabled` hard gate | API / Backend | Browser (hide button) | Mirror `compile_wiki_for_source` / `InsightsError::WikiDisabled` |
| Save destination picker | Browser / Client | OS dialog (Tauri plugin) | `save()` returns path or `null`; FE never invents dest |
| Toolbar export control + busy | Browser / Client | App shell | D-01..D-03, D-08; busy owned by App like compile |
| Success / empty / error messaging | Browser / Client | AppShell alert | D-07/D-09; failures via shared error bar |
| SQLite / store | — | — | Out of scope — export is FS-only; store remains sole DB owner |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `zip` | **7.2.0** (pin `7.2` / `7`) | Write Deflated ZIP with relative entry names | Official ZIP writer (`ZipWriter` + `SimpleFileOptions`); legitimacy OK; fits MSRV 1.85 [VERIFIED: crates.io API + `cargo info zip@7.2.0`] |
| `@tauri-apps/plugin-dialog` | **^2.7.1** (already in app) | `save({ defaultPath, filters })` → `string \| null` | Already wired (`tauri_plugin_dialog::init`, `dialog:default`); FE uses `open` today [VERIFIED: package.json + capabilities + typedefs] |
| `tempfile` | **3** (already `insights` dev-dep) | Unit-test wiki trees + dest zip | Existing wiki tests pattern [VERIFIED: crates/insights/Cargo.toml] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::fs` recursive walk | std | Enumerate wiki files | Prefer over new `walkdir` dep — wiki.rs already walks dirs [VERIFIED: crates/insights/src/wiki.rs] |
| Vitest 3 + Testing Library + jsdom | existing | Toolbar / hook wiring tests | Match Phase 11 LibraryView / useLibrary tests [VERIFIED: package.json, vite.config.ts] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `zip 7.2` | `zip 8.6.0` (absolute latest stable) | Newer API line but **MSRV 1.88** — requires workspace MSRV bump [VERIFIED: crates.io `rust_version`] |
| `zip 7.2` | `zip = "2"` → 2.4.2 | Matches plan sketch; older; still fine API-wise — skip unless pinning conservatively |
| `zip 7.2` | `9.0.0-pre2` | Pre-release — forbidden by jarvis-stack unless required [VERIFIED: crates.io] |
| Hand-rolled ZIP | — | CRC/central-directory bugs; Zip Slip risk — do not |

**Installation:**

```toml
# Cargo.toml [workspace.dependencies]
zip = { version = "7.2", default-features = false, features = ["deflate"] }

# crates/insights/Cargo.toml
zip = { workspace = true }
```

**Version verification (2026-07-23):**

| Candidate | Status | MSRV | Notes |
|-----------|--------|------|-------|
| `9.0.0-pre2` | pre-release | 1.88 | Reject |
| `8.6.0` | latest non-pre | **1.88** | Reject without MSRV bump |
| **`7.2.0`** | latest ≤ MSRV 1.85 | **1.83** | **Recommend** |
| `2.4.2` | plan sketch line | 1.73 | Acceptable fallback |

**Legitimacy:** `gsd-tools query package-legitimacy check --ecosystem crates zip` → **OK** (exists since 2014, ~4.1M weekly downloads, repo `zip-rs/zip2`) [VERIFIED: gsd-tools package-legitimacy].

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `zip` | crates.io | since 2014-11 | ~4.1M/wk | https://github.com/zip-rs/zip2 | OK | Approved — pin **7.2** (MSRV-fit) |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none  

No new npm packages required (`@tauri-apps/plugin-dialog` already present).

## Architecture Patterns

### System Architecture Diagram

```text
[Library toolbar] wiki.enabled?
        | no → (no button)
        | yes → click 「导出 Wiki」
        v
[FE] invoke wiki_export_preflight_cmd
        |
        +-- empty → setErr「还没有可导出的笔记…」; STOP (no dialog)
        +-- has notes → save({ defaultPath: jarvis-wiki-YYYY-MM-DD.zip, filters: zip })
                |
                +-- null (cancel) → STOP (no IPC)
                +-- destPath → setBusy → export_wiki_zip_cmd({ destPath })
                        |
                        +-- backend: wiki.enabled? else WikiDisabled
                        +-- wiki_has_exportable_notes? else hard-reject
                        +-- walk wiki_root → path-safe relative entries
                        +-- skip on-disk .obsidian/**
                        +-- inject .obsidian/app.json stub into zip only
                        +-- write Deflated zip to destPath
                        v
                 success notice (path/filename) / reportError on failure
```

### Recommended Project Structure

```text
crates/insights/src/
├── wiki.rs              # reuse path-safety + promote empty-scan helper
├── wiki_export.rs       # NEW: export_wiki_zip, wiki_has_exportable_notes
├── error.rs             # WikiEmpty (+ Zip from zip::result::ZipError)
└── lib.rs               # re-export export API

src-tauri/src/commands/wiki.rs
├── compile_wiki_cmd       # existing
├── wiki_export_preflight_cmd  # NEW
└── export_wiki_zip_cmd        # NEW

src/
├── views/LibraryView.tsx      # toolbar button after 选择文件索引
├── hooks/useLibrary.ts        # exportWiki / preflight helpers (or App handler)
├── lib/tauri.ts               # invoke wrappers
└── *.test.tsx / useLibrary.test.ts
```

### Pattern 1: Thin Tauri command + fat crate (mirror compile)

**What:** Resolve `wiki_root` as `config_path.parent()/wiki`, pass `cfg.wiki.enabled`, map errors to `String`.  
**When to use:** All new wiki IPC.  
**Example:** Existing `compile_wiki_cmd` in `src-tauri/src/commands/wiki.rs` [VERIFIED: codebase].

### Pattern 2: Path-safe relative entry names

**What:** For each file under `wiki_root`, `strip_prefix(wiki_root)`, reject absolute / `ParentDir` / `Prefix` components (same mindset as `resolve_wiki_page_path`), normalize `\` → `/`, skip first component `.obsidian`.  
**When to use:** Every zip entry before `start_file`.  
**Example:** Reuse logic from:

```301:314:crates/insights/src/wiki.rs
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

[VERIFIED: codebase]

### Pattern 3: Empty semantics (D-10)

**What:** Empty iff no `.md` under `sources/`, `entities/`, or `concepts/` — root-only `index.md` counts as empty.  
**When to use:** Preflight + export hard-reject.  
**Example:** Test helper `any_content_md` already encodes this; promote to public/crate-visible `wiki_has_exportable_notes` [VERIFIED: `crates/insights/src/wiki.rs` tests ~727–741].

### Pattern 4: FE hide + App busy

**What:** Render export only when `config?.wiki?.enabled === true`; wrap handler with `setBusy(true/false)` like `handleCompileWiki`.  
**When to use:** Toolbar export.  
**Example:** `LibraryView` compile gate + `App.tsx` `handleCompileWiki` [VERIFIED: codebase].

### Anti-Patterns to Avoid

- **Writing `.obsidian/` to disk under wiki_root:** Violates D-14; pollutes rebuildable wiki tree.
- **Packing on-disk `.obsidian/`:** Violates D-15; non-deterministic / user-local settings leak.
- **Opening Save dialog before preflight:** Violates D-09/D-11.
- **Putting zip logic in Tauri command:** Breaks thin-shell / unit-testability.
- **Using zip 8.x without MSRV bump:** Breaks declared workspace `rust-version = "1.85"`.
- **Shipping Phase 12 without path-safety unit tests:** Success criterion #2.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ZIP container + Deflate + CRC | Custom zip bytes | `zip` crate `ZipWriter` | Easy to get wrong; Zip Slip on extract side for consumers |
| Save file dialog | Custom web modal | `@tauri-apps/plugin-dialog` `save` | Native UX; scope integration; already licensed in app |
| Recursive path safety | Ad-hoc string replace only | `Path` components + `strip_prefix` + `/` normalize | Windows `\` and `..` edge cases |
| Empty detection | FE-only heuristics | Shared `wiki_has_exportable_notes` | Must match Phase 10 scan dirs |

**Key insight:** Path safety and empty semantics belong in `insights` once; UI only orchestrates dialog + messaging.

## Common Pitfalls

### Pitfall 1: Zip Slip / absolute entry names
**What goes wrong:** Entries like `../evil` or `C:\...` escape vault on extract.  
**Why it happens:** Using `path.to_string_lossy()` without `strip_prefix` + component checks.  
**How to avoid:** Reject non-`Normal` components; assert in unit tests that archive names have no `..`, no drive prefixes, no leading `/`.  
**Warning signs:** Test fixtures with `wiki_root/../outside.md` somehow appearing in zip.

### Pitfall 2: Windows backslash entry names
**What goes wrong:** Obsidian/unzip tools mishandle `\`.  
**Why it happens:** `PathDisplay` on Windows.  
**How to avoid:** Force `/` separators (or `start_file_from_path` which normalizes) [CITED: docs.rs/zip/7.2.0 ZipWriter::start_file_from_path].  
**Warning signs:** Zip listing shows `sources\foo.md`.

### Pitfall 3: Partial/corrupt zip on failure
**What goes wrong:** Failed mid-write leaves broken `.zip` at `destPath`.  
**Why it happens:** Writing directly then erroring (called out in zip `write_dir` example).  
**How to avoid:** Write to `*.zip.tmp` / tempfile in same dir, `finish`, then atomic rename; delete temp on error [CITED: zip2 examples/write_dir.rs SECURITY NOTE].  
**Warning signs:** Zero-byte or unreadable zip after cancelled/failed export.

### Pitfall 4: Stub written to wiki_root
**What goes wrong:** Next compile/scan sees `.obsidian`; disk polluted.  
**Why it happens:** “Create stub then zip folder” shortcut.  
**How to avoid:** `start_file(".obsidian/app.json", …)` with in-memory bytes only (D-14).  
**Warning signs:** `{app_data}/wiki/.obsidian` appears after export.

### Pitfall 5: Treating root `index.md` as non-empty
**What goes wrong:** Export of empty vault with only scan-built index.  
**Why it happens:** `read_dir(wiki_root)` finds `index.md`.  
**How to avoid:** D-10 / `any_content_md` section dirs only.  
**Warning signs:** Preflight allows export when Library has no wiki pages.

### Pitfall 6: Success message via red error bar only
**What goes wrong:** Success looks like failure.  
**Why it happens:** AppShell alert is red (`border-red-400/30`) [VERIFIED: AppShell.tsx].  
**How to avoid:** Discretion — prefer soft/non-error notice slot (`data-testid="wiki-export-done"`) or dual-tone banner; keep failures on existing `err` bar (D-07).  
**Warning signs:** Users dismiss “success” as an error.

### Pitfall 7: Forgetting backend gate when UI hidden
**What goes wrong:** Direct IPC export while disabled.  
**Why it happens:** UI-only gate.  
**How to avoid:** Return `InsightsError::WikiDisabled` like compile [VERIFIED: wiki.rs compile path].

## Code Examples

### Zip write (Deflated, relative name) — zip 7.2

```rust
// Source: https://docs.rs/zip/7.2.0/zip/write/struct.ZipWriter.html [CITED]
use std::io::{Write, Cursor};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

let mut cur = Cursor::new(Vec::new());
let mut zip = ZipWriter::new(&mut cur);
let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated);
zip.start_file("sources/demo.md", options)?;
zip.write_all(b"# Demo\n")?;
// stub inject-only:
zip.start_file(".obsidian/app.json", options)?;
zip.write_all(br#"{"legacyEditor":false}"#)?;
zip.finish()?;
```

### Unit-test roundtrip listing

```rust
// Source: docs.rs ZipWriter::finish_into_readable pattern [CITED]
// After export_wiki_zip(wiki_root, &dest):
let f = std::fs::File::open(&dest)?;
let mut archive = zip::ZipArchive::new(f)?;
let names: Vec<_> = (0..archive.len()).map(|i| archive.by_index(i).unwrap().name().to_string()).collect();
assert!(names.iter().any(|n| n == "index.md" || n.ends_with(".md")));
assert!(names.iter().any(|n| n == ".obsidian/app.json"));
assert!(names.iter().all(|n| !n.contains("..") && !Path::new(n).is_absolute()));
assert!(names.iter().filter(|n| n.starts_with(".obsidian/")).all(|n| n == ".obsidian/app.json"));
```

### Tauri Save dialog (FE)

```typescript
// Source: @tauri-apps/plugin-dialog dist-js typedefs [VERIFIED: node_modules]
import { save } from "@tauri-apps/plugin-dialog";

const date = new Date().toISOString().slice(0, 10); // YYYY-MM-DD
const destPath = await save({
  defaultPath: `jarvis-wiki-${date}.zip`,
  filters: [{ name: "Zip", extensions: ["zip"] }],
});
if (destPath === null) return; // D-05
await exportWikiZip(destPath);
```

### Recommended crate API surface

```rust
/// True if any .md exists under sources|entities|concepts (D-10).
pub fn wiki_has_exportable_notes(wiki_root: &Path) -> bool;

/// Pack wiki_root → dest_zip; inject stub; skip on-disk .obsidian; reject empty / unsafe paths.
pub fn export_wiki_zip(wiki_root: &Path, dest_zip: &Path) -> Result<()>;
```

### Discretion recommendations (planner may adopt)

| Topic | Recommendation |
|-------|----------------|
| Preflight IPC name | `wiki_export_preflight_cmd` → serde `{ "hasNotes": bool }` (camelCase) |
| Empty hard-reject error | `InsightsError::WikiEmpty` with Display usable by FE if preflight skipped |
| Success UI slot | Soft notice (emerald/cyan), `data-testid="wiki-export-done"`, text includes filename/path — **not** red err styling |
| Empty UI | `setErr("还没有可导出的笔记，请先生成笔记")` exact string (D-09) |
| zip pin | Workspace `zip = { version = "7.2", default-features = false, features = ["deflate"] }` |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Plan sketch `zip = "2"` | Prefer `7.2` under MSRV 1.85 | zip 7/8 lines 2025–2026 | Newer writer API (`SimpleFileOptions`); still Deflated |
| Absolute latest `8.6` | Blocked by MSRV 1.88 | 2026-02+ zip 8 | Do not adopt without workspace MSRV bump |
| Full Obsidian config tree | Minimal `app.json` stub only | Locked D-13 | Sufficient for “open folder as vault”; Obsidian regenerates rest [CITED: obsidian.md/help/data-storage] |

**Deprecated/outdated:**
- Assuming `cargo search` “latest” without checking pre-release / MSRV — `9.0.0-pre2` and `8.6.0` are traps for this repo.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Minimal `.obsidian/app.json` alone is enough for Obsidian to treat unzipped folder as a vault (other configs auto-created on open) | Stub sufficiency | User may need one extra stub file — still locked to D-13; verify manually in Phase 13 E2E/UAT if needed |
| A2 | ISO date `toISOString().slice(0,10)` is acceptable for D-06 local calendar day (UTC vs local) | FE default filename | Filename off-by-one near midnight UTC — prefer local `YYYY-MM-DD` if planner wants strict local date |

**If empty:** Not empty — A1/A2 need awareness, not blocking.

## Open Questions (RESOLVED)

1. **Success notice ownership** — **RESOLVED** (12-02-PLAN)
   - What we know: AppShell only has red `err` alert today [VERIFIED].
   - Decision (locked in Plan 02): App-level **soft** non-red notice (`data-testid="wiki-export-done"`, cyan/emerald), dismissible like err; failures stay on shared red `err` bar. Empty copy still uses `reportError` / `setErr` (D-07 Discretion + D-09).

2. **Timezone for default zip filename** — **RESOLVED** (12-02-PLAN)
   - What we know: D-06 wants `jarvis-wiki-YYYY-MM-DD.zip`.
   - Decision (locked in Plan 02): **Local calendar** `YYYY-MM-DD` (not UTC-only `toISOString().slice(0,10)` near midnight) → `jarvis-wiki-${date}.zip` (D-06; RESEARCH A2).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | `zip` build + unit tests | ✓ | rustc/cargo 1.97.0 | — |
| Node / npm | Vitest | ✓ | Node 24.5 / npm 12 | — |
| `zip` crate | Export | ✗ (not in workspace yet) | add 7.2 | — |
| Obsidian app | Manual open-vault check | optional | — | Phase 13 / UAT; not required for unit/Vitest |
| `@tauri-apps/plugin-dialog` | Save UX | ✓ | ^2.7.1 | — |

**Missing dependencies with no fallback:** none blocking research (add `zip` in Wave 0 of plan).  
**Missing dependencies with fallback:** Obsidian GUI (manual) — automated zip content asserts suffice for Phase 12.

Step 2.6: External tooling beyond cargo/npm not required for implementation.

## Validation Architecture

> `workflow.nyquist_validation` is **true** in `.planning/config.json` — section required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (insights) + Vitest **^3.2.4** + Testing Library |
| Config file | workspace Cargo / `vite.config.ts` (`test.include`: `src/**/*.test.ts(x)`) |
| Quick run command | `cargo test -p insights export_wiki -- --nocapture` && `npx vitest run src/views/LibraryView.test.tsx src/hooks/useLibrary.test.ts` |
| Full suite command | `cargo test -p insights` && `npm test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIKI-07 | Zip contains wiki md paths + `.obsidian/app.json` stub | unit | `cargo test -p insights export_wiki_zip_includes_stub_and_pages` | ❌ Wave 0 |
| WIKI-07 | Entries relative + path-safe (no `..` / absolute) | unit | `cargo test -p insights export_wiki_zip_rejects_unsafe_paths` | ❌ Wave 0 |
| WIKI-07 | On-disk `.obsidian` skipped; injected stub wins | unit | `cargo test -p insights export_skips_ondisk_obsidian_injects_stub` | ❌ Wave 0 |
| WIKI-07 | Empty tree (index-only) hard-rejected | unit | `cargo test -p insights export_rejects_empty_wiki` | ❌ Wave 0 |
| WIKI-07 | `wiki.enabled=false` → error / no zip | unit (+ cmd) | `cargo test -p insights export_rejects_when_disabled` | ❌ Wave 0 |
| WIKI-07 | Stub not written under wiki_root | unit | assert `!wiki_root.join(".obsidian").exists()` after export | ❌ Wave 0 |
| WIKI-07 | Toolbar shows「导出 Wiki」only when enabled; after file-index | Vitest | `npx vitest run src/views/LibraryView.test.tsx` | ❌ extend existing |
| WIKI-07 | Preflight empty → no `save`; cancel → no export invoke | Vitest | `npx vitest run src/hooks/useLibrary.test.ts` (mock dialog) | ❌ Wave 0 |
| WIKI-09 | Full E2E export journey | e2e | deferred | ❌ Phase 13 (locked) |

### Sampling Rate

- **Per task commit:** targeted `cargo test -p insights <export_filter>` and/or focused Vitest file
- **Per wave merge:** `cargo test -p insights` + `npm test`
- **Phase gate:** above green; **do not** require `npm run test:e2e:local` for Phase 12 (deferred to 13 per CONTEXT) — document exception vs `.cursor/rules/e2e-required.mdc`

### Wave 0 Gaps

- [ ] `crates/insights` unit module/tests for `export_wiki_zip` / path safety / empty / stub inject / no disk stub
- [ ] Promote or duplicate `any_content_md` → `wiki_has_exportable_notes`
- [ ] Add workspace + insights `zip` dependency
- [ ] Extend `LibraryView.test.tsx` for `wiki-export` visibility/placement
- [ ] Extend `useLibrary.test.ts` (or App handler tests) for preflight → save → export / cancel / empty copy
- [ ] Register Tauri commands + `src/lib/tauri.ts` wrappers (implementation wave; tests mock invoke)

*(E2E `e2e/specs/wiki.spec.ts` is Phase 13 — not a Phase 12 Wave 0 gap.)*

## Security Domain

> `security_enforcement` not set to `false` in config — included.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop app; no new auth |
| V3 Session Management | no | — |
| V4 Access Control | partial | Export gated on `wiki.enabled`; dest path chosen by user via OS dialog |
| V5 Input Validation | **yes** | Path-safe zip entry construction; reject `..` / absolute; skip packing foreign `.obsidian` |
| V6 Cryptography | no | Deflate only; no custom crypto — use `zip` crate |

### Known Threat Patterns for zip export

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Zip Slip (malicious entry names) | Tampering / Elevation | Component allowlist + relative-only names; unit tests |
| Path traversal via symlink follow | Tampering | Pack **regular files only**; do not follow symlinks out of `wiki_root` [ASSUMED: follow `symlink_metadata`/`is_file` without follow] |
| Overwriting unexpected dest | Tampering | User-chosen Save path; optional refuse if exists or overwrite explicitly (OS dialog typically confirms) |
| Leaking secrets / build artifacts | Information Disclosure | D-16: no `target/`; wiki tree only; skip `.obsidian` on disk |
| Partial zip left behind | Tampering / Denial | Temp file + rename; delete on failure |

## Project Constraints (from .cursor/rules/)

| Rule | Directive relevant to Phase 12 |
|------|--------------------------------|
| `tdd-goal-driven.mdc` | Goal → failing tests → implement; unit for path safety; inject traits where needed; done checklist |
| `karpathy-guidelines.mdc` | Simplicity; surgical diffs; no speculative features; state assumptions |
| `jarvis-stack.mdc` | Rust stable / MSRV **1.85**; no pre-release deps; core logic in crates; thin Tauri commands; sole SQLite owner = `store` |
| `e2e-required.mdc` | User-facing features need E2E — **Phase 12 CONTEXT explicitly defers E2E to Phase 13**; Phase 12 ships unit + Vitest; planner must track WIKI-09 for E2E |
| `frontend-taste.mdc` | Dark tech + liquid glass; loading/empty/error states; match existing Library toolbar `btn-ghost` patterns (no redesign) |

## Sources

### Primary (HIGH confidence)
- crates.io API + `cargo info zip@7.2.0` / `@8.6.0` / `@2.4.2` — versions + MSRV [VERIFIED]
- `gsd-tools query package-legitimacy check --ecosystem crates zip` → OK [VERIFIED]
- Codebase: `crates/insights/src/wiki.rs`, `error.rs`, `src-tauri/src/commands/wiki.rs`, `LibraryView.tsx`, `useLibrary.ts`, `App.tsx`, `AppShell.tsx`, capabilities [VERIFIED]
- `@tauri-apps/plugin-dialog` `save` / `SaveDialogOptions` typedefs [VERIFIED]
- docs.rs `zip` 7.2.0 `ZipWriter` [CITED: https://docs.rs/zip/7.2.0/zip/write/struct.ZipWriter.html]
- zip2 `examples/write_dir.rs` [CITED: https://github.com/zip-rs/zip2/blob/master/examples/write_dir.rs]

### Secondary (MEDIUM confidence)
- Obsidian Help — vault = folder; `.obsidian` holds preferences [CITED: https://obsidian.md/help/data-storage]
- Web community notes that Obsidian regenerates missing config files on open [ASSUMED/MEDIUM] — stub shape locked by D-13 regardless

### Tertiary (LOW confidence)
- Exact Obsidian UX for “Open folder as vault” with only `app.json` — confirm in Phase 13 / manual UAT [ASSUMED]

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — crates.io + legitimacy + MSRV cross-check
- Architecture: **HIGH** — maps onto existing wiki compile IPC / Library patterns
- Pitfalls: **HIGH** — Zip Slip / Windows separators / stub-on-disk are well-known; empty semantics verified in-repo

**Research date:** 2026-07-23  
**Valid until:** ~2026-08-22 (30 days; re-check `zip` latest vs MSRV if planning delayed)
