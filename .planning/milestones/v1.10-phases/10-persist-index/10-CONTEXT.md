# Phase 10: Persist + index - Context

**Gathered:** 2026-07-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver end-to-end wiki persist + index: after successful analyze→render, write Markdown under `{app_data}/wiki/`, rebuild `index.md`, and upsert pages as `SourceKind::WikiPage` via the existing ingest→`index_document` path with stable `wiki://{slug}` URIs and `content_hash` idempotency. Expose a library `compile_wiki_for_source` plus a thin Tauri command (no UI). Reject `WikiPage` compile inputs and hard-gate on `wiki.enabled`.

**Requirements:** WIKI-04  
**Success criteria (ROADMAP):** files + `index.md` on disk; indexed `WikiPage` with `wiki://{slug}`; identical re-compile does not duplicate sources; `WikiPage` sources are not compile inputs.

**Out of scope this phase:** Library/Settings compile UX (Phase 11); Obsidian zip (Phase 12); E2E journey + citation trust (Phase 13); `auto_on_insights` wiring; cross-corpus entity merge; progress-event streaming for bulk compile.

</domain>

<decisions>
## Implementation Decisions

### index.md rebuild
- **D-01:** After each successful compile, **rebuild `index.md` by scanning the wiki tree** (do not surgically patch a prior catalog).
- **D-02:** Scan **only** `sources/`, `entities/`, and `concepts/` `*.md`. Ignore root clutter; never treat `index.md` as a catalog entry source.
- **D-03:** Catalog link display names come from frontmatter **`title`**, rendered as `[[path|title]]`. If `title` is missing, fall back to the file slug.
- **D-04:** Within each section, sort entries by **path/slug lexicographic order** (stable, independent of compile order).
- **D-05:** Omit empty sections (same shape as Phase 08 `index_markdown`: `# Wiki` + `## Sources` / `## Entities` / `## Concepts` only when non-empty).

### Stale page cleanup
- **D-06:** On re-compile of a source, **delete generated pages that disappeared** from this compile’s output for that source, and **remove the corresponding `wiki://` indexed sources**.
- **D-07:** A page is in scope for cleanup when its frontmatter **`sources` contains this source’s URI** and it was not produced by the current compile.
- **D-08:** **Never delete** pages that lack `generated: true` (user-curated / edited notes are retained even if still tagged with the source URI).
- **D-09:** If the compile wants to write a slug that already exists **without** `generated: true`, **skip writing** that file (keep the user version). Still proceed with other pages.

### content_hash
- **D-10:** `content_hash` lives **only on the Document / DB indexer path**. Do **not** inject `content_hash` into Markdown frontmatter (Phase 08 omission stays permanent for v1.10).
- **D-11:** Hash input is the **full on-disk Markdown body** as written (including existing YAML: `title`, `type`, `sources`, `generated`).
- **D-12:** When write is skipped due to a user-edited file (D-09), still **index from the existing on-disk file** via normal hash-skip so Library/RAG reflect the user’s note.
- **D-13:** Root **`index.md` is not indexed** as a `WikiPage`. Only pages under `sources/` · `entities/` · `concepts/` enter the index.

### Phase-10 API surface
- **D-14:** Ship **`compile_wiki_for_source` (library)** plus a **thin Tauri command** with **no UI** this phase (Library wiring is Phase 11). Prefer a dedicated command module (e.g. `commands/wiki.rs`) if that matches shell layout.
- **D-15:** When `wiki.enabled == false`, **hard-reject** compile (clear error); no disk writes, no index upserts.
- **D-16:** When the input source kind is **`WikiPage`**, **hard-reject** with a clear error (no analyze / write / index) — compile-loop guard.
- **D-17:** On success, the Tauri command returns a **compact summary**: pages written, created/updated/skipped(user-edit)/cleaned counts, and wiki root. **No** progress events in this phase.

### Claude's Discretion
- Exact Tauri command name and summary struct field names (as long as D-14…D-17 hold).
- Exact error variant/message strings for disabled wiki and WikiPage-input rejection.
- Whether cleanup+write+index run as one transactional-ish sequence vs best-effort with documented failure modes — prefer fail-closed where practical (align with Phase 09); planner/researcher decide feasibility against current indexer APIs.
- Test layout: integration tests with tempfile + MockChatModel + MockEmbedder (plan Task 4) vs additional unit tests for index rebuild/cleanup helpers.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / phase
- `.planning/ROADMAP.md` — Phase 10 goal, success criteria, WIKI-04
- `.planning/REQUIREMENTS.md` — WIKI-04
- `.planning/PROJECT.md` — v1.10 wiki goals; local-first; single DB owner
- `.planning/STATE.md` — current phase position

### Research
- `.planning/research/SUMMARY.md` — Phase 10 deliverables; `generated: true` overwrite; skip WikiPage inputs
- `.planning/research/FEATURES.md` — persist + index pipeline; index.md merge; hash skip
- `.planning/research/STACK.md` — Document→indexer path; no second vector store
- `.planning/research/PITFALLS.md` — dual-write / duplicate sources; compile loops; partial write

### Implementation plan
- `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md` — Task 4 (`compile_wiki_for_source`, write + index). Apply discussion overrides: scan-rebuild `index.md`; DB-only `content_hash`; stale cleanup rules; thin Tauri + summary return.

### Prior phases
- `.planning/phases/09-llm-wiki-analysis/09-CONTEXT.md` — analyze → write gate; fail-closed; `write_wiki_pages_to_dir`
- `.planning/phases/08-deterministic-markdown-renderer/08-CONTEXT.md` — tree layout, frontmatter (`generated: true`, no `content_hash` in render), index shape, wikilinks
- `.planning/phases/07-wiki-kind-config/07-CONTEXT.md` — `WikiPage` kind, `WikiConfig` defaults (no re-litigate)

### Code integration points
- `crates/insights/src/wiki.rs` — `render_wiki_pages`, `write_wiki_pages_to_dir`, analyze; extend with compile/persist/cleanup/index rebuild
- `crates/indexer` — `index_document` + existing content_hash skip
- `crates/ingest` — `Document` shape for wiki pages
- `crates/store` — `SourceKind::WikiPage`, source CRUD / delete for cleanup
- `src-tauri` — thin wiki compile command + `insights_ops` / command module glue
- `crates/config` — `WikiConfig.enabled` gate

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `write_wiki_pages_to_dir` — already writes sources/entities/concepts tree + `index.md` from one `WikiCompileResult` (Phase 09); Phase 10 must evolve write path for overwrite policy, cleanup, and scan-rebuild index
- `analyze_source_for_wiki` + Mock 「笔记编译」 — happy-path LLM for integration tests
- `indexer::index_document` — hash skip for idempotent re-compile
- `SourceKind::WikiPage` + `wiki_page` — ready since Phase 07
- Memory/`memory://` normalize-to-Document pattern — precedent for non-file URIs

### Established Patterns
- Insights: load Indexed source → LLM → parse → persist side effects
- Fail-closed: never write partial tree on analyze failure (Phase 09)
- Frontmatter already emits `generated: true`; unit tests currently assert `content_hash` omitted from render — keep that; hash only at index time
- Thin Tauri commands delegating to crates / `*_ops`

### Integration Points
- Compile entry: library API callable from Tauri command and future Library UI (Phase 11)
- Wiki root: `{app_data}/wiki/` (WIKI-04)
- URI scheme: `wiki://{slug}` where slug matches draft paths like `entities/acme`
- Phase 12 zip packages the same on-disk tree; Phase 13 E2E exercises enable → compile → list wiki page

</code_context>

<specifics>
## Specific Ideas

- Prefer whole-tree index rebuild over merge patches so multi-source vaults stay consistent with disk.
- User-edited notes (`generated` removed) are sacred: skip overwrite, never delete on stale cleanup, but still index from disk when skipped.
- Keep vault Markdown free of `content_hash` so Obsidian edits do not fight an embedded hash field.

</specifics>

<deferred>
## Deferred Ideas

- Library 「生成笔记」 + Settings wiki toggle UI — Phase 11
- Obsidian zip export — Phase 12
- E2E enable → compile → list → export + citation trust — Phase 13
- `auto_on_insights` wiring / bulk compile + progress events — Future (WIKI-F01/F02)
- Cross-corpus entity merge — Future (WIKI-F03)
- Progress-event streaming for compile — deferred (D-17 chose summary-only)

None — discussion stayed within phase scope (no folded todos)

</deferred>

---

*Phase: 10-Persist + index*
*Context gathered: 2026-07-21*
