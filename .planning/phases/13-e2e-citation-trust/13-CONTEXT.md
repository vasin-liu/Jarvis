# Phase 13: E2E + citation trust - Context

**Gathered:** 2026-07-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver WIKI-08 and WIKI-09: a mocked E2E journey proves wiki enable → compile → list「笔记页」→ export; default-off does not expose wiki Library controls in `full-ui`; after wiki is on and compiled, RAG Q&A still cites at least one non-`wiki://` original fixture source. No new product features beyond test harness hooks / small UI testability attrs needed for those assertions.

</domain>

<decisions>
## Implementation Decisions

### Wiki E2E journey (`wiki.spec.ts`)
- **D-01:** Cover **happy-path + default-off**: assert wiki compile/export controls absent when disabled, then enable and run compile → list → export.
- **D-02:** After compile, identify success by **at least one Library row labeled「笔记页」**.
- **D-03:** **Hybrid enable:** assert default-off first → Settings UI toggle「启用 Wiki 笔记层」→「保存配置」→ Library compile/export (do not seed `wiki.enabled=true` at process start for the positive path).
- **D-04:** Compile the **first eligible** source: first non-`wiki_page` indexed row that exposes a `wiki-compile-{id}` button (do not hard-bind fixture display name).

### Export under WebDriver
- **D-05:** In E2E, **bypass the native Save dialog**. Prefer a FE hook `__JARVIS_E2E_WIKI_EXPORT_PATH__`: when set, `exportWiki` skips `save()` and calls real `export_wiki_zip(destPath)`.
- **D-06:** Assert **success notice** (`wiki-export-done` or equivalent) **and** that the zip file **exists with size > 0**. Do **not** require unzipping / asserting `index.md` / `.obsidian` contents in E2E (Phase 12 unit coverage owns stub/path safety).
- **D-07:** Fixed dest path: **OS temp + `jarvis-e2e-wiki.zip`** (e.g. `%TEMP%/jarvis-e2e-wiki.zip` on Windows). Spec may unlink beforehand to avoid stale size assertions.

### Citation trust (WIKI-08)
- **D-08:** Prove citation trust **primarily in E2E** inside `wiki.spec` after wiki is enabled and compiled. Do **not** require a new Rust RAG integration test for this phase (planner may add one optionally; not locked).
- **D-09:** Add **`data-source-uri={c.source_uri}`** (or equivalent) on citation buttons in Chat so E2E can read URIs.
- **D-10:** After asking **`What is xyzzy-plugh?`**, require **at least one citation whose `data-source-uri` does not start with `wiki://`**. Wiki citations may appear alongside; forbid only “all citations are wiki” / sole replacement.
- **D-11:** Leave **`qa.spec.ts` unchanged** (default-off Mock answer + `citation-excerpt` regression stays as-is).

### Regression placement & docs
- **D-12:** **`wiki.spec`** owns the full positive journey + its opening default-off assert. **`full-ui.spec.ts`** gets **light default-off only** (e.g. Library has no `wiki-export` / no `wiki-compile-*`; no positive enable/compile/export in `full-ui`).
- **D-13:** Update **both** `.cursor/rules/e2e-required.mdc` and `e2e/README.md` spec maps with **Wiki → `wiki.spec.ts`**.

### Claude's Discretion
- Exact helper placement for setting `__JARVIS_E2E_WIKI_EXPORT_PATH__` / reading zip size in `e2e/helpers.ts`.
- Whether Settings default-off assert checks unchecked `wiki-enabled-toggle` vs absence of Library controls only (Library absence is mandatory per D-12).
- Mock chat / `e2e.rs` tweaks if wiki compile JSON collides with other Mock branches — keep deterministic; extend Mock only as needed.
- Optional Vitest for `data-source-uri` rendering — not required if E2E covers it.
- Optional Rust citation integration with wiki indexed — not required (D-08).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — **WIKI-08** (citations additive), **WIKI-09** (E2E enable → compile → list → export; default-off `full-ui`)
- `.planning/ROADMAP.md` — Phase 13 goal + success criteria
- `.planning/PROJECT.md` — Core Value / wiki beside RAG, not replacing citations
- `.planning/STATE.md` — current milestone position

### Prior phase decisions
- `.planning/phases/12-obsidian-zip-export/12-CONTEXT.md` — export button/testids, Save dialog UX, empty preflight, success notice; E2E deferred here
- `.planning/phases/11-library-settings-ui/11-CONTEXT.md` — Settings toggle,「生成笔记」, hide-when-disabled, testids
- `.planning/phases/10-persist-index/10-CONTEXT.md` — `compile_wiki_for_source`, `wiki://` URIs, WikiPage indexing

### Implementation sketch
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — **Task 7** E2E journey + validation docs (apply discussion overrides above)

### E2E / product UI
- `e2e/specs/qa.spec.ts` — existing citation/fixture patterns (leave alone per D-11)
- `e2e/fixtures/sample.md` — `xyzzy-plugh` keyword
- `e2e/helpers.ts` — React-safe input / existing `__JARVIS_E2E_*` hooks
- `.cursor/rules/e2e-required.mdc` — spec map (must add Wiki row)
- `e2e/README.md` — spec map (must add Wiki row)
- `src/views/ChatView.tsx` — citation UI (`citation-excerpt`; add `data-source-uri`)
- `src/hooks/useLibrary.ts` — `exportWiki` + `save()` (E2E path bypass)
- `src/views/LibraryView.tsx` / `src/views/SettingsView.tsx` — existing wiki testids

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Testids: `wiki-enabled-toggle`, `settings-section-wiki`, `wiki-compile-{id}`, `wiki-export`, `wiki-export-done`
- `askQuestion` / `openNav` / `setReactCheckbox` patterns in `e2e/helpers.ts`
- Window E2E hooks precedent: `__JARVIS_E2E_ASK__`, `__JARVIS_E2E_SET_ORCHESTRATION__`, `__JARVIS_E2E_REFRESH_CONFIG__`
- `MockChatModel` already returns wiki compile JSON when system contains「笔记编译」
- Fixture seeded via `JARVIS_E2E_FIXTURE` → `e2e/fixtures/sample.md`

### Established Patterns
- Default-off wiki: hide controls (not disable) when `wiki.enabled !== true`
- E2E: `JARVIS_E2E=1`, Mock providers, single WDIO instance
- Citation payload has `source_uri` but UI currently does not expose it for selectors
- Export: preflight → Save dialog → `export_wiki_zip` — dialog is the E2E pain point

### Integration Points
- New `e2e/specs/wiki.spec.ts` picked up automatically by CI `e2e` job (`e2e/specs/**/*.spec.ts`)
- FE: wire export path hook in `useLibrary.exportWiki`; citation attr in `ChatView`
- Docs: `.cursor/rules/e2e-required.mdc` + `e2e/README.md`
- Light asserts in `e2e/specs/full-ui.spec.ts` (Library/Settings default-off only)

</code_context>

<specifics>
## Specific Ideas

- Export hook name: `__JARVIS_E2E_WIKI_EXPORT_PATH__`
- Zip path filename: `jarvis-e2e-wiki.zip` under OS temp
- Citation trust question exactly: `What is xyzzy-plugh?`
- Library success signal: visible「笔记页」label (Phase 07 `sourceKindLabel`)
- Empty-export E2E path explicitly **not** required (Phase 12 unit/Vitest already cover)

</specifics>

<deferred>
## Deferred Ideas

- Empty-export E2E assertion (「还没有可导出的笔记…」) — covered by Phase 12 unit/Vitest; optional later
- Unzip / Obsidian stub contents in E2E — Phase 12 unit ownership
- Mandatory Rust RAG+wiki citation integration test — optional, not locked
- Bidirectional Obsidian sync / bulk compile / `auto_on_insights` UX — post-v1.10 backlog

None else — discussion stayed within phase scope

</deferred>

---

*Phase: 13-E2E + citation trust*
*Context gathered: 2026-07-24*
