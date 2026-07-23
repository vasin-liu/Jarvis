# Phase 12: Obsidian zip export - Context

**Gathered:** 2026-07-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver WIKI-07: user can export the on-disk wiki tree as a path-safe `.zip` that opens as an Obsidian vault (Markdown tree + minimal `.obsidian` stub), gated on `wiki.enabled`. Phase includes unit tests for path safety / zip contents, Vitest for Library export control wiring, and thin Tauri IPC. Full E2E journey (`wiki.spec.ts`) remains Phase 13. No bidirectional Obsidian sync.

</domain>

<decisions>
## Implementation Decisions

### Export button placement
- **D-01:** Export control lives **only on the Library toolbar** (not Settings). Place it **after**「选择文件索引」.
- **D-02:** Button label **「导出 Wiki」** with `data-testid="wiki-export"`.
- **D-03:** When `wiki.enabled !== true`, **do not render** the export button (same hide pattern as「生成笔记」).

### Save destination UX
- **D-04:** Use the system **Save** dialog (`@tauri-apps/plugin-dialog` `save`). User picks destination; FE passes `destPath` into export IPC.
- **D-05:** If the user **cancels** the dialog: **no side effects** — do not invoke export, do not write any file.
- **D-06:** Default suggested filename: **`jarvis-wiki-YYYY-MM-DD.zip`** with a `.zip` file filter.
- **D-07:** On success, show a **brief success message** including path or filename. On failure, use existing Library/App **`reportError`** / shared error bar (no per-button inline error UI).
- **D-08:** While export runs, use the existing App-level **`busy`** flag (disable Library toolbar actions including export — same as compile/index).

### Empty / missing wiki behavior
- **D-09:** If the wiki tree is empty, **block before** opening the Save dialog and show: **「还没有可导出的笔记，请先生成笔记」**.
- **D-10:** Empty means: **no `.md` files under `sources/`, `entities/`, or `concepts/`** (root-only `index.md` or missing dirs count as empty — aligned with Phase 10 scan semantics).
- **D-11:** Add a **lightweight preflight IPC** (disk-based) that FE calls on click; empty → message + no dialog; non-empty → Save dialog → export.
- **D-12:** `export_wiki_zip` must **hard-reject** empty trees as defense-in-depth (even after a successful preflight).

### `.obsidian` stub & zip payload
- **D-13:** Stub is **minimal**: only `.obsidian/app.json` with `{"legacyEditor":false}`.
- **D-14:** Inject the stub **into the zip only** — do **not** write `.obsidian/` into `{app_data}/wiki/` on disk.
- **D-15:** If `.obsidian/` already exists on disk under `wiki_root`, **skip packing it**; the zip **always** uses Jarvis’s minimal stub (deterministic export).
- **D-16:** Zip includes **all regular files** under `wiki_root` as **relative, path-safe** entries (reject `..` / absolute paths) **plus** the injected stub. Skip on-disk `.obsidian/` per D-15. No secrets / no `target/`.

### Claude's Discretion
- Preflight command naming (`wiki_export_preflight` vs similar) and exact success-message UI slot (reuse error bar styling vs soft status text) left to planner — must satisfy D-07/D-09/D-11.
- Add workspace `zip` crate (plan suggests `zip = "2"`) if still absent — researcher confirms latest stable.
- Whether preflight returns a boolean vs a small status struct — planner chooses; FE needs a clear empty/ok signal.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — **WIKI-07** (Pending → Phase 12): Obsidian-compatible zip with minimal `.obsidian/` stub
- `.planning/ROADMAP.md` — Phase 12 success criteria (zip contents, path-safe entries, `wiki.enabled` gate)
- `.planning/PROJECT.md` — export-only (no bidirectional Obsidian sync); wiki layer beside RAG

### Prior phase decisions
- `.planning/phases/11-library-settings-ui/11-CONTEXT.md` — deferred export UI/IPC to Phase 12; hide-when-disabled pattern; Settings「Wiki 笔记」toggle only `wiki.enabled`
- `.planning/phases/10-persist-index/10-CONTEXT.md` — `compile_wiki_for_source`, scan-rebuild `index.md`, sections `sources|entities|concepts`, wiki root under app data

### Design / implementation sketch
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — **Task 6** Obsidian zip export (`export_wiki_zip`, stub shape, `wiki-export` testid, dialog save); E2E Task is Phase 13

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/insights/src/wiki.rs` — wiki_root write/scan; `resolve_wiki_page_path` path-safety mindset to reuse for zip entry names
- `src-tauri/src/commands/wiki.rs` — thin `compile_wiki_cmd` pattern for new export + preflight commands
- `src/views/LibraryView.tsx` — toolbar (`一键洞察` / `选择文件索引`); insert「导出 Wiki」after file-index button
- `src/hooks/useLibrary.ts` + `src/lib/tauri.ts` — invoke + `reportError` + refresh patterns; App `busy` wrapper
- `@tauri-apps/plugin-dialog` — already used for `open({ directory })`; add `save` for dest path (`dialog:default` capability present)

### Established Patterns
- Gate UI with `config?.wiki?.enabled === true` (hide, don’t disable)
- Backend hard-reject when `wiki.enabled=false` (mirror compile)
- wiki_root = sibling of `config.json` under app data (`…/wiki`)
- Phase 12 verification: unit + Vitest; E2E deferred to Phase 13

### Integration Points
- New `export_wiki_zip` (+ preflight) in `insights`, thin Tauri cmds beside `compile_wiki_cmd`
- FE: Library toolbar button → preflight → `save` → export invoke → success message / `reportError`
- No DB writes for export (FS-only zip); `store` remains sole SQLite owner

</code_context>

<specifics>
## Specific Ideas

- Default zip name pattern: `jarvis-wiki-YYYY-MM-DD.zip`
- Empty-state copy exactly: 「还没有可导出的笔记，请先生成笔记」
- Stub JSON exactly: `{"legacyEditor":false}` in `.obsidian/app.json`
- Success feedback should include path or filename (not silent like「生成笔记」)

</specifics>

<deferred>
## Deferred Ideas

- Full `wiki.spec.ts` enable → compile → list → export journey — **Phase 13** (WIKI-09)
- Citation trust / RAG still cites originals — **Phase 13** (WIKI-08)
- Bidirectional Obsidian sync — out of product scope
- Settings duplicate export button — explicitly rejected (Library only)

</deferred>

---

*Phase: 12-Obsidian zip export*
*Context gathered: 2026-07-23*
