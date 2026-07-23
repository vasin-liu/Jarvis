# Roadmap: Jarvis

**Updated:** 2026-07-17  
**Current milestone:** v1.10 Wiki Compile Layer

## Shipped

| Milestone | Summary | Archive |
|-----------|---------|---------|
| **v1.9** Structural Refactor | 6 phases / 22 plans — FE extraction, shell split, keychain, memory://, JSON agent protocol, Settings + arch review | [v1.9-ROADMAP.md](./milestones/v1.9-ROADMAP.md) |

## v1.10 Wiki Compile Layer

**Goal:** Add an optional, rebuildable Markdown wiki layer beside RAG — with Obsidian zip export — without replacing hybrid retrieval or citations.

**Research:** `.planning/research/SUMMARY.md`  
**Plan draft:** `docs/superpowers/plans/2026-07-16-wiki-compile-layer.md`

| # | Phase | Goal | Requirements | Success criteria |
|---|-------|------|--------------|------------------|
| 07 | Wiki kind + config | 3/3 | Complete    | 2026-07-18 |
| 08 | Markdown renderer | 2/2 | Complete    | 2026-07-19 |
| 09 | LLM wiki analysis | 2/2 | Complete    | 2026-07-19 |
| 10 | Persist + index | 2/2 | Complete    | 2026-07-21 |
| 11 | Library / Settings UI | 2/2 | Complete    | 2026-07-23 |
| 12 | Obsidian zip export | 1/2 | In Progress|  |
| 13 | E2E + citation trust | User journey + RAG regression gate | WIKI-08, WIKI-09 | 4 |

---

### Phase 07: Wiki kind + config

**Goal:** Users can turn wiki on/off in config with no surprise behavior on upgrade; wiki pages have a first-class source kind and Library label.

**Requirements:** WIKI-01, WIKI-02

**Success criteria:**

1. Loading a pre-v1.10 `config.json` yields `wiki.enabled == false` and `wiki.auto_on_insights == false`
2. `SourceKind::WikiPage` round-trips as `"wiki_page"` in store tests
3. Frontend `sourceDisplay` maps `wiki_page` to a visible Wiki / 笔记页 label (Vitest)

**Plans:** 3/3 plans complete

Plans:

- [x] 07-01-PLAN.md — SourceKind::WikiPage + fail-closed reindex stub
- [x] 07-02-PLAN.md — Nested WikiConfig default-off + config round-trip tests
- [x] 07-03-PLAN.md — Library label「笔记页」+ FE wiki config pass-through

---

### Phase 08: Deterministic Markdown renderer

**Goal:** Given a `WikiAnalysis`, the system produces stable Markdown pages (frontmatter, wikilinks, index) without LLM or disk I/O.

**Requirements:** WIKI-03 (render half)

**Success criteria:**

1. Empty entities/concepts still yields a source-summary page with `sources:` frontmatter
2. Non-empty entities emit `entities/` pages and `[[wikilink]]` from the summary page
3. CJK / unsafe names produce filesystem-safe slugs (ASCII or `e-{hash6}` fallback) in unit tests

**Plans:** 2/2 plans complete

Plans:
**Wave 1**

- [x] 08-01-PLAN.md — Wiki types, slugify/collision, frontmatter/wikilink helpers (Wave 1)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 08-02-PLAN.md — render_wiki_pages + index_markdown + ROADMAP render tests (Wave 2)

---

### Phase 09: LLM wiki analysis

**Goal:** User's indexed source can be analyzed into `WikiAnalysis` via ChatModel; bad model output never starts a write.

**Requirements:** WIKI-03 (analyze half), WIKI-05

**Success criteria:**

1. MockChatModel returning valid JSON yields populated summary/entities/concepts
2. Invalid / non-JSON model output returns a typed parse error
3. Parse failure path writes **zero** files under a tempfile wiki root in tests

**Plans:** 2/2 plans complete

Plans:

**Wave 1**

- [x] 09-01-PLAN.md — InvalidWikiJson + Mock 「笔记编译」 + analyze_source_for_wiki happy path (WIKI-03)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 09-02-PLAN.md — Parse strictness + write_wiki_pages_to_dir + D-14 fail-closed zero-file tests (WIKI-05)

---

### Phase 10: Persist + index

**Goal:** Compiled pages exist on disk under `{app_data}/wiki/` and appear as indexed `WikiPage` sources without duplicates on re-compile.

**Requirements:** WIKI-04

**Success criteria:**

1. Successful compile creates Markdown files and `index.md` under the wiki root
2. Pages are indexed with `wiki://{slug}` URIs and `SourceKind::WikiPage`
3. Compiling twice with identical content does not duplicate sources (hash skip)
4. `WikiPage` sources are not accepted as compile inputs (no compile loop)

**Plans:** 2/2 plans complete

Plans:

**Wave 1**

- [x] 10-01-PLAN.md — Wave 0 scaffolds + compile_wiki_for_source (gates/write/scan-rebuild/index/hash skip) + thin compile_wiki_cmd (WIKI-04)

**Wave 2** *(blocked on Wave 1)*

- [x] 10-02-PLAN.md — rebuild_index_md unit coverage + stale cleanup D-06…D-08 + user-edit D-09/D-12 (WIKI-04)

---

### Phase 11: Library / Settings UI

**Goal:** When enabled, user can compile from Library and toggle wiki in Settings; when disabled, those controls are absent.

**Requirements:** WIKI-06

**Success criteria:**

1. Settings exposes wiki enabled toggle with stable `data-testid`
2. With wiki enabled, Library shows “生成笔记” (or equivalent) for an indexed source
3. With wiki disabled, compile/export controls are not shown (default E2E config)

**Plans:** 2/2 plans complete

Plans:

**Wave 1**

- [x] 11-01-PLAN.md — Settings「Wiki 笔记」AccordionSection + wiki.enabled toggle + Vitest

**Wave 2** *(blocked on Wave 1)*

- [x] 11-02-PLAN.md — tauri/useLibrary/App compile wiring + Library「生成笔记」gates + Vitest

---

### Phase 12: Obsidian zip export

**Goal:** User can download/export a zip that opens as an Obsidian vault (wiki tree + minimal `.obsidian` stub).

**Requirements:** WIKI-07

**Success criteria:**

1. Export produces a `.zip` containing wiki Markdown paths and `.obsidian/` stub
2. Zip entries are relative and path-safe (no `..` / absolute paths) in unit tests
3. Export is gated on `wiki.enabled` (errors or no-ops when disabled)

**Plans:** 1/2 plans executed

Plans:

**Wave 1**

- [x] 12-01-PLAN.md — Rust `export_wiki_zip` + path-safe packing + zip 7.2 + unit tests (WIKI-07)

**Wave 2** *(blocked on Wave 1)*

- [ ] 12-02-PLAN.md — Tauri preflight/export IPC + Library「导出 Wiki」+ Save dialog + Vitest (WIKI-07)

---

### Phase 13: E2E + citation trust

**Goal:** Full mocked journey proves wiki works; RAG still cites original fixture sources when relevant.

**Requirements:** WIKI-08, WIKI-09

**Success criteria:**

1. `e2e/specs/wiki.spec.ts` passes: enable → compile → see wiki page → export
2. Default-off `full-ui` (or equivalent) does not show wiki compile controls
3. QA / citation assertions still resolve to original fixture sources when wiki is off
4. With wiki on, answers remain usable and wiki does not solely replace original citations in the asserted journey

**Plans:** TBD via `/gsd-plan-phase 13`

---

## Coverage

| Requirement | Phase | Status |
|-------------|-------|--------|
| WIKI-01 | 07 | Complete |
| WIKI-02 | 07 | Complete |
| WIKI-03 | 08 + 09 | Complete |
| WIKI-04 | 10 | Complete |
| WIKI-05 | 09 | Complete |
| WIKI-06 | 11 | Complete |
| WIKI-07 | 12 | Pending |
| WIKI-08 | 13 | Pending |
| WIKI-09 | 13 | Pending |

**v1 requirements:** 9 · **Mapped:** 9 · **Unmapped:** 0 ✓

*Note:* WIKI-03 spans Phases 08 (render) and 09 (analyze) by design — one requirement, two delivery boundaries.

## Backlog (post-v1.10)

- Related-docs / MCP read-only
- Bulk compile + `auto_on_insights` UX
- Cross-corpus entity merge

---

*Prior detailed v1.9 phase text:* `.planning/milestones/v1.9-ROADMAP.md`  
*v1.9 phase artifacts:* `.planning/milestones/v1.9-phases/`
